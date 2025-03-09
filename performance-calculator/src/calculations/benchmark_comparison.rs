use rust_decimal::Decimal;
use rust_decimal::prelude::*;
use anyhow::{Result, anyhow};
use std::collections::HashMap;
use chrono::{DateTime, Utc, NaiveDate};
use serde::{Serialize, Deserialize};
use crate::calculations::risk_metrics::{ReturnSeries, calculate_beta, calculate_alpha, calculate_tracking_error, calculate_information_ratio};

/// Benchmark comparison results
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BenchmarkComparison {
    /// Portfolio return for the period
    pub portfolio_return: Decimal,
    /// Benchmark return for the period
    pub benchmark_return: Decimal,
    /// Excess return (portfolio - benchmark)
    pub excess_return: Decimal,
    /// Beta (systematic risk)
    pub beta: Option<Decimal>,
    /// Alpha (excess return over benchmark)
    pub alpha: Option<Decimal>,
    /// Tracking error (standard deviation of return differences)
    pub tracking_error: Option<Decimal>,
    /// Information ratio (excess return per unit of tracking error)
    pub information_ratio: Option<Decimal>,
    /// Up capture ratio (performance in up markets)
    pub up_capture: Option<Decimal>,
    /// Down capture ratio (performance in down markets)
    pub down_capture: Option<Decimal>,
    /// Batting average (percentage of periods outperforming)
    pub batting_average: Option<Decimal>,
}

impl Default for BenchmarkComparison {
    fn default() -> Self {
        Self {
            portfolio_return: Decimal::ZERO,
            benchmark_return: Decimal::ZERO,
            excess_return: Decimal::ZERO,
            beta: None,
            alpha: None,
            tracking_error: None,
            information_ratio: None,
            up_capture: None,
            down_capture: None,
            batting_average: None,
        }
    }
}

/// Calculate up capture ratio (how the portfolio performs in up markets)
pub fn calculate_up_capture(
    portfolio_returns: &[Decimal],
    benchmark_returns: &[Decimal]
) -> Option<Decimal> {
    if portfolio_returns.len() != benchmark_returns.len() || portfolio_returns.is_empty() {
        return None;
    }
    
    let mut portfolio_up_return = Decimal::ZERO;
    let mut benchmark_up_return = Decimal::ZERO;
    let mut up_periods = 0;
    
    for i in 0..benchmark_returns.len() {
        if benchmark_returns[i] > Decimal::ZERO {
            portfolio_up_return += portfolio_returns[i];
            benchmark_up_return += benchmark_returns[i];
            up_periods += 1;
        }
    }
    
    if up_periods == 0 || benchmark_up_return == Decimal::ZERO {
        return None;
    }
    
    Some((portfolio_up_return / Decimal::from(up_periods)) / 
         (benchmark_up_return / Decimal::from(up_periods)) * 
         Decimal::from(100))
}

/// Calculate down capture ratio (how the portfolio performs in down markets)
pub fn calculate_down_capture(
    portfolio_returns: &[Decimal],
    benchmark_returns: &[Decimal]
) -> Option<Decimal> {
    if portfolio_returns.len() != benchmark_returns.len() || portfolio_returns.is_empty() {
        return None;
    }
    
    let mut portfolio_down_return = Decimal::ZERO;
    let mut benchmark_down_return = Decimal::ZERO;
    let mut down_periods = 0;
    
    for i in 0..benchmark_returns.len() {
        if benchmark_returns[i] < Decimal::ZERO {
            portfolio_down_return += portfolio_returns[i];
            benchmark_down_return += benchmark_returns[i];
            down_periods += 1;
        }
    }
    
    if down_periods == 0 || benchmark_down_return == Decimal::ZERO {
        return None;
    }
    
    Some((portfolio_down_return / Decimal::from(down_periods)) / 
         (benchmark_down_return / Decimal::from(down_periods)) * 
         Decimal::from(100))
}

/// Calculate batting average (percentage of periods outperforming)
pub fn calculate_batting_average(
    portfolio_returns: &[Decimal],
    benchmark_returns: &[Decimal]
) -> Option<Decimal> {
    if portfolio_returns.len() != benchmark_returns.len() || portfolio_returns.is_empty() {
        return None;
    }
    
    let mut outperforming_periods = 0;
    
    for i in 0..portfolio_returns.len() {
        if portfolio_returns[i] > benchmark_returns[i] {
            outperforming_periods += 1;
        }
    }
    
    Some(Decimal::from(outperforming_periods) / Decimal::from(portfolio_returns.len()))
}

/// Calculate all benchmark comparison metrics
pub fn calculate_benchmark_comparison(
    portfolio_series: &ReturnSeries,
    benchmark_series: &ReturnSeries,
    annualized_portfolio_return: Decimal,
    annualized_benchmark_return: Decimal,
    risk_free_rate: Option<Decimal>
) -> Result<BenchmarkComparison> {
    if portfolio_series.dates.len() != portfolio_series.values.len() ||
       benchmark_series.dates.len() != benchmark_series.values.len() ||
       portfolio_series.dates.len() != benchmark_series.dates.len() {
        return Err(anyhow!("Date and return series must have the same length"));
    }
    
    let portfolio_returns = &portfolio_series.values;
    let benchmark_returns = &benchmark_series.values;
    
    // Calculate cumulative returns
    let portfolio_return = portfolio_returns.iter().fold(Decimal::ONE, |acc, r| acc * (Decimal::ONE + *r)) - Decimal::ONE;
    let benchmark_return = benchmark_returns.iter().fold(Decimal::ONE, |acc, r| acc * (Decimal::ONE + *r)) - Decimal::ONE;
    let excess_return = portfolio_return - benchmark_return;
    
    // Calculate beta
    let beta = calculate_beta(portfolio_returns, benchmark_returns);
    
    // Calculate alpha
    let alpha = beta.and_then(|b| {
        Some(calculate_alpha(
            annualized_portfolio_return,
            b,
            annualized_benchmark_return,
            risk_free_rate
        ))
    });
    
    // Calculate tracking error
    let tracking_error = calculate_tracking_error(portfolio_returns, benchmark_returns);
    
    // Calculate information ratio
    let information_ratio = if tracking_error == Decimal::ZERO {
        None
    } else {
        Some(excess_return / tracking_error)
    };
    
    // Calculate up/down capture
    let up_capture = calculate_up_capture(portfolio_returns, benchmark_returns);
    let down_capture = calculate_down_capture(portfolio_returns, benchmark_returns);
    
    // Calculate batting average
    let batting_average = calculate_batting_average(portfolio_returns, benchmark_returns);
    
    Ok(BenchmarkComparison {
        portfolio_return,
        benchmark_return,
        excess_return,
        beta,
        alpha,
        tracking_error: Some(tracking_error),
        information_ratio,
        up_capture,
        down_capture,
        batting_average,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;
    
    #[test]
    fn test_benchmark_comparison() {
        // Create test data
        let dates = vec![
            NaiveDate::from_ymd_opt(2023, 1, 31).unwrap(),
            NaiveDate::from_ymd_opt(2023, 2, 28).unwrap(),
            NaiveDate::from_ymd_opt(2023, 3, 31).unwrap(),
            NaiveDate::from_ymd_opt(2023, 4, 30).unwrap(),
            NaiveDate::from_ymd_opt(2023, 5, 31).unwrap(),
            NaiveDate::from_ymd_opt(2023, 6, 30).unwrap(),
        ];
        
        let portfolio_returns = vec![
            dec!(0.02),   // 2%
            dec!(-0.015), // -1.5%
            dec!(0.035),  // 3.5%
            dec!(0.01),   // 1%
            dec!(-0.01),  // -1%
            dec!(0.03),   // 3%
        ];
        
        let benchmark_returns = vec![
            dec!(0.015),  // 1.5%
            dec!(-0.02),  // -2%
            dec!(0.025),  // 2.5%
            dec!(0.01),   // 1%
            dec!(-0.015), // -1.5%
            dec!(0.025),  // 2.5%
        ];
        
        let portfolio_series = ReturnSeries {
            dates: dates.clone(),
            values: portfolio_returns.clone(),
        };
        
        let benchmark_series = ReturnSeries {
            dates,
            values: benchmark_returns.clone(),
        };
        
        // Annualized returns (simplified for test)
        let annualized_portfolio_return = dec!(0.08); // 8%
        let annualized_benchmark_return = dec!(0.06); // 6%
        
        // Calculate benchmark comparison
        let comparison = calculate_benchmark_comparison(
            &portfolio_series,
            &benchmark_series,
            annualized_portfolio_return,
            annualized_benchmark_return,
            None
        ).unwrap();
        
        // Test results
        assert!(comparison.portfolio_return > comparison.benchmark_return);
        assert!(comparison.excess_return > dec!(0));
        assert!(comparison.beta.unwrap() > dec!(0));
        
        // Test up/down capture
        let up_capture = calculate_up_capture(&portfolio_returns, &benchmark_returns).unwrap();
        assert!(up_capture > dec!(100)); // Portfolio outperforms in up markets
        
        let down_capture = calculate_down_capture(&portfolio_returns, &benchmark_returns).unwrap();
        assert!(down_capture < dec!(100)); // Portfolio outperforms in down markets
        
        // Test batting average
        let batting = calculate_batting_average(&portfolio_returns, &benchmark_returns).unwrap();
        assert!(batting > dec!(0.5)); // Portfolio outperforms more than half the time
    }
} 