# 🚀 Rust Investment Performance Calculator

A high-performance, scalable system for calculating investment portfolio performance metrics, built with Rust and AWS serverless architecture.

[![Build Status](https://img.shields.io/badge/build-passing-brightgreen.svg)](https://github.com/yourusername/investment-performance-calculator)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

## 📋 Overview

The Rust Investment Performance Calculator provides a comprehensive platform for calculating and analyzing investment performance metrics for portfolios, accounts, and securities. It leverages the performance and safety of Rust along with the scalability of AWS serverless architecture.

![Architecture Diagram](docs/images/architecture.png)

## ✨ Key Features

- **Portfolio Management**: Create, read, update, delete portfolios and their components
- **Performance Metrics**: Calculate TWR, MWR, volatility, Sharpe ratio, and more
- **Risk Analysis**: Calculate VaR, Expected Shortfall, and other risk metrics
- **Benchmark Comparison**: Compare portfolio performance against benchmarks
- **Multi-Currency Support**: Handle investments in different currencies
- **Multi-Tenant Architecture**: Secure isolation between tenants
- **Distributed Caching**: Improve performance with caching
- **Audit Trail**: Track all operations for compliance and debugging

## 🏁 Quick Start

### Prerequisites

- Rust 1.70 or later
- AWS CLI
- Docker
- Make

### Installation

```bash
# Clone the repository
git clone https://github.com/yourusername/investment-performance-calculator.git
cd investment-performance-calculator

# Build the application
cargo build

# Run tests
cargo test --all

# Deploy to AWS
make deploy-dev
```

For more detailed instructions, see the [Developer Quickstart Guide](docs/developer_quickstart.md).

## 📊 API Usage

```bash
# Create a portfolio
curl -X POST https://api.example.com/v1/portfolios \
  -H "Authorization: Bearer <your_token>" \
  -H "Content-Type: application/json" \
  -d '{
    "name": "My Portfolio",
    "description": "My investment portfolio",
    "base_currency": "USD"
  }'

# Calculate portfolio performance
curl -X POST https://api.example.com/v1/portfolios/{portfolio_id}/performance \
  -H "Authorization: Bearer <your_token>" \
  -H "Content-Type: application/json" \
  -d '{
    "start_date": "2023-01-01",
    "end_date": "2023-12-31",
    "metrics": ["twr", "mwr", "volatility", "sharpe_ratio"]
  }'
```

For more API examples, see the [API Reference](docs/api_reference.md).

## 📚 Documentation

- [Platform Overview](docs/platform_overview.md): Comprehensive overview of the platform
- [Technical Architecture](docs/technical_architecture.md): Detailed technical architecture
- [Developer Quickstart](docs/developer_quickstart.md): Quick guide for developers
- [User Guide](docs/user-guide.md): Guide for API users
- [API Reference](docs/api_reference.md): API documentation
- [Future Roadmap](docs/future_roadmap.md): Planned improvements and enhancements

## 🧪 Testing

The application includes comprehensive tests for all components:

```bash
# Run all tests
cargo test --all

# Run tests for a specific component
cargo test -p performance-calculator
```

## 🤝 Contributing

Contributions are welcome! Please read the [Contributing Guide](CONTRIBUTING.md) for more information.

## 📝 License

This project is licensed under the terms specified in the [LICENSE](LICENSE) file.

## 📧 Contact

For questions or support, please contact [your-email@example.com](mailto:your-email@example.com). 