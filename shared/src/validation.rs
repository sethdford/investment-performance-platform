use chrono::{NaiveDate, Utc};
use std::fmt;
use regex::Regex;

use crate::models::{
    Account, Benchmark, Client, Portfolio, Position, Price,
    Security, Transaction, TransactionType,
};

/// Represents validation errors that can occur during input validation
#[derive(Debug, Clone)]
pub enum ValidationError {
    EmptyString(&'static str),
    InvalidFormat(&'static str),
    NegativeValue(&'static str),
    ZeroValue(&'static str),
    InvalidDateRange(&'static str),
    MissingRequiredField(&'static str),
    InvalidReference(&'static str),
    InvalidEnumValue(&'static str),
    InconsistentData(&'static str),
    Other(String),
}

impl fmt::Display for ValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ValidationError::EmptyString(msg) => write!(f, "Empty string not allowed: {}", msg),
            ValidationError::InvalidFormat(msg) => write!(f, "Invalid format: {}", msg),
            ValidationError::NegativeValue(msg) => write!(f, "Negative value not allowed: {}", msg),
            ValidationError::ZeroValue(msg) => write!(f, "Zero value not allowed: {}", msg),
            ValidationError::InvalidDateRange(msg) => write!(f, "Invalid date range: {}", msg),
            ValidationError::MissingRequiredField(msg) => {
                write!(f, "Missing required field: {}", msg)
            }
            ValidationError::InvalidReference(msg) => write!(f, "Invalid reference: {}", msg),
            ValidationError::InvalidEnumValue(msg) => write!(f, "Invalid enum value: {}", msg),
            ValidationError::InconsistentData(msg) => write!(f, "Inconsistent data: {}", msg),
            ValidationError::Other(msg) => write!(f, "Validation error: {}", msg),
        }
    }
}

impl std::error::Error for ValidationError {}

/// Trait for validating data models
pub trait Validate {
    /// Validates the model and returns a Result
    fn validate(&self) -> Result<(), ValidationError>;
}

/// Validates that a string is not empty
pub fn validate_non_empty_string(value: &str, field_name: &'static str) -> Result<(), ValidationError> {
    if value.trim().is_empty() {
        return Err(ValidationError::EmptyString(field_name));
    }
    Ok(())
}

/// Validates that a value is not negative
pub fn validate_non_negative(value: f64, field_name: &'static str) -> Result<(), ValidationError> {
    if value < 0.0 {
        return Err(ValidationError::NegativeValue(field_name));
    }
    Ok(())
}

/// Validates that a value is positive (greater than zero)
pub fn validate_positive(value: f64, field_name: &'static str) -> Result<(), ValidationError> {
    if value <= 0.0 {
        return Err(ValidationError::ZeroValue(field_name));
    }
    Ok(())
}

/// Validates that a date is not in the future
pub fn validate_not_future_date(
    date: NaiveDate,
    _field_name: &'static str,
) -> Result<(), ValidationError> {
    let today = Utc::now().date_naive();
    if date > today {
        return Err(ValidationError::InvalidDateRange(
            "Date cannot be in the future",
        ));
    }
    Ok(())
}

/// Validates a date range
pub fn validate_date_range(
    start_date: NaiveDate,
    end_date: NaiveDate,
    field_name: &'static str,
) -> Result<(), ValidationError> {
    if end_date < start_date {
        return Err(ValidationError::InvalidDateRange(field_name));
    }
    Ok(())
}

/// Validates a currency code (ISO 4217)
pub fn validate_currency_code(code: &str, _field_name: &'static str) -> Result<(), ValidationError> {
    // ISO 4217 currency codes are 3 uppercase letters
    let re = Regex::new(r"^[A-Z]{3}$").unwrap();
    if !re.is_match(code) {
        return Err(ValidationError::InvalidFormat("Currency code must be 3 uppercase letters"));
    }
    Ok(())
}

/// Validates an email address
pub fn validate_email(email: &str, _field_name: &'static str) -> Result<(), ValidationError> {
    // Simple email validation
    let re = Regex::new(r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$").unwrap();
    if !re.is_match(email) {
        return Err(ValidationError::InvalidFormat("Invalid email format"));
    }
    Ok(())
}

/// Implementation of Validate for Portfolio
impl Validate for Portfolio {
    fn validate(&self) -> Result<(), ValidationError> {
        // Validate required string fields
        validate_non_empty_string(&self.id, "portfolio.id")?;
        validate_non_empty_string(&self.name, "portfolio.name")?;
        validate_non_empty_string(&self.client_id, "portfolio.client_id")?;

        // Validate inception date is not in the future
        validate_not_future_date(self.inception_date, "portfolio.inception_date")?;

        // Validate benchmark_id if present
        if let Some(ref benchmark_id) = self.benchmark_id {
            validate_non_empty_string(benchmark_id, "portfolio.benchmark_id")?;
        }

        // Additional validations can be added here
        // For example, validating that created_at is not after updated_at
        if self.updated_at < self.created_at {
            return Err(ValidationError::InvalidDateRange(
                "portfolio.updated_at cannot be before portfolio.created_at",
            ));
        }

        Ok(())
    }
}

/// Implementation of Validate for Transaction
impl Validate for Transaction {
    fn validate(&self) -> Result<(), ValidationError> {
        // Validate required string fields
        validate_non_empty_string(&self.id, "transaction.id")?;
        validate_non_empty_string(&self.account_id, "transaction.account_id")?;
        
        // Validate security_id if present
        if let Some(ref security_id) = self.security_id {
            validate_non_empty_string(security_id, "transaction.security_id")?;
        }

        // Validate dates
        validate_not_future_date(self.transaction_date, "transaction.transaction_date")?;
        
        if let Some(settlement_date) = self.settlement_date {
            validate_not_future_date(settlement_date, "transaction.settlement_date")?;
            
            // Settlement date should not be before transaction date
            if settlement_date < self.transaction_date {
                return Err(ValidationError::InvalidDateRange(
                    "transaction.settlement_date cannot be before transaction.transaction_date",
                ));
            }
        }

        // Validate amount based on transaction type
        match self.transaction_type {
            TransactionType::Buy | TransactionType::Sell => {
                // For buy/sell, we need quantity and price
                if self.quantity.is_none() {
                    return Err(ValidationError::MissingRequiredField(
                        "transaction.quantity is required for Buy/Sell transactions",
                    ));
                }
                
                if self.price.is_none() {
                    return Err(ValidationError::MissingRequiredField(
                        "transaction.price is required for Buy/Sell transactions",
                    ));
                }
                
                // Security ID is required for buy/sell
                if self.security_id.is_none() {
                    return Err(ValidationError::MissingRequiredField(
                        "transaction.security_id is required for Buy/Sell transactions",
                    ));
                }
                
                // Validate quantity is positive
                if let Some(quantity) = self.quantity {
                    validate_positive(quantity, "transaction.quantity")?;
                }
                
                // Validate price is positive
                if let Some(price) = self.price {
                    validate_positive(price, "transaction.price")?;
                }
            },
            TransactionType::Deposit | TransactionType::Withdrawal => {
                // For deposits/withdrawals, amount should be positive
                validate_positive(self.amount, "transaction.amount")?;
            },
            TransactionType::Dividend | TransactionType::Interest => {
                // For dividends/interest, amount should be positive and security_id should be present
                validate_positive(self.amount, "transaction.amount")?;
                
                if self.security_id.is_none() {
                    return Err(ValidationError::MissingRequiredField(
                        "transaction.security_id is required for Dividend/Interest transactions",
                    ));
                }
            },
            TransactionType::Fee => {
                // For fees, amount should be positive
                validate_positive(self.amount, "transaction.amount")?;
            },
            TransactionType::Transfer => {
                // For transfers, amount should be non-zero
                if self.amount == 0.0 {
                    return Err(ValidationError::ZeroValue("transaction.amount"));
                }
            },
            TransactionType::Split => {
                // For splits, quantity and security_id are required
                if self.quantity.is_none() {
                    return Err(ValidationError::MissingRequiredField(
                        "transaction.quantity is required for Split transactions",
                    ));
                }
                
                if self.security_id.is_none() {
                    return Err(ValidationError::MissingRequiredField(
                        "transaction.security_id is required for Split transactions",
                    ));
                }
            },
            TransactionType::Other(_) => {
                // For other transaction types, just ensure amount is non-zero
                if self.amount == 0.0 {
                    return Err(ValidationError::ZeroValue("transaction.amount"));
                }
            },
        }

        // Validate currency code
        validate_currency_code(&self.currency, "transaction.currency")?;

        // Validate fees if present
        if let Some(fees) = self.fees {
            validate_non_negative(fees, "transaction.fees")?;
        }

        // Validate timestamps
        if self.updated_at < self.created_at {
            return Err(ValidationError::InvalidDateRange(
                "transaction.updated_at cannot be before transaction.created_at",
            ));
        }

        Ok(())
    }
}

/// Implementation of Validate for Account
impl Validate for Account {
    fn validate(&self) -> Result<(), ValidationError> {
        // Validate required string fields
        validate_non_empty_string(&self.id, "account.id")?;
        validate_non_empty_string(&self.account_number, "account.account_number")?;
        validate_non_empty_string(&self.name, "account.name")?;
        validate_non_empty_string(&self.portfolio_id, "account.portfolio_id")?;
        
        // Validate inception date is not in the future
        validate_not_future_date(self.inception_date, "account.inception_date")?;
        
        // Validate timestamps
        if self.updated_at < self.created_at {
            return Err(ValidationError::InvalidDateRange(
                "account.updated_at cannot be before account.created_at",
            ));
        }
        
        Ok(())
    }
}

/// Implementation of Validate for Security
impl Validate for Security {
    fn validate(&self) -> Result<(), ValidationError> {
        // Validate required string fields
        validate_non_empty_string(&self.id, "security.id")?;
        validate_non_empty_string(&self.symbol, "security.symbol")?;
        validate_non_empty_string(&self.name, "security.name")?;
        
        // Validate identifiers if present
        if let Some(ref cusip) = self.cusip {
            // CUSIP is 9 characters
            if cusip.len() != 9 {
                return Err(ValidationError::InvalidFormat(
                    "security.cusip must be 9 characters",
                ));
            }
        }
        
        if let Some(ref isin) = self.isin {
            // ISIN is 12 characters
            if isin.len() != 12 {
                return Err(ValidationError::InvalidFormat(
                    "security.isin must be 12 characters",
                ));
            }
        }
        
        if let Some(ref sedol) = self.sedol {
            // SEDOL is 7 characters
            if sedol.len() != 7 {
                return Err(ValidationError::InvalidFormat(
                    "security.sedol must be 7 characters",
                ));
            }
        }
        
        // Validate timestamps
        if self.updated_at < self.created_at {
            return Err(ValidationError::InvalidDateRange(
                "security.updated_at cannot be before security.created_at",
            ));
        }
        
        Ok(())
    }
}

/// Implementation of Validate for Price
impl Validate for Price {
    fn validate(&self) -> Result<(), ValidationError> {
        // Validate required string fields
        validate_non_empty_string(&self.security_id, "price.security_id")?;
        validate_non_empty_string(&self.source, "price.source")?;
        
        // Validate price is positive
        validate_positive(self.price, "price.price")?;
        
        // Validate currency code
        validate_currency_code(&self.currency, "price.currency")?;
        
        // Validate date is not in the future
        validate_not_future_date(self.date, "price.date")?;
        
        Ok(())
    }
}

/// Implementation of Validate for Position
impl Validate for Position {
    fn validate(&self) -> Result<(), ValidationError> {
        // Validate required string fields
        validate_non_empty_string(&self.account_id, "position.account_id")?;
        validate_non_empty_string(&self.security_id, "position.security_id")?;
        
        // Validate numeric fields
        validate_non_negative(self.quantity, "position.quantity")?;
        validate_non_negative(self.market_value, "position.market_value")?;
        
        // Validate cost basis if present
        if let Some(cost_basis) = self.cost_basis {
            validate_non_negative(cost_basis, "position.cost_basis")?;
        }
        
        // Validate currency code
        validate_currency_code(&self.currency, "position.currency")?;
        
        // Validate date is not in the future
        validate_not_future_date(self.date, "position.date")?;
        
        // Validate timestamps
        if self.updated_at < self.created_at {
            return Err(ValidationError::InvalidDateRange(
                "position.updated_at cannot be before position.created_at",
            ));
        }
        
        Ok(())
    }
}

/// Implementation of Validate for Client
impl Validate for Client {
    fn validate(&self) -> Result<(), ValidationError> {
        // Validate required string fields
        validate_non_empty_string(&self.id, "client.id")?;
        validate_non_empty_string(&self.name, "client.name")?;
        validate_non_empty_string(&self.classification, "client.classification")?;
        
        // Validate contact information if present
        if let Some(ref email) = self.contact.email {
            validate_email(email, "client.contact.email")?;
        }
        
        // Validate timestamps
        if self.updated_at < self.created_at {
            return Err(ValidationError::InvalidDateRange(
                "client.updated_at cannot be before client.created_at",
            ));
        }
        
        Ok(())
    }
}

/// Implementation of Validate for Benchmark
impl Validate for Benchmark {
    fn validate(&self) -> Result<(), ValidationError> {
        // Validate required string fields
        validate_non_empty_string(&self.id, "benchmark.id")?;
        validate_non_empty_string(&self.name, "benchmark.name")?;
        
        // Validate symbol if present
        if let Some(ref symbol) = self.symbol {
            validate_non_empty_string(symbol, "benchmark.symbol")?;
        }
        
        // Validate timestamps
        if self.updated_at < self.created_at {
            return Err(ValidationError::InvalidDateRange(
                "benchmark.updated_at cannot be before benchmark.created_at",
            ));
        }
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_non_empty_string() {
        assert!(validate_non_empty_string("test", "field").is_ok());
        assert!(validate_non_empty_string("", "field").is_err());
    }
    
    #[test]
    fn test_validate_non_negative() {
        assert!(validate_non_negative(0.0, "field").is_ok());
        assert!(validate_non_negative(1.0, "field").is_ok());
        assert!(validate_non_negative(-1.0, "field").is_err());
    }
    
    #[test]
    fn test_validate_positive() {
        assert!(validate_positive(1.0, "field").is_ok());
        assert!(validate_positive(0.0, "field").is_err());
        assert!(validate_positive(-1.0, "field").is_err());
    }
    
    #[test]
    fn test_validate_currency_code() {
        assert!(validate_currency_code("USD", "field").is_ok());
        assert!(validate_currency_code("EUR", "field").is_ok());
        assert!(validate_currency_code("JPY", "field").is_ok());
        assert!(validate_currency_code("usd", "field").is_err());
        assert!(validate_currency_code("US", "field").is_err());
        assert!(validate_currency_code("USDD", "field").is_err());
    }
    
    #[test]
    fn test_validate_email() {
        assert!(validate_email("test@example.com", "field").is_ok());
        assert!(validate_email("test.name@example.co.uk", "field").is_ok());
        assert!(validate_email("test", "field").is_err());
        assert!(validate_email("test@", "field").is_err());
        assert!(validate_email("@example.com", "field").is_err());
    }
    
    // Add more tests for the Validate implementations
} 