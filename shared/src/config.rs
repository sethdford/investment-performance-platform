use std::env;
use tracing::info;

#[derive(Debug, Clone)]
pub struct AppConfig {
    pub environment: String,
    pub log_level: String,
    pub table_name: String,
    pub jwt_secret: String,
    pub timestream_database: String,
    pub timestream_table: String,
}

impl AppConfig {
    pub fn from_env() -> Self {
        let environment = env::var("ENVIRONMENT").unwrap_or_else(|_| "dev".to_string());
        let log_level = env::var("LOG_LEVEL").unwrap_or_else(|_| "INFO".to_string());
        let table_name = env::var("TABLE_NAME").unwrap_or_else(|_| "Items".to_string());
        let jwt_secret = env::var("JWT_SECRET").unwrap_or_else(|_| "default_secret_key_for_development_only".to_string());
        let timestream_database = env::var("TIMESTREAM_DATABASE").unwrap_or_else(|_| "investment_performance".to_string());
        let timestream_table = env::var("TIMESTREAM_TABLE").unwrap_or_else(|_| "performance_metrics".to_string());
        
        let config = Self {
            environment,
            log_level,
            table_name,
            jwt_secret,
            timestream_database,
            timestream_table,
        };
        
        info!("Loaded configuration: {:?}", config);
        
        config
    }
    
    pub fn is_production(&self) -> bool {
        self.environment == "prod"
    }
} 