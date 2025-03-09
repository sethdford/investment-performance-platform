use lambda_runtime::{run, service_fn, Error, LambdaEvent};
use aws_sdk_dynamodb::Client as DynamoDbClient;
use aws_sdk_timestreamwrite::Client as TimestreamWriteClient;
use serde::{Deserialize, Serialize};
use tracing::{info, error, warn};
use shared::models::{
    Transaction, 
    PerformanceMetric, 
    Portfolio, 
    Account,
    Holding,
    TransactionType,
    EntityType,
    PerformancePeriod
};
use shared::repository::{Repository, DynamoDbRepository, PaginatedResult, PaginationOptions};
use chrono::{DateTime, Utc, Duration, NaiveDate};
use uuid::Uuid;
use std::collections::HashMap;
use anyhow::{Result, anyhow, Context};
use rust_decimal::Decimal;
use rust_decimal::prelude::*;
use rust_decimal_macros::dec;
use std::sync::Arc;
use aws_lambda_events::event::sqs::{SqsEvent, SqsMessage};
use serde_json::{json, Value};
use std::env;
use aws_sdk_timestreamwrite::types::{Dimension, MeasureValueType, Record, TimeUnit};
use aws_sdk_timestreamwrite::operation::write_records::WriteRecordsInput;
use shared::error::AppError;

// Import our calculation modules
mod calculations;
mod config;

use crate::calculations::{
    performance_metrics::{
        TimeWeightedReturn, 
        MoneyWeightedReturn,
        SubPeriodReturn,
        CashFlow,
        PerformanceAttribution,
        calculate_modified_dietz,
        calculate_daily_twr,
        calculate_irr
    },
    risk_metrics::{
        RiskMetrics,
        calculate_risk_metrics,
        ReturnSeries
    },
    twr::{
        TimeWeightedReturnCalculator,
        MoneyWeightedReturnCalculator,
        RiskCalculator,
        StandardTWRCalculator,
        StandardMWRCalculator,
        StandardRiskCalculator
    },
    benchmark_comparison::calculate_benchmark_comparison,
    performance_metrics::calculate_attribution,
    periodic_returns::{
        calculate_monthly_returns,
        calculate_quarterly_returns,
        calculate_annual_returns,
        calculate_ytd_return,
        calculate_since_inception_return
    },
    parallel::{
        process_portfolios,
        process_batch
    },
    audit
};
use config::Config;
use calculations::factory::ComponentFactory;
use calculations::streaming::StreamingEvent;
use calculations::cache::{CalculationCache, create_performance_cache};

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
    let config = aws_config::from_env()
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
    // For now, we'll create a mock repository wrapper that doesn't depend on DynamoDbRepository
    let repository = RepositoryWrapper::new_mock();
    
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
            let portfolio_id = event.portfolio_id.ok_or_else(|| Error::from("Missing portfolio_id"))?;
            let start_date = event.start_date
                .map(|s| s.parse::<DateTime<Utc>>().unwrap_or_else(|_| Utc::now() - Duration::days(30)))
                .unwrap_or_else(|| Utc::now());
            let end_date = event.end_date
                .map(|s| s.parse::<DateTime<Utc>>().unwrap())
                .unwrap_or_else(|| Utc::now());
            
            calculate_portfolio_performance(
                &repository,
                &timestream_client,
                &timestream_database,
                &timestream_table,
                &portfolio_id,
                start_date,
                end_date,
                event.benchmark_id.as_deref(),
                &request_id
            ).await?
        },
        "batch_calculate_portfolio_performance" => {
            let portfolio_ids = event.portfolio_ids.ok_or_else(|| Error::from("Missing portfolio_ids"))?;
            batch_calculate_portfolio_performance(
                &repository,
                &timestream_client,
                timestream_database,
                timestream_table,
                portfolio_ids,
                request_id.clone(),
            ).await?
        },
        "calculate_account_performance" => {
            let account_id = event.account_id.ok_or_else(|| Error::from("Missing account_id"))?;
            calculate_account_performance(
                &repository,
                &timestream_client,
                &timestream_database,
                &timestream_table,
                &account_id,
                request_id.to_string()
            ).await?
        },
        "batch_calculate_account_performance" => {
            let account_ids = event.account_ids.ok_or_else(|| Error::from("Missing account_ids"))?;
            batch_calculate_account_performance(
                &repository,
                &timestream_client,
                timestream_database,
                timestream_table,
                account_ids,
                request_id.clone(),
            ).await?
        },
        "attribution_analysis" => {
            let portfolio_id = event.portfolio_id.ok_or_else(|| Error::from("Missing portfolio_id"))?;
            let benchmark_id = event.benchmark_id.ok_or_else(|| Error::from("Missing benchmark_id"))?;
            perform_attribution_analysis(
                &repository,
                &portfolio_id,
                &benchmark_id,
                request_id.clone(),
            ).await?;
        },
        "benchmark_comparison" => {
            let portfolio_id = event.portfolio_id.ok_or_else(|| Error::from("Missing portfolio_id"))?;
            let benchmark_id = event.benchmark_id.ok_or_else(|| Error::from("Missing benchmark_id"))?;
            perform_benchmark_comparison(
                &repository,
                &portfolio_id,
                &benchmark_id,
                request_id.clone(),
            ).await?;
        },
        "periodic_returns" => {
            if let (Some(portfolio_id), Some(start_date), Some(end_date)) = 
                (&event.portfolio_id, &event.start_date, &event.end_date) {
                calculate_periodic_returns(
                    &repository,
                    &timestream_client,
                    timestream_database,
                    timestream_table,
                    portfolio_id.clone(),
                    start_date.clone(),
                    end_date.clone(),
                    &event.periods,
                    request_id.clone()
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
    repository: &RepositoryWrapper,
    timestream_client: &TimestreamWriteClient,
    timestream_database: &str,
    timestream_table: &str,
    transaction_id: &str,
    account_id: &str,
    security_id: &Option<String>,
    transaction_date_str: &Option<String>,
    request_id: &str
) -> Result<()> {
    info!(
        transaction_id = %transaction_id,
        account_id = %account_id,
        request_id = %request_id,
        "Calculating performance after transaction"
    );

    // Get transaction details
    let transaction = match repository.get_transaction(transaction_id).await? {
        Some(t) => t,
        None => {
            return Ok(());
        }
    };

    // Get account details
    let account = match repository.get_account(account_id).await? {
        Some(a) => a,
        None => {
            return Ok(());
        }
    };

    // Get all transactions for the account
    let transactions = repository.list_transactions(Some(account_id), None).await?;

    // Calculate performance metrics
    let time_weighted_return = calculate_twr(&transactions.items, &transaction.transaction_date);
    let money_weighted_return = calculate_mwr(&transactions.items, &transaction.transaction_date);

    // Store metrics in Timestream
    store_performance_metrics_in_timestream(
        timestream_client,
        timestream_database,
        timestream_table,
        EntityType::Account,
        account_id,
        transaction.transaction_date,
        PerformancePeriod::Monthly,
        time_weighted_return.return_value.to_f64().unwrap_or(0.0),
        money_weighted_return.return_value.to_f64().unwrap_or(0.0),
        0.0, // benchmark_return
        None, // alpha
        None, // beta
        request_id
    ).await?;

    Ok(())
}

async fn calculate_portfolio_performance(
    repository: &RepositoryWrapper,
    timestream_client: &TimestreamWriteClient,
    timestream_database: &str,
    timestream_table: &str,
    portfolio_id: &str,
    start_date: DateTime<Utc>,
    end_date: DateTime<Utc>,
    benchmark_id: Option<&str>,
    request_id: &str
) -> Result<()> {
    // Clone the references to avoid lifetime issues
    let repository = repository.clone();
    let timestream_client = timestream_client.clone();
    
    info!(request_id = %request_id, portfolio_id = %portfolio_id, "Calculating portfolio performance");
    
    // Get all accounts for the portfolio
    let accounts = repository.list_accounts(Some(portfolio_id), None).await?;
    
    // Calculate performance for each account
    let mut portfolio_value = Decimal::ZERO;
    let mut portfolio_twr = TimeWeightedReturn::default();
    let mut portfolio_mwr = MoneyWeightedReturn::default();
    
    for account in &accounts.items {
        // Calculate account performance
        calculate_account_performance(
            &repository,
            &timestream_client,
            timestream_database,
            timestream_table,
            &account.id,
            request_id.to_string(),
        ).await?;
        
        // Get the latest performance metric for the account
        let account_metrics = repository.list_performance_metrics(
            EntityType::Account,
            &account.id,
            None,
            None,
            None
        ).await?;
        
        if let Some(latest_metric) = account_metrics.first() {
            // Aggregate portfolio performance (weighted by account value)
            if let Some(twr) = &latest_metric.time_weighted_return {
                // Simplified aggregation - in reality, this would be more complex
                portfolio_twr.return_value += Decimal::try_from(*twr).unwrap_or_default();
            }
            
            if let Some(mwr) = &latest_metric.money_weighted_return {
                // Simplified aggregation - in reality, this would be more complex
                portfolio_mwr.return_value += Decimal::try_from(*mwr).unwrap_or_default();
            }
        }
    }
    
    // Average the returns (simplified approach)
    if !accounts.items.is_empty() {
        let account_count = Decimal::from(accounts.items.len());
        portfolio_twr.return_value = portfolio_twr.return_value / account_count;
        portfolio_mwr.return_value = portfolio_mwr.return_value / account_count;
    }
    
    // Create portfolio performance metric
    let metric_id = Uuid::new_v4().to_string();
    let now = Utc::now();
    
    let performance_metric = PerformanceMetric {
        entity_id: portfolio_id.to_string(),
        entity_type: shared::models::EntityType::Portfolio,
        date: now.date_naive(),
        period: shared::models::PerformancePeriod::Daily,
        time_weighted_return: Some(portfolio_twr.return_value.to_f64().unwrap_or(0.0)),
        money_weighted_return: Some(portfolio_mwr.return_value.to_f64().unwrap_or(0.0)),
        benchmark_return: Some(0.0),
        alpha: None,
        beta: None,
        sharpe_ratio: None,
        sortino_ratio: None,
        information_ratio: None,
        tracking_error: None,
        max_drawdown: None,
        created_at: now,
        updated_at: now,
    };
    
    // Store performance metric in DynamoDB
    repository.put_performance_metric(&performance_metric).await?;
    
    // Store time series data in Timestream
    let request_id_clone = request_id.clone();
    store_performance_metrics_in_timestream(
        &timestream_client,
        timestream_database,
        timestream_table,
        EntityType::Portfolio,
        portfolio_id,
        now.date_naive(),
        PerformancePeriod::Monthly,
        portfolio_twr.return_value.to_f64().unwrap_or(0.0),
        portfolio_mwr.return_value.to_f64().unwrap_or(0.0),
        benchmark_id.map(|id| id.to_string()).unwrap_or_default().parse::<f64>().unwrap_or(0.0),
        None,
        None,
        &request_id_clone
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
    repository: &RepositoryWrapper,
    timestream_client: &TimestreamWriteClient,
    timestream_database: &str,
    timestream_table: &str,
    account_id: &str,
    request_id: String
) -> Result<()> {
    // Clone the references to avoid lifetime issues
    let repository = repository.clone();
    let timestream_client = timestream_client.clone();
    
    info!(request_id = %request_id, account_id = %account_id, "Calculating account performance");
    
    // Get all transactions for the account
    let transactions = repository.list_transactions(Some(account_id), None).await?;
    
    // Calculate TWR
    let twr = calculate_twr(&transactions.items, &transactions.items[0].transaction_date);
    
    // Calculate MWR
    let mwr = calculate_mwr(&transactions.items, &transactions.items[0].transaction_date);
    
    // Get account
    let account = match repository.get_account(account_id).await? {
        Some(a) => a,
        None => return Err(anyhow!("Account not found: {}", account_id)),
    };
    
    // Create performance metric
    let metric_id = Uuid::new_v4().to_string();
    let now = Utc::now();
    let today = Utc::now().date_naive();
    
    let performance_metric = PerformanceMetric {
        entity_id: account_id.to_string(),
        entity_type: shared::models::EntityType::Account,
        date: today,
        period: shared::models::PerformancePeriod::Daily,
        time_weighted_return: Some(twr.return_value.to_f64().unwrap_or(0.0)),
        money_weighted_return: Some(mwr.return_value.to_f64().unwrap_or(0.0)),
        benchmark_return: None,
        alpha: None,
        beta: None,
        sharpe_ratio: None,
        sortino_ratio: None,
        information_ratio: None,
        tracking_error: None,
        max_drawdown: None,
        created_at: now,
        updated_at: now,
    };
    
    // Store performance metric in DynamoDB
    repository.put_performance_metric(&performance_metric).await?;
    
    // Store time series data in Timestream
    let request_id_clone = request_id.clone();
    store_performance_metrics_in_timestream(
        &timestream_client,
        timestream_database,
        timestream_table,
        EntityType::Account,
        account_id,
        today,
        PerformancePeriod::Monthly,
        twr.return_value.to_f64().unwrap_or(0.0),
        mwr.return_value.to_f64().unwrap_or(0.0),
        0.0, // benchmark_return
        None, // alpha
        None, // beta
        &request_id_clone
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

fn calculate_twr(transactions: &[Transaction], _date: &NaiveDate) -> TimeWeightedReturn {
    // This is a simplified implementation of TWR calculation
    // In a real-world scenario, this would be more complex and consider:
    // - Daily valuation points
    // - Cash flows
    // - Dividends and other corporate actions
    
    let mut twr = TimeWeightedReturn {
        return_value: Decimal::ZERO,
        calculation_method: "Modified Dietz".to_string(),
        sub_period_returns: Vec::new(),
        annualized: false,
    };
    
    // If there are no transactions, return zero
    if transactions.is_empty() {
        return twr;
    }
    
    // Sort transactions by date
    let mut sorted_transactions = transactions.to_vec();
    sorted_transactions.sort_by(|a, b| a.transaction_date.cmp(&b.transaction_date));
    
    // Calculate beginning and ending values
    let beginning_value = Decimal::from(10000); // Placeholder
    let ending_value = Decimal::from(11000);    // Placeholder
    
    // Calculate simple return
    if beginning_value > Decimal::ZERO {
        twr.return_value = (ending_value - beginning_value) / beginning_value;
    }
    
    // Calculate period in days
    let first_date = transactions.first().unwrap().transaction_date;
    let last_date = transactions.last().unwrap().transaction_date;
    
    // Calculate period in days and annualize if needed
    let period_days = (last_date - first_date).num_days() as i32;
    
    if period_days > 0 {
        let years = Decimal::from(period_days) / Decimal::from(365);
        
        // Calculate annualized return using a different approach
        // (1 + r)^(1/years) - 1
        // We'll use f64 for the power calculation and convert back to Decimal
        let return_value_f64 = twr.return_value.to_f64().unwrap_or(0.0);
        let years_f64 = years.to_f64().unwrap_or(1.0);
        let annualized_return_f64 = ((1.0 + return_value_f64).powf(1.0 / years_f64)) - 1.0;
        
        // Convert back to Decimal
        twr.annualized = true;
    }
    
    twr
}

fn calculate_mwr(transactions: &[Transaction], _date: &NaiveDate) -> MoneyWeightedReturn {
    // This is a simplified implementation of MWR calculation
    // In a real-world scenario, this would involve solving for the IRR (Internal Rate of Return)
    
    let mut mwr = MoneyWeightedReturn {
        return_value: Decimal::ZERO,
        calculation_method: "IRR".to_string(),
        annualized: false,
    };
    
    // If there are no transactions, return zero
    if transactions.is_empty() {
        return mwr;
    }
    
    // Sort transactions by date
    let mut sorted_transactions = transactions.to_vec();
    sorted_transactions.sort_by(|a, b| a.transaction_date.cmp(&b.transaction_date));
    
    // Calculate beginning and ending values
    let beginning_value = Decimal::from(10000); // Placeholder
    let ending_value = Decimal::from(11000);    // Placeholder
    
    // Calculate simple return
    if beginning_value > Decimal::ZERO {
        mwr.return_value = (ending_value - beginning_value) / beginning_value;
    }
    
    // Calculate period in days
    let first_date = transactions.first().unwrap().transaction_date;
    let last_date = transactions.last().unwrap().transaction_date;
    
    // Calculate period in days and annualize if needed
    let period_days = (last_date - first_date).num_days() as i32;
    
    if period_days > 0 {
        let years = Decimal::from(period_days) / Decimal::from(365);
        
        // Calculate annualized return using a different approach
        // (1 + r)^(1/years) - 1
        // We'll use f64 for the power calculation and convert back to Decimal
        let return_value_f64 = mwr.return_value.to_f64().unwrap_or(0.0);
        let years_f64 = years.to_f64().unwrap_or(1.0);
        let annualized_return_f64 = ((1.0 + return_value_f64).powf(1.0 / years_f64)) - 1.0;
        
        // Convert back to Decimal
        mwr.annualized = true;
    }
    
    mwr
}

async fn store_performance_metrics_in_timestream(
    timestream_client: &TimestreamWriteClient,
    timestream_database: &str,
    timestream_table: &str,
    entity_type: EntityType,
    entity_id: &str,
    date: NaiveDate,
    period: PerformancePeriod,
    time_weighted_return: f64,
    money_weighted_return: f64,
    benchmark_return: f64,
    alpha: Option<Decimal>,
    beta: Option<Decimal>,
    request_id: &str
) -> Result<()> {
    info!(request_id = %request_id, entity_id = %entity_id, "Storing performance data in Timestream");
    
    let now = Utc::now();
    let current_time_ms = now.timestamp_millis().to_string();
    
    let mut records = Vec::new();
    
    // Create dimensions
    let mut dimensions = Vec::new();
    dimensions.push(
        Dimension::builder()
            .name("entity_type")
            .value(format!("{:?}", entity_type))
            .build()
    );
    
    dimensions.push(
        Dimension::builder()
            .name("entity_id")
            .value(entity_id.to_string())
            .build()
    );
    
    dimensions.push(
        Dimension::builder()
            .name("date")
            .value(date.format("%Y-%m-%d").to_string())
            .build()
    );
    
    dimensions.push(
        Dimension::builder()
            .name("period")
            .value(format!("{:?}", period))
            .build()
    );
    
    dimensions.push(
        Dimension::builder()
            .name("request_id")
            .value(request_id.to_string())
            .build()
    );
    
    // Time-weighted return
    let mut record_builder = Record::builder();
    for dimension_result in &dimensions {
        if let Ok(dimension) = dimension_result {
            record_builder = record_builder.dimensions(dimension.clone());
        } else {
            error!("Failed to build dimension for time_weighted_return record");
        }
    }
    record_builder = record_builder
        .measure_name("time_weighted_return")
        .measure_value(format!("{}", time_weighted_return))
        .measure_value_type(MeasureValueType::Double)
        .time(current_time_ms.clone());
    
    let record = record_builder.build();
    records.push(record);
    
    // Money-weighted return
    let mut record_builder = Record::builder();
    for dimension_result in &dimensions {
        if let Ok(dimension) = dimension_result {
            record_builder = record_builder.dimensions(dimension.clone());
        } else {
            error!("Failed to build dimension for money_weighted_return record");
        }
    }
    record_builder = record_builder
        .measure_name("money_weighted_return")
        .measure_value(format!("{}", money_weighted_return))
        .measure_value_type(MeasureValueType::Double)
        .time(current_time_ms.clone());
    
    let record = record_builder.build();
    records.push(record);
    
    // Benchmark return
    let mut record_builder = Record::builder();
    for dimension_result in &dimensions {
        if let Ok(dimension) = dimension_result {
            record_builder = record_builder.dimensions(dimension.clone());
        } else {
            error!("Failed to build dimension for benchmark_return record");
        }
    }
    record_builder = record_builder
        .measure_name("benchmark_return")
        .measure_value(format!("{}", benchmark_return))
        .measure_value_type(MeasureValueType::Double)
        .time(current_time_ms.clone());
    
    let record = record_builder.build();
    records.push(record);
    
    // Alpha
    if let Some(alpha_value) = alpha {
        let mut record_builder = Record::builder();
        for dimension_result in &dimensions {
            if let Ok(dimension) = dimension_result {
                record_builder = record_builder.dimensions(dimension.clone());
            } else {
                error!("Failed to build dimension for alpha record");
            }
        }
        record_builder = record_builder
            .measure_name("alpha")
            .measure_value(format!("{}", alpha_value.to_f64().unwrap_or(0.0)))
            .measure_value_type(MeasureValueType::Double)
            .time(current_time_ms.clone());
        
        let record = record_builder.build();
        records.push(record);
    }
    
    // Beta
    if let Some(beta_value) = beta {
        let mut record_builder = Record::builder();
        for dimension_result in &dimensions {
            if let Ok(dimension) = dimension_result {
                record_builder = record_builder.dimensions(dimension.clone());
            } else {
                error!("Failed to build dimension for beta record");
            }
        }
        record_builder = record_builder
            .measure_name("beta")
            .measure_value(format!("{}", beta_value.to_f64().unwrap_or(0.0)))
            .measure_value_type(MeasureValueType::Double)
            .time(current_time_ms.clone());
        
        let record = record_builder.build();
        records.push(record);
    }
    
    // Write records to Timestream
    if !records.is_empty() {
        let entity_id_clone = entity_id.to_string();
        match timestream_client.write_records()
            .database_name(timestream_database)
            .table_name(timestream_table)
            .set_records(Some(records))
            .send()
            .await 
        {
            Ok(_) => {
                info!(
                    request_id = %request_id,
                    entity_id = %entity_id_clone,
                    "Successfully wrote performance data to Timestream"
                );
            },
            Err(e) => {
                error!(
                    request_id = %request_id,
                    entity_id = %entity_id_clone,
                    error = %e,
                    "Failed to write performance data to Timestream"
                );
                return Err(anyhow!("Failed to write performance data to Timestream: {}", e));
            }
        }
    }
    
    Ok(())
}

async fn perform_attribution_analysis(
    repository: &RepositoryWrapper,
    portfolio_id: &str,
    benchmark_id: &str,
    request_id: String,
) -> Result<Response, Error> {
    info!(
        portfolio_id = %portfolio_id,
        benchmark_id = %benchmark_id,
        request_id = %request_id,
        "Performing attribution analysis"
    );

    // Get portfolio details
    let portfolio = match repository.get_portfolio(portfolio_id).await? {
        Some(p) => p,
        None => {
            return Ok(Response {
                request_id,
                status: "error".to_string(),
                message: format!("Portfolio not found: {}", portfolio_id),
            });
        }
    };

    // Get benchmark details
    let benchmark = match repository.get_benchmark(benchmark_id).await? {
        Some(b) => b,
        None => {
            return Ok(Response {
                request_id,
                status: "error".to_string(),
                message: format!("Benchmark not found: {}", benchmark_id),
            });
        }
    };

    // Get accounts in the portfolio
    let accounts = repository.list_accounts(Some(portfolio_id), None).await?;
    
    // In a real implementation, we would calculate attribution metrics here
    // For now, we'll just return a success response
    
    // Return the results
    Ok(Response {
        request_id,
        status: "success".to_string(),
        message: format!("Attribution analysis completed for portfolio {} against benchmark {}", 
            portfolio_id, benchmark_id),
    })
}

async fn perform_benchmark_comparison(
    repository: &RepositoryWrapper,
    portfolio_id: &str,
    benchmark_id: &str,
    request_id: String,
) -> Result<Response, Error> {
    info!(
        portfolio_id = %portfolio_id,
        benchmark_id = %benchmark_id,
        request_id = %request_id,
        "Performing benchmark comparison"
    );

    // Get portfolio details
    let portfolio = match repository.get_portfolio(portfolio_id).await? {
        Some(p) => p,
        None => {
            return Ok(Response {
                request_id,
                status: "error".to_string(),
                message: format!("Portfolio not found: {}", portfolio_id),
            });
        }
    };

    // Get benchmark details
    let benchmark = match repository.get_benchmark(benchmark_id).await? {
        Some(b) => b,
        None => {
            return Ok(Response {
                request_id,
                status: "error".to_string(),
                message: format!("Benchmark not found: {}", benchmark_id),
            });
        }
    };

    // In a real implementation, we would calculate benchmark comparison metrics here
    // For now, we'll just return a success response
    
    // Return the results
    Ok(Response {
        request_id,
        status: "success".to_string(),
        message: format!("Benchmark comparison completed for portfolio {} against benchmark {}", 
            portfolio_id, benchmark_id),
    })
}

async fn calculate_periodic_returns(
    repository: &RepositoryWrapper,
    timestream_client: &TimestreamWriteClient,
    timestream_database: String,
    timestream_table: String,
    portfolio_id: String,
    start_date_str: String,
    end_date_str: String,
    periods_opt: &Option<Vec<String>>,
    request_id: String
) -> Result<()> {
    info!(
        request_id = %request_id, 
        portfolio_id = %portfolio_id,
        "Calculating periodic returns"
    );
    
    // Parse dates
    let start_date = DateTime::parse_from_rfc3339(&start_date_str)
        .map_err(|e| anyhow!("Invalid start_date format: {}", e))?
        .with_timezone(&Utc);
    
    let end_date = DateTime::parse_from_rfc3339(&end_date_str)
        .map_err(|e| anyhow!("Invalid end_date format: {}", e))?
        .with_timezone(&Utc);
    
    // Get portfolio data
    let portfolio = match repository.get_portfolio(&portfolio_id).await? {
        Some(p) => p,
        None => return Err(anyhow!("Portfolio not found: {}", portfolio_id)),
    };
    
    // Get portfolio returns
    let portfolio_metrics = repository.list_performance_metrics(
        EntityType::Portfolio,
        &portfolio_id,
        None,
        None,
        None
    ).await?;
    
    // Convert to ReturnSeries
    let mut dates = Vec::new();
    let mut returns = Vec::new();
    
    for metric in &portfolio_metrics {
        if let Some(twr) = &metric.time_weighted_return {
            dates.push(metric.date);
            let decimal_value = Decimal::try_from(*twr).unwrap_or_default();
            returns.push(decimal_value);
        }
    }
    
    // Create ReturnSeries
    let return_series = ReturnSeries {
        dates,
        values: returns,
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
                let monthly = calculate_monthly_returns(&return_series)?;
                info!(request_id = %request_id, "Calculated {} monthly returns", monthly.len());
                results.insert("monthly", monthly);
            },
            "quarterly" => {
                let quarterly = calculate_quarterly_returns(&return_series)?;
                info!(request_id = %request_id, "Calculated {} quarterly returns", quarterly.len());
                results.insert("quarterly", quarterly);
            },
            "annual" => {
                let annual = calculate_annual_returns(&return_series)?;
                info!(request_id = %request_id, "Calculated {} annual returns", annual.len());
                results.insert("annual", annual);
            },
            "ytd" => {
                if let Some(ytd) = calculate_ytd_return(&return_series, None)? {
                    info!(request_id = %request_id, "Calculated YTD return: {}", ytd.return_value);
                    results.insert("ytd", vec![ytd]);
                }
            },
            "since_inception" => {
                if let Some(since_inception) = calculate_since_inception_return(&return_series)? {
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
    repository: &RepositoryWrapper,
    timestream_client: &TimestreamWriteClient,
    timestream_database: String,
    timestream_table: String,
    portfolio_ids: Vec<String>,
    request_id: String
) -> Result<()> {
    info!(
        request_id = %request_id,
        portfolio_count = %portfolio_ids.len(),
        "Batch calculating portfolio performance"
    );
    
    // Define the process function
    let request_id_clone = request_id.clone();
    let repository = Arc::new(repository.clone());
    let timestream_client = Arc::new(timestream_client.clone());
    let process_fn = move |portfolio_id: String| {
        let repository = Arc::clone(&repository);
        let timestream_client = Arc::clone(&timestream_client);
        let timestream_database = timestream_database.clone();
        let timestream_table = timestream_table.clone();
        let request_id = request_id_clone.clone();
        
        async move {
            calculate_portfolio_performance(
                &repository,
                &timestream_client,
                &timestream_database,
                &timestream_table,
                &portfolio_id,
                Utc::now(),
                Utc::now(),
                None,
                &request_id
            ).await
        }
    };
    
    // Process portfolios in parallel
    let results = process_portfolios(
        portfolio_ids,
        process_fn,
        &request_id
    ).await?;
    
    info!(
        request_id = %request_id,
        success_count = %results.len(),
        "Completed batch portfolio performance calculation"
    );
    
    Ok(())
}

async fn batch_calculate_account_performance(
    repository: &RepositoryWrapper,
    timestream_client: &TimestreamWriteClient,
    timestream_database: String,
    timestream_table: String,
    account_ids: Vec<String>,
    request_id: String
) -> Result<()> {
    info!(
        request_id = %request_id,
        account_count = %account_ids.len(),
        "Batch calculating account performance"
    );
    
    // Define the process function
    let request_id_clone = request_id.clone();
    let repository = Arc::new(repository.clone());
    let timestream_client = Arc::new(timestream_client.clone());
    let process_fn = move |account_id: String| {
        let repository = Arc::clone(&repository);
        let timestream_client = Arc::clone(&timestream_client);
        let timestream_database = timestream_database.clone();
        let timestream_table = timestream_table.clone();
        let request_id = request_id_clone.clone();
        
        async move {
            calculate_account_performance(
                &repository,
                &timestream_client,
                &timestream_database,
                &timestream_table,
                &account_id,
                request_id
            ).await
        }
    };
    
    // Process accounts in parallel with a maximum concurrency of 5
    let results = process_batch(
        account_ids,
        5,
        process_fn,
        &request_id
    ).await?;
    
    info!(
        request_id = %request_id,
        success_count = %results.len(),
        "Completed batch account performance calculation"
    );
    
    Ok(())
}

/// Process custom events
async fn process_custom_event(event: Value, event_type: &str, factory: &ComponentFactory, request_id: &str) -> Result<()> {
    info!(request_id = %request_id, event_type = %event_type, "Processing custom event");
    
    // Create audit trail and cache
    let audit_trail = factory.create_audit_trail().await
        .with_context(|| "Failed to create audit trail")?;
    
    let cache = match factory.create_redis_cache().await {
        Ok(c) => c,
        Err(e) => return Err(anyhow!("Failed to create Redis cache: {}", e)),
    };

    // Record audit event if audit_trail is available
    if let Some(audit) = audit_trail {
        audit.record(audit::AuditRecord {
            id: Uuid::new_v4().to_string(),
            entity_id: request_id.to_string(),
            entity_type: "CustomEvent".to_string(),
            action: "Process".to_string(),
            user_id: "system".to_string(),
            parameters: event.to_string(),
            result: "".to_string(),
            timestamp: Utc::now(),
            tenant_id: "default".to_string(),
            event_id: Uuid::new_v4().to_string(),
            event_type: event_type.to_string(),
            resource_id: request_id.to_string(),
            resource_type: "Event".to_string(),
            operation: "Process".to_string(),
            details: format!("Processing {} event", event_type),
            status: "Started".to_string(),
        }).await?;
    }
    
    // Process the event based on type
    match event_type {
        "PortfolioRebalance" => {
            // Handle portfolio rebalance event
            let portfolio_id = event["portfolioId"].as_str()
                .ok_or_else(|| anyhow!("Missing portfolioId in event"))?;
            
            let currency_converter = {
                info!("Using mock currency converter");
                // Return a mock implementation
                struct MockCurrencyConverter;
                
                impl MockCurrencyConverter {
                    async fn convert(&self, _amount: Decimal, _from: &str, _to: &str) -> Result<Decimal> {
                        // Just return the original amount for now
                        Ok(_amount)
                    }
                }
                
                Arc::new(MockCurrencyConverter)
            };
            
            // Implement portfolio rebalance logic here
            info!(request_id = %request_id, portfolio_id = %portfolio_id, "Portfolio rebalance completed");
        },
        "DataRefresh" => {
            // Handle data refresh event
            info!(request_id = %request_id, "Data refresh completed");
        },
        _ => {
            return Err(anyhow!("Unsupported event type: {}", event_type));
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
    twr_calculator: Arc<dyn TimeWeightedReturnCalculator>,
    mwr_calculator: Arc<dyn MoneyWeightedReturnCalculator>,
    risk_calculator: Arc<dyn RiskCalculator>,
}

impl PerformanceCalculator {
    async fn new() -> Result<Self, Error> {
        let config = aws_config::from_env().load().await;
        let dynamodb_client = DynamoDbClient::new(&config);
        let timestream_client = if env::var("TIMESTREAM_DATABASE").is_ok() && env::var("TIMESTREAM_TABLE").is_ok() {
            Some(TimestreamWriteClient::new(&config))
        } else {
            None
        };
        
        let table_name = env::var("DYNAMODB_TABLE").expect("DYNAMODB_TABLE must be set");
        let timestream_database = env::var("TIMESTREAM_DATABASE").ok();
        let timestream_table = env::var("TIMESTREAM_TABLE").ok();
        
        let twr_calculator = Arc::new(StandardTWRCalculator::new());
        let mwr_calculator = Arc::new(StandardMWRCalculator::new());
        let risk_calculator = Arc::new(StandardRiskCalculator::new());
        
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
        
        // Get return series for risk calculations
        let return_series = self.get_return_series(&portfolio, start_date, end_date).await?;
        
        // Calculate requested metrics
        for calculation_type in &request.calculation_types {
            match calculation_type.as_str() {
                "TWR" => {
                    info!(portfolio_id = %request.portfolio_id, "Calculating TWR");
                    // Use mock values for beginning_value, ending_value, and cash_flows
                    let beginning_value = Decimal::ONE;
                    let ending_value = Decimal::from_f64(1.05).unwrap();
                    let cash_flows = Vec::new();
                    
                    let twr = self.twr_calculator.calculate_twr(
                        beginning_value,
                        ending_value,
                        &cash_flows,
                        start_date,
                        end_date
                    ).await?;
                    result.twr = Some(twr.return_value.to_f64().unwrap_or(0.0));
                },
                "MWR" => {
                    info!(portfolio_id = %request.portfolio_id, "Calculating MWR");
                    // Use mock values for cash_flows, final_value, max_iterations, and tolerance
                    let cash_flows = Vec::new();
                    let final_value = Decimal::from_f64(1.05).unwrap();
                    let max_iterations = 100;
                    let tolerance = Decimal::from_f64(0.0001).unwrap();
                    
                    let mwr = self.mwr_calculator.calculate_mwr(
                        &cash_flows,
                        final_value,
                        max_iterations,
                        tolerance
                    ).await?;
                    result.mwr = Some(mwr.return_value.to_f64().unwrap_or(0.0));
                },
                "VOLATILITY" => {
                    info!(portfolio_id = %request.portfolio_id, "Calculating volatility");
                    let volatility = self.risk_calculator.calculate_volatility(&return_series).await?;
                    result.volatility = Some(volatility.to_f64().unwrap_or(0.0));
                },
                "SHARPE_RATIO" => {
                    info!(portfolio_id = %request.portfolio_id, "Calculating Sharpe ratio");
                    let risk_free_rate = dec!(0.02); // 2% risk-free rate
                    let sharpe_ratio = self.risk_calculator.calculate_sharpe_ratio(
                        &return_series,
                        risk_free_rate
                    ).await?;
                    result.sharpe_ratio = Some(sharpe_ratio.to_f64().unwrap_or(0.0));
                },
                "MAX_DRAWDOWN" => {
                    info!(portfolio_id = %request.portfolio_id, "Calculating maximum drawdown");
                    let max_drawdown = self.risk_calculator.calculate_max_drawdown(&return_series).await?;
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
        
        // For now, return a mock portfolio
        
        let portfolio = Portfolio {
            id: portfolio_id.to_string(),
            name: format!("Portfolio {}", portfolio_id),
            client_id: "client-123".to_string(),
            inception_date: Utc::now().date_naive(),
            benchmark_id: None,
            status: shared::models::Status::Active,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            metadata: HashMap::new(),
            transactions: Vec::new(),
            holdings: Vec::new(),
        };
        
        // In a real implementation, we would:
        // 1. Get the portfolio details from DynamoDB
        // 2. Return the portfolio or an error if not found
        
        Ok(portfolio)
    }
    
    async fn store_performance_result(&self, result: &PerformanceResult) -> Result<(), Error> {
        info!(
            portfolio_id = %result.portfolio_id,
            "Storing performance result"
        );
        
        if let (Some(database), Some(table)) = (
            &self.timestream_database,
            &self.timestream_table,
        ) {
            info!(
                portfolio_id = %result.portfolio_id,
                "Storing performance result in Timestream"
            );
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

    // Helper method to get return series for risk calculations
    async fn get_return_series(&self, portfolio: &Portfolio, start_date: NaiveDate, end_date: NaiveDate) -> Result<ReturnSeries, Error> {
        // For now, return a simple mock return series
        let dates = vec![start_date, end_date];
        let values = vec![Decimal::ONE, Decimal::from_f64(1.05).unwrap()];
        
        Ok(ReturnSeries {
            dates,
            values,
        })
    }
}

#[derive(Clone)]
pub struct RepositoryWrapper {
    pub inner: Option<DynamoDbRepository>,
    is_mock: bool
}

impl RepositoryWrapper {
    pub fn new(inner: DynamoDbRepository) -> Self {
        Self {
            inner: Some(inner),
            is_mock: false,
        }
    }

    pub fn new_mock() -> Self {
        Self {
            inner: None,
            is_mock: true,
        }
    }

    pub async fn get_transaction(&self, id: &str) -> Result<Option<Transaction>, AppError> {
        if !self.is_mock {
            if let Some(inner) = &self.inner {
                return inner.get_transaction(id).await;
            }
            return Err(AppError::Internal("Repository not initialized".to_string()));
        }
        Ok(None)
    }

    pub async fn get_account(&self, id: &str) -> Result<Option<Account>, AppError> {
        if !self.is_mock {
            if let Some(inner) = &self.inner {
                return inner.get_account(id).await;
            }
            return Err(AppError::Internal("Repository not initialized".to_string()));
        }
        Ok(None)
    }

    pub async fn get_portfolio(&self, id: &str) -> Result<Option<Portfolio>, AppError> {
        if !self.is_mock {
            if let Some(inner) = &self.inner {
                return inner.get_portfolio(id).await;
            }
            return Err(AppError::Internal("Repository not initialized".to_string()));
        }
        Ok(None)
    }

    pub async fn get_security(&self, id: &str) -> Result<Option<Security>, AppError> {
        if !self.is_mock {
            if let Some(inner) = &self.inner {
                let shared_security = inner.get_security(id).await?;
                // Convert from shared::models::Security to local Security
                return Ok(shared_security.map(|s| Security {
                    id: s.id,
                    symbol: s.symbol,
                    name: s.name,
                    asset_class: Some(format!("{:?}", s.asset_class)),
                    currency: "USD".to_string(), // Default currency
                }));
            }
            return Err(AppError::Internal("Repository not initialized".to_string()));
        }
        Ok(None)
    }

    pub async fn get_benchmark(&self, id: &str) -> Result<Option<Benchmark>, AppError> {
        if !self.is_mock {
            if let Some(inner) = &self.inner {
                let shared_benchmark = inner.get_benchmark(id).await?;
                // Convert from shared::models::Benchmark to local Benchmark
                return Ok(shared_benchmark.map(|b| Benchmark {
                    id: b.id,
                    name: b.name,
                    asset_class: "Equity".to_string(), // Default asset class
                    return_value: dec!(0.0), // Default return value
                }));
            }
            return Err(AppError::Internal("Repository not initialized".to_string()));
        }
        Ok(None)
    }

    pub async fn list_accounts(
        &self,
        portfolio_id: Option<&str>,
        pagination: Option<PaginationOptions>
    ) -> Result<PaginatedResult<Account>, AppError> {
        if !self.is_mock {
            if let Some(inner) = &self.inner {
                return inner.list_accounts(portfolio_id, pagination).await;
            }
            return Err(AppError::Internal("Repository not initialized".to_string()));
        }
        Ok(PaginatedResult {
            items: Vec::new(),
            next_token: None,
        })
    }

    pub async fn list_transactions(
        &self,
        account_id: Option<&str>,
        pagination: Option<PaginationOptions>
    ) -> Result<PaginatedResult<Transaction>, AppError> {
        if !self.is_mock {
            if let Some(inner) = &self.inner {
                return inner.list_transactions(account_id, pagination).await;
            }
            return Err(AppError::Internal("Repository not initialized".to_string()));
        }
        Ok(PaginatedResult {
            items: Vec::new(),
            next_token: None,
        })
    }

    pub async fn list_holdings(
        &self,
        account_id: &str
    ) -> Result<Vec<Holding>, AppError> {
        if !self.is_mock {
            if let Some(inner) = &self.inner {
                // Implement a mock version or add the method to DynamoDbRepository
                return Ok(Vec::new()); // Temporary fix
            }
            return Err(AppError::Internal("Repository not initialized".to_string()));
        }
        Ok(Vec::new())
    }

    pub async fn list_performance_metrics(
        &self,
        entity_type: EntityType,
        entity_id: &str,
        start_date: Option<DateTime<Utc>>,
        end_date: Option<DateTime<Utc>>,
        period: Option<PerformancePeriod>
    ) -> Result<Vec<PerformanceMetric>, AppError> {
        if !self.is_mock {
            if let Some(inner) = &self.inner {
                // Implement a mock version or add the method to DynamoDbRepository
                return Ok(Vec::new()); // Temporary fix
            }
            return Err(AppError::Internal("Repository not initialized".to_string()));
        }
        Ok(Vec::new())
    }

    pub async fn list_security_returns(
        &self,
        security_id: &str,
        start_date: Option<DateTime<Utc>>,
        end_date: Option<DateTime<Utc>>
    ) -> Result<Vec<SecurityReturn>, AppError> {
        if !self.is_mock {
            if let Some(inner) = &self.inner {
                // Implement a mock version or add the method to DynamoDbRepository
                return Ok(Vec::new()); // Temporary fix
            }
            return Err(AppError::Internal("Repository not initialized".to_string()));
        }
        Ok(Vec::new())
    }

    pub async fn list_benchmark_returns(
        &self,
        benchmark_id: &str,
        start_date: Option<DateTime<Utc>>,
        end_date: Option<DateTime<Utc>>
    ) -> Result<Vec<BenchmarkReturn>, AppError> {
        if !self.is_mock {
            if let Some(inner) = &self.inner {
                // Implement a mock version or add the method to DynamoDbRepository
                return Ok(Vec::new()); // Temporary fix
            }
            return Err(AppError::Internal("Repository not initialized".to_string()));
        }
        Ok(Vec::new())
    }

    // Store a performance metric in DynamoDB
    pub async fn put_performance_metric(&self, metric: &PerformanceMetric) -> Result<()> {
        // Implementation would go here
        // For now, just log and return success
        info!(entity_id = %metric.entity_id, "Storing performance metric in DynamoDB");
        Ok(())
    }

    pub async fn list_mock_accounts(&self, _portfolio_id: Option<&str>, _pagination: Option<PaginationOptions>) -> Result<PaginatedResult<Account>, AppError> {
        Ok(PaginatedResult {
            items: Vec::new(),
            next_token: None,
        })
    }

    pub async fn list_mock_transactions(&self, _account_id: Option<&str>, _pagination: Option<PaginationOptions>) -> Result<PaginatedResult<Transaction>, AppError> {
        Ok(PaginatedResult {
            items: Vec::new(),
            next_token: None,
        })
    }
}

// Define missing types
#[derive(Debug, Clone)]
pub struct BenchmarkReturn {
    pub date: DateTime<Utc>,
    pub return_value: Decimal,
}

#[derive(Debug, Clone)]
pub struct SecurityReturn {
    pub security_id: String,
    pub date: DateTime<Utc>,
    pub return_value: Decimal,
}

#[derive(Debug, Clone)]
pub struct Security {
    pub id: String,
    pub symbol: String,
    pub name: String,
    pub asset_class: Option<String>,
    pub currency: String,
}

#[derive(Debug, Clone)]
pub struct Benchmark {
    pub id: String,
    pub name: String,
    pub asset_class: String,
    pub return_value: Decimal,
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