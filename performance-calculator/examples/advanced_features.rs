use anyhow::Result;
use chrono::{Utc, NaiveDate};
use std::sync::Arc;
use tokio::signal;

use performance_calculator::calculations::{
    config::AppConfig,
    factory::ComponentFactory,
    audit::{AuditTrailManager, InMemoryAuditTrailStorage},
    currency::{CurrencyConverter, MockExchangeRateProviderMock},
    query_api::{PerformanceQueryParams, MockDataAccessService},
    scheduler::{ScheduledCalculation, ScheduledCalculationType, ScheduleFrequency, NotificationChannel},
};

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt::init();
    
    // Load configuration
    let config = AppConfig::from_env();
    
    // Create component factory
    let factory = ComponentFactory::new(config);
    
    // Create audit trail manager
    let storage = Arc::new(InMemoryAuditTrailStorage::new());
    let audit_manager = Arc::new(AuditTrailManager::new(storage.clone()));
    
    // Create currency converter
    let exchange_rate_provider = Arc::new(MockExchangeRateProviderMock::new());
    let currency_converter = Arc::new(CurrencyConverter::new(
        exchange_rate_provider,
        "USD".to_string(),
    ));
    
    // Create data access service
    let data_service = Arc::new(MockDataAccessService);
    
    // Create query API
    let query_api = factory.create_query_api(
        audit_manager.clone(),
        currency_converter.clone(),
        data_service.clone(),
    ).await?;
    
    // Create streaming processor
    let streaming_processor = match factory.create_streaming_processor(audit_manager.clone()).await {
        Ok(processor) => Some(processor),
        Err(e) => {
            eprintln!("Failed to create streaming processor: {}", e);
            None
        }
    };
    
    // Create calculation scheduler
    let scheduler = match factory.create_calculation_scheduler(
        query_api.clone(),
        audit_manager.clone(),
    ).await {
        Ok(scheduler) => Some(scheduler),
        Err(e) => {
            eprintln!("Failed to create calculation scheduler: {}", e);
            None
        }
    };
    
    // Start streaming processor if available
    if let Some(processor) = &streaming_processor {
        processor.start().await?;
        println!("Streaming processor started");
    }
    
    // Start scheduler if available
    if let Some(scheduler) = &scheduler {
        scheduler.start().await?;
        println!("Calculation scheduler started");
        
        // Add a sample scheduled calculation
        let schedule = ScheduledCalculation {
            id: "daily-performance".to_string(),
            name: "Daily Performance Report".to_string(),
            description: Some("Calculates performance metrics daily".to_string()),
            calculation_type: ScheduledCalculationType::Performance(
                PerformanceQueryParams {
                    portfolio_id: "sample-portfolio".to_string(),
                    start_date: NaiveDate::from_ymd_opt(2023, 1, 1).unwrap(),
                    end_date: NaiveDate::from_ymd_opt(2023, 12, 31).unwrap(),
                    twr_method: Some("daily".to_string()),
                    include_risk_metrics: Some(true),
                    include_periodic_returns: Some(true),
                    benchmark_id: Some("SPY".to_string()),
                    currency: None,
                    annualize: Some(true),
                    custom_params: None,
                }
            ),
            frequency: ScheduleFrequency::Daily {
                hour: 8,
                minute: 0,
            },
            enabled: true,
            notification_channels: vec![
                NotificationChannel::Email {
                    recipients: vec!["user@example.com".to_string()],
                    subject_template: "Daily Performance Report".to_string(),
                    body_template: "Performance report for {{schedule.name}} is ready.".to_string(),
                },
            ],
            last_run_time: None,
            next_run_time: None,
            created_by: "system".to_string(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };
        
        scheduler.add_schedule(schedule).await?;
        println!("Added sample scheduled calculation");
        
        // Run a calculation immediately
        let run_id = scheduler.run_now("daily-performance").await?;
        println!("Triggered immediate calculation with run ID: {}", run_id);
    }
    
    // Execute a sample query
    let query_params = PerformanceQueryParams {
        portfolio_id: "sample-portfolio".to_string(),
        start_date: NaiveDate::from_ymd_opt(2023, 1, 1).unwrap(),
        end_date: NaiveDate::from_ymd_opt(2023, 3, 31).unwrap(),
        twr_method: Some("daily".to_string()),
        include_risk_metrics: Some(true),
        include_periodic_returns: Some(true),
        benchmark_id: Some("SPY".to_string()),
        currency: None,
        annualize: Some(false),
        custom_params: None,
    };
    
    let result = query_api.calculate_performance(query_params).await?;
    println!("Query result: {:?}", result);
    
    // Wait for Ctrl+C
    println!("Press Ctrl+C to exit");
    signal::ctrl_c().await?;
    
    // Stop services
    if let Some(processor) = &streaming_processor {
        processor.stop().await?;
        println!("Streaming processor stopped");
    }
    
    if let Some(scheduler) = &scheduler {
        scheduler.stop().await?;
        println!("Calculation scheduler stopped");
    }
    
    println!("Application shutdown complete");
    
    Ok(())
} 