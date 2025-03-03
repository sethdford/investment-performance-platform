use shared::{
    models::{Portfolio, Transaction, Account, Security, Client, Benchmark, Price, Position},
    validation::{Validate, ValidationError},
    AppError,
};
use serde_json::Value;
use chrono::{NaiveDate, DateTime, Utc};
use std::str::FromStr;

/// Converts a ValidationError to an AppError
pub fn validation_error_to_app_error(err: ValidationError) -> AppError {
    AppError::Validation(err.to_string())
}

/// Validates a portfolio from an API request
pub fn validate_portfolio_request(value: &Value) -> Result<Portfolio, AppError> {
    let portfolio: Portfolio = serde_json::from_value(value.clone())
        .map_err(|e| AppError::Validation(format!("Invalid portfolio data: {}", e)))?;
    
    // Validate using the shared validation trait
    portfolio.validate()
        .map_err(validation_error_to_app_error)?;
    
    Ok(portfolio)
}

/// Validates a transaction from an API request
pub fn validate_transaction_request(value: &Value) -> Result<Transaction, AppError> {
    let transaction: Transaction = serde_json::from_value(value.clone())
        .map_err(|e| AppError::Validation(format!("Invalid transaction data: {}", e)))?;
    
    // Validate using the shared validation trait
    transaction.validate()
        .map_err(validation_error_to_app_error)?;
    
    Ok(transaction)
}

/// Validates an account from an API request
pub fn validate_account_request(value: &Value) -> Result<Account, AppError> {
    let account: Account = serde_json::from_value(value.clone())
        .map_err(|e| AppError::Validation(format!("Invalid account data: {}", e)))?;
    
    // Validate using the shared validation trait
    account.validate()
        .map_err(validation_error_to_app_error)?;
    
    Ok(account)
}

/// Validates a security from an API request
pub fn validate_security_request(value: &Value) -> Result<Security, AppError> {
    let security: Security = serde_json::from_value(value.clone())
        .map_err(|e| AppError::Validation(format!("Invalid security data: {}", e)))?;
    
    // Validate using the shared validation trait
    security.validate()
        .map_err(validation_error_to_app_error)?;
    
    Ok(security)
}

/// Validates a client from an API request
pub fn validate_client_request(value: &Value) -> Result<Client, AppError> {
    let client: Client = serde_json::from_value(value.clone())
        .map_err(|e| AppError::Validation(format!("Invalid client data: {}", e)))?;
    
    // Validate using the shared validation trait
    client.validate()
        .map_err(validation_error_to_app_error)?;
    
    Ok(client)
}

/// Validates a benchmark from an API request
pub fn validate_benchmark_request(value: &Value) -> Result<Benchmark, AppError> {
    let benchmark: Benchmark = serde_json::from_value(value.clone())
        .map_err(|e| AppError::Validation(format!("Invalid benchmark data: {}", e)))?;
    
    // Validate using the shared validation trait
    benchmark.validate()
        .map_err(validation_error_to_app_error)?;
    
    Ok(benchmark)
}

/// Validates a price from an API request
pub fn validate_price_request(value: &Value) -> Result<Price, AppError> {
    let price: Price = serde_json::from_value(value.clone())
        .map_err(|e| AppError::Validation(format!("Invalid price data: {}", e)))?;
    
    // Validate using the shared validation trait
    price.validate()
        .map_err(validation_error_to_app_error)?;
    
    Ok(price)
}

/// Validates a position from an API request
pub fn validate_position_request(value: &Value) -> Result<Position, AppError> {
    let position: Position = serde_json::from_value(value.clone())
        .map_err(|e| AppError::Validation(format!("Invalid position data: {}", e)))?;
    
    // Validate using the shared validation trait
    position.validate()
        .map_err(validation_error_to_app_error)?;
    
    Ok(position)
}

/// Validates a date string in ISO format (YYYY-MM-DD)
pub fn validate_date_string(date_str: &str, field_name: &str) -> Result<NaiveDate, AppError> {
    NaiveDate::from_str(date_str)
        .map_err(|_| AppError::Validation(format!("Invalid date format for {}: use YYYY-MM-DD", field_name)))
}

/// Validates a date range
pub fn validate_date_range(start_date: NaiveDate, end_date: NaiveDate) -> Result<(), AppError> {
    if end_date < start_date {
        return Err(AppError::Validation("End date cannot be before start date".to_string()));
    }
    Ok(())
}

/// Validates calculation parameters
pub fn validate_calculation_params(params: &Value) -> Result<(), AppError> {
    // Check that required fields exist
    if !params.is_object() {
        return Err(AppError::Validation("Calculation parameters must be an object".to_string()));
    }
    
    let obj = params.as_object().unwrap();
    
    // Check for required fields
    if !obj.contains_key("portfolio_id") {
        return Err(AppError::Validation("Missing required field: portfolio_id".to_string()));
    }
    
    if !obj.contains_key("start_date") {
        return Err(AppError::Validation("Missing required field: start_date".to_string()));
    }
    
    if !obj.contains_key("end_date") {
        return Err(AppError::Validation("Missing required field: end_date".to_string()));
    }
    
    // Validate portfolio_id
    let portfolio_id = obj.get("portfolio_id")
        .and_then(|v| v.as_str())
        .ok_or_else(|| AppError::Validation("portfolio_id must be a string".to_string()))?;
    
    if portfolio_id.trim().is_empty() {
        return Err(AppError::Validation("portfolio_id cannot be empty".to_string()));
    }
    
    // Validate dates
    let start_date_str = obj.get("start_date")
        .and_then(|v| v.as_str())
        .ok_or_else(|| AppError::Validation("start_date must be a string".to_string()))?;
    
    let end_date_str = obj.get("end_date")
        .and_then(|v| v.as_str())
        .ok_or_else(|| AppError::Validation("end_date must be a string".to_string()))?;
    
    let start_date = validate_date_string(start_date_str, "start_date")?;
    let end_date = validate_date_string(end_date_str, "end_date")?;
    
    validate_date_range(start_date, end_date)?;
    
    // Validate optional fields if present
    if let Some(benchmark_id) = obj.get("benchmark_id") {
        if !benchmark_id.is_string() || benchmark_id.as_str().unwrap().trim().is_empty() {
            return Err(AppError::Validation("benchmark_id must be a non-empty string".to_string()));
        }
    }
    
    if let Some(include_details) = obj.get("include_details") {
        if !include_details.is_boolean() {
            return Err(AppError::Validation("include_details must be a boolean".to_string()));
        }
    }
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    
    #[test]
    fn test_validate_calculation_params_valid() {
        let params = json!({
            "portfolio_id": "portfolio-123",
            "start_date": "2023-01-01",
            "end_date": "2023-12-31",
            "benchmark_id": "benchmark-456",
            "include_details": true
        });
        
        assert!(validate_calculation_params(&params).is_ok());
    }
    
    #[test]
    fn test_validate_calculation_params_missing_fields() {
        let params = json!({
            "portfolio_id": "portfolio-123",
            "start_date": "2023-01-01"
            // missing end_date
        });
        
        assert!(validate_calculation_params(&params).is_err());
    }
    
    #[test]
    fn test_validate_calculation_params_invalid_date_range() {
        let params = json!({
            "portfolio_id": "portfolio-123",
            "start_date": "2023-12-31",
            "end_date": "2023-01-01" // end before start
        });
        
        assert!(validate_calculation_params(&params).is_err());
    }
    
    #[test]
    fn test_validate_calculation_params_invalid_date_format() {
        let params = json!({
            "portfolio_id": "portfolio-123",
            "start_date": "01/01/2023", // wrong format
            "end_date": "2023-12-31"
        });
        
        assert!(validate_calculation_params(&params).is_err());
    }
} 