# Investment Performance Calculator - Project Summary

## Project Overview

The Investment Performance Calculator is a comprehensive serverless application built with Rust and AWS services. It provides a robust platform for calculating and analyzing investment performance metrics for portfolios, accounts, and securities.

## Key Components

### Core Repositories

1. **DynamoDB Repository**
   - Provides data access patterns for clients, portfolios, accounts, securities, and transactions
   - Implements efficient querying with GSIs for relationship traversal
   - Handles audit trail recording

2. **Timestream Repository**
   - Manages time-series performance data
   - Provides efficient querying for historical performance analysis
   - Supports various time intervals (daily, weekly, monthly, quarterly, yearly)

### Lambda Functions

1. **API Handler**
   - Processes REST API requests
   - Handles basic CRUD operations
   - Routes requests to appropriate handlers

2. **Data Ingestion**
   - Processes incoming data (transactions, securities, etc.)
   - Validates and normalizes data
   - Publishes events for asynchronous processing

3. **Performance Calculator**
   - Calculates performance metrics (TWR, MWR, volatility, etc.)
   - Processes events from SQS queue
   - Stores results in Timestream

4. **GraphQL API**
   - Provides a flexible API for data retrieval
   - Integrates with both DynamoDB and Timestream repositories
   - Supports complex queries and relationships

### Core Features

1. **Multi-Currency Support**
   - Handle investments in different currencies
   - Automatic currency conversion

2. **Distributed Caching**
   - Improve performance with Redis-based caching
   - Cache frequently accessed data

3. **Audit Trail**
   - Track all operations for compliance and debugging
   - Record who did what and when

4. **Performance Metrics**
   - Time-Weighted Return (TWR)
   - Money-Weighted Return (MWR)
   - Volatility
   - Sharpe Ratio
   - Maximum Drawdown

5. **Risk Metrics**
   - Value at Risk (VaR)
   - Expected Shortfall
   - Tracking Error
   - Information Ratio

6. **Benchmark Comparison**
   - Compare portfolio performance against benchmarks
   - Calculate relative performance metrics

7. **Factor Analysis**
   - Analyze performance attribution by factors
   - Identify sources of return

8. **Scenario Analysis**
   - Evaluate portfolio performance under different scenarios
   - Stress testing

9. **Visualization**
   - Generate charts for performance analysis
   - Create comprehensive reports

## Technical Architecture

### Data Model

The application uses a single-table design in DynamoDB with the following key structure:

- **PK (Partition Key)**: Entity type and ID (e.g., "CLIENT#123", "PORTFOLIO#456")
- **SK (Sort Key)**: Entity metadata or relationship (e.g., "METADATA", "PORTFOLIO#789")
- **GSI1PK/GSI1SK**: Global Secondary Index for efficient querying

Time-series data is stored in Amazon Timestream with the following structure:

- **Dimensions**: portfolio_id, benchmark_id
- **Measures**: twr, mwr, volatility, sharpe_ratio, max_drawdown, etc.
- **Time**: Timestamp of the data point

### Event Flow

1. User submits data through REST API
2. API Handler validates and stores the data in DynamoDB
3. API Handler publishes an event to SQS
4. Performance Calculator processes the event
5. Performance Calculator calculates metrics and stores results in Timestream
6. Performance Calculator creates an audit record in DynamoDB

### API Design

#### REST API

- `/api/clients` - Client management
- `/api/portfolios` - Portfolio management
- `/api/accounts` - Account management
- `/api/securities` - Security management
- `/api/transactions` - Transaction management
- `/api/performance` - Performance metrics
- `/ingest/*` - Data ingestion endpoints

#### GraphQL API

- `/graphql` - GraphQL endpoint
- Supports complex queries and relationships
- Integrates with both DynamoDB and Timestream repositories

## Deployment

The application is deployed using AWS SAM with the following resources:

- DynamoDB Table
- Timestream Database and Table
- SQS Queue with Dead Letter Queue
- API Gateway
- Lambda Functions
- CloudWatch Logs

## Future Enhancements

1. **Machine Learning Integration**
   - Predictive analytics for portfolio performance
   - Anomaly detection for transactions

2. **Advanced Visualization**
   - Interactive dashboards
   - Custom report generation

3. **Multi-Tenant Support**
   - Enhanced security and isolation for multi-tenant usage
   - Role-based access control

4. **Real-Time Updates**
   - WebSocket support for real-time updates
   - Push notifications for significant events

5. **Mobile Application**
   - Native mobile applications for iOS and Android
   - Offline support and synchronization

## Conclusion

The Investment Performance Calculator provides a robust, scalable, and cost-effective solution for investment performance analysis. Built with Rust and AWS serverless technologies, it offers high performance, reliability, and security for financial applications. 