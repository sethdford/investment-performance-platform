use aws_lambda_events::event::sqs::{SqsEvent, SqsMessage};
use aws_sdk_dynamodb::Client as DynamoDbClient;
use aws_sdk_sqs::Client as SqsClient;
use aws_sdk_timestreamwrite::Client as TimestreamClient;
use chrono::{NaiveDate, Utc};
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use serde_json::json;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::{PerformanceCalculator, PerformanceCalculationRequest, PerformanceResult};
use performance_calculator::calculations::portfolio::Portfolio;

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

// Test helper function to create a mock SQS event
fn create_mock_sqs_event(body: &str) -> SqsEvent {
    let mut message = SqsMessage::default();
    message.body = Some(body.to_string());
    message.message_id = Some("test-message-id".to_string());
    
    SqsEvent {
        records: vec![message],
    }
}

#[tokio::test]
async fn test_calculate_performance() {
    // Create mock clients
    let dynamodb_client = DynamoDbClient::new(&aws_config::from_env().region("us-east-1").build());
    let timestream_client = Some(TimestreamClient::new(&aws_config::from_env().region("us-east-1").build()));
    
    // Create performance calculator
    let calculator = PerformanceCalculator {
        dynamodb_client,
        timestream_client,
        table_name: "test-table".to_string(),
        timestream_database: Some("test-database".to_string()),
        timestream_table: Some("test-table".to_string()),
        twr_calculator: performance_calculator::calculations::performance_metrics::TimeWeightedReturnCalculator::new(),
        mwr_calculator: performance_calculator::calculations::performance_metrics::MoneyWeightedReturnCalculator::new(),
        risk_calculator: performance_calculator::calculations::risk_metrics::RiskMetricsCalculator::new(),
    };
    
    // Create a mock performance calculation request
    let calculation_request = PerformanceCalculationRequest {
        portfolio_id: "portfolio1".to_string(),
        start_date: "2023-01-01T00:00:00Z".to_string(),
        end_date: "2023-12-31T23:59:59Z".to_string(),
        calculation_types: vec!["TWR".to_string(), "MWR".to_string(), "VOLATILITY".to_string()],
        request_id: "req-123456".to_string(),
    };
    
    // Create a mock SQS event
    let event_json = serde_json::to_string(&calculation_request).unwrap();
    let sqs_event = create_mock_sqs_event(&event_json);
    
    // Process the event
    let result = calculator.process_event(sqs_event).await;
    
    // Verify the result
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_get_portfolio() {
    // Create mock clients
    let dynamodb_client = DynamoDbClient::new(&aws_config::from_env().region("us-east-1").build());
    let timestream_client = None;
    
    // Create performance calculator
    let calculator = PerformanceCalculator {
        dynamodb_client,
        timestream_client,
        table_name: "test-table".to_string(),
        timestream_database: None,
        timestream_table: None,
        twr_calculator: performance_calculator::calculations::performance_metrics::TimeWeightedReturnCalculator::new(),
        mwr_calculator: performance_calculator::calculations::performance_metrics::MoneyWeightedReturnCalculator::new(),
        risk_calculator: performance_calculator::calculations::risk_metrics::RiskMetricsCalculator::new(),
    };
    
    // Get a portfolio
    let portfolio = calculator.get_portfolio("portfolio1").await.unwrap();
    
    // Verify the portfolio
    assert_eq!(portfolio.id, "portfolio1");
    assert_eq!(portfolio.base_currency, "USD");
}

#[tokio::test]
async fn test_store_performance_result() {
    // Create mock clients
    let dynamodb_client = DynamoDbClient::new(&aws_config::from_env().region("us-east-1").build());
    let timestream_client = None;
    
    // Create performance calculator
    let calculator = PerformanceCalculator {
        dynamodb_client,
        timestream_client,
        table_name: "test-table".to_string(),
        timestream_database: None,
        timestream_table: None,
        twr_calculator: performance_calculator::calculations::performance_metrics::TimeWeightedReturnCalculator::new(),
        mwr_calculator: performance_calculator::calculations::performance_metrics::MoneyWeightedReturnCalculator::new(),
        risk_calculator: performance_calculator::calculations::risk_metrics::RiskMetricsCalculator::new(),
    };
    
    // Create a performance result
    let result = PerformanceResult {
        portfolio_id: "portfolio1".to_string(),
        start_date: "2023-01-01T00:00:00Z".to_string(),
        end_date: "2023-12-31T23:59:59Z".to_string(),
        twr: Some(0.0875),
        mwr: Some(0.0825),
        volatility: Some(0.12),
        sharpe_ratio: Some(0.65),
        max_drawdown: Some(0.08),
        calculated_at: Utc::now().to_rfc3339(),
        request_id: "req-123456".to_string(),
    };
    
    // Store the result
    let store_result = calculator.store_performance_result(&result).await;
    
    // Verify the result
    assert!(store_result.is_ok());
}

#[tokio::test]
async fn test_create_audit_record() {
    // Create mock clients
    let dynamodb_client = DynamoDbClient::new(&aws_config::from_env().region("us-east-1").build());
    let timestream_client = None;
    
    // Create performance calculator
    let calculator = PerformanceCalculator {
        dynamodb_client,
        timestream_client,
        table_name: "test-table".to_string(),
        timestream_database: None,
        timestream_table: None,
        twr_calculator: performance_calculator::calculations::performance_metrics::TimeWeightedReturnCalculator::new(),
        mwr_calculator: performance_calculator::calculations::performance_metrics::MoneyWeightedReturnCalculator::new(),
        risk_calculator: performance_calculator::calculations::risk_metrics::RiskMetricsCalculator::new(),
    };
    
    // Create an audit record
    let result = calculator.create_audit_record(
        "portfolio1",
        "portfolio",
        "calculate_performance",
        "system",
        "Calculated performance metrics for portfolio portfolio1",
    ).await;
    
    // Verify the result
    assert!(result.is_ok());
} 