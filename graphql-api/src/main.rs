use lambda_http::{run, service_fn, Body, Error, Request, Response};
use lambda_runtime::Context;
use aws_sdk_dynamodb::Client as DynamoDbClient;
use aws_sdk_timestreamquery::Client as TimestreamQueryClient;
use aws_config::BehaviorVersion;
use serde_json::json;
use tracing::{info, error};
use shared::models::{Client, Portfolio, Account, Security, Transaction, PerformanceMetric};
use shared::repository::{Repository, DynamoDbRepository};
use async_graphql::{Schema, EmptySubscription, http::GraphiQLSource};
use async_graphql_lambda::{graphql_handler, GraphQLRequest};
use http::StatusCode;
use std::sync::Arc;
use anyhow::{Result, anyhow};
use async_graphql::{
    Context, EmptyMutation, EmptySubscription, Object, Schema, SimpleObject, ID, Float, InputObject,
};
use async_graphql::http::{GraphQLPlaygroundConfig, playground_source};
use aws_lambda_events::event::apigw::{ApiGatewayProxyRequest, ApiGatewayProxyResponse};
use aws_lambda_events::encodings::Body;
use chrono::{DateTime, Utc, NaiveDate};
use lambda_runtime::{service_fn, LambdaEvent};
use serde::{Deserialize, Serialize};
use aws_sdk_dynamodb::types::AttributeValue;
use std::collections::HashMap;
use thiserror::Error;

mod schema;
use schema::{Query, Mutation, InvestmentSchema};

async fn function_handler(event: Request, _: Context) -> Result<Response<Body>, Error> {
    info!("GraphQL API Lambda received request");
    
    // Initialize AWS clients
    let config = aws_config::defaults(BehaviorVersion::latest())
        .load()
        .await;
    
    let dynamodb_client = DynamoDbClient::new(&config);
    let timestream_client = TimestreamQueryClient::new(&config);
    
    // Get table name from environment variable
    let table_name = std::env::var("DYNAMODB_TABLE")
        .map_err(|_| anyhow!("DYNAMODB_TABLE environment variable not set"))?;
    
    // Get Timestream database name from environment variable
    let timestream_database = std::env::var("TIMESTREAM_DATABASE")
        .map_err(|_| anyhow!("TIMESTREAM_DATABASE environment variable not set"))?;
    
    // Create repository
    let repository = Arc::new(DynamoDbRepository::new(dynamodb_client, table_name));
    
    // Create GraphQL schema
    let schema = Schema::build(Query::default(), Mutation::default(), EmptySubscription)
        .data(repository)
        .data(timestream_client)
        .data(timestream_database)
        .finish();
    
    // Handle GraphiQL request
    if event.uri().path() == "/graphiql" {
        let html = GraphiQLSource::build().endpoint("/graphql").finish();
        return Ok(Response::builder()
            .status(StatusCode::OK)
            .header("Content-Type", "text/html")
            .body(Body::from(html))
            .expect("Failed to build GraphiQL response"));
    }
    
    // Handle GraphQL request
    let schema_ref = schema.clone();
    let response = graphql_handler(schema_ref, event).await?;
    
    Ok(response)
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_ansi(false)
        .with_max_level(tracing::Level::INFO)
        .with_target(false)
        .init();
    
    info!("GraphQL API Lambda starting up");
    
    run(service_fn(function_handler)).await
}

// GraphQL schema module
mod schema {
    use async_graphql::{Context, Object, Schema, SimpleObject, InputObject, ID, Enum};
    use chrono::{DateTime, Utc};
    use shared::models::{Client, Portfolio, Account, Security, Transaction, PerformanceMetric};
    use shared::repository::{Repository, DynamoDbRepository};
    use aws_sdk_timestreamquery::Client as TimestreamQueryClient;
    use aws_sdk_timestreamquery::model::QueryRequest;
    use std::sync::Arc;
    use uuid::Uuid;
    use tracing::{info, error};
    use anyhow::{Result, anyhow};
    use rust_decimal::Decimal;
    use serde::{Serialize, Deserialize};
    
    pub type InvestmentSchema = Schema<Query, Mutation, async_graphql::EmptySubscription>;
    
    #[derive(Default)]
    pub struct Query;
    
    #[Object]
    impl Query {
        // Client queries
        async fn client(&self, ctx: &Context<'_>, id: ID) -> async_graphql::Result<Option<Client>> {
            let repository = ctx.data::<Arc<DynamoDbRepository>>()?;
            let client = repository.get_client(&id).await?;
            Ok(client)
        }
        
        async fn clients(&self, ctx: &Context<'_>) -> async_graphql::Result<Vec<Client>> {
            let repository = ctx.data::<Arc<DynamoDbRepository>>()?;
            let clients = repository.list_clients().await?;
            Ok(clients)
        }
        
        // Portfolio queries
        async fn portfolio(&self, ctx: &Context<'_>, id: ID) -> async_graphql::Result<Option<Portfolio>> {
            let repository = ctx.data::<Arc<DynamoDbRepository>>()?;
            let portfolio = repository.get_portfolio(&id).await?;
            Ok(portfolio)
        }
        
        async fn portfolios(&self, ctx: &Context<'_>, client_id: Option<ID>) -> async_graphql::Result<Vec<Portfolio>> {
            let repository = ctx.data::<Arc<DynamoDbRepository>>()?;
            let portfolios = repository.list_portfolios(client_id.as_ref().map(|id| id.as_str())).await?;
            Ok(portfolios)
        }
        
        // Account queries
        async fn account(&self, ctx: &Context<'_>, id: ID) -> async_graphql::Result<Option<Account>> {
            let repository = ctx.data::<Arc<DynamoDbRepository>>()?;
            let account = repository.get_account(&id).await?;
            Ok(account)
        }
        
        async fn accounts(&self, ctx: &Context<'_>, portfolio_id: Option<ID>) -> async_graphql::Result<Vec<Account>> {
            let repository = ctx.data::<Arc<DynamoDbRepository>>()?;
            let accounts = repository.list_accounts(portfolio_id.as_ref().map(|id| id.as_str())).await?;
            Ok(accounts)
        }
        
        // Security queries
        async fn security(&self, ctx: &Context<'_>, id: ID) -> async_graphql::Result<Option<Security>> {
            let repository = ctx.data::<Arc<DynamoDbRepository>>()?;
            let security = repository.get_security(&id).await?;
            Ok(security)
        }
        
        async fn securities(&self, ctx: &Context<'_>, account_id: Option<ID>) -> async_graphql::Result<Vec<Security>> {
            let repository = ctx.data::<Arc<DynamoDbRepository>>()?;
            let securities = repository.list_securities(account_id.as_ref().map(|id| id.as_str())).await?;
            Ok(securities)
        }
        
        // Transaction queries
        async fn transaction(&self, ctx: &Context<'_>, id: ID) -> async_graphql::Result<Option<Transaction>> {
            let repository = ctx.data::<Arc<DynamoDbRepository>>()?;
            let transaction = repository.get_transaction(&id).await?;
            Ok(transaction)
        }
        
        async fn transactions(
            &self, 
            ctx: &Context<'_>, 
            account_id: Option<ID>,
            security_id: Option<ID>
        ) -> async_graphql::Result<Vec<Transaction>> {
            let repository = ctx.data::<Arc<DynamoDbRepository>>()?;
            let transactions = repository.list_transactions(
                account_id.as_ref().map(|id| id.as_str()),
                security_id.as_ref().map(|id| id.as_str())
            ).await?;
            Ok(transactions)
        }
        
        // Performance queries
        async fn performance_metric(&self, ctx: &Context<'_>, id: ID) -> async_graphql::Result<Option<PerformanceMetric>> {
            let repository = ctx.data::<Arc<DynamoDbRepository>>()?;
            let metric = repository.get_performance_metric(&id).await?;
            Ok(metric)
        }
        
        async fn performance_metrics(
            &self, 
            ctx: &Context<'_>, 
            portfolio_id: Option<ID>,
            account_id: Option<ID>,
            start_date: Option<DateTime<Utc>>,
            end_date: Option<DateTime<Utc>>
        ) -> async_graphql::Result<Vec<PerformanceMetric>> {
            let repository = ctx.data::<Arc<DynamoDbRepository>>()?;
            let metrics = repository.list_performance_metrics(
                portfolio_id.as_ref().map(|id| id.as_str()),
                account_id.as_ref().map(|id| id.as_str()),
                start_date,
                end_date
            ).await?;
            Ok(metrics)
        }
        
        // Time series performance query
        async fn performance_time_series(
            &self,
            ctx: &Context<'_>,
            portfolio_id: Option<ID>,
            account_id: Option<ID>,
            metric_type: PerformanceMetricType,
            start_date: DateTime<Utc>,
            end_date: DateTime<Utc>,
            interval: TimeSeriesInterval
        ) -> async_graphql::Result<Vec<TimeSeriesDataPoint>> {
            let timestream_client = ctx.data::<TimestreamQueryClient>()?;
            let database = ctx.data::<String>()?;
            
            // Build the Timestream query
            let mut query = format!(
                "SELECT time, measure_value::double as value, 
                 {} as dimension_id
                 FROM \"{}\".\"performance_metrics\"
                 WHERE time BETWEEN '{}' AND '{}'
                 AND measure_name = '{}'",
                if portfolio_id.is_some() { "portfolio_id" } else { "account_id" },
                database,
                start_date.to_rfc3339(),
                end_date.to_rfc3339(),
                match metric_type {
                    PerformanceMetricType::TimeWeightedReturn => "time_weighted_return",
                    PerformanceMetricType::MoneyWeightedReturn => "money_weighted_return",
                }
            );
            
            // Add dimension filter
            if let Some(id) = &portfolio_id {
                query.push_str(&format!(" AND portfolio_id = '{}'", id));
            } else if let Some(id) = &account_id {
                query.push_str(&format!(" AND account_id = '{}'", id));
            }
            
            // Add time bucketing based on interval
            query.push_str(&format!(" GROUP BY TIMESTAMP_TRUNC(time, {})", match interval {
                TimeSeriesInterval::Day => "DAY",
                TimeSeriesInterval::Week => "WEEK",
                TimeSeriesInterval::Month => "MONTH",
                TimeSeriesInterval::Quarter => "QUARTER",
                TimeSeriesInterval::Year => "YEAR",
            }));
            
            query.push_str(" ORDER BY time ASC");
            
            // Execute the query
            let query_result = timestream_client.query()
                .query_string(query)
                .send()
                .await
                .map_err(|e| async_graphql::Error::new(format!("Timestream query error: {}", e)))?;
            
            // Parse the results
            let mut data_points = Vec::new();
            
            if let Some(rows) = query_result.rows() {
                for row in rows {
                    if let Some(data) = row.data() {
                        // Extract time
                        let time_str = data.get(0)
                            .and_then(|d| d.scalar_value())
                            .ok_or_else(|| async_graphql::Error::new("Missing time value"))?;
                        
                        let time = DateTime::parse_from_rfc3339(time_str)
                            .map_err(|e| async_graphql::Error::new(format!("Invalid time format: {}", e)))?
                            .with_timezone(&Utc);
                        
                        // Extract value
                        let value_str = data.get(1)
                            .and_then(|d| d.scalar_value())
                            .ok_or_else(|| async_graphql::Error::new("Missing value"))?;
                        
                        let value = value_str.parse::<f64>()
                            .map_err(|e| async_graphql::Error::new(format!("Invalid value format: {}", e)))?;
                        
                        // Extract dimension ID
                        let dimension_id = data.get(2)
                            .and_then(|d| d.scalar_value())
                            .ok_or_else(|| async_graphql::Error::new("Missing dimension ID"))?
                            .to_string();
                        
                        data_points.push(TimeSeriesDataPoint {
                            time,
                            value,
                            dimension_id,
                        });
                    }
                }
            }
            
            Ok(data_points)
        }
    }
    
    #[derive(Default)]
    pub struct Mutation;
    
    #[Object]
    impl Mutation {
        // Client mutations
        async fn create_client(&self, ctx: &Context<'_>, input: ClientInput) -> async_graphql::Result<Client> {
            let repository = ctx.data::<Arc<DynamoDbRepository>>()?;
            
            let client = Client {
                id: Uuid::new_v4().to_string(),
                name: input.name,
                ..Default::default()
            };
            
            repository.put_client(&client).await?;
            
            Ok(client)
        }
        
        async fn update_client(&self, ctx: &Context<'_>, id: ID, input: ClientInput) -> async_graphql::Result<Client> {
            let repository = ctx.data::<Arc<DynamoDbRepository>>()?;
            
            let mut client = repository.get_client(&id)
                .await?
                .ok_or_else(|| async_graphql::Error::new(format!("Client not found: {}", id)))?;
            
            client.name = input.name;
            
            repository.put_client(&client).await?;
            
            Ok(client)
        }
        
        async fn delete_client(&self, ctx: &Context<'_>, id: ID) -> async_graphql::Result<ID> {
            let repository = ctx.data::<Arc<DynamoDbRepository>>()?;
            
            repository.delete_client(&id).await?;
            
            Ok(id)
        }
        
        // Portfolio mutations
        async fn create_portfolio(&self, ctx: &Context<'_>, input: PortfolioInput) -> async_graphql::Result<Portfolio> {
            let repository = ctx.data::<Arc<DynamoDbRepository>>()?;
            
            // Verify client exists
            repository.get_client(&input.client_id)
                .await?
                .ok_or_else(|| async_graphql::Error::new(format!("Client not found: {}", input.client_id)))?;
            
            let portfolio = Portfolio {
                id: Uuid::new_v4().to_string(),
                name: input.name,
                client_id: input.client_id,
                ..Default::default()
            };
            
            repository.put_portfolio(&portfolio).await?;
            
            Ok(portfolio)
        }
        
        async fn update_portfolio(&self, ctx: &Context<'_>, id: ID, input: PortfolioInput) -> async_graphql::Result<Portfolio> {
            let repository = ctx.data::<Arc<DynamoDbRepository>>()?;
            
            let mut portfolio = repository.get_portfolio(&id)
                .await?
                .ok_or_else(|| async_graphql::Error::new(format!("Portfolio not found: {}", id)))?;
            
            // Verify client exists
            repository.get_client(&input.client_id)
                .await?
                .ok_or_else(|| async_graphql::Error::new(format!("Client not found: {}", input.client_id)))?;
            
            portfolio.name = input.name;
            portfolio.client_id = input.client_id;
            
            repository.put_portfolio(&portfolio).await?;
            
            Ok(portfolio)
        }
        
        async fn delete_portfolio(&self, ctx: &Context<'_>, id: ID) -> async_graphql::Result<ID> {
            let repository = ctx.data::<Arc<DynamoDbRepository>>()?;
            
            repository.delete_portfolio(&id).await?;
            
            Ok(id)
        }
        
        // Account mutations
        async fn create_account(&self, ctx: &Context<'_>, input: AccountInput) -> async_graphql::Result<Account> {
            let repository = ctx.data::<Arc<DynamoDbRepository>>()?;
            
            // Verify portfolio exists
            repository.get_portfolio(&input.portfolio_id)
                .await?
                .ok_or_else(|| async_graphql::Error::new(format!("Portfolio not found: {}", input.portfolio_id)))?;
            
            let account = Account {
                id: Uuid::new_v4().to_string(),
                name: input.name,
                portfolio_id: input.portfolio_id,
                account_type: input.account_type,
                ..Default::default()
            };
            
            repository.put_account(&account).await?;
            
            Ok(account)
        }
        
        async fn update_account(&self, ctx: &Context<'_>, id: ID, input: AccountInput) -> async_graphql::Result<Account> {
            let repository = ctx.data::<Arc<DynamoDbRepository>>()?;
            
            let mut account = repository.get_account(&id)
                .await?
                .ok_or_else(|| async_graphql::Error::new(format!("Account not found: {}", id)))?;
            
            // Verify portfolio exists
            repository.get_portfolio(&input.portfolio_id)
                .await?
                .ok_or_else(|| async_graphql::Error::new(format!("Portfolio not found: {}", input.portfolio_id)))?;
            
            account.name = input.name;
            account.portfolio_id = input.portfolio_id;
            account.account_type = input.account_type;
            
            repository.put_account(&account).await?;
            
            Ok(account)
        }
        
        async fn delete_account(&self, ctx: &Context<'_>, id: ID) -> async_graphql::Result<ID> {
            let repository = ctx.data::<Arc<DynamoDbRepository>>()?;
            
            repository.delete_account(&id).await?;
            
            Ok(id)
        }
        
        // Transaction mutations
        async fn create_transaction(&self, ctx: &Context<'_>, input: TransactionInput) -> async_graphql::Result<Transaction> {
            let repository = ctx.data::<Arc<DynamoDbRepository>>()?;
            
            // Verify account exists
            repository.get_account(&input.account_id)
                .await?
                .ok_or_else(|| async_graphql::Error::new(format!("Account not found: {}", input.account_id)))?;
            
            // Verify security exists
            repository.get_security(&input.security_id)
                .await?
                .ok_or_else(|| async_graphql::Error::new(format!("Security not found: {}", input.security_id)))?;
            
            let transaction = Transaction {
                id: Uuid::new_v4().to_string(),
                account_id: input.account_id,
                security_id: input.security_id,
                transaction_date: input.transaction_date,
                transaction_type: input.transaction_type,
                amount: input.amount,
                quantity: input.quantity,
                price: input.price,
                ..Default::default()
            };
            
            repository.put_transaction(&transaction).await?;
            
            Ok(transaction)
        }
        
        async fn delete_transaction(&self, ctx: &Context<'_>, id: ID) -> async_graphql::Result<ID> {
            let repository = ctx.data::<Arc<DynamoDbRepository>>()?;
            
            repository.delete_transaction(&id).await?;
            
            Ok(id)
        }
    }
    
    // Input types
    #[derive(InputObject)]
    pub struct ClientInput {
        pub name: String,
    }
    
    #[derive(InputObject)]
    pub struct PortfolioInput {
        pub name: String,
        pub client_id: String,
    }
    
    #[derive(InputObject)]
    pub struct AccountInput {
        pub name: String,
        pub portfolio_id: String,
        pub account_type: String,
    }
    
    #[derive(InputObject)]
    pub struct TransactionInput {
        pub account_id: String,
        pub security_id: String,
        pub transaction_date: DateTime<Utc>,
        pub transaction_type: String,
        pub amount: Decimal,
        pub quantity: Decimal,
        pub price: Decimal,
    }
    
    // Time series types
    #[derive(SimpleObject)]
    pub struct TimeSeriesDataPoint {
        pub time: DateTime<Utc>,
        pub value: f64,
        pub dimension_id: String,
    }
    
    #[derive(Enum, Copy, Clone, Eq, PartialEq)]
    pub enum PerformanceMetricType {
        TimeWeightedReturn,
        MoneyWeightedReturn,
    }
    
    #[derive(Enum, Copy, Clone, Eq, PartialEq)]
    pub enum TimeSeriesInterval {
        Day,
        Week,
        Month,
        Quarter,
        Year,
    }
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