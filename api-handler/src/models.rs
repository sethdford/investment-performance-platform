//! API models

use serde::{Deserialize, Serialize};
use chrono::{DateTime, NaiveDate, Utc};
use std::collections::HashMap;

/// API response wrapper
#[derive(Debug, Serialize, Deserialize)]
pub struct ApiResponse<T> {
    /// Response data
    pub data: T,
    /// Request ID for tracing
    pub request_id: String,
}

/// Error response
#[derive(Debug, Serialize, Deserialize)]
pub struct ErrorResponse {
    /// Error message
    pub message: String,
    /// Request ID for tracing
    pub request_id: String,
}

/// Portfolio creation request
#[derive(Debug, Serialize, Deserialize)]
pub struct CreatePortfolioRequest {
    /// Portfolio name
    pub name: String,
    /// Client ID
    pub client_id: String,
    /// Portfolio inception date
    pub inception_date: NaiveDate,
    /// Portfolio benchmark ID
    pub benchmark_id: Option<String>,
    /// Additional portfolio metadata
    #[serde(default)]
    pub metadata: HashMap<String, String>,
}

/// Portfolio update request
#[derive(Debug, Serialize, Deserialize)]
pub struct UpdatePortfolioRequest {
    /// Portfolio name
    pub name: String,
    /// Portfolio benchmark ID
    pub benchmark_id: Option<String>,
    /// Portfolio status
    pub status: String,
    /// Additional portfolio metadata
    #[serde(default)]
    pub metadata: HashMap<String, String>,
}

/// Transaction creation request
#[derive(Debug, Serialize, Deserialize)]
pub struct CreateTransactionRequest {
    /// Account ID
    pub account_id: String,
    /// Security ID
    pub security_id: Option<String>,
    /// Transaction date
    pub transaction_date: NaiveDate,
    /// Settlement date
    pub settlement_date: Option<NaiveDate>,
    /// Transaction type
    pub transaction_type: String,
    /// Transaction amount
    pub amount: f64,
    /// Transaction quantity
    pub quantity: Option<f64>,
    /// Transaction price
    pub price: Option<f64>,
    /// Transaction fees
    pub fees: Option<f64>,
    /// Transaction currency
    pub currency: String,
    /// Additional transaction metadata
    #[serde(default)]
    pub metadata: HashMap<String, String>,
}

/// Performance calculation request
#[derive(Debug, Serialize, Deserialize)]
pub struct CalculatePerformanceRequest {
    /// Portfolio ID
    pub portfolio_id: String,
    /// Start date
    pub start_date: NaiveDate,
    /// End date
    pub end_date: NaiveDate,
    /// Benchmark ID
    pub benchmark_id: Option<String>,
    /// Include details flag
    #[serde(default)]
    pub include_details: bool,
}

/// Performance calculation response
#[derive(Debug, Serialize, Deserialize)]
pub struct CalculatePerformanceResponse {
    /// Portfolio ID
    pub portfolio_id: String,
    /// Start date
    pub start_date: NaiveDate,
    /// End date
    pub end_date: NaiveDate,
    /// Time-weighted return
    pub twr: f64,
    /// Money-weighted return
    pub mwr: f64,
    /// Volatility
    pub volatility: Option<f64>,
    /// Sharpe ratio
    pub sharpe_ratio: Option<f64>,
    /// Maximum drawdown
    pub max_drawdown: Option<f64>,
    /// Benchmark return
    pub benchmark_return: Option<f64>,
    /// Benchmark ID
    pub benchmark_id: Option<String>,
    /// Calculation timestamp
    pub calculated_at: DateTime<Utc>,
    /// Performance details (if requested)
    pub details: Option<PerformanceDetails>,
}

/// Performance details
#[derive(Debug, Serialize, Deserialize)]
pub struct PerformanceDetails {
    /// Daily returns
    pub daily_returns: Vec<DailyReturn>,
    /// Monthly returns
    pub monthly_returns: Vec<MonthlyReturn>,
    /// Quarterly returns
    pub quarterly_returns: Vec<QuarterlyReturn>,
    /// Annual returns
    pub annual_returns: Vec<AnnualReturn>,
}

/// Daily return
#[derive(Debug, Serialize, Deserialize)]
pub struct DailyReturn {
    /// Date
    pub date: NaiveDate,
    /// Return
    pub return_value: f64,
    /// Benchmark return
    pub benchmark_return: Option<f64>,
}

/// Monthly return
#[derive(Debug, Serialize, Deserialize)]
pub struct MonthlyReturn {
    /// Year
    pub year: i32,
    /// Month
    pub month: u32,
    /// Return
    pub return_value: f64,
    /// Benchmark return
    pub benchmark_return: Option<f64>,
}

/// Quarterly return
#[derive(Debug, Serialize, Deserialize)]
pub struct QuarterlyReturn {
    /// Year
    pub year: i32,
    /// Quarter
    pub quarter: u32,
    /// Return
    pub return_value: f64,
    /// Benchmark return
    pub benchmark_return: Option<f64>,
}

/// Annual return
#[derive(Debug, Serialize, Deserialize)]
pub struct AnnualReturn {
    /// Year
    pub year: i32,
    /// Return
    pub return_value: f64,
    /// Benchmark return
    pub benchmark_return: Option<f64>,
} 