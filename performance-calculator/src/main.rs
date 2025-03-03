use lambda_runtime::{run, service_fn, Error, LambdaEvent};
use aws_sdk_dynamodb::Client as DynamoDbClient;
use aws_sdk_timestreamwrite::Client as TimestreamWriteClient;
use aws_config::BehaviorVersion;
use serde::{Deserialize, Serialize};
use tracing::{info, error};
use shared::models::{Transaction, PerformanceMetric};
use shared::repository::{Repository, DynamoDbRepository};
use chrono::{DateTime, Utc, Duration};
use uuid::Uuid;
use std::collections::HashMap;
use anyhow::{Result, anyhow};
use rust_decimal::Decimal;
use rust_decimal::prelude::*;
use std::sync::Arc;
use aws_lambda_events::event::sqs::{SqsEvent, SqsMessage};
use serde_json::json;

// Import our calculation modules
mod calculations;
mod config;

use calculations::{
    TimeWeightedReturn, 
    MoneyWeightedReturn,
    calculate_modified_dietz,
    calculate_daily_twr,
    calculate_irr,
    RiskMetrics,
    calculate_risk_metrics,
    CalculationCache,
    create_performance_cache,
    with_db_retry
};
use config::Config;
use calculations::factory::ComponentFactory;
use calculations::streaming::StreamingEvent;
use calculations::query_api::PerformanceQuery;

// Global configuration
static mut CONFIG: Option<Arc<Config>> = None;
// Global cache
static mut CACHE: Option<Arc<CalculationCache<String, serde_json::Value>>> = None;

#[derive(Debug, Deserialize)]
struct PerformanceCalculationEvent {
    #[serde(rename = "type")]
    event_type: String,
    transaction_id: Option<String>,
    account_id: Option<String>,
    portfolio_id: Option<String>,
    portfolio_ids: Option<Vec<String>>,
    account_ids: Option<Vec<String>>,
    security_id: Option<String>,
    transaction_date: Option<String>,
    benchmark_id: Option<String>,
    start_date: Option<String>,
    end_date: Option<String>,
    periods: Option<Vec<String>>,
    request_id: String,
}

#[derive(Debug, Serialize)]
struct Response {
    request_id: String,
    status: String,
    message: String,
}

async fn function_handler(event: LambdaEvent<PerformanceCalculationEvent>) -> Result<Response, Error> {
    let (event, context) = event.into_parts();
    let request_id = event.request_id.clone();
    
    info!(request_id = %request_id, event_type = %event.event_type, "Processing performance calculation event");
    
    // Initialize AWS clients
    let config = aws_config::defaults(BehaviorVersion::latest())
        .load()
        .await;
    
    let dynamodb_client = DynamoDbClient::new(&config);
    let timestream_client = TimestreamWriteClient::new(&config);
    
    // Get table name from environment variable
    let table_name = std::env::var("DYNAMODB_TABLE")
        .map_err(|_| anyhow!("DYNAMODB_TABLE environment variable not set"))?;
    
    // Get Timestream database and table names from environment variables
    let timestream_database = std::env::var("TIMESTREAM_DATABASE")
        .map_err(|_| anyhow!("TIMESTREAM_DATABASE environment variable not set"))?;
    
    let timestream_table = std::env::var("TIMESTREAM_TABLE")
        .map_err(|_| anyhow!("TIMESTREAM_TABLE environment variable not set"))?;
    
    // Create repository
    let repository = DynamoDbRepository::new(dynamodb_client, table_name);
    
    // Process the event based on its type
    match event.event_type.as_str() {
        "transaction_processed" => {
            if let (Some(transaction_id), Some(account_id)) = (&event.transaction_id, &event.account_id) {
                calculate_performance_after_transaction(
                    &repository,
                    &timestream_client,
                    &timestream_database,
                    &timestream_table,
                    transaction_id,
                    account_id,
                    &event.security_id,
                    &event.transaction_date,
                    &request_id
                ).await?;
            } else {
                return Ok(Response {
                    request_id,
                    status: "error".to_string(),
                    message: "Missing required fields for transaction_processed event".to_string(),
                });
            }
        },
        "calculate_portfolio_performance" => {
            if let Some(portfolio_id) = &event.portfolio_id {
                calculate_portfolio_performance(
                    &repository,
                    &timestream_client,
                    &timestream_database,
                    &timestream_table,
                    portfolio_id,
                    &request_id
                ).await?;
            } else {
                return Ok(Response {
                    request_id,
                    status: "error".to_string(),
                    message: "Missing portfolio_id for calculate_portfolio_performance event".to_string(),
                });
            }
        },
        "batch_calculate_portfolio_performance" => {
            if let Some(portfolio_ids) = &event.portfolio_ids {
                if portfolio_ids.is_empty() {
                    return Ok(Response {
                        request_id,
                        status: "error".to_string(),
                        message: "Empty portfolio_ids for batch_calculate_portfolio_performance event".to_string(),
                    });
                }
                
                batch_calculate_portfolio_performance(
                    &repository,
                    &timestream_client,
                    &timestream_database,
                    &timestream_table,
                    portfolio_ids,
                    &request_id
                ).await?;
            } else {
                return Ok(Response {
                    request_id,
                    status: "error".to_string(),
                    message: "Missing portfolio_ids for batch_calculate_portfolio_performance event".to_string(),
                });
            }
        },
        "calculate_account_performance" => {
            if let Some(account_id) = &event.account_id {
                calculate_account_performance(
                    &repository,
                    &timestream_client,
                    &timestream_database,
                    &timestream_table,
                    account_id,
                    &request_id
                ).await?;
            } else {
                return Ok(Response {
                    request_id,
                    status: "error".to_string(),
                    message: "Missing account_id for calculate_account_performance event".to_string(),
                });
            }
        },
        "batch_calculate_account_performance" => {
            if let Some(account_ids) = &event.account_ids {
                if account_ids.is_empty() {
                    return Ok(Response {
                        request_id,
                        status: "error".to_string(),
                        message: "Empty account_ids for batch_calculate_account_performance event".to_string(),
                    });
                }
                
                batch_calculate_account_performance(
                    &repository,
                    &timestream_client,
                    &timestream_database,
                    &timestream_table,
                    account_ids,
                    &request_id
                ).await?;
            } else {
                return Ok(Response {
                    request_id,
                    status: "error".to_string(),
                    message: "Missing account_ids for batch_calculate_account_performance event".to_string(),
                });
            }
        },
        "attribution_analysis" => {
            if let (Some(portfolio_id), Some(benchmark_id), Some(start_date), Some(end_date)) = 
                (&event.portfolio_id, &event.benchmark_id, &event.start_date, &event.end_date) {
                perform_attribution_analysis(
                    &repository,
                    &timestream_client,
                    &timestream_database,
                    &timestream_table,
                    portfolio_id,
                    benchmark_id,
                    start_date,
                    end_date,
                    &request_id
                ).await?;
            } else {
                return Ok(Response {
                    request_id,
                    status: "error".to_string(),
                    message: "Missing required fields for attribution_analysis event".to_string(),
                });
            }
        },
        "benchmark_comparison" => {
            if let (Some(portfolio_id), Some(benchmark_id), Some(start_date), Some(end_date)) = 
                (&event.portfolio_id, &event.benchmark_id, &event.start_date, &event.end_date) {
                perform_benchmark_comparison(
                    &repository,
                    &timestream_client,
                    &timestream_database,
                    &timestream_table,
                    portfolio_id,
                    benchmark_id,
                    start_date,
                    end_date,
                    &request_id
                ).await?;
            } else {
                return Ok(Response {
                    request_id,
                    status: "error".to_string(),
                    message: "Missing required fields for benchmark_comparison event".to_string(),
                });
            }
        },
        "periodic_returns" => {
            if let (Some(portfolio_id), Some(start_date), Some(end_date)) = 
                (&event.portfolio_id, &event.start_date, &event.end_date) {
                calculate_periodic_returns(
                    &repository,
                    &timestream_client,
                    &timestream_database,
                    &timestream_table,
                    portfolio_id,
                    start_date,
                    end_date,
                    &event.periods,
                    &request_id
                ).await?;
            } else {
                return Ok(Response {
                    request_id,
                    status: "error".to_string(),
                    message: "Missing required fields for periodic_returns event".to_string(),
                });
            }
        },
        _ => {
            return Ok(Response {
                request_id,
                status: "error".to_string(),
                message: format!("Unsupported event type: {}", event.event_type),
            });
        }
    }
    
    Ok(Response {
        request_id,
        status: "success".to_string(),
        message: "Performance calculation completed successfully".to_string(),
    })
}

async fn calculate_performance_after_transaction(
    repository: &DynamoDbRepository,
    timestream_client: &TimestreamWriteClient,
    timestream_database: &str,
    timestream_table: &str,
    transaction_id: &str,
    account_id: &str,
    security_id: &Option<String>,
    transaction_date_str: &Option<String>,
    request_id: &str
) -> Result<()> {
    info!(request_id = %request_id, transaction_id = %transaction_id, "Calculating performance after transaction");
    
    // Get the transaction
    let transaction = match repository.get_transaction(transaction_id).await? {
        Some(t) => t,
        None => return Err(anyhow!("Transaction not found: {}", transaction_id)),
    };
    
    // Get account
    let account = match repository.get_account(account_id).await? {
        Some(a) => a,
        None => return Err(anyhow!("Account not found: {}", account_id)),
    };
    
    // Get all transactions for the account, ordered by date
    let transactions = repository.list_transactions(Some(account_id), None).await?;
    
    // Calculate TWR
    let twr = calculate_time_weighted_return(&transactions)?;
    
    // Calculate MWR
    let mwr = calculate_money_weighted_return(&transactions)?;
    
    // Create performance metric
    let metric_id = Uuid::new_v4().to_string();
    let now = Utc::now();
    
    let performance_metric = PerformanceMetric {
        id: metric_id.clone(),
        account_id: Some(account_id.to_string()),
        portfolio_id: Some(account.portfolio_id.clone()),
        security_id: security_id.clone(),
        calculation_date: now,
        start_date: transactions.first().map(|t| t.transaction_date).unwrap_or(now),
        end_date: now,
        time_weighted_return: Some(twr),
        money_weighted_return: Some(mwr),
        benchmark_id: None,
        benchmark_return: None,
    };
    
    // Store performance metric in DynamoDB
    repository.put_performance_metric(&performance_metric).await?;
    
    // Store time series data in Timestream
    store_performance_in_timestream(
        timestream_client,
        timestream_database,
        timestream_table,
        &performance_metric,
        request_id
    ).await?;
    
    info!(
        request_id = %request_id, 
        transaction_id = %transaction_id, 
        account_id = %account_id,
        twr = %twr.return_value,
        mwr = %mwr.return_value,
        "Performance calculation completed"
    );
    
    Ok(())
}

async fn calculate_portfolio_performance(
    repository: &DynamoDbRepository,
    timestream_client: &TimestreamWriteClient,
    timestream_database: &str,
    timestream_table: &str,
    portfolio_id: &str,
    request_id: &str
) -> Result<()> {
    info!(request_id = %request_id, portfolio_id = %portfolio_id, "Calculating portfolio performance");
    
    // Get all accounts for the portfolio
    let accounts = repository.list_accounts(Some(portfolio_id)).await?;
    
    // Calculate performance for each account
    let mut portfolio_value = Decimal::ZERO;
    let mut portfolio_twr = TimeWeightedReturn::default();
    let mut portfolio_mwr = MoneyWeightedReturn::default();
    
    for account in &accounts {
        // Calculate account performance
        calculate_account_performance(
            repository,
            timestream_client,
            timestream_database,
            timestream_table,
            &account.id,
            request_id
        ).await?;
        
        // Get the latest performance metric for the account
        let account_metrics = repository.list_performance_metrics(
            None,
            Some(&account.id),
            None,
            None
        ).await?;
        
        if let Some(latest_metric) = account_metrics.first() {
            // Aggregate portfolio performance (weighted by account value)
            if let Some(twr) = &latest_metric.time_weighted_return {
                // Simplified aggregation - in reality, this would be more complex
                portfolio_twr.return_value += twr.return_value;
            }
            
            if let Some(mwr) = &latest_metric.money_weighted_return {
                // Simplified aggregation - in reality, this would be more complex
                portfolio_mwr.return_value += mwr.return_value;
            }
        }
    }
    
    // Average the returns (simplified approach)
    if !accounts.is_empty() {
        let account_count = Decimal::from(accounts.len());
        portfolio_twr.return_value = portfolio_twr.return_value / account_count;
        portfolio_mwr.return_value = portfolio_mwr.return_value / account_count;
    }
    
    // Create portfolio performance metric
    let metric_id = Uuid::new_v4().to_string();
    let now = Utc::now();
    
    let performance_metric = PerformanceMetric {
        id: metric_id.clone(),
        account_id: None,
        portfolio_id: Some(portfolio_id.to_string()),
        security_id: None,
        calculation_date: now,
        start_date: now - Duration::days(30), // Simplified - should be based on actual data
        end_date: now,
        time_weighted_return: Some(portfolio_twr),
        money_weighted_return: Some(portfolio_mwr),
        benchmark_id: None,
        benchmark_return: None,
    };
    
    // Store performance metric in DynamoDB
    repository.put_performance_metric(&performance_metric).await?;
    
    // Store time series data in Timestream
    store_performance_in_timestream(
        timestream_client,
        timestream_database,
        timestream_table,
        &performance_metric,
        request_id
    ).await?;
    
    info!(
        request_id = %request_id, 
        portfolio_id = %portfolio_id,
        twr = %portfolio_twr.return_value,
        mwr = %portfolio_mwr.return_value,
        "Portfolio performance calculation completed"
    );
    
    Ok(())
}

async fn calculate_account_performance(
    repository: &DynamoDbRepository,
    timestream_client: &TimestreamWriteClient,
    timestream_database: &str,
    timestream_table: &str,
    account_id: &str,
    request_id: &str
) -> Result<()> {
    info!(request_id = %request_id, account_id = %account_id, "Calculating account performance");
    
    // Get all transactions for the account
    let transactions = repository.list_transactions(Some(account_id), None).await?;
    
    // Calculate TWR
    let twr = calculate_time_weighted_return(&transactions)?;
    
    // Calculate MWR
    let mwr = calculate_money_weighted_return(&transactions)?;
    
    // Get account
    let account = match repository.get_account(account_id).await? {
        Some(a) => a,
        None => return Err(anyhow!("Account not found: {}", account_id)),
    };
    
    // Create performance metric
    let metric_id = Uuid::new_v4().to_string();
    let now = Utc::now();
    
    let performance_metric = PerformanceMetric {
        id: metric_id.clone(),
        account_id: Some(account_id.to_string()),
        portfolio_id: Some(account.portfolio_id.clone()),
        security_id: None,
        calculation_date: now,
        start_date: transactions.first().map(|t| t.transaction_date).unwrap_or(now),
        end_date: now,
        time_weighted_return: Some(twr),
        money_weighted_return: Some(mwr),
        benchmark_id: None,
        benchmark_return: None,
    };
    
    // Store performance metric in DynamoDB
    repository.put_performance_metric(&performance_metric).await?;
    
    // Store time series data in Timestream
    store_performance_in_timestream(
        timestream_client,
        timestream_database,
        timestream_table,
        &performance_metric,
        request_id
    ).await?;
    
    info!(
        request_id = %request_id, 
        account_id = %account_id,
        twr = %twr.return_value,
        mwr = %mwr.return_value,
        "Account performance calculation completed"
    );
    
    Ok(())
}

fn calculate_time_weighted_return(transactions: &[Transaction]) -> Result<TimeWeightedReturn> {
    // This is a simplified implementation of TWR calculation
    // In a real-world scenario, this would be more complex and consider:
    // - Proper sub-period returns
    // - Geometric linking
    // - Handling of cash flows
    
    // For demonstration purposes, we'll use a simplified approach
    let mut twr = TimeWeightedReturn {
        return_value: Decimal::ZERO,
        calculation_method: "Modified Dietz".to_string(),
        sub_period_returns: Vec::new(),
        annualized: false,
    };
    
    // If there are no transactions, return zero
    if transactions.is_empty() {
        return Ok(twr);
    }
    
    // Sort transactions by date
    let mut sorted_transactions = transactions.to_vec();
    sorted_transactions.sort_by(|a, b| a.transaction_date.cmp(&b.transaction_date));
    
    // Calculate beginning and ending values
    let beginning_value = Decimal::ONE; // Simplified - should be based on actual data
    let mut ending_value = beginning_value;
    
    // Apply transactions to calculate ending value
    for transaction in &sorted_transactions {
        match transaction.transaction_type.as_str() {
            "BUY" => {
                ending_value += transaction.amount;
            },
            "SELL" => {
                ending_value -= transaction.amount;
            },
            "DIVIDEND" => {
                ending_value += transaction.amount;
            },
            _ => {
                // Handle other transaction types
            }
        }
    }
    
    // Calculate simple return
    if beginning_value != Decimal::ZERO {
        twr.return_value = (ending_value - beginning_value) / beginning_value;
    }
    
    Ok(twr)
}

fn calculate_money_weighted_return(transactions: &[Transaction]) -> Result<MoneyWeightedReturn> {
    // This is a simplified implementation of MWR calculation
    // In a real-world scenario, this would involve solving for the IRR (Internal Rate of Return)
    // which requires iterative numerical methods
    
    // For demonstration purposes, we'll use a simplified approach
    let mut mwr = MoneyWeightedReturn {
        return_value: Decimal::ZERO,
        calculation_method: "Internal Rate of Return".to_string(),
        annualized: false,
    };
    
    // If there are no transactions, return zero
    if transactions.is_empty() {
        return Ok(mwr);
    }
    
    // Sort transactions by date
    let mut sorted_transactions = transactions.to_vec();
    sorted_transactions.sort_by(|a, b| a.transaction_date.cmp(&b.transaction_date));
    
    // Calculate beginning and ending values
    let beginning_value = Decimal::ONE; // Simplified - should be based on actual data
    let mut ending_value = beginning_value;
    
    // Apply transactions to calculate ending value
    for transaction in &sorted_transactions {
        match transaction.transaction_type.as_str() {
            "BUY" => {
                ending_value += transaction.amount;
            },
            "SELL" => {
                ending_value -= transaction.amount;
            },
            "DIVIDEND" => {
                ending_value += transaction.amount;
            },
            _ => {
                // Handle other transaction types
            }
        }
    }
    
    // Calculate simple return (simplified approximation of IRR)
    if beginning_value != Decimal::ZERO {
        mwr.return_value = (ending_value - beginning_value) / beginning_value;
    }
    
    Ok(mwr)
}

async fn store_performance_in_timestream(
    timestream_client: &TimestreamWriteClient,
    database: &str,
    table: &str,
    metric: &PerformanceMetric,
    request_id: &str
) -> Result<()> {
    use aws_sdk_timestreamwrite::model::{Dimension, MeasureValue, Record, TimeUnit};
    
    info!(request_id = %request_id, metric_id = %metric.id, "Storing performance data in Timestream");
    
    let now = Utc::now();
    let timestamp = now.timestamp_millis().to_string();
    
    // Create dimensions
    let mut dimensions = Vec::new();
    
    dimensions.push(Dimension::builder()
        .name("metric_id")
        .value(&metric.id)
        .build());
    
    if let Some(account_id) = &metric.account_id {
        dimensions.push(Dimension::builder()
            .name("account_id")
            .value(account_id)
            .build());
    }
    
    if let Some(portfolio_id) = &metric.portfolio_id {
        dimensions.push(Dimension::builder()
            .name("portfolio_id")
            .value(portfolio_id)
            .build());
    }
    
    if let Some(security_id) = &metric.security_id {
        dimensions.push(Dimension::builder()
            .name("security_id")
            .value(security_id)
            .build());
    }
    
    // Create records
    let mut records = Vec::new();
    
    // TWR record
    if let Some(twr) = &metric.time_weighted_return {
        records.push(Record::builder()
            .dimensions(dimensions.clone())
            .measure_name("time_weighted_return")
            .measure_value(MeasureValue::Double(twr.return_value.to_f64().unwrap_or(0.0)))
            .measure_value_type("DOUBLE")
            .time(timestamp.clone())
            .time_unit(TimeUnit::Milliseconds)
            .build());
    }
    
    // MWR record
    if let Some(mwr) = &metric.money_weighted_return {
        records.push(Record::builder()
            .dimensions(dimensions.clone())
            .measure_name("money_weighted_return")
            .measure_value(MeasureValue::Double(mwr.return_value.to_f64().unwrap_or(0.0)))
            .measure_value_type("DOUBLE")
            .time(timestamp)
            .time_unit(TimeUnit::Milliseconds)
            .build());
    }
    
    // Write records to Timestream
    timestream_client.write_records()
        .database_name(database)
        .table_name(table)
        .set_records(Some(records))
        .send()
        .await
        .map_err(|e| anyhow!("Failed to write to Timestream: {}", e))?;
    
    info!(request_id = %request_id, metric_id = %metric.id, "Performance data stored in Timestream");
    
    Ok(())
}

async fn perform_attribution_analysis(
    repository: &DynamoDbRepository,
    timestream_client: &TimestreamWriteClient,
    timestream_database: &str,
    timestream_table: &str,
    portfolio_id: &str,
    benchmark_id: &str,
    start_date_str: &str,
    end_date_str: &str,
    request_id: &str
) -> Result<()> {
    info!(
        request_id = %request_id, 
        portfolio_id = %portfolio_id,
        benchmark_id = %benchmark_id,
        "Performing attribution analysis"
    );
    
    // Parse dates
    let start_date = DateTime::parse_from_rfc3339(start_date_str)
        .map_err(|e| anyhow!("Invalid start_date format: {}", e))?
        .with_timezone(&Utc);
    
    let end_date = DateTime::parse_from_rfc3339(end_date_str)
        .map_err(|e| anyhow!("Invalid end_date format: {}", e))?
        .with_timezone(&Utc);
    
    // Get portfolio data
    let portfolio = match repository.get_portfolio(portfolio_id).await? {
        Some(p) => p,
        None => return Err(anyhow!("Portfolio not found: {}", portfolio_id)),
    };
    
    // Get benchmark data
    let benchmark = match repository.get_benchmark(benchmark_id).await? {
        Some(b) => b,
        None => return Err(anyhow!("Benchmark not found: {}", benchmark_id)),
    };
    
    // Get all accounts for the portfolio
    let accounts = repository.list_accounts(Some(portfolio_id)).await?;
    
    // Get holdings by asset class for the portfolio
    let mut portfolio_weights = HashMap::new();
    let mut portfolio_returns = HashMap::new();
    
    for account in &accounts {
        // Get holdings for the account
        let holdings = repository.list_holdings(&account.id).await?;
        
        for holding in holdings {
            if let Some(security_id) = &holding.security_id {
                // Get security details
                if let Some(security) = repository.get_security(security_id).await? {
                    let asset_class = security.security_type.clone();
                    
                    // Add to portfolio weights
                    *portfolio_weights.entry(asset_class.clone()).or_insert(Decimal::ZERO) += 
                        holding.market_value / portfolio.total_value;
                    
                    // Get security returns
                    let security_returns = repository.list_security_returns(
                        security_id,
                        Some(start_date),
                        Some(end_date)
                    ).await?;
                    
                    if !security_returns.is_empty() {
                        // Calculate average return for the period
                        let total_return: Decimal = security_returns.iter()
                            .map(|r| r.return_value)
                            .sum();
                        
                        let avg_return = total_return / Decimal::from(security_returns.len());
                        
                        // Add to portfolio returns
                        *portfolio_returns.entry(asset_class).or_insert(Decimal::ZERO) += 
                            avg_return * (holding.market_value / portfolio.total_value);
                    }
                }
            }
        }
    }
    
    // Get benchmark weights and returns
    let mut benchmark_weights = HashMap::new();
    let mut benchmark_returns = HashMap::new();
    
    // For demonstration, we'll use placeholder data
    // In a real implementation, you would retrieve this from your data source
    let asset_classes = vec!["EQUITY", "FIXED_INCOME", "CASH", "ALTERNATIVE"];
    
    for asset_class in asset_classes {
        benchmark_weights.insert(asset_class.to_string(), Decimal::from_f64(0.25).unwrap());
        benchmark_returns.insert(asset_class.to_string(), Decimal::from_f64(0.05).unwrap());
    }
    
    // Calculate attribution
    let attribution = calculations::calculate_attribution(
        &portfolio_returns,
        &benchmark_returns,
        &portfolio_weights,
        &benchmark_weights
    )?;
    
    // Store attribution results
    // This would typically be stored in a database
    info!(
        request_id = %request_id,
        portfolio_id = %portfolio_id,
        benchmark_id = %benchmark_id,
        portfolio_return = %attribution.portfolio_return,
        benchmark_return = %attribution.benchmark_return,
        excess_return = %attribution.excess_return,
        "Attribution analysis completed"
    );
    
    Ok(())
}

async fn perform_benchmark_comparison(
    repository: &DynamoDbRepository,
    timestream_client: &TimestreamWriteClient,
    timestream_database: &str,
    timestream_table: &str,
    portfolio_id: &str,
    benchmark_id: &str,
    start_date_str: &str,
    end_date_str: &str,
    request_id: &str
) -> Result<()> {
    info!(
        request_id = %request_id, 
        portfolio_id = %portfolio_id,
        benchmark_id = %benchmark_id,
        "Performing benchmark comparison"
    );
    
    // Parse dates
    let start_date = DateTime::parse_from_rfc3339(start_date_str)
        .map_err(|e| anyhow!("Invalid start_date format: {}", e))?
        .with_timezone(&Utc);
    
    let end_date = DateTime::parse_from_rfc3339(end_date_str)
        .map_err(|e| anyhow!("Invalid end_date format: {}", e))?
        .with_timezone(&Utc);
    
    // Get portfolio data
    let portfolio = match repository.get_portfolio(portfolio_id).await? {
        Some(p) => p,
        None => return Err(anyhow!("Portfolio not found: {}", portfolio_id)),
    };
    
    // Get benchmark data
    let benchmark = match repository.get_benchmark(benchmark_id).await? {
        Some(b) => b,
        None => return Err(anyhow!("Benchmark not found: {}", benchmark_id)),
    };
    
    // Get portfolio returns
    let portfolio_metrics = repository.list_performance_metrics(
        Some(portfolio_id),
        None,
        Some(start_date),
        Some(end_date)
    ).await?;
    
    // Get benchmark returns
    let benchmark_metrics = repository.list_benchmark_returns(
        benchmark_id,
        Some(start_date),
        Some(end_date)
    ).await?;
    
    // Convert to ReturnSeries
    let mut portfolio_dates = Vec::new();
    let mut portfolio_returns = Vec::new();
    
    for metric in &portfolio_metrics {
        if let Some(twr) = &metric.time_weighted_return {
            portfolio_dates.push(metric.calculation_date.date().naive_utc());
            portfolio_returns.push(twr.return_value);
        }
    }
    
    let mut benchmark_dates = Vec::new();
    let mut benchmark_returns = Vec::new();
    
    for br in &benchmark_metrics {
        benchmark_dates.push(br.date.naive_utc());
        benchmark_returns.push(br.return_value);
    }
    
    // Create ReturnSeries
    let portfolio_series = calculations::ReturnSeries {
        dates: portfolio_dates,
        returns: portfolio_returns,
    };
    
    let benchmark_series = calculations::ReturnSeries {
        dates: benchmark_dates,
        returns: benchmark_returns,
    };
    
    // Calculate annualized returns (simplified)
    let days = (end_date - start_date).num_days();
    let years = Decimal::from(days) / Decimal::from(365);
    
    let portfolio_return = portfolio_series.returns.iter()
        .fold(Decimal::ONE, |acc, r| acc * (Decimal::ONE + *r)) - Decimal::ONE;
    
    let benchmark_return = benchmark_series.returns.iter()
        .fold(Decimal::ONE, |acc, r| acc * (Decimal::ONE + *r)) - Decimal::ONE;
    
    let annualized_portfolio_return = if years > Decimal::ZERO {
        ((Decimal::ONE + portfolio_return).powf(Decimal::ONE / years).to_f64().unwrap_or(1.0) - 1.0)
            .into()
    } else {
        portfolio_return
    };
    
    let annualized_benchmark_return = if years > Decimal::ZERO {
        ((Decimal::ONE + benchmark_return).powf(Decimal::ONE / years).to_f64().unwrap_or(1.0) - 1.0)
            .into()
    } else {
        benchmark_return
    };
    
    // Calculate benchmark comparison
    let comparison = calculations::calculate_benchmark_comparison(
        &portfolio_series,
        &benchmark_series,
        annualized_portfolio_return,
        annualized_benchmark_return,
        None
    )?;
    
    // Store benchmark comparison results
    // This would typically be stored in a database
    info!(
        request_id = %request_id,
        portfolio_id = %portfolio_id,
        benchmark_id = %benchmark_id,
        portfolio_return = %comparison.portfolio_return,
        benchmark_return = %comparison.benchmark_return,
        excess_return = %comparison.excess_return,
        beta = ?comparison.beta,
        alpha = ?comparison.alpha,
        "Benchmark comparison completed"
    );
    
    Ok(())
}

async fn calculate_periodic_returns(
    repository: &DynamoDbRepository,
    timestream_client: &TimestreamWriteClient,
    timestream_database: &str,
    timestream_table: &str,
    portfolio_id: &str,
    start_date_str: &str,
    end_date_str: &str,
    periods_opt: &Option<Vec<String>>,
    request_id: &str
) -> Result<()> {
    info!(
        request_id = %request_id, 
        portfolio_id = %portfolio_id,
        "Calculating periodic returns"
    );
    
    // Parse dates
    let start_date = DateTime::parse_from_rfc3339(start_date_str)
        .map_err(|e| anyhow!("Invalid start_date format: {}", e))?
        .with_timezone(&Utc);
    
    let end_date = DateTime::parse_from_rfc3339(end_date_str)
        .map_err(|e| anyhow!("Invalid end_date format: {}", e))?
        .with_timezone(&Utc);
    
    // Get portfolio data
    let portfolio = match repository.get_portfolio(portfolio_id).await? {
        Some(p) => p,
        None => return Err(anyhow!("Portfolio not found: {}", portfolio_id)),
    };
    
    // Get portfolio returns
    let portfolio_metrics = repository.list_performance_metrics(
        Some(portfolio_id),
        None,
        Some(start_date),
        Some(end_date)
    ).await?;
    
    // Convert to ReturnSeries
    let mut dates = Vec::new();
    let mut returns = Vec::new();
    
    for metric in &portfolio_metrics {
        if let Some(twr) = &metric.time_weighted_return {
            dates.push(metric.calculation_date.date().naive_utc());
            returns.push(twr.return_value);
        }
    }
    
    // Create ReturnSeries
    let return_series = calculations::ReturnSeries {
        dates,
        returns,
    };
    
    // Determine which periods to calculate
    let all_periods = vec![
        "monthly".to_string(),
        "quarterly".to_string(),
        "annual".to_string(),
        "ytd".to_string(),
        "since_inception".to_string()
    ];
    
    let periods = periods_opt.as_ref().unwrap_or(&all_periods);
    
    // Calculate periodic returns
    let mut results = HashMap::new();
    
    for period in periods {
        match period.to_lowercase().as_str() {
            "monthly" => {
                let monthly = calculations::calculate_monthly_returns(&return_series)?;
                info!(request_id = %request_id, "Calculated {} monthly returns", monthly.len());
                results.insert("monthly", monthly);
            },
            "quarterly" => {
                let quarterly = calculations::calculate_quarterly_returns(&return_series)?;
                info!(request_id = %request_id, "Calculated {} quarterly returns", quarterly.len());
                results.insert("quarterly", quarterly);
            },
            "annual" => {
                let annual = calculations::calculate_annual_returns(&return_series)?;
                info!(request_id = %request_id, "Calculated {} annual returns", annual.len());
                results.insert("annual", annual);
            },
            "ytd" => {
                if let Some(ytd) = calculations::calculate_ytd_return(&return_series, None)? {
                    info!(request_id = %request_id, "Calculated YTD return: {}", ytd.return_value);
                    results.insert("ytd", vec![ytd]);
                }
            },
            "since_inception" => {
                if let Some(since_inception) = calculations::calculate_since_inception_return(&return_series)? {
                    info!(request_id = %request_id, "Calculated since inception return: {}", since_inception.return_value);
                    results.insert("since_inception", vec![since_inception]);
                }
            },
            _ => {
                info!(request_id = %request_id, "Unknown period type: {}", period);
            }
        }
    }
    
    // Store periodic returns results
    // This would typically be stored in a database
    info!(
        request_id = %request_id,
        portfolio_id = %portfolio_id,
        "Periodic returns calculation completed"
    );
    
    Ok(())
}

async fn batch_calculate_portfolio_performance(
    repository: &DynamoDbRepository,
    timestream_client: &TimestreamWriteClient,
    timestream_database: &str,
    timestream_table: &str,
    portfolio_ids: &[String],
    request_id: &str
) -> Result<()> {
    info!(
        request_id = %request_id,
        portfolio_count = portfolio_ids.len(),
        "Batch calculating portfolio performance"
    );
    
    // Create a process function that calculates performance for a single portfolio
    let process_fn = |portfolio_id: String| {
        let repository = repository.clone();
        let timestream_client = timestream_client.clone();
        let timestream_database = timestream_database.to_string();
        let timestream_table = timestream_table.to_string();
        let request_id = request_id.to_string();
        
        async move {
            calculate_portfolio_performance(
                &repository,
                &timestream_client,
                &timestream_database,
                &timestream_table,
                &portfolio_id,
                &request_id
            ).await
        }
    };
    
    // Process portfolios in parallel
    let results = calculations::process_portfolios(
        portfolio_ids.to_vec(),
        process_fn,
        request_id
    ).await?;
    
    info!(
        request_id = %request_id,
        successful_count = results.len(),
        "Batch portfolio performance calculation completed"
    );
    
    Ok(())
}

async fn batch_calculate_account_performance(
    repository: &DynamoDbRepository,
    timestream_client: &TimestreamWriteClient,
    timestream_database: &str,
    timestream_table: &str,
    account_ids: &[String],
    request_id: &str
) -> Result<()> {
    info!(
        request_id = %request_id,
        account_count = account_ids.len(),
        "Batch calculating account performance"
    );
    
    // Create a process function that calculates performance for a single account
    let process_fn = |account_id: String| {
        let repository = repository.clone();
        let timestream_client = timestream_client.clone();
        let timestream_database = timestream_database.to_string();
        let timestream_table = timestream_table.to_string();
        let request_id = request_id.to_string();
        
        async move {
            calculate_account_performance(
                &repository,
                &timestream_client,
                &timestream_database,
                &timestream_table,
                &account_id,
                &request_id
            ).await
        }
    };
    
    // Process accounts in parallel with a maximum concurrency of 5
    let results = calculations::process_batch(
        account_ids.to_vec(),
        5,
        process_fn,
        request_id
    ).await?;
    
    info!(
        request_id = %request_id,
        successful_count = results.len(),
        "Batch account performance calculation completed"
    );
    
    Ok(())
}

/// Process custom events
async fn process_custom_event(event: Value, event_type: &str, factory: &ComponentFactory, request_id: &str) -> Result<()> {
    // Create required components
    let audit_trail = factory.create_audit_trail().await?
        .context("Failed to create audit trail")?;
    let cache = factory.create_redis_cache().await?
        .context("Failed to create Redis cache")?;
    
    match event_type {
        "calculate_portfolio_performance" => {
            // Extract parameters
            let portfolio_id = event.get("portfolio_id")
                .and_then(|v| v.as_str())
                .context("Missing portfolio_id")?;
            let start_date = event.get("start_date")
                .and_then(|v| v.as_str())
                .context("Missing start_date")?;
            let end_date = event.get("end_date")
                .and_then(|v| v.as_str())
                .context("Missing end_date")?;
            let base_currency = event.get("base_currency")
                .and_then(|v| v.as_str())
                .unwrap_or("USD");
            
            info!(
                request_id = %request_id,
                portfolio_id = %portfolio_id,
                start_date = %start_date,
                end_date = %end_date,
                base_currency = %base_currency,
                "Calculating portfolio performance"
            );
            
            // Create cache key
            let cache_key = format!(
                "portfolio:{}:performance:{}:{}:{}",
                portfolio_id, start_date, end_date, base_currency
            );
            
            // Try to get from cache
            if let Some(cached_result) = cache.get(&cache_key).await? {
                info!(request_id = %request_id, "Using cached performance result");
                
                // Record cache hit in audit trail
                audit_trail.record(audit::AuditRecord {
                    id: uuid::Uuid::new_v4().to_string(),
                    timestamp: chrono::Utc::now(),
                    entity_id: portfolio_id.to_string(),
                    entity_type: "portfolio".to_string(),
                    action: "cache_hit".to_string(),
                    user_id: "system".to_string(),
                    parameters: format!("cache_key={}", cache_key),
                    result: cached_result.clone(),
                }).await?;
                
                return Ok(());
            }
            
            // Create currency converter if needed
            let currency_converter = if base_currency != "USD" {
                Some(factory.create_currency_converter().await?
                    .context("Failed to create currency converter")?)
            } else {
                None
            };
            
            // Fetch portfolio data
            // ... (code to fetch portfolio data)
            
            // Calculate performance
            // ... (code to calculate performance)
            
            // Cache the result
            // ... (code to cache the result)
            
            // Record in audit trail
            // ... (code to record in audit trail)
        },
        "calculate_benchmark_comparison" => {
            // Similar implementation for benchmark comparison
            // ...
        },
        _ => {
            error!(request_id = %request_id, event_type = %event_type, "Unknown custom event type");
            return Err(anyhow::anyhow!("Unknown custom event type: {}", event_type).into());
        }
    }
    
    Ok(())
}

#[derive(Debug, Serialize, Deserialize)]
struct PerformanceCalculationRequest {
    portfolio_id: String,
    start_date: String,
    end_date: String,
    calculation_types: Vec<String>,
    request_id: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct PerformanceResult {
    portfolio_id: String,
    start_date: String,
    end_date: String,
    twr: Option<f64>,
    mwr: Option<f64>,
    volatility: Option<f64>,
    sharpe_ratio: Option<f64>,
    max_drawdown: Option<f64>,
    calculated_at: String,
    request_id: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct AuditRecord {
    id: String,
    entity_id: String,
    entity_type: String,
    action: String,
    user_id: String,
    timestamp: String,
    details: String,
}

struct PerformanceCalculator {
    dynamodb_client: DynamoDbClient,
    timestream_client: Option<TimestreamWriteClient>,
    table_name: String,
    timestream_database: Option<String>,
    timestream_table: Option<String>,
    twr_calculator: TimeWeightedReturnCalculator,
    mwr_calculator: MoneyWeightedReturnCalculator,
    risk_calculator: RiskMetricsCalculator,
}

impl PerformanceCalculator {
    async fn new() -> Result<Self, Error> {
        let config = aws_config::load_from_env().await;
        let dynamodb_client = DynamoDbClient::new(&config);
        let timestream_client = if env::var("TIMESTREAM_DATABASE").is_ok() && env::var("TIMESTREAM_TABLE").is_ok() {
            Some(TimestreamWriteClient::new(&config))
        } else {
            None
        };
        
        let table_name = env::var("DYNAMODB_TABLE").expect("DYNAMODB_TABLE must be set");
        let timestream_database = env::var("TIMESTREAM_DATABASE").ok();
        let timestream_table = env::var("TIMESTREAM_TABLE").ok();
        
        let twr_calculator = TimeWeightedReturnCalculator::new();
        let mwr_calculator = MoneyWeightedReturnCalculator::new();
        let risk_calculator = RiskMetricsCalculator::new();
        
        Ok(Self {
            dynamodb_client,
            timestream_client,
            table_name,
            timestream_database,
            timestream_table,
            twr_calculator,
            mwr_calculator,
            risk_calculator,
        })
    }
    
    async fn process_event(&self, event: SqsEvent) -> Result<(), Error> {
        info!("Processing {} SQS messages", event.records.len());
        
        for record in event.records {
            match self.process_sqs_message(&record).await {
                Ok(_) => info!("Successfully processed message {}", record.message_id.as_deref().unwrap_or("unknown")),
                Err(e) => error!("Error processing message {}: {}", record.message_id.as_deref().unwrap_or("unknown"), e),
            }
        }
        
        Ok(())
    }
    
    async fn process_sqs_message(&self, message: &SqsMessage) -> Result<(), Error> {
        let body = match &message.body {
            Some(body) => body,
            None => {
                warn!("Received SQS message with no body");
                return Ok(());
            }
        };
        
        info!("Processing SQS message: {}", message.message_id.as_deref().unwrap_or("unknown"));
        
        // Parse the calculation request
        let calculation_request: PerformanceCalculationRequest = serde_json::from_str(body)?;
        
        // Process the calculation request
        self.calculate_performance(calculation_request).await?;
        
        Ok(())
    }
    
    async fn calculate_performance(&self, request: PerformanceCalculationRequest) -> Result<(), Error> {
        info!(
            portfolio_id = %request.portfolio_id,
            start_date = %request.start_date,
            end_date = %request.end_date,
            request_id = %request.request_id,
            "Calculating performance metrics"
        );
        
        // Parse dates
        let start_date = DateTime::parse_from_rfc3339(&request.start_date)
            .map_err(|e| {
                error!("Invalid start_date format: {}", e);
                Box::new(e) as Box<dyn std::error::Error + Send + Sync>
            })?
            .with_timezone(&Utc)
            .date_naive();
        
        let end_date = DateTime::parse_from_rfc3339(&request.end_date)
            .map_err(|e| {
                error!("Invalid end_date format: {}", e);
                Box::new(e) as Box<dyn std::error::Error + Send + Sync>
            })?
            .with_timezone(&Utc)
            .date_naive();
        
        // Get portfolio data
        let portfolio = self.get_portfolio(&request.portfolio_id).await?;
        
        // Initialize result
        let mut result = PerformanceResult {
            portfolio_id: request.portfolio_id.clone(),
            start_date: request.start_date.clone(),
            end_date: request.end_date.clone(),
            twr: None,
            mwr: None,
            volatility: None,
            sharpe_ratio: None,
            max_drawdown: None,
            calculated_at: Utc::now().to_rfc3339(),
            request_id: request.request_id.clone(),
        };
        
        // Calculate requested metrics
        for calculation_type in &request.calculation_types {
            match calculation_type.as_str() {
                "TWR" => {
                    info!(portfolio_id = %request.portfolio_id, "Calculating TWR");
                    let twr = self.twr_calculator.calculate_twr(&portfolio, start_date, end_date).await?;
                    result.twr = Some(twr.to_f64().unwrap_or(0.0));
                },
                "MWR" => {
                    info!(portfolio_id = %request.portfolio_id, "Calculating MWR");
                    let mwr = self.mwr_calculator.calculate_mwr(&portfolio, start_date, end_date).await?;
                    result.mwr = Some(mwr.to_f64().unwrap_or(0.0));
                },
                "VOLATILITY" => {
                    info!(portfolio_id = %request.portfolio_id, "Calculating volatility");
                    let volatility = self.risk_calculator.calculate_volatility(&portfolio, start_date, end_date).await?;
                    result.volatility = Some(volatility.to_f64().unwrap_or(0.0));
                },
                "SHARPE_RATIO" => {
                    info!(portfolio_id = %request.portfolio_id, "Calculating Sharpe ratio");
                    let risk_free_rate = dec!(0.02); // 2% risk-free rate
                    let sharpe_ratio = self.risk_calculator.calculate_sharpe_ratio(&portfolio, start_date, end_date, risk_free_rate).await?;
                    result.sharpe_ratio = Some(sharpe_ratio.to_f64().unwrap_or(0.0));
                },
                "MAX_DRAWDOWN" => {
                    info!(portfolio_id = %request.portfolio_id, "Calculating maximum drawdown");
                    let max_drawdown = self.risk_calculator.calculate_max_drawdown(&portfolio, start_date, end_date).await?;
                    result.max_drawdown = Some(max_drawdown.to_f64().unwrap_or(0.0));
                },
                _ => {
                    warn!("Unknown calculation type: {}", calculation_type);
                }
            }
        }
        
        // Store the result
        self.store_performance_result(&result).await?;
        
        // Create audit record
        self.create_audit_record(
            &request.portfolio_id,
            "portfolio",
            "calculate_performance",
            "system",
            &format!(
                "Calculated performance metrics for portfolio {} from {} to {}",
                request.portfolio_id, request.start_date, request.end_date
            ),
        ).await?;
        
        Ok(())
    }
    
    async fn get_portfolio(&self, portfolio_id: &str) -> Result<Portfolio, Error> {
        info!(portfolio_id = %portfolio_id, "Getting portfolio data");
        
        // TODO: Implement actual DynamoDB query to get portfolio data
        // For now, return a mock portfolio
        
        let portfolio = Portfolio::new(portfolio_id, "USD");
        
        // In a real implementation, we would:
        // 1. Get the portfolio details from DynamoDB
        // 2. Get the portfolio's holdings from DynamoDB
        // 3. Get the portfolio's transactions from DynamoDB
        // 4. Construct a Portfolio object with this data
        
        Ok(portfolio)
    }
    
    async fn store_performance_result(&self, result: &PerformanceResult) -> Result<(), Error> {
        info!(
            portfolio_id = %result.portfolio_id,
            request_id = %result.request_id,
            "Storing performance result"
        );
        
        // Store in DynamoDB
        // TODO: Implement actual DynamoDB write
        
        // Store in Timestream if available
        if let (Some(client), Some(database), Some(table)) = (
            &self.timestream_client,
            &self.timestream_database,
            &self.timestream_table,
        ) {
            info!(
                portfolio_id = %result.portfolio_id,
                database = %database,
                table = %table,
                "Storing performance metrics in Timestream"
            );
            
            // TODO: Implement actual Timestream write
        }
        
        Ok(())
    }
    
    async fn create_audit_record(
        &self,
        entity_id: &str,
        entity_type: &str,
        action: &str,
        user_id: &str,
        details: &str,
    ) -> Result<(), Error> {
        let audit_record = AuditRecord {
            id: Uuid::new_v4().to_string(),
            entity_id: entity_id.to_string(),
            entity_type: entity_type.to_string(),
            action: action.to_string(),
            user_id: user_id.to_string(),
            timestamp: Utc::now().to_rfc3339(),
            details: details.to_string(),
        };
        
        // TODO: Implement actual DynamoDB write
        // For now, just log the audit record
        info!(
            audit_id = %audit_record.id,
            entity_id = %audit_record.entity_id,
            entity_type = %audit_record.entity_type,
            action = %audit_record.action,
            "Created audit record"
        );
        
        Ok(())
    }
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .json()
        .init();
    
    info!("Performance Calculator starting up");
    
    // Create performance calculator
    let calculator = PerformanceCalculator::new().await?;
    
    // Start Lambda runtime
    lambda_runtime::run(service_fn(|event: LambdaEvent<SqsEvent>| async {
        let (event, _context) = event.into_parts();
        calculator.process_event(event).await
    })).await?;
    
    Ok(())
} 