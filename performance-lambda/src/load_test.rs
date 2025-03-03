use aws_sdk_sqs::{Client as SqsClient, Error};
use aws_config::meta::region::RegionProviderChain;
use aws_config::BehaviorVersion;
use chrono::{DateTime, Utc, TimeZone, Duration};
use serde_json::json;
use std::env;
use std::time::Instant;
use tokio::time::sleep;
use uuid::Uuid;
use futures::future::join_all;

// Structure to represent a performance calculation request
#[derive(serde::Serialize, serde::Deserialize)]
struct PerformanceCalculationRequest {
    portfolio_id: String,
    start_date: DateTime<Utc>,
    end_date: DateTime<Utc>,
    benchmark_id: Option<String>,
}

// Function to send a single message to SQS
async fn send_message(
    client: &SqsClient,
    queue_url: &str,
    portfolio_id: &str,
    start_date: DateTime<Utc>,
    end_date: DateTime<Utc>,
    benchmark_id: Option<String>,
) -> Result<String, Error> {
    let request = PerformanceCalculationRequest {
        portfolio_id: portfolio_id.to_string(),
        start_date,
        end_date,
        benchmark_id,
    };
    
    let message_body = serde_json::to_string(&request).unwrap();
    
    let response = client.send_message()
        .queue_url(queue_url)
        .message_body(message_body)
        .send()
        .await?;
    
    Ok(response.message_id().unwrap_or_default().to_string())
}

// Function to run the load test
async fn run_load_test(
    queue_url: &str,
    num_messages: usize,
    concurrency: usize,
    portfolio_prefix: &str,
) -> Result<(), Error> {
    // Create SQS client
    let region_provider = RegionProviderChain::default_provider().or_else("us-east-1");
    let config = aws_config::defaults(BehaviorVersion::latest())
        .region(region_provider)
        .load()
        .await;
    let client = SqsClient::new(&config);
    
    println!("Starting load test with {} messages and concurrency level {}", num_messages, concurrency);
    
    // Create a vector to hold all the futures
    let mut futures = Vec::with_capacity(num_messages);
    
    // Generate the requests
    for i in 0..num_messages {
        let portfolio_id = format!("{}-{}", portfolio_prefix, Uuid::new_v4());
        let start_date = Utc.ymd(2023, 1, 1).and_hms(0, 0, 0);
        let end_date = Utc.ymd(2023, 12, 31).and_hms(23, 59, 59);
        let benchmark_id = if i % 2 == 0 { Some("benchmark1".to_string()) } else { None };
        
        let client_clone = client.clone();
        let queue_url_clone = queue_url.to_string();
        
        futures.push(tokio::spawn(async move {
            send_message(
                &client_clone,
                &queue_url_clone,
                &portfolio_id,
                start_date,
                end_date,
                benchmark_id,
            ).await
        }));
        
        // If we've reached the concurrency limit, wait for all futures to complete
        if futures.len() >= concurrency {
            let results = join_all(futures).await;
            futures = Vec::with_capacity(concurrency);
            
            // Print progress
            let success_count = results.iter().filter(|r| r.is_ok() && r.as_ref().unwrap().is_ok()).count();
            let error_count = results.len() - success_count;
            println!("Processed batch: {} successful, {} errors", success_count, error_count);
            
            // Add a small delay to avoid overwhelming the SQS queue
            sleep(std::time::Duration::from_millis(100)).await;
        }
    }
    
    // Process any remaining futures
    if !futures.is_empty() {
        let results = join_all(futures).await;
        let success_count = results.iter().filter(|r| r.is_ok() && r.as_ref().unwrap().is_ok()).count();
        let error_count = results.len() - success_count;
        println!("Processed final batch: {} successful, {} errors", success_count, error_count);
    }
    
    println!("Load test completed successfully");
    Ok(())
}

// Function to monitor the SQS queue during the load test
async fn monitor_queue(
    client: &SqsClient,
    queue_url: &str,
    interval_seconds: u64,
    duration_seconds: u64,
) -> Result<(), Error> {
    let start_time = Instant::now();
    let end_time = start_time + std::time::Duration::from_secs(duration_seconds);
    
    println!("Starting queue monitoring for {} seconds", duration_seconds);
    
    while Instant::now() < end_time {
        // Get queue attributes
        let attributes = client.get_queue_attributes()
            .queue_url(queue_url)
            .attribute_names(aws_sdk_sqs::model::QueueAttributeName::All)
            .send()
            .await?;
        
        // Extract metrics
        let messages_available = attributes
            .attributes()
            .and_then(|attrs| attrs.get(aws_sdk_sqs::model::QueueAttributeName::ApproximateNumberOfMessages))
            .and_then(|val| val.parse::<i64>().ok())
            .unwrap_or(0);
        
        let messages_in_flight = attributes
            .attributes()
            .and_then(|attrs| attrs.get(aws_sdk_sqs::model::QueueAttributeName::ApproximateNumberOfMessagesNotVisible))
            .and_then(|val| val.parse::<i64>().ok())
            .unwrap_or(0);
        
        let messages_delayed = attributes
            .attributes()
            .and_then(|attrs| attrs.get(aws_sdk_sqs::model::QueueAttributeName::ApproximateNumberOfMessagesDelayed))
            .and_then(|val| val.parse::<i64>().ok())
            .unwrap_or(0);
        
        // Print metrics
        println!(
            "[{}s] Queue metrics - Available: {}, In Flight: {}, Delayed: {}",
            start_time.elapsed().as_secs(),
            messages_available,
            messages_in_flight,
            messages_delayed
        );
        
        // Wait for the next interval
        sleep(std::time::Duration::from_secs(interval_seconds)).await;
    }
    
    println!("Queue monitoring completed");
    Ok(())
}

// Main function to run the load test
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Get command line arguments
    let args: Vec<String> = env::args().collect();
    
    if args.len() < 2 {
        println!("Usage: {} <command> [options]", args[0]);
        println!("Commands:");
        println!("  run-load-test <queue-url> <num-messages> <concurrency> <portfolio-prefix>");
        println!("  monitor-queue <queue-url> <interval-seconds> <duration-seconds>");
        return Ok(());
    }
    
    match args[1].as_str() {
        "run-load-test" => {
            if args.len() < 6 {
                println!("Usage: {} run-load-test <queue-url> <num-messages> <concurrency> <portfolio-prefix>", args[0]);
                return Ok(());
            }
            
            let queue_url = &args[2];
            let num_messages = args[3].parse::<usize>().unwrap_or(100);
            let concurrency = args[4].parse::<usize>().unwrap_or(10);
            let portfolio_prefix = &args[5];
            
            run_load_test(queue_url, num_messages, concurrency, portfolio_prefix).await?;
        },
        "monitor-queue" => {
            if args.len() < 5 {
                println!("Usage: {} monitor-queue <queue-url> <interval-seconds> <duration-seconds>", args[0]);
                return Ok(());
            }
            
            let queue_url = &args[2];
            let interval_seconds = args[3].parse::<u64>().unwrap_or(5);
            let duration_seconds = args[4].parse::<u64>().unwrap_or(300);
            
            // Create SQS client
            let region_provider = RegionProviderChain::default_provider().or_else("us-east-1");
            let config = aws_config::defaults(BehaviorVersion::latest())
                .region(region_provider)
                .load()
                .await;
            let client = SqsClient::new(&config);
            
            monitor_queue(&client, queue_url, interval_seconds, duration_seconds).await?;
        },
        _ => {
            println!("Unknown command: {}", args[1]);
            println!("Usage: {} <command> [options]", args[0]);
            println!("Commands:");
            println!("  run-load-test <queue-url> <num-messages> <concurrency> <portfolio-prefix>");
            println!("  monitor-queue <queue-url> <interval-seconds> <duration-seconds>");
        }
    }
    
    Ok(())
}

// Instructions for running the load test:
// 
// 1. Make sure you have AWS credentials configured
// 2. Build the load test binary:
//    cargo build --release --bin load_test
// 
// 3. Run the load test:
//    ./target/release/load_test run-load-test <queue-url> <num-messages> <concurrency> <portfolio-prefix>
//    
//    Example:
//    ./target/release/load_test run-load-test https://sqs.us-east-1.amazonaws.com/123456789012/performance-calculator-queue 1000 50 test-portfolio
// 
// 4. Monitor the queue during the load test:
//    ./target/release/load_test monitor-queue <queue-url> <interval-seconds> <duration-seconds>
//    
//    Example:
//    ./target/release/load_test monitor-queue https://sqs.us-east-1.amazonaws.com/123456789012/performance-calculator-queue 5 300 