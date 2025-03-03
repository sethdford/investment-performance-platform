# 🏗️ Rust Investment Performance Calculator - Technical Architecture

## 📋 Overview

This document provides a detailed technical architecture overview of the Rust Investment Performance Calculator. It covers the system's components, data flow, implementation details, and design patterns.

## 🧩 System Components

### Lambda Functions Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                                                                 │
│                        API Gateway                              │
│                                                                 │
└───────────────────────────────┬─────────────────────────────────┘
                                │
                                ▼
┌─────────────────────────────────────────────────────────────────┐
│                                                                 │
│                        API Handler Lambda                       │
│                                                                 │
│  ┌─────────────┐    ┌─────────────┐    ┌─────────────────────┐  │
│  │             │    │             │    │                     │  │
│  │ Auth Module │    │ Validation  │    │ Request Router      │  │
│  │             │    │             │    │                     │  │
│  └─────────────┘    └─────────────┘    └─────────────────────┘  │
│                                                                 │
└───────────────────────────────┬─────────────────────────────────┘
                                │
                                ▼
┌─────────────────────────────────────────────────────────────────┐
│                                                                 │
│                        SQS Queue                                │
│                                                                 │
└───────────────────────────────┬─────────────────────────────────┘
                                │
                                ▼
┌─────────────────────────────────────────────────────────────────┐
│                                                                 │
│                  Performance Calculator Lambda                  │
│                                                                 │
│  ┌─────────────┐    ┌─────────────┐    ┌─────────────────────┐  │
│  │             │    │             │    │                     │  │
│  │ Calculation │    │ Data Access │    │ Result Storage      │  │
│  │ Modules     │    │ Layer       │    │                     │  │
│  └─────────────┘    └─────────────┘    └─────────────────────┘  │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

### Data Storage Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                                                                 │
│                        DynamoDB                                 │
│                                                                 │
│  ┌─────────────┐    ┌─────────────┐    ┌─────────────────────┐  │
│  │             │    │             │    │                     │  │
│  │ Portfolios  │    │ Transactions│    │ Audit Trail         │  │
│  │             │    │             │    │                     │  │
│  └─────────────┘    └─────────────┘    └─────────────────────┘  │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────────┐
│                                                                 │
│                        Timestream                               │
│                                                                 │
│  ┌─────────────┐    ┌─────────────┐    ┌─────────────────────┐  │
│  │             │    │             │    │                     │  │
│  │ Daily       │    │ Monthly     │    │ Custom Period       │  │
│  │ Metrics     │    │ Metrics     │    │ Metrics             │  │
│  └─────────────┘    └─────────────┘    └─────────────────────┘  │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

## 🔄 Data Flow

### Request Processing Flow

1. **Client Request**:
   - Client sends a request to the API Gateway
   - Request includes authentication token and request parameters

2. **API Gateway Processing**:
   - API Gateway validates the request format
   - Routes the request to the appropriate Lambda function

3. **API Handler Processing**:
   - Authenticates and authorizes the user
   - Validates the request parameters
   - Performs any immediate operations (e.g., data retrieval)
   - For calculation requests, publishes an event to SQS
   - Returns an immediate response to the client

4. **Asynchronous Processing**:
   - Performance Calculator Lambda is triggered by SQS event
   - Processes the calculation request
   - Stores results in Timestream
   - Updates status in DynamoDB
   - Creates audit records

5. **Result Retrieval**:
   - Client polls for results using a request ID
   - API Handler retrieves results from DynamoDB/Timestream
   - Returns results to the client

### Data Storage Flow

1. **Entity Data**:
   - Stored in DynamoDB
   - Uses a single-table design with GSIs for efficient querying
   - Includes portfolios, transactions, securities, etc.

2. **Time-Series Data**:
   - Stored in Timestream
   - Organized by portfolio/benchmark and time period
   - Includes performance metrics, risk metrics, etc.

3. **Audit Data**:
   - Stored in DynamoDB
   - Includes operation details, user information, timestamps
   - Linked to entities via GSIs

## 🛠️ Implementation Details

### Code Organization

The codebase is organized into several key components:

```
.
├── api-handler/            # API Handler Lambda function
│   ├── src/
│   │   ├── api.rs          # API route definitions
│   │   ├── auth.rs         # Authentication and authorization
│   │   ├── handlers/       # Request handlers
│   │   └── main.rs         # Lambda entry point
│
├── event-processor/        # Event Processor Lambda function
│   ├── src/
│   │   ├── events/         # Event type definitions
│   │   ├── processors/     # Event processors
│   │   └── main.rs         # Lambda entry point
│
├── performance-calculator/ # Performance Calculator Lambda function
│   ├── src/
│   │   ├── calculations/   # Core calculation modules
│   │   │   ├── twr.rs      # Time-Weighted Return calculations
│   │   │   ├── mwr.rs      # Money-Weighted Return calculations
│   │   │   ├── risk_metrics.rs # Risk metrics calculations
│   │   │   └── ...
│   │   ├── main.rs         # Lambda entry point
│   │   └── ...
│
├── shared/                 # Shared code and utilities
│   ├── src/
│   │   ├── models.rs       # Shared data models
│   │   ├── repository/     # Data access layer
│   │   └── ...
│
└── ...
```

### Key Design Patterns

#### Repository Pattern

The application uses the Repository pattern to abstract data access:

```rust
pub trait Repository<T, K> {
    async fn get(&self, id: K) -> Result<Option<T>>;
    async fn create(&self, item: T) -> Result<T>;
    async fn update(&self, item: T) -> Result<T>;
    async fn delete(&self, id: K) -> Result<()>;
    async fn list(&self, filter: Option<Filter>) -> Result<Vec<T>>;
}
```

Implementations:
- `DynamoDbRepository`: Implements the Repository pattern for DynamoDB
- `TimestreamRepository`: Implements the Repository pattern for Timestream

#### Factory Pattern

The application uses the Factory pattern to create components:

```rust
pub struct ComponentFactory {
    config: Config,
    dynamodb_client: DynamoDbClient,
    timestream_client: TimestreamWriteClient,
    // ...
}

impl ComponentFactory {
    pub async fn create_twr_calculator(&self) -> Result<Arc<dyn TimeWeightedReturnCalculator>> {
        // ...
    }
    
    pub async fn create_mwr_calculator(&self) -> Result<Arc<dyn MoneyWeightedReturnCalculator>> {
        // ...
    }
    
    // ...
}
```

#### Strategy Pattern

The application uses the Strategy pattern for calculation algorithms:

```rust
pub trait TimeWeightedReturnCalculator: Send + Sync {
    async fn calculate_twr(&self, portfolio_id: &str, start_date: NaiveDate, end_date: NaiveDate) -> Result<TimeWeightedReturn>;
    // ...
}

pub struct ModifiedDietzCalculator {
    // ...
}

impl TimeWeightedReturnCalculator for ModifiedDietzCalculator {
    async fn calculate_twr(&self, portfolio_id: &str, start_date: NaiveDate, end_date: NaiveDate) -> Result<TimeWeightedReturn> {
        // Implementation using Modified Dietz method
    }
    // ...
}

pub struct DailyTWRCalculator {
    // ...
}

impl TimeWeightedReturnCalculator for DailyTWRCalculator {
    async fn calculate_twr(&self, portfolio_id: &str, start_date: NaiveDate, end_date: NaiveDate) -> Result<TimeWeightedReturn> {
        // Implementation using daily valuation points
    }
    // ...
}
```

#### Middleware Pattern

The application uses the Middleware pattern for cross-cutting concerns:

```rust
pub struct TenantMiddleware<T: TenantAware> {
    inner: T,
    tenant_manager: Arc<dyn TenantManager>,
}

impl<T: TenantAware> TenantMiddleware<T> {
    pub fn new(inner: T, tenant_manager: Arc<dyn TenantManager>) -> Self {
        Self { inner, tenant_manager }
    }
    
    pub fn with_tenant(&self, tenant_context: TenantContext) -> Result<Self> {
        // ...
    }
    
    pub fn inner(&self) -> &T {
        &self.inner
    }
}
```

### Multi-Tenant Implementation

The multi-tenant implementation uses several key components:

#### Tenant Context

```rust
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TenantContext {
    pub tenant_id: String,
    pub metadata: HashMap<String, String>,
}
```

#### Tenant-Aware Trait

```rust
pub trait TenantAware {
    fn tenant_context(&self) -> &TenantContext;
    fn with_tenant_context(&self, tenant_context: TenantContext) -> Self;
}
```

#### Tenant Manager

```rust
pub trait TenantManager: Send + Sync {
    async fn get_tenant(&self, tenant_id: &str) -> Result<Tenant>;
    async fn create_tenant(&self, tenant: Tenant) -> Result<Tenant>;
    async fn update_tenant(&self, tenant: Tenant) -> Result<Tenant>;
    async fn delete_tenant(&self, tenant_id: &str) -> Result<()>;
    async fn list_tenants(&self) -> Result<Vec<Tenant>>;
}
```

### Transaction Support

The transaction support implementation provides atomic operations:

#### Transaction Operation

```rust
pub enum TransactionOperation {
    Put {
        table_name: String,
        item: HashMap<String, AttributeValue>,
        condition_expression: Option<String>,
        expression_attribute_names: Option<HashMap<String, String>>,
        expression_attribute_values: Option<HashMap<String, AttributeValue>>,
    },
    Delete {
        table_name: String,
        key: HashMap<String, AttributeValue>,
        condition_expression: Option<String>,
        expression_attribute_names: Option<HashMap<String, String>>,
        expression_attribute_values: Option<HashMap<String, AttributeValue>>,
    },
    Update {
        table_name: String,
        key: HashMap<String, AttributeValue>,
        update_expression: String,
        condition_expression: Option<String>,
        expression_attribute_names: Option<HashMap<String, String>>,
        expression_attribute_values: Option<HashMap<String, AttributeValue>>,
    },
    ConditionCheck {
        table_name: String,
        key: HashMap<String, AttributeValue>,
        condition_expression: String,
        expression_attribute_names: Option<HashMap<String, String>>,
        expression_attribute_values: Option<HashMap<String, AttributeValue>>,
    },
}
```

#### Transaction Manager

```rust
pub struct TransactionManager {
    client: DynamoDbClient,
    tenant_context: TenantContext,
}

impl TransactionManager {
    pub async fn execute_transaction(&self, operations: Vec<TransactionOperation>) -> Result<()> {
        // Implementation
    }
    
    pub async fn execute_transaction_with_token(&self, operations: Vec<TransactionOperation>, client_request_token: String) -> Result<()> {
        // Implementation with idempotency
    }
}
```

### Distributed Caching

The distributed caching implementation provides efficient data access:

#### Cache Interface

```rust
pub trait Cache<K, V> {
    fn get(&self, key: &K) -> Option<V>;
    fn set(&self, key: K, value: V, ttl_seconds: u64);
    fn remove(&self, key: &K) -> Option<V>;
    fn clear(&self);
}
```

#### Cache Implementations

```rust
pub struct LocalCache<K, V> {
    // Implementation using a local in-memory cache
}

pub struct RedisCache<K, V> {
    // Implementation using Redis
}

pub struct TieredCache<K, V> {
    // Implementation using multiple cache levels
}
```

## 🔐 Security Implementation

### Authentication and Authorization

The system implements a robust authentication and authorization system:

1. **JWT Authentication**:
   - Uses JSON Web Tokens (JWT) for authentication
   - Validates tokens using a public key
   - Checks token expiration and signature

2. **Role-Based Access Control (RBAC)**:
   - Defines roles (Admin, User, ReadOnly, etc.)
   - Maps roles to permissions
   - Validates permissions for each API endpoint

3. **Tenant-Based Authorization**:
   - Ensures users can only access their tenant's data
   - Validates tenant ID in the token against requested resources
   - Implements tenant isolation at the repository level

### Data Encryption

The system implements comprehensive data encryption:

1. **Data at Rest**:
   - DynamoDB tables are encrypted using AWS-managed keys
   - Timestream tables are encrypted using AWS-managed keys
   - S3 buckets are encrypted using AWS-managed keys

2. **Data in Transit**:
   - All API endpoints use HTTPS
   - Internal AWS service communications use TLS
   - VPC endpoints are used where applicable

3. **Sensitive Data Handling**:
   - PII is encrypted using envelope encryption
   - Encryption keys are managed using AWS KMS
   - Key rotation is implemented for long-term keys

## 📊 Monitoring and Observability

### Logging

The system implements structured logging:

```rust
pub fn log_event(event_type: &str, details: HashMap<String, Value>, level: LogLevel) {
    let log_entry = LogEntry {
        timestamp: Utc::now(),
        event_type: event_type.to_string(),
        level,
        details,
        // Additional context
    };
    
    // Log to CloudWatch
}
```

### Metrics

The system collects various metrics:

1. **Performance Metrics**:
   - API response times
   - Calculation execution times
   - Database query times

2. **Operational Metrics**:
   - Lambda invocation counts
   - Error rates
   - SQS queue depths

3. **Business Metrics**:
   - Number of portfolios
   - Number of transactions
   - Number of calculations

### Tracing

The system implements distributed tracing:

1. **Request Tracing**:
   - Generates a unique trace ID for each request
   - Propagates the trace ID through all components
   - Links related events in logs

2. **Performance Tracing**:
   - Measures execution time of key operations
   - Identifies bottlenecks in the system
   - Provides insights for optimization

## 🚀 Deployment Architecture

### AWS Infrastructure

The system is deployed using AWS CloudFormation/SAM:

```yaml
AWSTemplateFormatVersion: '2010-09-09'
Transform: AWS::Serverless-2016-10-31
Resources:
  ApiHandlerFunction:
    Type: AWS::Serverless::Function
    Properties:
      CodeUri: api-handler/
      Handler: bootstrap
      Runtime: provided.al2
      Architectures:
        - arm64
      Events:
        ApiEvent:
          Type: Api
          Properties:
            Path: /{proxy+}
            Method: ANY
      Environment:
        Variables:
          DYNAMODB_TABLE: !Ref DynamoDBTable
          SQS_QUEUE_URL: !Ref SQSQueue
          # Other environment variables
  
  PerformanceCalculatorFunction:
    Type: AWS::Serverless::Function
    Properties:
      CodeUri: performance-calculator/
      Handler: bootstrap
      Runtime: provided.al2
      Architectures:
        - arm64
      Events:
        SQSEvent:
          Type: SQS
          Properties:
            Queue: !GetAtt SQSQueue.Arn
            BatchSize: 10
      Environment:
        Variables:
          DYNAMODB_TABLE: !Ref DynamoDBTable
          TIMESTREAM_DATABASE: !Ref TimestreamDatabase
          TIMESTREAM_TABLE: !Ref TimestreamTable
          # Other environment variables
  
  # Other resources (DynamoDB, Timestream, SQS, etc.)
```

### CI/CD Pipeline

The system uses a CI/CD pipeline for automated deployment:

1. **Build Stage**:
   - Compiles the Rust code
   - Runs unit tests
   - Creates deployment packages

2. **Test Stage**:
   - Deploys to a test environment
   - Runs integration tests
   - Validates functionality

3. **Deploy Stage**:
   - Deploys to production
   - Runs smoke tests
   - Monitors for issues

## 📚 Additional Technical Resources

- [API Reference](api_reference.md): Detailed API documentation
- [Developer Guide](developer-guide.md): Guide for developers
- [Security Hardening Guide](security-hardening-guide.md): Security best practices
- [Performance Tuning Guide](performance-tuning-guide.md): Performance optimization tips 