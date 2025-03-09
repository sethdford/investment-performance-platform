use lambda_http::{run, service_fn, Body, Error, Request, Response, http::StatusCode};
use tracing::info;
use aws_sdk_sqs::Client as SqsClient;
use serde_json::json;
use std::env;
use chrono::Utc;
use aws_sdk_dynamodb::Client as DynamoDbClient;

/// Main entry point for the Lambda function
#[tokio::main]
async fn main() -> Result<(), Error> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_ansi(false)
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .json()
        .init();
    
    info!("API Handler Lambda starting up");
    
    // Initialize AWS clients
    let config = aws_config::from_env().load().await;
    let dynamodb_client = DynamoDbClient::new(&config);
    let sqs_client = SqsClient::new(&config);
    
    // Get environment variables
    let table_name = env::var("TABLE_NAME").unwrap_or_else(|_| "Items".to_string());
    let queue_url = env::var("QUEUE_URL").unwrap_or_else(|_| "".to_string());
    
    // Create API handler
    let api_handler = ApiHandler {
        dynamodb_client,
        sqs_client,
        table_name,
        queue_url,
    };
    
    // Run the Lambda service
    run(service_fn(move |event: Request| {
        let api_handler = api_handler.clone();
        async move {
            handle_request(event, api_handler).await
        }
    })).await
}

/// API Handler struct
#[derive(Clone)]
struct ApiHandler {
    dynamodb_client: DynamoDbClient,
    sqs_client: SqsClient,
    table_name: String,
    queue_url: String,
}

/// Handle HTTP request
async fn handle_request(event: Request, _api_handler: ApiHandler) -> Result<Response<Body>, Error> {
    // Extract the HTTP method and path
    let method = event.method().as_str();
    let path = event.uri().path();
    
    info!(method = %method, path = %path, "Received request");
    
    // Route the request to the appropriate handler
    match (method, path) {
        // Health check endpoint
        ("GET", "/health") => {
            Ok(Response::builder()
                .status(StatusCode::OK)
                .body(Body::from(json!({
                    "status": "healthy",
                    "timestamp": Utc::now().to_rfc3339()
                }).to_string()))?
            )
        },
        
        // Unknown endpoint
        _ => {
            Ok(Response::builder()
                .status(StatusCode::NOT_FOUND)
                .body(Body::from(json!({
                    "error": "Not found",
                    "message": "The requested endpoint does not exist"
                }).to_string()))?
            )
        }
    }
} 