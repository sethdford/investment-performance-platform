use aws_lambda_events::event::sqs::SqsEvent;
use aws_sdk_dynamodb::{Client as DynamoDbClient};
use chrono::{DateTime, Utc};
use lambda_runtime::{run, service_fn, Error, LambdaEvent};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::env;
use timestream_repository::{PerformanceDataPoint, TimestreamRepository};
use tracing::{info, error, warn};
use uuid::Uuid;
use std::sync::{Arc, Mutex};
use std::any::Any;
use std::time::Duration;
use lru::LruCache;
use std::collections::HashMap;
use thiserror::Error;

// Import the DynamoDB repository
use dynamodb_repository::DynamoDbRepository;

// Domain models for the DynamoDB repository
mod dynamodb_repository {
    use super::*;
    
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct Portfolio {
        pub id: String,
        pub name: String,
        pub description: Option<String>,
        pub tenant_id: String,
        pub user_id: String,
        pub created_at: DateTime<Utc>,
        pub updated_at: DateTime<Utc>,
        pub currency: String,
        pub status: String,
    }
    
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct Account {
        pub id: String,
        pub portfolio_id: String,
        pub name: String,
        pub description: Option<String>,
        pub account_type: String,
        pub currency: String,
        pub created_at: DateTime<Utc>,
        pub updated_at: DateTime<Utc>,
        pub status: String,
    }
    
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct Transaction {
        pub id: String,
        pub account_id: String,
        pub portfolio_id: String,
        pub transaction_type: String,
        pub transaction_date: DateTime<Utc>,
        pub settlement_date: Option<DateTime<Utc>>,
        pub amount: f64,
        pub currency: String,
        pub security_id: Option<String>,
        pub quantity: Option<f64>,
        pub price: Option<f64>,
        pub fees: Option<f64>,
        pub taxes: Option<f64>,
        pub notes: Option<String>,
        pub created_at: DateTime<Utc>,
        pub updated_at: DateTime<Utc>,
    }
    
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct Valuation {
        pub id: String,
        pub portfolio_id: String,
        pub date: DateTime<Utc>,
        pub value: f64,
        pub cash_balance: f64,
        pub currency: String,
        pub created_at: DateTime<Utc>,
    }
    
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct Benchmark {
        pub id: String,
        pub name: String,
        pub description: Option<String>,
        pub currency: String,
        pub provider: String,
        pub ticker: Option<String>,
        pub created_at: DateTime<Utc>,
        pub updated_at: DateTime<Utc>,
    }
    
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct BenchmarkReturn {
        pub id: String,
        pub benchmark_id: String,
        pub date: DateTime<Utc>,
        pub return_value: f64,
        pub created_at: DateTime<Utc>,
    }
}

// Repository error type
#[derive(Debug, Error)]
pub enum RepositoryError {
    #[error("DynamoDB error: {0}")]
    DynamoDb(#[from] aws_sdk_dynamodb::Error),
    
    #[error("Timestream Write error: {0}")]
    TimestreamWrite(#[from] aws_sdk_timestreamwrite::Error),
    
    #[error("Timestream Query error: {0}")]
    TimestreamQuery(#[from] aws_sdk_timestreamquery::Error),
    
    #[error("Serialization error: {0}")]
    Serialization(String),
    
    #[error("Deserialization error: {0}")]
    Deserialization(String),
    
    #[error("Item not found: {0}")]
    NotFound(String),
    
    #[error("Invalid data: {0}")]
    InvalidData(String),
    
    #[error("Validation error: {0}")]
    Validation(String),
    
    #[error("Concurrency error: {0}")]
    Concurrency(String),
    
    #[error("Rate limit exceeded: {0}")]
    RateLimit(String),
    
    #[error("Circuit breaker open: {0}")]
    CircuitBreakerOpen(String),
    
    #[error("Timeout: {0}")]
    Timeout(String),
    
    #[error("Other error: {0}")]
    Other(String),
}

// Calculation error type
#[derive(Debug, Error)]
pub enum CalculationError {
    #[error("Insufficient data: {0}")]
    InsufficientData(String),
    
    #[error("Invalid input: {0}")]
    InvalidInput(String),
    
    #[error("Calculation failed to converge: {0}")]
    ConvergenceFailure(String),
    
    #[error("Repository error: {0}")]
    Repository(#[from] RepositoryError),
    
    #[error("Unexpected error: {0}")]
    Other(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct PerformanceCalculationRequest {
    portfolio_id: String,
    start_date: DateTime<Utc>,
    end_date: DateTime<Utc>,
    benchmark_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct PerformanceCalculationResult {
    portfolio_id: String,
    start_date: DateTime<Utc>,
    end_date: DateTime<Utc>,
    twr: f64,
    mwr: f64,
    volatility: Option<f64>,
    sharpe_ratio: Option<f64>,
    max_drawdown: Option<f64>,
    benchmark_id: Option<String>,
    benchmark_return: Option<f64>,
    tracking_error: Option<f64>,
    information_ratio: Option<f64>,
    calculated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct AuditRecord {
    id: String,
    event_type: String,
    entity_id: String,
    entity_type: String,
    user_id: Option<String>,
    timestamp: DateTime<Utc>,
    details: Value,
}

pub struct PaginationOptions {
    pub limit: Option<i32>,
    pub start_key: Option<HashMap<String, AttributeValue>>,
}

pub struct PaginatedResult<T> {
    pub items: Vec<T>,
    pub last_evaluated_key: Option<HashMap<String, AttributeValue>>,
}

pub struct CachedDynamoDbRepository<T: Repository> {
    repository: T,
    cache: Arc<Mutex<LruCache<String, Box<dyn Any + Send + Sync>>>>,
    ttl: Duration,
}

impl<T: Repository> CachedDynamoDbRepository<T> {
    pub fn new(repository: T, capacity: usize, ttl: Duration) -> Self {
        Self {
            repository,
            cache: Arc::new(Mutex::new(LruCache::new(capacity))),
            ttl,
        }
    }
    
    // Implement caching methods
}

async fn function_handler(event: LambdaEvent<SqsEvent>) -> Result<(), Error> {
    // Initialize AWS clients
    let config = aws_config::load_from_env().await;
    let dynamodb_client = DynamoDbClient::new(&config);
    
    // Initialize repositories
    let table_name = env::var("DYNAMODB_TABLE").expect("DYNAMODB_TABLE must be set");
    let repository = DynamoDbRepository::new(dynamodb_client, table_name);
    let timestream_repository = TimestreamRepository::from_env().await?;
    
    // Process each SQS message
    for record in event.payload.records {
        if let Some(body) = record.body {
            info!("Processing SQS message: {}", body);
            
            // Parse the message body as a PerformanceCalculationRequest
            match serde_json::from_str::<PerformanceCalculationRequest>(&body) {
                Ok(request) => {
                    info!("Processing performance calculation request for portfolio: {}", request.portfolio_id);
                    
                    // Get the portfolio from DynamoDB
                    let portfolio = match repository.get_portfolio(&request.portfolio_id).await? {
                        Some(p) => p,
                        None => {
                            error!("Portfolio not found: {}", request.portfolio_id);
                            continue;
                        }
                    };
                    
                    // Get the accounts for the portfolio
                    let (accounts, _) = repository.list_accounts_by_portfolio(&request.portfolio_id, None, None).await?;
                    
                    if accounts.is_empty() {
                        error!("No accounts found for portfolio: {}", request.portfolio_id);
                        continue;
                    }
                    
                    // Get transactions for each account
                    let mut all_transactions = Vec::new();
                    for account in &accounts {
                        let (transactions, _) = repository.list_transactions_by_account(&account.id, None, None).await?;
                        all_transactions.extend(transactions);
                    }
                    
                    // Filter transactions by date range
                    let filtered_transactions = all_transactions.into_iter()
                        .filter(|t| t.transaction_date >= request.start_date && t.transaction_date <= request.end_date)
                        .collect::<Vec<_>>();
                    
                    if filtered_transactions.is_empty() {
                        error!("No transactions found for portfolio: {} in date range", request.portfolio_id);
                        continue;
                    }
                    
                    // Get valuations for the portfolio
                    let (valuations, _) = repository.list_valuations_by_portfolio(&request.portfolio_id, None, None).await?;
                    
                    if valuations.is_empty() {
                        error!("No valuations found for portfolio: {}", request.portfolio_id);
                        continue;
                    }
                    
                    // Filter valuations by date range and add buffer periods for calculations
                    // We need valuations from before the start date to calculate returns properly
                    let buffer_days = 30; // Get valuations from 30 days before start date
                    let buffer_start_date = request.start_date - chrono::Duration::days(buffer_days);
                    
                    let filtered_valuations = valuations.into_iter()
                        .filter(|v| v.date >= buffer_start_date && v.date <= request.end_date)
                        .collect::<Vec<_>>();
                    
                    if filtered_valuations.len() < 2 {
                        error!("Insufficient valuations found for portfolio: {} in date range", request.portfolio_id);
                        continue;
                    }
                    
                    // Calculate performance metrics using sophisticated algorithms
                    
                    // Calculate TWR (Time-Weighted Return)
                    let twr = match calculate_twr(&filtered_transactions, &filtered_valuations) {
                        Ok(value) => value,
                        Err(e) => {
                            error!("Failed to calculate TWR: {}", e);
                            continue;
                        }
                    };
                    
                    // Calculate MWR (Money-Weighted Return)
                    let mwr = match calculate_mwr(&filtered_transactions, &filtered_valuations) {
                        Ok(value) => value,
                        Err(e) => {
                            error!("Failed to calculate MWR: {}", e);
                            continue;
                        }
                    };
                    
                    // Get risk-free rate from configuration or use default
                    let risk_free_rate = 0.02; // 2% annual risk-free rate (could be fetched from a service)
                    
                    // Calculate risk metrics
                    let volatility = match calculate_volatility(&filtered_transactions, &filtered_valuations) {
                        Ok(value) => value,
                        Err(e) => {
                            warn!("Failed to calculate volatility: {}", e);
                            None
                        }
                    };
                    
                    let sharpe_ratio = match (volatility, calculate_sharpe_ratio(twr, volatility.unwrap_or(0.0), risk_free_rate)) {
                        (Some(vol), Ok(ratio)) if vol > 0.0 => Some(ratio),
                        _ => None,
                    };
                    
                    let max_drawdown = match calculate_max_drawdown(&filtered_valuations) {
                        Ok(value) => Some(value),
                        Err(e) => {
                            warn!("Failed to calculate maximum drawdown: {}", e);
                            None
                        }
                    };
                    
                    // Calculate benchmark metrics if a benchmark is specified
                    let (benchmark_return, tracking_error, information_ratio) = 
                        if let Some(benchmark_id) = &request.benchmark_id {
                            // Get benchmark data
                            match repository.get_benchmark(benchmark_id).await? {
                                Some(benchmark) => {
                                    // Get benchmark returns
                                    let (benchmark_returns, _) = repository.list_benchmark_returns(
                                        benchmark_id, 
                                        buffer_start_date, 
                                        request.end_date,
                                        None
                                    ).await?;
                                    
                                    if benchmark_returns.is_empty() {
                                        warn!("No benchmark returns found for benchmark: {}", benchmark_id);
                                        (None, None, None)
                                    } else {
                                        // Calculate benchmark return over the period
                                        let benchmark_return = calculate_benchmark_return(&benchmark_returns, request.start_date, request.end_date);
                                        
                                        // Calculate tracking error and information ratio if we have volatility
                                        let (tracking_error, information_ratio) = if let Some(vol) = volatility {
                                            // Convert valuations to return series for tracking error calculation
                                            let portfolio_returns = convert_valuations_to_returns(&filtered_valuations);
                                            
                                            // Convert benchmark returns to the same format
                                            let formatted_benchmark_returns = benchmark_returns.iter()
                                                .map(|br| (br.date, br.return_value))
                                                .collect::<Vec<_>>();
                                            
                                            // Calculate tracking error
                                            let tracking_error = match calculate_tracking_error(
                                                &portfolio_returns,
                                                &formatted_benchmark_returns,
                                                true // Annualize
                                            ) {
                                                Ok(te) => Some(te),
                                                Err(e) => {
                                                    warn!("Failed to calculate tracking error: {}", e);
                                                    None
                                                }
                                            };
                                            
                                            // Calculate information ratio
                                            let information_ratio = match (tracking_error, benchmark_return) {
                                                (Some(te), Some(br)) => {
                                                    match calculate_information_ratio(twr, br, te) {
                                                        Ok(ir) => Some(ir),
                                                        Err(e) => {
                                                            warn!("Failed to calculate information ratio: {}", e);
                                                            None
                                                        }
                                                    }
                                                },
                                                _ => None,
                                            };
                                            
                                            (tracking_error, information_ratio)
                                        } else {
                                            (None, None)
                                        };
                                        
                                        (benchmark_return, tracking_error, information_ratio)
                                    }
                                },
                                None => {
                                    warn!("Benchmark not found: {}", benchmark_id);
                                    (None, None, None)
                                }
                            }
                        } else {
                            (None, None, None)
                        };
                    
                    // Create the performance calculation result
                    let result = PerformanceCalculationResult {
                        portfolio_id: request.portfolio_id.clone(),
                        start_date: request.start_date,
                        end_date: request.end_date,
                        twr,
                        mwr,
                        volatility,
                        sharpe_ratio,
                        max_drawdown,
                        benchmark_id: request.benchmark_id.clone(),
                        benchmark_return,
                        tracking_error,
                        information_ratio,
                        calculated_at: Utc::now(),
                    };
                    
                    // Store the result in Timestream
                    let data_point = PerformanceDataPoint {
                        portfolio_id: result.portfolio_id.clone(),
                        timestamp: result.calculated_at,
                        twr: result.twr,
                        mwr: result.mwr,
                        volatility: result.volatility,
                        sharpe_ratio: result.sharpe_ratio,
                        max_drawdown: result.max_drawdown,
                        benchmark_id: result.benchmark_id.clone(),
                        benchmark_return: result.benchmark_return,
                        tracking_error: result.tracking_error,
                        information_ratio: result.information_ratio,
                    };
                    
                    timestream_repository.store_performance_data(&data_point).await?;
                    
                    // Create an audit record
                    let audit_record = AuditRecord {
                        id: Uuid::new_v4().to_string(),
                        event_type: "PERFORMANCE_CALCULATED".to_string(),
                        entity_id: request.portfolio_id.clone(),
                        entity_type: "PORTFOLIO".to_string(),
                        user_id: None, // System-generated
                        timestamp: Utc::now(),
                        details: serde_json::to_value(&result)?,
                    };
                    
                    // Store the audit record in DynamoDB
                    repository.create_audit_record(&audit_record).await?;
                    
                    info!("Performance calculation completed for portfolio: {}", request.portfolio_id);
                }
                Err(e) => {
                    error!("Failed to parse performance calculation request: {}", e);
                }
            }
        }
    }
    
    Ok(())
}

// Simplified performance calculation functions
// In a real implementation, these would use more sophisticated algorithms

/// Calculate Time-Weighted Return using the Modified Dietz method with daily valuation points
/// 
/// This implementation:
/// - Handles multiple cash flows during the period
/// - Accounts for the timing of cash flows
/// - Uses geometric linking of sub-period returns
/// - Handles both deposits and withdrawals correctly
fn calculate_twr(transactions: &[dynamodb_repository::Transaction], valuations: &[dynamodb_repository::Valuation]) -> Result<f64, CalculationError> {
    if transactions.is_empty() || valuations.len() < 2 {
        return Err(CalculationError::InsufficientData("Need at least two valuations for TWR calculation".to_string()));
    }

    // Sort valuations by date
    let mut sorted_valuations = valuations.to_vec();
    sorted_valuations.sort_by(|a, b| a.date.cmp(&b.date));
    
    // Calculate sub-period returns and link them geometrically
    let mut sub_period_returns: Vec<f64> = Vec::new();
    
    for window in sorted_valuations.windows(2) {
        let start_valuation = &window[0];
        let end_valuation = &window[1];
        
        // Get cash flows during this sub-period
        let period_flows: Vec<&dynamodb_repository::Transaction> = transactions.iter()
            .filter(|t| {
                let tx_date = t.transaction_date;
                tx_date > start_valuation.date && tx_date <= end_valuation.date
            })
            .collect();
        
        // Calculate weighted cash flows
        let mut weighted_flows = 0.0;
        for flow in &period_flows {
            // Weight is the proportion of the period that the money was invested
            let total_days = (end_valuation.date - start_valuation.date).num_days() as f64;
            let days_invested = (end_valuation.date - flow.transaction_date).num_days() as f64;
            let weight = days_invested / total_days;
            
            // Deposits are negative flows from portfolio perspective, withdrawals are positive
            let flow_amount = match flow.transaction_type.as_str() {
                "DEPOSIT" => -flow.amount,
                "WITHDRAWAL" => flow.amount,
                _ => 0.0, // Ignore other transaction types for cash flow purposes
            };
            
            weighted_flows += flow_amount * weight;
        }
        
        // Calculate sub-period return using Modified Dietz method
        let start_value = start_valuation.value;
        let end_value = end_valuation.value;
        
        // Sum of all flows (not weighted)
        let total_flows: f64 = period_flows.iter()
            .map(|f| match f.transaction_type.as_str() {
                "DEPOSIT" => -f.amount,
                "WITHDRAWAL" => f.amount,
                _ => 0.0,
            })
            .sum();
        
        // Modified Dietz formula: r = (EMV - BMV - CF) / (BMV + weighted CF)
        let sub_period_return = if start_value + weighted_flows == 0.0 {
            0.0 // Avoid division by zero
        } else {
            (end_value - start_value - total_flows) / (start_value + weighted_flows)
        };
        
        sub_period_returns.push(sub_period_return);
    }
    
    // Geometrically link the sub-period returns
    // (1+r₁) × (1+r₂) × ... × (1+rₙ) - 1
    let linked_return = sub_period_returns.iter()
        .fold(1.0, |acc, &r| acc * (1.0 + r)) - 1.0;
    
    Ok(linked_return)
}

fn calculate_mwr(transactions: &[dynamodb_repository::Transaction], valuations: &[dynamodb_repository::Valuation]) -> Result<f64, CalculationError> {
    if transactions.is_empty() || valuations.is_empty() {
        return Err(CalculationError::InsufficientData("Need transactions and valuations for MWR calculation".to_string()));
    }

    // Get initial and final valuations
    let initial_valuation = valuations.iter()
        .min_by_key(|v| v.date)
        .ok_or_else(|| CalculationError::InsufficientData("No initial valuation found".to_string()))?;
    
    let final_valuation = valuations.iter()
        .max_by_key(|v| v.date)
        .ok_or_else(|| CalculationError::InsufficientData("No final valuation found".to_string()))?;
    
    // Create cash flow timeline
    let mut cash_flows: Vec<(DateTime<Utc>, f64)> = Vec::new();
    
    // Initial valuation is treated as a negative cash flow (money invested)
    cash_flows.push((initial_valuation.date, -initial_valuation.value));
    
    // Add all transactions
    for tx in transactions {
        if tx.transaction_date > initial_valuation.date && tx.transaction_date <= final_valuation.date {
            let flow_amount = match tx.transaction_type.as_str() {
                "DEPOSIT" => -tx.amount, // Money going into the portfolio (negative from IRR perspective)
                "WITHDRAWAL" => tx.amount, // Money coming out of the portfolio (positive from IRR perspective)
                _ => 0.0, // Ignore other transaction types for cash flow purposes
            };
            
            cash_flows.push((tx.transaction_date, flow_amount));
        }
    }
    
    // Final valuation is treated as a positive cash flow (money returned)
    cash_flows.push((final_valuation.date, final_valuation.value));
    
    // Sort cash flows by date
    cash_flows.sort_by(|a, b| a.0.cmp(&b.0));
    
    // Calculate IRR using Newton-Raphson method
    let irr = calculate_irr(&cash_flows, 0.1, 0.0001, 100)?;
    
    Ok(irr)
}

/// Calculate Internal Rate of Return using Newton-Raphson method
fn calculate_irr(
    cash_flows: &[(DateTime<Utc>, f64)], 
    initial_guess: f64,
    precision: f64,
    max_iterations: usize
) -> Result<f64, CalculationError> {
    if cash_flows.len() < 2 {
        return Err(CalculationError::InsufficientData("Need at least two cash flows for IRR calculation".to_string()));
    }
    
    let base_date = cash_flows[0].0;
    
    // Newton-Raphson iteration
    let mut rate = initial_guess;
    let mut iteration = 0;
    
    while iteration < max_iterations {
        let mut npv = 0.0;
        let mut derivative = 0.0;
        
        for (date, amount) in cash_flows {
            let years = (*date - base_date).num_days() as f64 / 365.25;
            let discount_factor = (1.0 + rate).powf(years);
            
            npv += amount / discount_factor;
            derivative -= years * amount / ((1.0 + rate).powf(years + 1.0));
        }
        
        // Check if we've reached desired precision
        if npv.abs() < precision {
            return Ok(rate);
        }
        
        // Avoid division by zero
        if derivative.abs() < 1e-10 {
            return Err(CalculationError::ConvergenceFailure("IRR calculation failed to converge - derivative too small".to_string()));
        }
        
        // Update rate using Newton-Raphson formula
        let new_rate = rate - npv / derivative;
        
        // Check for convergence
        if (new_rate - rate).abs() < precision {
            return Ok(new_rate);
        }
        
        rate = new_rate;
        iteration += 1;
    }
    
    Err(CalculationError::ConvergenceFailure(format!("IRR calculation failed to converge after {} iterations", max_iterations)))
}

fn calculate_volatility(transactions: &[dynamodb_repository::Transaction], valuations: &[dynamodb_repository::Valuation]) -> Result<Option<f64>, CalculationError> {
    if valuations.len() < 30 {
        return Err(CalculationError::InsufficientData(
            "Need at least 30 valuation points for reliable volatility calculation".to_string()
        ));
    }
    
    // Sort valuations by date
    let mut sorted_valuations = valuations.to_vec();
    sorted_valuations.sort_by(|a, b| a.date.cmp(&b.date));
    
    // Calculate daily returns
    let mut returns: Vec<f64> = Vec::new();
    
    for window in sorted_valuations.windows(2) {
        let start_valuation = &window[0];
        let end_valuation = &window[1];
        
        // Get cash flows during this sub-period
        let period_flows: Vec<&dynamodb_repository::Transaction> = transactions.iter()
            .filter(|t| {
                let tx_date = t.transaction_date;
                tx_date > start_valuation.date && tx_date <= end_valuation.date
            })
            .collect();
        
        // Sum of all flows
        let total_flows: f64 = period_flows.iter()
            .map(|f| match f.transaction_type.as_str() {
                "DEPOSIT" => -f.amount,
                "WITHDRAWAL" => f.amount,
                _ => 0.0,
            })
            .sum();
        
        // Calculate return for this period
        let period_return = if start_valuation.value == 0.0 {
            0.0 // Avoid division by zero
        } else {
            (end_valuation.value - start_valuation.value - total_flows) / start_valuation.value
        };
        
        returns.push(period_return);
    }
    
    // Calculate exponentially weighted volatility
    // This gives more weight to recent observations
    let lambda = 0.94; // Decay factor (typical value for daily returns)
    let mut weights: Vec<f64> = Vec::with_capacity(returns.len());
    
    // Calculate weights
    let mut sum_weights = 0.0;
    for i in 0..returns.len() {
        let weight = (1.0 - lambda) * lambda.powi((returns.len() - 1 - i) as i32);
        weights.push(weight);
        sum_weights += weight;
    }
    
    // Normalize weights
    for weight in &mut weights {
        *weight /= sum_weights;
    }
    
    // Calculate mean return
    let mean_return: f64 = returns.iter()
        .zip(weights.iter())
        .map(|(&r, &w)| r * w)
        .sum();
    
    // Calculate variance
    let variance: f64 = returns.iter()
        .zip(weights.iter())
        .map(|(&r, &w)| w * (r - mean_return).powi(2))
        .sum();
    
    // Calculate standard deviation
    let volatility = variance.sqrt();
    
    // Annualize volatility (assuming daily returns, multiply by sqrt(252))
    let avg_days = if returns.len() > 1 {
        let first_date = sorted_valuations.first().unwrap().date;
        let last_date = sorted_valuations.last().unwrap().date;
        let total_days = (last_date - first_date).num_days() as f64;
        total_days / (returns.len() as f64)
    } else {
        1.0 // Default to daily if we can't determine
    };
    
    // Calculate annualization factor
    let annualization_factor = (252.0 / avg_days).sqrt();
    Ok(Some(volatility * annualization_factor))
}

fn calculate_sharpe_ratio(return_value: f64, volatility: f64, risk_free_rate: f64) -> Result<f64, CalculationError> {
    if volatility <= 0.0 {
        return Err(CalculationError::InvalidInput("Volatility must be positive for Sharpe ratio calculation".to_string()));
    }
    
    // Calculate excess return
    let excess_return = return_value - risk_free_rate;
    
    // Calculate Sharpe ratio
    Ok(excess_return / volatility)
}

fn calculate_max_drawdown(valuations: &[dynamodb_repository::Valuation]) -> Result<f64, CalculationError> {
    if valuations.len() < 2 {
        return Err(CalculationError::InsufficientData("Need at least two valuations for drawdown calculation".to_string()));
    }
    
    // Sort valuations by date
    let mut sorted_valuations = valuations.to_vec();
    sorted_valuations.sort_by(|a, b| a.date.cmp(&b.date));
    
    let mut max_value = sorted_valuations[0].value;
    let mut max_drawdown = 0.0;
    
    for valuation in &sorted_valuations {
        if valuation.value > max_value {
            // New peak
            max_value = valuation.value;
        } else {
            // Calculate current drawdown
            let current_drawdown = (max_value - valuation.value) / max_value;
            
            // Check if this is a new maximum drawdown
            if current_drawdown > max_drawdown {
                max_drawdown = current_drawdown;
            }
        }
    }
    
    Ok(max_drawdown)
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .with_target(false)
        .without_time()
        .init();
    
    info!("Performance Lambda starting up");
    
    // Initialize resilience features
    performance_calculator::resilience::init_circuit_breakers();
    
    // Register health checks
    performance_calculator::resilience::register_health_check(
        "dynamodb",
        Box::new(|config| {
            Box::pin(async move {
                let dynamodb_client = DynamoDbClient::new(&config);
                let table_name = env::var("DYNAMODB_TABLE").expect("DYNAMODB_TABLE must be set");
                
                match dynamodb_client.describe_table().table_name(&table_name).send().await {
                    Ok(_) => performance_calculator::resilience::HealthCheckResult::healthy("DynamoDB is available"),
                    Err(e) => performance_calculator::resilience::HealthCheckResult::unhealthy(format!("DynamoDB health check failed: {}", e)),
                }
            })
        }),
    );
    
    performance_calculator::resilience::register_health_check(
        "sqs",
        Box::new(|config| {
            Box::pin(async move {
                let sqs_client = aws_sdk_sqs::Client::new(&config);
                let queue_url = env::var("SQS_QUEUE_URL").expect("SQS_QUEUE_URL must be set");
                
                match sqs_client.get_queue_attributes().queue_url(queue_url).attribute_names(aws_sdk_sqs::model::QueueAttributeName::All).send().await {
                    Ok(_) => performance_calculator::resilience::HealthCheckResult::healthy("SQS is available"),
                    Err(e) => performance_calculator::resilience::HealthCheckResult::unhealthy(format!("SQS health check failed: {}", e)),
                }
            })
        }),
    );
    
    // Start health check monitor
    let health_monitor = performance_calculator::resilience::start_health_check_monitor();
    
    // Run the Lambda function
    let result = run(service_fn(function_handler)).await;
    
    // Stop health check monitor
    health_monitor.stop().await;
    
    info!("Performance Lambda shutting down");
    
    result
}

// Add a trait for the Timestream repository
#[async_trait]
pub trait TimeseriesRepository {
    async fn save_performance_metrics(
        &self, 
        portfolio_id: &str, 
        metrics: &PerformanceMetrics,
        timestamp: DateTime<Utc>
    ) -> Result<(), RepositoryError>;
    
    async fn get_performance_history(
        &self,
        portfolio_id: &str,
        start_date: DateTime<Utc>,
        end_date: DateTime<Utc>,
        interval: TimeInterval
    ) -> Result<Vec<PerformanceMetrics>, RepositoryError>;
}

// Update relevant methods to support pagination
async fn list_transactions_paginated(
    &self,
    pagination: PaginationOptions
) -> Result<PaginatedResult<Transaction>, RepositoryError>;

// Add to Repository trait
async fn batch_get_items(
    &self, 
    ids: &[String], 
    entity_type: &str
) -> Result<Vec<HashMap<String, AttributeValue>>, RepositoryError>;

async fn batch_write_items<T: Serialize>(
    &self, 
    items: &[T], 
    entity_type: &str
) -> Result<(), RepositoryError>;

// Add to Repository trait
async fn transact_write_items(
    &self,
    operations: Vec<TransactWriteItem>
) -> Result<(), RepositoryError>;

// Add resilience to repository operations
async fn get_item_with_resilience(&self, id: &str) -> Result<Option<Item>, RepositoryError> {
    with_retry_and_circuit_breaker(
        "get_item",
        RetryConfig::default(),
        || async {
            self.get_item(id).await
        }
    ).await.map_err(|e| RepositoryError::Resilience(e.to_string()))
}

// Add tenant ID to repository methods
async fn get_item(&self, tenant_id: &str, id: &str) -> Result<Option<Item>, RepositoryError>;

// Or create a tenant-specific repository factory
pub fn for_tenant(&self, tenant_id: String) -> TenantRepository {
    TenantRepository {
        repository: self.clone(),
        tenant_id,
    }
}

impl DynamoDbRepository {
    pub fn new(client: DynamoDbClient, table_name: String) -> Self {
        Self { client, table_name }
    }

    // Resilient method to get an item with circuit breaker and retry
    async fn get_item_with_resilience<T: DeserializeOwned>(
        &self,
        id: &str,
        entity_type: &str
    ) -> Result<Option<T>, RepositoryError> {
        // Get circuit breaker for DynamoDB
        let circuit_breaker = performance_calculator::resilience::get_circuit_breaker("dynamodb")
            .ok_or_else(|| RepositoryError::Other("Failed to get circuit breaker for DynamoDB".to_string()))?;
        
        // Check if circuit breaker is open
        if !circuit_breaker.is_closed() {
            return Err(RepositoryError::CircuitBreakerOpen("DynamoDB circuit breaker is open".to_string()));
        }
        
        // Create retry policy
        let retry_policy = performance_calculator::resilience::create_retry_policy();
        
        // Execute with retry
        let result = retry_policy
            .retry_if(
                || async {
                    let result = self.client
                        .get_item()
                        .table_name(&self.table_name)
                        .key("id", AttributeValue::S(id.to_string()))
                        .key("entity_type", AttributeValue::S(entity_type.to_string()))
                        .send()
                        .await;
                    
                    match result {
                        Ok(response) => Ok(response),
                        Err(e) => {
                            // Record failure in circuit breaker
                            circuit_breaker.record_failure();
                            Err(e)
                        }
                    }
                },
                |err: &aws_sdk_dynamodb::Error| {
                    // Only retry on transient errors
                    matches!(
                        err.code(),
                        Some("ProvisionedThroughputExceededException") | 
                        Some("ThrottlingException") |
                        Some("InternalServerError") |
                        Some("ServiceUnavailable")
                    )
                }
            )
            .await
            .map_err(|e| {
                // If we've exhausted retries, record a failure
                circuit_breaker.record_failure();
                RepositoryError::DynamoDb(e)
            })?;
        
        // Record success in circuit breaker
        circuit_breaker.record_success();
        
        if let Some(item) = result.item {
            Ok(Some(self.from_dynamodb_item(item)?))
        } else {
            Ok(None)
        }
    }
    
    // Resilient method to query items with circuit breaker and retry
    async fn query_with_resilience(
        &self,
        query_builder: aws_sdk_dynamodb::operation::query::QueryFluentBuilder,
    ) -> Result<aws_sdk_dynamodb::operation::query::QueryOutput, RepositoryError> {
        // Get circuit breaker for DynamoDB
        let circuit_breaker = performance_calculator::resilience::get_circuit_breaker("dynamodb")
            .ok_or_else(|| RepositoryError::Other("Failed to get circuit breaker for DynamoDB".to_string()))?;
        
        // Check if circuit breaker is open
        if !circuit_breaker.is_closed() {
            return Err(RepositoryError::CircuitBreakerOpen("DynamoDB circuit breaker is open".to_string()));
        }
        
        // Create retry policy
        let retry_policy = performance_calculator::resilience::create_retry_policy();
        
        // Execute with retry
        let result = retry_policy
            .retry_if(
                || async {
                    let result = query_builder.clone().send().await;
                    
                    match result {
                        Ok(response) => Ok(response),
                        Err(e) => {
                            // Record failure in circuit breaker
                            circuit_breaker.record_failure();
                            Err(e)
                        }
                    }
                },
                |err: &aws_sdk_dynamodb::Error| {
                    // Only retry on transient errors
                    matches!(
                        err.code(),
                        Some("ProvisionedThroughputExceededException") | 
                        Some("ThrottlingException") |
                        Some("InternalServerError") |
                        Some("ServiceUnavailable")
                    )
                }
            )
            .await
            .map_err(|e| {
                // If we've exhausted retries, record a failure
                circuit_breaker.record_failure();
                RepositoryError::DynamoDb(e)
            })?;
        
        // Record success in circuit breaker
        circuit_breaker.record_success();
        
        Ok(result)
    }
    
    // Resilient method to put an item with circuit breaker and retry
    async fn put_item_with_resilience(
        &self,
        item: HashMap<String, AttributeValue>,
    ) -> Result<(), RepositoryError> {
        // Get circuit breaker for DynamoDB
        let circuit_breaker = performance_calculator::resilience::get_circuit_breaker("dynamodb")
            .ok_or_else(|| RepositoryError::Other("Failed to get circuit breaker for DynamoDB".to_string()))?;
        
        // Check if circuit breaker is open
        if !circuit_breaker.is_closed() {
            return Err(RepositoryError::CircuitBreakerOpen("DynamoDB circuit breaker is open".to_string()));
        }
        
        // Create retry policy
        let retry_policy = performance_calculator::resilience::create_retry_policy();
        
        // Execute with retry
        retry_policy
            .retry_if(
                || async {
                    let result = self.client
                        .put_item()
                        .table_name(&self.table_name)
                        .set_item(Some(item.clone()))
                        .send()
                        .await;
                    
                    match result {
                        Ok(_) => Ok(()),
                        Err(e) => {
                            // Record failure in circuit breaker
                            circuit_breaker.record_failure();
                            Err(e)
                        }
                    }
                },
                |err: &aws_sdk_dynamodb::Error| {
                    // Only retry on transient errors
                    matches!(
                        err.code(),
                        Some("ProvisionedThroughputExceededException") | 
                        Some("ThrottlingException") |
                        Some("InternalServerError") |
                        Some("ServiceUnavailable")
                    )
                }
            )
            .await
            .map_err(|e| {
                // If we've exhausted retries, record a failure
                circuit_breaker.record_failure();
                RepositoryError::DynamoDb(e)
            })?;
        
        // Record success in circuit breaker
        circuit_breaker.record_success();
        
        Ok(())
    }

    async fn save_performance_result(&self, result: &PerformanceCalculationResult) -> Result<(), RepositoryError> {
        let dynamodb_item = self.to_dynamodb_item(result, "performance_result")?;
        self.put_item_with_resilience(dynamodb_item).await
    }
    
    async fn get_latest_performance_result(&self, portfolio_id: &str) -> Result<Option<PerformanceCalculationResult>, RepositoryError> {
        let query_builder = self.client
            .query()
            .table_name(&self.table_name)
            .index_name("PortfolioIndex")
            .key_condition_expression("portfolio_id = :portfolio_id AND entity_type = :entity_type")
            .expression_attribute_values(":portfolio_id", AttributeValue::S(portfolio_id.to_string()))
            .expression_attribute_values(":entity_type", AttributeValue::S("performance_result".to_string()))
            .limit(1)
            .scan_index_forward(false); // Get the most recent first
            
        let result = self.query_with_resilience(query_builder).await?;
            
        if let Some(items) = result.items {
            if !items.is_empty() {
                return Ok(Some(self.from_dynamodb_item(items[0].clone())?));
            }
        }
        
        Ok(None)
    }
    
    async fn get_portfolio(&self, portfolio_id: &str) -> Result<Option<dynamodb_repository::Portfolio>, RepositoryError> {
        self.get_item_with_resilience(portfolio_id, "portfolio").await
    }
    
    async fn list_accounts_by_portfolio(
        &self, 
        portfolio_id: &str,
        limit: Option<i32>,
        start_key: Option<HashMap<String, AttributeValue>>
    ) -> Result<(Vec<dynamodb_repository::Account>, Option<HashMap<String, AttributeValue>>), RepositoryError> {
        let mut query_builder = self.client
            .query()
            .table_name(&self.table_name)
            .index_name("PortfolioIndex")
            .key_condition_expression("portfolio_id = :portfolio_id AND entity_type = :entity_type")
            .expression_attribute_values(":portfolio_id", AttributeValue::S(portfolio_id.to_string()))
            .expression_attribute_values(":entity_type", AttributeValue::S("account".to_string()));
            
        if let Some(limit_val) = limit {
            query_builder = query_builder.limit(limit_val);
        }
        
        if let Some(start_key_val) = start_key {
            query_builder = query_builder.set_exclusive_start_key(Some(start_key_val));
        }
        
        let result = self.query_with_resilience(query_builder).await?;
        
        let accounts = if let Some(items) = result.items {
            let mut accounts = Vec::with_capacity(items.len());
            for item in items {
                accounts.push(self.account_from_dynamodb_item(item)?);
            }
            accounts
        } else {
            Vec::new()
        };
        
        Ok((accounts, result.last_evaluated_key))
    }
    
    async fn list_transactions_by_account(
        &self, 
        account_id: &str,
        limit: Option<i32>,
        start_key: Option<HashMap<String, AttributeValue>>
    ) -> Result<(Vec<dynamodb_repository::Transaction>, Option<HashMap<String, AttributeValue>>), RepositoryError> {
        let mut query_builder = self.client
            .query()
            .table_name(&self.table_name)
            .index_name("AccountIndex")
            .key_condition_expression("account_id = :account_id AND entity_type = :entity_type")
            .expression_attribute_values(":account_id", AttributeValue::S(account_id.to_string()))
            .expression_attribute_values(":entity_type", AttributeValue::S("transaction".to_string()));
            
        if let Some(limit_val) = limit {
            query_builder = query_builder.limit(limit_val);
        }
        
        if let Some(start_key_val) = start_key {
            query_builder = query_builder.set_exclusive_start_key(Some(start_key_val));
        }
        
        let result = self.query_with_resilience(query_builder).await?;
        
        let transactions = if let Some(items) = result.items {
            let mut transactions = Vec::with_capacity(items.len());
            for item in items {
                transactions.push(self.transaction_from_dynamodb_item(item)?);
            }
            transactions
        } else {
            Vec::new()
        };
        
        Ok((transactions, result.last_evaluated_key))
    }
    
    async fn list_valuations_by_portfolio(
        &self, 
        portfolio_id: &str,
        limit: Option<i32>,
        start_key: Option<HashMap<String, AttributeValue>>
    ) -> Result<(Vec<dynamodb_repository::Valuation>, Option<HashMap<String, AttributeValue>>), RepositoryError> {
        let mut query_builder = self.client
            .query()
            .table_name(&self.table_name)
            .index_name("PortfolioIndex")
            .key_condition_expression("portfolio_id = :portfolio_id AND entity_type = :entity_type")
            .expression_attribute_values(":portfolio_id", AttributeValue::S(portfolio_id.to_string()))
            .expression_attribute_values(":entity_type", AttributeValue::S("valuation".to_string()));
            
        if let Some(limit_val) = limit {
            query_builder = query_builder.limit(limit_val);
        }
        
        if let Some(start_key_val) = start_key {
            query_builder = query_builder.set_exclusive_start_key(Some(start_key_val));
        }
        
        let result = self.query_with_resilience(query_builder).await?;
        
        let valuations = if let Some(items) = result.items {
            let mut valuations = Vec::with_capacity(items.len());
            for item in items {
                valuations.push(self.valuation_from_dynamodb_item(item)?);
            }
            valuations
        } else {
            Vec::new()
        };
        
        Ok((valuations, result.last_evaluated_key))
    }
    
    async fn get_benchmark(&self, benchmark_id: &str) -> Result<Option<dynamodb_repository::Benchmark>, RepositoryError> {
        self.get_item_with_resilience(benchmark_id, "benchmark").await
    }
    
    async fn list_benchmark_returns(
        &self, 
        benchmark_id: &str,
        start_date: DateTime<Utc>,
        end_date: DateTime<Utc>,
        limit: Option<i32>
    ) -> Result<(Vec<dynamodb_repository::BenchmarkReturn>, Option<HashMap<String, AttributeValue>>), RepositoryError> {
        let mut query_builder = self.client
            .query()
            .table_name(&self.table_name)
            .index_name("BenchmarkIndex")
            .key_condition_expression("benchmark_id = :benchmark_id AND entity_type = :entity_type")
            .filter_expression("date BETWEEN :start_date AND :end_date")
            .expression_attribute_values(":benchmark_id", AttributeValue::S(benchmark_id.to_string()))
            .expression_attribute_values(":entity_type", AttributeValue::S("benchmark_return".to_string()))
            .expression_attribute_values(":start_date", AttributeValue::S(start_date.to_rfc3339()))
            .expression_attribute_values(":end_date", AttributeValue::S(end_date.to_rfc3339()));
            
        if let Some(limit_val) = limit {
            query_builder = query_builder.limit(limit_val);
        }
        
        let result = self.query_with_resilience(query_builder).await?;
        
        let benchmark_returns = if let Some(items) = result.items {
            let mut returns = Vec::with_capacity(items.len());
            for item in items {
                returns.push(self.benchmark_return_from_dynamodb_item(item)?);
            }
            returns
        } else {
            Vec::new()
        };
        
        Ok((benchmark_returns, result.last_evaluated_key))
    }
    
    // Helper methods for converting between domain models and DynamoDB items
    
    fn to_dynamodb_item<T: Serialize>(&self, item: &T, entity_type: &str) -> Result<HashMap<String, AttributeValue>, RepositoryError> {
        let mut dynamodb_item = serde_dynamo::to_item(item)
            .map_err(|e| RepositoryError::Serialization(e.to_string()))?;
            
        dynamodb_item.insert("entity_type".to_string(), AttributeValue::S(entity_type.to_string()));
        
        Ok(dynamodb_item)
    }
    
    fn from_dynamodb_item<T: DeserializeOwned>(&self, item: HashMap<String, AttributeValue>) -> Result<T, RepositoryError> {
        serde_dynamo::from_item(item)
            .map_err(|e| RepositoryError::Deserialization(e.to_string()))
    }
    
    fn portfolio_from_dynamodb_item(&self, item: HashMap<String, AttributeValue>) -> Result<dynamodb_repository::Portfolio, RepositoryError> {
        self.from_dynamodb_item(item)
    }
    
    fn account_from_dynamodb_item(&self, item: HashMap<String, AttributeValue>) -> Result<dynamodb_repository::Account, RepositoryError> {
        self.from_dynamodb_item(item)
    }
    
    fn transaction_from_dynamodb_item(&self, item: HashMap<String, AttributeValue>) -> Result<dynamodb_repository::Transaction, RepositoryError> {
        self.from_dynamodb_item(item)
    }
    
    fn valuation_from_dynamodb_item(&self, item: HashMap<String, AttributeValue>) -> Result<dynamodb_repository::Valuation, RepositoryError> {
        self.from_dynamodb_item(item)
    }
    
    fn benchmark_from_dynamodb_item(&self, item: HashMap<String, AttributeValue>) -> Result<dynamodb_repository::Benchmark, RepositoryError> {
        self.from_dynamodb_item(item)
    }
    
    fn benchmark_return_from_dynamodb_item(&self, item: HashMap<String, AttributeValue>) -> Result<dynamodb_repository::BenchmarkReturn, RepositoryError> {
        self.from_dynamodb_item(item)
    }
}

/// Calculate Sortino Ratio - a variation of Sharpe ratio that only penalizes downside volatility
/// 
/// This implementation:
/// - Focuses on downside risk rather than total volatility
/// - Uses a minimum acceptable return (MAR) threshold
/// - Better represents risk for asymmetric return distributions
fn calculate_sortino_ratio(
    return_value: f64,
    returns: &[f64],
    risk_free_rate: f64,
    mar: Option<f64>
) -> Result<f64, CalculationError> {
    if returns.is_empty() {
        return Err(CalculationError::InsufficientData("Need return series for Sortino ratio calculation".to_string()));
    }
    
    // Use risk-free rate as MAR if not specified
    let minimum_acceptable_return = mar.unwrap_or(risk_free_rate);
    
    // Calculate downside deviation
    let downside_returns: Vec<f64> = returns.iter()
        .filter_map(|&r| {
            if r < minimum_acceptable_return {
                Some((r - minimum_acceptable_return).powi(2))
            } else {
                None
            }
        })
        .collect();
    
    if downside_returns.is_empty() {
        // No downside returns, perfect Sortino ratio (infinity)
        // Return a large number instead
        return Ok(1000.0);
    }
    
    let downside_deviation = (downside_returns.iter().sum::<f64>() / downside_returns.len() as f64).sqrt();
    
    if downside_deviation == 0.0 {
        // Avoid division by zero
        return Ok(1000.0);
    }
    
    // Calculate Sortino ratio
    Ok((return_value - risk_free_rate) / downside_deviation)
}

/// Calculate Treynor Ratio - measures returns earned in excess of risk-free rate per unit of market risk
/// 
/// This implementation:
/// - Uses beta (systematic risk) instead of volatility
/// - Requires calculation of portfolio beta against a benchmark
/// - Provides insight into risk-adjusted return relative to market risk
fn calculate_treynor_ratio(
    return_value: f64,
    portfolio_beta: f64,
    risk_free_rate: f64
) -> Result<f64, CalculationError> {
    if portfolio_beta == 0.0 {
        return Err(CalculationError::InvalidInput("Beta cannot be zero for Treynor ratio calculation".to_string()));
    }
    
    // Calculate excess return
    let excess_return = return_value - risk_free_rate;
    
    // Calculate Treynor ratio
    Ok(excess_return / portfolio_beta.abs())
}

/// Calculate portfolio beta against a benchmark
/// 
/// This implementation:
/// - Computes covariance between portfolio and benchmark returns
/// - Divides by benchmark variance to get beta
/// - Represents systematic risk exposure
fn calculate_portfolio_beta(
    portfolio_returns: &[f64],
    benchmark_returns: &[f64]
) -> Result<f64, CalculationError> {
    if portfolio_returns.len() != benchmark_returns.len() || portfolio_returns.is_empty() {
        return Err(CalculationError::InsufficientData("Portfolio and benchmark return series must be the same length and non-empty".to_string()));
    }
    
    // Calculate means
    let portfolio_mean: f64 = portfolio_returns.iter().sum::<f64>() / portfolio_returns.len() as f64;
    let benchmark_mean: f64 = benchmark_returns.iter().sum::<f64>() / benchmark_returns.len() as f64;
    
    // Calculate covariance
    let mut covariance = 0.0;
    let mut benchmark_variance = 0.0;
    
    for i in 0..portfolio_returns.len() {
        let p_deviation = portfolio_returns[i] - portfolio_mean;
        let b_deviation = benchmark_returns[i] - benchmark_mean;
        
        covariance += p_deviation * b_deviation;
        benchmark_variance += b_deviation * b_deviation;
    }
    
    covariance /= portfolio_returns.len() as f64;
    benchmark_variance /= benchmark_returns.len() as f64;
    
    if benchmark_variance == 0.0 {
        return Err(CalculationError::InvalidInput("Benchmark variance is zero, cannot calculate beta".to_string()));
    }
    
    // Calculate beta
    Ok(covariance / benchmark_variance)
}

/// Calculate Tracking Error against a benchmark
/// 
/// This implementation:
/// - Computes the standard deviation of the difference between portfolio and benchmark returns
/// - Handles time-aligned return series
/// - Provides annualized tracking error
fn calculate_tracking_error(
    portfolio_returns: &[(DateTime<Utc>, f64)],
    benchmark_returns: &[(DateTime<Utc>, f64)],
    annualize: bool
) -> Result<f64, CalculationError> {
    if portfolio_returns.is_empty() || benchmark_returns.is_empty() {
        return Err(CalculationError::InsufficientData("Need both portfolio and benchmark returns for tracking error calculation".to_string()));
    }
    
    // Create a map of benchmark returns by date for easy lookup
    let benchmark_map: HashMap<DateTime<Utc>, f64> = benchmark_returns
        .iter()
        .map(|(date, ret)| (*date, *ret))
        .collect();
    
    // Calculate return differences for matching dates
    let mut return_differences: Vec<f64> = Vec::new();
    
    for (date, portfolio_return) in portfolio_returns {
        if let Some(benchmark_return) = benchmark_map.get(date) {
            let difference = portfolio_return - benchmark_return;
            return_differences.push(difference);
        }
    }
    
    if return_differences.len() < 2 {
        return Err(CalculationError::InsufficientData("Need at least two matching return periods for tracking error calculation".to_string()));
    }
    
    // Calculate standard deviation of differences
    let mean_difference: f64 = return_differences.iter().sum::<f64>() / return_differences.len() as f64;
    
    let variance: f64 = return_differences.iter()
        .map(|diff| (diff - mean_difference).powi(2))
        .sum::<f64>() / (return_differences.len() - 1) as f64;
    
    let tracking_error = variance.sqrt();
    
    // Annualize if requested
    if annualize {
        // Determine observation frequency
        let first_date = portfolio_returns.first().unwrap().0;
        let last_date = portfolio_returns.last().unwrap().0;
        let total_days = (last_date - first_date).num_days() as f64;
        let avg_days_between_observations = total_days / (portfolio_returns.len() - 1) as f64;
        
        // Calculate annualization factor
        let periods_per_year = 365.25 / avg_days_between_observations;
        Ok(tracking_error * periods_per_year.sqrt())
    } else {
        Ok(tracking_error)
    }
}

/// Calculate Information Ratio
/// 
/// This implementation:
/// - Computes the ratio of active return to tracking error
/// - Provides a measure of risk-adjusted active return
fn calculate_information_ratio(
    portfolio_return: f64,
    benchmark_return: f64,
    tracking_error: f64
) -> Result<f64, CalculationError> {
    if tracking_error <= 0.0 {
        return Err(CalculationError::InvalidInput("Tracking error must be positive for information ratio calculation".to_string()));
    }
    
    // Calculate active return
    let active_return = portfolio_return - benchmark_return;
    
    // Calculate information ratio
    Ok(active_return / tracking_error)
}

// Helper function to calculate benchmark return over a period
fn calculate_benchmark_return(
    benchmark_returns: &[dynamodb_repository::BenchmarkReturn],
    start_date: DateTime<Utc>,
    end_date: DateTime<Utc>
) -> Option<f64> {
    // Filter returns within the period
    let period_returns = benchmark_returns.iter()
        .filter(|br| br.date >= start_date && br.date <= end_date)
        .collect::<Vec<_>>();
    
    if period_returns.is_empty() {
        return None;
    }
    
    // Calculate cumulative return using geometric linking
    let cumulative_return = period_returns.iter()
        .fold(1.0, |acc, br| acc * (1.0 + br.return_value)) - 1.0;
    
    Some(cumulative_return)
}

// Helper function to convert valuations to return series
fn convert_valuations_to_returns(valuations: &[dynamodb_repository::Valuation]) -> Vec<(DateTime<Utc>, f64)> {
    if valuations.len() < 2 {
        return Vec::new();
    }
    
    // Sort valuations by date
    let mut sorted_valuations = valuations.to_vec();
    sorted_valuations.sort_by(|a, b| a.date.cmp(&b.date));
    
    // Calculate returns
    let mut returns = Vec::new();
    
    for window in sorted_valuations.windows(2) {
        let start_valuation = &window[0];
        let end_valuation = &window[1];
        
        // Simple return calculation (no cash flow adjustment here)
        let period_return = if start_valuation.value == 0.0 {
            0.0 // Avoid division by zero
        } else {
            (end_valuation.value - start_valuation.value) / start_valuation.value
        };
        
        returns.push((end_valuation.date, period_return));
    }
    
    returns
}

// Timestream repository for storing time-series performance data
mod timestream_repository {
    use super::*;
    use aws_sdk_timestreamwrite::{Client as TimestreamWriteClient, Error as TimestreamWriteError};
    use aws_sdk_timestreamquery::{Client as TimestreamQueryClient, Error as TimestreamQueryError};
    use aws_sdk_timestreamwrite::model::{Dimension, MeasureValue, Record, TimeUnit, WriteRecordsRequest};
    use async_trait::async_trait;
    use std::str::FromStr;
    
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
    pub struct PerformanceMetrics {
        pub timestamp: DateTime<Utc>,
        pub twr: f64,
        pub mwr: f64,
        pub volatility: Option<f64>,
        pub sharpe_ratio: Option<f64>,
        pub max_drawdown: Option<f64>,
        pub benchmark_return: Option<f64>,
        pub tracking_error: Option<f64>,
        pub information_ratio: Option<f64>,
    }
    
    #[derive(Debug, Clone, Copy)]
    pub enum TimeInterval {
        Daily,
        Weekly,
        Monthly,
        Quarterly,
        Yearly,
    }
    
    impl TimeInterval {
        fn to_sql_interval(&self) -> &'static str {
            match self {
                TimeInterval::Daily => "1d",
                TimeInterval::Weekly => "1w",
                TimeInterval::Monthly => "1mo",
                TimeInterval::Quarterly => "3mo",
                TimeInterval::Yearly => "1y",
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
        
        pub async fn from_env() -> Result<Self, Error> {
            let config = aws_config::load_from_env().await;
            let write_client = TimestreamWriteClient::new(&config);
            let query_client = TimestreamQueryClient::new(&config);
            
            let database_name = env::var("TIMESTREAM_DATABASE").expect("TIMESTREAM_DATABASE must be set");
            let table_name = env::var("TIMESTREAM_TABLE").expect("TIMESTREAM_TABLE must be set");
            
            Ok(Self::new(
                write_client,
                query_client,
                database_name,
                table_name,
            ))
        }
        
        pub async fn store_performance_data(&self, data_point: &PerformanceDataPoint) -> Result<(), Error> {
            // Create dimensions
            let mut dimensions = vec![
                Dimension::builder()
                    .name("portfolio_id")
                    .value(&data_point.portfolio_id)
                    .build(),
            ];
            
            // Add benchmark dimension if available
            if let Some(benchmark_id) = &data_point.benchmark_id {
                dimensions.push(
                    Dimension::builder()
                        .name("benchmark_id")
                        .value(benchmark_id)
                        .build(),
                );
            }
            
            // Create measures
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
            
            // Add optional measures
            if let Some(volatility) = data_point.volatility {
                measures.push(
                    Record::builder()
                        .dimensions(dimensions.clone())
                        .measure_name("volatility")
                        .measure_value(volatility.to_string())
                        .measure_value_type(MeasureValue::Double)
                        .time(data_point.timestamp.timestamp_millis().to_string())
                        .time_unit(TimeUnit::Milliseconds)
                        .build(),
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
                        .build(),
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
                        .build(),
                );
            }
            
            if let Some(benchmark_return) = data_point.benchmark_return {
                measures.push(
                    Record::builder()
                        .dimensions(dimensions.clone())
                        .measure_name("benchmark_return")
                        .measure_value(benchmark_return.to_string())
                        .measure_value_type(MeasureValue::Double)
                        .time(data_point.timestamp.timestamp_millis().to_string())
                        .time_unit(TimeUnit::Milliseconds)
                        .build(),
                );
            }
            
            if let Some(tracking_error) = data_point.tracking_error {
                measures.push(
                    Record::builder()
                        .dimensions(dimensions.clone())
                        .measure_name("tracking_error")
                        .measure_value(tracking_error.to_string())
                        .measure_value_type(MeasureValue::Double)
                        .time(data_point.timestamp.timestamp_millis().to_string())
                        .time_unit(TimeUnit::Milliseconds)
                        .build(),
                );
            }
            
            if let Some(information_ratio) = data_point.information_ratio {
                measures.push(
                    Record::builder()
                        .dimensions(dimensions.clone())
                        .measure_name("information_ratio")
                        .measure_value(information_ratio.to_string())
                        .measure_value_type(MeasureValue::Double)
                        .time(data_point.timestamp.timestamp_millis().to_string())
                        .time_unit(TimeUnit::Milliseconds)
                        .build(),
                );
            }
            
            // Create write records request
            let write_records_request = WriteRecordsRequest::builder()
                .database_name(&self.database_name)
                .table_name(&self.table_name)
                .records(measures)
                .build();
            
            // Write records to Timestream
            self.write_client
                .write_records()
                .set_database_name(Some(self.database_name.clone()))
                .set_table_name(Some(self.table_name.clone()))
                .set_records(write_records_request.records)
                .send()
                .await
                .map_err(|e| Error::from(e))?;
            
            Ok(())
        }
    }
    
    #[async_trait]
    impl super::TimeseriesRepository for TimestreamRepository {
        async fn save_performance_metrics(
            &self, 
            portfolio_id: &str, 
            metrics: &PerformanceMetrics,
            timestamp: DateTime<Utc>
        ) -> Result<(), RepositoryError> {
            // Convert PerformanceMetrics to PerformanceDataPoint
            let data_point = PerformanceDataPoint {
                portfolio_id: portfolio_id.to_string(),
                timestamp,
                twr: metrics.twr,
                mwr: metrics.mwr,
                volatility: metrics.volatility,
                sharpe_ratio: metrics.sharpe_ratio,
                max_drawdown: metrics.max_drawdown,
                benchmark_id: None, // Not included in metrics
                benchmark_return: metrics.benchmark_return,
                tracking_error: metrics.tracking_error,
                information_ratio: metrics.information_ratio,
            };
            
            // Store the data point
            self.store_performance_data(&data_point)
                .await
                .map_err(|e| RepositoryError::Other(e.to_string()))
        }
        
        async fn get_performance_history(
            &self,
            portfolio_id: &str,
            start_date: DateTime<Utc>,
            end_date: DateTime<Utc>,
            interval: TimeInterval
        ) -> Result<Vec<PerformanceMetrics>, RepositoryError> {
            // Build the query
            let query = format!(
                r#"
                SELECT 
                    BIN(time, {}) AS binned_time,
                    portfolio_id,
                    ROUND(AVG(CASE WHEN measure_name = 'twr' THEN measure_value::double ELSE NULL END), 6) AS twr,
                    ROUND(AVG(CASE WHEN measure_name = 'mwr' THEN measure_value::double ELSE NULL END), 6) AS mwr,
                    ROUND(AVG(CASE WHEN measure_name = 'volatility' THEN measure_value::double ELSE NULL END), 6) AS volatility,
                    ROUND(AVG(CASE WHEN measure_name = 'sharpe_ratio' THEN measure_value::double ELSE NULL END), 6) AS sharpe_ratio,
                    ROUND(AVG(CASE WHEN measure_name = 'max_drawdown' THEN measure_value::double ELSE NULL END), 6) AS max_drawdown,
                    ROUND(AVG(CASE WHEN measure_name = 'benchmark_return' THEN measure_value::double ELSE NULL END), 6) AS benchmark_return,
                    ROUND(AVG(CASE WHEN measure_name = 'tracking_error' THEN measure_value::double ELSE NULL END), 6) AS tracking_error,
                    ROUND(AVG(CASE WHEN measure_name = 'information_ratio' THEN measure_value::double ELSE NULL END), 6) AS information_ratio
                FROM "{}"."{}"."{}"
                WHERE 
                    portfolio_id = '{}' AND
                    time BETWEEN from_iso8601_timestamp('{}') AND from_iso8601_timestamp('{}')
                GROUP BY 
                    BIN(time, {}), portfolio_id
                ORDER BY 
                    binned_time ASC
                "#,
                interval.to_sql_interval(),
                self.database_name,
                self.table_name,
                portfolio_id,
                start_date.to_rfc3339(),
                end_date.to_rfc3339(),
                interval.to_sql_interval()
            );
            
            // Execute the query
            let query_result = self.query_client
                .query()
                .query_string(query)
                .send()
                .await
                .map_err(|e| RepositoryError::Other(e.to_string()))?;
            
            // Process the results
            let mut metrics = Vec::new();
            
            if let Some(result_rows) = query_result.rows {
                for row in result_rows {
                    if let Some(data) = row.data {
                        // Extract timestamp
                        let timestamp_str = data[0].scalar_value.as_ref()
                            .ok_or_else(|| RepositoryError::InvalidData("Missing timestamp".to_string()))?;
                        let timestamp = DateTime::parse_from_rfc3339(timestamp_str)
                            .map_err(|e| RepositoryError::InvalidData(format!("Invalid timestamp: {}", e)))?
                            .with_timezone(&Utc);
                        
                        // Extract metrics
                        let twr = data[2].scalar_value.as_ref()
                            .and_then(|v| f64::from_str(v).ok())
                            .unwrap_or(0.0);
                        
                        let mwr = data[3].scalar_value.as_ref()
                            .and_then(|v| f64::from_str(v).ok())
                            .unwrap_or(0.0);
                        
                        let volatility = data[4].scalar_value.as_ref()
                            .and_then(|v| f64::from_str(v).ok());
                        
                        let sharpe_ratio = data[5].scalar_value.as_ref()
                            .and_then(|v| f64::from_str(v).ok());
                        
                        let max_drawdown = data[6].scalar_value.as_ref()
                            .and_then(|v| f64::from_str(v).ok());
                        
                        let benchmark_return = data[7].scalar_value.as_ref()
                            .and_then(|v| f64::from_str(v).ok());
                        
                        let tracking_error = data[8].scalar_value.as_ref()
                            .and_then(|v| f64::from_str(v).ok());
                        
                        let information_ratio = data[9].scalar_value.as_ref()
                            .and_then(|v| f64::from_str(v).ok());
                        
                        // Create PerformanceMetrics
                        let metric = PerformanceMetrics {
                            timestamp,
                            twr,
                            mwr,
                            volatility,
                            sharpe_ratio,
                            max_drawdown,
                            benchmark_return,
                            tracking_error,
                            information_ratio,
                        };
                        
                        metrics.push(metric);
                    }
                }
            }
            
            Ok(metrics)
        }
    }
}

#[async_trait]
pub trait TimeseriesRepository {
    async fn save_performance_metrics(
        &self, 
        portfolio_id: &str, 
        metrics: &timestream_repository::PerformanceMetrics,
        timestamp: DateTime<Utc>
    ) -> Result<(), RepositoryError>;
    
    async fn get_performance_history(
        &self,
        portfolio_id: &str,
        start_date: DateTime<Utc>,
        end_date: DateTime<Utc>,
        interval: timestream_repository::TimeInterval
    ) -> Result<Vec<timestream_repository::PerformanceMetrics>, RepositoryError>;
}

#[cfg(test)]
mod tests;

#[cfg(test)]
mod repository_tests;

#[cfg(test)]
mod timestream_tests;

#[cfg(test)]
mod integration_tests;

#[cfg(test)]
mod resilience_tests;

#[cfg(test)]
mod property_tests;

mod tenant;

use tenant::{TenantAware, TenantContext, TenantManager, TenantMiddleware};

// Update the DynamoDbRepository to implement TenantAware
impl TenantAware for DynamoDbRepository {
    fn tenant_context(&self) -> &TenantContext {
        &self.tenant_context
    }
    
    fn with_tenant(mut self, tenant_context: TenantContext) -> Self {
        self.tenant_context = tenant_context;
        self
    }
}

// Update the DynamoDbRepository struct to include tenant_context
#[derive(Debug, Clone)]
pub struct DynamoDbRepository {
    client: aws_sdk_dynamodb::Client,
    table_name: String,
    tenant_context: TenantContext,
    // ... existing fields ...
}

// Update the DynamoDbRepository::new method to accept a tenant_context
impl DynamoDbRepository {
    pub fn new(
        client: aws_sdk_dynamodb::Client,
        table_name: impl Into<String>,
        tenant_context: TenantContext,
    ) -> Self {
        Self {
            client,
            table_name: table_name.into(),
            tenant_context,
            // ... existing fields initialization ...
        }
    }
    
    // ... existing methods ...
    
    // Update the get_item method to include tenant filtering
    async fn get_item(
        &self,
        key: HashMap<String, AttributeValue>,
    ) -> Result<Option<HashMap<String, AttributeValue>>, RepositoryError> {
        // Add tenant ID to the key
        let mut tenant_key = key.clone();
        self.add_tenant_id_to_item(&mut tenant_key);
        
        let result = self.client
            .get_item()
            .table_name(&self.table_name)
            .set_key(Some(tenant_key))
            .send()
            .await
            .map_err(|e| RepositoryError::DynamoDb(e.to_string()))?;
        
        if let Some(item) = result.item {
            // Validate tenant ownership
            self.validate_tenant_ownership(&item)
                .map_err(|e| RepositoryError::AccessDenied(e.to_string()))?;
            
            Ok(Some(item))
        } else {
            Ok(None)
        }
    }
    
    // Update the query method to include tenant filtering
    async fn query(
        &self,
        key_condition_expression: String,
        expression_attribute_values: HashMap<String, AttributeValue>,
        index_name: Option<String>,
    ) -> Result<Vec<HashMap<String, AttributeValue>>, RepositoryError> {
        // Add tenant ID to the condition expression
        let tenant_condition = self.tenant_condition_expression();
        let key_condition = format!("{} AND {}", key_condition_expression, tenant_condition);
        
        // Add tenant ID to the expression attribute values
        let mut tenant_values = expression_attribute_values.clone();
        let tenant_attr_values = self.tenant_expression_attribute_values();
        tenant_values.extend(tenant_attr_values);
        
        let mut query = self.client
            .query()
            .table_name(&self.table_name)
            .key_condition_expression(key_condition)
            .set_expression_attribute_values(Some(tenant_values));
        
        if let Some(idx_name) = index_name {
            query = query.index_name(idx_name);
        }
        
        let result = query
            .send()
            .await
            .map_err(|e| RepositoryError::DynamoDb(e.to_string()))?;
        
        if let Some(items) = result.items {
            // Validate tenant ownership for each item
            for item in &items {
                self.validate_tenant_ownership(item)
                    .map_err(|e| RepositoryError::AccessDenied(e.to_string()))?;
            }
            
            Ok(items)
        } else {
            Ok(Vec::new())
        }
    }
    
    // Update the put_item method to include tenant ID
    async fn put_item(
        &self,
        mut item: HashMap<String, AttributeValue>,
    ) -> Result<(), RepositoryError> {
        // Add tenant ID to the item
        self.add_tenant_id_to_item(&mut item);
        
        self.client
            .put_item()
            .table_name(&self.table_name)
            .set_item(Some(item))
            .send()
            .await
            .map_err(|e| RepositoryError::DynamoDb(e.to_string()))?;
        
        Ok(())
    }
    
    // Update the delete_item method to include tenant filtering
    async fn delete_item(
        &self,
        key: HashMap<String, AttributeValue>,
    ) -> Result<(), RepositoryError> {
        // Add tenant ID to the key
        let mut tenant_key = key.clone();
        self.add_tenant_id_to_item(&mut tenant_key);
        
        // Add condition expression to ensure tenant ownership
        let condition_expression = self.tenant_condition_expression();
        let expression_attribute_values = self.tenant_expression_attribute_values();
        
        self.client
            .delete_item()
            .table_name(&self.table_name)
            .set_key(Some(tenant_key))
            .condition_expression(condition_expression)
            .set_expression_attribute_values(Some(expression_attribute_values))
            .send()
            .await
            .map_err(|e| RepositoryError::DynamoDb(e.to_string()))?;
        
        Ok(())
    }
    
    // ... existing methods ...
}

// Update the function_handler to include tenant context
async fn function_handler(event: LambdaEvent<SQSEvent>) -> Result<(), Error> {
    // ... existing code ...
    
    // Extract tenant ID from the event or use a default
    let tenant_id = event.payload.records.first()
        .and_then(|record| record.message_attributes.get("tenant_id"))
        .and_then(|attr| attr.string_value.as_ref())
        .unwrap_or("default").to_string();
    
    let tenant_context = TenantContext::new(tenant_id);
    
    // Create the DynamoDB repository with tenant context
    let dynamodb_repository = DynamoDbRepository::new(
        dynamodb_client.clone(),
        table_name.clone(),
        tenant_context.clone(),
    );
    
    // Create the Timestream repository with tenant context
    let timestream_repository = TimestreamRepository::new(
        timestream_write_client.clone(),
        timestream_query_client.clone(),
        database_name.clone(),
        table_name.clone(),
        tenant_context,
    );
    
    // ... existing code ...
}

// Update the TimestreamRepository to include tenant context
#[derive(Debug, Clone)]
pub struct TimestreamRepository {
    write_client: aws_sdk_timestreamwrite::Client,
    query_client: aws_sdk_timestreamquery::Client,
    database_name: String,
    table_name: String,
    tenant_context: TenantContext,
}

impl TimestreamRepository {
    pub fn new(
        write_client: aws_sdk_timestreamwrite::Client,
        query_client: aws_sdk_timestreamquery::Client,
        database_name: impl Into<String>,
        table_name: impl Into<String>,
        tenant_context: TenantContext,
    ) -> Self {
        Self {
            write_client,
            query_client,
            database_name: database_name.into(),
            table_name: table_name.into(),
            tenant_context,
        }
    }
    
    // ... existing methods ...
    
    // Update store_performance_data to include tenant ID
    pub async fn store_performance_data(
        &self,
        data_point: PerformanceDataPoint,
    ) -> Result<(), RepositoryError> {
        // ... existing code ...
        
        // Add tenant ID as a dimension
        dimensions.push(
            Dimension::builder()
                .name("tenant_id")
                .value(self.tenant_context.tenant_id.clone())
                .build(),
        );
        
        // ... existing code ...
    }
    
    // ... existing methods ...
}

impl TenantAware for TimestreamRepository {
    fn tenant_context(&self) -> &TenantContext {
        &self.tenant_context
    }
    
    fn with_tenant(mut self, tenant_context: TenantContext) -> Self {
        self.tenant_context = tenant_context;
        self
    }
}

// ... existing code ... 

#[cfg(test)]
mod tenant_tests;

mod tenant_example;
mod transaction_example;

#[cfg(test)]
mod transaction_tests;