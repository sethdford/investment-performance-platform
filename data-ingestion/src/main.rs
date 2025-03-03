use aws_lambda_events::event::apigw::{ApiGatewayProxyRequest, ApiGatewayProxyResponse};
use aws_lambda_events::encodings::Body;
use aws_sdk_dynamodb::Client as DynamoDbClient;
use aws_sdk_sqs::Client as SqsClient;
use chrono::{NaiveDate, Utc};
use lambda_runtime::{service_fn, Error, LambdaEvent};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::env;
use tracing::{info, error, warn};
use uuid::Uuid;
use shared::{
    models::{Portfolio, Transaction, Account, Security, TransactionType},
    validation::{Validate, ValidationError},
};
use std::str::FromStr;

#[derive(Debug, Serialize, Deserialize)]
struct IngestRequest {
    #[serde(rename = "type")]
    request_type: String,
    source: String,
    data: serde_json::Value,
}

#[derive(Debug, Serialize, Deserialize)]
struct TransactionData {
    account_id: String,
    security_id: Option<String>,
    transaction_date: String,
    transaction_type: String,
    amount: f64,
    quantity: Option<f64>,
    price: Option<f64>,
}

#[derive(Debug, Serialize, Deserialize)]
struct PortfolioData {
    name: String,
    client_id: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct AccountData {
    name: String,
    portfolio_id: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct SecurityData {
    symbol: String,
    name: String,
    security_type: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct AuditRecord {
    id: String,
    entity_id: String,
    entity_type: String,
    action: String,
    user_id: String,
    timestamp: String,
    details: String,
}

struct DataIngestion {
    dynamodb_client: DynamoDbClient,
    sqs_client: SqsClient,
    table_name: String,
    processing_queue_url: String,
}

impl DataIngestion {
    async fn new() -> Result<Self, Error> {
        let config = aws_config::load_from_env().await;
        let dynamodb_client = DynamoDbClient::new(&config);
        let sqs_client = SqsClient::new(&config);
        let table_name = env::var("DYNAMODB_TABLE").expect("DYNAMODB_TABLE must be set");
        let processing_queue_url = env::var("PROCESSING_QUEUE_URL").expect("PROCESSING_QUEUE_URL must be set");
        
        Ok(Self {
            dynamodb_client,
            sqs_client,
            table_name,
            processing_queue_url,
        })
    }
    
    async fn handle_request(&self, event: ApiGatewayProxyRequest) -> Result<ApiGatewayProxyResponse, Error> {
        let path = event.path.unwrap_or_default();
        let method = event.http_method;
        
        info!(path = %path, method = %method, "Received API request");
        
        // Extract request ID for tracing
        let request_id = event.request_context
            .and_then(|ctx| ctx.request_id)
            .unwrap_or_else(|| "unknown".to_string());
        
        // Only handle POST requests
        if method != "POST" {
            return Ok(ApiGatewayProxyResponse {
                status_code: 405,
                headers: Default::default(),
                multi_value_headers: Default::default(),
                body: Some(Body::Text(json!({
                    "error": "Method Not Allowed",
                    "message": "Only POST method is supported"
                }).to_string())),
                is_base64_encoded: Some(false),
            });
        }
        
        // Parse request body
        let body = match event.body {
            Some(body) => body,
            None => {
                return Ok(ApiGatewayProxyResponse {
                    status_code: 400,
                    headers: Default::default(),
                    multi_value_headers: Default::default(),
                    body: Some(Body::Text(json!({
                        "error": "Bad Request",
                        "message": "Request body is required"
                    }).to_string())),
                    is_base64_encoded: Some(false),
                });
            }
        };
        
        // Parse as IngestRequest
        let ingest_request: IngestRequest = match serde_json::from_str(&body) {
            Ok(req) => req,
            Err(e) => {
                error!(request_id = %request_id, error = %e, "Failed to parse request body");
                return Ok(ApiGatewayProxyResponse {
                    status_code: 400,
                    headers: Default::default(),
                    multi_value_headers: Default::default(),
                    body: Some(Body::Text(json!({
                        "error": "Bad Request",
                        "message": format!("Invalid request format: {}", e)
                    }).to_string())),
                    is_base64_encoded: Some(false),
                });
            }
        };
        
        // Process based on request type
        let result = match ingest_request.request_type.as_str() {
            "transaction" => self.process_transaction(ingest_request.data, &request_id).await,
            "portfolio" => self.process_portfolio(ingest_request.data, &request_id).await,
            "account" => self.process_account(ingest_request.data, &request_id).await,
            "security" => self.process_security(ingest_request.data, &request_id).await,
            _ => {
                warn!(request_id = %request_id, request_type = %ingest_request.request_type, "Unknown request type");
                return Ok(ApiGatewayProxyResponse {
                    status_code: 400,
                    headers: Default::default(),
                    multi_value_headers: Default::default(),
                    body: Some(Body::Text(json!({
                        "error": "Bad Request",
                        "message": format!("Unknown request type: {}", ingest_request.request_type)
                    }).to_string())),
                    is_base64_encoded: Some(false),
                });
            }
        };
        
        // Handle result
        match result {
            Ok(entity_id) => {
                Ok(ApiGatewayProxyResponse {
                    status_code: 201,
                    headers: Default::default(),
                    multi_value_headers: Default::default(),
                    body: Some(Body::Text(json!({
                        "id": entity_id,
                        "message": "Data ingested successfully",
                        "request_id": request_id
                    }).to_string())),
                    is_base64_encoded: Some(false),
                })
            },
            Err(e) => {
                error!(request_id = %request_id, error = %e, "Failed to process request");
                Ok(ApiGatewayProxyResponse {
                    status_code: 500,
                    headers: Default::default(),
                    multi_value_headers: Default::default(),
                    body: Some(Body::Text(json!({
                        "error": "Internal Server Error",
                        "message": format!("Failed to process request: {}", e),
                        "request_id": request_id
                    }).to_string())),
                    is_base64_encoded: Some(false),
                })
            }
        }
    }
    
    async fn process_transaction(&self, data: serde_json::Value, request_id: &str) -> Result<String, Error> {
        info!(request_id = %request_id, "Processing transaction data");
        
        // Parse transaction data
        let transaction_data: TransactionData = serde_json::from_value(data.clone())?;
        
        // Convert to shared Transaction model
        let transaction_date = NaiveDate::from_str(&transaction_data.transaction_date)
            .map_err(|e| format!("Invalid transaction date format: {}", e))?;
        
        let transaction_type = match transaction_data.transaction_type.as_str() {
            "buy" | "Buy" => TransactionType::Buy,
            "sell" | "Sell" => TransactionType::Sell,
            "deposit" | "Deposit" => TransactionType::Deposit,
            "withdrawal" | "Withdrawal" => TransactionType::Withdrawal,
            "dividend" | "Dividend" => TransactionType::Dividend,
            "interest" | "Interest" => TransactionType::Interest,
            "fee" | "Fee" => TransactionType::Fee,
            "transfer" | "Transfer" => TransactionType::Transfer,
            "split" | "Split" => TransactionType::Split,
            other => TransactionType::Other(other.to_string()),
        };
        
        // Create a Transaction object
        let transaction = Transaction {
            id: Uuid::new_v4().to_string(),
            account_id: transaction_data.account_id,
            security_id: transaction_data.security_id,
            transaction_date,
            settlement_date: None,
            transaction_type,
            amount: transaction_data.amount,
            quantity: transaction_data.quantity,
            price: transaction_data.price,
            fees: None,
            currency: "USD".to_string(), // Default currency
            created_at: Utc::now(),
            updated_at: Utc::now(),
            metadata: Default::default(),
        };
        
        // Validate the transaction
        if let Err(e) = transaction.validate() {
            return Err(format!("Transaction validation failed: {}", e).into());
        }
        
        // Store transaction in DynamoDB
        // TODO: Implement actual DynamoDB write
        
        // Create audit record
        self.create_audit_record(
            &transaction.id,
            "transaction",
            "create",
            "system",
            &format!(
                "Transaction created: {:?} {} for account {}",
                transaction.transaction_type,
                transaction.amount,
                transaction.account_id
            ),
        ).await?;
        
        // Send message to processing queue
        let message = json!({
            "event_type": "TRANSACTION_CREATED",
            "transaction": {
                "id": transaction.id,
                "account_id": transaction.account_id,
                "security_id": transaction.security_id,
                "transaction_type": format!("{:?}", transaction.transaction_type),
                "amount": transaction.amount,
                "quantity": transaction.quantity,
                "price": transaction.price,
                "transaction_date": transaction.transaction_date.to_string(),
                "created_at": transaction.created_at.to_rfc3339()
            }
        });
        
        self.send_to_processing_queue(message.to_string()).await?;
        
        Ok(transaction.id)
    }
    
    async fn process_portfolio(&self, data: serde_json::Value, request_id: &str) -> Result<String, Error> {
        info!(request_id = %request_id, "Processing portfolio data");
        
        // Parse portfolio data
        let portfolio_data: PortfolioData = serde_json::from_value(data.clone())?;
        
        // Create a Portfolio object
        let portfolio = Portfolio {
            id: Uuid::new_v4().to_string(),
            name: portfolio_data.name,
            client_id: portfolio_data.client_id,
            inception_date: Utc::now().date_naive(),
            benchmark_id: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            status: shared::models::Status::Active,
            metadata: Default::default(),
        };
        
        // Validate the portfolio
        if let Err(e) = portfolio.validate() {
            return Err(format!("Portfolio validation failed: {}", e).into());
        }
        
        // Store portfolio in DynamoDB
        // TODO: Implement actual DynamoDB write
        
        // Create audit record
        self.create_audit_record(
            &portfolio.id,
            "portfolio",
            "create",
            "system",
            &format!(
                "Portfolio created: {} for client {}",
                portfolio.name,
                portfolio.client_id
            ),
        ).await?;
        
        // Send message to processing queue
        let message = json!({
            "event_type": "PORTFOLIO_CREATED",
            "portfolio": {
                "id": portfolio.id,
                "name": portfolio.name,
                "client_id": portfolio.client_id,
                "inception_date": portfolio.inception_date.to_string(),
                "created_at": portfolio.created_at.to_rfc3339()
            }
        });
        
        self.send_to_processing_queue(message.to_string()).await?;
        
        Ok(portfolio.id)
    }
    
    async fn process_account(&self, data: serde_json::Value, request_id: &str) -> Result<String, Error> {
        info!(request_id = %request_id, "Processing account data");
        
        // Parse account data
        let account_data: AccountData = serde_json::from_value(data.clone())?;
        
        // Create an Account object
        let account = Account {
            id: Uuid::new_v4().to_string(),
            account_number: format!("ACC-{}", Uuid::new_v4().to_string().split('-').next().unwrap()),
            name: account_data.name,
            portfolio_id: account_data.portfolio_id,
            account_type: shared::models::AccountType::Regular,
            tax_status: shared::models::TaxStatus::Taxable,
            inception_date: Utc::now().date_naive(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
            status: shared::models::Status::Active,
            metadata: Default::default(),
        };
        
        // Validate the account
        if let Err(e) = account.validate() {
            return Err(format!("Account validation failed: {}", e).into());
        }
        
        // Store account in DynamoDB
        // TODO: Implement actual DynamoDB write
        
        // Create audit record
        self.create_audit_record(
            &account.id,
            "account",
            "create",
            "system",
            &format!(
                "Account created: {} for portfolio {}",
                account.name,
                account.portfolio_id
            ),
        ).await?;
        
        // Send message to processing queue
        let message = json!({
            "event_type": "ACCOUNT_CREATED",
            "account": {
                "id": account.id,
                "account_number": account.account_number,
                "name": account.name,
                "portfolio_id": account.portfolio_id,
                "account_type": format!("{:?}", account.account_type),
                "tax_status": format!("{:?}", account.tax_status),
                "inception_date": account.inception_date.to_string(),
                "created_at": account.created_at.to_rfc3339()
            }
        });
        
        self.send_to_processing_queue(message.to_string()).await?;
        
        Ok(account.id)
    }
    
    async fn process_security(&self, data: serde_json::Value, request_id: &str) -> Result<String, Error> {
        info!(request_id = %request_id, "Processing security data");
        
        // Parse security data
        let security_data: SecurityData = serde_json::from_value(data.clone())?;
        
        // Create a Security object
        let security = Security {
            id: Uuid::new_v4().to_string(),
            symbol: security_data.symbol,
            name: security_data.name,
            security_type: match security_data.security_type.as_str() {
                "equity" | "Equity" => shared::models::SecurityType::Equity,
                "bond" | "Bond" => shared::models::SecurityType::Bond,
                "mutual_fund" | "MutualFund" => shared::models::SecurityType::MutualFund,
                "etf" | "ETF" => shared::models::SecurityType::ETF,
                "option" | "Option" => shared::models::SecurityType::Option,
                "future" | "Future" => shared::models::SecurityType::Future,
                "forex" | "Forex" => shared::models::SecurityType::Forex,
                "crypto" | "Crypto" => shared::models::SecurityType::Crypto,
                _ => shared::models::SecurityType::Other(security_data.security_type),
            },
            asset_class: shared::models::AssetClass::Other("Unknown".to_string()),
            cusip: None,
            isin: None,
            sedol: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            metadata: Default::default(),
        };
        
        // Validate the security
        if let Err(e) = security.validate() {
            return Err(format!("Security validation failed: {}", e).into());
        }
        
        // Store security in DynamoDB
        // TODO: Implement actual DynamoDB write
        
        // Create audit record
        self.create_audit_record(
            &security.id,
            "security",
            "create",
            "system",
            &format!(
                "Security created: {} ({}) - {:?}",
                security.name,
                security.symbol,
                security.security_type
            ),
        ).await?;
        
        // Send message to processing queue
        let message = json!({
            "event_type": "SECURITY_CREATED",
            "security": {
                "id": security.id,
                "symbol": security.symbol,
                "name": security.name,
                "security_type": format!("{:?}", security.security_type),
                "asset_class": format!("{:?}", security.asset_class),
                "created_at": security.created_at.to_rfc3339()
            }
        });
        
        self.send_to_processing_queue(message.to_string()).await?;
        
        Ok(security.id)
    }
    
    async fn create_audit_record(
        &self,
        entity_id: &str,
        entity_type: &str,
        action: &str,
        user_id: &str,
        details: &str,
    ) -> Result<(), Error> {
        let audit_record = AuditRecord {
            id: Uuid::new_v4().to_string(),
            entity_id: entity_id.to_string(),
            entity_type: entity_type.to_string(),
            action: action.to_string(),
            user_id: user_id.to_string(),
            timestamp: Utc::now().to_rfc3339(),
            details: details.to_string(),
        };
        
        // TODO: Implement actual DynamoDB write
        // For now, just log the audit record
        info!(
            audit_id = %audit_record.id,
            entity_id = %audit_record.entity_id,
            entity_type = %audit_record.entity_type,
            action = %audit_record.action,
            "Created audit record"
        );
        
        Ok(())
    }
    
    async fn send_to_processing_queue(&self, message: String) -> Result<(), Error> {
        info!("Sending message to processing queue");
        
        let send_result = self.sqs_client
            .send_message()
            .queue_url(&self.processing_queue_url)
            .message_body(message)
            .send()
            .await;
        
        match send_result {
            Ok(output) => {
                info!(
                    message_id = %output.message_id().unwrap_or("unknown"),
                    "Message sent to processing queue"
                );
            },
            Err(e) => {
                error!("Failed to send message to processing queue: {}", e);
                return Err(Box::new(e));
            }
        }
        
        Ok(())
    }
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_ansi(false)
        .without_time()
        .with_max_level(tracing::Level::INFO)
        .init();
    
    info!("Data Ingestion starting up");
    
    // Create data ingestion
    let ingestion = DataIngestion::new().await?;
    
    // Start Lambda runtime
    lambda_runtime::run(service_fn(|event: LambdaEvent<ApiGatewayProxyRequest>| async {
        let (event, _context) = event.into_parts();
        ingestion.handle_request(event).await
    })).await?;
    
    Ok(())
} 