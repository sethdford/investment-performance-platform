use lambda_http::{run, service_fn, Body, Error as LambdaError, Request, Response};
use lambda_runtime::Context;
use aws_sdk_dynamodb::Client as DynamoDbClient;
use serde_json::json;
use tracing::{info, error};
use shared::models::{Client, Portfolio, Account, Security, Transaction, PerformanceMetric};
use shared::repository::{Repository, DynamoDbRepository};
use async_graphql::{Schema, EmptySubscription, http::GraphiQLSource};
use http::StatusCode;
use std::sync::Arc;
use serde_json::{Map, Value};
use thiserror::Error;
use std::error::Error as StdError;

mod schema;
use schema::{Query, Mutation, InvestmentSchema};

pub type LambdaResult<T> = std::result::Result<T, LambdaError>;

async fn function_handler(event: Request) -> LambdaResult<Response<Body>> {
    info!("GraphQL API Lambda received request");
    
    // Initialize AWS clients
    let config = aws_config::from_env()
        .load()
        .await;
    
    let dynamodb_client = DynamoDbClient::new(&config);
    
    // Initialize repository
    let repository = Arc::new(DynamoDbRepository::new(dynamodb_client.clone(), "investment_performance".to_string()));
    
    // Initialize GraphQL schema
    let schema = InvestmentSchema::build(Query::default(), Mutation::default(), EmptySubscription)
        .data(repository)
        .finish();
    
    // Handle request based on path
    let path = event.uri().path();
    
    if path.starts_with("/graphql") {
        // Handle GraphQL API request
        let body = match event.body() {
            Body::Text(text) => text.clone(),
            Body::Binary(binary) => String::from_utf8_lossy(binary).to_string(),
            _ => "".to_string(),
        };
        
        let query = serde_json::from_str::<serde_json::Value>(&body)
            .map_err(|_| LambdaError::from("Invalid JSON body"))?;
        
        let operation_name = query.get("operationName")
            .and_then(|v| v.as_str())
            .map(String::from);
        
        let query_str = query.get("query")
            .and_then(|v| v.as_str())
            .ok_or_else(|| LambdaError::from("Missing query field"))?
            .to_string();
        
        // Convert variables to async_graphql::Variables
        let variables = match query.get("variables") {
            Some(vars) if !vars.is_null() => {
                // Convert from serde_json::Value to async_graphql::Variables
                serde_json::from_value(vars.clone())
                    .map_err(|e| LambdaError::from(format!("Invalid variables: {}", e)))?
            },
            _ => async_graphql::Variables::default(),
        };
        
        // Create the request with the query
        let mut request = async_graphql::Request::new(query_str);
        
        // Add operation name if present
        if let Some(op_name) = operation_name {
            request = request.operation_name(op_name);
        }
        
        // Add variables
        request = request.variables(variables);
        
        let gql_response = schema.execute(request).await;
        let response_json = serde_json::to_string(&gql_response)
            .map_err(|e| LambdaError::from(format!("Failed to serialize GraphQL response: {}", e)))?;
        
        Ok(Response::builder()
            .status(StatusCode::OK)
            .header("Content-Type", "application/json")
            .body(Body::Text(response_json))
            .map_err(|e| LambdaError::from(format!("Failed to build response: {}", e)))?)
    } else if path.starts_with("/playground") {
        // Handle GraphQL Playground request
        let html = GraphiQLSource::build()
            .endpoint("/graphql")
            .finish();
        
        Ok(Response::builder()
            .status(StatusCode::OK)
            .header("Content-Type", "text/html")
            .body(Body::Text(html))
            .map_err(|e| LambdaError::from(format!("Failed to build response: {}", e)))?)
    } else {
        // Handle unknown path
        Ok(Response::builder()
            .status(StatusCode::NOT_FOUND)
            .body(Body::Text(json!({
                "error": "Not Found",
                "message": "The requested resource was not found"
            }).to_string()))
            .map_err(|e| LambdaError::from(format!("Failed to build response: {}", e)))?)
    }
}

#[tokio::main]
async fn main() -> LambdaResult<()> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_ansi(false)
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();
    
    info!("GraphQL API Lambda starting up");
    
    // Run the Lambda function
    run(service_fn(function_handler)).await
}

#[derive(Debug, Error)]
pub enum RepositoryError {
    #[error("DynamoDB error: {0}")]
    DynamoDb(#[from] aws_sdk_dynamodb::Error),
    
    #[error("Item not found: {0}")]
    NotFound(String),
    
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
    
    #[error("Internal error: {0}")]
    Internal(String),
    
    #[error("Unauthorized: {0}")]
    Unauthorized(String),
}

pub type Result<T> = std::result::Result<T, RepositoryError>; 