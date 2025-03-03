use aws_config::BehaviorVersion;
use aws_sdk_dynamodb::Client as DynamoDbClient;
use aws_sdk_timestreamwrite::Client as TimestreamWriteClient;
use aws_sdk_timestreamquery::Client as TimestreamQueryClient;
use chrono::{DateTime, NaiveDate, Utc};
use shared::{
    repository::{
        dynamodb::DynamoDbRepository,
        Repository,
    },
    models::{
        Portfolio,
        Item,
        Transaction,
    },
};
use timestream_repository::{
    TimestreamRepository,
    PerformanceDataPoint,
    PerformanceTimeSeriesQuery,
    TimeSeriesInterval,
};
use performance_calculator::{
    calculator::PerformanceCalculator,
    models::{CalculationRequest, CalculationResult},
    batch_processor::{BatchProcessor, BatchCalculationRequest},
};
use uuid::Uuid;
use std::env;
use std::collections::HashMap;

// Helper function to generate a unique ID
fn generate_id() -> String {
    Uuid::new_v4().to_string()
}

// Helper function to create a test portfolio
async fn create_test_portfolio(repository: &DynamoDbRepository) -> Portfolio {
    let portfolio_id = generate_id();
    let portfolio = Portfolio {
        id: portfolio_id.clone(),
        tenant_id: "test-tenant".to_string(),
        name: format!("Test Portfolio {}", portfolio_id),
        client_id: "test-client".to_string(),
        inception_date: NaiveDate::from_ymd_opt(2022, 1, 1).unwrap(),
        benchmark_id: Some("SPY".to_string()),
        created_at: Utc::now(),
        updated_at: Utc::now(),
        status: "active".to_string(),
        metadata: HashMap::new(),
    };
    
    repository.create_portfolio(&portfolio).await.unwrap();
    
    portfolio
}

// Helper function to create test items
async fn create_test_items(repository: &DynamoDbRepository, portfolio_id: &str, count: usize) -> Vec<Item> {
    let mut items = Vec::new();
    
    for i in 0..count {
        let item_id = generate_id();
        let item = Item {
            id: item_id.clone(),
            name: format!("Test Item {}", item_id),
            description: None,
            created_at: Utc::now().to_string(),
            updated_at: None,
        };
        
        repository.create_item(&item).await.unwrap();
        
        items.push(item);
    }
    
    items
}

// Helper function to create test transactions
async fn create_test_transactions(repository: &DynamoDbRepository, portfolio_id: &str, item_id: &str, count: usize) {
    for _ in 0..count {
        let transaction = Transaction {
            id: generate_id(),
            portfolio_id: portfolio_id.to_string(),
            item_id: item_id.to_string(),
            amount: 100.0,
            date: Utc::now().to_string(),
        };
        
        repository.create_transaction(&transaction).await.unwrap();
    }
}

#[tokio::test]
async fn test_end_to_end_flow() {
    // Load AWS configuration
    let config = aws_config::from_env()
        .behavior_version(BehaviorVersion::latest())
        .load()
        .await;
        
    // Create DynamoDB client
    let dynamodb_client = DynamoDbClient::new(&config);
    
    // Create Timestream clients
    let timestream_write_client = TimestreamWriteClient::new(&config);
    let timestream_query_client = TimestreamQueryClient::new(&config);
    
    // Create repositories
    let dynamodb_repository = DynamoDbRepository::new(
        dynamodb_client,
        env::var("TABLE_NAME").unwrap_or_else(|_| "test-table".to_string()),
    );
    
    let timestream_repository = TimestreamRepository::new(
        timestream_write_client,
        timestream_query_client,
        env::var("TIMESTREAM_DATABASE_NAME").unwrap_or_else(|_| "test-database".to_string()),
        env::var("TIMESTREAM_TABLE_NAME").unwrap_or_else(|_| "test-table".to_string()),
    );
    
    // Create test portfolio
    let portfolio = create_test_portfolio(&dynamodb_repository).await;
    
    // Create test items
    let items = create_test_items(&dynamodb_repository, &portfolio.id, 3).await;
    
    // Create test transactions for each item
    for item in &items {
        create_test_transactions(&dynamodb_repository, &portfolio.id, &item.id, 10).await;
    }
    
    // Calculate performance
    let calculator = PerformanceCalculator::new(dynamodb_repository.clone());
    
    let request = CalculationRequest {
        portfolio_id: portfolio.id.clone(),
        start_date: NaiveDate::from_ymd_opt(2022, 1, 1).unwrap(),
        end_date: NaiveDate::from_ymd_opt(2022, 12, 31).unwrap(),
        include_details: true,
    };
    
    let result = calculator.calculate(request).await.unwrap();
    
    // Store performance data in Timestream
    let performance_data = PerformanceDataPoint {
        portfolio_id: portfolio.id.clone(),
        timestamp: Utc::now(),
        twr: result.twr,
        mwr: result.mwr,
        volatility: Some(result.volatility),
        sharpe_ratio: Some(result.sharpe_ratio),
        max_drawdown: Some(result.max_drawdown),
        benchmark_id: result.benchmark_id.clone(),
        benchmark_return: result.benchmark_return,
        tracking_error: result.tracking_error,
        information_ratio: result.information_ratio,
    };
    
    timestream_repository.store_performance_data(&performance_data).await.unwrap();
    
    // Store detailed performance if available
    if let Some(details) = &result.details {
        for detail in &details.time_series {
            let detail_data = PerformanceDataPoint {
                portfolio_id: portfolio.id.clone(),
                timestamp: chrono::DateTime::<Utc>::from_naive_utc_and_offset(
                    chrono::NaiveDateTime::new(detail.date, chrono::NaiveTime::from_hms_opt(0, 0, 0).unwrap()),
                    Utc,
                ),
                twr: detail.twr,
                mwr: detail.mwr,
                volatility: Some(detail.volatility),
                sharpe_ratio: None,
                max_drawdown: None,
                benchmark_id: result.benchmark_id.clone(),
                benchmark_return: detail.benchmark_return,
                tracking_error: None,
                information_ratio: None,
            };
            
            timestream_repository.store_performance_data(&detail_data).await.unwrap();
        }
    }
    
    // Query performance data
    let query = PerformanceTimeSeriesQuery {
        portfolio_id: portfolio.id.clone(),
        start_date: Utc::now() - chrono::Duration::days(365),
        end_date: Utc::now(),
        interval: TimeSeriesInterval::Monthly,
        include_benchmark: true,
    };
    
    let time_series = timestream_repository.query_performance_time_series(&query).await.unwrap();
    
    // Verify time series data
    assert!(!time_series.is_empty());
    
    // Get latest performance data
    let latest = timestream_repository.get_latest_performance_data(&portfolio.id).await.unwrap();
    
    // Verify latest data
    assert!(latest.is_some());
    let latest_data = latest.unwrap();
    assert_eq!(latest_data.portfolio_id, portfolio.id);
    assert!(latest_data.twr != 0.0);
    assert!(latest_data.mwr != 0.0);
    
    // Clean up
    dynamodb_repository.delete_portfolio(&portfolio.id).await.unwrap();
}

## Contributing

1. Fork the repository
2. Create a feature branch: `git checkout -b feature/my-feature`
3. Commit your changes: `git commit -am 'Add my feature'`
4. Push to 