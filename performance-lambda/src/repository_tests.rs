#[cfg(test)]
mod dynamodb_repository_tests {
    use crate::dynamodb_repository::{Portfolio, Account, Transaction, Valuation, Benchmark, BenchmarkReturn};
    use aws_sdk_dynamodb::{Client as DynamoDbClient, Error as DynamoDbError};
    use aws_sdk_dynamodb::model::{AttributeValue, GetItemOutput, QueryOutput, PutItemOutput};
    use aws_smithy_runtime::client::http::test_util::{ReplayEvent, StaticReplayClient};
    use aws_smithy_types::body::SdkBody;
    use aws_smithy_types::byte_stream::ByteStream;
    use chrono::{DateTime, Utc};
    use serde_json::{json, Value};
    use std::collections::HashMap;
    use std::sync::Arc;
    use tokio::sync::Mutex;
    
    // Mock DynamoDB client for testing
    struct MockDynamoDbClient {
        get_item_responses: Arc<Mutex<HashMap<String, Result<GetItemOutput, DynamoDbError>>>>,
        query_responses: Arc<Mutex<HashMap<String, Result<QueryOutput, DynamoDbError>>>>,
        put_item_responses: Arc<Mutex<HashMap<String, Result<PutItemOutput, DynamoDbError>>>>,
    }
    
    impl MockDynamoDbClient {
        fn new() -> Self {
            Self {
                get_item_responses: Arc::new(Mutex::new(HashMap::new())),
                query_responses: Arc::new(Mutex::new(HashMap::new())),
                put_item_responses: Arc::new(Mutex::new(HashMap::new())),
            }
        }
        
        async fn add_get_item_response(&self, key: String, response: Result<GetItemOutput, DynamoDbError>) {
            let mut responses = self.get_item_responses.lock().await;
            responses.insert(key, response);
        }
        
        async fn add_query_response(&self, key: String, response: Result<QueryOutput, DynamoDbError>) {
            let mut responses = self.query_responses.lock().await;
            responses.insert(key, response);
        }
        
        async fn add_put_item_response(&self, key: String, response: Result<PutItemOutput, DynamoDbError>) {
            let mut responses = self.put_item_responses.lock().await;
            responses.insert(key, response);
        }
    }
    
    // Helper function to create a test portfolio
    fn create_test_portfolio(id: &str, tenant_id: &str, user_id: &str) -> Portfolio {
        Portfolio {
            id: id.to_string(),
            name: format!("Portfolio {}", id),
            description: Some(format!("Test portfolio {}", id)),
            tenant_id: tenant_id.to_string(),
            user_id: user_id.to_string(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
            currency: "USD".to_string(),
            status: "ACTIVE".to_string(),
        }
    }
    
    // Helper function to create a test account
    fn create_test_account(id: &str, portfolio_id: &str) -> Account {
        Account {
            id: id.to_string(),
            portfolio_id: portfolio_id.to_string(),
            name: format!("Account {}", id),
            description: Some(format!("Test account {}", id)),
            account_type: "INVESTMENT".to_string(),
            currency: "USD".to_string(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
            status: "ACTIVE".to_string(),
        }
    }
    
    // Helper function to create a test transaction
    fn create_test_transaction(
        id: &str,
        account_id: &str,
        portfolio_id: &str,
        transaction_type: &str,
        date: DateTime<Utc>,
        amount: f64,
    ) -> Transaction {
        Transaction {
            id: id.to_string(),
            account_id: account_id.to_string(),
            portfolio_id: portfolio_id.to_string(),
            transaction_type: transaction_type.to_string(),
            transaction_date: date,
            settlement_date: Some(date),
            amount,
            currency: "USD".to_string(),
            security_id: None,
            quantity: None,
            price: None,
            fees: None,
            taxes: None,
            notes: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }
    
    // Helper function to create a test valuation
    fn create_test_valuation(
        id: &str,
        portfolio_id: &str,
        date: DateTime<Utc>,
        value: f64,
    ) -> Valuation {
        Valuation {
            id: id.to_string(),
            portfolio_id: portfolio_id.to_string(),
            date,
            value,
            cash_balance: value * 0.1, // 10% cash for testing
            currency: "USD".to_string(),
            created_at: Utc::now(),
        }
    }
    
    // Helper function to create a test benchmark
    fn create_test_benchmark(id: &str) -> Benchmark {
        Benchmark {
            id: id.to_string(),
            name: format!("Benchmark {}", id),
            description: Some(format!("Test benchmark {}", id)),
            currency: "USD".to_string(),
            provider: "TEST".to_string(),
            ticker: Some(format!("TEST{}", id)),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }
    
    // Helper function to create a test benchmark return
    fn create_test_benchmark_return(
        id: &str,
        benchmark_id: &str,
        date: DateTime<Utc>,
        return_value: f64,
    ) -> BenchmarkReturn {
        BenchmarkReturn {
            id: id.to_string(),
            benchmark_id: benchmark_id.to_string(),
            date,
            return_value,
            created_at: Utc::now(),
        }
    }
    
    // Helper function to convert a struct to DynamoDB item
    fn to_dynamodb_item<T: serde::Serialize>(item: &T) -> HashMap<String, AttributeValue> {
        let json = serde_json::to_value(item).unwrap();
        json_to_dynamodb_item(&json)
    }
    
    // Helper function to convert JSON to DynamoDB item
    fn json_to_dynamodb_item(json: &Value) -> HashMap<String, AttributeValue> {
        let mut item = HashMap::new();
        
        match json {
            Value::Object(map) => {
                for (key, value) in map {
                    item.insert(key.clone(), json_value_to_attribute_value(value));
                }
            },
            _ => panic!("Expected JSON object"),
        }
        
        item
    }
    
    // Helper function to convert JSON value to AttributeValue
    fn json_value_to_attribute_value(value: &Value) -> AttributeValue {
        match value {
            Value::Null => AttributeValue::Null(true),
            Value::Bool(b) => AttributeValue::Bool(*b),
            Value::Number(n) => {
                if n.is_i64() {
                    AttributeValue::N(n.as_i64().unwrap().to_string())
                } else if n.is_u64() {
                    AttributeValue::N(n.as_u64().unwrap().to_string())
                } else {
                    AttributeValue::N(n.as_f64().unwrap().to_string())
                }
            },
            Value::String(s) => AttributeValue::S(s.clone()),
            Value::Array(arr) => {
                let values: Vec<AttributeValue> = arr.iter()
                    .map(|v| json_value_to_attribute_value(v))
                    .collect();
                AttributeValue::L(values)
            },
            Value::Object(map) => {
                let mut m = HashMap::new();
                for (k, v) in map {
                    m.insert(k.clone(), json_value_to_attribute_value(v));
                }
                AttributeValue::M(m)
            },
        }
    }
    
    #[tokio::test]
    async fn test_get_portfolio() {
        // Create a mock DynamoDB client
        let mock_client = MockDynamoDbClient::new();
        
        // Create a test portfolio
        let portfolio_id = "test-portfolio-1";
        let tenant_id = "test-tenant-1";
        let user_id = "test-user-1";
        let portfolio = create_test_portfolio(portfolio_id, tenant_id, user_id);
        
        // Convert portfolio to DynamoDB item
        let item = to_dynamodb_item(&portfolio);
        
        // Add a mock response for get_item
        let get_item_output = GetItemOutput::builder()
            .item(item)
            .build();
        
        mock_client.add_get_item_response(
            format!("portfolio:{}", portfolio_id),
            Ok(get_item_output)
        ).await;
        
        // Create a DynamoDbRepository with the mock client
        // Note: This is a simplified version - in a real test, you would need to create a proper
        // DynamoDbClient with the mock HTTP client
        
        // Test the get_portfolio method
        // This would be implemented with the actual repository once we have the proper mock setup
        // let repository = DynamoDbRepository::new(client, "test-table".to_string());
        // let result = repository.get_portfolio(portfolio_id).await;
        
        // Assert the result matches the expected portfolio
        // assert!(result.is_ok());
        // let retrieved_portfolio = result.unwrap().unwrap();
        // assert_eq!(retrieved_portfolio.id, portfolio.id);
        // assert_eq!(retrieved_portfolio.name, portfolio.name);
        // assert_eq!(retrieved_portfolio.tenant_id, portfolio.tenant_id);
    }
    
    #[tokio::test]
    async fn test_list_accounts_by_portfolio() {
        // Similar to test_get_portfolio, but for listing accounts
        // This would test the query functionality
    }
    
    #[tokio::test]
    async fn test_save_performance_result() {
        // Test saving a performance calculation result
        // This would test the put_item functionality
    }
    
    #[tokio::test]
    async fn test_resilience_features() {
        // Test the resilience features like circuit breakers and retries
        // This would involve simulating failures and ensuring the repository handles them correctly
    }
} 