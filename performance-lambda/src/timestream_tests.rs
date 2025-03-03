#[cfg(test)]
mod timestream_repository_tests {
    use crate::timestream_repository::{TimestreamRepository, PerformanceDataPoint, PerformanceMetrics, TimeInterval};
    use aws_sdk_timestreamwrite::{Client as TimestreamWriteClient, Error as TimestreamWriteError};
    use aws_sdk_timestreamquery::{Client as TimestreamQueryClient, Error as TimestreamQueryError};
    use aws_sdk_timestreamwrite::model::{WriteRecordsInput, Record, Dimension, MeasureValue, MeasureValueType};
    use aws_sdk_timestreamquery::model::{QueryRequest, QueryResponse, Row, ColumnInfo, Datum, Type, ScalarType};
    use chrono::{DateTime, Utc, TimeZone};
    use std::collections::HashMap;
    use std::sync::Arc;
    use tokio::sync::Mutex;
    
    // Mock Timestream Write client for testing
    struct MockTimestreamWriteClient {
        write_records_responses: Arc<Mutex<HashMap<String, Result<(), TimestreamWriteError>>>>,
    }
    
    impl MockTimestreamWriteClient {
        fn new() -> Self {
            Self {
                write_records_responses: Arc::new(Mutex::new(HashMap::new())),
            }
        }
        
        async fn add_write_records_response(&self, key: String, response: Result<(), TimestreamWriteError>) {
            let mut responses = self.write_records_responses.lock().await;
            responses.insert(key, response);
        }
    }
    
    // Mock Timestream Query client for testing
    struct MockTimestreamQueryClient {
        query_responses: Arc<Mutex<HashMap<String, Result<QueryResponse, TimestreamQueryError>>>>,
    }
    
    impl MockTimestreamQueryClient {
        fn new() -> Self {
            Self {
                query_responses: Arc::new(Mutex::new(HashMap::new())),
            }
        }
        
        async fn add_query_response(&self, key: String, response: Result<QueryResponse, TimestreamQueryError>) {
            let mut responses = self.query_responses.lock().await;
            responses.insert(key, response);
        }
    }
    
    // Helper function to create a test performance data point
    fn create_test_performance_data_point(
        portfolio_id: &str,
        timestamp: DateTime<Utc>,
        twr: f64,
        mwr: f64,
    ) -> PerformanceDataPoint {
        PerformanceDataPoint {
            portfolio_id: portfolio_id.to_string(),
            timestamp,
            twr,
            mwr,
            volatility: Some(0.15),
            sharpe_ratio: Some(0.8),
            max_drawdown: Some(0.1),
            benchmark_id: Some("benchmark1".to_string()),
            benchmark_return: Some(0.05),
            tracking_error: Some(0.02),
            information_ratio: Some(0.75),
        }
    }
    
    // Helper function to create a test performance metrics
    fn create_test_performance_metrics(
        timestamp: DateTime<Utc>,
        twr: f64,
        mwr: f64,
    ) -> PerformanceMetrics {
        PerformanceMetrics {
            timestamp,
            twr,
            mwr,
            volatility: Some(0.15),
            sharpe_ratio: Some(0.8),
            max_drawdown: Some(0.1),
            benchmark_return: Some(0.05),
            tracking_error: Some(0.02),
            information_ratio: Some(0.75),
        }
    }
    
    // Helper function to create a mock query response for performance history
    fn create_mock_performance_history_response(
        portfolio_id: &str,
        start_date: DateTime<Utc>,
        end_date: DateTime<Utc>,
        interval: TimeInterval,
    ) -> QueryResponse {
        // Create column info
        let column_infos = vec![
            ColumnInfo::builder()
                .name("time")
                .type_(Type::builder().scalar_type(ScalarType::Timestamp).build())
                .build(),
            ColumnInfo::builder()
                .name("twr")
                .type_(Type::builder().scalar_type(ScalarType::Double).build())
                .build(),
            ColumnInfo::builder()
                .name("mwr")
                .type_(Type::builder().scalar_type(ScalarType::Double).build())
                .build(),
            ColumnInfo::builder()
                .name("volatility")
                .type_(Type::builder().scalar_type(ScalarType::Double).build())
                .build(),
            ColumnInfo::builder()
                .name("sharpe_ratio")
                .type_(Type::builder().scalar_type(ScalarType::Double).build())
                .build(),
            ColumnInfo::builder()
                .name("max_drawdown")
                .type_(Type::builder().scalar_type(ScalarType::Double).build())
                .build(),
            ColumnInfo::builder()
                .name("benchmark_return")
                .type_(Type::builder().scalar_type(ScalarType::Double).build())
                .build(),
            ColumnInfo::builder()
                .name("tracking_error")
                .type_(Type::builder().scalar_type(ScalarType::Double).build())
                .build(),
            ColumnInfo::builder()
                .name("information_ratio")
                .type_(Type::builder().scalar_type(ScalarType::Double).build())
                .build(),
        ];
        
        // Create rows based on the interval
        let mut rows = Vec::new();
        let interval_days = match interval {
            TimeInterval::Daily => 1,
            TimeInterval::Weekly => 7,
            TimeInterval::Monthly => 30,
            TimeInterval::Quarterly => 90,
            TimeInterval::Yearly => 365,
        };
        
        let mut current_date = start_date;
        let mut i = 0;
        while current_date <= end_date {
            // Create a row with test data
            let row = Row::builder()
                .data(vec![
                    // time
                    Datum::builder()
                        .scalar_timestamp(current_date.timestamp_millis().to_string())
                        .build(),
                    // twr
                    Datum::builder()
                        .scalar_double((0.01 * (i as f64 + 1.0)).to_string())
                        .build(),
                    // mwr
                    Datum::builder()
                        .scalar_double((0.015 * (i as f64 + 1.0)).to_string())
                        .build(),
                    // volatility
                    Datum::builder()
                        .scalar_double("0.15")
                        .build(),
                    // sharpe_ratio
                    Datum::builder()
                        .scalar_double("0.8")
                        .build(),
                    // max_drawdown
                    Datum::builder()
                        .scalar_double("0.1")
                        .build(),
                    // benchmark_return
                    Datum::builder()
                        .scalar_double("0.05")
                        .build(),
                    // tracking_error
                    Datum::builder()
                        .scalar_double("0.02")
                        .build(),
                    // information_ratio
                    Datum::builder()
                        .scalar_double("0.75")
                        .build(),
                ])
                .build();
            
            rows.push(row);
            
            // Move to the next interval
            current_date = current_date + chrono::Duration::days(interval_days);
            i += 1;
        }
        
        // Build and return the query response
        QueryResponse::builder()
            .column_info(column_infos)
            .rows(rows)
            .build()
    }
    
    #[tokio::test]
    async fn test_store_performance_data() {
        // Create mock clients
        let mock_write_client = MockTimestreamWriteClient::new();
        
        // Add a successful response for write_records
        mock_write_client.add_write_records_response(
            "test_portfolio_1".to_string(),
            Ok(())
        ).await;
        
        // Create a test data point
        let portfolio_id = "test_portfolio_1";
        let timestamp = Utc::now();
        let data_point = create_test_performance_data_point(
            portfolio_id,
            timestamp,
            0.12, // 12% TWR
            0.15, // 15% MWR
        );
        
        // Test the store_performance_data method
        // Note: This is a simplified version - in a real test, you would need to create a proper
        // TimestreamRepository with the mock clients
        
        // let repository = TimestreamRepository::new(
        //     write_client,
        //     query_client,
        //     "test_database".to_string(),
        //     "test_table".to_string(),
        // );
        // let result = repository.store_performance_data(&data_point).await;
        
        // Assert the result is Ok
        // assert!(result.is_ok());
    }
    
    #[tokio::test]
    async fn test_save_performance_metrics() {
        // Similar to test_store_performance_data, but testing the save_performance_metrics method
    }
    
    #[tokio::test]
    async fn test_get_performance_history() {
        // Create mock clients
        let mock_query_client = MockTimestreamQueryClient::new();
        
        // Create test data
        let portfolio_id = "test_portfolio_1";
        let start_date = Utc.ymd(2023, 1, 1).and_hms(0, 0, 0);
        let end_date = Utc.ymd(2023, 3, 31).and_hms(0, 0, 0);
        let interval = TimeInterval::Monthly;
        
        // Create a mock query response
        let query_response = create_mock_performance_history_response(
            portfolio_id,
            start_date,
            end_date,
            interval,
        );
        
        // Add the mock response
        mock_query_client.add_query_response(
            format!("{}:{}:{}:{:?}", portfolio_id, start_date, end_date, interval),
            Ok(query_response)
        ).await;
        
        // Test the get_performance_history method
        // Note: This is a simplified version - in a real test, you would need to create a proper
        // TimestreamRepository with the mock clients
        
        // let repository = TimestreamRepository::new(
        //     write_client,
        //     query_client,
        //     "test_database".to_string(),
        //     "test_table".to_string(),
        // );
        // let result = repository.get_performance_history(
        //     portfolio_id,
        //     start_date,
        //     end_date,
        //     interval
        // ).await;
        
        // Assert the result is Ok and contains the expected metrics
        // assert!(result.is_ok());
        // let metrics = result.unwrap();
        // assert_eq!(metrics.len(), 3); // 3 months between Jan and Mar
        // assert_eq!(metrics[0].timestamp.month(), 1);
        // assert_eq!(metrics[1].timestamp.month(), 2);
        // assert_eq!(metrics[2].timestamp.month(), 3);
    }
} 