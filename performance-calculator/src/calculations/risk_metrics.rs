use rust_decimal::Decimal;
use rust_decimal::prelude::*;
use rust_decimal_macros::dec;
use anyhow::{Result, anyhow};
use std::collections::HashMap;
use chrono::{DateTime, Utc, NaiveDate};
use serde::{Serialize, Deserialize};
use std::cmp::Ordering;

/// Risk-free rate used for Sharpe and Sortino ratio calculations
/// This should ideally be configurable or retrieved from a data source
const RISK_FREE_RATE: f64 = 0.02; // 2% annual risk-free rate

/// Helper functions for Decimal math operations
fn sqrt(value: Decimal) -> Decimal {
    // Convert to f64, take sqrt, convert back to Decimal
    let f64_val = value.to_f64().unwrap_or(0.0);
    let sqrt_val = f64_val.sqrt();
    Decimal::from_f64_retain(sqrt_val).unwrap_or(Decimal::ZERO)
}

fn powu(value: Decimal, exp: u32) -> Decimal {
    // For squaring, just multiply the value by itself
    if exp == 2 {
        return value * value;
    }
    
    // For other powers, convert to f64, use powf, convert back
    let f64_val = value.to_f64().unwrap_or(0.0);
    let pow_val = f64_val.powi(exp as i32);
    Decimal::from_f64_retain(pow_val).unwrap_or(Decimal::ZERO)
}

/// Represents a time series of returns
#[derive(Debug, Clone)]
pub struct ReturnSeries {
    /// Dates of the returns
    pub dates: Vec<NaiveDate>,
    /// Return values
    pub values: Vec<Decimal>,
}

impl ReturnSeries {
    /// Create a new ReturnSeries
    pub fn new(dates: Vec<NaiveDate>, values: Vec<Decimal>) -> Self {
        Self { dates, values }
    }
    
    /// Get the return values as a slice
    pub fn values(&self) -> &[Decimal] {
        &self.values
    }
    
    /// Convert the return values to f64
    pub fn to_f64_values(&self) -> Vec<f64> {
        self.values
            .iter()
            .map(|v| v.to_f64().unwrap_or(0.0))
            .collect()
    }
}

/// Risk metrics for a portfolio or account
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RiskMetrics {
    /// Volatility (standard deviation of returns)
    pub volatility: Decimal,
    /// Sharpe ratio (excess return per unit of risk)
    pub sharpe_ratio: Decimal,
    /// Sortino ratio (excess return per unit of downside risk)
    pub sortino_ratio: Decimal,
    /// Maximum drawdown (largest peak-to-trough decline)
    pub max_drawdown: Decimal,
    /// Value at Risk (VaR) at 95% confidence level
    pub var_95: Decimal,
    /// Conditional Value at Risk (CVaR) at 95% confidence level
    pub cvar_95: Decimal,
    /// Beta (measure of systematic risk)
    pub beta: Option<Decimal>,
    /// Alpha (excess return over benchmark)
    pub alpha: Option<Decimal>,
    /// Tracking error (standard deviation of return differences)
    pub tracking_error: Option<Decimal>,
    /// Information ratio (excess return per unit of tracking error)
    pub information_ratio: Option<Decimal>,
    /// Treynor ratio (excess return per unit of systematic risk)
    pub treynor_ratio: Option<Decimal>,
}

impl Default for RiskMetrics {
    fn default() -> Self {
        Self {
            volatility: Decimal::ZERO,
            sharpe_ratio: Decimal::ZERO,
            sortino_ratio: Decimal::ZERO,
            max_drawdown: Decimal::ZERO,
            var_95: Decimal::ZERO,
            cvar_95: Decimal::ZERO,
            beta: None,
            alpha: None,
            tracking_error: None,
            information_ratio: None,
            treynor_ratio: None,
        }
    }
}

/// Calculate volatility (standard deviation of returns)
pub fn calculate_volatility(returns: &[Decimal]) -> Decimal {
    if returns.is_empty() {
        return Decimal::ZERO;
    }
    
    // Calculate mean
    let sum: Decimal = returns.iter().sum();
    let mean = sum / Decimal::from(returns.len());
    
    // Calculate sum of squared differences
    let sum_squared_diff: Decimal = returns
        .iter()
        .map(|r| {
            let diff = *r - mean;
            powu(diff, 2)  // Use our powu helper function
        })
        .sum();
    
    // Calculate variance
    let variance = sum_squared_diff / Decimal::from(returns.len());
    
    // Calculate standard deviation (square root of variance)
    sqrt(variance)  // Use our sqrt helper function
}

/// Calculate Sharpe ratio (excess return per unit of risk)
pub fn calculate_sharpe_ratio(
    annualized_return: Decimal,
    volatility: Decimal,
    risk_free_rate: Option<Decimal>
) -> Decimal {
    let rf = risk_free_rate.unwrap_or(Decimal::from_f64_retain(RISK_FREE_RATE).unwrap_or(Decimal::ZERO));
    
    if volatility == Decimal::ZERO {
        return Decimal::ZERO;
    }
    
    (annualized_return - rf) / volatility
}

/// Calculate Sortino ratio (excess return per unit of downside risk)
pub fn calculate_sortino_ratio(
    annualized_return: Decimal,
    returns: &[Decimal],
    risk_free_rate: Option<Decimal>
) -> Decimal {
    let rf = risk_free_rate.unwrap_or(Decimal::from_f64_retain(RISK_FREE_RATE).unwrap_or(Decimal::ZERO));
    
    // Calculate downside returns (returns below risk-free rate)
    let mut downside_returns: Vec<Decimal> = returns
        .iter()
        .filter_map(|r| {
            if *r < rf {
                Some(powu(*r - rf, 2))  // Use our powu helper function
            } else {
                None
            }
        })
        .collect();
    
    // If no downside returns, return zero
    if downside_returns.is_empty() {
        return Decimal::ZERO;
    }
    
    let downside_deviation = sqrt(downside_returns.iter().sum::<Decimal>() / Decimal::from(downside_returns.len()));  // Use our sqrt helper function
    
    if downside_deviation == Decimal::ZERO {
        return Decimal::ZERO;
    }
    
    (annualized_return - rf) / downside_deviation
}

/// Calculate maximum drawdown (largest peak-to-trough decline)
pub fn calculate_max_drawdown(cumulative_returns: &[Decimal]) -> Decimal {
    if cumulative_returns.is_empty() {
        return Decimal::ZERO;
    }
    
    let mut max_drawdown = Decimal::ZERO;
    let mut peak = cumulative_returns[0];
    
    for &return_value in cumulative_returns {
        if return_value > peak {
            peak = return_value;
        } else {
            let drawdown = (peak - return_value) / peak;
            if drawdown > max_drawdown {
                max_drawdown = drawdown;
            }
        }
    }
    
    max_drawdown
}

/// Calculate Value at Risk (VaR) at a given confidence level
pub fn calculate_var(returns: &[Decimal], confidence_level: f64) -> Decimal {
    if returns.is_empty() {
        return Decimal::ZERO;
    }
    
    // Sort returns in ascending order
    let mut sorted_returns = returns.to_vec();
    sorted_returns.sort();
    
    // Find the index corresponding to the confidence level
    let index = ((1.0 - confidence_level) * returns.len() as f64).floor() as usize;
    
    // Return the value at that index (negative of the loss)
    sorted_returns.get(index).cloned().unwrap_or(Decimal::ZERO)
}

/// Calculate Conditional Value at Risk (CVaR) at a given confidence level
pub fn calculate_cvar(returns: &[Decimal], confidence_level: f64) -> Decimal {
    if returns.is_empty() {
        return Decimal::ZERO;
    }
    
    // Sort returns in ascending order
    let mut sorted_returns = returns.to_vec();
    sorted_returns.sort();
    
    // Find the index corresponding to the confidence level
    let index = ((1.0 - confidence_level) * returns.len() as f64).floor() as usize;
    
    // Calculate the average of returns below the VaR
    let tail_returns = &sorted_returns[0..=index];
    
    if tail_returns.is_empty() {
        return Decimal::ZERO;
    }
    
    let sum: Decimal = tail_returns.iter().sum();
    sum / Decimal::from(tail_returns.len())
}

/// Calculate beta (measure of systematic risk)
pub fn calculate_beta(returns: &[Decimal], benchmark_returns: &[Decimal]) -> Option<Decimal> {
    if returns.len() != benchmark_returns.len() || returns.is_empty() {
        return None;
    }
    
    // Calculate covariance
    let portfolio_mean: Decimal = returns.iter().sum::<Decimal>() / Decimal::from(returns.len());
    let benchmark_mean: Decimal = benchmark_returns.iter().sum::<Decimal>() / Decimal::from(benchmark_returns.len());
    
    let mut covariance = Decimal::ZERO;
    for i in 0..returns.len() {
        covariance += (returns[i] - portfolio_mean) * (benchmark_returns[i] - benchmark_mean);
    }
    covariance = covariance / Decimal::from(returns.len());
    
    // Calculate benchmark variance
    let benchmark_variance: Decimal = benchmark_returns
        .iter()
        .map(|r| powu(*r - benchmark_mean, 2))  // Use our powu helper function
        .sum::<Decimal>() / Decimal::from(benchmark_returns.len());
    
    if benchmark_variance == Decimal::ZERO {
        return None;
    }
    
    Some(covariance / benchmark_variance)
}

/// Calculate alpha (excess return over benchmark)
pub fn calculate_alpha(
    annualized_return: Decimal,
    beta: Decimal,
    annualized_benchmark_return: Decimal,
    risk_free_rate: Option<Decimal>
) -> Decimal {
    let rf = risk_free_rate.unwrap_or(Decimal::from_f64_retain(RISK_FREE_RATE).unwrap_or(Decimal::ZERO));
    
    annualized_return - (rf + beta * (annualized_benchmark_return - rf))
}

/// Calculate tracking error (standard deviation of return differences)
pub fn calculate_tracking_error(portfolio_returns: &[Decimal], benchmark_returns: &[Decimal]) -> Decimal {
    if portfolio_returns.len() != benchmark_returns.len() || portfolio_returns.is_empty() {
        return Decimal::ZERO;
    }
    
    // Calculate return differences
    let mut return_diffs = Vec::with_capacity(portfolio_returns.len());
    for i in 0..portfolio_returns.len() {
        return_diffs.push(portfolio_returns[i] - benchmark_returns[i]);
    }
    
    // Calculate mean of return differences
    let sum: Decimal = return_diffs.iter().sum();
    let mean = sum / Decimal::from(return_diffs.len());
    
    // Calculate sum of squared differences
    let sum_squared_diff: Decimal = return_diffs
        .iter()
        .map(|r| {
            // Use multiplication instead of powu
            let diff = *r - mean;
            powu(diff, 2)  // Use our powu helper function
        })
        .sum();
    
    // Calculate variance
    let variance = sum_squared_diff / Decimal::from(return_diffs.len());
    
    // Calculate standard deviation (square root of variance)
    sqrt(variance)  // Use our sqrt helper function
}

/// Calculate information ratio (excess return per unit of tracking error)
pub fn calculate_information_ratio(
    annualized_return: Decimal,
    annualized_benchmark_return: Decimal,
    tracking_error: Decimal
) -> Option<Decimal> {
    if tracking_error == Decimal::ZERO {
        return None;
    }
    
    Some((annualized_return - annualized_benchmark_return) / tracking_error)
}

/// Calculate Treynor ratio (excess return per unit of systematic risk)
pub fn calculate_treynor_ratio(
    annualized_return: Decimal,
    beta: Decimal,
    risk_free_rate: Option<Decimal>
) -> Option<Decimal> {
    let rf = risk_free_rate.unwrap_or(Decimal::from_f64_retain(RISK_FREE_RATE).unwrap_or(Decimal::ZERO));
    
    if beta == Decimal::ZERO {
        return None;
    }
    
    Some((annualized_return - rf) / beta)
}

/// Calculate all risk metrics for a return series
pub fn calculate_risk_metrics(
    return_series: &ReturnSeries,
    annualized_return: Decimal,
    benchmark_return_series: Option<&ReturnSeries>,
    annualized_benchmark_return: Option<Decimal>,
    risk_free_rate: Option<Decimal>
) -> RiskMetrics {
    let mut metrics = RiskMetrics::default();
    
    // Get the return values from the series
    let returns = &return_series.values;
    
    // Calculate volatility
    metrics.volatility = calculate_volatility(returns);
    
    // Calculate Sharpe ratio
    metrics.sharpe_ratio = calculate_sharpe_ratio(annualized_return, metrics.volatility, risk_free_rate);
    
    // Calculate Sortino ratio
    metrics.sortino_ratio = calculate_sortino_ratio(annualized_return, returns, risk_free_rate);
    
    // Calculate maximum drawdown
    // First, convert returns to cumulative returns
    let mut cumulative_returns = Vec::with_capacity(returns.len());
    let mut cumulative = Decimal::ONE;
    for &r in returns {
        cumulative = cumulative * (Decimal::ONE + r);
        cumulative_returns.push(cumulative);
    }
    metrics.max_drawdown = calculate_max_drawdown(&cumulative_returns);
    
    // Calculate VaR and CVaR at 95% confidence level
    metrics.var_95 = calculate_var(returns, 0.95);
    metrics.cvar_95 = calculate_cvar(returns, 0.95);
    
    // Calculate benchmark-related metrics if benchmark data is available
    if let (Some(bench_return_series), Some(bench_annual_return)) = (benchmark_return_series, annualized_benchmark_return) {
        let bench_returns = &bench_return_series.values;
        
        // Calculate beta
        if let Some(beta) = calculate_beta(returns, bench_returns) {
            metrics.beta = Some(beta);
            
            // Calculate alpha
            metrics.alpha = Some(calculate_alpha(
                annualized_return,
                beta,
                bench_annual_return,
                risk_free_rate
            ));
            
            // Calculate Treynor ratio
            metrics.treynor_ratio = calculate_treynor_ratio(
                annualized_return,
                beta,
                risk_free_rate
            );
        }
        
        // Calculate tracking error
        metrics.tracking_error = Some(calculate_tracking_error(returns, bench_returns));
        
        // Calculate information ratio
        if let Some(tracking_error) = metrics.tracking_error {
            metrics.information_ratio = calculate_information_ratio(
                annualized_return,
                bench_annual_return,
                tracking_error
            );
        }
    }
    
    metrics
}

/// Helper function to create a ReturnSeries from f64 values
pub fn create_return_series_from_f64(dates: Vec<NaiveDate>, values: Vec<f64>) -> ReturnSeries {
    let decimal_values = values
        .into_iter()
        .map(|v| Decimal::from_f64_retain(v).unwrap_or(Decimal::ZERO))
        .collect();
    
    ReturnSeries {
        dates,
        values: decimal_values,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_calculate_volatility() {
        let returns = vec![
            Decimal::from_f64_retain(0.01).unwrap(),
            Decimal::from_f64_retain(-0.02).unwrap(),
            Decimal::from_f64_retain(0.03).unwrap(),
            Decimal::from_f64_retain(0.01).unwrap(),
            Decimal::from_f64_retain(-0.01).unwrap(),
        ];
        
        let volatility = calculate_volatility(&returns);
        assert!(volatility > Decimal::ZERO);
    }
    
    #[test]
    fn test_calculate_sharpe_ratio() {
        let annualized_return = Decimal::from_f64_retain(0.10).unwrap(); // 10%
        let volatility = Decimal::from_f64_retain(0.15).unwrap(); // 15%
        let risk_free_rate = Decimal::from_f64_retain(0.02).unwrap(); // 2%
        
        let sharpe = calculate_sharpe_ratio(annualized_return, volatility, Some(risk_free_rate));
        
        // Expected: (0.10 - 0.02) / 0.15 = 0.533...
        let expected = Decimal::from_f64_retain(0.533333).unwrap();
        assert!((sharpe - expected).abs() < Decimal::from_f64_retain(0.001).unwrap());
    }
} 