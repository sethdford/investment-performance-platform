use anyhow::{Result, anyhow};
use serde::{Deserialize, Serialize};
use std::env;
use std::fs::File;
use std::io::Read;
use std::path::Path;
use tracing::info;

/// Configuration for the performance calculator
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// DynamoDB table name
    pub dynamodb_table: String,
    /// Timestream database name
    pub timestream_database: String,
    /// Timestream table name
    pub timestream_table: String,
    /// Cache configuration
    pub cache: CacheConfig,
    /// Parallel processing configuration
    pub parallel: ParallelConfig,
    /// Risk metrics configuration
    pub risk_metrics: RiskMetricsConfig,
}

/// Cache configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheConfig {
    /// Whether to enable caching
    pub enabled: bool,
    /// Cache TTL in seconds
    pub ttl_seconds: i64,
}

/// Parallel processing configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParallelConfig {
    /// Maximum concurrency for parallel processing
    pub max_concurrency: usize,
}

/// Risk metrics configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskMetricsConfig {
    /// Risk-free rate used for Sharpe and Sortino ratio calculations
    pub risk_free_rate: f64,
    /// Confidence level for Value at Risk (VaR) calculations
    pub var_confidence_level: f64,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            dynamodb_table: "performance_metrics".to_string(),
            timestream_database: "investment_performance".to_string(),
            timestream_table: "performance_metrics".to_string(),
            cache: CacheConfig {
                enabled: true,
                ttl_seconds: 300, // 5 minutes
            },
            parallel: ParallelConfig {
                max_concurrency: 5,
            },
            risk_metrics: RiskMetricsConfig {
                risk_free_rate: 0.02, // 2% annual risk-free rate
                var_confidence_level: 0.95, // 95% confidence level
            },
        }
    }
}

impl Config {
    /// Load configuration from environment variables
    pub fn from_env() -> Result<Self> {
        let mut config = Config::default();
        
        // Load from environment variables
        if let Ok(table) = env::var("DYNAMODB_TABLE") {
            config.dynamodb_table = table;
        }
        
        if let Ok(db) = env::var("TIMESTREAM_DATABASE") {
            config.timestream_database = db;
        }
        
        if let Ok(table) = env::var("TIMESTREAM_TABLE") {
            config.timestream_table = table;
        }
        
        if let Ok(enabled) = env::var("CACHE_ENABLED") {
            config.cache.enabled = enabled.to_lowercase() == "true";
        }
        
        if let Ok(ttl) = env::var("CACHE_TTL_SECONDS") {
            if let Ok(ttl) = ttl.parse::<i64>() {
                config.cache.ttl_seconds = ttl;
            }
        }
        
        if let Ok(concurrency) = env::var("PARALLEL_MAX_CONCURRENCY") {
            if let Ok(concurrency) = concurrency.parse::<usize>() {
                config.parallel.max_concurrency = concurrency;
            }
        }
        
        if let Ok(rate) = env::var("RISK_FREE_RATE") {
            if let Ok(rate) = rate.parse::<f64>() {
                config.risk_metrics.risk_free_rate = rate;
            }
        }
        
        if let Ok(level) = env::var("VAR_CONFIDENCE_LEVEL") {
            if let Ok(level) = level.parse::<f64>() {
                config.risk_metrics.var_confidence_level = level;
            }
        }
        
        Ok(config)
    }
    
    /// Load configuration from a JSON file
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let mut file = File::open(path)?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;
        
        let config: Config = serde_json::from_str(&contents)?;
        Ok(config)
    }
    
    /// Load configuration from environment variables or a JSON file if specified
    pub fn load() -> Result<Self> {
        // Check if a config file is specified
        if let Ok(config_path) = env::var("CONFIG_FILE") {
            info!("Loading configuration from file: {}", config_path);
            return Self::from_file(config_path);
        }
        
        // Otherwise, load from environment variables
        info!("Loading configuration from environment variables");
        Self::from_env()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;
    use tempfile::NamedTempFile;
    use std::io::Write;
    
    #[test]
    fn test_default_config() {
        let config = Config::default();
        
        assert_eq!(config.dynamodb_table, "performance_metrics");
        assert_eq!(config.timestream_database, "investment_performance");
        assert_eq!(config.timestream_table, "performance_metrics");
        assert!(config.cache.enabled);
        assert_eq!(config.cache.ttl_seconds, 300);
        assert_eq!(config.parallel.max_concurrency, 5);
        assert_eq!(config.risk_metrics.risk_free_rate, 0.02);
        assert_eq!(config.risk_metrics.var_confidence_level, 0.95);
    }
    
    #[test]
    fn test_config_from_env() {
        // Set environment variables
        env::set_var("DYNAMODB_TABLE", "test_table");
        env::set_var("TIMESTREAM_DATABASE", "test_db");
        env::set_var("TIMESTREAM_TABLE", "test_metrics");
        env::set_var("CACHE_ENABLED", "false");
        env::set_var("CACHE_TTL_SECONDS", "600");
        env::set_var("PARALLEL_MAX_CONCURRENCY", "10");
        env::set_var("RISK_FREE_RATE", "0.03");
        env::set_var("VAR_CONFIDENCE_LEVEL", "0.99");
        
        let config = Config::from_env().unwrap();
        
        assert_eq!(config.dynamodb_table, "test_table");
        assert_eq!(config.timestream_database, "test_db");
        assert_eq!(config.timestream_table, "test_metrics");
        assert!(!config.cache.enabled);
        assert_eq!(config.cache.ttl_seconds, 600);
        assert_eq!(config.parallel.max_concurrency, 10);
        assert_eq!(config.risk_metrics.risk_free_rate, 0.03);
        assert_eq!(config.risk_metrics.var_confidence_level, 0.99);
        
        // Clean up
        env::remove_var("DYNAMODB_TABLE");
        env::remove_var("TIMESTREAM_DATABASE");
        env::remove_var("TIMESTREAM_TABLE");
        env::remove_var("CACHE_ENABLED");
        env::remove_var("CACHE_TTL_SECONDS");
        env::remove_var("PARALLEL_MAX_CONCURRENCY");
        env::remove_var("RISK_FREE_RATE");
        env::remove_var("VAR_CONFIDENCE_LEVEL");
    }
    
    #[test]
    fn test_config_from_file() {
        // Create a temporary config file
        let mut file = NamedTempFile::new().unwrap();
        
        let config_json = r#"{
            "dynamodb_table": "file_table",
            "timestream_database": "file_db",
            "timestream_table": "file_metrics",
            "cache": {
                "enabled": false,
                "ttl_seconds": 900
            },
            "parallel": {
                "max_concurrency": 15
            },
            "risk_metrics": {
                "risk_free_rate": 0.04,
                "var_confidence_level": 0.975
            }
        }"#;
        
        file.write_all(config_json.as_bytes()).unwrap();
        
        let config = Config::from_file(file.path()).unwrap();
        
        assert_eq!(config.dynamodb_table, "file_table");
        assert_eq!(config.timestream_database, "file_db");
        assert_eq!(config.timestream_table, "file_metrics");
        assert!(!config.cache.enabled);
        assert_eq!(config.cache.ttl_seconds, 900);
        assert_eq!(config.parallel.max_concurrency, 15);
        assert_eq!(config.risk_metrics.risk_free_rate, 0.04);
        assert_eq!(config.risk_metrics.var_confidence_level, 0.975);
    }
} 