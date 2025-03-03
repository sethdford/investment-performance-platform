# 🧪 Testing Guide for Performance Calculator

This guide explains how to test the Performance Calculator application, with a focus on making testing accessible for engineers of all experience levels.

## 🎯 Testing Goals

Our testing strategy aims to ensure:

1. **Correctness**: Calculations produce accurate results
2. **Reliability**: The system handles errors gracefully
3. **Performance**: The system meets performance requirements
4. **Security**: The system protects sensitive data

## 📋 Types of Tests

### Unit Tests

Unit tests verify that individual functions and methods work correctly in isolation.

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;
    
    #[test]
    fn test_calculate_twr() {
        // Arrange
        let start_value = dec!(1000);
        let end_value = dec!(1100);
        let cash_flow = dec!(0);
        
        // Act
        let result = calculate_twr(start_value, end_value, cash_flow);
        
        // Assert
        assert_eq!(result, dec!(0.1)); // 10% return
    }
}
```

> 💡 **For beginners**: Unit tests follow the "Arrange-Act-Assert" pattern:
> 1. **Arrange**: Set up the test data
> 2. **Act**: Call the function you're testing
> 3. **Assert**: Check that the result matches what you expect

### Integration Tests

Integration tests verify that different components work together correctly.

```rust
#[tokio::test]
async fn test_performance_calculation_flow() {
    // Arrange
    let repo = MockRepository::new();
    let calculator = PerformanceCalculator::new(repo);
    
    // Act
    let result = calculator
        .calculate_portfolio_performance("portfolio-123", "2023-01-01", "2023-12-31")
        .await;
    
    // Assert
    assert!(result.is_ok());
    let performance = result.unwrap();
    assert!(performance.twr > dec!(0));
    assert!(performance.mwr > dec!(0));
}
```

> 💡 **What's different**: Integration tests often involve multiple components and may require async testing with Tokio.

### Property-Based Tests

Property-based tests generate random inputs to verify that certain properties hold true for all inputs.

```rust
#[test]
fn test_twr_properties() {
    // Property: If start_value = end_value and cash_flow = 0, TWR should be 0
    proptest!(|(start_value in 1.0..100000.0)| {
        let start = Decimal::from_f64(start_value).unwrap();
        let end = start;
        let cash_flow = dec!(0);
        
        let result = calculate_twr(start, end, cash_flow);
        prop_assert_eq!(result, dec!(0));
    });
    
    // Property: If end_value = 2 * start_value and cash_flow = 0, TWR should be 1 (100%)
    proptest!(|(start_value in 1.0..100000.0)| {
        let start = Decimal::from_f64(start_value).unwrap();
        let end = start * dec!(2);
        let cash_flow = dec!(0);
        
        let result = calculate_twr(start, end, cash_flow);
        prop_assert_eq!(result, dec!(1));
    });
}
```

> 💡 **Why property-based testing**: Instead of testing specific examples, property-based testing verifies that mathematical properties hold true across a wide range of inputs.

### Performance Tests

Performance tests verify that the system meets performance requirements.

```rust
#[tokio::test]
async fn test_calculation_performance() {
    // Arrange
    let repo = MockRepository::new_with_large_dataset();
    let calculator = PerformanceCalculator::new(repo);
    
    // Act
    let start = Instant::now();
    let result = calculator
        .calculate_portfolio_performance("large-portfolio", "2023-01-01", "2023-12-31")
        .await;
    let duration = start.elapsed();
    
    // Assert
    assert!(result.is_ok());
    assert!(duration < Duration::from_secs(1)); // Should complete in under 1 second
}
```

> 💡 **Performance testing tip**: Set realistic thresholds based on your requirements, and run performance tests in a controlled environment to get consistent results.

## 🛠️ Testing Tools

### Rust Testing Framework

Rust has a built-in testing framework that you can use with `cargo test`:

```bash
# Run all tests
cargo test

# Run tests with a specific name
cargo test calculate_twr

# Run tests with output
cargo test -- --nocapture
```

### Mocking

For testing components that depend on external services, we 