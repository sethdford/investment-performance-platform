use chrono::{DateTime, Utc};
use crate::validation::{
    validate_non_empty, validate_date_order, validate_all_non_negative, 
    validate_min_items, ValidationError
};
use std::error::Error;
use std::fmt;

/// Represents errors that can occur during financial calculations
#[derive(Debug)]
pub enum CalculationError {
    Validation(ValidationError),
    InsufficientData(&'static str),
    NonConvergence(&'static str),
    DivisionByZero(&'static str),
    InvalidInput(&'static str),
    Other(String),
}

impl fmt::Display for CalculationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CalculationError::Validation(err) => write!(f, "Validation error: {}", err),
            CalculationError::InsufficientData(msg) => write!(f, "Insufficient data: {}", msg),
            CalculationError::NonConvergence(msg) => write!(f, "Non-convergence: {}", msg),
            CalculationError::DivisionByZero(msg) => write!(f, "Division by zero: {}", msg),
            CalculationError::InvalidInput(msg) => write!(f, "Invalid input: {}", msg),
            CalculationError::Other(msg) => write!(f, "Calculation error: {}", msg),
        }
    }
}

impl Error for CalculationError {}

impl From<ValidationError> for CalculationError {
    fn from(err: ValidationError) -> Self {
        CalculationError::Validation(err)
    }
}

/// Calculate the Internal Rate of Return (IRR) with improved error handling
///
/// # Arguments
///
/// * `cash_flows` - A slice of (date, amount) tuples representing cash flows
///
/// # Returns
///
/// * `Result<f64, CalculationError>` - The IRR as a decimal (e.g., 0.05 for 5%) or an error
pub fn calculate_irr(cash_flows: &[(DateTime<Utc>, f64)]) -> Result<f64, CalculationError> {
    // Validate inputs
    validate_non_empty(cash_flows, "cash_flows")?;
    validate_date_order(cash_flows, "cash_flows")?;
    validate_min_items(cash_flows, 2, "cash_flows")?;
    
    // Check if there's at least one positive and one negative cash flow
    let has_positive = cash_flows.iter().any(|(_, amount)| *amount > 0.0);
    let has_negative = cash_flows.iter().any(|(_, amount)| *amount < 0.0);
    
    if !has_positive || !has_negative {
        return Err(CalculationError::InvalidInput(
            "IRR calculation requires at least one positive and one negative cash flow"
        ));
    }
    
    // Convert dates to days from first cash flow
    let base_date = cash_flows[0].0;
    let days_cash_flows: Vec<(f64, f64)> = cash_flows
        .iter()
        .map(|(date, amount)| {
            let days = (*date - base_date).num_days() as f64 / 365.0; // Convert to years
            (days, *amount)
        })
        .collect();
    
    // Newton-Raphson method with improved convergence and error handling
    let mut rate = 0.1; // Initial guess
    let max_iterations = 100;
    let tolerance = 1e-10;
    
    for iteration in 0..max_iterations {
        let mut npv = 0.0;
        let mut derivative = 0.0;
        
        for (time, amount) in &days_cash_flows {
            let discount_factor = (1.0 + rate).powf(-time);
            npv += amount * discount_factor;
            derivative -= time * amount * discount_factor / (1.0 + rate);
        }
        
        // Check for division by zero
        if derivative.abs() < 1e-10 {
            return Err(CalculationError::DivisionByZero(
                "Derivative is too close to zero in IRR calculation"
            ));
        }
        
        let next_rate = rate - npv / derivative;
        
        // Check for convergence
        if (next_rate - rate).abs() < tolerance {
            return Ok(next_rate);
        }
        
        // Check for non-finite values
        if !next_rate.is_finite() {
            return Err(CalculationError::NonConvergence(
                "IRR calculation produced non-finite value"
            ));
        }
        
        rate = next_rate;
        
        // Prevent unreasonable rates
        if rate < -0.999 || rate > 100.0 {
            return Err(CalculationError::NonConvergence(
                "IRR calculation diverged to unreasonable value"
            ));
        }
    }
    
    Err(CalculationError::NonConvergence(
        "IRR calculation did not converge within maximum iterations"
    ))
}

/// Calculate the Time-Weighted Return (TWR) with improved error handling
///
/// # Arguments
///
/// * `valuations` - A slice of (date, value) tuples representing portfolio valuations
/// * `cash_flows` - A slice of (date, amount) tuples representing cash flows
///
/// # Returns
///
/// * `Result<f64, CalculationError>` - The TWR as a decimal (e.g., 0.05 for 5%) or an error
pub fn calculate_twr(
    valuations: &[(DateTime<Utc>, f64)],
    cash_flows: &[(DateTime<Utc>, f64)],
) -> Result<f64, CalculationError> {
    // Validate inputs
    validate_non_empty(valuations, "valuations")?;
    validate_date_order(valuations, "valuations")?;
    validate_all_non_negative(
        &valuations.iter().map(|(_, v)| *v).collect::<Vec<f64>>(),
        "valuations"
    )?;
    
    if !cash_flows.is_empty() {
        validate_date_order(cash_flows, "cash_flows")?;
    }
    
    if valuations.len() < 2 {
        return Err(CalculationError::InsufficientData(
            "TWR calculation requires at least two valuations"
        ));
    }
    
    // Merge valuations and cash flows
    let mut merged: Vec<(DateTime<Utc>, f64, Option<f64>)> = Vec::new();
    
    // Add valuations
    for (date, value) in valuations {
        merged.push((*date, *value, None));
    }
    
    // Add cash flows
    for (date, amount) in cash_flows {
        merged.push((*date, 0.0, Some(*amount)));
    }
    
    // Sort by date
    merged.sort_by(|a, b| a.0.cmp(&b.0));
    
    // Calculate sub-period returns
    let mut sub_period_returns: Vec<f64> = Vec::new();
    let mut current_value = 0.0;
    let mut has_value = false;
    
    for i in 0..merged.len() {
        let (date, value, cash_flow) = merged[i];
        
        if value > 0.0 {
            // This is a valuation point
            if has_value {
                // Calculate return for this sub-period
                let mut end_value = value;
                let mut start_value = current_value;
                
                // Adjust for any cash flows between the last valuation and this one
                for j in 0..i {
                    let (cf_date, _, cf_amount) = merged[j];
                    if cf_amount.is_some() && cf_date > merged[i-1].0 && cf_date <= date {
                        start_value -= cf_amount.unwrap();
                    }
                }
                
                // Avoid division by zero
                if start_value.abs() < 1e-10 {
                    return Err(CalculationError::DivisionByZero(
                        "Starting value is too close to zero in TWR calculation"
                    ));
                }
                
                let sub_period_return = end_value / start_value - 1.0;
                sub_period_returns.push(sub_period_return);
            }
            
            current_value = value;
            has_value = true;
        } else if let Some(amount) = cash_flow {
            // This is a cash flow
            current_value += amount;
        }
    }
    
    // Calculate the cumulative TWR
    let mut twr = 1.0;
    for r in sub_period_returns {
        twr *= (1.0 + r);
    }
    
    Ok(twr - 1.0)
}

/// Calculate the Money-Weighted Return (MWR) with improved error handling
///
/// # Arguments
///
/// * `valuations` - A slice of (date, value) tuples representing portfolio valuations
/// * `cash_flows` - A slice of (date, amount) tuples representing cash flows
///
/// # Returns
///
/// * `Result<f64, CalculationError>` - The MWR as a decimal (e.g., 0.05 for 5%) or an error
pub fn calculate_mwr(
    valuations: &[(DateTime<Utc>, f64)],
    cash_flows: &[(DateTime<Utc>, f64)],
) -> Result<f64, CalculationError> {
    // Validate inputs
    validate_non_empty(valuations, "valuations")?;
    validate_date_order(valuations, "valuations")?;
    validate_all_non_negative(
        &valuations.iter().map(|(_, v)| *v).collect::<Vec<f64>>(),
        "valuations"
    )?;
    
    if valuations.len() < 2 {
        return Err(CalculationError::InsufficientData(
            "MWR calculation requires at least two valuations"
        ));
    }
    
    // Create adjusted cash flows for IRR calculation
    let mut adjusted_cash_flows = Vec::new();
    
    // Add initial valuation as negative cash flow (outflow)
    adjusted_cash_flows.push((valuations[0].0, -valuations[0].1));
    
    // Add all cash flows
    for (date, amount) in cash_flows {
        adjusted_cash_flows.push((*date, *amount));
    }
    
    // Add final valuation as positive cash flow (inflow)
    let last_valuation = valuations.last().unwrap();
    adjusted_cash_flows.push((last_valuation.0, last_valuation.1));
    
    // Calculate IRR on adjusted cash flows
    calculate_irr(&adjusted_cash_flows)
}

/// Calculate volatility (standard deviation of returns) with improved error handling
///
/// # Arguments
///
/// * `returns` - A slice of return values
///
/// # Returns
///
/// * `Result<f64, CalculationError>` - The volatility or an error
pub fn calculate_volatility(returns: &[f64]) -> Result<f64, CalculationError> {
    // Validate inputs
    validate_non_empty(returns, "returns")?;
    
    if returns.len() < 2 {
        return Err(CalculationError::InsufficientData(
            "Volatility calculation requires at least two return values"
        ));
    }
    
    // Calculate mean
    let mean = returns.iter().sum::<f64>() / returns.len() as f64;
    
    // Calculate sum of squared deviations
    let sum_squared_deviations = returns
        .iter()
        .map(|r| (r - mean).powi(2))
        .sum::<f64>();
    
    // Calculate variance (using n-1 for sample standard deviation)
    let variance = sum_squared_deviations / (returns.len() - 1) as f64;
    
    // Calculate standard deviation
    if variance < 0.0 {
        return Err(CalculationError::InvalidInput(
            "Variance is negative in volatility calculation"
        ));
    }
    
    Ok(variance.sqrt())
}

/// Calculate Sharpe ratio with improved error handling
///
/// # Arguments
///
/// * `returns` - A slice of return values
/// * `risk_free_rate` - The risk-free rate as a decimal (e.g., 0.02 for 2%)
///
/// # Returns
///
/// * `Result<f64, CalculationError>` - The Sharpe ratio or an error
pub fn calculate_sharpe_ratio(
    returns: &[f64],
    risk_free_rate: f64,
) -> Result<f64, CalculationError> {
    // Validate inputs
    validate_non_empty(returns, "returns")?;
    
    if returns.len() < 2 {
        return Err(CalculationError::InsufficientData(
            "Sharpe ratio calculation requires at least two return values"
        ));
    }
    
    // Calculate mean return
    let mean_return = returns.iter().sum::<f64>() / returns.len() as f64;
    
    // Calculate volatility
    let volatility = calculate_volatility(returns)?;
    
    // Avoid division by zero
    if volatility.abs() < 1e-10 {
        return Err(CalculationError::DivisionByZero(
            "Volatility is too close to zero in Sharpe ratio calculation"
        ));
    }
    
    // Calculate Sharpe ratio
    Ok((mean_return - risk_free_rate) / volatility)
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;
    
    #[test]
    fn test_calculate_irr_valid() {
        let cash_flows = vec![
            (Utc.ymd(2020, 1, 1).and_hms(0, 0, 0), -1000.0),
            (Utc.ymd(2021, 1, 1).and_hms(0, 0, 0), 1100.0),
        ];
        
        let result = calculate_irr(&cash_flows);
        assert!(result.is_ok());
        
        let irr = result.unwrap();
        assert!((irr - 0.1).abs() < 0.001); // Should be close to 10%
    }
    
    #[test]
    fn test_calculate_irr_invalid() {
        // Empty cash flows
        let empty: Vec<(DateTime<Utc>, f64)> = vec![];
        assert!(calculate_irr(&empty).is_err());
        
        // All positive cash flows
        let all_positive = vec![
            (Utc.ymd(2020, 1, 1).and_hms(0, 0, 0), 1000.0),
            (Utc.ymd(2021, 1, 1).and_hms(0, 0, 0), 1100.0),
        ];
        assert!(calculate_irr(&all_positive).is_err());
        
        // All negative cash flows
        let all_negative = vec![
            (Utc.ymd(2020, 1, 1).and_hms(0, 0, 0), -1000.0),
            (Utc.ymd(2021, 1, 1).and_hms(0, 0, 0), -1100.0),
        ];
        assert!(calculate_irr(&all_negative).is_err());
    }
    
    #[test]
    fn test_calculate_twr_valid() {
        let valuations = vec![
            (Utc.ymd(2020, 1, 1).and_hms(0, 0, 0), 1000.0),
            (Utc.ymd(2021, 1, 1).and_hms(0, 0, 0), 1100.0),
        ];
        
        let cash_flows = vec![
            (Utc.ymd(2020, 6, 1).and_hms(0, 0, 0), 100.0),
        ];
        
        let result = calculate_twr(&valuations, &cash_flows);
        assert!(result.is_ok());
    }
    
    #[test]
    fn test_calculate_twr_invalid() {
        // Empty valuations
        let empty: Vec<(DateTime<Utc>, f64)> = vec![];
        let cash_flows = vec![];
        assert!(calculate_twr(&empty, &cash_flows).is_err());
        
        // Only one valuation
        let single_valuation = vec![
            (Utc.ymd(2020, 1, 1).and_hms(0, 0, 0), 1000.0),
        ];
        assert!(calculate_twr(&single_valuation, &cash_flows).is_err());
        
        // Negative valuation
        let negative_valuation = vec![
            (Utc.ymd(2020, 1, 1).and_hms(0, 0, 0), 1000.0),
            (Utc.ymd(2021, 1, 1).and_hms(0, 0, 0), -1100.0),
        ];
        assert!(calculate_twr(&negative_valuation, &cash_flows).is_err());
    }
    
    #[test]
    fn test_calculate_mwr_valid() {
        let valuations = vec![
            (Utc.ymd(2020, 1, 1).and_hms(0, 0, 0), 1000.0),
            (Utc.ymd(2021, 1, 1).and_hms(0, 0, 0), 1100.0),
        ];
        
        let cash_flows = vec![
            (Utc.ymd(2020, 6, 1).and_hms(0, 0, 0), -100.0),
        ];
        
        let result = calculate_mwr(&valuations, &cash_flows);
        assert!(result.is_ok());
    }
    
    #[test]
    fn test_calculate_volatility_valid() {
        let returns = vec![0.01, 0.02, -0.01, 0.03, 0.01];
        
        let result = calculate_volatility(&returns);
        assert!(result.is_ok());
    }
    
    #[test]
    fn test_calculate_volatility_invalid() {
        // Empty returns
        let empty: Vec<f64> = vec![];
        assert!(calculate_volatility(&empty).is_err());
        
        // Only one return
        let single_return = vec![0.01];
        assert!(calculate_volatility(&single_return).is_err());
    }
    
    #[test]
    fn test_calculate_sharpe_ratio_valid() {
        let returns = vec![0.01, 0.02, -0.01, 0.03, 0.01];
        let risk_free_rate = 0.005;
        
        let result = calculate_sharpe_ratio(&returns, risk_free_rate);
        assert!(result.is_ok());
    }
    
    #[test]
    fn test_calculate_sharpe_ratio_invalid() {
        // Empty returns
        let empty: Vec<f64> = vec![];
        let risk_free_rate = 0.005;
        assert!(calculate_sharpe_ratio(&empty, risk_free_rate).is_err());
        
        // Only one return
        let single_return = vec![0.01];
        assert!(calculate_sharpe_ratio(&single_return, risk_free_rate).is_err());
    }
} 