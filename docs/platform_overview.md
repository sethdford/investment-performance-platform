# 🚀 Rust Investment Performance Calculator - Platform Overview

## 📋 Introduction

The Rust Investment Performance Calculator is a high-performance, scalable system built on AWS serverless architecture. It provides a comprehensive platform for calculating and analyzing investment performance metrics for portfolios, accounts, and securities.

This document provides a consolidated overview of the platform, including its architecture, key components, technical design, and advanced features.

## 🏗️ System Architecture

The application follows a serverless, event-driven architecture:

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
                                        │  DynamoDB   │
                                        │             │
                                        └─────────────┘
```

### Event Flow

1. User submits a request through the API Gateway
2. API Handler processes the request and validates the input
3. API Handler publishes an event to SQS for asynchronous processing
4. Performance Calculator Lambda function processes the event
5. Performance Calculator calculates metrics and stores results in Timestream
6. Performance Calculator creates an audit record in DynamoDB

## 🧩 Core Components

### 1. Lambda Functions

#### API Handler
- Processes REST API requests
- Handles authentication and authorization
- Validates input data
- Routes requests to appropriate handlers
- Publishes events to SQS for asynchronous processing

#### Event Processor
- Consumes events from SQS
- Processes events asynchronously
- Handles retries and error recovery
- Updates DynamoDB with processing status

#### Performance Calculator
- Calculates performance metrics (TWR, MWR, volatility, etc.)
- Processes events from SQS queue
- Stores results in Timestream
- Creates audit records in DynamoDB

### 2. Data Storage

#### DynamoDB Repository
- Provides data access patterns for clients, portfolios, accounts, securities, and transactions
- Implements efficient querying with GSIs for relationship traversal
- Handles audit trail recording
- Supports multi-tenant isolation
- Implements transaction support for atomic operations

#### Timestream Repository
- Manages time-series performance metrics
- Provides efficient querying for historical performance analysis
- Supports various time intervals (daily, weekly, monthly, quarterly, yearly)
- Optimized for time-series data storage and retrieval

### 3. Calculation Modules

The Performance Calculator includes several specialized calculation modules:

- **TWR (Time-Weighted Return)**: Calculates time-weighted returns using various methodologies
- **MWR (Money-Weighted Return)**: Calculates money-weighted returns (IRR)
- **Risk Metrics**: Calculates volatility, Sharpe ratio, maximum drawdown, etc.
- **Benchmark Comparison**: Compares portfolio performance against benchmarks
- **Currency Conversion**: Handles multi-currency portfolios and FX impact
- **Performance Attribution**: Analyzes sources of return
- **Analytics Engine**: Performs factor analysis and scenario analysis

## 🔑 Advanced Features

### Multi-Tenant Isolation

The platform implements robust tenant isolation to ensure data security in a multi-tenant environment:

1. **Tenant Context**: A `TenantContext` struct that contains the tenant ID and optional metadata
2. **Tenant Manager**: A `TenantManager` that provides utility methods for working with tenant data
3. **Tenant-Aware Repositories**: Repositories that implement the `TenantAware` trait to ensure tenant isolation
4. **Tenant Middleware**: A middleware pattern that wraps repositories to enforce tenant isolation

Key benefits:
- Complete data isolation between tenants
- Tenant-specific configuration and customization
- Resource usage tracking and limits per tenant
- Tenant-specific audit trails

### Transaction Support

The platform provides transaction support for atomic operations on multiple items in DynamoDB:

1. **TransactionOperation**: Represents different types of operations (Put, Delete, Update, ConditionCheck)
2. **TransactionManager**: Executes transactions with tenant isolation
3. **Idempotency Support**: Prevents duplicate transactions with client request tokens

Key benefits:
- Atomic operations across multiple items
- Consistent data state
- Tenant isolation within transactions
- Idempotent operations for reliability

### Distributed Caching

The platform implements a sophisticated distributed caching system:

1. **Cache Interface**: A generic interface for different cache implementations
2. **Local Cache**: In-memory cache for fast access
3. **Distributed Cache**: Redis-based cache for shared state
4. **Cache Invalidation**: Strategies for keeping cache data fresh

Key benefits:
- Improved performance for frequently accessed data
- Reduced database load
- Configurable TTL (Time-To-Live) for cached items
- Cache statistics and monitoring

### Audit Trail

The platform maintains comprehensive audit trails for all operations:

1. **Audit Events**: Records who did what and when
2. **Calculation Events**: Tracks calculation inputs, outputs, and performance
3. **Audit Storage**: Stores audit records in DynamoDB with efficient querying
4. **Audit Retrieval**: APIs for retrieving and analyzing audit data

Key benefits:
- Compliance with regulatory requirements
- Debugging and troubleshooting support
- Performance analysis and optimization
- Security incident investigation

### Streaming Processor

The platform includes a streaming processor for real-time data processing:

1. **Event Handlers**: Process different types of events
2. **Batch Processing**: Efficiently process events in batches
3. **Error Handling**: Robust error handling and retry mechanisms
4. **Monitoring**: Real-time monitoring of event processing

Key benefits:
- Real-time data processing
- Scalable event handling
- Resilient to failures
- Configurable processing options

## 🛠️ Development Environment

### Prerequisites

Before you begin, you'll need to install the following tools:

| Tool | Purpose | Installation Command | Notes |
|------|---------|---------------------|-------|
| **Rust** | Programming language | `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs \| sh` | This installs the Rust compiler and Cargo package manager |
| **AWS SAM CLI** | Local testing and deployment | See AWS documentation | Used for local testing and deploying to AWS |
| **Docker** | Container platform | See Docker documentation | Required for local testing with SAM CLI |
| **AWS CLI** | AWS command line tool | `pip install awscli` | Used to interact with AWS services |

### Building and Testing

```bash
# Build the application
cargo build

# Run tests
cargo test --all

# Run specific component tests
cargo test -p api-handler
cargo test -p event-processor
cargo test -p performance-calculator
```

### Deployment

```bash
# Deploy to development environment
make deploy-dev

# Deploy to test environment
make deploy-test

# Deploy to production environment
make deploy-prod
```

## 📊 Performance Metrics

The platform calculates various performance metrics:

### Return Metrics
- **Time-Weighted Return (TWR)**: Measures the compound rate of growth in a portfolio
- **Money-Weighted Return (MWR)**: Measures the internal rate of return (IRR) of a portfolio
- **Modified Dietz Method**: Approximates TWR by weighting cash flows
- **Daily, Monthly, Quarterly, and Annual Returns**: Returns over different time periods

### Risk Metrics
- **Volatility**: Standard deviation of returns
- **Sharpe Ratio**: Risk-adjusted return
- **Maximum Drawdown**: Largest peak-to-trough decline
- **Value at Risk (VaR)**: Potential loss with a given confidence level
- **Expected Shortfall**: Average loss beyond VaR

### Benchmark Comparison
- **Tracking Error**: Difference between portfolio and benchmark returns
- **Information Ratio**: Excess return per unit of risk
- **Up/Down Capture Ratio**: Performance in up and down markets
- **Batting Average**: Frequency of outperforming the benchmark

### Attribution Analysis
- **Factor Analysis**: Attribution of returns to different factors
- **Sector Attribution**: Contribution of different sectors
- **Security Selection**: Contribution of security selection
- **Asset Allocation**: Contribution of asset allocation decisions

## 📚 Additional Resources

- [User Guide](user-guide.md): Guide for API users
- [Developer Guide](developer-guide.md): Guide for developers
- [API Reference](api_reference.md): API documentation
- [Security Hardening Guide](security-hardening-guide.md): Security best practices
- [Disaster Recovery Plan](disaster-recovery-plan.md): Procedures for disaster recovery
- [Cost Optimization Guide](cost-optimization-guide.md): Cost optimization recommendations
- [Future Roadmap](future_roadmap.md): Planned improvements and enhancements 