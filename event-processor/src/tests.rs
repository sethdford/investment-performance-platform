#[cfg(test)]
mod tests {
    use aws_lambda_events::sqs::{SqsEvent, SqsMessage};
    use lambda_runtime::LambdaEvent;
    use shared::{
        models::{Item, ItemEvent, ItemEventType},
        repository::DynamoDbRepository,
    };
    use chrono::{DateTime, Utc};
    use std::sync::{Arc, Mutex};
    use std::collections::HashMap;
    use aws_sdk_dynamodb::Client as DynamoDbClient;
    use aws_sdk_sqs::Client as SqsClient;
    use serde_json::json;
    use tokio::sync::Mutex as AsyncMutex;

    // Mock DynamoDB repository
    struct MockDynamoDbRepository {
        items: Arc<Mutex<HashMap<String, Item>>>,
        processed_events: Arc<Mutex<Vec<ItemEvent>>>,
    }

    impl MockDynamoDbRepository {
        fn new() -> Self {
            Self {
                items: Arc::new(Mutex::new(HashMap::new())),
                processed_events: Arc::new(Mutex::new(Vec::new())),
            }
        }

        async fn create_item(&self, item: &Item) -> Result<(), aws_sdk_dynamodb::Error> {
            let mut items = self.items.lock().unwrap();
            items.insert(item.id.clone(), item.clone());
            Ok(())
        }

        async fn get_item(&self, id: &str) -> Result<Option<Item>, aws_sdk_dynamodb::Error> {
            let items = self.items.lock().unwrap();
            Ok(items.get(id).cloned())
        }

        async fn list_items(&self) -> Result<Vec<Item>, aws_sdk_dynamodb::Error> {
            let items = self.items.lock().unwrap();
            Ok(items.values().cloned().collect())
        }

        async fn delete_item(&self, id: &str) -> Result<(), aws_sdk_dynamodb::Error> {
            let mut items = self.items.lock().unwrap();
            items.remove(id);
            Ok(())
        }

        fn record_processed_event(&self, event: ItemEvent) {
            let mut events = self.processed_events.lock().unwrap();
            events.push(event);
        }

        fn get_processed_events(&self) -> Vec<ItemEvent> {
            let events = self.processed_events.lock().unwrap();
            events.clone()
        }
    }

    // Helper function to create a test item
    fn create_test_item() -> Item {
        Item {
            id: "test-id".to_string(),
            name: "Test Item".to_string(),
            description: Some("Test Description".to_string()),
            created_at: DateTime::parse_from_rfc3339("2023-01-01T00:00:00Z")
                .unwrap()
                .with_timezone(&Utc),
        }
    }

    // Helper function to create a test SQS event
    fn create_test_sqs_event(event_type: ItemEventType, item: Item) -> SqsEvent {
        let item_event = ItemEvent {
            event_type,
            item,
            timestamp: Utc::now(),
        };

        let event_json = serde_json::to_string(&item_event).unwrap();

        SqsEvent {
            records: vec![SqsMessage {
                message_id: Some("test-message-id".to_string()),
                receipt_handle: Some("test-receipt-handle".to_string()),
                body: Some(event_json),
                md5_of_body: Some("test-md5".to_string()),
                md5_of_message_attributes: None,
                attributes: Default::default(),
                message_attributes: Default::default(),
                event_source_arn: Some("arn:aws:sqs:us-east-1:123456789012:test-queue".to_string()),
                event_source: Some("aws:sqs".to_string()),
                aws_region: Some("us-east-1".to_string()),
            }],
        }
    }

    // Mock DynamoDB client for testing
    struct MockDynamoDbClient {
        items: Arc<AsyncMutex<HashMap<String, serde_json::Value>>>,
    }

    impl MockDynamoDbClient {
        fn new() -> Self {
            Self {
                items: Arc::new(AsyncMutex::new(HashMap::new())),
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
        messages: Arc<AsyncMutex<Vec<String>>>,
    }

    impl MockSqsClient {
        fn new() -> Self {
            Self {
                messages: Arc::new(AsyncMutex::new(Vec::new())),
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
    async fn test_process_item_created_event() {
        // Create mock clients
        let dynamodb_client = DynamoDbClient::new(&aws_config::from_env().region("us-east-1").build());
        let sqs_client = SqsClient::new(&aws_config::from_env().region("us-east-1").build());
        
        // Create event processor
        let processor = EventProcessor {
            dynamodb_client,
            sqs_client,
            table_name: "test-table".to_string(),
            performance_queue_url: None,
        };
        
        // Create a mock item event
        let item = Item {
            id: "test-item-1".to_string(),
            name: "Test Item".to_string(),
            description: Some("This is a test item".to_string()),
            created_at: "2023-01-01T00:00:00Z".to_string(),
            updated_at: None,
        };
        
        let item_event = ItemEvent {
            event_type: "ITEM_CREATED".to_string(),
            item: item.clone(),
        };
        
        let event_json = serde_json::to_string(&item_event).unwrap();
        let sqs_event = create_mock_sqs_event(&event_json);
        
        // Process the event
        let result = processor.process_event(sqs_event).await;
        
        // Verify the result
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_process_portfolio_created_event() {
        // Create mock clients
        let dynamodb_client = DynamoDbClient::new(&aws_config::from_env().region("us-east-1").build());
        let sqs_client = SqsClient::new(&aws_config::from_env().region("us-east-1").build());
        
        // Create event processor
        let processor = EventProcessor {
            dynamodb_client,
            sqs_client,
            table_name: "test-table".to_string(),
            performance_queue_url: None,
        };
        
        // Create a mock portfolio event
        let portfolio = Portfolio {
            id: "test-portfolio-1".to_string(),
            name: "Test Portfolio".to_string(),
            client_id: "test-client-1".to_string(),
            created_at: "2023-01-01T00:00:00Z".to_string(),
            updated_at: None,
        };
        
        let portfolio_event = PortfolioEvent {
            event_type: "PORTFOLIO_CREATED".to_string(),
            portfolio: portfolio.clone(),
        };
        
        let event_json = serde_json::to_string(&portfolio_event).unwrap();
        let sqs_event = create_mock_sqs_event(&event_json);
        
        // Process the event
        let result = processor.process_event(sqs_event).await;
        
        // Verify the result
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_process_transaction_created_event() {
        // Create mock clients
        let dynamodb_client = DynamoDbClient::new(&aws_config::from_env().region("us-east-1").build());
        let sqs_client = SqsClient::new(&aws_config::from_env().region("us-east-1").build());
        
        // Create event processor
        let processor = EventProcessor {
            dynamodb_client,
            sqs_client,
            table_name: "test-table".to_string(),
            performance_queue_url: Some("test-queue-url".to_string()),
        };
        
        // Create a mock transaction event
        let transaction = Transaction {
            id: "test-transaction-1".to_string(),
            account_id: "test-account-1".to_string(),
            security_id: Some("test-security-1".to_string()),
            transaction_type: "BUY".to_string(),
            amount: 1000.0,
            quantity: Some(10.0),
            price: Some(100.0),
            transaction_date: "2023-01-01T00:00:00Z".to_string(),
            created_at: "2023-01-01T00:00:00Z".to_string(),
            updated_at: None,
        };
        
        let transaction_event = TransactionEvent {
            event_type: "TRANSACTION_CREATED".to_string(),
            transaction: transaction.clone(),
        };
        
        let event_json = serde_json::to_string(&transaction_event).unwrap();
        let sqs_event = create_mock_sqs_event(&event_json);
        
        // Process the event
        let result = processor.process_event(sqs_event).await;
        
        // Verify the result
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_process_unknown_event() {
        // Create mock clients
        let dynamodb_client = DynamoDbClient::new(&aws_config::from_env().region("us-east-1").build());
        let sqs_client = SqsClient::new(&aws_config::from_env().region("us-east-1").build());
        
        // Create event processor
        let processor = EventProcessor {
            dynamodb_client,
            sqs_client,
            table_name: "test-table".to_string(),
            performance_queue_url: None,
        };
        
        // Create a mock unknown event
        let unknown_event = json!({
            "event_type": "UNKNOWN_EVENT",
            "data": {
                "id": "test-data-1",
                "name": "Test Data"
            }
        });
        
        let event_json = unknown_event.to_string();
        let sqs_event = create_mock_sqs_event(&event_json);
        
        // Process the event
        let result = processor.process_event(sqs_event).await;
        
        // Verify the result
        assert!(result.is_ok());
    }

    // Tests would go here, but they require access to the handler functions
    // which are not easily testable without refactoring the main.rs file
    // to separate the handler logic from the AWS Lambda runtime setup.
    //
    // In a real-world scenario, we would:
    // 1. Refactor the handler functions to be more testable
    // 2. Create unit tests for each event type
    // 3. Use the mock repository for testing
} 