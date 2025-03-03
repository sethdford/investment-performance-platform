use aws_sdk_dynamodb::{Client as DynamoDbClient, Error as DynamoDbError};
use aws_sdk_sqs::{Client as SqsClient, Error as SqsError};
use aws_config::meta::region::RegionProviderChain;
use aws_config::BehaviorVersion;
use chrono::{DateTime, Utc, TimeZone, Duration};
use serde_json::json;
use std::env;
use std::time::Instant;
use tokio::time::sleep;
use uuid::Uuid;
use std::sync::Arc;
use tokio::sync::Mutex;
use rand::{Rng, thread_rng};
use std::collections::HashMap;

// Structure to represent a chaos test configuration
struct ChaosTestConfig {
    // Failure rates (0.0 - 1.0)
    dynamodb_failure_rate: f64,
    sqs_failure_rate: f64,
    
    // Latency settings (in milliseconds)
    min_latency_ms: u64,
    max_latency_ms: u64,
    
    // Test duration in seconds
    test_duration_seconds: u64,
}

// Mock DynamoDB client that introduces chaos
struct ChaosDynamoDbClient {
    inner_client: DynamoDbClient,
    config: Arc<ChaosTestConfig>,
    failure_counter: Arc<Mutex<HashMap<String, usize>>>,
}

impl ChaosDynamoDbClient {
    fn new(client: DynamoDbClient, config: Arc<ChaosTestConfig>) -> Self {
        Self {
            inner_client: client,
            config,
            failure_counter: Arc::new(Mutex::new(HashMap::new())),
        }
    }
    
    async fn maybe_fail(&self, operation: &str) -> Result<(), DynamoDbError> {
        // Record operation
        {
            let mut counter = self.failure_counter.lock().await;
            *counter.entry(operation.to_string()).or_insert(0) += 1;
        }
        
        // Introduce random latency
        let latency = thread_rng().gen_range(
            self.config.min_latency_ms..=self.config.max_latency_ms
        );
        sleep(std::time::Duration::from_millis(latency)).await;
        
        // Maybe fail the operation
        if thread_rng().gen::<f64>() < self.config.dynamodb_failure_rate {
            return Err(DynamoDbError::Unhandled(Box::new(std::io::Error::new(
                std::io::ErrorKind::ConnectionReset,
                "Chaos-induced failure",
            ))));
        }
        
        Ok(())
    }
    
    // Example of a wrapped DynamoDB operation
    async fn put_item(
        &self,
        table_name: &str,
        item: HashMap<String, aws_sdk_dynamodb::model::AttributeValue>,
    ) -> Result<(), DynamoDbError> {
        // Maybe introduce chaos
        if let Err(e) = self.maybe_fail("put_item").await {
            return Err(e);
        }
        
        // Perform the actual operation
        self.inner_client.put_item()
            .table_name(table_name)
            .set_item(Some(item))
            .send()
            .await?;
            
        Ok(())
    }
    
    // Add more wrapped operations as needed
}

// Mock SQS client that introduces chaos
struct ChaosSqsClient {
    inner_client: SqsClient,
    config: Arc<ChaosTestConfig>,
    failure_counter: Arc<Mutex<HashMap<String, usize>>>,
}

impl ChaosSqsClient {
    fn new(client: SqsClient, config: Arc<ChaosTestConfig>) -> Self {
        Self {
            inner_client: client,
            config,
            failure_counter: Arc::new(Mutex::new(HashMap::new())),
        }
    }
    
    async fn maybe_fail(&self, operation: &str) -> Result<(), SqsError> {
        // Record operation
        {
            let mut counter = self.failure_counter.lock().await;
            *counter.entry(operation.to_string()).or_insert(0) += 1;
        }
        
        // Introduce random latency
        let latency = thread_rng().gen_range(
            self.config.min_latency_ms..=self.config.max_latency_ms
        );
        sleep(std::time::Duration::from_millis(latency)).await;
        
        // Maybe fail the operation
        if thread_rng().gen::<f64>() < self.config.sqs_failure_rate {
            return Err(SqsError::Unhandled(Box::new(std::io::Error::new(
                std::io::ErrorKind::ConnectionReset,
                "Chaos-induced failure",
            ))));
        }
        
        Ok(())
    }
    
    // Example of a wrapped SQS operation
    async fn send_message(
        &self,
        queue_url: &str,
        message_body: &str,
    ) -> Result<String, SqsError> {
        // Maybe introduce chaos
        if let Err(e) = self.maybe_fail("send_message").await {
            return Err(e);
        }
        
        // Perform the actual operation
        let result = self.inner_client.send_message()
            .queue_url(queue_url)
            .message_body(message_body)
            .send()
            .await?;
            
        Ok(result.message_id().unwrap_or_default().to_string())
    }
    
    // Add more wrapped operations as needed
}

// Function to run a chaos test scenario
async fn run_chaos_test(config: ChaosTestConfig) -> Result<(), Box<dyn std::error::Error>> {
    println!("Starting chaos test with the following configuration:");
    println!("  DynamoDB failure rate: {:.2}%", config.dynamodb_failure_rate * 100.0);
    println!("  SQS failure rate: {:.2}%", config.sqs_failure_rate * 100.0);
    println!("  Latency range: {}-{} ms", config.min_latency_ms, config.max_latency_ms);
    println!("  Test duration: {} seconds", config.test_duration_seconds);
    
    // Create AWS clients
    let region_provider = RegionProviderChain::default_provider().or_else("us-east-1");
    let aws_config = aws_config::defaults(BehaviorVersion::latest())
        .region(region_provider)
        .load()
        .await;
    
    let dynamodb_client = DynamoDbClient::new(&aws_config);
    let sqs_client = SqsClient::new(&aws_config);
    
    // Create chaos clients
    let config_arc = Arc::new(config);
    let chaos_dynamodb = Arc::new(ChaosDynamoDbClient::new(
        dynamodb_client,
        config_arc.clone(),
    ));
    let chaos_sqs = Arc::new(ChaosSqsClient::new(
        sqs_client,
        config_arc.clone(),
    ));
    
    // Get environment variables
    let table_name = env::var("DYNAMODB_TABLE").unwrap_or_else(|_| "performance-data".to_string());
    let queue_url = env::var("SQS_QUEUE_URL").unwrap_or_else(|_| {
        "https://sqs.us-east-1.amazonaws.com/123456789012/performance-calculator-queue".to_string()
    });
    
    // Start the test
    let start_time = Instant::now();
    let end_time = start_time + std::time::Duration::from_secs(config_arc.test_duration_seconds);
    
    // Track success and failure counts
    let mut total_operations = 0;
    let mut successful_operations = 0;
    let mut failed_operations = 0;
    
    // Run the test until the duration expires
    while Instant::now() < end_time {
        // Generate a random portfolio ID
        let portfolio_id = format!("chaos-test-{}", Uuid::new_v4());
        
        // Perform a DynamoDB put_item operation
        let mut item = HashMap::new();
        item.insert(
            "id".to_string(),
            aws_sdk_dynamodb::model::AttributeValue::S(portfolio_id.clone()),
        );
        item.insert(
            "entity_type".to_string(),
            aws_sdk_dynamodb::model::AttributeValue::S("portfolio".to_string()),
        );
        item.insert(
            "created_at".to_string(),
            aws_sdk_dynamodb::model::AttributeValue::S(Utc::now().to_rfc3339()),
        );
        
        total_operations += 1;
        match chaos_dynamodb.put_item(&table_name, item).await {
            Ok(_) => {
                successful_operations += 1;
                println!("DynamoDB put_item succeeded for portfolio {}", portfolio_id);
            }
            Err(e) => {
                failed_operations += 1;
                println!("DynamoDB put_item failed for portfolio {}: {}", portfolio_id, e);
            }
        }
        
        // Perform an SQS send_message operation
        let message_body = json!({
            "portfolio_id": portfolio_id,
            "start_date": Utc.ymd(2023, 1, 1).and_hms(0, 0, 0).to_rfc3339(),
            "end_date": Utc.ymd(2023, 12, 31).and_hms(23, 59, 59).to_rfc3339(),
        }).to_string();
        
        total_operations += 1;
        match chaos_sqs.send_message(&queue_url, &message_body).await {
            Ok(message_id) => {
                successful_operations += 1;
                println!("SQS send_message succeeded with message ID: {}", message_id);
            }
            Err(e) => {
                failed_operations += 1;
                println!("SQS send_message failed: {}", e);
            }
        }
        
        // Add a small delay between iterations
        sleep(std::time::Duration::from_millis(500)).await;
    }
    
    // Print test results
    println!("\nChaos test completed. Results:");
    println!("  Total operations: {}", total_operations);
    println!("  Successful operations: {} ({:.2}%)", 
        successful_operations, 
        (successful_operations as f64 / total_operations as f64) * 100.0
    );
    println!("  Failed operations: {} ({:.2}%)", 
        failed_operations,
        (failed_operations as f64 / total_operations as f64) * 100.0
    );
    
    // Print operation counts by type
    println!("\nDynamoDB operation counts:");
    let dynamodb_counters = chaos_dynamodb.failure_counter.lock().await;
    for (op, count) in dynamodb_counters.iter() {
        println!("  {}: {}", op, count);
    }
    
    println!("\nSQS operation counts:");
    let sqs_counters = chaos_sqs.failure_counter.lock().await;
    for (op, count) in sqs_counters.iter() {
        println!("  {}: {}", op, count);
    }
    
    Ok(())
}

// Main function to run the chaos test
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Get command line arguments
    let args: Vec<String> = env::args().collect();
    
    // Default configuration
    let mut config = ChaosTestConfig {
        dynamodb_failure_rate: 0.1,  // 10% failure rate
        sqs_failure_rate: 0.1,       // 10% failure rate
        min_latency_ms: 50,          // Minimum 50ms latency
        max_latency_ms: 500,         // Maximum 500ms latency
        test_duration_seconds: 60,   // 1 minute test
    };
    
    // Parse command line arguments
    if args.len() > 1 {
        for i in 1..args.len() {
            let arg = &args[i];
            if arg.starts_with("--dynamodb-failure-rate=") {
                if let Some(value) = arg.strip_prefix("--dynamodb-failure-rate=") {
                    if let Ok(rate) = value.parse::<f64>() {
                        config.dynamodb_failure_rate = rate.clamp(0.0, 1.0);
                    }
                }
            } else if arg.starts_with("--sqs-failure-rate=") {
                if let Some(value) = arg.strip_prefix("--sqs-failure-rate=") {
                    if let Ok(rate) = value.parse::<f64>() {
                        config.sqs_failure_rate = rate.clamp(0.0, 1.0);
                    }
                }
            } else if arg.starts_with("--min-latency=") {
                if let Some(value) = arg.strip_prefix("--min-latency=") {
                    if let Ok(latency) = value.parse::<u64>() {
                        config.min_latency_ms = latency;
                    }
                }
            } else if arg.starts_with("--max-latency=") {
                if let Some(value) = arg.strip_prefix("--max-latency=") {
                    if let Ok(latency) = value.parse::<u64>() {
                        config.max_latency_ms = latency;
                    }
                }
            } else if arg.starts_with("--duration=") {
                if let Some(value) = arg.strip_prefix("--duration=") {
                    if let Ok(duration) = value.parse::<u64>() {
                        config.test_duration_seconds = duration;
                    }
                }
            } else if arg == "--help" {
                print_usage();
                return Ok(());
            }
        }
    }
    
    // Run the chaos test
    run_chaos_test(config).await?;
    
    Ok(())
}

fn print_usage() {
    println!("Chaos Test for Performance Calculator");
    println!("\nUsage:");
    println!("  cargo run --bin chaos_test [options]");
    println!("\nOptions:");
    println!("  --dynamodb-failure-rate=RATE   Set the DynamoDB failure rate (0.0-1.0, default: 0.1)");
    println!("  --sqs-failure-rate=RATE        Set the SQS failure rate (0.0-1.0, default: 0.1)");
    println!("  --min-latency=MS               Set the minimum latency in milliseconds (default: 50)");
    println!("  --max-latency=MS               Set the maximum latency in milliseconds (default: 500)");
    println!("  --duration=SECONDS             Set the test duration in seconds (default: 60)");
    println!("  --help                         Show this help message");
    println!("\nExample:");
    println!("  cargo run --bin chaos_test --dynamodb-failure-rate=0.2 --sqs-failure-rate=0.3 --duration=120");
}

// Instructions for running the chaos test:
// 
// 1. Make sure you have AWS credentials configured
// 2. Set the environment variables:
//    export DYNAMODB_TABLE=your-dynamodb-table-name
//    export SQS_QUEUE_URL=your-sqs-queue-url
// 
// 3. Build and run the chaos test:
//    cargo run --bin chaos_test
// 
// 4. To customize the test parameters:
//    cargo run --bin chaos_test --dynamodb-failure-rate=0.2 --sqs-failure-rate=0.3 --min-latency=100 --max-latency=1000 --duration=300 