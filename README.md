# Investment Performance Platform

A serverless Rust application for calculating investment portfolio performance metrics.

## Overview

The Investment Performance Platform is a high-performance, scalable system built on AWS serverless architecture. It provides APIs for managing investment portfolios, items, and transactions, and calculates various performance metrics such as Time-Weighted Return (TWR), Money-Weighted Return (MWR), volatility, Sharpe ratio, and more.

## Architecture

The application follows a serverless, event-driven architecture:

- **API Handler**: AWS Lambda function that processes API requests
- **Event Processor**: AWS Lambda function that processes events from SQS
- **Performance Calculator**: AWS Lambda function that calculates performance metrics
- **Data Storage**: DynamoDB for entity data and Timestream for time-series metrics
- **Message Queue**: SQS for asynchronous processing

## Features

- Portfolio management (create, read, update, delete)
- Item management within portfolios
- Transaction recording and management
- Performance calculation with various methodologies
- Batch calculation for multiple portfolios
- Time series data for historical performance
- Multi-tenant support with data isolation
- Caching for improved performance
- Comprehensive error handling and logging

## Getting Started

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

## Documentation

- [User Guide](docs/user-guide.md): Guide for API users
- [Developer Guide](docs/developer-guide.md): Guide for developers
- [API Reference](docs/api_reference.md): API documentation
- [Security Hardening Guide](docs/security-hardening-guide.md): Security best practices
- [Disaster Recovery Plan](docs/disaster-recovery-plan.md): Procedures for disaster recovery
- [Cost Optimization Guide](docs/cost-optimization-guide.md): Cost optimization recommendations

## Development

### Project Structure

```
.
├── api-handler/            # API Handler Lambda function
├── event-processor/        # Event Processor Lambda function
├── performance-calculator/ # Performance Calculator Lambda function
├── shared/                 # Shared code and utilities
├── infrastructure/         # Infrastructure as Code (Terraform, CloudFormation)
├── scripts/                # Utility scripts
├── docs/                   # Documentation
└── tests/                  # Tests
```

### Building

```
make build
```

### Testing

```
make test
```

### Deploying

```
make deploy-dev    # Deploy to development environment
make deploy-test   # Deploy to test environment
make deploy-prod   # Deploy to production environment
```

## License

This project is proprietary and confidential.

## Contact

For questions or support, please contact [your-email@example.com](mailto:your-email@example.com). 
