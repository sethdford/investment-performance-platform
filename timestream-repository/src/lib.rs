use aws_sdk_timestreamwrite::{Client as TimestreamWriteClient, types::{Dimension, MeasureValue, Record, TimeUnit}};
use aws_sdk_timestreamquery::{Client as TimestreamQueryClient};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::env;
use thiserror::Error;
use tracing::{info, error};
use uuid::Uuid;

#[derive(Debug, Error)]
pub enum TimestreamError {
    #[error("Timestream write error: {0}")]
    Write(#[from] aws_sdk_timestreamwrite::Error),
    
    #[error("Timestream query error: {0}")]
    Query(#[from] aws_sdk_timestreamquery::Error),
    
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
    
    #[error("Internal error: {0}")]
    Internal(String),
}

pub type Result<T> = std::result::Result<T, TimestreamError>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceDataPoint {
    pub portfolio_id: String,
    pub timestamp: DateTime<Utc>,
    pub twr: f64,
    pub mwr: f64,
    pub volatility: Option<f64>,
    pub sharpe_ratio: Option<f64>,
    pub max_drawdown: Option<f64>,
    pub benchmark_id: Option<String>,
    pub benchmark_return: Option<f64>,
    pub tracking_error: Option<f64>,
    pub information_ratio: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceTimeSeriesQuery {
    pub portfolio_id: String,
    pub start_date: DateTime<Utc>,
    pub end_date: DateTime<Utc>,
    pub interval: TimeSeriesInterval,
    pub include_benchmark: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TimeSeriesInterval {
    Daily,
    Weekly,
    Monthly,
    Quarterly,
    Yearly,
}

impl TimeSeriesInterval {
    fn to_sql_interval(&self) -> &'static str {
        match self {
            Self::Daily => "1d",
            Self::Weekly => "1w",
            Self::Monthly => "1mo",
            Self::Quarterly => "3mo",
            Self::Yearly => "1y",
        }
    }
}

pub struct TimestreamRepository {
    write_client: TimestreamWriteClient,
    query_client: TimestreamQueryClient,
    database_name: String,
    table_name: String,
}

impl TimestreamRepository {
    pub fn new(
        write_client: TimestreamWriteClient,
        query_client: TimestreamQueryClient,
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
    
    pub async fn from_env() -> Result<Self> {
        let config = aws_config::load_from_env().await;
        let write_client = TimestreamWriteClient::new(&config);
        let query_client = TimestreamQueryClient::new(&config);
        
        let database_name = env::var("TIMESTREAM_DATABASE")
            .map_err(|_| TimestreamError::Internal("TIMESTREAM_DATABASE environment variable not set".to_string()))?;
        
        let table_name = env::var("TIMESTREAM_TABLE")
            .map_err(|_| TimestreamError::Internal("TIMESTREAM_TABLE environment variable not set".to_string()))?;
        
        Ok(Self {
            write_client,
            query_client,
            database_name,
            table_name,
        })
    }
    
    pub async fn store_performance_data(&self, data_point: &PerformanceDataPoint) -> Result<()> {
        let dimensions = vec![
            Dimension::builder()
                .name("portfolio_id")
                .value(&data_point.portfolio_id)
                .build(),
        ];
        
        let mut measures = vec![
            Record::builder()
                .dimensions(dimensions.clone())
                .measure_name("twr")
                .measure_value(data_point.twr.to_string())
                .measure_value_type(MeasureValue::Double)
                .time(data_point.timestamp.timestamp_millis().to_string())
                .time_unit(TimeUnit::Milliseconds)
                .build(),
            
            Record::builder()
                .dimensions(dimensions.clone())
                .measure_name("mwr")
                .measure_value(data_point.mwr.to_string())
                .measure_value_type(MeasureValue::Double)
                .time(data_point.timestamp.timestamp_millis().to_string())
                .time_unit(TimeUnit::Milliseconds)
                .build(),
        ];
        
        // Add optional measures if they exist
        if let Some(volatility) = data_point.volatility {
            measures.push(
                Record::builder()
                    .dimensions(dimensions.clone())
                    .measure_name("volatility")
                    .measure_value(volatility.to_string())
                    .measure_value_type(MeasureValue::Double)
                    .time(data_point.timestamp.timestamp_millis().to_string())
                    .time_unit(TimeUnit::Milliseconds)
                    .build()
            );
        }
        
        if let Some(sharpe_ratio) = data_point.sharpe_ratio {
            measures.push(
                Record::builder()
                    .dimensions(dimensions.clone())
                    .measure_name("sharpe_ratio")
                    .measure_value(sharpe_ratio.to_string())
                    .measure_value_type(MeasureValue::Double)
                    .time(data_point.timestamp.timestamp_millis().to_string())
                    .time_unit(TimeUnit::Milliseconds)
                    .build()
            );
        }
        
        if let Some(max_drawdown) = data_point.max_drawdown {
            measures.push(
                Record::builder()
                    .dimensions(dimensions.clone())
                    .measure_name("max_drawdown")
                    .measure_value(max_drawdown.to_string())
                    .measure_value_type(MeasureValue::Double)
                    .time(data_point.timestamp.timestamp_millis().to_string())
                    .time_unit(TimeUnit::Milliseconds)
                    .build()
            );
        }
        
        // Add benchmark related measures if they exist
        if let Some(benchmark_id) = &data_point.benchmark_id {
            let benchmark_dimensions = vec![
                Dimension::builder()
                    .name("portfolio_id")
                    .value(&data_point.portfolio_id)
                    .build(),
                Dimension::builder()
                    .name("benchmark_id")
                    .value(benchmark_id)
                    .build(),
            ];
            
            if let Some(benchmark_return) = data_point.benchmark_return {
                measures.push(
                    Record::builder()
                        .dimensions(benchmark_dimensions.clone())
                        .measure_name("benchmark_return")
                        .measure_value(benchmark_return.to_string())
                        .measure_value_type(MeasureValue::Double)
                        .time(data_point.timestamp.timestamp_millis().to_string())
                        .time_unit(TimeUnit::Milliseconds)
                        .build()
                );
            }
            
            if let Some(tracking_error) = data_point.tracking_error {
                measures.push(
                    Record::builder()
                        .dimensions(benchmark_dimensions.clone())
                        .measure_name("tracking_error")
                        .measure_value(tracking_error.to_string())
                        .measure_value_type(MeasureValue::Double)
                        .time(data_point.timestamp.timestamp_millis().to_string())
                        .time_unit(TimeUnit::Milliseconds)
                        .build()
                );
            }
            
            if let Some(information_ratio) = data_point.information_ratio {
                measures.push(
                    Record::builder()
                        .dimensions(benchmark_dimensions.clone())
                        .measure_name("information_ratio")
                        .measure_value(information_ratio.to_string())
                        .measure_value_type(MeasureValue::Double)
                        .time(data_point.timestamp.timestamp_millis().to_string())
                        .time_unit(TimeUnit::Milliseconds)
                        .build()
                );
            }
        }
        
        self.write_client.write_records()
            .database_name(&self.database_name)
            .table_name(&self.table_name)
            .set_records(Some(measures))
            .send()
            .await?;
        
        Ok(())
    }
    
    pub async fn query_performance_time_series(&self, query: &PerformanceTimeSeriesQuery) -> Result<Vec<PerformanceDataPoint>> {
        let start_date = query.start_date.to_rfc3339();
        let end_date = query.end_date.to_rfc3339();
        let interval = query.interval.to_sql_interval();
        
        let sql = format!(
            r#"
            SELECT 
                bin(time, {interval}) AS time_bin,
                portfolio_id,
                ROUND(AVG(CASE WHEN measure_name = 'twr' THEN measure_value::double ELSE NULL END), 6) AS twr,
                ROUND(AVG(CASE WHEN measure_name = 'mwr' THEN measure_value::double ELSE NULL END), 6) AS mwr,
                ROUND(AVG(CASE WHEN measure_name = 'volatility' THEN measure_value::double ELSE NULL END), 6) AS volatility,
                ROUND(AVG(CASE WHEN measure_name = 'sharpe_ratio' THEN measure_value::double ELSE NULL END), 6) AS sharpe_ratio,
                ROUND(AVG(CASE WHEN measure_name = 'max_drawdown' THEN measure_value::double ELSE NULL END), 6) AS max_drawdown
                {benchmark_fields}
            FROM "{database}"."{table}"
            WHERE portfolio_id = '{portfolio_id}'
                AND time BETWEEN '{start_date}' AND '{end_date}'
            GROUP BY bin(time, {interval}), portfolio_id
            ORDER BY time_bin ASC
            "#,
            interval = interval,
            database = self.database_name,
            table = self.table_name,
            portfolio_id = query.portfolio_id,
            start_date = start_date,
            end_date = end_date,
            benchmark_fields = if query.include_benchmark {
                r#",
                MAX(benchmark_id) AS benchmark_id,
                ROUND(AVG(CASE WHEN measure_name = 'benchmark_return' THEN measure_value::double ELSE NULL END), 6) AS benchmark_return,
                ROUND(AVG(CASE WHEN measure_name = 'tracking_error' THEN measure_value::double ELSE NULL END), 6) AS tracking_error,
                ROUND(AVG(CASE WHEN measure_name = 'information_ratio' THEN measure_value::double ELSE NULL END), 6) AS information_ratio"#
            } else {
                ""
            }
        );
        
        let query_result = self.query_client.query()
            .query_string(sql)
            .send()
            .await?;
        
        let mut data_points = Vec::new();
        
        if let Some(result) = query_result.query_status() {
            info!(
                "Query execution time: {} ms, Data scanned: {} bytes",
                result.cumulative_execution_time_in_millis().unwrap_or_default(),
                result.cumulative_bytes_scanned().unwrap_or_default()
            );
        }
        
        if let Some(rows) = query_result.rows() {
            for row in rows {
                if let Some(data) = row.data() {
                    // Parse timestamp
                    let timestamp = data.get(0)
                        .and_then(|d| d.scalar_value())
                        .ok_or_else(|| TimestreamError::Internal("Missing timestamp".to_string()))?;
                    
                    let timestamp = DateTime::parse_from_rfc3339(timestamp)
                        .map_err(|e| TimestreamError::Internal(format!("Invalid timestamp: {}", e)))?
                        .with_timezone(&Utc);
                    
                    // Parse portfolio_id
                    let portfolio_id = data.get(1)
                        .and_then(|d| d.scalar_value())
                        .ok_or_else(|| TimestreamError::Internal("Missing portfolio_id".to_string()))?
                        .to_string();
                    
                    // Parse twr
                    let twr = data.get(2)
                        .and_then(|d| d.scalar_value())
                        .and_then(|v| v.parse::<f64>().ok())
                        .unwrap_or(0.0);
                    
                    // Parse mwr
                    let mwr = data.get(3)
                        .and_then(|d| d.scalar_value())
                        .and_then(|v| v.parse::<f64>().ok())
                        .unwrap_or(0.0);
                    
                    // Parse volatility
                    let volatility = data.get(4)
                        .and_then(|d| d.scalar_value())
                        .and_then(|v| v.parse::<f64>().ok());
                    
                    // Parse sharpe_ratio
                    let sharpe_ratio = data.get(5)
                        .and_then(|d| d.scalar_value())
                        .and_then(|v| v.parse::<f64>().ok());
                    
                    // Parse max_drawdown
                    let max_drawdown = data.get(6)
                        .and_then(|d| d.scalar_value())
                        .and_then(|v| v.parse::<f64>().ok());
                    
                    // Parse benchmark fields if included
                    let (benchmark_id, benchmark_return, tracking_error, information_ratio) = 
                        if query.include_benchmark && data.len() > 7 {
                            (
                                data.get(7).and_then(|d| d.scalar_value()).map(|s| s.to_string()),
                                data.get(8).and_then(|d| d.scalar_value()).and_then(|v| v.parse::<f64>().ok()),
                                data.get(9).and_then(|d| d.scalar_value()).and_then(|v| v.parse::<f64>().ok()),
                                data.get(10).and_then(|d| d.scalar_value()).and_then(|v| v.parse::<f64>().ok()),
                            )
                        } else {
                            (None, None, None, None)
                        };
                    
                    data_points.push(PerformanceDataPoint {
                        portfolio_id,
                        timestamp,
                        twr,
                        mwr,
                        volatility,
                        sharpe_ratio,
                        max_drawdown,
                        benchmark_id,
                        benchmark_return,
                        tracking_error,
                        information_ratio,
                    });
                }
            }
        }
        
        Ok(data_points)
    }
    
    pub async fn get_latest_performance_data(&self, portfolio_id: &str) -> Result<Option<PerformanceDataPoint>> {
        let sql = format!(
            r#"
            SELECT 
                time,
                portfolio_id,
                ROUND(AVG(CASE WHEN measure_name = 'twr' THEN measure_value::double ELSE NULL END), 6) AS twr,
                ROUND(AVG(CASE WHEN measure_name = 'mwr' THEN measure_value::double ELSE NULL END), 6) AS mwr,
                ROUND(AVG(CASE WHEN measure_name = 'volatility' THEN measure_value::double ELSE NULL END), 6) AS volatility,
                ROUND(AVG(CASE WHEN measure_name = 'sharpe_ratio' THEN measure_value::double ELSE NULL END), 6) AS sharpe_ratio,
                ROUND(AVG(CASE WHEN measure_name = 'max_drawdown' THEN measure_value::double ELSE NULL END), 6) AS max_drawdown,
                MAX(benchmark_id) AS benchmark_id,
                ROUND(AVG(CASE WHEN measure_name = 'benchmark_return' THEN measure_value::double ELSE NULL END), 6) AS benchmark_return,
                ROUND(AVG(CASE WHEN measure_name = 'tracking_error' THEN measure_value::double ELSE NULL END), 6) AS tracking_error,
                ROUND(AVG(CASE WHEN measure_name = 'information_ratio' THEN measure_value::double ELSE NULL END), 6) AS information_ratio
            FROM "{database}"."{table}"
            WHERE portfolio_id = '{portfolio_id}'
            GROUP BY time, portfolio_id
            ORDER BY time DESC
            LIMIT 1
            "#,
            database = self.database_name,
            table = self.table_name,
            portfolio_id = portfolio_id,
        );
        
        let query_result = self.query_client.query()
            .query_string(sql)
            .send()
            .await?;
        
        if let Some(rows) = query_result.rows() {
            if let Some(row) = rows.first() {
                if let Some(data) = row.data() {
                    // Parse timestamp
                    let timestamp = data.get(0)
                        .and_then(|d| d.scalar_value())
                        .ok_or_else(|| TimestreamError::Internal("Missing timestamp".to_string()))?;
                    
                    let timestamp = DateTime::parse_from_rfc3339(timestamp)
                        .map_err(|e| TimestreamError::Internal(format!("Invalid timestamp: {}", e)))?
                        .with_timezone(&Utc);
                    
                    // Parse portfolio_id
                    let portfolio_id = data.get(1)
                        .and_then(|d| d.scalar_value())
                        .ok_or_else(|| TimestreamError::Internal("Missing portfolio_id".to_string()))?
                        .to_string();
                    
                    // Parse twr
                    let twr = data.get(2)
                        .and_then(|d| d.scalar_value())
                        .and_then(|v| v.parse::<f64>().ok())
                        .unwrap_or(0.0);
                    
                    // Parse mwr
                    let mwr = data.get(3)
                        .and_then(|d| d.scalar_value())
                        .and_then(|v| v.parse::<f64>().ok())
                        .unwrap_or(0.0);
                    
                    // Parse volatility
                    let volatility = data.get(4)
                        .and_then(|d| d.scalar_value())
                        .and_then(|v| v.parse::<f64>().ok());
                    
                    // Parse sharpe_ratio
                    let sharpe_ratio = data.get(5)
                        .and_then(|d| d.scalar_value())
                        .and_then(|v| v.parse::<f64>().ok());
                    
                    // Parse max_drawdown
                    let max_drawdown = data.get(6)
                        .and_then(|d| d.scalar_value())
                        .and_then(|v| v.parse::<f64>().ok());
                    
                    // Parse benchmark_id
                    let benchmark_id = data.get(7)
                        .and_then(|d| d.scalar_value())
                        .map(|s| s.to_string());
                    
                    // Parse benchmark_return
                    let benchmark_return = data.get(8)
                        .and_then(|d| d.scalar_value())
                        .and_then(|v| v.parse::<f64>().ok());
                    
                    // Parse tracking_error
                    let tracking_error = data.get(9)
                        .and_then(|d| d.scalar_value())
                        .and_then(|v| v.parse::<f64>().ok());
                    
                    // Parse information_ratio
                    let information_ratio = data.get(10)
                        .and_then(|d| d.scalar_value())
                        .and_then(|v| v.parse::<f64>().ok());
                    
                    return Ok(Some(PerformanceDataPoint {
                        portfolio_id,
                        timestamp,
                        twr,
                        mwr,
                        volatility,
                        sharpe_ratio,
                        max_drawdown,
                        benchmark_id,
                        benchmark_return,
                        tracking_error,
                        information_ratio,
                    }));
                }
            }
        }
        
        Ok(None)
    }
    
    pub async fn get_performance_summary(&self, portfolio_id: &str, start_date: &DateTime<Utc>, end_date: &DateTime<Utc>) -> Result<Option<PerformanceDataPoint>> {
        let start_date_str = start_date.to_rfc3339();
        let end_date_str = end_date.to_rfc3339();
        
        let sql = format!(
            r#"
            SELECT 
                '{end_date_str}' as time,
                portfolio_id,
                ROUND(AVG(CASE WHEN measure_name = 'twr' THEN measure_value::double ELSE NULL END), 6) AS twr,
                ROUND(AVG(CASE WHEN measure_name = 'mwr' THEN measure_value::double ELSE NULL END), 6) AS mwr,
                ROUND(AVG(CASE WHEN measure_name = 'volatility' THEN measure_value::double ELSE NULL END), 6) AS volatility,
                ROUND(AVG(CASE WHEN measure_name = 'sharpe_ratio' THEN measure_value::double ELSE NULL END), 6) AS sharpe_ratio,
                ROUND(AVG(CASE WHEN measure_name = 'max_drawdown' THEN measure_value::double ELSE NULL END), 6) AS max_drawdown,
                MAX(benchmark_id) AS benchmark_id,
                ROUND(AVG(CASE WHEN measure_name = 'benchmark_return' THEN measure_value::double ELSE NULL END), 6) AS benchmark_return,
                ROUND(AVG(CASE WHEN measure_name = 'tracking_error' THEN measure_value::double ELSE NULL END), 6) AS tracking_error,
                ROUND(AVG(CASE WHEN measure_name = 'information_ratio' THEN measure_value::double ELSE NULL END), 6) AS information_ratio
            FROM "{database}"."{table}"
            WHERE portfolio_id = '{portfolio_id}'
                AND time BETWEEN '{start_date_str}' AND '{end_date_str}'
            GROUP BY portfolio_id
            "#,
            database = self.database_name,
            table = self.table_name,
            portfolio_id = portfolio_id,
            start_date_str = start_date_str,
            end_date_str = end_date_str,
        );
        
        let query_result = self.query_client.query()
            .query_string(sql)
            .send()
            .await?;
        
        if let Some(rows) = query_result.rows() {
            if let Some(row) = rows.first() {
                if let Some(data) = row.data() {
                    // Parse timestamp (using end_date)
                    let timestamp = *end_date;
                    
                    // Parse portfolio_id
                    let portfolio_id = data.get(1)
                        .and_then(|d| d.scalar_value())
                        .ok_or_else(|| TimestreamError::Internal("Missing portfolio_id".to_string()))?
                        .to_string();
                    
                    // Parse twr
                    let twr = data.get(2)
                        .and_then(|d| d.scalar_value())
                        .and_then(|v| v.parse::<f64>().ok())
                        .unwrap_or(0.0);
                    
                    // Parse mwr
                    let mwr = data.get(3)
                        .and_then(|d| d.scalar_value())
                        .and_then(|v| v.parse::<f64>().ok())
                        .unwrap_or(0.0);
                    
                    // Parse volatility
                    let volatility = data.get(4)
                        .and_then(|d| d.scalar_value())
                        .and_then(|v| v.parse::<f64>().ok());
                    
                    // Parse sharpe_ratio
                    let sharpe_ratio = data.get(5)
                        .and_then(|d| d.scalar_value())
                        .and_then(|v| v.parse::<f64>().ok());
                    
                    // Parse max_drawdown
                    let max_drawdown = data.get(6)
                        .and_then(|d| d.scalar_value())
                        .and_then(|v| v.parse::<f64>().ok());
                    
                    // Parse benchmark_id
                    let benchmark_id = data.get(7)
                        .and_then(|d| d.scalar_value())
                        .map(|s| s.to_string());
                    
                    // Parse benchmark_return
                    let benchmark_return = data.get(8)
                        .and_then(|d| d.scalar_value())
                        .and_then(|v| v.parse::<f64>().ok());
                    
                    // Parse tracking_error
                    let tracking_error = data.get(9)
                        .and_then(|d| d.scalar_value())
                        .and_then(|v| v.parse::<f64>().ok());
                    
                    // Parse information_ratio
                    let information_ratio = data.get(10)
                        .and_then(|d| d.scalar_value())
                        .and_then(|v| v.parse::<f64>().ok());
                    
                    return Ok(Some(PerformanceDataPoint {
                        portfolio_id,
                        timestamp,
                        twr,
                        mwr,
                        volatility,
                        sharpe_ratio,
                        max_drawdown,
                        benchmark_id,
                        benchmark_return,
                        tracking_error,
                        information_ratio,
                    }));
                }
            }
        }
        
        Ok(None)
    }
} 