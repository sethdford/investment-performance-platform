use aws_lambda_events::event::sqs::{SqsEvent, SqsMessage};
use aws_sdk_dynamodb::Client as DynamoDbClient;
use aws_sdk_sqs::Client as SqsClient;
use chrono::Utc;
use lambda_runtime::{service_fn, Error, LambdaEvent};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::env;
use tracing::{info, error, warn, debug, instrument};
use uuid::Uuid;
use shared::{
    logging,
    repository::{
        dynamodb::DynamoDbRepository,
        Repository,
    },
    metrics::performance::PerformanceMetricsCollector,
    resilience::{
        retry::RetryConfig,
        circuit_breaker::CircuitBreakerConfig,
        bulkhead::BulkheadConfig,
        with_resilience,
    },
};
use timestream_repository::TimestreamRepository;
use performance_calculator::batch_processor::{BatchProcessor, BatchCalculationRequest};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;

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
struct PortfolioEvent {
    event_type: String,
    portfolio: Portfolio,
}

#[derive(Debug, Serialize, Deserialize)]
struct Portfolio {
    id: String,
    name: String,
    client_id: String,
    inception_date: String,
    benchmark_id: Option<String>,
    created_at: String,
    updated_at: Option<String>,
    status: String,
    metadata: HashMap<String, String>,
    holdings: Vec<Holding>,
    transactions: Vec<Transaction>,
}

#[derive(Debug, Serialize, Deserialize)]
struct Holding {
    symbol: String,
    quantity: f64,
    cost_basis: Option<f64>,
    currency: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct TransactionEvent {
    event_type: String,
    transaction: Transaction,
}

#[derive(Debug, Serialize, Deserialize)]
struct Transaction {
    id: String,
    account_id: String,
    security_id: Option<String>,
    transaction_type: String,
    amount: f64,
    quantity: Option<f64>,
    price: Option<f64>,
    transaction_date: String,
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
    details: String,
}

/// Event type
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
enum EventType {
    /// Item created
    ItemCreated,
    /// Item updated
    ItemUpdated,
    /// Item deleted
    ItemDeleted,
    /// Portfolio created
    PortfolioCreated,
    /// Portfolio updated
    PortfolioUpdated,
    /// Portfolio deleted
    PortfolioDeleted,
    /// Transaction created
    TransactionCreated,
    /// Transaction updated
    TransactionUpdated,
    /// Transaction deleted
    TransactionDeleted,
    /// Performance calculation requested
    PerformanceCalculationRequested,
}

/// Event payload
#[derive(Debug, Clone, Serialize, Deserialize)]
struct EventPayload {
    /// Event type
    event_type: EventType,
    /// Tenant ID
    tenant_id: String,
    /// Entity ID
    entity_id: String,
    /// Portfolio ID (optional)
    portfolio_id: Option<String>,
    /// Item ID (optional)
    item_id: Option<String>,
    /// Transaction ID (optional)
    transaction_id: Option<String>,
    /// Timestamp
    timestamp: String,
    /// Additional data
    data: Option<Value>,
}

/// Event processor
#[derive(Clone)]
struct EventProcessor {
    dynamodb_client: DynamoDbClient,
    sqs_client: SqsClient,
    table_name: String,
    performance_queue_url: Option<String>,
    dynamodb_repository: DynamoDbRepository,
    timestream_repository: TimestreamRepository,
    metrics_collector: PerformanceMetricsCollector,
}

impl EventProcessor {
    /// Create a new event processor
    async fn new() -> Result<Self, Error> {
        // Initialize logging
        logging::init_lambda_logging();
        
        // Load AWS configuration
        let config = aws_config::load_from_env().await;
        
        // Create AWS clients
        let dynamodb_client = DynamoDbClient::new(&config);
        let sqs_client = SqsClient::new(&config);
        
        // Get environment variables
        let table_name = env::var("TABLE_NAME")
            .map_err(|_| Error::from("TABLE_NAME environment variable not set"))?;
            
        let performance_queue_url = env::var("PERFORMANCE_QUEUE_URL").ok();
        
        // Initialize repositories
        let dynamodb_repository = DynamoDbRepository::from_env().await?;
        let timestream_repository = TimestreamRepository::from_env().await?;
        
        // Initialize metrics collector
        let metrics_collector = PerformanceMetricsCollector::new(true);
        
        Ok(Self {
            dynamodb_client,
            sqs_client,
            table_name,
            performance_queue_url,
            dynamodb_repository,
            timestream_repository,
            metrics_collector,
        })
    }
    
    /// Process SQS event
    #[instrument(skip(self, event), fields(event_count = event.records.len()))]
    async fn process_event(&self, event: SqsEvent) -> Result<(), Error> {
        info!("Processing {} SQS messages", event.records.len());
        
        // Track affected portfolios for performance recalculation
        let mut affected_portfolios: HashSet<(String, String)> = HashSet::new();
        
        // Process each message
        for record in event.records {
            if let Err(e) = self.process_message(&record, &mut affected_portfolios).await {
                error!("Failed to process message: {}", e);
                // Continue processing other messages
            }
        }
        
        // Trigger performance calculations for affected portfolios
        if !affected_portfolios.is_empty() {
            info!("Triggering performance calculations for {} affected portfolios", affected_portfolios.len());
            
            // Group portfolios by tenant
            let mut tenant_portfolios: HashMap<String, Vec<String>> = HashMap::new();
            
            for (tenant_id, portfolio_id) in affected_portfolios {
                tenant_portfolios
                    .entry(tenant_id)
                    .or_insert_with(Vec::new)
                    .push(portfolio_id);
            }
            
            // Process each tenant's portfolios
            for (tenant_id, portfolio_ids) in tenant_portfolios {
                if let Err(e) = self.calculate_portfolio_performance(&tenant_id, &portfolio_ids).await {
                    error!("Failed to calculate performance for tenant {}: {}", tenant_id, e);
                }
            }
        }
        
        // Log metrics summary
        self.metrics_collector.log_summary();
        
        Ok(())
    }
    
    /// Process SQS message
    #[instrument(skip(self, affected_portfolios))]
    async fn process_message(&self, message: &SqsMessage, affected_portfolios: &mut HashSet<(String, String)>) -> Result<(), Error> {
        // Parse message body
        let body = message.body.as_ref()
            .ok_or_else(|| Error::from("Message body is empty"))?;
            
        let payload: EventPayload = serde_json::from_str(body)
            .map_err(|e| Error::from(format!("Failed to parse message body: {}", e)))?;
            
        debug!("Processing event: {:?}", payload.event_type);
        
        // Process based on event type
        match payload.event_type {
            EventType::ItemCreated | EventType::ItemUpdated | EventType::ItemDeleted => {
                if let Some(portfolio_id) = &payload.portfolio_id {
                    affected_portfolios.insert((payload.tenant_id.clone(), portfolio_id.clone()));
                }
            },
            EventType::PortfolioCreated | EventType::PortfolioUpdated => {
                affected_portfolios.insert((payload.tenant_id.clone(), payload.entity_id.clone()));
            },
            EventType::PortfolioDeleted => {
                // No need to calculate performance for deleted portfolios
            },
            EventType::TransactionCreated | EventType::TransactionUpdated | EventType::TransactionDeleted => {
                if let Some(portfolio_id) = &payload.portfolio_id {
                    affected_portfolios.insert((payload.tenant_id.clone(), portfolio_id.clone()));
                }
            },
            EventType::PerformanceCalculationRequested => {
                // Direct performance calculation request
                if let Some(data) = &payload.data {
                    if let Some(portfolio_ids) = data.get("portfolio_ids").and_then(|v| v.as_array()) {
                        for portfolio_id_value in portfolio_ids {
                            if let Some(portfolio_id) = portfolio_id_value.as_str() {
                                affected_portfolios.insert((payload.tenant_id.clone(), portfolio_id.to_string()));
                            }
                        }
                    } else {
                        // Single portfolio
                        affected_portfolios.insert((payload.tenant_id.clone(), payload.entity_id.clone()));
                    }
                } else {
                    // Single portfolio
                    affected_portfolios.insert((payload.tenant_id.clone(), payload.entity_id.clone()));
                }
            },
        }
        
        // Create audit record
        self.create_audit_record(&payload).await?;
        
        Ok(())
    }
    
    /// Calculate portfolio performance
    #[instrument(skip(self), fields(portfolio_count = portfolio_ids.len()))]
    async fn calculate_portfolio_performance(&self, tenant_id: &str, portfolio_ids: &[String]) -> Result<(), Error> {
        info!("Calculating performance for {} portfolios in tenant {}", portfolio_ids.len(), tenant_id);
        
        // Create batch processor
        let batch_processor = BatchProcessor::new(
            self.dynamodb_repository.clone(),
            10, // max_batch_size
            4,  // max_concurrency
        );
        
        // Get calculation date range
        let end_date = Utc::now().date_naive();
        let start_date = end_date.checked_sub_months(chrono::Months::new(12))
            .unwrap_or_else(|| end_date.checked_sub_days(chrono::Days::new(30)).unwrap_or(end_date));
        
        // Create batch calculation request
        let batch_request = BatchCalculationRequest {
            portfolio_ids: portfolio_ids.to_vec(),
            start_date,
            end_date,
            include_details: true,
        };
        
        // Process batch calculation with resilience patterns
        let retry_config = RetryConfig {
            max_retries: 3,
            initial_backoff_ms: 100,
            max_backoff_ms: 1000,
            backoff_multiplier: 2.0,
            jitter_factor: 0.1,
        };
        
        let circuit_breaker_config = CircuitBreakerConfig {
            failure_threshold: 50.0,
            minimum_requests: 5,
            reset_timeout_ms: 5000,
            window_size: 10,
        };
        
        let bulkhead_config = BulkheadConfig {
            max_concurrent_requests: 10,
            max_queue_size: 10,
        };
        
        let result = with_resilience(
            "batch_calculation",
            retry_config,
            circuit_breaker_config,
            bulkhead_config,
            || async {
                self.metrics_collector.measure_async("batch_calculation", || async {
                    batch_processor.process(batch_request.clone()).await
                }).await
            },
        ).await.map_err(|e| Error::from(format!("Failed to process batch calculation: {}", e)))?;
        
        // Store performance data in Timestream
        for (portfolio_id, calculation_result) in &result.results {
            match calculation_result {
                Ok(result) => {
                    // Store overall performance
                    let performance_data = timestream_repository::PerformanceDataPoint {
                        portfolio_id: portfolio_id.clone(),
                        timestamp: Utc::now(),
                        twr: result.twr,
                        mwr: result.mwr,
                        volatility: Some(result.volatility),
                        sharpe_ratio: Some(result.sharpe_ratio),
                        max_drawdown: Some(result.max_drawdown),
                        benchmark_id: result.benchmark_id.clone(),
                        benchmark_return: result.benchmark_return,
                        tracking_error: result.tracking_error,
                        information_ratio: result.information_ratio,
                    };
                    
                    if let Err(e) = self.timestream_repository.store_performance_data(&performance_data).await {
                        error!("Failed to store performance data for portfolio {}: {}", portfolio_id, e);
                    }
                    
                    // Store detailed performance if available
                    if let Some(details) = &result.details {
                        for detail in &details.time_series {
                            let detail_data = timestream_repository::PerformanceDataPoint {
                                portfolio_id: portfolio_id.clone(),
                                timestamp: chrono::DateTime::<Utc>::from_naive_utc_and_offset(
                                    chrono::NaiveDateTime::new(detail.date, chrono::NaiveTime::from_hms_opt(0, 0, 0).unwrap()),
                                    Utc,
                                ),
                                twr: detail.twr,
                                mwr: detail.mwr,
                                volatility: Some(detail.volatility),
                                sharpe_ratio: None,
                                max_drawdown: None,
                                benchmark_id: result.benchmark_id.clone(),
                                benchmark_return: detail.benchmark_return,
                                tracking_error: None,
                                information_ratio: None,
                            };
                            
                            if let Err(e) = self.timestream_repository.store_performance_data(&detail_data).await {
                                error!("Failed to store detailed performance data for portfolio {}: {}", portfolio_id, e);
                            }
                        }
                    }
                },
                Err(e) => {
                    error!("Failed to calculate performance for portfolio {}: {}", portfolio_id, e);
                }
            }
        }
        
        info!("Completed performance calculations for {} portfolios in tenant {}", portfolio_ids.len(), tenant_id);
        
        Ok(())
    }
    
    /// Create audit record
    async fn create_audit_record(&self, payload: &EventPayload) -> Result<(), Error> {
        let audit_id = Uuid::new_v4().to_string();
        
        let entity_type = match payload.event_type {
            EventType::ItemCreated | EventType::ItemUpdated | EventType::ItemDeleted => "item",
            EventType::PortfolioCreated | EventType::PortfolioUpdated | EventType::PortfolioDeleted => "portfolio",
            EventType::TransactionCreated | EventType::TransactionUpdated | EventType::TransactionDeleted => "transaction",
            EventType::PerformanceCalculationRequested => "performance_calculation",
        };
        
        let action = match payload.event_type {
            EventType::ItemCreated | EventType::PortfolioCreated | EventType::TransactionCreated => "created",
            EventType::ItemUpdated | EventType::PortfolioUpdated | EventType::TransactionUpdated => "updated",
            EventType::ItemDeleted | EventType::PortfolioDeleted | EventType::TransactionDeleted => "deleted",
            EventType::PerformanceCalculationRequested => "requested",
        };
        
        let audit_record = AuditRecord {
            id: audit_id,
            entity_id: payload.entity_id.clone(),
            entity_type: entity_type.to_string(),
            action: action.to_string(),
            user_id: payload.data.as_ref()
                .and_then(|d| d.get("user_id"))
                .and_then(|u| u.as_str())
                .unwrap_or("system")
                .to_string(),
            timestamp: payload.timestamp.clone(),
            details: serde_json::to_string(&payload)
                .unwrap_or_else(|_| "{}".to_string()),
        };
        
        // Convert to DynamoDB item
        let item = aws_sdk_dynamodb::types::AttributeValue::M(
            serde_dynamo::to_item(&audit_record)
                .map_err(|e| Error::from(format!("Failed to convert audit record to DynamoDB item: {}", e)))?
        );
        
        // Store in DynamoDB
        self.dynamodb_client
            .put_item()
            .table_name(&self.table_name)
            .item("PK", aws_sdk_dynamodb::types::AttributeValue::S(format!("AUDIT#{}", audit_record.id)))
            .item("SK", aws_sdk_dynamodb::types::AttributeValue::S(format!("AUDIT#{}", audit_record.timestamp)))
            .item("GSI1PK", aws_sdk_dynamodb::types::AttributeValue::S(format!("ENTITY#{}", audit_record.entity_type)))
            .item("GSI1SK", aws_sdk_dynamodb::types::AttributeValue::S(format!("ENTITY#{}#{}", audit_record.entity_id, audit_record.timestamp)))
            .item("Data", item)
            .send()
            .await
            .map_err(|e| Error::from(format!("Failed to store audit record: {}", e)))?;
            
        Ok(())
    }
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    // Create event processor
    let processor = EventProcessor::new().await?;
    
    // Create Lambda handler
    let handler_fn = service_fn(move |event: LambdaEvent<SqsEvent>| {
        let processor = processor.clone();
        async move {
            processor.process_event(event.payload).await?;
            Ok::<(), Error>(())
        }
    });
    
    // Run Lambda handler
    lambda_runtime::run(handler_fn).await?;
    
    Ok(())
} 