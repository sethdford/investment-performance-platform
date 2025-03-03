# Performance Calculator

A high-performance investment portfolio calculator built in Rust.

## Features

### Phase 1: Core Functionality

- **Multi-Currency Support**: Convert between currencies using real-time or historical exchange rates.
- **Distributed Caching with Redis**: Improve performance by caching calculation results.
- **Audit Trail and Calculation Lineage**: Track all inputs, parameters, and outputs for compliance.
- **Configuration Management**: Support for environment variables and configuration files.
- **Component Factory**: Simplified component creation and dependency injection.
- **Performance Metrics**: Calculate Time-Weighted Return (TWR) and Money-Weighted Return (MWR).
- **Risk Metrics**: Calculate volatility, Sharpe ratio, and maximum drawdown.

### Phase 2: Advanced Processing

- **Streaming Processing**: Real-time processing of transactions, price updates, and other events.
- **Query API**: Flexible API for querying performance data with filtering, sorting, and pagination.
- **Scheduler**: Schedule calculations to run at specific times or intervals.

### Phase 3: Enterprise Features

- **Advanced Analytics**:
  - **Factor Analysis**: Analyze portfolio performance attribution using factor models.
  - **Scenario Analysis**: Stress test portfolios under different market scenarios.
  - **Risk Decomposition**: Break down portfolio risk by factors and positions.

- **Visualization**:
  - **Chart Generation**: Create various chart types (line, bar, pie, etc.) for performance data.
  - **Report Templates**: Define and generate customized reports with charts and tables.
  - **Data Export**: Export data in various formats (Excel, CSV, JSON).

- **Enterprise Integration**:
  - **API Integration**: Connect to external systems via REST APIs.
  - **Notification Services**: Send email and webhook notifications.
  - **Data Import/Export**: Import and export data from various sources.

## Project Structure

The project is organized into the following directories and modules:

### Source Code Structure

- **src/calculations/**: Core calculation modules
  - **performance_metrics.rs**: TWR, MWR, and other performance calculations
  - **risk_metrics.rs**: Volatility, Sharpe ratio, and other risk metrics
  - **periodic_returns.rs**: Daily, monthly, and other periodic return calculations
  - **benchmark_comparison.rs**: Benchmark comparison metrics

- **src/calculations/**: Infrastructure modules
  - **distributed_cache.rs**: Redis and in-memory caching implementations
  - **currency.rs**: Currency conversion and exchange rate handling
  - **audit.rs**: Audit trail and calculation lineage
  - **config.rs**: Configuration management
  - **factory.rs**: Component factory for dependency injection
  - **error_handling.rs**: Error types and handling utilities
  - **parallel.rs**: Parallel processing utilities

- **src/calculations/**: Phase 2 modules
  - **streaming.rs**: Real-time event processing
  - **query_api.rs**: Query interface for performance data
  - **scheduler.rs**: Job scheduling and execution

- **src/calculations/**: Phase 3 modules
  - **analytics.rs**: Factor analysis, scenario analysis, and risk decomposition
  - **visualization.rs**: Chart generation and report templates
  - **integration.rs**: API integration, notifications, and data import/export

### Test Structure

- **src/calculations/tests/**: Test modules
  - **phase2_integration_tests.rs**: Integration tests for Phase 2 features
  - **phase3_integration_tests.rs**: Integration tests for Phase 3 features
  - **complete_integration_test.rs**: Comprehensive test of all features

### Demo Applications

- **src/bin/**: Demo applications
  - **phase1_demo.rs**: Demonstrates Phase 1 features
  - **phase2_demo.rs**: Demonstrates Phase 2 features
  - **phase3_demo.rs**: Demonstrates Phase 3 features

## Installation

### Prerequisites

- Rust 1.70 or later
- Redis (optional, for caching)

### Building from Source

```bash
git clone https://github.com/yourusername/performance-calculator.git
cd performance-calculator
cargo build --release
```

## Usage

### Basic Usage

```bash
# Run with default configuration
./target/release/performance-calculator

# Run with custom configuration file
./target/release/performance-calculator --config config.json
```

### Environment Variables

The calculator can be configured using environment variables:

```bash
# Redis configuration
export REDIS_ENABLED=true
export REDIS_URL=redis://localhost:6379
export REDIS_PREFIX=perf:
export REDIS_TTL_SECONDS=3600

# Currency configuration
export BASE_CURRENCY=USD
export EXCHANGE_RATE_PROVIDER=mock

# Phase 2 features
export STREAMING_ENABLED=true
export QUERY_API_ENABLED=true
export SCHEDULER_ENABLED=true

# Phase 3 features
export ANALYTICS_ENABLED=true
export VISUALIZATION_ENABLED=true
export INTEGRATION_ENABLED=true
```

### Demo Applications

The repository includes several demo applications to showcase different features:

```bash
# Phase 1 demo (core functionality)
cargo run --bin phase1_demo

# Phase 2 demo (advanced processing)
cargo run --bin phase2_demo

# Phase 3 demo (enterprise features)
cargo run --bin phase3_demo
```

## Examples

### Calculate Time-Weighted Return (TWR)

```rust
use performance_calculator::calculations::{
    config::Config,
    factory::ComponentFactory,
    portfolio::{Portfolio, Holding, CashBalance, Transaction},
};

// Create configuration
let config = Config::default();

// Create component factory
let factory = ComponentFactory::new(config);

// Create portfolio
let mut portfolio = Portfolio::new("PORTFOLIO1", "USD");

// Add holdings, cash balances, and transactions
// ...

// Calculate TWR
let twr_calculator = factory.create_twr_calculator();
let twr = twr_calculator.calculate_twr(&portfolio, start_date, end_date).await?;
println!("TWR: {}", twr);
```

### Advanced Analytics

```rust
use performance_calculator::calculations::{
    config::Config,
    factory::ComponentFactory,
    analytics::{Factor, Scenario},
};

// Create configuration with analytics enabled
let mut config = Config::default();
config.analytics = Some(performance_calculator::calculations::analytics::AnalyticsConfig {
    enabled: true,
    max_concurrent_scenarios: 5,
    max_factors: 10,
    enable_caching: true,
    cache_ttl_seconds: 3600,
});

// Create component factory
let factory = ComponentFactory::new(config);

// Create analytics engine
let analytics_engine = factory.create_analytics_engine().unwrap();

// Register factors and scenarios
// ...

// Perform factor analysis
let factor_analysis = analytics_engine.perform_factor_analysis(
    "PORTFOLIO1",
    start_date,
    end_date,
    None,
    "REQUEST-ID",
).await?;

// Perform scenario analysis
let scenario_analysis = analytics_engine.perform_scenario_analysis(
    "PORTFOLIO1",
    "MARKET_CRASH",
    analysis_date,
    "REQUEST-ID",
).await?;
```

## Documentation

For more detailed documentation, see:

- [API Documentation](docs/API.md)
- [Technical Design](docs/TECHNICAL_DESIGN.md)
- [Security Controls](docs/SECURITY.md)
- [Project Summary](PROJECT_SUMMARY.md)

## License

This project is licensed under the MIT License - see the LICENSE file for details. 