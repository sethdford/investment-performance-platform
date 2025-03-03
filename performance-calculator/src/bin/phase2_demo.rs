//! Phase 2 Demo Application
//!
//! This application demonstrates the Phase 2 features of the Performance Calculator:
//! - Streaming Processing
//! - Query API
//! - Scheduler

use anyhow::{Result, Context};
use chrono::{Utc, Duration, NaiveTime};
use performance_calculator::calculations::{
    config::Config,
    factory::ComponentFactory,
    streaming::StreamingEvent,
    query_api::PerformanceQuery,
    scheduler::{Job, ScheduleType},
};
use serde_json::json;
use std::sync::Arc;
use tokio::time::sleep;
use uuid::Uuid;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();
    
    println!("Performance Calculator - Phase 2 Demo");
    println!("=====================================");
    
    // Create configuration with Phase 2 features enabled
    let config = Config {
        redis_cache: performance_calculator::calculations::config::RedisCacheConfig {
            enabled: true,
            url: "redis://localhost:6379".to_string(),
            ttl_seconds: 3600,
            prefix: "demo:".to_string(),
        },
        audit: performance_calculator::calculations::config::AuditConfig {
            enabled: true,
            use_dynamodb: false,
            dynamodb_table: "".to_string(),
            dynamodb_region: "".to_string(),
        },
        streaming: performance_calculator::calculations::config::StreamingConfig {
            enabled: true,
            max_concurrent_events: 10,
            buffer_size: 100,
            enable_batch_processing: true,
            max_batch_size: 10,
            batch_wait_ms: 100,
            register_default_handlers: true,
        },
        query_api: performance_calculator::calculations::config::QueryApiConfig {
            enabled: true,
            max_results: 100,
            default_page_size: 10,
            cache_ttl_seconds: 300,
            enable_caching: true,
        },
        scheduler: performance_calculator::calculations::config::SchedulerConfig {
            enabled: true,
            max_concurrent_jobs: 5,
            default_retry_count: 3,
            default_retry_delay_seconds: 60,
            history_retention_days: 30,
            register_default_handlers: true,
        },
        ..Config::default()
    };
    
    // Create component factory
    let factory = ComponentFactory::new_with_mocks(config);
    println!("\n🏭 Created component factory with mock components");
    
    // Create Phase 2 components
    println!("\n🧩 Creating Phase 2 components...");
    
    let streaming_processor = factory.create_streaming_processor().await?
        .context("Failed to create streaming processor")?;
    println!("  ✅ Created streaming processor");
    
    let query_api = factory.create_query_api().await?
        .context("Failed to create query API")?;
    println!("  ✅ Created query API");
    
    let scheduler = factory.create_scheduler().await?
        .context("Failed to create scheduler")?;
    println!("  ✅ Created scheduler");
    
    // Start the scheduler
    scheduler.start().await?;
    println!("  ✅ Started scheduler");
    
    // Demonstrate streaming processing
    println!("\n🔄 Demonstrating Streaming Processing:");
    
    // Create and submit streaming events
    let events = vec![
        StreamingEvent {
            id: Uuid::new_v4().to_string(),
            timestamp: Utc::now(),
            event_type: "transaction".to_string(),
            source: "demo".to_string(),
            entity_id: "portfolio-123".to_string(),
            payload: json!({
                "transaction_type": "buy",
                "security_id": "AAPL",
                "quantity": 100,
                "price": 150.0,
                "currency": "USD",
                "date": "2023-01-15",
            }),
        },
        StreamingEvent {
            id: Uuid::new_v4().to_string(),
            timestamp: Utc::now(),
            event_type: "price_update".to_string(),
            source: "demo".to_string(),
            entity_id: "AAPL".to_string(),
            payload: json!({
                "price": 155.0,
                "currency": "USD",
                "date": "2023-01-16",
            }),
        },
    ];
    
    println!("  Submitting {} streaming events...", events.len());
    for event in events {
        println!("    Event: {} - {}", event.id, event.event_type);
        streaming_processor.submit_event(event).await?;
    }
    
    // Wait for events to be processed
    println!("  Waiting for events to be processed...");
    sleep(std::time::Duration::from_secs(2)).await;
    
    // Demonstrate Query API
    println!("\n🔍 Demonstrating Query API:");
    
    // Create and execute a query
    let query = PerformanceQuery {
        portfolio_ids: Some(vec!["portfolio-123".to_string()]),
        start_date: Some(chrono::NaiveDate::from_ymd_opt(2023, 1, 1).unwrap()),
        end_date: Some(chrono::NaiveDate::from_ymd_opt(2023, 12, 31).unwrap()),
        base_currency: Some("USD".to_string()),
        metrics: Some(vec!["twr".to_string(), "mwr".to_string(), "volatility".to_string()]),
        include_periodic_returns: Some(true),
        period_type: Some("monthly".to_string()),
        group_by: Some("asset_class".to_string()),
        pagination: None,
        sort_by: None,
    };
    
    println!("  Executing performance query...");
    let result = query_api.execute_query(query.clone()).await?;
    
    println!("  Query results:");
    println!("    Execution time: {} ms", result.metadata.execution_time_ms);
    println!("    From cache: {}", result.metadata.from_cache);
    println!("    Results count: {}", result.results.len());
    
    for (i, portfolio_result) in result.results.iter().enumerate() {
        println!("    Result {}: Portfolio {}", i + 1, portfolio_result.portfolio_id);
        println!("      TWR: {}", portfolio_result.metrics.get("twr").unwrap_or(&rust_decimal::Decimal::ZERO));
        println!("      MWR: {}", portfolio_result.metrics.get("mwr").unwrap_or(&rust_decimal::Decimal::ZERO));
    }
    
    // Execute the same query again to demonstrate caching
    println!("  Executing the same query again (should be cached)...");
    let result2 = query_api.execute_query(query).await?;
    println!("    From cache: {}", result2.metadata.from_cache);
    println!("    Execution time: {} ms", result2.metadata.execution_time_ms);
    
    // Demonstrate Scheduler
    println!("\n⏰ Demonstrating Scheduler:");
    
    // Create and schedule jobs
    let jobs = vec![
        // One-time job
        Job::new(
            "One-time Performance Calculation".to_string(),
            "performance_calculation".to_string(),
            json!({
                "portfolio_id": "portfolio-123",
                "start_date": "2023-01-01",
                "end_date": "2023-12-31",
                "base_currency": "USD",
            }),
            ScheduleType::OneTime {
                execution_time: Utc::now() + Duration::seconds(5),
            },
        ),
        // Recurring job
        Job::new(
            "Recurring Performance Calculation".to_string(),
            "performance_calculation".to_string(),
            json!({
                "portfolio_id": "portfolio-456",
                "start_date": "2023-01-01",
                "end_date": "2023-12-31",
                "base_currency": "EUR",
            }),
            ScheduleType::Recurring {
                interval_seconds: 60,
                start_time: Some(Utc::now() + Duration::seconds(10)),
                end_time: Some(Utc::now() + Duration::seconds(120)),
            },
        ),
        // Daily job
        Job::new(
            "Daily Performance Calculation".to_string(),
            "performance_calculation".to_string(),
            json!({
                "portfolio_id": "portfolio-789",
                "start_date": "2023-01-01",
                "end_date": "2023-12-31",
                "base_currency": "GBP",
            }),
            ScheduleType::Daily {
                times: vec![
                    NaiveTime::from_hms_opt(9, 0, 0).unwrap(),
                    NaiveTime::from_hms_opt(17, 0, 0).unwrap(),
                ],
                days_of_week: Some(vec![1, 2, 3, 4, 5]), // Monday to Friday
            },
        ),
    ];
    
    println!("  Scheduling {} jobs...", jobs.len());
    for job in jobs {
        println!("    Job: {} - {}", job.name, job.job_type);
        let job_id = scheduler.schedule_job(job).await?;
        println!("      Scheduled with ID: {}", job_id);
    }
    
    // Wait for jobs to execute
    println!("  Waiting for jobs to execute...");
    sleep(std::time::Duration::from_secs(15)).await;
    
    // List recent job executions
    let jobs = scheduler.list_jobs().await?;
    println!("  Scheduled jobs: {}", jobs.len());
    
    for job in &jobs {
        let executions = scheduler.list_executions_for_job(&job.id).await?;
        println!("    Job '{}' executions: {}", job.name, executions.len());
        
        for execution in executions {
            println!("      Execution: {} - Status: {:?}", execution.id, execution.status);
            if let Some(result) = execution.result {
                println!("        Result: {}", result);
            }
        }
    }
    
    // Stop the scheduler
    scheduler.stop().await?;
    println!("  ✅ Stopped scheduler");
    
    println!("\n✅ Phase 2 Demo completed successfully");
    
    Ok(())
} 