# 🚀 Performance Calculator - Quick Start Guide

Welcome to the Performance Calculator project! This guide will help you get up and running quickly, even if you're new to the project or to Rust development.

## 📋 What You'll Learn

- How to set up your development environment
- How to run the application locally
- How to test your changes
- How to deploy the application
- Common tasks and troubleshooting tips

## 🛠️ Prerequisites

Before you begin, make sure you have the following installed:

| Tool | Version | Purpose | Installation Link |
|------|---------|---------|------------------|
| Rust | 1.60+ | Programming language | [Install Rust](https://www.rust-lang.org/tools/install) |
| Cargo | (comes with Rust) | Package manager | Included with Rust |
| AWS CLI | 2.0+ | AWS command line tool | [Install AWS CLI](https://aws.amazon.com/cli/) |
| Docker | Latest | Container platform (optional) | [Install Docker](https://docs.docker.com/get-docker/) |
| Git | Latest | Version control | [Install Git](https://git-scm.com/downloads) |

> 💡 **Tip for beginners**: If you're new to Rust, consider going through the [Rust Book](https://doc.rust-lang.org/book/) to get familiar with the language basics.

## 🏁 Getting Started

### Step 1: Clone the Repository

```bash
git clone https://github.com/your-org/rust-investment-performance.git
cd rust-investment-performance/performance-calculator
```

### Step 2: Build the Project

```bash
# Build the project
cargo build

# This might take a few minutes the first time as it downloads and compiles dependencies
```

You should see output similar to this:
```
   Compiling rust_decimal v1.25.0
   Compiling chrono v0.4.23
   ...
   Compiling performance-calculator v0.1.0
    Finished dev [unoptimized + debuginfo] target(s) in 2m 15s
```

## 💻 Local Development

### Running the Application Locally

#### Step 1: Set Up Environment Variables

Environment variables configure how the application behaves. For local development, you'll need to set these:

```bash
# For macOS/Linux
export DYNAMODB_TABLE=performance_metrics
export TIMESTREAM_DATABASE=investment_performance
export TIMESTREAM_TABLE=performance_metrics

# For Windows (Command Prompt)
set DYNAMODB_TABLE=performance_metrics
set TIMESTREAM_DATABASE=investment_performance
set TIMESTREAM_TABLE=performance_metrics

# For Windows (PowerShell)
$env:DYNAMODB_TABLE="performance_metrics"
$env:TIMESTREAM_DATABASE="investment_performance"
$env:TIMESTREAM_TABLE="performance_metrics"
```

#### Step 2: Run with a Sample Event

```bash
# Run the application with a sample event
cargo run -- -e ../events/performance-calculation-event.json
```

This command:
1. Builds the application if needed
2. Runs it with the specified event file as input
3. Shows the output in your terminal

### Using Local DynamoDB and Timestream

For development, you can use a local DynamoDB instance instead of connecting to AWS:

#### Step 1: Start DynamoDB Local with Docker

```bash
# Start DynamoDB Local in a Docker container
docker run -p 8000:8000 amazon/dynamodb-local

# This will run in the foreground, so you might want to open a new terminal window
```

You should see output like:
```
Initializing DynamoDB Local with the following configuration:
Port:   8000
InMemory:       true
...
```

#### Step 2: Configure the Application to Use Local DynamoDB

```bash
# Set the endpoint URL to point to your local DynamoDB
export AWS_ENDPOINT_URL=http://localhost:8000

# For Windows (Command Prompt)
set AWS_ENDPOINT_URL=http://localhost:8000

# For Windows (PowerShell)
$env:AWS_ENDPOINT_URL="http://localhost:8000"
```

#### Step 3: Create Required Tables

```bash
# Create the DynamoDB table
aws dynamodb create-table \
    --table-name performance_metrics \
    --attribute-definitions AttributeName=id,AttributeType=S AttributeName=calculation_date,AttributeType=S \
    --key-schema AttributeName=id,KeyType=HASH AttributeName=calculation_date,KeyType=RANGE \
    --provisioned-throughput ReadCapacityUnits=5,WriteCapacityUnits=5 \
    --endpoint-url http://localhost:8000
```

> 💡 **Note for beginners**: For Timestream, we use a mock implementation in development mode since there's no local version available.

## 🧪 Testing

### Running Tests

The project includes various tests to ensure everything works correctly:

#### Run All Tests

```bash
# Run all tests
cargo test

# You should see output showing which tests passed
```

#### Run Specific Tests

```bash
# Run tests that match a specific name
cargo test calculate_twr

# This will run only tests with "calculate_twr" in their name
```

#### Run Tests with Logging

```bash
# Run tests with log output
RUST_LOG=info cargo test -- --nocapture

# This shows log messages during test execution
```

### Sample Events for Testing

The repository includes several sample events you can use for testing:

| Event File | Purpose |
|------------|---------|
| `events/performance-calculation-event.json` | Calculate portfolio performance |
| `events/transaction-event.json` | Process a transaction |
| `events/batch-portfolio-performance-event.json` | Calculate performance for multiple portfolios |
| `events/attribution-event.json` | Perform attribution analysis |
| `events/benchmark-comparison-event.json` | Compare against a benchmark |
| `events/periodic-returns-event.json` | Calculate periodic returns |

Example of using a sample event:

```bash
# Run with a specific event
cargo run -- -e ../events/benchmark-comparison-event.json
```

## 📂 Project Structure

Understanding the project structure will help you navigate the codebase:

```
performance-calculator/
├── src/                          # Source code
│   ├── main.rs                   # Entry point and event handlers
│   ├── config.rs                 # Configuration management
│   └── calculations/             # Calculation modules
│       ├── mod.rs                # Module exports
│       ├── performance_metrics.rs # Performance metrics calculations
│       ├── risk_metrics.rs       # Risk metrics calculations
│       ├── currency.rs           # Multi-currency support
│       ├── distributed_cache.rs  # Redis-based caching
│       ├── audit.rs              # Audit trail functionality
│       ├── streaming.rs          # Streaming data processing
│       ├── query_api.rs          # Interactive query API
│       ├── scheduler.rs          # Scheduled calculations
│       └── tests.rs              # Integration tests
├── events/                       # Sample events for testing
├── config.json                   # Sample configuration
├── Cargo.toml                    # Project dependencies
├── README.md                     # Project documentation
├── TECHNICAL_DESIGN.md           # Technical design document
└── QUICKSTART.md                 # This guide
```

### Key Files Explained

- **main.rs**: The entry point of the application. It handles incoming events and routes them to the appropriate handlers.
- **config.rs**: Manages application configuration from environment variables and files.
- **calculations/mod.rs**: Exports all calculation functions from the various modules.
- **calculations/performance_metrics.rs**: Implements performance calculations like TWR and MWR.
- **calculations/risk_metrics.rs**: Implements risk metrics like volatility and Sharpe ratio.

## 🔧 Common Tasks

### Adding a New Calculation

Here's how to add a new calculation to the project:

#### Step 1: Identify the Appropriate Module

Determine which module your calculation belongs in:
- Performance metrics → `performance_metrics.rs`
- Risk metrics → `risk_metrics.rs`
- Currency-related → `currency.rs`
- etc.

#### Step 2: Implement the Calculation Function

Add your function to the appropriate module:

```rust
// In calculations/risk_metrics.rs
/// Calculates a custom risk metric
///
/// # Arguments
///
/// * `returns` - A slice of decimal returns
///
/// # Returns
///
/// * `Decimal` - The calculated metric value
pub fn calculate_custom_metric(returns: &[Decimal]) -> Decimal {
    // Your implementation here
    let sum = returns.iter().sum();
    sum / Decimal::from(returns.len())
}

// Don't forget to add tests!
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_calculate_custom_metric() {
        // Test implementation
    }
}
```

#### Step 3: Export the Function

Add your function to the module exports in `calculations/mod.rs`:

```rust
// In calculations/mod.rs
pub use risk_metrics::{
    // ... existing exports
    calculate_custom_metric,
};
```

#### Step 4: Update the Event Handler (if needed)

If your calculation needs to be accessible from events, update the event handler in `main.rs`:

```rust
// In main.rs
match event.event_type.as_str() {
    // ... existing event types
    "custom_metric" => {
        if let Some(portfolio_id) = &event.portfolio_id {
            let returns = repository.get_returns(portfolio_id).await?;
            let custom_metric = calculations::calculate_custom_metric(&returns);
            
            // Store the result
            repository.store_custom_metric(portfolio_id, custom_metric).await?;
        } else {
            return Ok(Response {
                request_id,
                status: "error".to_string(),
                message: "Missing portfolio_id for custom_metric event".to_string(),
            });
        }
    },
    // ... other event types
}
```

### Adding a New Event Type

Here's how to add support for a new event type:

#### Step 1: Update the Event Handler

Add a new match arm in the event handler in `main.rs`:

```rust
// In main.rs
match event.event_type.as_str() {
    // ... existing event types
    "new_event_type" => {
        if let Some(required_field) = &event.required_field {
            handle_new_event_type(
                &repository,
                &timestream_client,
                &timestream_database,
                &timestream_table,
                required_field,
                &request_id
            ).await?;
        } else {
            return Ok(Response {
                request_id,
                status: "error".to_string(),
                message: "Missing required_field for new_event_type event".to_string(),
            });
        }
    },
    // ... other event types
}
```

#### Step 2: Implement the Handler Function

Add the handler function to `main.rs`:

```rust
/// Handles a new event type
///
/// # Arguments
///
/// * `repository` - The DynamoDB repository
/// * `timestream_client` - The Timestream client
/// * `timestream_database` - The Timestream database name
/// * `timestream_table` - The Timestream table name
/// * `required_field` - The required field from the event
/// * `request_id` - The request ID
///
/// # Returns
///
/// * `Result<()>` - Success or error
async fn handle_new_event_type(
    repository: &DynamoDbRepository,
    timestream_client: &TimestreamWriteClient,
    timestream_database: &str,
    timestream_table: &str,
    required_field: &str,
    request_id: &str
) -> Result<()> {
    // Implementation
    info!(request_id = %request_id, "Processing new event type");
    
    // Your implementation here
    
    Ok(())
}
```

#### Step 3: Create a Sample Event

Create a sample event file in the `events` directory:

```json
// In events/new-event-type.json
{
  "type": "new_event_type",
  "required_field": "example-value",
  "request_id": "req-12345"
}
```

## 🚢 Deployment

### Building for AWS Lambda

To deploy the application to AWS Lambda, follow these steps:

#### Step 1: Build for Amazon Linux 2

```bash
# Install the target
rustup target add x86_64-unknown-linux-musl

# Install musl tools (on Ubuntu/Debian)
# sudo apt-get install musl-tools

# Build for the target
cargo build --release --target x86_64-unknown-linux-musl
```

#### Step 2: Package the Binary

```bash
# Rename the binary to "bootstrap" (required by AWS Lambda)
cp ./target/x86_64-unknown-linux-musl/release/performance-calculator ./bootstrap

# Create a ZIP file
zip -j rust.zip ./bootstrap
```

#### Step 3: Deploy to AWS Lambda

```bash
# Upload the ZIP file to Lambda
aws lambda update-function-code \
  --function-name performance-calculator \
  --zip-file fileb://rust.zip
```

### Setting Up AWS Resources

The application requires several AWS resources:

#### DynamoDB Table

```bash
# Create the DynamoDB table
aws dynamodb create-table \
  --table-name performance_metrics \
  --attribute-definitions AttributeName=id,AttributeType=S AttributeName=calculation_date,AttributeType=S \
  --key-schema AttributeName=id,KeyType=HASH AttributeName=calculation_date,KeyType=RANGE \
  --billing-mode PAY_PER_REQUEST
```

#### Timestream Database and Table

```bash
# Create the Timestream database
aws timestream-write create-database \
  --database-name investment_performance

# Create the Timestream table
aws timestream-write create-table \
  --database-name investment_performance \
  --table-name performance_metrics \
  --retention-properties "{\"MemoryStoreRetentionPeriodInHours\":24,\"MagneticStoreRetentionPeriodInDays\":7}"
```

#### Lambda Function

```bash
# Create the Lambda function
aws lambda create-function \
  --function-name performance-calculator \
  --runtime provided.al2 \
  --role arn:aws:iam::123456789012:role/lambda-role \
  --handler bootstrap \
  --zip-file fileb://rust.zip \
  --environment "Variables={DYNAMODB_TABLE=performance_metrics,TIMESTREAM_DATABASE=investment_performance,TIMESTREAM_TABLE=performance_metrics}" \
  --timeout 30 \
  --memory-size 512
```

> 💡 **Note**: You'll need to create an IAM role with appropriate permissions for the Lambda function.

## ❓ Troubleshooting

### Common Issues and Solutions

#### 1. Build Errors

**Issue**: Errors when building the project.

**Solution**:
- Make sure you have the latest Rust version: `rustup update`
- Clean the build and try again: `cargo clean && cargo build`
- Check for missing dependencies (like musl-tools for Linux builds)

#### 2. Runtime Errors

**Issue**: Application crashes or returns errors when running.

**Solution**:
- Check environment variables are set correctly
- Verify AWS credentials are configured: `aws configure`
- Look at the error message for specific issues
- Enable debug logging: `RUST_LOG=debug cargo run -- -e ../events/performance-calculation-event.json`

#### 3. AWS Connectivity Issues

**Issue**: Cannot connect to AWS services.

**Solution**:
- Check your internet connection
- Verify AWS credentials: `aws sts get-caller-identity`
- For local DynamoDB, make sure the container is running
- Check if you need to use a VPN to access AWS resources

#### 4. Permission Denied Errors

**Issue**: AWS operations fail with permission errors.

**Solution**:
- Check IAM permissions for your AWS user/role
- For Lambda, verify the execution role has appropriate policies
- For local development, check AWS CLI configuration

### Logging

The application uses structured logging via the `tracing` crate:

```rust
// Example log statements
info!(request_id = %request_id, "Processing event");
warn!(request_id = %request_id, error = %err, "Warning occurred");
error!(request_id = %request_id, error = %err, "Error occurred");
```

To enable debug logging locally:

```bash
RUST_LOG=debug cargo run -- -e ../events/performance-calculation-event.json
```

## 🤝 Getting Help

If you're stuck or have questions:

- Check the [README.md](README.md) for general information
- Refer to the [TECHNICAL_DESIGN.md](TECHNICAL_DESIGN.md) for implementation details
- Look at the code comments and documentation
- Ask in the team Slack channel: #performance-calculator
- Contact the team at [team-email@example.com](mailto:team-email@example.com)

## 📚 Additional Resources

- [Rust Documentation](https://doc.rust-lang.org/)
- [AWS Lambda Rust Runtime](https://github.com/awslabs/aws-lambda-rust-runtime)
- [DynamoDB Developer Guide](https://docs.aws.amazon.com/amazondynamodb/latest/developerguide/)
- [Timestream Developer Guide](https://docs.aws.amazon.com/timestream/latest/developerguide/)
- [Investment Performance Calculation Methods](https://www.cfainstitute.org/en/membership/professional-development/refresher-readings/measuring-and-managing-performance) 