# Rust Investment Performance Calculator - Consolidated Documentation

## 📋 Project Overview

The Rust Investment Performance Calculator is a high-performance, scalable system built on AWS serverless architecture. It provides APIs for managing investment portfolios, items, and transactions, and calculates various performance metrics such as Time-Weighted Return (TWR), Money-Weighted Return (MWR), volatility, Sharpe ratio, and more.

## 🏗️ Architecture

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

### Key Components

1. **API Handler**: AWS Lambda function that processes API requests
2. **Event Processor**: AWS Lambda function that processes events from SQS
3. **Performance Calculator**: AWS Lambda function that calculates performance metrics
4. **Data Storage**: 
   - DynamoDB for entity data (portfolios, transactions, etc.)
   - Timestream for time-series metrics
5. **Message Queue**: SQS for asynchronous processing

## 🧩 Core Features

1. **Portfolio Management**
   - Create, read, update, delete portfolios
   - Manage items within portfolios
   - Record and manage transactions

2. **Performance Calculation**
   - Time-Weighted Return (TWR)
   - Money-Weighted Return (MWR)
   - Volatility
   - Sharpe Ratio
   - Maximum Drawdown

3. **Risk Analysis**
   - Value at Risk (VaR)
   - Expected Shortfall
   - Tracking Error
   - Information Ratio

4. **Benchmark Comparison**
   - Compare portfolio performance against benchmarks
   - Calculate relative performance metrics

5. **Advanced Analytics**
   - Factor analysis
   - Scenario analysis
   - Performance attribution

6. **Multi-Currency Support**
   - Handle investments in different currencies
   - Automatic currency conversion

7. **Distributed Caching**
   - Improve performance with caching
   - Cache frequently accessed data

8. **Audit Trail**
   - Track all operations for compliance and debugging
   - Record who did what and when

## 🧪 Project Structure

```
.
├── api-handler/            # API Handler Lambda function
├── event-processor/        # Event Processor Lambda function
├── performance-calculator/ # Performance Calculator Lambda function
│   ├── src/
│   │   ├── calculations/   # Core calculation modules
│   │   │   ├── twr.rs      # Time-Weighted Return calculations
│   │   │   ├── mwr.rs      # Money-Weighted Return calculations
│   │   │   ├── risk_metrics.rs # Risk metrics calculations
│   │   │   └── ...
│   │   ├── main.rs         # Lambda function entry point
│   │   └── ...
├── shared/                 # Shared code and utilities
├── infrastructure/         # Infrastructure as Code
├── scripts/                # Utility scripts
├── docs/                   # Documentation
└── tests/                  # Tests
```

## 🚀 Getting Started

### Prerequisites

- Rust 1.70 or later
- AWS CLI
- Terraform or AWS CloudFormation
- Make

### Installation

1. Clone the repository:
   ```
   git clone https://github.com/yourusername/investment-performance-calculator.git
   cd investment-performance-calculator
   ```

2. Build the application:
   ```
   make build
   ```

3. Deploy to AWS:
   ```
   make deploy-dev
   ```

## 📊 API Usage

The API provides endpoints for managing portfolios, items, transactions, and calculating performance metrics.

### Authentication

All API requests require authentication using a JWT token:

```
Authorization: Bearer <your_token>
```

### Example API Calls

#### Create a Portfolio

```bash
curl -X POST https://api.example.com/v1/portfolios \
  -H "Authorization: Bearer <your_token>" \
  -H "Content-Type: application/json" \
  -d '{
    "name": "My Portfolio",
    "description": "My investment portfolio",
    "base_currency": "USD"
  }'
```

#### Calculate Portfolio Performance

```bash
curl -X POST https://api.example.com/v1/portfolios/{portfolio_id}/performance \
  -H "Authorization: Bearer <your_token>" \
  -H "Content-Type: application/json" \
  -d '{
    "start_date": "2023-01-01",
    "end_date": "2023-12-31",
    "metrics": ["twr", "mwr", "volatility", "sharpe_ratio"]
  }'
```

## 🧪 Testing

The application includes comprehensive tests for all calculation modules:

```
make test
```

## 📚 Additional Resources

- [User Guide](user-guide.md): Guide for API users
- [Developer Guide](developer-guide.md): Guide for developers
- [API Reference](api_reference.md): API documentation
- [Security Hardening Guide](security-hardening-guide.md): Security best practices
- [Disaster Recovery Plan](disaster-recovery-plan.md): Procedures for disaster recovery
- [Cost Optimization Guide](cost-optimization-guide.md): Cost optimization recommendations

## 📝 License

This project is licensed under the terms specified in the LICENSE file. 