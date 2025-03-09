//! Configuration management for the performance calculator.
//!
//! This module provides structures and functionality for managing application configuration
//! from various sources including files (JSON, TOML, YAML) and environment variables.
//! 
//! # Examples
//!
//! ```
//! use performance_calculator::calculations::config::AppConfig;
//!
//! // Load from environment variables
//! let config = AppConfig::from_env();
//!
//! // Or load from a file
//! let config = AppConfig::from_file("config.json").expect("Failed to load config");
//! ```

use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use std::path::Path;
use std::fs;
use anyhow::{Result, anyhow, Context};
use std::env;
use std::fs::File;
use std::io::Read;
use crate::calculations::analytics::AnalyticsConfig;
use crate::calculations::visualization::VisualizationConfig;
use crate::calculations::integration::IntegrationConfig;

/// Main configuration for the performance calculator
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// Database configuration
    pub database: DatabaseConfig,
    /// Streaming configuration
    pub streaming: Option<StreamingConfig>,
    /// Query API configuration
    pub query_api: Option<QueryApiConfig>,
    /// Scheduler configuration
    pub scheduler: Option<SchedulerConfig>,
    /// Audit trail configuration
    pub audit_trail: Option<AuditTrailConfig>,
    /// Redis cache configuration
    pub redis_cache: Option<RedisCacheConfig>,
    /// AWS services configuration
    pub aws: AwsConfig,
    /// Email notification configuration
    pub email: EmailConfig,
    /// Audit trail configuration
    pub audit: AuditConfig,
    /// Analytics configuration (Phase 3)
    pub analytics: Option<AnalyticsConfig>,
    /// Visualization configuration (Phase 3)
    pub visualization: Option<VisualizationConfig>,
    /// Integration configuration (Phase 3)
    pub integration: Option<IntegrationConfig>,
}

impl Config {
    /// Loads configuration from a YAML file
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let content = fs::read_to_string(path)
            .context("Failed to read config file")?;
        
        serde_yaml::from_str(&content)
            .context("Failed to parse config file")
    }
    
    /// Loads configuration from environment variables
    pub fn from_env() -> Result<Self> {
        // Placeholder implementation
        Ok(Self {
            database: DatabaseConfig {
                dynamodb_table: "performance_data".to_string(),
                timestream_database: "performance_metrics".to_string(),
                timestream_table: "calculations".to_string(),
            },
            streaming: None,
            query_api: None,
            scheduler: None,
            audit_trail: None,
            redis_cache: None,
            aws: AwsConfig::default(),
            email: EmailConfig::default(),
            audit: AuditConfig::default(),
            analytics: None,
            visualization: None,
            integration: None,
        })
    }

    /// Convert Config to AppConfig
    pub fn into_app_config(self) -> AppConfig {
        AppConfig {
            streaming: self.streaming.unwrap_or_default(),
            query_api: self.query_api.unwrap_or_default(),
            scheduler: self.scheduler.unwrap_or_default(),
            redis_cache: self.redis_cache.unwrap_or_default(),
            aws: self.aws,
            email: self.email,
            audit: self.audit,
            analytics: self.analytics,
            visualization: self.visualization,
            integration: self.integration,
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            redis_cache: None,
            streaming: None,
            query_api: None,
            scheduler: None,
            analytics: None,
            visualization: None,
            integration: None,
            database: DatabaseConfig {
                dynamodb_table: "performance_data".to_string(),
                timestream_database: "performance_metrics".to_string(),
                timestream_table: "calculations".to_string(),
            },
            audit_trail: None,
            aws: AwsConfig::default(),
            email: EmailConfig::default(),
            audit: AuditConfig::default(),
        }
    }
}

/// Database configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseConfig {
    /// DynamoDB table name
    pub dynamodb_table: String,
    /// Timestream database name
    pub timestream_database: String,
    /// Timestream table name
    pub timestream_table: String,
}

/// Audit trail configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditTrailConfig {
    /// Whether to enable the audit trail
    pub enabled: bool,
    /// Type of audit trail storage
    pub storage_type: String,
    /// DynamoDB table name for audit trail
    pub dynamodb_table: Option<String>,
}

/// Configuration for the streaming data processing.
///
/// Controls how the application processes streaming data events,
/// including Kafka connection settings and processing parameters.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamingConfig {
    /// Kafka bootstrap servers connection string (e.g., "localhost:9092")
    pub kafka_bootstrap_servers: Option<String>,
    
    /// List of Kafka topics to consume events from
    pub kafka_topics: Vec<String>,
    
    /// Kafka consumer group ID for tracking consumption progress
    pub kafka_consumer_group_id: Option<String>,
    
    /// Maximum number of events to process in parallel
    /// Higher values increase throughput but require more resources
    pub max_parallel_events: usize,
    
    /// Whether to enable streaming processing
    /// When false, the streaming processor will not start
    pub enabled: bool,
    
    /// Size of the event buffer
    pub buffer_size: usize,
    
    /// Whether to enable batch processing
    pub enable_batch_processing: bool,
    
    /// Maximum batch size when batch processing is enabled
    pub max_batch_size: usize,
    
    /// Maximum wait time in milliseconds for batch processing
    pub batch_wait_ms: u64,
}

impl Default for StreamingConfig {
    /// Creates a default streaming configuration with conservative settings.
    fn default() -> Self {
        Self {
            kafka_bootstrap_servers: None,
            kafka_topics: Vec::new(),
            kafka_consumer_group_id: None,
            max_parallel_events: 10,
            enabled: false,
            buffer_size: 1000,
            enable_batch_processing: true,
            max_batch_size: 50,
            batch_wait_ms: 100,
        }
    }
}

/// Configuration for the query API.
///
/// Controls how the application handles performance calculation queries,
/// including caching behavior and resource limits.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryApiConfig {
    /// Cache time-to-live in seconds
    /// Determines how long query results are cached before recalculation
    pub cache_ttl_seconds: u64,
    
    /// Maximum number of concurrent queries allowed
    /// Prevents resource exhaustion under heavy load
    pub max_concurrent_queries: usize,
    
    /// Default query timeout in seconds
    /// Queries that exceed this duration will be cancelled
    pub default_query_timeout_seconds: u64,
    
    /// Whether to enable query result caching
    /// Disabling may be useful for debugging or when results must always be fresh
    pub enable_caching: bool,
    
    /// Maximum query complexity score allowed
    /// Complex queries with many parameters or large date ranges have higher scores
    pub max_query_complexity: u32,
    
    /// API endpoint
    pub endpoint: String,
    
    /// API key
    pub api_key: Option<String>,
    
    /// Whether to enable the query API
    pub enabled: bool,
}

impl Default for QueryApiConfig {
    /// Creates a default query API configuration with conservative settings.
    fn default() -> Self {
        Self {
            cache_ttl_seconds: 300, // 5 minutes
            max_concurrent_queries: 5,
            default_query_timeout_seconds: 30,
            enable_caching: true,
            max_query_complexity: 50,
            endpoint: "http://localhost:8080".to_string(),
            api_key: None,
            enabled: false,
        }
    }
}

/// Configuration for the scheduler.
///
/// Controls how the application schedules and executes periodic calculations,
/// including concurrency limits and notification settings.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchedulerConfig {
    /// Scheduler check interval in seconds
    /// How often the scheduler checks for calculations that need to run
    pub check_interval_seconds: u64,
    
    /// Maximum number of concurrent scheduled calculations
    /// Prevents resource exhaustion when many schedules trigger simultaneously
    pub max_concurrent_calculations: usize,
    
    /// Whether to enable the scheduler
    /// When false, scheduled calculations will not run
    pub enabled: bool,
    
    /// Default notification channels for all scheduled calculations
    /// Can be overridden by individual schedules
    pub default_notification_channels: Vec<NotificationChannelConfig>,
    
    /// Maximum number of historical results to keep per schedule
    /// Older results beyond this limit will be pruned
    pub max_results_per_schedule: usize,
    
    /// Cron expression for scheduling
    pub cron_expression: String,
}

impl Default for SchedulerConfig {
    /// Creates a default scheduler configuration with conservative settings.
    fn default() -> Self {
        Self {
            check_interval_seconds: 60,
            max_concurrent_calculations: 2,
            enabled: false,
            default_notification_channels: Vec::new(),
            max_results_per_schedule: 10,
            cron_expression: "0 0 * * *".to_string(), // Daily at midnight
        }
    }
}

/// Configuration for notification channels
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationChannelConfig {
    /// Channel type (e.g., "email", "webhook", "sns", "sqs")
    pub channel_type: String,
    
    /// Channel-specific configuration parameters
    /// For email: "recipients", "subject_template", "body_template"
    /// For webhook: "url", "method", "headers"
    /// For SNS: "topic_arn"
    /// For SQS: "queue_url"
    pub config: HashMap<String, String>,
}

/// Redis cache configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RedisCacheConfig {
    /// Redis connection URL (e.g., "redis://localhost:6379")
    pub url: String,
    
    /// Maximum number of connections in the pool
    /// Higher values allow more concurrent operations but consume more resources
    pub max_connections: usize,
    
    /// Default time-to-live in seconds for cached items
    /// Can be overridden for specific cache operations
    pub default_ttl_seconds: u64,
}

impl Default for RedisCacheConfig {
    fn default() -> Self {
        Self {
            url: "redis://localhost:6379".to_string(),
            max_connections: 5,
            default_ttl_seconds: 300, // 5 minutes
        }
    }
}

/// Audit trail configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditConfig {
    /// Whether audit trail is enabled
    pub enabled: bool,
    /// Whether to use DynamoDB for audit storage
    pub use_dynamodb: bool,
    /// DynamoDB table name for audit trail
    pub dynamodb_table: Option<String>,
}

impl Default for AuditConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            use_dynamodb: false,
            dynamodb_table: None,
        }
    }
}

/// Main application configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    /// Streaming data processing configuration
    pub streaming: StreamingConfig,
    
    /// Query API configuration
    pub query_api: QueryApiConfig,
    
    /// Scheduler configuration
    pub scheduler: SchedulerConfig,
    
    /// Redis cache configuration
    pub redis_cache: RedisCacheConfig,
    
    /// AWS services configuration
    pub aws: AwsConfig,
    
    /// Email notification configuration
    pub email: EmailConfig,
    
    /// Audit trail configuration
    pub audit: AuditConfig,
    
    /// Analytics configuration (Phase 3)
    pub analytics: Option<AnalyticsConfig>,
    
    /// Visualization configuration (Phase 3)
    pub visualization: Option<VisualizationConfig>,
    
    /// Integration configuration (Phase 3)
    pub integration: Option<IntegrationConfig>,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            streaming: StreamingConfig::default(),
            query_api: QueryApiConfig::default(),
            scheduler: SchedulerConfig::default(),
            redis_cache: RedisCacheConfig::default(),
            aws: AwsConfig::default(),
            email: EmailConfig::default(),
            audit: AuditConfig::default(),
            analytics: None,
            visualization: None,
            integration: None,
        }
    }
}

/// AWS services configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AwsConfig {
    /// AWS region (e.g., "us-east-1")
    pub region: String,
    
    /// SNS topic ARN for sending notifications
    /// If None, SNS notifications will be disabled
    pub notification_topic_arn: Option<String>,
    
    /// SQS queue URL for sending notifications
    /// If None, SQS notifications will be disabled
    pub notification_queue_url: Option<String>,
}

impl Default for AwsConfig {
    fn default() -> Self {
        Self {
            region: "us-east-1".to_string(),
            notification_topic_arn: None,
            notification_queue_url: None,
        }
    }
}

/// Email notification configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmailConfig {
    /// SMTP server hostname or IP address
    pub smtp_server: Option<String>,
    
    /// SMTP server port (typically 25, 465, or 587)
    pub smtp_port: Option<u16>,
    
    /// SMTP authentication username
    pub smtp_username: Option<String>,
    
    /// SMTP authentication password
    pub smtp_password: Option<String>,
    
    /// Email address to use in the "From" field
    pub from_address: Option<String>,
}

impl Default for EmailConfig {
    fn default() -> Self {
        Self {
            smtp_server: None,
            smtp_port: None,
            smtp_username: None,
            smtp_password: None,
            from_address: None,
        }
    }
}

impl AppConfig {
    /// Load configuration from environment variables
    pub fn from_env() -> Self {
        // Placeholder implementation
        Self::default()
    }
    
    /// Load configuration from a YAML file
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let content = fs::read_to_string(path)
            .context("Failed to read config file")?;
        
        serde_yaml::from_str(&content)
            .context("Failed to parse config file")
    }
    
    /// Save configuration to a YAML file
    pub fn to_file<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let content = serde_yaml::to_string(self)
            .context("Failed to serialize config")?;
        
        fs::write(path, content)
            .context("Failed to write config file")?;
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_default_config() {
        let config = AppConfig::default();
        assert!(!config.streaming.enabled);
        assert!(!config.query_api.enabled);
        assert!(!config.scheduler.enabled);
        assert_eq!(config.aws.region, "us-east-1");
    }
    
    #[test]
    fn test_config_serialization() {
        let config = AppConfig::default();
        let yaml = serde_yaml::to_string(&config).unwrap();
        let deserialized: AppConfig = serde_yaml::from_str(&yaml).unwrap();
        assert_eq!(deserialized.streaming.enabled, config.streaming.enabled);
        assert_eq!(deserialized.query_api.enabled, config.query_api.enabled);
        assert_eq!(deserialized.scheduler.enabled, config.scheduler.enabled);
    }
} 