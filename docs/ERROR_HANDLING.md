# Error Handling Best Practices

This document outlines the best practices for error handling in the Investment Management Platform.

## Table of Contents

1. [Error Types](#error-types)
2. [Error Handling Patterns](#error-handling-patterns)
3. [Validation](#validation)
4. [Logging Errors](#logging-errors)
5. [Error Propagation](#error-propagation)
6. [User-Facing Errors](#user-facing-errors)
7. [Examples](#examples)

## Error Types

The Investment Management Platform uses a structured approach to error handling with several error types:

### Error / ApiError

The `Error` enum in `src/common/error/mod.rs` is the main error type for the platform. It includes various error variants for different scenarios:

- `Api`: API-related errors
- `Database`: Database-related errors
- `Validation`: Input validation errors
- `NotFound`: Entity not found errors
- `Authentication`: Authentication errors
- `Authorization`: Authorization errors
- `ExternalService`: External service errors
- `Internal`: Internal server errors
- `Serialization`: Serialization/deserialization errors
- `Io`: IO errors
- `ModelPortfolioError`: Model Portfolio specific errors
- `AwsSdk`: AWS SDK errors
- `Json`: JSON parsing errors
- `Utf8`: UTF-8 conversion errors
- `Other`: Other errors

For backward compatibility, `ApiError` is a type alias for `Error`.

### Module-Specific Error Types

Each module can define its own error types for specific scenarios:

- `ModelPortfolioErrorType`: Types of errors specific to model portfolios
  - `SleeveCreationError`: Error when creating sleeves from a model
  - `ChildModelNotFound`: Error when a child model is not found
  - `InvalidWeights`: Error when model weights don't sum to 1.0
  - `InvalidModelStructure`: Error when a model has an invalid structure
  - `SecurityNotFound`: Error when a security is not found
  - `DuplicateSecurities`: Error when a model has duplicate securities
  - `InvalidAllocation`: Error when a model has invalid allocation

## Error Handling Patterns

The platform provides several utilities for consistent error handling patterns:

### ResultExt Trait

The `ResultExt` trait extends `Result` with methods for converting errors to `ApiError`:

```rust
use investment_management::common::ResultExt;

fn process_data(data: &str) -> ApiResult<()> {
    // Convert any error to an ApiError with a custom message
    parse_data(data).with_context(|| "Failed to parse data")
}
```

Available methods:

- `with_context`: Convert the error to an ApiError with a custom message
- `with_error_type`: Convert the error to an ApiError with the error type as the message
- `not_found`: Convert the error to a NotFound ApiError
- `validation_error`: Convert the error to a Validation ApiError
- `invalid_parameter`: Convert the error to a Validation ApiError with parameter information

### OptionExt Trait

The `OptionExt` trait extends `Option` with methods for converting `None` to `ApiError`:

```rust
use investment_management::common::OptionExt;

fn get_user(id: &str) -> ApiResult<User> {
    // Convert None to a NotFound ApiError
    find_user(id).ok_or_not_found("User", id)
}
```

Available methods:

- `ok_or_not_found`: Convert None to a NotFound ApiError
- `ok_or_validation_error`: Convert None to a Validation ApiError
- `ok_or_invalid_parameter`: Convert None to a Validation ApiError with parameter information

### Helper Functions

The platform provides helper functions for creating common errors:

```rust
use investment_management::common::error::Error;

fn process_request(id: &str, param: &str) -> ApiResult<()> {
    if id.is_empty() {
        return Err(Error::validation("ID cannot be empty"));
    }
    
    if param.len() < 3 {
        return Err(Error::validation(format!("Parameter {} must be at least 3 characters", param)));
    }
    
    let entity = find_entity(id)?;
    if entity.is_none() {
        return Err(Error::not_found(format!("Entity with ID {} not found", id)));
    }
    
    // Process the request
    Ok(())
}
```

## Validation

The platform provides utilities for validating input data:

```rust
use investment_management::common::error_utils;

fn validate_portfolio(portfolio: &Portfolio) -> ApiResult<()> {
    // Validate that the name is not empty
    error_utils::validate_not_empty(&portfolio.name, "name")?;
    
    // Validate that the allocation is within range
    error_utils::validate_range(portfolio.allocation, 0.0, 1.0, "allocation")?;
    
    // Validate that the weights sum to 1.0
    error_utils::validate_weights_sum_to_one(&portfolio.weights, "weights", None)?;
    
    Ok(())
}
```

Available validation functions:

- `validate_not_empty`: Validate that a value is not empty
- `validate_range`: Validate that a value is within a range
- `validate_min`: Validate that a value is greater than a minimum
- `validate_max`: Validate that a value is less than a maximum
- `validate_pattern`: Validate that a string matches a regex pattern
- `validate_min_length`: Validate that a collection has a minimum length
- `validate_max_length`: Validate that a collection has a maximum length
- `validate_min_string_length`: Validate that a string has a minimum length
- `validate_max_string_length`: Validate that a string has a maximum length
- `validate_allowed_values`: Validate that a value is one of a set of allowed values
- `validate_weights_sum_to_one`: Validate that a collection of weights sums to 1.0

## Logging Errors

Always log errors with appropriate context:

```rust
use investment_management::{log_error, log_warning};

fn process_request(request: &Request) -> ApiResult<Response> {
    match do_something(request) {
        Ok(response) => Ok(response),
        Err(err) => {
            log_error!("process_request", err, request_id = request.id);
            Err(Error::internal("Failed to process request"))
        }
    }
}
```

Use the appropriate logging level:

- `log_error!`: For errors that require immediate attention
- `log_warning!`: For warnings that might indicate a problem
- `log_info!`: For informational messages
- `log_method_entry!`: For logging method entry with parameters
- `log_method_exit!`: For logging method exit with result

## Error Propagation

Use the `?` operator to propagate errors:

```rust
fn process_request(request: &Request) -> ApiResult<Response> {
    let data = parse_request(request)?;
    let result = process_data(&data)?;
    let response = create_response(result)?;
    Ok(response)
}
```

When you need to convert from one error type to another, use the `map_err` method or the `ResultExt` trait:

```rust
fn process_request(request: &Request) -> ApiResult<Response> {
    let data = parse_request(request)
        .map_err(|e| Error::validation(format!("Invalid request: {}", e)))?;
    
    let result = process_data(&data)
        .with_context(|| format!("Failed to process data for request {}", request.id))?;
    
    let response = create_response(result)?;
    Ok(response)
}
```

## User-Facing Errors

When returning errors to users, ensure they are user-friendly and actionable:

```rust
fn validate_input(input: &Input) -> ApiResult<()> {
    if input.name.is_empty() {
        return Err(Error::validation("Name is required"));
    }
    
    if input.email.is_empty() {
        return Err(Error::validation("Email is required"));
    }
    
    if !input.email.contains('@') {
        return Err(Error::validation("Email must be a valid email address"));
    }
    
    Ok(())
}
```

For internal errors that should not be exposed to users, log the detailed error but return a generic message:

```rust
fn process_payment(payment: &Payment) -> ApiResult<Receipt> {
    match payment_processor.process(payment) {
        Ok(receipt) => Ok(receipt),
        Err(err) => {
            log_error!("process_payment", err, payment_id = payment.id);
            Err(Error::internal("Payment processing failed. Please try again later."))
        }
    }
}
```

## Examples

### Example 1: Validating Input

```rust
use investment_management::common::error_utils;
use investment_management::common::ApiResult;

fn create_portfolio(name: &str, allocation: f64, weights: &[f64]) -> ApiResult<Portfolio> {
    // Validate input
    error_utils::validate_not_empty(name, "name")?;
    error_utils::validate_range(allocation, 0.0, 1.0, "allocation")?;
    error_utils::validate_weights_sum_to_one(weights, "weights", None)?;
    
    // Create portfolio
    let portfolio = Portfolio {
        name: name.to_string(),
        allocation,
        weights: weights.to_vec(),
    };
    
    Ok(portfolio)
}
```

### Example 2: Error Propagation with Context

```rust
use investment_management::common::ResultExt;
use investment_management::common::ApiResult;

fn process_portfolio(portfolio_id: &str) -> ApiResult<PortfolioAnalysis> {
    // Get portfolio
    let portfolio = get_portfolio(portfolio_id)
        .with_context(|| format!("Failed to get portfolio {}", portfolio_id))?;
    
    // Calculate metrics
    let metrics = calculate_metrics(&portfolio)
        .with_context(|| format!("Failed to calculate metrics for portfolio {}", portfolio_id))?;
    
    // Generate analysis
    let analysis = generate_analysis(&portfolio, &metrics)
        .with_context(|| format!("Failed to generate analysis for portfolio {}", portfolio_id))?;
    
    Ok(analysis)
}
```

### Example 3: Handling Optional Values

```rust
use investment_management::common::OptionExt;
use investment_management::common::ApiResult;

fn get_household_member(household_id: &str, member_id: &str) -> ApiResult<Member> {
    // Get household
    let household = get_household(household_id)
        .ok_or_not_found("Household", household_id)?;
    
    // Get member
    let member = household.members.iter()
        .find(|m| m.id == member_id)
        .ok_or_not_found("Member", member_id)?;
    
    Ok(member.clone())
}
```

### Example 4: Logging Errors

```rust
use investment_management::{log_error, log_method_entry, log_method_exit};
use investment_management::common::{ApiResult, Error};

fn process_transaction(transaction: &Transaction) -> ApiResult<Receipt> {
    log_method_entry!("process_transaction", transaction_id = transaction.id);
    
    let result = match execute_transaction(transaction) {
        Ok(receipt) => Ok(receipt),
        Err(err) => {
            log_error!("process_transaction", err, transaction_id = transaction.id);
            Err(Error::internal("Transaction processing failed"))
        }
    };
    
    log_method_exit!("process_transaction", &result);
    result
}
``` 