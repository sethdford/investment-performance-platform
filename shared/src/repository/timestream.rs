use async_trait::async_trait;
use aws_sdk_timestreamwrite as tswrite;
use aws_sdk_timestreamquery as tsquery;
use chrono::{DateTime, NaiveDate, Utc};
use std::collections::HashMap;
use tracing::{info, error};

use crate::error::AppError;
use crate::models::{PerformanceMetric, TimeSeriesPoint};

pub struct TimestreamRepository {
    write_client: tswrite::Client,
    query_client: tsquery::Client,
    database_name: String,
    table_name: String,
}

impl TimestreamRepository {
    pub fn new(
        write_client: tswrite::Client,
        query_client: tsquery::Client,
        database_name: String,
        table_name: String,
    ) -> Self {
        Self {
            write_client,
            query_client,
            database_name,
            table_name,
        }
    }
    
    pub async fn from_env() -> Result<Self, AppError> {
        let config = aws_config::load_from_env().await;
        let write_client = tswrite::Client::new(&config);
        let query_client = tsquery::Client::new(&config);
        
        let database_name = std::env::var("TIMESTREAM_DATABASE")
            .map_err(|_| AppError::Configuration("TIMESTREAM_DATABASE environment variable not set".to_string()))?;
        
        let table_name = std::env::var("TIMESTREAM_TABLE")
            .map_err(|_| AppError::Configuration("TIMESTREAM_TABLE environment variable not set".to_string()))?;
        
        Ok(Self {
            write_client,
            query_client,
            database_name,
            table_name,
        })
    }
} 