use crate::common::error::{ApiError, ApiResult};

/// Validate that a value is not empty
pub fn validate_not_empty<T: AsRef<str>>(value: T, field_name: &str) -> ApiResult<()> {
    if value.as_ref().trim().is_empty() {
        return Err(ApiError::Validation(format!("{} cannot be empty", field_name)));
    }
    Ok(())
}

/// Validate that a value is within a range
pub fn validate_range<T: PartialOrd + std::fmt::Display>(
    value: T,
    min: T,
    max: T,
    field_name: &str,
) -> ApiResult<()> {
    if value < min || value > max {
        return Err(ApiError::Validation(format!(
            "{} must be between {} and {}, got {}",
            field_name, min, max, value
        )));
    }
    Ok(())
}

/// Validate that a value is greater than a minimum
pub fn validate_min<T: PartialOrd + std::fmt::Display>(
    value: T,
    min: T,
    field_name: &str,
) -> ApiResult<()> {
    if value < min {
        return Err(ApiError::Validation(format!("{} must be at least {}, got {}", field_name, min, value)));
    }
    Ok(())
}

/// Validate that a value is less than a maximum
pub fn validate_max<T: PartialOrd + std::fmt::Display>(
    value: T,
    max: T,
    field_name: &str,
) -> ApiResult<()> {
    if value > max {
        return Err(ApiError::Validation(format!("{} must be at most {}, got {}", field_name, max, value)));
    }
    Ok(())
}

/// Validate that a string matches a regex pattern
pub fn validate_pattern(value: &str, pattern: &str, field_name: &str) -> ApiResult<()> {
    let regex = regex::Regex::new(pattern).map_err(|e| ApiError::Internal(
        format!("Invalid regex pattern: {}", e)
    ))?;

    if !regex.is_match(value) {
        return Err(ApiError::Validation(format!("{} does not match the required pattern", field_name)));
    }
    Ok(())
}

/// Validate that a collection has a minimum length
pub fn validate_min_length<T>(
    collection: &[T],
    min_length: usize,
    field_name: &str,
) -> ApiResult<()> {
    if collection.len() < min_length {
        return Err(ApiError::Validation(format!(
            "{} must have at least {} items, got {}",
            field_name,
            min_length,
            collection.len()
        )));
    }
    Ok(())
}

/// Validate that a collection has a maximum length
pub fn validate_max_length<T>(
    collection: &[T],
    max_length: usize,
    field_name: &str,
) -> ApiResult<()> {
    if collection.len() > max_length {
        return Err(ApiError::Validation(format!(
            "{} must have at most {} items, got {}",
            field_name,
            max_length,
            collection.len()
        )));
    }
    Ok(())
}

/// Validate that a string has a minimum length
pub fn validate_min_string_length(
    value: &str,
    min_length: usize,
    field_name: &str,
) -> ApiResult<()> {
    if value.len() < min_length {
        return Err(ApiError::Validation(format!(
            "{} must have at least {} characters, got {}",
            field_name,
            min_length,
            value.len()
        )));
    }
    Ok(())
}

/// Validate that a string has a maximum length
pub fn validate_max_string_length(
    value: &str,
    max_length: usize,
    field_name: &str,
) -> ApiResult<()> {
    if value.len() > max_length {
        return Err(ApiError::Validation(format!(
            "{} must have at most {} characters, got {}",
            field_name,
            max_length,
            value.len()
        )));
    }
    Ok(())
}

/// Validate that a value is one of a set of allowed values
pub fn validate_allowed_values<T: PartialEq + std::fmt::Debug>(
    value: &T,
    allowed_values: &[T],
    field_name: &str,
) -> ApiResult<()> {
    if !allowed_values.contains(value) {
        return Err(ApiError::Validation(format!(
            "{} must be one of {:?}, got {:?}",
            field_name, allowed_values, value
        )));
    }
    Ok(())
}

/// Validate that a collection of weights sums to 1.0 (within a small epsilon)
pub fn validate_weights_sum_to_one(
    weights: &[f64],
    field_name: &str,
    epsilon: Option<f64>,
) -> ApiResult<()> {
    let epsilon = epsilon.unwrap_or(0.0001);
    let sum: f64 = weights.iter().sum();
    if (sum - 1.0).abs() > epsilon {
        return Err(ApiError::Validation(format!(
            "{} must sum to 1.0, got {} (difference: {})",
            field_name,
            sum,
            (sum - 1.0).abs()
        )));
    }
    Ok(())
} 