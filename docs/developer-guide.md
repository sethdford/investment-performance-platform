# Developer Guide

This guide provides detailed information for developers who want to understand and contribute to the Investment Performance Calculator project.

## Table of Contents

1. [Architecture Overview](#architecture-overview)
2. [Development Environment Setup](#development-environment-setup)
3. [Project Structure](#project-structure)
4. [Key Components](#key-components)
5. [Data Models](#data-models)
6. [Performance Calculation Methodology](#performance-calculation-methodology)
7. [Testing Strategy](#testing-strategy)
8. [Deployment Process](#deployment-process)
9. [Monitoring and Observability](#monitoring-and-observability)
10. [Contributing Guidelines](#contributing-guidelines)

## Architecture Overview

The Investment Performance Calculator is built using a serverless architecture on AWS. The main components are:

- **API Handler**: AWS Lambda function that handles API requests from clients.
- **Event Processor**: AWS Lambda function that processes events from SQS.
- **Performance Calculator**: AWS Lambda function that calculates performance metrics.
- **DynamoDB**: NoSQL database for storing portfolio, item, and transaction data.
- **Timestream**: Time-series database for storing performance metrics.
- **SQS**: Message queue for event-driven processing.
- **API Gateway**: API management service for exposing the API.

The application follows a microservices architecture, with each component deployed as a separate AWS Lambda function. This allows for independent scaling, deployment, and maintenance of each component.

### Architecture Diagram

```
┌─────────────┐     ┌─────────────┐     ┌─────────────┐
│             │     │             │     │             │
│    Client   │────▶│ API Gateway │────▶│ API Handler │
│             │     │             │     │             │
└─────────────┘     └─────────────┘     └──────┬──────┘
                                               │
                                               ▼
┌─────────────┐     ┌─────────────┐     ┌─────────────┐
│             │     │             │     │             │
│  Timestream │◀────│ Performance │◀────│     SQS     │
│             │     │ Calculator  │     │             │
└─────────────┘     └─────────────┘     └──────┬──────┘
                                               │
                                               ▼
                                        ┌─────────────┐
                                        │             │
                                        │    Event    │
                                        │  Processor  │
                                        │             │
                                        └──────┬──────┘
                                               │
                                               ▼
                                        ┌─────────────┐
                                        │             │
                                        │   DynamoDB  │
                                        │             │
                                        └─────────────┘
```

## Development Environment Setup

### Prerequisites

- Rust (latest stable version)
- AWS CLI
- AWS SAM CLI
- Docker
- Git

### Setup Steps

1. Clone the repository:

```bash
git clone https://github.com/yourusername/investment-performance-calculator.git
cd investment-performance-calculator
```

2. Install Rust dependencies:

```bash
cargo build
```

3. Set up AWS credentials:

```bash
aws configure
```

4. Set up local development environment:

```bash
# Create local DynamoDB
docker run -p 8000:8000 amazon/dynamodb-local

# Create local SQS
sam local start-api
```

5. Run tests:

```bash
cargo test
```

## Project Structure

The project is organized into several crates:

- `api-handler/`: API handler Lambda function
- `event-processor/`: Event processor Lambda function
- `performance-calculator/`: Performance calculator Lambda function
- `shared/`: Shared code (models, repositories, utilities)
- `timestream-repository/`: Timestream repository for time-series data
- `tests/`: Integration tests
- `infrastructure/`: CloudFormation templates and other infrastructure code
- `docs/`: Documentation

### Key Files

- `api-handler/src/main.rs`: Entry point for the API handler Lambda function
- `event-processor/src/main.rs`: Entry point for the event processor Lambda function
- `performance-calculator/src/main.rs`: Entry point for the performance calculator Lambda function
- `shared/src/models/`: Data models
- `shared/src/repository/`: Repository implementations
- `shared/src/validation/`: Input validation
- `shared/src/logging/`: Logging utilities
- `shared/src/error/`: Error handling
- `shared/src/resilience/`: Resilience patterns
- `shared/src/metrics/`: Metrics collection
- `shared/src/tenant/`: Tenant isolation
- `shared/src/sanitization/`: Input sanitization
- `shared/src/encryption/`: Encryption utilities
- `timestream-repository/src/lib.rs`: Timestream repository implementation
- `infrastructure/cloudformation.yaml`: CloudFormation template for AWS resources
- `infrastructure/cloudwatch-dashboard.yaml`: CloudWatch dashboard template
- `.github/workflows/ci-cd.yml`: CI/CD pipeline configuration

## Key Components

### API Handler

The API handler is responsible for handling API requests from clients. It validates input, processes requests, and returns responses. It uses the DynamoDB repository to store and retrieve data, and the Timestream repository to store and retrieve performance metrics.

Key features:
- Input validation
- Authentication and authorization
- Error handling
- Logging
- Metrics collection
- Resilience patterns

### Event Processor

The event processor is responsible for processing events from SQS. It handles events such as portfolio updates, item updates, and transaction updates. It triggers performance calculations when necessary.

Key features:
- Event processing
- Performance calculation triggering
- Error handling
- Logging
- Metrics collection
- Resilience patterns

### Performance Calculator

The performance calculator is responsible for calculating performance metrics for portfolios. It uses the DynamoDB repository to retrieve portfolio, item, and transaction data, and the Timestream repository to store performance metrics.

Key features:
- Performance calculation
- Batch processing
- Error handling
- Logging
- Metrics collection
- Resilience patterns

### DynamoDB Repository

The DynamoDB repository is responsible for storing and retrieving portfolio, item, and transaction data. It uses the AWS SDK for DynamoDB to interact with the DynamoDB service.

Key features:
- CRUD operations for portfolios, items, and transactions
- Query operations
- Pagination
- Error handling
- Caching
- Resilience patterns

### Timestream Repository

The Timestream repository is responsible for storing and retrieving time-series performance data. It uses the AWS SDK for Timestream to interact with the Timestream service.

Key features:
- Store performance data
- Query time-series data
- Error handling
- Resilience patterns

## Data Models

### Portfolio

```rust
struct Portfolio {
    id: String,
    tenant_id: String,
    name: String,
    client_id: String,
    inception_date: NaiveDate,
    benchmark_id: Option<String>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
    status: String,
    metadata: HashMap<String, String>,
}
```

### Item

```rust
struct Item {
    id: String,
    tenant_id: String,
    portfolio_id: String,
    name: String,
    description: Option<String>,
    asset_class: String,
    security_id: Option<String>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
    status: String,
    metadata: HashMap<String, String>,
}
```

### Transaction

```rust
struct Transaction {
    id: String,
    tenant_id: String,
    portfolio_id: String,
    item_id: String,
    transaction_type: String,
    amount: f64,
    quantity: Option<f64>,
    price: Option<f64>,
    transaction_date: NaiveDate,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
    status: String,
    metadata: HashMap<String, String>,
}
```

### Performance Metrics

```rust
struct PerformanceMetrics {
    portfolio_id: String,
    timestamp: DateTime<Utc>,
    twr: f64,
    mwr: f64,
    volatility: Option<f64>,
    sharpe_ratio: Option<f64>,
    max_drawdown: Option<f64>,
    benchmark_id: Option<String>,
    benchmark_return: Option<f64>,
    tracking_error: Option<f64>,
    information_ratio: Option<f64>,
}
```

## Performance Calculation Methodology

### Time-Weighted Return (TWR)

The time-weighted return is calculated using the Modified Dietz method, which accounts for cash flows during the period. The formula is:

```
TWR = (EMV - BMV - CF) / (BMV + (CF * W))
```

Where:
- EMV = Ending market value
- BMV = Beginning market value
- CF = Net cash flow during the period
- W = Weight of the cash flow (based on the timing of the cash flow)

### Money-Weighted Return (MWR)

The money-weighted return is calculated using the Internal Rate of Return (IRR) method, which solves for the rate of return that makes the net present value of all cash flows equal to zero. The formula is:

```
0 = BMV + CF1 / (1 + MWR)^t1 + CF2 / (1 + MWR)^t2 + ... + CFn / (1 + MWR)^tn - EMV
```

Where:
- BMV = Beginning market value
- CF = Cash flow
- t = Time of cash flow (in years)
- EMV = Ending market value

### Volatility

Volatility is calculated as the standard deviation of returns over the period. The formula is:

```
Volatility = sqrt(sum((r - r_avg)^2) / (n - 1))
```

Where:
- r = Return for each period
- r_avg = Average return over all periods
- n = Number of periods

### Sharpe Ratio

The Sharpe ratio is calculated as the excess return per unit of risk. The formula is:

```
Sharpe Ratio = (r - rf) / volatility
```

Where:
- r = Portfolio return
- rf = Risk-free rate
- volatility = Portfolio volatility

### Maximum Drawdown

Maximum drawdown is calculated as the maximum loss from a peak to a trough during the period. The formula is:

```
Maximum Drawdown = min((EMV / peak_value) - 1)
```

Where:
- EMV = Ending market value
- peak_value = Maximum market value reached before EMV

### Tracking Error

Tracking error is calculated as the standard deviation of the difference between the portfolio return and the benchmark return. The formula is:

```
Tracking Error = sqrt(sum((r - b)^2) / (n - 1))
```

Where:
- r = Portfolio return
- b = Benchmark return
- n = Number of periods

### Information Ratio

The information ratio is calculated as the excess return over the benchmark per unit of tracking error. The formula is:

```
Information Ratio = (r - b) / tracking_error
```

Where:
- r = Portfolio return
- b = Benchmark return
- tracking_error = Tracking error

## Testing Strategy

The project uses a combination of unit tests, integration tests, and end-to-end tests to ensure quality and correctness.

### Unit Tests

Unit tests are written for individual functions and methods to ensure they work correctly in isolation. They are located in the same file as the code they test, using the `#[cfg(test)]` attribute.

### Integration Tests

Integration tests are written to test the interaction between different components. They are located in the `tests/` directory and use the `#[test]` attribute.

### End-to-End Tests

End-to-end tests are written to test the entire application from the API to the database. They are located in the `tests/` directory and use the `#[test]` attribute.

### Running Tests

```bash
# Run all tests
cargo test

# Run unit tests only
cargo test --lib

# Run integration tests only
cargo test --test integration_tests

# Run a specific test
cargo test test_name
```

## Deployment Process

The project uses a CI/CD pipeline with GitHub Actions to automate the deployment process.

### CI/CD Pipeline

The CI/CD pipeline is defined in `.github/workflows/ci-cd.yml` and consists of the following stages:

1. **Test**: Run tests to ensure code quality and correctness.
2. **Build**: Build the Lambda functions.
3. **Deploy to Development**: Deploy to the development environment when pushing to the `develop` branch.
4. **Deploy to Production**: Deploy to the production environment when pushing to the `main` branch.

### Manual Deployment

You can also deploy manually using the AWS CLI:

```bash
aws cloudformation deploy \
  --template-file infrastructure/cloudformation.yaml \
  --stack-name investment-performance-calculator \
  --parameter-overrides Environment=dev \
  --capabilities CAPABILITY_IAM CAPABILITY_NAMED_IAM
```

## Monitoring and Observability

The project uses AWS CloudWatch for monitoring and observability.

### CloudWatch Logs

Each Lambda function logs to CloudWatch Logs, which can be used to troubleshoot issues and monitor the application.

### CloudWatch Metrics

The project collects custom metrics using the `metrics` module, which are published to CloudWatch Metrics. These metrics can be used to monitor the performance and health of the application.

### CloudWatch Dashboard

The project includes a CloudWatch dashboard defined in `infrastructure/cloudwatch-dashboard.yaml`, which provides a visual overview of the application's performance and health. The dashboard includes metrics for:

- Lambda invocations, duration, errors, and throttles
- DynamoDB consumed capacity
- SQS queue metrics
- API Gateway metrics
- Timestream latency
- Error logs

### CloudWatch Alarms

The project includes CloudWatch alarms for critical metrics, which trigger notifications when thresholds are exceeded. The alarms are defined in `infrastructure/cloudformation.yaml` and include:

- Lambda error rate
- Lambda duration
- API Gateway 4xx and 5xx errors
- DynamoDB throttling
- SQS queue depth

## Contributing Guidelines

### Code Style

The project follows the Rust style guide and uses `rustfmt` for formatting. To format your code, run:

```bash
cargo fmt
```

The project also uses `clippy` for linting. To lint your code, run:

```bash
cargo clippy
```

### Commit Messages

Commit messages should follow the [Conventional Commits](https://www.conventionalcommits.org/) specification:

```
<type>[optional scope]: <description>

[optional body]

[optional footer(s)]
```

Types:
- `feat`: A new feature
- `fix`: A bug fix
- `docs`: Documentation only changes
- `style`: Changes that do not affect the meaning of the code (white-space, formatting, etc.)
- `refactor`: A code change that neither fixes a bug nor adds a feature
- `perf`: A code change that improves performance
- `test`: Adding missing tests or correcting existing tests
- `chore`: Changes to the build process or auxiliary tools and libraries

Example:
```
feat(api): add endpoint for batch calculation

Add a new endpoint for batch calculation of performance metrics for multiple portfolios.
This allows clients to calculate performance for multiple portfolios in a single request.

Closes #123
```

### Pull Requests

Pull requests should:
- Have a clear and descriptive title
- Include a description of the changes
- Reference any related issues
- Pass all CI checks
- Be reviewed by at least one maintainer

### Development Workflow

1. Create a feature branch from `develop`
2. Make your changes
3. Write tests for your changes
4. Run tests locally
5. Push your changes
6. Create a pull request to `develop`
7. Wait for CI checks to pass
8. Get your pull request reviewed
9. Merge your pull request

### Release Process

1. Create a release branch from `develop`
2. Update version numbers
3. Create a pull request to `main`
4. Wait for CI checks to pass
5. Get your pull request reviewed
6. Merge your pull request
7. Tag the release
8. Deploy to production

## Troubleshooting

### Common Issues

#### Lambda Timeouts

If you're experiencing Lambda timeouts, check the following:
- Lambda timeout configuration
- Database query performance
- External API calls
- Large response payloads

#### DynamoDB Throttling

If you're experiencing DynamoDB throttling, check the following:
- Provisioned capacity
- Access patterns
- Hot keys
- Batch operations

#### API Gateway Errors

If you're experiencing API Gateway errors, check the following:
- Lambda errors
- Request validation
- Authentication/authorization
- Request/response size limits

### Debugging

#### CloudWatch Logs

Each Lambda function logs to CloudWatch Logs. You can view the logs in the AWS Console or using the AWS CLI:

```bash
aws logs get-log-events --log-group-name /aws/lambda/function-name --log-stream-name stream-name
```

#### X-Ray Tracing

The project uses AWS X-Ray for distributed tracing. You can view the traces in the AWS Console or using the AWS CLI:

```bash
aws xray get-trace-summaries --start-time timestamp --end-time timestamp
```

#### Local Testing

You can test the Lambda functions locally using the AWS SAM CLI:

```bash
sam local invoke FunctionName --event event.json
```

## Performance Optimization

### DynamoDB

- Use appropriate partition and sort keys
- Use sparse indexes
- Use query instead of scan
- Use batch operations
- Use caching

### Lambda

- Optimize cold starts
- Reuse connections
- Use async/await
- Use batch processing
- Optimize memory allocation

### API Gateway

- Use caching
- Use compression
- Use pagination
- Use request validation
- Use response transformation

## Security Best Practices

### Authentication and Authorization

- Use JWT for authentication
- Validate tokens
- Check permissions
- Use least privilege principle
- Implement multi-tenancy

### Data Protection

- Encrypt sensitive data
- Sanitize inputs
- Validate outputs
- Use HTTPS
- Implement proper error handling

### Infrastructure Security

- Use IAM roles
- Use VPC
- Use security groups
- Use WAF
- Implement logging and monitoring

## Conclusion

This developer guide provides a comprehensive overview of the Investment Performance Calculator project. It covers the architecture, development environment setup, key components, data models, performance calculation methodology, testing strategy, deployment process, monitoring and observability, and contributing guidelines.

For more information, please refer to the project documentation or contact the project maintainers. 