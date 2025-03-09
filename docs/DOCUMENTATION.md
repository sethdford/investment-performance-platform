# Investment Management Platform Documentation

This document serves as the main entry point for all documentation related to the Investment Management Platform. It provides an overview of the platform's capabilities, architecture, and guides for various aspects of development, deployment, and usage.

## Table of Contents

- [Investment Management Platform Documentation](#investment-management-platform-documentation)
  - [Table of Contents](#table-of-contents)
  - [Platform Overview](#platform-overview)
  - [Platform Capabilities](#platform-capabilities)
    - [Portfolio Management](#portfolio-management)
    - [Tax Optimization](#tax-optimization)
    - [Household Management](#household-management)
    - [Charitable Giving](#charitable-giving)
    - [Risk Analysis](#risk-analysis)
    - [Performance Analysis](#performance-analysis)
  - [Architecture](#architecture)
    - [High-Level Architecture Diagram](#high-level-architecture-diagram)
  - [Core Modules](#core-modules)
    - [Core Modules](#core-modules-1)
  - [Getting Started](#getting-started)
    - [Prerequisites](#prerequisites)
    - [Installation](#installation)
  - [API Reference](#api-reference)
  - [Development Guide](#development-guide)
  - [Operations Guide](#operations-guide)
    - [Deployment](#deployment)
    - [Monitoring](#monitoring)
  - [Testing](#testing)
  - [Security](#security)
  - [Roadmap and Future Plans](#roadmap-and-future-plans)
  - [Project Status](#project-status)
  - [Contributing](#contributing)
  - [License](#license)

## Platform Overview

The Investment Management Platform is a comprehensive Rust-based system designed to provide advanced portfolio management, tax optimization, charitable giving, and household financial planning capabilities. The platform aims to help financial advisors and individuals manage their investments more effectively, optimize tax strategies, and plan for financial goals.

## Platform Capabilities

The Investment Management Platform provides a comprehensive set of capabilities for investment management:

### Portfolio Management
- Model portfolio creation and management
- Asset allocation and rebalancing
- Factor-based portfolio construction
- ESG screening and impact reporting
- Portfolio optimization algorithms
- Direct indexing and customization

### Tax Optimization
- Tax-loss harvesting with wash sale prevention
- Tax-efficient asset location
- Tax-aware rebalancing
- Charitable giving tax strategies
- Multi-year tax planning
- Tax-lot accounting and specific lot identification

### Household Management
- Multi-account household management
- Financial goal tracking and planning
- Risk analysis and recommendations
- Estate planning and beneficiary management
- Withdrawal planning and RMD calculations
- Fee billing and household-level reporting

### Charitable Giving
- Donation tracking and tax impact analysis
- Charitable vehicle management (DAFs, QCDs, trusts)
- Donation strategy recommendations
- Charitable giving reporting
- In-kind donation support
- Multi-year donation planning

### Risk Analysis
- Portfolio volatility calculation
- Value at Risk (VaR) and Conditional VaR
- Concentration risk analysis
- Factor-based risk decomposition
- Stress testing and scenario analysis
- Risk attribution by factor exposure

### Performance Analysis
- Time-weighted return (TWR) calculation
- Money-weighted return (MWR) calculation
- Performance attribution
- Benchmark comparison
- Multi-currency support
- Custom reporting periods

## Architecture

The Investment Management Platform follows a modular, cloud-native architecture designed for scalability, reliability, and security. For detailed architecture information, see the [Architecture](ARCHITECTURE.md) document.

### High-Level Architecture Diagram

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

## Core Modules

The Investment Management Platform is organized into the following modules:

### Core Modules

- **portfolio**: Portfolio management and analysis
  - **model**: Model portfolios and investment strategies
  - **rebalancing**: Portfolio rebalancing and trade generation
  - **factor**: Factor model analysis and risk decomposition
- **tax**: Tax optimization and management
  - **tlh**: Algorithmic tax-loss harvesting
- **analytics**: Analytics and visualization
- **cloud**: Cloud integration for scalable deployment
  - **aws**: AWS-specific integrations
- **api**: API for interacting with the platform
- **common**: Common utilities and error handling
- **logging**: Structured logging and observability

## Getting Started

### Prerequisites

- Rust 1.70 or later
- AWS CLI
- Docker
- Make

### Installation

```bash
# Clone the repository
git clone https://github.com/yourusername/investment-management-platform.git
cd investment-management-platform

# Build the application
cargo build

# Run tests
cargo test --all

# Deploy to AWS
make deploy-dev
```

For more detailed instructions, see the [Developer Guide](DEVELOPMENT_GUIDE.md).

## API Reference

The platform provides a comprehensive API for interacting with all aspects of the system. For the complete API reference, see the [API Reference](API_REFERENCE.md) document.

## Development Guide

For detailed development information, including project structure, development workflow, and best practices, see the [Development Guide](DEVELOPMENT_GUIDE.md) document.

## Operations Guide

### Deployment

The project includes AWS SAM templates for different deployment scenarios:

- **Production**: `template-production.yaml`
- **Development**: `template-development.yml`

For more information on deployment, see the [Disaster Recovery](DISASTER_RECOVERY.md) document.

### Monitoring

The platform includes comprehensive monitoring and observability features:

- CloudWatch metrics and alarms
- X-Ray distributed tracing
- Structured logging
- Health checks and dashboards

## Testing

The platform includes comprehensive tests for all components. For more information on testing, see the [Testing](TESTING.md) document.

## Security

The platform implements comprehensive security measures. For more detailed security information, see the [Security](SECURITY.md) document.

## Roadmap and Future Plans

For information on the project roadmap and future plans, see the [Roadmap](../ROADMAP.md) document.

## Project Status

The platform is currently in active development. For the current status, see the [Current Sprint Tasks](../CURRENT_SPRINT_TASKS.md) document.

## Contributing

For information on how to contribute to the project, see the [Contributing](../CONTRIBUTING.md) document.

## License

This project is licensed under the MIT License - see the [LICENSE](../LICENSE) file for details. 