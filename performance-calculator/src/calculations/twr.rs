use async_trait::async_trait;
use chrono::{NaiveDate, Utc};
use rust_decimal::Decimal;
use std::sync::Arc;
use anyhow::Result;

use crate::calculations::performance_metrics::{
    TimeWeightedReturn, MoneyWeightedReturn, CashFlow,
    calculate_modified_dietz, calculate_daily_twr, calculate_irr
};
use crate::calculations::risk_metrics::{
    calculate_volatility, calculate_sharpe_ratio, calculate_sortino_ratio,
    calculate_max_drawdown, ReturnSeries
};

/// Trait for time-weighted return calculations
#[async_trait]
pub trait TimeWeightedReturnCalculator: Send + Sync {
    /// Calculate time-weighted return for a set of transactions
    async fn calculate_twr(
        &self,
        beginning_value: Decimal,
        ending_value: Decimal,
        cash_flows: &[CashFlow],
        start_date: NaiveDate,
        end_date: NaiveDate
    ) -> Result<TimeWeightedReturn>;
    
    /// Calculate daily time-weighted return
    async fn calculate_daily_twr(
        &self,
        daily_values: &[(NaiveDate, Decimal)],
        cash_flows: &[CashFlow]
    ) -> Result<TimeWeightedReturn>;
}

/// Standard implementation of TWR calculator
pub struct StandardTWRCalculator;

impl StandardTWRCalculator {
    /// Create a new TWR calculator
    pub fn new() -> Self {
        Self {}
    }
}

#[async_trait]
impl TimeWeightedReturnCalculator for StandardTWRCalculator {
    async fn calculate_twr(
        &self,
        beginning_value: Decimal,
        ending_value: Decimal,
        cash_flows: &[CashFlow],
        start_date: NaiveDate,
        end_date: NaiveDate
    ) -> Result<TimeWeightedReturn> {
        calculate_modified_dietz(beginning_value, ending_value, cash_flows, start_date, end_date)
    }
    
    async fn calculate_daily_twr(
        &self,
        daily_values: &[(NaiveDate, Decimal)],
        cash_flows: &[CashFlow]
    ) -> Result<TimeWeightedReturn> {
        calculate_daily_twr(daily_values, cash_flows)
    }
}

/// Trait for money-weighted return calculations
#[async_trait]
pub trait MoneyWeightedReturnCalculator: Send + Sync {
    /// Calculate money-weighted return (IRR)
    async fn calculate_mwr(
        &self,
        cash_flows: &[CashFlow],
        final_value: Decimal,
        max_iterations: usize,
        tolerance: Decimal
    ) -> Result<MoneyWeightedReturn>;
}

/// Standard implementation of MWR calculator
pub struct StandardMWRCalculator;

impl StandardMWRCalculator {
    /// Create a new MWR calculator
    pub fn new() -> Self {
        Self {}
    }
}

#[async_trait]
impl MoneyWeightedReturnCalculator for StandardMWRCalculator {
    async fn calculate_mwr(
        &self,
        cash_flows: &[CashFlow],
        final_value: Decimal,
        max_iterations: usize,
        tolerance: Decimal
    ) -> Result<MoneyWeightedReturn> {
        calculate_irr(cash_flows, final_value, max_iterations, tolerance)
    }
}

/// Trait for risk metrics calculations
#[async_trait]
pub trait RiskCalculator: Send + Sync {
    /// Calculate volatility
    async fn calculate_volatility(&self, returns: &ReturnSeries) -> Result<Decimal>;
    
    /// Calculate Sharpe ratio
    async fn calculate_sharpe_ratio(
        &self,
        returns: &ReturnSeries,
        risk_free_rate: Decimal
    ) -> Result<Decimal>;
    
    /// Calculate Sortino ratio
    async fn calculate_sortino_ratio(
        &self,
        returns: &ReturnSeries,
        risk_free_rate: Decimal,
        target_return: Option<Decimal>
    ) -> Result<Decimal>;
    
    /// Calculate maximum drawdown
    async fn calculate_max_drawdown(&self, returns: &ReturnSeries) -> Result<Decimal>;
}

/// Standard implementation of risk calculator
pub struct StandardRiskCalculator;

impl StandardRiskCalculator {
    /// Create a new risk calculator
    pub fn new() -> Self {
        Self {}
    }
}

#[async_trait]
impl RiskCalculator for StandardRiskCalculator {
    async fn calculate_volatility(&self, returns: &ReturnSeries) -> Result<Decimal> {
        // Convert ReturnSeries to a slice of Decimal
        let return_values = &returns.values;
        Ok(calculate_volatility(return_values))
    }
    
    async fn calculate_sharpe_ratio(
        &self,
        returns: &ReturnSeries,
        risk_free_rate: Decimal
    ) -> Result<Decimal> {
        // Calculate average return
        let avg_return = if !returns.values.is_empty() {
            returns.values.iter().sum::<Decimal>() / Decimal::from(returns.values.len())
        } else {
            Decimal::ZERO
        };
        
        // Calculate volatility
        let volatility = calculate_volatility(&returns.values);
        
        // Calculate Sharpe ratio
        Ok(calculate_sharpe_ratio(avg_return, volatility, Some(risk_free_rate)))
    }
    
    async fn calculate_sortino_ratio(
        &self,
        returns: &ReturnSeries,
        risk_free_rate: Decimal,
        target_return: Option<Decimal>
    ) -> Result<Decimal> {
        // Calculate average return
        let avg_return = if !returns.values.is_empty() {
            returns.values.iter().sum::<Decimal>() / Decimal::from(returns.values.len())
        } else {
            Decimal::ZERO
        };
        
        // Calculate Sortino ratio
        Ok(calculate_sortino_ratio(avg_return, &returns.values, Some(risk_free_rate)))
    }
    
    async fn calculate_max_drawdown(&self, returns: &ReturnSeries) -> Result<Decimal> {
        // Convert ReturnSeries to a slice of Decimal
        let return_values = &returns.values;
        Ok(calculate_max_drawdown(return_values))
    }
} 