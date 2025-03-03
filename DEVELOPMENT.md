# 🛠️ Development Guide for Rust Investment Performance Application

This guide provides detailed information for developers working on the Rust Investment Performance application, with a focus on making it accessible for engineers of all experience levels.

## 🚀 Getting Started

### 📋 Prerequisites

Before you begin, you'll need to install the following tools:

| Tool | Purpose | Installation Command | Notes |
|------|---------|---------------------|-------|
| **Rust** | Programming language | `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs \| sh` | This installs the Rust compiler and Cargo package manager |
| **AWS SAM CLI** | Local testing and deployment | See installation instructions below | Used for local testing and deploying to AWS |
| **Docker** | Container platform | See installation instructions below | Required for local testing with SAM CLI |
| **AWS CLI** | AWS command line tool | `pip install awscli` | Used to interact with AWS services |

#### Installing AWS SAM CLI

```bash
# macOS
brew tap aws/tap
brew install aws-sam-cli

# Linux/Windows
# See https://docs.aws.amazon.com/serverless-application-model/latest/developerguide/serverless-sam-cli-install.html
```

> 💡 **Tip for beginners**: SAM (Serverless Application Model) is an AWS framework that makes it easier to build and deploy serverless applications.

#### Installing Docker

```bash
# macOS
brew install --cask docker

# Linux/Windows
# See https://docs.docker.com/get-docker/
```

#### Configuring AWS CLI

```bash
# Run this command and follow the prompts
aws configure

# You'll need to provide:
# - AWS Access Key ID
# - AWS Secret Access Key
# - Default region (e.g., us-east-1)
# - Default output format (json is recommended)
```

> ⚠️ **Important**: Never commit your AWS credentials to version control!

### 📥 Clone the Repository

```bash
# Clone the repository
git clone https://github.com/yourusername/rust-investment-performance.git

# Navigate to the project directory
cd rust-investment-performance
```

## 📂 Project Structure

The application is organized as a workspace with multiple components:

```
rust-investment-performance/
├── api-handler/            # Lambda function for handling API Gateway requests
│   ├── src/
│   │   ├── main.rs         # Entry point for the API handler
│   │   └── tests.rs        # Tests for the API handler
│   └── Cargo.toml          # Dependencies for the API handler
├── event-processor/        # Lambda function for processing SQS events
│   ├── src/
│   │   ├── main.rs         # Entry point for the event processor
│   │   └── tests.rs        # Tests for the event processor
│   └── Cargo.toml          # Dependencies for the event processor
├── performance-calculator/ # Performance calculation Lambda function
│   ├── src/
│   │   ├── main.rs         # Entry point for the calculator
│   │   └── calculations/   # Performance calculation modules
│   └── Cargo.toml          # Dependencies for the calculator
├── shared/                 # Common code shared between Lambda functions
│   ├── src/
│   │   ├── lib.rs          # Library entry point
│   │   ├── models.rs       # Data models
│   │   ├── repository.rs   # Database access
│   │   ├── error.rs        # Error handling
│   │   └── config.rs       # Configuration management
│   └── Cargo.toml          # Dependencies for the shared library
├── events/                 # Sample events for testing
│   └── test/
│       ├── get-items.json  # Test event for API handler
│       └── sqs-event.json  # Test event for event processor
├── template-production.yaml # AWS SAM template for production deployment
├── template-development.yml # AWS SAM template for development/testing
└── Cargo.toml              # Workspace configuration
```

> 💡 **For beginners**: A workspace in Rust allows you to manage multiple related packages (crates) from a single location. Each component has its own `Cargo.toml` file that defines its dependencies.

## 🏗️ Building the Application

You can build the application using either SAM CLI or Cargo:

### Using SAM CLI (Recommended for Deployment)

```bash
# Build the entire application
sam build --template template-production.yaml

# For development deployment
sam build --template template-development.yml
```

This command:
1. Reads the appropriate template file (`template-production.yaml` for production or `template-development.yml` for development)
2. Builds each Lambda function
3. Prepares the application for local testing or deployment

To specify which template to use, add the `--template` parameter:
```bash
# For production deployment
sam build --template template-production.yaml

# For development deployment
sam build --template template-development.yml
```

### Using Cargo (Recommended for Development)

```bash
# Build the entire workspace in debug mode
cargo build

# Build for release (optimized)
cargo build --release

# Build a specific component
cargo build -p api-handler
```

> 💡 **What's the difference?** 
> - `cargo build` is faster and includes debug information, making it better for development
> - `sam build` packages the application for AWS Lambda, making it ready for deployment

## 💻 Local Development

### Running the API Locally

```bash
# Start a local API Gateway
sam local start-api

# The API will be available at http://localhost:3000
```

Example API requests:

```bash
# Get all items
curl http://localhost:3000/items

# Get a specific item
curl http://localhost:3000/items/123

# Create a new item
curl -X POST http://localhost:3000/items -d '{"id": "456", "name": "New Item"}'

# Delete an item
curl -X DELETE http://localhost:3000/items/456
```

### Invoking Lambda Functions Locally

```bash
# Invoke the API handler with a test event
sam local invoke ApiHandler -e events/test/get-items.json

# Invoke the event processor with a test event
sam local invoke EventProcessor -e events/test/sqs-event.json

# Invoke the performance calculator with a test event
sam local invoke PerformanceCalculator -e events/test/performance-calculation-event.json
```

### Setting Up Local Services

#### Local DynamoDB

For local development, you can use DynamoDB Local to avoid using the actual AWS service:

```bash
# Start DynamoDB Local in a Docker container
docker run -p 8000:8000 amazon/dynamodb-local

# Create a table
aws dynamodb create-table \
    --table-name Items \
    --attribute-definitions AttributeName=id,AttributeType=S \
    --key-schema AttributeName=id,KeyType=HASH \
    --billing-mode PAY_PER_REQUEST \
    --endpoint-url http://localhost:8000
```

Set the environment variables to use DynamoDB Local:

```bash
export DYNAMODB_TABLE=Items
export AWS_ENDPOINT_URL=http://localhost:8000
```

#### Local SQS

For local development, you can use LocalStack for SQS:

```bash
# Start LocalStack in a Docker container
docker run -p 4566:4566 localstack/localstack

# Create an SQS queue
aws sqs create-queue \
    --queue-name ItemEvents \
    --endpoint-url http://localhost:4566
```

Set the environment variable to use the local SQS queue:

```bash
export EVENT_QUEUE_URL=http://localhost:4566/000000000000/ItemEvents
```

> 💡 **Tip**: You can create a `.env` file with these environment variables and use a tool like `direnv` to automatically load them when you enter the project directory.

## 🧩 Code Organization

### API Handler

The API handler (`api-handler/src/main.rs`) processes HTTP requests from API Gateway:

```rust
// Simplified example of the API handler
#[tokio::main]
async fn main() -> Result<(), Error> {
    // Initialize the Lambda function
    lambda_http::run(handler).await?;
    Ok(())
}

async fn handler(event: Request, _: Context) -> Result<Response<Body>, Error> {
    // Extract HTTP method and path
    let method = event.method().as_str();
    let path = event.uri().path();
    
    // Route the request to the appropriate handler
    match (method, path) {
        ("GET", "/items") => get_items().await,
        ("GET", p) if p.starts_with("/items/") => {
            let id = p.trim_start_matches("/items/");
            get_item(id).await
        },
        ("POST", "/items") => create_item(event.body()).await,
        ("DELETE", p) if p.starts_with("/items/") => {
            let id = p.trim_start_matches("/items/");
            delete_item(id).await
        },
        _ => Ok(Response::builder()
            .status(404)
            .body(Body::from("Not Found"))
            .unwrap())
    }
}
```

Key functions:
- `handler`: Routes requests to the appropriate handler
- `get_items`: Lists all items
- `get_item`: Gets a specific item
- `create_item`: Creates a new item
- `delete_item`: Deletes an item

### Event Processor

The event processor (`event-processor/src/main.rs`) processes events from SQS:

```rust
// Simplified example of the event processor
#[tokio::main]
async fn main() -> Result<(), Error> {
    // Initialize the Lambda function
    lambda_runtime::run(handler).await?;
    Ok(())
}

async fn handler(event: SqsEvent, _: Context) -> Result<(), Error> {
    // Process each SQS message
    for record in event.records {
        process_sqs_message(&record).await?;
    }
    Ok(())
}

async fn process_sqs_message(record: &SqsRecord) -> Result<(), Error> {
    // Parse the message body
    let body = record.body.as_ref().unwrap();
    let event: ItemEvent = serde_json::from_str(body)?;
    
    // Process the event based on its type
    match event.event_type.as_str() {
        "CREATED" => process_item_created(event).await?,
        "DELETED" => process_item_deleted(event).await?,
        _ => log::warn!("Unknown event type: {}", event.event_type),
    }
    Ok(())
}
```

Key functions:
- `handler`: Processes SQS events
- `process_sqs_message`: Processes a single SQS message
- `process_item_created`: Handles item creation events
- `process_item_deleted`: Handles item deletion events

### Shared Library

The shared library (`shared/src/lib.rs`) contains common code used by both Lambda functions:

```rust
// Simplified example of the shared library
pub mod models;
pub mod repository;
pub mod error;
pub mod config;

// Example of a model
pub mod models {
    use serde::{Deserialize, Serialize};
    
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct Item {
        pub id: String,
        pub name: String,
        pub created_at: String,
    }
    
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct ItemEvent {
        pub event_type: String,
        pub item: Item,
    }
}

// Example of a repository
pub mod repository {
    use aws_sdk_dynamodb::Client as DynamoDbClient;
    use crate::models::Item;
    
    pub struct DynamoDbRepository {
        client: DynamoDbClient,
        table_name: String,
    }
    
    impl DynamoDbRepository {
        pub fn new(client: DynamoDbClient, table_name: String) -> Self {
            Self { client, table_name }
        }
        
        pub async fn get_item(&self, id: &str) -> Result<Option<Item>, aws_sdk_dynamodb::Error> {
            // Implementation
        }
        
        pub async fn put_item(&self, item: &Item) -> Result<(), aws_sdk_dynamodb::Error> {
            // Implementation
        }
        
        pub async fn delete_item(&self, id: &str) -> Result<(), aws_sdk_dynamodb::Error> {
            // Implementation
        }
    }
}
```

## ✨ Adding a New Feature

To add a new feature to the application, follow these steps:

### 1. Update the Shared Library (if needed)

If your feature requires new data models or repository methods:

```rust
// Add a new model in shared/src/models.rs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Portfolio {
    pub id: String,
    pub name: String,
    pub client_id: String,
    pub created_at: String,
}

// Add a new repository method in shared/src/repository.rs
impl DynamoDbRepository {
    pub async fn get_portfolios_by_client(&self, client_id: &str) -> Result<Vec<Portfolio>, aws_sdk_dynamodb::Error> {
        // Implementation
    }
}
```

### 2. Update the API Handler (if needed)

If your feature requires a new API endpoint:

```rust
// Add a new handler function in api-handler/src/main.rs
async fn get_portfolios_by_client(
    repo: &DynamoDbRepository,
    client_id: &str,
) -> Result<Response<Body>, AppError> {
    let portfolios = repo.get_portfolios_by_client(client_id).await?;
    let json = serde_json::to_string(&portfolios)?;
    Ok(Response::builder()
        .status(200)
        .header("Content-Type", "application/json")
        .body(Body::from(json))
        .unwrap())
}

// Update the request handler to route the new endpoint
let result = match (method.as_str(), path.as_str()) {
    // Existing routes...
    ("GET", p) if p.starts_with("/clients/") && p.ends_with("/portfolios") => {
        let parts: Vec<&str> = p.split('/').collect();
        if parts.len() == 4 {
            let client_id = parts[2];
            get_portfolios_by_client(repo, client_id).await
        } else {
            Ok(error_response("Invalid path".to_string(), 400))
        }
    },
    // Other routes...
};
```

### 3. Update the Event Processor (if needed)

If your feature requires handling new event types:

```rust
// Add a new event type in shared/src/models.rs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortfolioEvent {
    pub event_type: String,
    pub portfolio: Portfolio,
}

// Add a new handler function in event-processor/src/main.rs
async fn process_portfolio_created(event: PortfolioEvent) -> Result<(), Error> {
    // Implementation
    Ok(())
}

// Update the message processor to handle the new event type
async fn process_sqs_message(record: &SqsRecord) -> Result<(), Error> {
    // Parse the message body
    let body = record.body.as_ref().unwrap();
    
    // Try to parse as different event types
    if let Ok(event) = serde_json::from_str::<ItemEvent>(body) {
        // Process item event
        // ...
    } else if let Ok(event) = serde_json::from_str::<PortfolioEvent>(body) {
        // Process portfolio event
        match event.event_type.as_str() {
            "CREATED" => process_portfolio_created(event).await?,
            // Other event types...
            _ => log::warn!("Unknown portfolio event type: {}", event.event_type),
        }
    } else {
        log::warn!("Unknown event format: {}", body);
    }
    
    Ok(())
}
```

### 4. Add Tests

Always add tests for your new feature:

```rust
// Add a test in api-handler/src/tests.rs
#[tokio::test]
async fn test_get_portfolios_by_client() {
    // Set up test data
    let client_id = "client123";
    let portfolios = vec![
        Portfolio {
            id: "portfolio1".to_string(),
            name: "Portfolio 1".to_string(),
            client_id: client_id.to_string(),
            created_at: "2023-01-01T00:00:00Z".to_string(),
        },
        Portfolio {
            id: "portfolio2".to_string(),
            name: "Portfolio 2".to_string(),
            client_id: client_id.to_string(),
            created_at: "2023-01-02T00:00:00Z".to_string(),
        },
    ];
    
    // Create a mock repository
    let mut mock_repo = MockDynamoDbRepository::new();
    mock_repo.expect_get_portfolios_by_client()
        .with(eq(client_id))
        .returning(move |_| Ok(portfolios.clone()));
    
    // Call the handler
    let response = get_portfolios_by_client(&mock_repo, client_id).await.unwrap();
    
    // Verify the response
    assert_eq!(response.status(), 200);
    let body = response.body();
    let body_str = match body {
        Body::Text(text) => text,
        Body::Binary(bytes) => std::str::from_utf8(bytes).unwrap(),
        _ => panic!("Unexpected body type"),
    };
    let response_portfolios: Vec<Portfolio> = serde_json::from_str(body_str).unwrap();
    assert_eq!(response_portfolios.len(), 2);
    assert_eq!(response_portfolios[0].id, "portfolio1");
    assert_eq!(response_portfolios[1].id, "portfolio2");
}
```

### 5. Update the SAM Template (if needed)

If your feature requires new AWS resources or permissions:

```yaml
# Add a new resource in the appropriate template file
# (template-production.yaml for production or template-development.yml for development)
Resources:
  # Existing resources...
  
  PortfoliosTable:
    Type: AWS::DynamoDB::Table
    Properties:
      TableName: !Sub ${Environment}-Portfolios
      BillingMode: PAY_PER_REQUEST
      AttributeDefinitions:
        - AttributeName: id
          AttributeType: S
        - AttributeName: client_id
          AttributeType: S
      KeySchema:
        - AttributeName: id
          KeyType: HASH
      GlobalSecondaryIndexes:
        - IndexName: ClientIdIndex
          KeySchema:
            - AttributeName: client_id
              KeyType: HASH
          Projection:
            ProjectionType: ALL
  
  # Update the API handler's permissions
  ApiHandlerFunction:
    Type: AWS::Serverless::Function
    Properties:
      # Existing properties...
      Policies:
        - DynamoDBCrudPolicy:
            TableName: !Ref PortfoliosTable
        # Other policies...
```

## 🚀 Deployment

### Deploying to AWS

```bash
# First-time deployment (interactive)
sam deploy --guided

# Subsequent deployments
sam deploy
```

During the guided deployment, you'll be asked to provide:
- Stack name (e.g., `rust-investment-performance`)
- AWS Region (e.g., `us-east-1`)
- Parameter values (see below)
- Confirmation of IAM role creation
- Deployment preferences

### Deployment Parameters

The SAM template supports the following parameters:

| Parameter | Description | Example Value |
|-----------|-------------|---------------|
| `Environment` | Deployment environment | `dev`, `test`, `prod` |
| `DynamoDBTableName` | DynamoDB table name prefix | `Items` |
| `SQSQueueName` | SQS queue name prefix | `ItemEvents` |

Example deployment with parameters:

```bash
sam deploy --parameter-overrides \
  Environment=dev \
  DynamoDBTableName=Items \
  SQSQueueName=ItemEvents
```

## 🔍 Monitoring and Debugging

### CloudWatch Logs

You can view the Lambda function logs in CloudWatch:

```bash
# View API handler logs
aws logs filter-log-events \
  --log-group-name /aws/lambda/rust-investment-performance-ApiHandler

# View event processor logs
aws logs filter-log-events \
  --log-group-name /aws/lambda/rust-investment-performance-EventProcessor

# View performance calculator logs
aws logs filter-log-events \
  --log-group-name /aws/lambda/rust-investment-performance-PerformanceCalculator
```

> 💡 **Tip**: You can use the CloudWatch Logs Insights feature in the AWS Console for more advanced querying of logs.

### X-Ray Tracing

The application is configured for X-Ray tracing, which helps you analyze and debug distributed applications:

1. Open the AWS X-Ray console
2. Select "Traces" from the left navigation
3. Filter by time range or trace ID
4. Click on a trace to see detailed information

X-Ray shows you:
- The complete request flow
- Time spent in each component
- Errors and exceptions
- Dependencies between services

## 🏆 Best Practices

### Error Handling

Use the `AppError` enum for application-specific errors:

```rust
// In shared/src/error.rs
#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("Item not found: {0}")]
    ItemNotFound(String),
    
    #[error("DynamoDB error: {0}")]
    DynamoDb(#[from] aws_sdk_dynamodb::Error),
    
    #[error("SQS error: {0}")]
    Sqs(#[from] aws_sdk_sqs::Error),
    
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
    
    #[error("Internal error: {0}")]
    Internal(String),
}

// Example usage
fn get_item(id: &str) -> Result<Item, AppError> {
    let item = repository.get_item(id).await?;
    item.ok_or_else(|| AppError::ItemNotFound(id.to_string()))
}
```

### Logging

Use structured logging with the `tracing` crate:

```rust
// Initialize tracing in main.rs
fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .json()
        .init();
    
    // Rest of the code...
}

// Example log statements
tracing::info!(request_id = %request_id, "Processing request");
tracing::warn!(request_id = %request_id, error = %err, "Warning occurred");
tracing::error!(request_id = %request_id, error = %err, "Error occurred");
```

### Security

Follow these security best practices:

| Practice | Description | Example |
|----------|-------------|---------|
| **Least Privilege** | Give Lambda functions only the permissions they need | Use specific IAM policies in the SAM template |
| **Input Validation** | Validate all input data | Use Rust's type system and validation libraries |
| **Environment Variables** | Use environment variables for configuration | Store sensitive values in AWS Secrets Manager |
| **No Sensitive Logging** | Don't log sensitive information | Redact sensitive fields before logging |
| **HTTPS Only** | Use HTTPS for all API endpoints | Configure API Gateway to require HTTPS |

### Performance

Optimize your Lambda functions:

| Practice | Description | Example |
|----------|-------------|---------|
| **Small Functions** | Keep Lambda functions small and focused | Split large functions into smaller ones |
| **Async/Await** | Use async/await for I/O-bound operations | Use Tokio for asynchronous programming |
| **Minimize Dependencies** | Keep dependencies minimal | Only include what you need in Cargo.toml |
| **Connection Pooling** | Reuse connections across invocations | Store clients in static variables with `lazy_static` |
| **Caching** | Cache frequently accessed data | Use DynamoDB DAX or in-memory caching |

## ❓ Troubleshooting

### Common Issues and Solutions

#### 1. Lambda Function Times Out

**Issue**: The Lambda function takes too long to execute and times out.

**Solution**:
- Increase the timeout in the SAM template:
  ```yaml
  ApiHandlerFunction:
    Type: AWS::Serverless::Function
    Properties:
      Timeout: 30  # Increase from default 3 seconds
  ```
- Optimize the function code to complete faster:
  - Use async/await for I/O operations
  - Implement caching
  - Reduce unnecessary processing

#### 2. Permission Denied Errors

**Issue**: The Lambda function doesn't have permission to access AWS resources.

**Solution**:
- Check the IAM role permissions in the SAM template:
  ```yaml
  ApiHandlerFunction:
    Type: AWS::Serverless::Function
    Properties:
      Policies:
        - DynamoDBCrudPolicy:
            TableName: !Ref ItemsTable
        - SQSSendMessagePolicy:
            QueueName: !GetAtt ItemEventsQueue.QueueName
  ```
- Check CloudTrail logs for specific permission errors
- Use the AWS Policy Simulator to test IAM policies

#### 3. DynamoDB Errors

**Issue**: The Lambda function can't access DynamoDB.

**Solution**:
- Check the DynamoDB table name in the environment variables:
  ```bash
  # In the Lambda function
  let table_name = std::env::var("DYNAMODB_TABLE")
      .expect("DYNAMODB_TABLE environment variable not set");
  ```
- Ensure the table exists and has the correct schema
- Check that the Lambda function has permission to access the table

#### 4. SQS Errors

**Issue**: The Lambda function can't send messages to SQS.

**Solution**:
- Check the SQS queue URL in the environment variables:
  ```bash
  # In the Lambda function
  let queue_url = std::env::var("EVENT_QUEUE_URL")
      .expect("EVENT_QUEUE_URL environment variable not set");
  ```
- Ensure the queue exists
- Check that the Lambda function has permission to send messages to the queue

#### 5. Cold Start Latency

**Issue**: The Lambda function has high latency on first invocation.

**Solution**:
- Minimize dependencies to reduce code size
- Use Provisioned Concurrency for critical functions
- Implement a warm-up mechanism (e.g., periodic pings)
- Consider using AWS Lambda SnapStart for Java functions

### Getting Help

If you're still having issues:

1. Check the CloudWatch Logs for error messages
2. Look at X-Ray traces for performance bottlenecks
3. Review the AWS documentation for the services you're using
4. Ask for help in the team Slack channel
5. Open an issue in the GitHub repository 