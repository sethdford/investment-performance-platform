use rust_decimal::Decimal;
use rust_decimal::prelude::*;
use anyhow::{Result, anyhow};
use std::collections::HashMap;
use chrono::{DateTime, Utc, NaiveDate};
use serde::{Serialize, Deserialize};

/// Risk-free rate used for Sharpe and Sortino ratio calculations
/// This should ideally be configurable or retrieved from a data source
const RISK_FREE_RATE: f64 = 0.02; // 2% annual risk-free rate

/// Represents a time series of returns
#[derive(Debug, Clone)]
pub struct ReturnSeries {
    /// Dates of the returns
    pub dates: Vec<NaiveDate>,
    /// Return values
    pub values: Vec<Decimal>,
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
        .map(|r| (*r - mean).powu(2))
        .sum();
    
    // Calculate variance
    let variance = sum_squared_diff / Decimal::from(returns.len());
    
    // Calculate standard deviation (square root of variance)
    variance.sqrt().unwrap_or(Decimal::ZERO)
}

/// Calculate Sharpe ratio (excess return per unit of risk)
pub fn calculate_sharpe_ratio(
    annualized_return: Decimal,
    volatility: Decimal,
    risk_free_rate: Option<Decimal>
) -> Decimal {
    let rf = risk_free_rate.unwrap_or(Decimal::from_f64(RISK_FREE_RATE).unwrap_or(Decimal::ZERO));
    
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
    let rf = risk_free_rate.unwrap_or(Decimal::from_f64(RISK_FREE_RATE).unwrap_or(Decimal::ZERO));
    
    // Calculate downside deviation
    let downside_returns: Vec<Decimal> = returns
        .iter()
        .filter_map(|r| {
            if *r < rf {
                Some((*r - rf).powu(2))
            } else {
                None
            }
        })
        .collect();
    
    if downside_returns.is_empty() {
        return Decimal::ZERO;
    }
    
    let downside_deviation = (downside_returns.iter().sum::<Decimal>() / Decimal::from(downside_returns.len()))
        .sqrt()
        .unwrap_or(Decimal::ZERO);
    
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
        .map(|r| (*r - benchmark_mean).powu(2))
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
    let rf = risk_free_rate.unwrap_or(Decimal::from_f64(RISK_FREE_RATE).unwrap_or(Decimal::ZERO));
    
    annualized_return - (rf + beta * (annualized_benchmark_return - rf))
}

/// Calculate tracking error (standard deviation of return differences)
pub fn calculate_tracking_error(returns: &[Decimal], benchmark_returns: &[Decimal]) -> Option<Decimal> {
    if returns.len() != benchmark_returns.len() || returns.is_empty() {
        return None;
    }
    
    // Calculate return differences
    let mut differences = Vec::with_capacity(returns.len());
    for i in 0..returns.len() {
        differences.push(returns[i] - benchmark_returns[i]);
    }
    
    // Calculate standard deviation of differences
    Some(calculate_volatility(&differences))
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
    let rf = risk_free_rate.unwrap_or(Decimal::from_f64(RISK_FREE_RATE).unwrap_or(Decimal::ZERO));
    
    if beta == Decimal::ZERO {
        return None;
    }
    
    Some((annualized_return - rf) / beta)
}

/// Calculate all risk metrics for a return series
pub fn calculate_risk_metrics(
    returns: &[Decimal],
    annualized_return: Decimal,
    benchmark_returns: Option<&[Decimal]>,
    annualized_benchmark_return: Option<Decimal>,
    risk_free_rate: Option<Decimal>
) -> RiskMetrics {
    let mut metrics = RiskMetrics::default();
    
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
    if let (Some(bench_returns), Some(bench_annual_return)) = (benchmark_returns, annualized_benchmark_return) {
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
        if let Some(tracking_error) = calculate_tracking_error(returns, bench_returns) {
            metrics.tracking_error = Some(tracking_error);
            
            // Calculate information ratio
            metrics.information_ratio = calculate_information_ratio(
                annualized_return,
                bench_annual_return,
                tracking_error
            );
        }
    }
    
    metrics
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_calculate_volatility() {
        let returns = vec![
            Decimal::from_f64(0.01).unwrap(),
            Decimal::from_f64(-0.02).unwrap(),
            Decimal::from_f64(0.03).unwrap(),
            Decimal::from_f64(0.01).unwrap(),
            Decimal::from_f64(-0.01).unwrap(),
        ];
        
        let volatility = calculate_volatility(&returns);
        assert!(volatility > Decimal::ZERO);
    }
    
    #[test]
    fn test_calculate_sharpe_ratio() {
        let annualized_return = Decimal::from_f64(0.10).unwrap(); // 10%
        let volatility = Decimal::from_f64(0.15).unwrap(); // 15%
        let risk_free_rate = Decimal::from_f64(0.02).unwrap(); // 2%
        
        let sharpe = calculate_sharpe_ratio(annualized_return, volatility, Some(risk_free_rate));
        
        // Expected: (0.10 - 0.02) / 0.15 = 0.533...
        let expected = Decimal::from_f64(0.533333).unwrap();
        assert!((sharpe - expected).abs() < Decimal::from_f64(0.001).unwrap());
    }
} 