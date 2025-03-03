use chrono::{DateTime, Utc};
use std::collections::HashMap;
use std::fmt;

/// Represents validation errors that can occur during input validation
#[derive(Debug, Clone)]
pub enum ValidationError {
    EmptyCollection(&'static str),
    NegativeValue(&'static str),
    ZeroValue(&'static str),
    UnsortedDates(&'static str),
    InvalidDateRange(&'static str),
    MissingRequiredField(&'static str),
    InconsistentData(&'static str),
    Other(String),
}

impl fmt::Display for ValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ValidationError::EmptyCollection(msg) => write!(f, "Empty collection: {}", msg),
            ValidationError::NegativeValue(msg) => write!(f, "Negative value not allowed: {}", msg),
            ValidationError::ZeroValue(msg) => write!(f, "Zero value not allowed: {}", msg),
            ValidationError::UnsortedDates(msg) => write!(f, "Dates are not in chronological order: {}", msg),
            ValidationError::InvalidDateRange(msg) => write!(f, "Invalid date range: {}", msg),
            ValidationError::MissingRequiredField(msg) => write!(f, "Missing required field: {}", msg),
            ValidationError::InconsistentData(msg) => write!(f, "Inconsistent data: {}", msg),
            ValidationError::Other(msg) => write!(f, "Validation error: {}", msg),
        }
    }
}

impl std::error::Error for ValidationError {}

/// Validates that a collection is not empty
pub fn validate_non_empty<T>(collection: &[T], name: &'static str) -> Result<(), ValidationError> {
    if collection.is_empty() {
        return Err(ValidationError::EmptyCollection(name));
    }
    Ok(())
}

/// Validates that a value is not negative
pub fn validate_non_negative(value: f64, name: &'static str) -> Result<(), ValidationError> {
    if value < 0.0 {
        return Err(ValidationError::NegativeValue(name));
    }
    Ok(())
}

/// Validates that a value is positive (greater than zero)
pub fn validate_positive(value: f64, name: &'static str) -> Result<(), ValidationError> {
    if value <= 0.0 {
        return Err(ValidationError::ZeroValue(name));
    }
    Ok(())
}

/// Validates that dates are in chronological order
pub fn validate_date_order<T>(
    items: &[(DateTime<Utc>, T)],
    name: &'static str,
) -> Result<(), ValidationError> {
    for i in 1..items.len() {
        if items[i].0 <= items[i - 1].0 {
            return Err(ValidationError::UnsortedDates(name));
        }
    }
    Ok(())
}

/// Validates a date range
pub fn validate_date_range(
    start_date: DateTime<Utc>,
    end_date: DateTime<Utc>,
    name: &'static str,
) -> Result<(), ValidationError> {
    if end_date <= start_date {
        return Err(ValidationError::InvalidDateRange(name));
    }
    Ok(())
}

/// Validates that all required fields are present in a map
pub fn validate_required_fields(
    map: &HashMap<String, String>,
    required_fields: &[&str],
    name: &'static str,
) -> Result<(), ValidationError> {
    for field in required_fields {
        if !map.contains_key(*field) {
            return Err(ValidationError::MissingRequiredField(&format!("{}: {}", name, field)));
        }
    }
    Ok(())
}

/// Validates that a collection has at least a minimum number of items
pub fn validate_min_items<T>(
    collection: &[T],
    min_items: usize,
    name: &'static str,
) -> Result<(), ValidationError> {
    if collection.len() < min_items {
        return Err(ValidationError::EmptyCollection(
            &format!("{} must have at least {} items", name, min_items)
        ));
    }
    Ok(())
}

/// Validates that all values in a collection are non-negative
pub fn validate_all_non_negative(
    values: &[f64],
    name: &'static str,
) -> Result<(), ValidationError> {
    for (i, &value) in values.iter().enumerate() {
        if value < 0.0 {
            return Err(ValidationError::NegativeValue(
                &format!("{} at index {}", name, i)
            ));
        }
    }
    Ok(())
}

/// Validates that all values in a collection are positive
pub fn validate_all_positive(
    values: &[f64],
    name: &'static str,
) -> Result<(), ValidationError> {
    for (i, &value) in values.iter().enumerate() {
        if value <= 0.0 {
            return Err(ValidationError::ZeroValue(
                &format!("{} at index {}", name, i)
            ));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    #[test]
    fn test_validate_non_empty() {
        let empty: Vec<i32> = vec![];
        let non_empty = vec![1, 2, 3];

        assert!(validate_non_empty(&non_empty, "test").is_ok());
        assert!(validate_non_empty(&empty, "test").is_err());
    }

    #[test]
    fn test_validate_non_negative() {
        assert!(validate_non_negative(0.0, "test").is_ok());
        assert!(validate_non_negative(1.0, "test").is_ok());
        assert!(validate_non_negative(-1.0, "test").is_err());
    }

    #[test]
    fn test_validate_positive() {
        assert!(validate_positive(1.0, "test").is_ok());
        assert!(validate_positive(0.0, "test").is_err());
        assert!(validate_positive(-1.0, "test").is_err());
    }

    #[test]
    fn test_validate_date_order() {
        let ordered = vec![
            (Utc.ymd(2023, 1, 1).and_hms(0, 0, 0), 1),
            (Utc.ymd(2023, 1, 2).and_hms(0, 0, 0), 2),
            (Utc.ymd(2023, 1, 3).and_hms(0, 0, 0), 3),
        ];

        let unordered = vec![
            (Utc.ymd(2023, 1, 1).and_hms(0, 0, 0), 1),
            (Utc.ymd(2023, 1, 3).and_hms(0, 0, 0), 3),
            (Utc.ymd(2023, 1, 2).and_hms(0, 0, 0), 2),
        ];

        assert!(validate_date_order(&ordered, "test").is_ok());
        assert!(validate_date_order(&unordered, "test").is_err());
    }

    #[test]
    fn test_validate_date_range() {
        let start = Utc.ymd(2023, 1, 1).and_hms(0, 0, 0);
        let end = Utc.ymd(2023, 1, 2).and_hms(0, 0, 0);
        let same = Utc.ymd(2023, 1, 1).and_hms(0, 0, 0);

        assert!(validate_date_range(start, end, "test").is_ok());
        assert!(validate_date_range(end, start, "test").is_err());
        assert!(validate_date_range(start, same, "test").is_err());
    }

    #[test]
    fn test_validate_required_fields() {
        let mut map = HashMap::new();
        map.insert("field1".to_string(), "value1".to_string());
        map.insert("field2".to_string(), "value2".to_string());

        let required = &["field1", "field2"];
        let missing = &["field1", "field3"];

        assert!(validate_required_fields(&map, required, "test").is_ok());
        assert!(validate_required_fields(&map, missing, "test").is_err());
    }

    #[test]
    fn test_validate_min_items() {
        let items = vec![1, 2, 3];

        assert!(validate_min_items(&items, 3, "test").is_ok());
        assert!(validate_min_items(&items, 4, "test").is_err());
    }

    #[test]
    fn test_validate_all_non_negative() {
        let all_non_negative = vec![0.0, 1.0, 2.0];
        let some_negative = vec![1.0, -1.0, 2.0];

        assert!(validate_all_non_negative(&all_non_negative, "test").is_ok());
        assert!(validate_all_non_negative(&some_negative, "test").is_err());
    }

    #[test]
    fn test_validate_all_positive() {
        let all_positive = vec![1.0, 2.0, 3.0];
        let some_zero = vec![1.0, 0.0, 2.0];
        let some_negative = vec![1.0, -1.0, 2.0];

        assert!(validate_all_positive(&all_positive, "test").is_ok());
        assert!(validate_all_positive(&some_zero, "test").is_err());
        assert!(validate_all_positive(&some_negative, "test").is_err());
    }
} 