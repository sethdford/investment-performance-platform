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
}

impl Default for QueryApiConfig {
    /// Creates a default query API configuration with reasonable settings.
    fn default() -> Self {
        Self {
            cache_ttl_seconds: 3600,  // 1 hour
            max_concurrent_queries: 20,
            default_query_timeout_seconds: 60,
            enable_caching: true,
            max_query_complexity: 100,
        }
    }
}

/// Configuration for the calculation scheduler.
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
}

impl Default for SchedulerConfig {
    /// Creates a default scheduler configuration with conservative settings.
    fn default() -> Self {
        Self {
            check_interval_seconds: 10,
            max_concurrent_calculations: 5,
            enabled: true,
            default_notification_channels: Vec::new(),
            max_results_per_schedule: 100,
        }
    }
}

/// Configuration for notification channels.
///
/// Defines how notifications are sent when scheduled calculations complete.
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

/// Configuration for Redis cache.
///
/// Controls how the application connects to and uses Redis for caching.
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
    /// Creates a default Redis cache configuration for local development.
    fn default() -> Self {
        Self {
            url: "redis://localhost:6379".to_string(),
            max_connections: 10,
            default_ttl_seconds: 3600,  // 1 hour
        }
    }
}

/// Complete application configuration.
///
/// Combines all configuration sections into a single structure.
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
    
    /// Analytics configuration (Phase 3)
    pub analytics: Option<AnalyticsConfig>,
    
    /// Visualization configuration (Phase 3)
    pub visualization: Option<VisualizationConfig>,
    
    /// Integration configuration (Phase 3)
    pub integration: Option<IntegrationConfig>,
}

impl Default for AppConfig {
    /// Creates a default application configuration suitable for local development.
    fn default() -> Self {
        Self {
            streaming: StreamingConfig::default(),
            query_api: QueryApiConfig::default(),
            scheduler: SchedulerConfig::default(),
            redis_cache: RedisCacheConfig::default(),
            aws: AwsConfig::default(),
            email: EmailConfig::default(),
            analytics: None,
            visualization: None,
            integration: None,
        }
    }
}

/// AWS services configuration.
///
/// Controls how the application connects to and uses AWS services.
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
    /// Creates a default AWS configuration using the us-east-1 region.
    fn default() -> Self {
        Self {
            region: "us-east-1".to_string(),
            notification_topic_arn: None,
            notification_queue_url: None,
        }
    }
}

/// Email notification configuration.
///
/// Controls how the application sends email notifications.
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
    /// Creates a default email configuration with no values set.
    fn default() -> Self {
        Self {
            smtp_server: None,
            smtp_port: Some(587),  // Default to TLS port
            smtp_username: None,
            smtp_password: None,
            from_address: None,
        }
    }
}

impl AppConfig {
    /// Loads configuration from a file.
    ///
    /// Supports JSON, TOML, and YAML formats based on file extension.
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the configuration file
    ///
    /// # Returns
    ///
    /// * `Result<AppConfig>` - Loaded configuration or error
    ///
    /// # Examples
    ///
    /// ```
    /// use performance_calculator::calculations::config::AppConfig;
    ///
    /// let config = AppConfig::from_file("config.json").expect("Failed to load config");
    /// ```
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let mut file = File::open(path).context("Failed to open config file")?;
        let mut contents = String::new();
        file.read_to_string(&mut contents).context("Failed to read config file")?;
        
        let config = if contents.trim().starts_with('{') {
            // JSON format
            serde_json::from_str(&contents).context("Failed to parse JSON config")?
        } else {
            // YAML format
            serde_yaml::from_str(&contents).context("Failed to parse YAML config")?
        };
        
        Ok(config)
    }
    
    /// Saves configuration to a file.
    ///
    /// Supports JSON, TOML, and YAML formats based on file extension.
    ///
    /// # Arguments
    ///
    /// * `path` - Path where the configuration file should be saved
    ///
    /// # Returns
    ///
    /// * `Result<()>` - Success or error
    ///
    /// # Examples
    ///
    /// ```
    /// use performance_calculator::calculations::config::AppConfig;
    ///
    /// let config = AppConfig::default();
    /// config.to_file("config.json").expect("Failed to save config");
    /// ```
    pub fn to_file<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let path = path.as_ref();
        let contents = if path.extension().and_then(|e| e.to_str()) == Some("json") {
            // JSON format
            serde_json::to_string_pretty(self).context("Failed to serialize config to JSON")?
        } else {
            // YAML format
            serde_yaml::to_string(self).context("Failed to serialize config to YAML")?
        };
        
        std::fs::write(path, contents).context("Failed to write config file")?;
        
        Ok(())
    }
    
    /// Loads configuration from environment variables.
    ///
    /// Environment variables take precedence over default values.
    /// Variable names are uppercase with underscores, e.g., KAFKA_BOOTSTRAP_SERVERS.
    ///
    /// # Returns
    ///
    /// * `AppConfig` - Configuration loaded from environment variables
    ///
    /// # Examples
    ///
    /// ```
    /// use performance_calculator::calculations::config::AppConfig;
    ///
    /// // Set environment variables before calling
    /// std::env::set_var("REDIS_URL", "redis://redis.example.com:6379");
    ///
    /// let config = AppConfig::from_env();
    /// assert_eq!(config.redis_cache.url, "redis://redis.example.com:6379");
    /// ```
    pub fn from_env() -> Self {
        let mut config = Self::default();
        
        // Streaming config
        if let Ok(servers) = std::env::var("KAFKA_BOOTSTRAP_SERVERS") {
            config.streaming.kafka_bootstrap_servers = Some(servers);
        }
        
        if let Ok(topics) = std::env::var("KAFKA_TOPICS") {
            // Split comma-separated list of topics
            config.streaming.kafka_topics = topics.split(',').map(String::from).collect();
        }
        
        if let Ok(group_id) = std::env::var("KAFKA_CONSUMER_GROUP_ID") {
            config.streaming.kafka_consumer_group_id = Some(group_id);
        }
        
        if let Ok(max_parallel) = std::env::var("MAX_PARALLEL_EVENTS") {
            if let Ok(value) = max_parallel.parse() {
                config.streaming.max_parallel_events = value;
            }
        }
        
        if let Ok(enabled) = std::env::var("ENABLE_STREAMING") {
            config.streaming.enabled = enabled.to_lowercase() == "true";
        }
        
        // Query API config
        if let Ok(ttl) = std::env::var("CACHE_TTL_SECONDS") {
            if let Ok(value) = ttl.parse() {
                config.query_api.cache_ttl_seconds = value;
            }
        }
        
        if let Ok(max_queries) = std::env::var("MAX_CONCURRENT_QUERIES") {
            if let Ok(value) = max_queries.parse() {
                config.query_api.max_concurrent_queries = value;
            }
        }
        
        if let Ok(timeout) = std::env::var("QUERY_TIMEOUT_SECONDS") {
            if let Ok(value) = timeout.parse() {
                config.query_api.default_query_timeout_seconds = value;
            }
        }
        
        if let Ok(enable_caching) = std::env::var("ENABLE_QUERY_CACHING") {
            config.query_api.enable_caching = enable_caching.to_lowercase() == "true";
        }
        
        // Scheduler config
        if let Ok(interval) = std::env::var("SCHEDULER_CHECK_INTERVAL") {
            if let Ok(value) = interval.parse() {
                config.scheduler.check_interval_seconds = value;
            }
        }
        
        if let Ok(max_calcs) = std::env::var("MAX_CONCURRENT_CALCULATIONS") {
            if let Ok(value) = max_calcs.parse() {
                config.scheduler.max_concurrent_calculations = value;
            }
        }
        
        if let Ok(enabled) = std::env::var("ENABLE_SCHEDULER") {
            config.scheduler.enabled = enabled.to_lowercase() == "true";
        }
        
        // Redis config
        if let Ok(url) = std::env::var("REDIS_URL") {
            config.redis_cache.url = url;
        }
        
        if let Ok(max_conn) = std::env::var("REDIS_MAX_CONNECTIONS") {
            if let Ok(value) = max_conn.parse() {
                config.redis_cache.max_connections = value;
            }
        }
        
        // AWS config
        if let Ok(region) = std::env::var("AWS_REGION") {
            config.aws.region = region;
        }
        
        if let Ok(topic_arn) = std::env::var("SNS_NOTIFICATION_TOPIC") {
            config.aws.notification_topic_arn = Some(topic_arn);
        }
        
        if let Ok(queue_url) = std::env::var("SQS_NOTIFICATION_QUEUE") {
            config.aws.notification_queue_url = Some(queue_url);
        }
        
        // Email config
        if let Ok(server) = std::env::var("SMTP_SERVER") {
            config.email.smtp_server = Some(server);
        }
        
        if let Ok(port) = std::env::var("SMTP_PORT") {
            if let Ok(value) = port.parse() {
                config.email.smtp_port = Some(value);
            }
        }
        
        if let Ok(username) = std::env::var("SMTP_USERNAME") {
            config.email.smtp_username = Some(username);
        }
        
        if let Ok(password) = std::env::var("SMTP_PASSWORD") {
            config.email.smtp_password = Some(password);
        }
        
        if let Ok(from) = std::env::var("EMAIL_FROM_ADDRESS") {
            config.email.from_address = Some(from);
        }
        
        // Load analytics configuration
        if let Ok(enabled) = env::var("ANALYTICS_ENABLED") {
            let enabled = enabled.to_lowercase() == "true";
            if enabled {
                let max_concurrent_scenarios = env::var("ANALYTICS_MAX_CONCURRENT_SCENARIOS")
                    .map(|v| v.parse::<usize>().unwrap_or(5))
                    .unwrap_or(5);
                
                let max_factors = env::var("ANALYTICS_MAX_FACTORS")
                    .map(|v| v.parse::<usize>().unwrap_or(10))
                    .unwrap_or(10);
                
                let enable_caching = env::var("ANALYTICS_ENABLE_CACHING")
                    .map(|v| v.to_lowercase() == "true")
                    .unwrap_or(true);
                
                let cache_ttl_seconds = env::var("ANALYTICS_CACHE_TTL_SECONDS")
                    .map(|v| v.parse::<u64>().unwrap_or(3600))
                    .unwrap_or(3600);
                
                config.analytics = Some(AnalyticsConfig {
                    enabled,
                    max_concurrent_scenarios,
                    max_factors,
                    enable_caching,
                    cache_ttl_seconds,
                });
            }
        }
        
        // Load visualization configuration
        if let Ok(enabled) = env::var("VISUALIZATION_ENABLED") {
            let enabled = enabled.to_lowercase() == "true";
            if enabled {
                let max_data_points = env::var("VISUALIZATION_MAX_DATA_POINTS")
                    .map(|v| v.parse::<usize>().unwrap_or(1000))
                    .unwrap_or(1000);
                
                let default_chart_width = env::var("VISUALIZATION_DEFAULT_CHART_WIDTH")
                    .map(|v| v.parse::<u32>().unwrap_or(800))
                    .unwrap_or(800);
                
                let default_chart_height = env::var("VISUALIZATION_DEFAULT_CHART_HEIGHT")
                    .map(|v| v.parse::<u32>().unwrap_or(600))
                    .unwrap_or(600);
                
                let enable_caching = env::var("VISUALIZATION_ENABLE_CACHING")
                    .map(|v| v.to_lowercase() == "true")
                    .unwrap_or(true);
                
                let cache_ttl_seconds = env::var("VISUALIZATION_CACHE_TTL_SECONDS")
                    .map(|v| v.parse::<u64>().unwrap_or(3600))
                    .unwrap_or(3600);
                
                config.visualization = Some(VisualizationConfig {
                    enabled,
                    max_data_points,
                    default_chart_width,
                    default_chart_height,
                    enable_caching,
                    cache_ttl_seconds,
                });
            }
        }
        
        // Load integration configuration
        if let Ok(enabled) = env::var("INTEGRATION_ENABLED") {
            let enabled = enabled.to_lowercase() == "true";
            if enabled {
                let enable_caching = env::var("INTEGRATION_ENABLE_CACHING")
                    .map(|v| v.to_lowercase() == "true")
                    .unwrap_or(true);
                
                let cache_ttl_seconds = env::var("INTEGRATION_CACHE_TTL_SECONDS")
                    .map(|v| v.parse::<u64>().unwrap_or(3600))
                    .unwrap_or(3600);
                
                // Create a basic integration config
                // In a real implementation, this would load more detailed configuration
                let mut integration_config = IntegrationConfig::default();
                integration_config.enabled = enabled;
                integration_config.enable_caching = enable_caching;
                integration_config.cache_ttl_seconds = cache_ttl_seconds;
                
                config.integration = Some(integration_config);
            }
        }
        
        config
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    
    #[test]
    fn test_default_config() {
        // Test that default values are set correctly
        let config = AppConfig::default();
        
        assert_eq!(config.streaming.max_parallel_events, 10);
        assert_eq!(config.query_api.cache_ttl_seconds, 3600);
        assert_eq!(config.scheduler.check_interval_seconds, 10);
        assert_eq!(config.redis_cache.url, "redis://localhost:6379");
    }
    
    #[test]
    fn test_config_serialization() {
        // Test serialization and deserialization to different formats
        let config = AppConfig::default();
        
        // Test JSON serialization
        let json_file = NamedTempFile::new().unwrap();
        let json_path = json_file.path().with_extension("json");
        config.to_file(&json_path).unwrap();
        
        let loaded_config = AppConfig::from_file(&json_path).unwrap();
        assert_eq!(loaded_config.streaming.max_parallel_events, config.streaming.max_parallel_events);
        
        // Test YAML serialization
        let yaml_file = NamedTempFile::new().unwrap();
        let yaml_path = yaml_file.path().with_extension("yaml");
        config.to_file(&yaml_path).unwrap();
        
        let loaded_config = AppConfig::from_file(&yaml_path).unwrap();
        assert_eq!(loaded_config.scheduler.check_interval_seconds, config.scheduler.check_interval_seconds);
    }
} 