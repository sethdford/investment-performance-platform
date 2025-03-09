# Performance Calculator

A high-performance investment portfolio calculator built in Rust.

## Implemented Features

### Core Functionality (Phase 1)
- ✅ Multi-Currency Support with real-time/historical exchange rates
- ✅ Distributed Caching (Redis)
- ✅ Audit Trail and Calculation Lineage
- ✅ Configuration Management
- ✅ Performance Metrics (TWR, MWR)
- ✅ Risk Metrics (Volatility, Sharpe ratio, drawdown)

### Advanced Processing (Phase 2)
- ✅ Real-time Streaming Processing
- ✅ Query API with filtering and pagination
- ✅ Job Scheduler
- ✅ Event Processing System

### Enterprise Features (Phase 3)
#### Analytics
- ✅ Factor Analysis with multi-factor models
- ✅ Scenario Analysis with historical scenarios
- ✅ Risk Decomposition with factor attribution

#### Visualization
- ✅ Chart Generation (line, bar, pie)
  - Dynamic chart options
  - Custom color schemes
  - Interactive tooltips
- ✅ Report Templates
  - Markdown-based templates
  - Variable substitution
  - Chart embedding
- ✅ Data Export (CSV, JSON)
  - Configurable formats
  - Streaming export for large datasets

#### Integration
- ✅ API Integration
  - Multiple authentication methods
  - Configurable retry policies
  - Request/response logging
- ✅ Notification Services
  - Email notifications with attachments
  - Webhook notifications with retry
  - Event-based filtering
- ✅ Data Import/Export
  - Multiple format support (CSV, JSON)
  - Validation rules
  - Error handling and reporting

## Testing

### Unit Tests
```bash
# Run unit tests
cargo test

# Run specific test module
cargo test --test phase3_integration_tests
```

### Integration Tests
The project includes comprehensive integration tests:

1. Core Integration (`test_complete_workflow`):
   - Tests end-to-end portfolio calculations
   - Verifies caching and audit trail
   - Checks currency conversions

2. Phase 2 Integration (`test_phase2_integration`):
   - Tests streaming processing
   - Verifies query API functionality
   - Checks scheduler operations

3. Phase 3 Integration (`test_phase3_integration`):
   - Tests analytics engine
     - Factor analysis
     - Scenario analysis
     - Risk decomposition
   - Tests visualization engine
     - Chart generation
     - Report creation
     - Data export
   - Tests integration services
     - API connectivity
     - Notifications
     - Data import/export

### Mock Services
The test suite includes mock implementations for:
- Redis cache
- External APIs
- Email service
- Webhook endpoints
- Data import/export services

## Project Structure

```
src/calculations/
├── core/                 # Phase 1: Core functionality
│   ├── performance.rs    # Performance calculations
│   ├── risk.rs          # Risk metrics
│   ├── currency.rs      # Currency handling
│   └── cache.rs         # Caching implementation
├── processing/          # Phase 2: Advanced processing
│   ├── streaming.rs     # Real-time processing
│   ├── query.rs         # Query API
│   └── scheduler.rs     # Job scheduling
└── enterprise/         # Phase 3: Enterprise features
    ├── analytics.rs    # Analytics engine
    ├── visualization.rs # Visualization
    └── integration.rs  # External integrations
```

## Configuration

Key environment variables:
```bash
# Core
REDIS_URL=redis://localhost:6379
BASE_CURRENCY=USD

# Processing
STREAMING_ENABLED=true
QUERY_API_ENABLED=true

# Enterprise
ANALYTICS_ENABLED=true
VISUALIZATION_ENABLED=true
INTEGRATION_ENABLED=true

# Integration Services
SMTP_SERVER=smtp.example.com
SMTP_PORT=587
WEBHOOK_RETRY_ENABLED=true
API_TIMEOUT_SECONDS=30
```

## Documentation

- [API Documentation](docs/API.md)
- [Architecture](docs/ARCHITECTURE.md)
- [Contributing](CONTRIBUTING.md)

## License

MIT License - see [LICENSE](LICENSE) for details 