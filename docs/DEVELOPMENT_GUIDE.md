# Investment Management Platform Development Guide

This guide provides comprehensive information for developers working on the Investment Management Platform. It covers development environment setup, coding standards, testing procedures, and contribution guidelines.

## Table of Contents

1. [Development Environment Setup](#development-environment-setup)
2. [Project Structure](#project-structure)
3. [Coding Standards](#coding-standards)
4. [Testing](#testing)
5. [Documentation](#documentation)
6. [Contribution Workflow](#contribution-workflow)
7. [Debugging](#debugging)
8. [Performance Optimization](#performance-optimization)
9. [Common Issues](#common-issues)

## Development Environment Setup

### Prerequisites

- Rust (latest stable version)
- Cargo (comes with Rust)
- Git
- AWS CLI (for deployment)
- Docker (for local testing)
- Visual Studio Code or another IDE with Rust support

### Installation

1. Clone the repository:
   ```bash
   git clone https://github.com/yourusername/investment-management-platform.git
   cd investment-management-platform
   ```

2. Install Rust dependencies:
   ```bash
   rustup update stable
   rustup component add rustfmt clippy
   ```

3. Build the project:
   ```bash
   cargo build
   ```

4. Run tests to verify your setup:
   ```bash
   cargo test
   ```

### Environment Configuration

Create a `.env` file in the project root with the following variables:

```
AWS_REGION=us-west-2
DYNAMODB_TABLE_PREFIX=dev_
API_BASE_URL=http://localhost:3000
LOG_LEVEL=debug
```

For local development, you can use the provided Docker Compose file to set up local versions of AWS services:

```bash
docker-compose up -d
```

## Project Structure

The project follows a modular architecture with the following structure:

```
investment-management-platform/
├── src/
│   ├── api/              # API handlers and routes
│   ├── models/           # Data models and schemas
│   ├── services/         # Business logic services
│   │   ├── portfolio/    # Portfolio management
│   │   ├── tax/          # Tax optimization
│   │   ├── household/    # Household management
│   │   ├── charitable/   # Charitable giving
│   │   ├── risk/         # Risk analysis
│   │   └── performance/  # Performance analysis
│   ├── repositories/     # Data access layer
│   ├── utils/            # Utility functions
│   └── main.rs           # Application entry point
├── examples/             # Example code
├── tests/                # Integration tests
├── benches/              # Performance benchmarks
├── docs/                 # Documentation
├── scripts/              # Deployment and utility scripts
└── Cargo.toml            # Project dependencies
```

### Key Modules

- **API**: Handles HTTP requests and responses, input validation, and routing.
- **Models**: Defines data structures used throughout the application.
- **Services**: Contains the core business logic of the application.
- **Repositories**: Manages data persistence and retrieval.
- **Utils**: Provides common utility functions and helpers.

## Coding Standards

### Rust Style Guide

We follow the official Rust style guide and use `rustfmt` to enforce consistent formatting:

```bash
cargo fmt
```

### Code Quality

We use `clippy` to catch common mistakes and improve code quality:

```bash
cargo clippy -- -D warnings
```

### Naming Conventions

- **Types** (structs, enums, traits): `PascalCase`
- **Variables and functions**: `snake_case`
- **Constants**: `SCREAMING_SNAKE_CASE`
- **Modules**: `snake_case`

### Documentation

All public APIs should be documented with doc comments:

```rust
/// Calculates the time-weighted return for a portfolio.
///
/// # Arguments
///
/// * `portfolio_id` - The ID of the portfolio
/// * `start_date` - The start date for the calculation
/// * `end_date` - The end date for the calculation
///
/// # Returns
///
/// The time-weighted return as a decimal (e.g., 0.05 for 5%)
///
/// # Errors
///
/// Returns an error if the portfolio is not found or if there's insufficient data.
pub fn calculate_twr(portfolio_id: &str, start_date: NaiveDate, end_date: NaiveDate) -> Result<f64, Error> {
    // Implementation
}
```

## Testing

### Unit Tests

Write unit tests for individual functions and methods. Place them in the same file as the code they test:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_twr() {
        // Test implementation
    }
}
```

Run unit tests with:

```bash
cargo test
```

### Integration Tests

Integration tests are located in the `tests/` directory and test the interaction between different components:

```bash
cargo test --test integration_test
```

### Property-Based Testing

We use `proptest` for property-based testing. Define properties that should hold for your functions and let the framework generate test cases:

```rust
proptest! {
    #[test]
    fn test_portfolio_value_is_sum_of_positions(positions in vec(any::<Position>(), 0..100)) {
        let portfolio = Portfolio { positions: positions.clone(), .. Portfolio::default() };
        let total_value = portfolio.calculate_value();
        let sum_of_positions = positions.iter().map(|p| p.value).sum::<f64>();
        prop_assert!((total_value - sum_of_positions).abs() < 0.001);
    }
}
```

### Performance Benchmarks

We use `criterion` for benchmarking. Benchmarks are located in the `benches/` directory:

```bash
cargo bench
```

## Documentation

### Code Documentation

- Use doc comments (`///`) for public APIs
- Use regular comments (`//`) for implementation details
- Document complex algorithms thoroughly

### Architecture Documentation

Update the architecture documentation when making significant changes:

- `docs/ARCHITECTURE.md`: Overall system architecture
- `docs/API_REFERENCE.md`: API documentation
- `docs/DOCUMENTATION.md`: Main documentation entry point

### Generate Documentation

Generate and view the documentation locally:

```bash
cargo doc --open
```

## Contribution Workflow

### Branching Strategy

We use a feature branch workflow:

1. Create a new branch for your feature or bugfix:
   ```bash
   git checkout -b feature/your-feature-name
   ```

2. Make your changes and commit them with descriptive messages:
   ```bash
   git commit -m "Add feature X that does Y"
   ```

3. Push your branch to the remote repository:
   ```bash
   git push -u origin feature/your-feature-name
   ```

4. Create a pull request for review

### Code Review Process

- All code changes require at least one review
- Address all review comments before merging
- Ensure all tests pass
- Verify that documentation is updated

### Continuous Integration

Our CI pipeline runs the following checks on each pull request:

- Build the project
- Run unit and integration tests
- Run clippy for code quality
- Verify code formatting with rustfmt
- Generate and check documentation

## Debugging

### Logging

We use the `log` crate with `env_logger` for logging. Configure the log level using the `LOG_LEVEL` environment variable:

```rust
log::debug!("Portfolio value: {}", portfolio.value);
log::info!("Calculation completed for portfolio {}", portfolio_id);
log::warn!("Insufficient data for accurate calculation");
log::error!("Failed to retrieve portfolio: {}", error);
```

### Error Handling

Use the `thiserror` crate for defining error types:

```rust
#[derive(Debug, Error)]
pub enum PortfolioError {
    #[error("Portfolio not found: {0}")]
    NotFound(String),
    
    #[error("Insufficient data for calculation")]
    InsufficientData,
    
    #[error("Database error: {0}")]
    Database(#[from] DatabaseError),
}
```

### Local Testing with AWS Services

For testing AWS integrations locally:

1. Use LocalStack for AWS service emulation:
   ```bash
   docker run -d -p 4566:4566 -p 4571:4571 localstack/localstack
   ```

2. Configure your application to use LocalStack:
   ```
   AWS_ENDPOINT=http://localhost:4566
   ```

## Performance Optimization

### Profiling

Use the `flamegraph` crate to identify performance bottlenecks:

```bash
cargo install flamegraph
cargo flamegraph --bin investment-management-platform
```

### Optimization Guidelines

1. **Measure first**: Always profile before optimizing
2. **Optimize algorithms**: Choose appropriate data structures and algorithms
3. **Reduce allocations**: Minimize heap allocations in hot paths
4. **Use parallelism**: Leverage Rust's concurrency features for CPU-bound tasks
5. **Optimize I/O**: Use async I/O for I/O-bound operations

## Common Issues

### DynamoDB Connection Issues

If you're having trouble connecting to DynamoDB locally:

```bash
# Check if LocalStack is running
docker ps | grep localstack

# Create required tables
aws --endpoint-url=http://localhost:4566 dynamodb create-table --table-name dev_portfolios --attribute-definitions AttributeName=id,AttributeType=S --key-schema AttributeName=id,KeyType=HASH --provisioned-throughput ReadCapacityUnits=5,WriteCapacityUnits=5
```

### Compilation Errors

For common compilation errors:

1. **Borrow checker issues**: Review Rust's ownership rules and consider using `Rc<T>` or `Arc<T>` for shared ownership
2. **Type mismatches**: Check for implicit conversions and use explicit type annotations
3. **Missing traits**: Implement required traits or use derive macros

### AWS Deployment Issues

If deployment to AWS fails:

1. Verify AWS credentials are correctly configured
2. Check CloudFormation template for errors
3. Review IAM permissions for the deployment user

## Additional Resources

- [Rust Book](https://doc.rust-lang.org/book/)
- [Rust by Example](https://doc.rust-lang.org/rust-by-example/)
- [AWS SDK for Rust Documentation](https://docs.rs/aws-sdk-dynamodb/latest/aws_sdk_dynamodb/)
- [DynamoDB Developer Guide](https://docs.aws.amazon.com/amazondynamodb/latest/developerguide/Introduction.html)
- [Tokio Documentation](https://tokio.rs/docs/overview/)

## Getting Help

If you need assistance, you can:

1. Check the existing documentation
2. Search for similar issues in the issue tracker
3. Ask questions in the #dev channel on Slack
4. Contact the core development team at dev@example.com 