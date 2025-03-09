use rust_decimal::Decimal;
use rust_decimal::prelude::*;
use anyhow::{Result, anyhow};
use std::collections::HashMap;
use chrono::{DateTime, Utc, NaiveDate, Duration};
use serde::{Serialize, Deserialize};
use crate::calculations::risk_metrics::ReturnSeries;

/// Time-Weighted Return calculation
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TimeWeightedReturn {
    /// The calculated return value
    pub return_value: Decimal,
    /// The method used for calculation (e.g., "Modified Dietz", "Daily Valuation")
    pub calculation_method: String,
    /// Sub-period returns for multi-period calculations
    #[serde(default)]
    pub sub_period_returns: Vec<SubPeriodReturn>,
    /// Whether the return is annualized
    #[serde(default)]
    pub annualized: bool,
}

impl Default for TimeWeightedReturn {
    fn default() -> Self {
        Self {
            return_value: Decimal::ZERO,
            calculation_method: "Modified Dietz".to_string(),
            sub_period_returns: Vec::new(),
            annualized: false,
        }
    }
}

/// Money-Weighted Return calculation (Internal Rate of Return)
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MoneyWeightedReturn {
    /// The calculated return value
    pub return_value: Decimal,
    /// The method used for calculation (e.g., "IRR", "XIRR")
    pub calculation_method: String,
    /// Whether the return is annualized
    #[serde(default)]
    pub annualized: bool,
}

impl Default for MoneyWeightedReturn {
    fn default() -> Self {
        Self {
            return_value: Decimal::ZERO,
            calculation_method: "Internal Rate of Return".to_string(),
            annualized: false,
        }
    }
}

/// Sub-period return for multi-period TWR calculations
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SubPeriodReturn {
    /// Start date of the sub-period
    pub start_date: NaiveDate,
    /// End date of the sub-period
    pub end_date: NaiveDate,
    /// Return value for the sub-period
    pub return_value: Decimal,
    /// Beginning market value
    pub beginning_value: Decimal,
    /// Ending market value
    pub ending_value: Decimal,
    /// Net cash flow during the period
    pub net_cash_flow: Decimal,
    /// Weighted cash flow for Modified Dietz
    pub weighted_cash_flow: Option<Decimal>,
}

/// Cash flow for performance calculations
#[derive(Debug, Clone)]
pub struct CashFlow {
    /// Date of the cash flow
    pub date: NaiveDate,
    /// Amount of the cash flow (positive for inflows, negative for outflows)
    pub amount: Decimal,
    /// Description of the cash flow
    pub description: Option<String>,
}

/// Performance attribution analysis
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PerformanceAttribution {
    /// Total portfolio return
    pub portfolio_return: Decimal,
    /// Benchmark return
    pub benchmark_return: Decimal,
    /// Excess return (portfolio - benchmark)
    pub excess_return: Decimal,
    /// Allocation effect by asset class
    pub allocation_effect: HashMap<String, Decimal>,
    /// Selection effect by asset class
    pub selection_effect: HashMap<String, Decimal>,
    /// Interaction effect by asset class
    pub interaction_effect: HashMap<String, Decimal>,
    /// Currency effect by asset class (for international portfolios)
    pub currency_effect: Option<HashMap<String, Decimal>>,
}

/// Calculate time-weighted return using the Modified Dietz method
pub fn calculate_modified_dietz(
    beginning_value: Decimal,
    ending_value: Decimal,
    cash_flows: &[CashFlow],
    start_date: NaiveDate,
    end_date: NaiveDate
) -> Result<TimeWeightedReturn> {
    // Calculate total days in period
    let period_days = (end_date.signed_duration_since(start_date).num_days() as f64).max(1.0);
    
    // Calculate net cash flow and weighted cash flow
    let mut net_cash_flow = Decimal::ZERO;
    let mut weighted_cash_flow = Decimal::ZERO;
    
    for cf in cash_flows {
        if cf.date < start_date || cf.date > end_date {
            continue;
        }
        
        net_cash_flow += cf.amount;
        
        // Calculate weight based on days from start
        let days_from_start = cf.date.signed_duration_since(start_date).num_days() as f64;
        let weight = Decimal::from_f64((period_days - days_from_start) / period_days)
            .unwrap_or(Decimal::ZERO);
        
        weighted_cash_flow += cf.amount * weight;
    }
    
    // Calculate Modified Dietz return
    let denominator = beginning_value + weighted_cash_flow;
    
    if denominator == Decimal::ZERO {
        return Err(anyhow!("Cannot calculate Modified Dietz return with zero denominator"));
    }
    
    let return_value = (ending_value - beginning_value - net_cash_flow) / denominator;
    
    // Create sub-period return
    let sub_period = SubPeriodReturn {
        start_date,
        end_date,
        return_value,
        beginning_value,
        ending_value,
        net_cash_flow,
        weighted_cash_flow: Some(weighted_cash_flow),
    };
    
    Ok(TimeWeightedReturn {
        return_value,
        calculation_method: "Modified Dietz".to_string(),
        sub_period_returns: vec![sub_period],
        annualized: false,
    })
}

/// Calculate time-weighted return using daily valuation method
pub fn calculate_daily_twr(
    daily_values: &[(NaiveDate, Decimal)],
    cash_flows: &[CashFlow]
) -> Result<TimeWeightedReturn> {
    if daily_values.len() < 2 {
        return Err(anyhow!("Need at least two valuation points to calculate daily TWR"));
    }
    
    // Sort values by date
    let mut sorted_values = daily_values.to_vec();
    sorted_values.sort_by(|a, b| a.0.cmp(&b.0));
    
    // Calculate sub-period returns
    let mut sub_period_returns = Vec::new();
    let mut cumulative_return = Decimal::ONE;
    
    for i in 0..sorted_values.len() - 1 {
        let start_date = sorted_values[i].0;
        let end_date = sorted_values[i + 1].0;
        let beginning_value = sorted_values[i].1;
        let ending_value = sorted_values[i + 1].1;
        
        // Calculate net cash flow for this sub-period
        let mut net_cash_flow = Decimal::ZERO;
        for cf in cash_flows {
            if cf.date >= start_date && cf.date < end_date {
                net_cash_flow += cf.amount;
            }
        }
        
        // Calculate sub-period return
        let adjusted_beginning = beginning_value + net_cash_flow;
        
        if adjusted_beginning == Decimal::ZERO {
            return Err(anyhow!("Cannot calculate daily TWR with zero adjusted beginning value"));
        }
        
        let sub_return = (ending_value / adjusted_beginning) - Decimal::ONE;
        
        // Update cumulative return
        cumulative_return *= Decimal::ONE + sub_return;
        
        // Add sub-period return
        sub_period_returns.push(SubPeriodReturn {
            start_date,
            end_date,
            return_value: sub_return,
            beginning_value,
            ending_value,
            net_cash_flow,
            weighted_cash_flow: None,
        });
    }
    
    // Calculate overall return
    let return_value = cumulative_return - Decimal::ONE;
    
    Ok(TimeWeightedReturn {
        return_value,
        calculation_method: "Daily Valuation".to_string(),
        sub_period_returns,
        annualized: false,
    })
}

/// Calculate money-weighted return (internal rate of return)
pub fn calculate_irr(
    cash_flows: &[CashFlow],
    final_value: Decimal,
    max_iterations: usize,
    tolerance: Decimal
) -> Result<MoneyWeightedReturn> {
    if cash_flows.is_empty() {
        return Err(anyhow!("Cannot calculate IRR without cash flows"));
    }
    
    // Sort cash flows by date
    let mut sorted_flows = cash_flows.to_vec();
    sorted_flows.sort_by(|a, b| a.date.cmp(&b.date));
    
    // Add final value as a cash flow
    let final_date = sorted_flows.last().unwrap().date;
    let final_cf = CashFlow {
        date: final_date,
        amount: final_value,
        description: Some("Final Value".to_string()),
    };
    
    let mut all_flows = sorted_flows.clone();
    all_flows.push(final_cf);
    
    // Calculate days between cash flows
    let base_date = all_flows.first().unwrap().date;
    let days: Vec<i64> = all_flows
        .iter()
        .map(|cf| cf.date.signed_duration_since(base_date).num_days())
        .collect();
    
    // Extract amounts
    let amounts: Vec<Decimal> = all_flows.iter().map(|cf| cf.amount).collect();
    
    // Newton-Raphson method to find IRR
    let mut rate = Decimal::from_f64(0.1).unwrap(); // Initial guess: 10%
    
    for _ in 0..max_iterations {
        let mut npv = Decimal::ZERO;
        let mut derivative = Decimal::ZERO;
        
        for i in 0..amounts.len() {
            let t = Decimal::from(days[i]) / Decimal::from(365); // Time in years
            
            // Convert to f64 for power calculation, then back to Decimal
            let t_f64 = t.to_f64().unwrap();
            let rate_f64 = rate.to_f64().unwrap();
            let factor_f64 = (1.0 + rate_f64).powf(t_f64);
            let factor = Decimal::from_f64(factor_f64).unwrap();
            
            npv += amounts[i] / factor;
            
            // Calculate derivative using f64
            let derivative_term_f64 = amounts[i].to_f64().unwrap() * t_f64 / ((1.0 + rate_f64).powf(t_f64 + 1.0));
            let derivative_term = Decimal::from_f64(derivative_term_f64).unwrap();
            derivative -= derivative_term;
        }
        
        if derivative == Decimal::ZERO {
            break;
        }
        
        let new_rate = rate - npv / derivative;
        
        if (new_rate - rate).abs() < tolerance {
            rate = new_rate;
            break;
        }
        
        rate = new_rate;
    }
    
    Ok(MoneyWeightedReturn {
        return_value: rate,
        calculation_method: "Internal Rate of Return".to_string(),
        annualized: true,
    })
}

/// Annualize a return over a given period
pub fn annualize_return(
    return_value: Decimal,
    start_date: NaiveDate,
    end_date: NaiveDate
) -> Result<Decimal> {
    let days = end_date.signed_duration_since(start_date).num_days();
    
    if days <= 0 {
        return Err(anyhow!("End date must be after start date"));
    }
    
    let years = Decimal::from(days) / Decimal::from(365);
    
    if years == Decimal::ZERO {
        return Err(anyhow!("Period is too short to annualize"));
    }
    
    // Formula: (1 + r)^(1/years) - 1
    let base = Decimal::ONE + return_value;
    let exponent = Decimal::ONE / years;
    
    // Use f64 for power calculation
    let base_f64 = base.to_f64().ok_or_else(|| anyhow!("Failed to convert to f64"))?;
    let exponent_f64 = exponent.to_f64().ok_or_else(|| anyhow!("Failed to convert to f64"))?;
    
    let result_f64 = base_f64.powf(exponent_f64) - 1.0;
    let result = Decimal::from_f64(result_f64).ok_or_else(|| anyhow!("Failed to convert result to Decimal"))?;
    
    Ok(result)
}

/// Calculate performance attribution
pub fn calculate_attribution(
    portfolio_returns: &HashMap<String, Decimal>,
    benchmark_returns: &HashMap<String, Decimal>,
    portfolio_weights: &HashMap<String, Decimal>,
    benchmark_weights: &HashMap<String, Decimal>
) -> Result<PerformanceAttribution> {
    // Calculate total portfolio and benchmark returns
    let mut portfolio_return = Decimal::ZERO;
    let mut benchmark_return = Decimal::ZERO;
    
    for (asset_class, weight) in portfolio_weights {
        if let Some(return_value) = portfolio_returns.get(asset_class) {
            portfolio_return += weight * return_value;
        }
    }
    
    for (asset_class, weight) in benchmark_weights {
        if let Some(return_value) = benchmark_returns.get(asset_class) {
            benchmark_return += weight * return_value;
        }
    }
    
    let excess_return = portfolio_return - benchmark_return;
    
    // Calculate attribution effects
    let mut allocation_effect = HashMap::new();
    let mut selection_effect = HashMap::new();
    let mut interaction_effect = HashMap::new();
    
    for asset_class in portfolio_weights.keys().chain(benchmark_weights.keys()).collect::<std::collections::HashSet<_>>() {
        let p_weight = *portfolio_weights.get(asset_class).unwrap_or(&Decimal::ZERO);
        let b_weight = *benchmark_weights.get(asset_class).unwrap_or(&Decimal::ZERO);
        let p_return = *portfolio_returns.get(asset_class).unwrap_or(&Decimal::ZERO);
        let b_return = *benchmark_returns.get(asset_class).unwrap_or(&Decimal::ZERO);
        
        // Allocation effect: (portfolio weight - benchmark weight) * benchmark return
        let allocation = (p_weight - b_weight) * b_return;
        allocation_effect.insert(asset_class.clone(), allocation);
        
        // Selection effect: benchmark weight * (portfolio return - benchmark return)
        let selection = b_weight * (p_return - b_return);
        selection_effect.insert(asset_class.clone(), selection);
        
        // Interaction effect: (portfolio weight - benchmark weight) * (portfolio return - benchmark return)
        let interaction = (p_weight - b_weight) * (p_return - b_return);
        interaction_effect.insert(asset_class.clone(), interaction);
    }
    
    Ok(PerformanceAttribution {
        portfolio_return,
        benchmark_return,
        excess_return,
        allocation_effect,
        selection_effect,
        interaction_effect,
        currency_effect: None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_modified_dietz() {
        let beginning_value = Decimal::from_f64(100000.0).unwrap();
        let ending_value = Decimal::from_f64(110000.0).unwrap();
        let start_date = NaiveDate::from_ymd_opt(2023, 1, 1).unwrap();
        let end_date = NaiveDate::from_ymd_opt(2023, 3, 31).unwrap();
        
        let cash_flows = vec![
            CashFlow {
                date: NaiveDate::from_ymd_opt(2023, 2, 15).unwrap(),
                amount: Decimal::from_f64(5000.0).unwrap(),
                description: Some("Deposit".to_string()),
            },
        ];
        
        let result = calculate_modified_dietz(
            beginning_value,
            ending_value,
            &cash_flows,
            start_date,
            end_date
        ).unwrap();
        
        // Expected return: (110000 - 100000 - 5000) / (100000 + 5000*0.5) = 5000 / 102500 = 0.04878...
        assert!(result.return_value > Decimal::from_f64(0.048).unwrap());
        assert!(result.return_value < Decimal::from_f64(0.049).unwrap());
    }
    
    #[test]
    fn test_daily_twr() {
        let daily_values = vec![
            (NaiveDate::from_ymd_opt(2023, 1, 1).unwrap(), Decimal::from_f64(100000.0).unwrap()),
            (NaiveDate::from_ymd_opt(2023, 1, 31).unwrap(), Decimal::from_f64(102000.0).unwrap()),
            (NaiveDate::from_ymd_opt(2023, 2, 28).unwrap(), Decimal::from_f64(108000.0).unwrap()),
            (NaiveDate::from_ymd_opt(2023, 3, 31).unwrap(), Decimal::from_f64(110000.0).unwrap()),
        ];
        
        let cash_flows = vec![
            CashFlow {
                date: NaiveDate::from_ymd_opt(2023, 2, 15).unwrap(),
                amount: Decimal::from_f64(5000.0).unwrap(),
                description: Some("Deposit".to_string()),
            },
        ];
        
        let result = calculate_daily_twr(&daily_values, &cash_flows).unwrap();
        
        // Verify we have the right number of sub-periods
        assert_eq!(result.sub_period_returns.len(), 3);
        
        // Verify the return is positive
        assert!(result.return_value > Decimal::ZERO);
    }
} 