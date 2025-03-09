# Testing Framework

This document provides a comprehensive overview of the testing framework for the Modern Conversational Financial Advisor platform, including test categories, implementation details, and guidelines for creating and running tests.

## Overview

Our testing approach follows these principles:

1. **Test-Driven Development**: Write tests before implementing features
2. **Comprehensive Coverage**: Test all aspects of functionality
3. **Regression Prevention**: Ensure changes don't break existing functionality
4. **Performance Validation**: Verify system performance meets requirements
5. **Compliance Verification**: Ensure adherence to regulatory requirements

## Test Categories

1. **Unit Tests**: Validate individual components in isolation
2. **Integration Tests**: Verify interactions between components
3. **End-to-End Tests**: Test complete user flows
4. **Performance Tests**: Measure system performance under load
5. **Compliance Tests**: Verify adherence to regulatory requirements

## Directory Structure

```
tests/
├── data/                           # Test data files
│   ├── client_profiles/            # Sample client profiles
│   ├── market_data/                # Historical and simulated market data
│   ├── financial_products/         # Financial product information
│   └── conversation_logs/          # Sample conversation logs
├── mocks/                          # Mock implementations
│   ├── bedrock.rs                  # Mock AWS Bedrock client
│   └── ...                         # Other mock implementations
├── test_cases/                     # Test case documentation
│   ├── bedrock_integration_test.md # AWS Bedrock integration test
│   ├── retirement_planning_test.md # Retirement planning model test
│   ├── tax_optimization_test.md    # Tax optimization test
│   └── ...                         # Other test case documentation
└── ...                             # Test implementation files
```

## Implemented Components

### 1. Test Data

We've created a comprehensive set of test data to support various testing scenarios:

- **Client Profiles**: Sample client profiles with different ages, risk tolerances, financial goals, assets, liabilities, income, and expenses.
  - Location: `tests/data/client_profiles/sample_profiles.json`
  - Contains diverse profiles (moderate, conservative, and aggressive risk tolerance)

- **Market Data**: Historical returns for different asset classes, correlation matrices, and efficient frontier portfolios.
  - Location: `tests/data/market_data/historical_returns.json`
  - Includes data for US and international stocks, bonds, and other asset classes

- **Financial Products**: Catalog of investment vehicles including ETFs, mutual funds, target date funds, and tax-advantaged accounts.
  - Location: `tests/data/financial_products/investment_vehicles.json`
  - Contains detailed information about expense ratios, holdings, performance, etc.

- **Conversation Logs**: Sample conversation logs for testing NLP components.
  - Location: `tests/data/conversation_logs/retirement_planning.json`
  - Includes multi-turn conversations about retirement planning

### 2. Mock Implementations

We've created mock implementations to facilitate testing without external dependencies:

- **Mock Bedrock Client**: Simulates AWS Bedrock API responses
- **Mock Market Data Provider**: Provides simulated market data
- **Mock Portfolio Manager**: Simulates portfolio management operations
- **Mock Tax Calculator**: Simulates tax calculations

## Running Tests

### Prerequisites

- Rust toolchain (latest stable version)
- AWS credentials with Bedrock access (for integration tests)
- Cargo test runner

### Basic Test Commands

```bash
# Run all tests
cargo test

# Run tests for a specific component
cargo test -p performance-calculator

# Run a specific test
cargo test tax_optimization

# Run tests with verbose output
cargo test -- --nocapture
```

### Running Integration Tests

Integration tests require additional setup:

```bash
# Set up AWS credentials
export AWS_ACCESS_KEY_ID=your_access_key
export AWS_SECRET_ACCESS_KEY=your_secret_key
export AWS_REGION=us-east-1

# Run integration tests
cargo test --features integration-tests
```

## Writing Tests

### Test Case Template

Use the following template to create test cases:

```markdown
# Test Case Information

**Test ID**: [Unique identifier, e.g., TC-1.1.1]  
**Related Feature**: [Reference to the feature being tested]  
**Priority**: [High/Medium/Low]  
**Type**: [Unit/Integration/End-to-End/Performance/Compliance]  
**Created By**: [Your name]  
**Created Date**: [Date]  

## Test Objective

[Clearly state what aspect of functionality this test is validating]

## Prerequisites

- [List any prerequisites or setup required before running the test]
- [Include any specific data that needs to be loaded]
- [Mention any environment configuration needed]

## Test Data

| Input | Expected Output | Notes |
|-------|----------------|-------|
| [Input value 1] | [Expected result 1] | [Any additional notes] |
| [Input value 2] | [Expected result 2] | [Any additional notes] |

## Test Steps

1. [Step 1]
2. [Step 2]
3. [Step 3]

## Validation Criteria

- [Criterion 1]
- [Criterion 2]
- [Criterion 3]

## Notes and Issues

[Any additional notes or known issues]
```

### Example Test Implementation

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::setup_test_data;

    #[test]
    fn test_tax_loss_harvesting() {
        // Arrange
        let test_data = setup_test_data();
        let portfolio = test_data.get_portfolio("moderate_risk");
        let tax_optimizer = TaxOptimizer::new();

        // Act
        let harvest_opportunities = tax_optimizer.find_harvest_opportunities(
            portfolio,
            30.0, // Tax rate
            3000.0, // Max deduction
        );

        // Assert
        assert!(!harvest_opportunities.is_empty());
        assert_eq!(harvest_opportunities.len(), 3);
        assert!(harvest_opportunities[0].tax_savings > 0.0);
    }
}
```

## Test Coverage

We aim for high test coverage across all components:

- **Core Business Logic**: 90%+ coverage
- **API Layer**: 80%+ coverage
- **Utility Functions**: 70%+ coverage
- **UI Components**: 60%+ coverage

To check test coverage:

```bash
cargo tarpaulin --out Html
```

## Continuous Integration

Tests are automatically run in our CI pipeline:

1. **Pull Request**: All tests are run when a PR is created or updated
2. **Main Branch**: All tests are run when changes are merged to main
3. **Nightly**: Performance and integration tests are run nightly

## Module-Specific Testing

### Performance Calculator Testing

The performance calculator module has specific testing requirements:

- **Accuracy Tests**: Verify calculation accuracy against known results
- **Benchmark Tests**: Measure calculation performance
- **Edge Case Tests**: Test behavior with extreme inputs

See [performance-calculator/TESTING.md](../performance-calculator/TESTING.md) for details.

### Conversation System Testing

The conversation system has specific testing requirements:

- **Conversation Flow Tests**: Verify conversation management
- **Storage Tests**: Verify conversation persistence
- **Summarization Tests**: Verify summary generation

See [docs/CONVERSATION_SYSTEM.md](CONVERSATION_SYSTEM.md#testing-approach) for details.

## Test Automation

We use several tools to automate testing:

- **Cargo Test**: For running Rust tests
- **Tarpaulin**: For measuring test coverage
- **GitHub Actions**: For CI/CD integration
- **AWS CodeBuild**: For integration tests with AWS services

## Troubleshooting

Common testing issues and solutions:

- **Test Timeouts**: Increase timeout duration for integration tests
- **AWS Credential Issues**: Verify AWS credentials are correctly set
- **Resource Cleanup**: Ensure tests clean up resources after completion
- **Test Data Conflicts**: Use unique identifiers for test data 