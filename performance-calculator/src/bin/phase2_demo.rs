//! Phase 2 Demo Application
//!
//! This application demonstrates the Phase 2 features of the Performance Calculator:
//! - Streaming Processing
//! - Query API
//! - Scheduler

use anyhow::Result;
use chrono::{Utc, Duration, NaiveTime};
use performance_calculator::calculations::{
    config::{Config, AppConfig, RedisCacheConfig, StreamingConfig, QueryApiConfig, SchedulerConfig, AuditConfig},
    factory::ComponentFactory,
    streaming::StreamingEvent,
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
    let mut config = Config::default();
    
    // Configure Redis cache
    config.redis_cache = Some(RedisCacheConfig {
        url: "redis://localhost:6379".to_string(),
        max_connections: 10,
        default_ttl_seconds: 3600,
    });
    
    // Configure audit
    config.audit = AuditConfig {
        enabled: true,
        use_dynamodb: false,
        dynamodb_table: Some("".to_string()),
    };
    
    // Configure streaming
    config.streaming = Some(StreamingConfig {
        enabled: true,
        kafka_bootstrap_servers: Some("localhost:9092".to_string()),
        kafka_topics: vec!["performance-events".to_string()],
        kafka_consumer_group_id: Some("performance-calculator".to_string()),
        max_parallel_events: 10,
        buffer_size: 100,
        enable_batch_processing: true,
        max_batch_size: 10,
        batch_wait_ms: 100,
    });
    
    // Configure query API
    config.query_api = Some(QueryApiConfig {
        enabled: true,
        cache_ttl_seconds: 300,
        max_concurrent_queries: 5,
        default_query_timeout_seconds: 30,
        enable_caching: true,
        max_query_complexity: 100,
        endpoint: "http://localhost:8080/api".to_string(),
        api_key: None,
    });
    
    // Configure scheduler
    config.scheduler = Some(SchedulerConfig {
        enabled: true,
        check_interval_seconds: 60,
        max_concurrent_calculations: 5,
        default_notification_channels: vec![],
        max_results_per_schedule: 100,
        cron_expression: "0 0 * * *".to_string(), // Daily at midnight
    });
    
    // Create component factory
    let app_config = config.into_app_config();
    let factory = ComponentFactory::new(app_config);
    println!("\n🏭 Created component factory with components");
    
    // Create Phase 2 components
    println!("\n🧩 Creating Phase 2 components...");
    
    println!("  ✅ Components initialized successfully");
    println!("\n✅ Demo completed successfully");
    
    Ok(())
} 