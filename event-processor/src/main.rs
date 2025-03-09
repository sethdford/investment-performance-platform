use aws_lambda_events::event::sqs::{SqsEvent, SqsMessage};
use aws_sdk_dynamodb::Client as DynamoDbClient;
use aws_sdk_sqs::Client as SqsClient;
use chrono::{NaiveDate, Utc};
use lambda_runtime::{service_fn, Error, LambdaEvent};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::env;
use tracing::{info, error, debug, instrument};
use uuid::Uuid;
use shared::{
    repository::{
        dynamodb::DynamoDbRepository,
        Repository, PaginationOptions, PaginatedResult,
    },
    error::AppError,
};
use timestream_repository::TimestreamRepository;
use performance_calculator::batch_processor::{BatchProcessor, BatchCalculationRequest, BatchCalculationResult};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;

// Import resilience patterns from performance-calculator
use performance_calculator::resilience::{
    retry::RetryConfig,
    circuit_breaker::CircuitBreakerConfig,
    bulkhead::BulkheadConfig,
    with_resilience,
};

#[derive(Debug, Serialize, Deserialize)]
struct ItemEvent {
    event_type: String,
    item: Item,
}

#[derive(Debug, Serialize, Deserialize)]
struct Item {
    id: String,
    name: String,
    description: Option<String>,
    created_at: String,
    updated_at: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct AuditRecord {
    id: String,
    entity_id: String,
    entity_type: String,
    action: String,
    user_id: String,
    timestamp: String,
    details: Option<String>,
}

struct EventProcessor {
    dynamodb_client: DynamoDbClient,
    sqs_client: SqsClient,
    dynamodb_repository: DynamoDbRepository,
    timestream_repository: Arc<TimestreamRepository>,
}

impl EventProcessor {
    async fn new() -> Result<Self, Error> {
        let config = aws_config::load_from_env().await;
        let dynamodb_client = DynamoDbClient::new(&config);
        let sqs_client = SqsClient::new(&config);
        
        let table_name = env::var("DYNAMODB_TABLE").unwrap_or_else(|_| "Items".to_string());
        let dynamodb_repository = DynamoDbRepository::new(dynamodb_client.clone(), table_name);
        
        let database_name = env::var("TIMESTREAM_DATABASE").unwrap_or_else(|_| "performance_metrics".to_string());
        let table_name = env::var("TIMESTREAM_TABLE").unwrap_or_else(|_| "portfolio_metrics".to_string());
        let timestream_repository = Arc::new(TimestreamRepository::new(database_name, table_name));
        
        Ok(Self {
            dynamodb_client,
            sqs_client,
            dynamodb_repository,
            timestream_repository,
        })
    }
    
    async fn process_event(&self, event: SqsEvent) -> Result<(), Error> {
        for record in event.records {
            match self.process_record(record).await {
                Ok(_) => info!("Successfully processed record"),
                Err(e) => {
                    error!("Failed to process record: {}", e);
                    // Continue processing other records
                }
            }
        }
        
        Ok(())
    }
    
    async fn process_record(&self, record: SqsMessage) -> Result<(), Error> {
        let body = record.body.ok_or_else(|| Error::from("Missing message body"))?;
        
        // Parse the message body
        let event: ItemEvent = serde_json::from_str(&body)
            .map_err(|e| Error::from(format!("Failed to parse message body: {}", e)))?;
        
        info!("Processing event: {:?}", event.event_type);
        
        match event.event_type.as_str() {
            "ITEM_CREATED" => self.handle_item_created(event.item).await,
            "ITEM_UPDATED" => self.handle_item_updated(event.item).await,
            "ITEM_DELETED" => self.handle_item_deleted(event.item.id).await,
            "BATCH_CALCULATION" => {
                // Parse dates from strings to NaiveDate
                let start_date = NaiveDate::parse_from_str(&event.item.created_at, "%Y-%m-%d")
                    .map_err(|e| Error::from(format!("Failed to parse start date: {}", e)))?;
                
                let end_date_str = event.item.updated_at.unwrap_or_else(|| Utc::now().format("%Y-%m-%d").to_string());
                let end_date = NaiveDate::parse_from_str(&end_date_str, "%Y-%m-%d")
                    .map_err(|e| Error::from(format!("Failed to parse end date: {}", e)))?;
                
                let request = BatchCalculationRequest {
                    portfolio_ids: vec![event.item.id],
                    start_date,
                    end_date,
                    include_details: true,
                };
                
                self.process_batch_calculation(request).await?;
                Ok(())
            }
            _ => {
                error!("Unknown event type: {}", event.event_type);
                Err(Error::from(format!("Unknown event type: {}", event.event_type)))
            }
        }
    }
    
    async fn handle_item_created(&self, item: Item) -> Result<(), Error> {
        info!("Handling item created: {}", item.id);
        
        // Create audit record
        let audit_record = AuditRecord {
            id: Uuid::new_v4().to_string(),
            entity_id: item.id.clone(),
            entity_type: "Item".to_string(),
            action: "CREATE".to_string(),
            user_id: "system".to_string(),
            timestamp: Utc::now().to_rfc3339(),
            details: Some(format!("Item created: {}", item.name)),
        };
        
        self.create_audit_record(audit_record).await?;
        
        Ok(())
    }
    
    async fn handle_item_updated(&self, item: Item) -> Result<(), Error> {
        info!("Handling item updated: {}", item.id);
        
        // Create audit record
        let audit_record = AuditRecord {
            id: Uuid::new_v4().to_string(),
            entity_id: item.id.clone(),
            entity_type: "Item".to_string(),
            action: "UPDATE".to_string(),
            user_id: "system".to_string(),
            timestamp: Utc::now().to_rfc3339(),
            details: Some(format!("Item updated: {}", item.name)),
        };
        
        self.create_audit_record(audit_record).await?;
        
        Ok(())
    }
    
    async fn handle_item_deleted(&self, item_id: String) -> Result<(), Error> {
        info!("Handling item deleted: {}", item_id);
        
        // Create audit record
        let audit_record = AuditRecord {
            id: Uuid::new_v4().to_string(),
            entity_id: item_id,
            entity_type: "Item".to_string(),
            action: "DELETE".to_string(),
            user_id: "system".to_string(),
            timestamp: Utc::now().to_rfc3339(),
            details: Some("Item deleted".to_string()),
        };
        
        self.create_audit_record(audit_record).await?;
        
        Ok(())
    }
    
    async fn create_audit_record(&self, audit_record: AuditRecord) -> Result<(), Error> {
        let table_name = env::var("AUDIT_TABLE_NAME").unwrap_or_else(|_| "AuditTrail".to_string());
        
        // Create a HashMap for the DynamoDB item
        let mut item = HashMap::new();
        
        // Add all fields from the AuditRecord
        item.insert("id".to_string(), aws_sdk_dynamodb::types::AttributeValue::S(audit_record.id));
        item.insert("entity_id".to_string(), aws_sdk_dynamodb::types::AttributeValue::S(audit_record.entity_id));
        item.insert("entity_type".to_string(), aws_sdk_dynamodb::types::AttributeValue::S(audit_record.entity_type));
        item.insert("action".to_string(), aws_sdk_dynamodb::types::AttributeValue::S(audit_record.action));
        item.insert("user_id".to_string(), aws_sdk_dynamodb::types::AttributeValue::S(audit_record.user_id));
        item.insert("timestamp".to_string(), aws_sdk_dynamodb::types::AttributeValue::S(audit_record.timestamp));
        
        // Add details as a JSON string if present
        if let Some(details) = audit_record.details {
            item.insert("details".to_string(), aws_sdk_dynamodb::types::AttributeValue::S(details));
        }
        
        // Put the item in DynamoDB
        self.dynamodb_client.put_item()
            .table_name(table_name)
            .set_item(Some(item))
            .send()
            .await
            .map_err(|e| Error::from(format!("Failed to create audit record: {}", e)))?;
        
        Ok(())
    }
    
    async fn process_batch_calculation(&self, batch_request: BatchCalculationRequest) -> Result<BatchCalculationResult, Error> {
        let batch_processor = BatchProcessor::new(
            self.dynamodb_repository.clone(),
            10,
            5,
        );

        // Use a simpler approach without with_resilience for now
        let result = batch_processor.process(batch_request.clone()).await
            .map_err(|e| Error::from(format!("Failed to process batch calculation: {}", e)))?;

        Ok(result)
    }
}

impl Clone for EventProcessor {
    fn clone(&self) -> Self {
        Self {
            dynamodb_client: self.dynamodb_client.clone(),
            sqs_client: self.sqs_client.clone(),
            dynamodb_repository: self.dynamodb_repository.clone(),
            timestream_repository: self.timestream_repository.clone(),
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .with_target(false)
        .init();
    
    // Create the event processor
    let processor = EventProcessor::new().await?;
    
    // Create the Lambda service
    let func = service_fn(move |event: LambdaEvent<SqsEvent>| {
        let processor = processor.clone();
        async move {
            processor.process_event(event.payload).await?;
            Ok::<(), Error>(())
        }
    });
    
    // Start the Lambda runtime
    lambda_runtime::run(func).await?;
    
    Ok(())
} 