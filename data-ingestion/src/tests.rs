use aws_lambda_events::event::apigw::{ApiGatewayProxyRequest, ApiGatewayProxyResponse};
use aws_lambda_events::encodings::Body;
use aws_sdk_dynamodb::Client as DynamoDbClient;
use aws_sdk_sqs::Client as SqsClient;
use serde_json::json;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::{DataIngestion, IngestRequest, TransactionData, PortfolioData, AccountData, SecurityData};

// Mock DynamoDB client for testing
struct MockDynamoDbClient {
    items: Arc<Mutex<HashMap<String, serde_json::Value>>>,
}

impl MockDynamoDbClient {
    fn new() -> Self {
        Self {
            items: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    async fn add_item(&self, id: &str, item: serde_json::Value) {
        let mut items = self.items.lock().await;
        items.insert(id.to_string(), item);
    }

    async fn get_item(&self, id: &str) -> Option<serde_json::Value> {
        let items = self.items.lock().await;
        items.get(id).cloned()
    }
}

// Mock SQS client for testing
struct MockSqsClient {
    messages: Arc<Mutex<Vec<String>>>,
}

impl MockSqsClient {
    fn new() -> Self {
        Self {
            messages: Arc::new(Mutex::new(Vec::new())),
        }
    }

    async fn add_message(&self, message: String) {
        let mut messages = self.messages.lock().await;
        messages.push(message);
    }

    async fn get_messages(&self) -> Vec<String> {
        let messages = self.messages.lock().await;
        messages.clone()
    }
}

// Test helper function to create a mock API Gateway request
fn create_mock_request(method: &str, path: &str, body: Option<String>) -> ApiGatewayProxyRequest {
    let mut request = ApiGatewayProxyRequest::default();
    request.http_method = method.to_string();
    request.path = Some(path.to_string());
    request.body = body;
    
    // Add request context with request ID
    let mut request_context = aws_lambda_events::event::apigw::ApiGatewayProxyRequestContext::default();
    request_context.request_id = Some("test-request-id".to_string());
    request.request_context = Some(request_context);
    
    request
}

#[tokio::test]
async fn test_process_transaction() {
    // Create mock clients
    let dynamodb_client = DynamoDbClient::new(&aws_config::from_env().region("us-east-1").build());
    let sqs_client = SqsClient::new(&aws_config::from_env().region("us-east-1").build());
    
    // Create data ingestion
    let ingestion = DataIngestion {
        dynamodb_client,
        sqs_client,
        table_name: "test-table".to_string(),
        processing_queue_url: "test-queue-url".to_string(),
    };
    
    // Create a mock transaction data
    let transaction_data = TransactionData {
        account_id: "test-account-1".to_string(),
        security_id: Some("test-security-1".to_string()),
        transaction_date: "2023-01-01T00:00:00Z".to_string(),
        transaction_type: "BUY".to_string(),
        amount: 1000.0,
        quantity: Some(10.0),
        price: Some(100.0),
    };
    
    // Create a mock ingest request
    let ingest_request = IngestRequest {
        request_type: "transaction".to_string(),
        source: "test".to_string(),
        data: serde_json::to_value(transaction_data).unwrap(),
    };
    
    // Create a mock API Gateway request
    let request_body = serde_json::to_string(&ingest_request).unwrap();
    let request = create_mock_request("POST", "/ingest/transaction", Some(request_body));
    
    // Process the request
    let response = ingestion.handle_request(request).await.unwrap();
    
    // Verify the response
    assert_eq!(response.status_code, 201);
    assert!(response.body.is_some());
    
    // Parse the response body
    if let Some(Body::Text(body)) = response.body {
        let json: serde_json::Value = serde_json::from_str(&body).unwrap();
        assert!(json.get("id").is_some());
        assert_eq!(json["message"], "Data ingested successfully");
        assert_eq!(json["request_id"], "test-request-id");
    } else {
        panic!("Response body is not text");
    }
}

#[tokio::test]
async fn test_process_portfolio() {
    // Create mock clients
    let dynamodb_client = DynamoDbClient::new(&aws_config::from_env().region("us-east-1").build());
    let sqs_client = SqsClient::new(&aws_config::from_env().region("us-east-1").build());
    
    // Create data ingestion
    let ingestion = DataIngestion {
        dynamodb_client,
        sqs_client,
        table_name: "test-table".to_string(),
        processing_queue_url: "test-queue-url".to_string(),
    };
    
    // Create a mock portfolio data
    let portfolio_data = PortfolioData {
        name: "Test Portfolio".to_string(),
        client_id: "test-client-1".to_string(),
    };
    
    // Create a mock ingest request
    let ingest_request = IngestRequest {
        request_type: "portfolio".to_string(),
        source: "test".to_string(),
        data: serde_json::to_value(portfolio_data).unwrap(),
    };
    
    // Create a mock API Gateway request
    let request_body = serde_json::to_string(&ingest_request).unwrap();
    let request = create_mock_request("POST", "/ingest/portfolio", Some(request_body));
    
    // Process the request
    let response = ingestion.handle_request(request).await.unwrap();
    
    // Verify the response
    assert_eq!(response.status_code, 201);
    assert!(response.body.is_some());
    
    // Parse the response body
    if let Some(Body::Text(body)) = response.body {
        let json: serde_json::Value = serde_json::from_str(&body).unwrap();
        assert!(json.get("id").is_some());
        assert_eq!(json["message"], "Data ingested successfully");
        assert_eq!(json["request_id"], "test-request-id");
    } else {
        panic!("Response body is not text");
    }
}

#[tokio::test]
async fn test_process_account() {
    // Create mock clients
    let dynamodb_client = DynamoDbClient::new(&aws_config::from_env().region("us-east-1").build());
    let sqs_client = SqsClient::new(&aws_config::from_env().region("us-east-1").build());
    
    // Create data ingestion
    let ingestion = DataIngestion {
        dynamodb_client,
        sqs_client,
        table_name: "test-table".to_string(),
        processing_queue_url: "test-queue-url".to_string(),
    };
    
    // Create a mock account data
    let account_data = AccountData {
        name: "Test Account".to_string(),
        portfolio_id: "test-portfolio-1".to_string(),
    };
    
    // Create a mock ingest request
    let ingest_request = IngestRequest {
        request_type: "account".to_string(),
        source: "test".to_string(),
        data: serde_json::to_value(account_data).unwrap(),
    };
    
    // Create a mock API Gateway request
    let request_body = serde_json::to_string(&ingest_request).unwrap();
    let request = create_mock_request("POST", "/ingest/account", Some(request_body));
    
    // Process the request
    let response = ingestion.handle_request(request).await.unwrap();
    
    // Verify the response
    assert_eq!(response.status_code, 201);
    assert!(response.body.is_some());
    
    // Parse the response body
    if let Some(Body::Text(body)) = response.body {
        let json: serde_json::Value = serde_json::from_str(&body).unwrap();
        assert!(json.get("id").is_some());
        assert_eq!(json["message"], "Data ingested successfully");
        assert_eq!(json["request_id"], "test-request-id");
    } else {
        panic!("Response body is not text");
    }
}

#[tokio::test]
async fn test_process_security() {
    // Create mock clients
    let dynamodb_client = DynamoDbClient::new(&aws_config::from_env().region("us-east-1").build());
    let sqs_client = SqsClient::new(&aws_config::from_env().region("us-east-1").build());
    
    // Create data ingestion
    let ingestion = DataIngestion {
        dynamodb_client,
        sqs_client,
        table_name: "test-table".to_string(),
        processing_queue_url: "test-queue-url".to_string(),
    };
    
    // Create a mock security data
    let security_data = SecurityData {
        symbol: "AAPL".to_string(),
        name: "Apple Inc.".to_string(),
        security_type: "EQUITY".to_string(),
    };
    
    // Create a mock ingest request
    let ingest_request = IngestRequest {
        request_type: "security".to_string(),
        source: "test".to_string(),
        data: serde_json::to_value(security_data).unwrap(),
    };
    
    // Create a mock API Gateway request
    let request_body = serde_json::to_string(&ingest_request).unwrap();
    let request = create_mock_request("POST", "/ingest/security", Some(request_body));
    
    // Process the request
    let response = ingestion.handle_request(request).await.unwrap();
    
    // Verify the response
    assert_eq!(response.status_code, 201);
    assert!(response.body.is_some());
    
    // Parse the response body
    if let Some(Body::Text(body)) = response.body {
        let json: serde_json::Value = serde_json::from_str(&body).unwrap();
        assert!(json.get("id").is_some());
        assert_eq!(json["message"], "Data ingested successfully");
        assert_eq!(json["request_id"], "test-request-id");
    } else {
        panic!("Response body is not text");
    }
}

#[tokio::test]
async fn test_invalid_request_type() {
    // Create mock clients
    let dynamodb_client = DynamoDbClient::new(&aws_config::from_env().region("us-east-1").build());
    let sqs_client = SqsClient::new(&aws_config::from_env().region("us-east-1").build());
    
    // Create data ingestion
    let ingestion = DataIngestion {
        dynamodb_client,
        sqs_client,
        table_name: "test-table".to_string(),
        processing_queue_url: "test-queue-url".to_string(),
    };
    
    // Create a mock ingest request with invalid type
    let ingest_request = IngestRequest {
        request_type: "invalid".to_string(),
        source: "test".to_string(),
        data: json!({"name": "Test Data"}),
    };
    
    // Create a mock API Gateway request
    let request_body = serde_json::to_string(&ingest_request).unwrap();
    let request = create_mock_request("POST", "/ingest/invalid", Some(request_body));
    
    // Process the request
    let response = ingestion.handle_request(request).await.unwrap();
    
    // Verify the response
    assert_eq!(response.status_code, 400);
    assert!(response.body.is_some());
    
    // Parse the response body
    if let Some(Body::Text(body)) = response.body {
        let json: serde_json::Value = serde_json::from_str(&body).unwrap();
        assert_eq!(json["error"], "Bad Request");
        assert!(json["message"].as_str().unwrap().contains("Unknown request type"));
    } else {
        panic!("Response body is not text");
    }
} 