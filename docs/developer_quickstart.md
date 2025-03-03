# 🚀 Developer Quickstart Guide

Welcome to the Rust Investment Performance Calculator project! This guide will help you get up and running quickly.

## 📋 Prerequisites

Before you begin, make sure you have the following installed:

| Tool | Version | Purpose |
|------|---------|---------|
| Rust | 1.70+ | Programming language |
| Cargo | (comes with Rust) | Package manager |
| AWS CLI | 2.0+ | AWS command line tool |
| Docker | Latest | Container platform |
| Git | Latest | Version control |

## 🏁 Getting Started in 5 Minutes

### 1. Clone the Repository

```bash
git clone https://github.com/yourusername/investment-performance-calculator.git
cd investment-performance-calculator
```

### 2. Build the Project

```bash
# Build all components
cargo build

# Build a specific component
cargo build -p performance-calculator
```

### 3. Run Tests

```bash
# Run all tests
cargo test --all

# Run tests for a specific component
cargo test -p performance-calculator
```

### 4. Run Locally

```bash
# Start local DynamoDB
docker-compose up -d dynamodb-local

# Start local API
cargo run -p api-handler
```

## 🧩 Project Structure at a Glance

```
.
├── api-handler/            # API Handler Lambda function
├── event-processor/        # Event Processor Lambda function
├── performance-calculator/ # Performance Calculator Lambda function
│   ├── src/
│   │   ├── calculations/   # Core calculation modules
│   │   │   ├── twr.rs      # Time-Weighted Return calculations
│   │   │   ├── mwr.rs      # Money-Weighted Return calculations
│   │   │   └── ...
│   │   └── main.rs         # Lambda entry point
├── shared/                 # Shared code and utilities
└── docs/                   # Documentation
```

## 🛠️ Common Development Tasks

### Adding a New Calculation Method

1. Create a new file in `performance-calculator/src/calculations/`
2. Implement the appropriate trait (e.g., `TimeWeightedReturnCalculator`)
3. Register your implementation in the `ComponentFactory`
4. Add tests in the corresponding test module

Example:

```rust
// performance-calculator/src/calculations/my_calculator.rs
pub struct MyCalculator {
    // fields
}

impl TimeWeightedReturnCalculator for MyCalculator {
    async fn calculate_twr(&self, portfolio_id: &str, start_date: NaiveDate, end_date: NaiveDate) -> Result<TimeWeightedReturn> {
        // Implementation
    }
}

// In ComponentFactory
pub async fn create_twr_calculator(&self) -> Result<Arc<dyn TimeWeightedReturnCalculator>> {
    Ok(Arc::new(MyCalculator::new()))
}
```

### Adding a New API Endpoint

1. Add a new route in `api-handler/src/api.rs`
2. Create a handler function in `api-handler/src/handlers/`
3. Implement the handler logic
4. Add tests for the new endpoint

Example:

```rust
// api-handler/src/api.rs
pub fn routes() -> Router {
    Router::new()
        // Existing routes
        .route("/portfolios/:id/custom-metric", get(handlers::custom_metric::get_custom_metric))
}

// api-handler/src/handlers/custom_metric.rs
pub async fn get_custom_metric(
    Path(id): Path<String>,
    Query(params): Query<CustomMetricParams>,
    State(state): State<AppState>,
) -> Result<Json<CustomMetricResponse>, ApiError> {
    // Implementation
}
```

### Working with DynamoDB

```rust
// Get an item
let result = repository.get_item("portfolio_id").await?;

// Create an item
let new_item = Portfolio { id: "portfolio_id", name: "My Portfolio", ... };
repository.create_item(new_item).await?;

// Update an item
repository.update_item(updated_item).await?;

// Delete an item
repository.delete_item("portfolio_id").await?;
```

### Working with Timestream

```rust
// Store a metric
timestream_repository.store_metric(
    "portfolio_id",
    "twr",
    Decimal::from_str("0.0523")?,
    Utc::now(),
).await?;

// Query metrics
let metrics = timestream_repository.query_metrics(
    "portfolio_id",
    "twr",
    start_date,
    end_date,
).await?;
```

## 🔍 Debugging Tips

### Local Debugging

1. Use `println!` or the `log` crate for logging
2. Run with `RUST_BACKTRACE=1` for detailed backtraces
3. Use VS Code with the Rust Analyzer extension

```bash
RUST_BACKTRACE=1 cargo run -p performance-calculator
```

### AWS Lambda Debugging

1. Use CloudWatch Logs for Lambda function logs
2. Set the `LOG_LEVEL` environment variable to `debug` for detailed logs
3. Use X-Ray for tracing requests through the system

## 📚 Key Concepts

### Performance Metrics

- **TWR (Time-Weighted Return)**: Measures the compound rate of growth in a portfolio
- **MWR (Money-Weighted Return)**: Measures the internal rate of return (IRR) of a portfolio
- **Risk Metrics**: Volatility, Sharpe ratio, maximum drawdown, etc.

### Multi-Tenant Architecture

The system uses a tenant isolation approach:

- Each request includes a tenant ID
- Data is partitioned by tenant
- Repositories enforce tenant isolation

### Asynchronous Processing

The system uses an event-driven architecture:

1. API requests are validated and stored
2. Events are published to SQS
3. Lambda functions process events asynchronously
4. Results are stored in DynamoDB/Timestream

## 🔗 Useful Links

- [Full Documentation](platform_overview.md)
- [Technical Architecture](technical_architecture.md)
- [API Reference](api_reference.md)
- [Future Roadmap](future_roadmap.md)

## 🤝 Getting Help

- Check the existing documentation
- Look at the test cases for examples
- Reach out to the team on Slack (#investment-performance-calculator)
- Create an issue on GitHub 