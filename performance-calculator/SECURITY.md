# 🔒 Security Guide

This guide outlines additional security controls, compliance requirements, and architectural principles for deploying the Rust SAM application in financial services environments.

## 📜 Regulatory Compliance

Financial services applications must comply with various regulations:

| Regulation | Description | Applies To |
|------------|-------------|------------|
| **PCI DSS** | Payment Card Industry Data Security Standard | Applications handling payment card data |
| **SOX** | Sarbanes-Oxley Act | Publicly traded companies |
| **GDPR/CCPA** | Data privacy regulations | Applications handling personal data |
| **GLBA** | Gramm-Leach-Bliley Act | Financial institutions in the US |
| **FINRA** | Financial Industry Regulatory Authority | Broker-dealers |
| **Basel III/IV** | Banking regulations | Risk management systems |

> 💡 **For beginners**: These regulations ensure that financial applications protect sensitive data and maintain accurate records. Non-compliance can result in significant fines and reputational damage.

## 🛡️ Enhanced Security Controls

### Data Protection

#### 1. **Data Encryption**

Protect data both at rest and in transit:

```yaml
# In template-production.yaml - Encryption at rest for DynamoDB
Resources:
  ItemsTable:
    Type: AWS::DynamoDB::Table
    Properties:
      SSESpecification:
        SSEEnabled: true
```

> 💡 **What this does**: Encrypts all data stored in DynamoDB so that even if someone gains unauthorized access to the raw data files, they can't read the information without the encryption keys.

#### 2. **Data Classification**

Categorize data based on sensitivity:

```rust
// Example data classification in your models
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Item {
    pub id: String,
    pub name: String,
    
    /// Classification level of this item
    /// Options: PUBLIC, INTERNAL, CONFIDENTIAL, RESTRICTED
    #[serde(default = "default_classification")]
    pub classification: String,
}

fn default_classification() -> String {
    "INTERNAL".to_string()
}
```

> 💡 **Why this matters**: Different types of data require different levels of protection. By classifying data, you can apply appropriate security controls based on sensitivity.

#### 3. **Data Masking**

Hide sensitive information in logs and responses:

```rust
// Example masking function
fn mask_sensitive_data(data: &str) -> String {
    if data.len() <= 4 {
        return "****".to_string();
    }
    let visible = &data[0..4];
    let masked = "*".repeat(data.len() - 4);
    format!("{}{}", visible, masked)
}

// Usage example
let masked_account = mask_sensitive_data(&account_number); // "1234********"
```

> 💡 **Visual example**:
> - Original: "1234567890123456"
> - Masked: "1234************"

### Access Control

#### 1. **Fine-grained IAM Permissions**

Apply the principle of least privilege:

```yaml
# In template-production.yaml - Specific permissions for Lambda
ApiHandlerRole:
  Type: AWS::IAM::Role
  Properties:
    AssumeRolePolicyDocument:
      # Standard Lambda trust policy
    Policies:
      - PolicyName: DynamoDBAccess
        PolicyDocument:
          Version: '2012-10-17'
          Statement:
            - Effect: Allow
              Action:
                - dynamodb:GetItem
                - dynamodb:PutItem
                - dynamodb:DeleteItem
                - dynamodb:Scan
              Resource: !GetAtt ItemsTable.Arn
```

> 💡 **For beginners**: This gives the Lambda function permission to perform only specific actions (get, put, delete, scan) on a specific DynamoDB table. It can't access any other resources.

#### 2. **Multi-factor Authentication (MFA)**

Require additional verification beyond passwords:

- Require MFA for AWS Console access
- Implement MFA for API access using Cognito or a custom solution

> 💡 **What is MFA?** Multi-factor authentication requires users to provide two or more verification factors to gain access, typically something they know (password) and something they have (mobile device).

#### 3. **API Authorization**

Control who can access your API:

```yaml
# In template-production.yaml - Adding Cognito authorizer to API Gateway
ApiGatewayAuthorizer:
  Type: AWS::ApiGateway::Authorizer
  Properties:
    Name: CognitoAuthorizer
    Type: COGNITO_USER_POOLS
    IdentitySource: method.request.header.Authorization
    RestApiId: !Ref ApiGateway
    ProviderARNs:
      - !GetAtt UserPool.Arn
```

> 💡 **How this works**: When a user makes an API request, they must include an Authorization header with a valid token from Cognito. API Gateway verifies this token before allowing the request to proceed.

### Audit and Compliance

#### 1. **Comprehensive Logging**

Log all important actions with context:

```rust
// Example enhanced logging
tracing::info!(
    user_id = %user_id,
    action = "create_item",
    resource_id = %item.id,
    resource_type = "item",
    result = "success",
    "User {} created item {}",
    user_id, item.id
);
```

> 💡 **Why structured logging matters**: This approach makes it easier to search, filter, and analyze logs. You can quickly find all actions by a specific user or all actions on a specific resource.

#### 2. **Audit Trail**

Keep a record of all data modifications:

```rust
// Example audit record
#[derive(Debug, Serialize, Deserialize)]
pub struct AuditRecord {
    pub event_id: String,
    pub user_id: String,
    pub action: String,
    pub resource_id: String,
    pub resource_type: String,
    pub timestamp: DateTime<Utc>,
    pub previous_state: Option<String>,
    pub new_state: Option<String>,
}

// Example function to record an audit entry
async fn record_audit(
    user_id: &str,
    action: &str,
    resource_id: &str,
    resource_type: &str,
    previous_state: Option<&str>,
    new_state: Option<&str>,
) -> Result<(), Error> {
    let audit = AuditRecord {
        event_id: Uuid::new_v4().to_string(),
        user_id: user_id.to_string(),
        action: action.to_string(),
        resource_id: resource_id.to_string(),
        resource_type: resource_type.to_string(),
        timestamp: Utc::now(),
        previous_state: previous_state.map(|s| s.to_string()),
        new_state: new_state.map(|s| s.to_string()),
    };
    
    // Store in DynamoDB
    store_audit_record(&audit).await
}
```

> 💡 **Visual example**:
> ```json
> {
>   "event_id": "550e8400-e29b-41d4-a716-446655440000",
>   "user_id": "user123",
>   "action": "update_portfolio",
>   "resource_id": "portfolio456",
>   "resource_type": "portfolio",
>   "timestamp": "2023-06-15T14:30:00Z",
>   "previous_state": "{\"name\":\"Old Name\",\"value\":1000}",
>   "new_state": "{\"name\":\"New Name\",\"value\":1500}"
> }
> ```

#### 3. **Non-repudiation**

Ensure actions can't be denied later:

- Implement digital signatures for critical transactions
- Store hash values of original requests

> 💡 **For beginners**: Non-repudiation means ensuring that a user cannot deny having performed an action. It's like having a digital signature that proves they did something.

### Secure Development

#### 1. **Dependency Scanning**

Check for vulnerabilities in dependencies:

```bash
# Example using cargo-audit
cargo install cargo-audit
cargo audit
```

> 💡 **What this does**: Scans all your project dependencies for known security vulnerabilities and alerts you if any are found.

#### 2. **Static Code Analysis**

Automatically check code for potential issues:

```bash
# Example using clippy
cargo clippy -- -D warnings
```

> 💡 **For beginners**: This is like having a code reviewer that automatically checks your code for common mistakes, security issues, and performance problems.

#### 3. **Secret Management**

Securely store and access sensitive configuration:

```rust
// Example fetching secrets from AWS Secrets Manager
async fn get_secret(secret_name: &str) -> Result<String, Error> {
    let config = aws_config::load_from_env().await;
    let client = aws_sdk_secretsmanager::Client::new(&config);
    
    let response = client
        .get_secret_value()
        .secret_id(secret_name)
        .send()
        .await?;
    
    Ok(response.secret_string().unwrap_or_default().to_string())
}

// Usage
let api_key = get_secret("financial-app/api-key").await?;
```

> 💡 **Why this matters**: Hardcoding secrets in your application is a security risk. Using a secret manager keeps sensitive information secure and allows for easy rotation.

## 🏗️ Architectural Enhancements

### Defense in Depth

#### 1. **VPC Integration**

Deploy Lambda functions within a private network:

```yaml
# In template-production.yaml
ApiHandler:
  Type: AWS::Serverless::Function
  Properties:
    # Existing properties...
    VpcConfig:
      SecurityGroupIds:
        - !Ref LambdaSecurityGroup
      SubnetIds:
        - !Ref PrivateSubnet1
        - !Ref PrivateSubnet2
```

> 💡 **Visual explanation**:
> 
> ![VPC Integration](https://via.placeholder.com/600x300?text=VPC+Integration+Diagram)
> 
> Lambda functions in a VPC can access private resources and are isolated from the public internet.

#### 2. **WAF Integration**

Protect your API from common web attacks:

```yaml
# In template-production.yaml
ApiGatewayWafAssociation:
  Type: AWS::WAFv2::WebACLAssociation
  Properties:
    ResourceArn: !Sub arn:aws:apigateway:${AWS::Region}::/restapis/${ApiGateway}/stages/Prod
    WebACLArn: !Ref WebACL

WebACL:
  Type: AWS::WAFv2::WebACL
  Properties:
    Name: FinancialAppWAF
    Scope: REGIONAL
    DefaultAction:
      Allow: {}
    Rules:
      - Name: AWSManagedRulesCommonRuleSet
        Priority: 0
        OverrideAction:
          None: {}
        Statement:
          ManagedRuleGroupStatement:
            VendorName: AWS
            Name: AWSManagedRulesCommonRuleSet
        VisibilityConfig:
          SampledRequestsEnabled: true
          CloudWatchMetricsEnabled: true
          MetricName: AWSManagedRulesCommonRuleSet
```

> 💡 **What this does**: AWS WAF (Web Application Firewall) protects your API from common web vulnerabilities like SQL injection, cross-site scripting (XSS), and more.

#### 3. **Network Security**

Control network access to your resources:

- Implement private API endpoints
- Use VPC endpoints for AWS services

> 💡 **For beginners**: VPC endpoints allow your Lambda functions to communicate with AWS services (like DynamoDB) without going through the public internet, enhancing security.

### High Availability and Disaster Recovery

#### 1. **Multi-region Deployment**

Ensure your application works even if an AWS region goes down:

- Implement active-active or active-passive multi-region setup
- Use Global DynamoDB tables for multi-region data replication

> 💡 **Visual explanation**:
> 
> ![Multi-region Deployment](https://via.placeholder.com/600x300?text=Multi-region+Deployment+Diagram)
> 
> Active-active: Your application runs in multiple regions simultaneously
> Active-passive: Your application runs in one region but can quickly switch to another if needed

#### 2. **Backup and Recovery**

Ensure you can recover data if something goes wrong:

```yaml
# In template-production.yaml
ItemsTable:
  Type: AWS::DynamoDB::Table
  Properties:
    # Existing properties...
    PointInTimeRecoverySpecification:
      PointInTimeRecoveryEnabled: true
```

> 💡 **What this does**: Enables point-in-time recovery for your DynamoDB table, allowing you to restore the table to any point in time during the last 35 days.

#### 3. **Circuit Breakers**

Prevent cascading failures when external services fail:

```rust
// Example circuit breaker pattern
struct CircuitBreaker {
    failure_threshold: u32,
    reset_timeout: Duration,
    failure_count: AtomicU32,
    last_failure: AtomicU64,
    state: AtomicU8,
}

impl CircuitBreaker {
    const CLOSED: u8 = 0;
    const OPEN: u8 = 1;
    const HALF_OPEN: u8 = 2;
    
    pub fn new(failure_threshold: u32, reset_timeout: Duration) -> Self {
        Self {
            failure_threshold,
            reset_timeout,
            failure_count: AtomicU32::new(0),
            last_failure: AtomicU64::new(0),
            state: AtomicU8::new(Self::CLOSED),
        }
    }
    
    pub fn allow_request(&self) -> bool {
        match self.state.load(Ordering::SeqCst) {
            Self::CLOSED => true,
            Self::OPEN => {
                let now = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_secs();
                let last = self.last_failure.load(Ordering::SeqCst);
                
                if now - last > self.reset_timeout.as_secs() {
                    // Try half-open state
                    self.state.store(Self::HALF_OPEN, Ordering::SeqCst);
                    true
                } else {
                    false
                }
            },
            Self::HALF_OPEN => true,
            _ => false,
        }
    }
    
    pub fn record_success(&self) {
        if self.state.load(Ordering::SeqCst) == Self::HALF_OPEN {
            // Reset on success in half-open state
            self.state.store(Self::CLOSED, Ordering::SeqCst);
            self.failure_count.store(0, Ordering::SeqCst);
        }
    }
    
    pub fn record_failure(&self) {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        self.last_failure.store(now, Ordering::SeqCst);
        
        let state = self.state.load(Ordering::SeqCst);
        if state == Self::CLOSED {
            let failures = self.failure_count.fetch_add(1, Ordering::SeqCst) + 1;
            if failures >= self.failure_threshold {
                self.state.store(Self::OPEN, Ordering::SeqCst);
            }
        } else if state == Self::HALF_OPEN {
            self.state.store(Self::OPEN, Ordering::SeqCst);
        }
    }
}

// Usage example
async fn call_external_service_with_circuit_breaker(
    circuit_breaker: &CircuitBreaker,
) -> Result<Response, Error> {
    if !circuit_breaker.allow_request() {
        return Err(Error::ServiceUnavailable("Circuit breaker open".to_string()));
    }
    
    match call_external_service().await {
        Ok(response) => {
            circuit_breaker.record_success();
            Ok(response)
        },
        Err(err) => {
            circuit_breaker.record_failure();
            Err(err)
        }
    }
}
```

> 💡 **How circuit breakers work**:
> 1. **Closed state**: All requests go through normally
> 2. **Open state**: After too many failures, requests are blocked to prevent overloading the failing service
> 3. **Half-open state**: After a timeout, a test request is allowed to check if the service has recovered

### Monitoring and Alerting

#### 1. **Enhanced Metrics**

Track important business metrics:

```rust
// Example publishing custom metrics to CloudWatch
async fn publish_metric(name: &str, value: f64, unit: &str) -> Result<(), Error> {
    let config = aws_config::load_from_env().await;
    let client = aws_sdk_cloudwatch::Client::new(&config);
    
    client
        .put_metric_data()
        .namespace("FinancialApp")
        .metric_data(
            aws_sdk_cloudwatch::model::MetricDatum::builder()
                .metric_name(name)
                .value(value)
                .unit(unit)
                .build(),
        )
        .send()
        .await?;
    
    Ok(())
}

// Usage
publish_metric("TransactionCount", 1.0, "Count").await?;
publish_metric("TransactionValue", 1000.0, "USD").await?;
```

> 💡 **Why custom metrics matter**: They help you track business-specific information that isn't captured by default AWS metrics, like transaction volumes or processing times.

#### 2. **Anomaly Detection**

Automatically detect unusual patterns:

- Set up CloudWatch Anomaly Detection
- Implement rate limiting for unusual activity

> 💡 **For beginners**: Anomaly detection uses machine learning to establish normal patterns in your metrics and alert you when something unusual happens, like a sudden spike in failed login attempts.

#### 3. **Real-time Alerting**

Get notified immediately when issues occur:

```yaml
# In template-production.yaml
ErrorAlarm:
  Type: AWS::CloudWatch::Alarm
  Properties:
    AlarmName: HighErrorRate
    AlarmDescription: Alert when error rate exceeds threshold
    MetricName: Errors
    Namespace: AWS/Lambda
    Dimensions:
      - Name: FunctionName
        Value: !Ref ApiHandlerFunction
    Statistic: Sum
    Period: 60
    EvaluationPeriods: 1
    Threshold: 5
    ComparisonOperator: GreaterThanThreshold
    AlarmActions:
      - !Ref AlertTopic

AlertTopic:
  Type: AWS::SNS::Topic
  Properties:
    DisplayName: FinancialAppAlerts
```

> 💡 **How this works**: When the Lambda function reports more than 5 errors in a 1-minute period, CloudWatch triggers an alarm that sends a notification to the SNS topic. You can subscribe to this topic via email, SMS, or other methods.

## ✅ Implementation Checklist

### Immediate Security Enhancements

- [ ] Enable encryption at rest for DynamoDB
- [ ] Implement fine-grained IAM permissions
- [ ] Enhance logging for audit purposes
- [ ] Implement input validation for all API endpoints
- [ ] Set up WAF rules for common web attacks

### Short-term Improvements

- [ ] Implement authentication and authorization
- [ ] Set up automated dependency scanning
- [ ] Configure backup and recovery
- [ ] Implement data classification and masking
- [ ] Add comprehensive error handling

### Long-term Security Roadmap

- [ ] Implement multi-region deployment
- [ ] Set up comprehensive monitoring and alerting
- [ ] Conduct regular security assessments
- [ ] Implement circuit breakers and rate limiting
- [ ] Develop a comprehensive disaster recovery plan

## 💻 Code Examples

### Enhanced Input Validation

```rust
/// Validates an item before processing
fn validate_item(item: &Item) -> Result<(), AppError> {
    // Check for empty or invalid fields
    if item.name.is_empty() {
        return Err(AppError::Validation("Item name cannot be empty".to_string()));
    }
    
    // Check for malicious content
    if item.name.contains('<') || item.name.contains('>') {
        return Err(AppError::Validation("Item name contains invalid characters".to_string()));
    }
    
    // Check description if present
    if let Some(desc) = &item.description {
        if desc.len() > 1000 {
            return Err(AppError::Validation("Description too long".to_string()));
        }
        
        // Check for malicious content
        if desc.contains('<') || desc.contains('>') {
            return Err(AppError::Validation("Description contains invalid characters".to_string()));
        }
    }
    
    Ok(())
}

// Usage example
async fn create_item(item: Item) -> Result<Response<Body>, AppError> {
    // Validate the item first
    validate_item(&item)?;
    
    // Proceed with creation
    // ...
}
```

> 💡 **Why validation matters**: Input validation prevents both accidental errors and malicious attacks. For example, checking for HTML tags helps prevent cross-site scripting (XSS) attacks.

### Rate Limiting Implementation

```rust
/// Rate limiting middleware for API requests
async fn rate_limit(
    user_id: &str,
    action: &str,
    limit: u32,
    window_seconds: u64,
) -> Result<bool, AppError> {
    let redis_client = get_redis_client().await?;
    let key = format!("rate:{}:{}", user_id, action);
    
    // Get current count
    let count: u32 = redis_client.get(&key).await.unwrap_or(0);
    
    if count >= limit {
        return Ok(false); // Rate limit exceeded
    }
    
    // Increment count and set expiry if not exists
    let _: () = redis_client.incr(&key, 1).await?;
    if count == 0 {
        let _: () = redis_client.expire(&key, window_seconds).await?;
    }
    
    Ok(true) // Request allowed
}

// Usage example
async fn handle_request(user_id: &str, action: &str) -> Result<Response<Body>, AppError> {
    // Check rate limit: 100 requests per minute
    let allowed = rate_limit(user_id, action, 100, 60).await?;
    
    if !allowed {
        return Ok(Response::builder()
            .status(429)
            .body(Body::from("Too Many Requests"))
            .unwrap());
    }
    
    // Process the request
    // ...
}
```

> 💡 **How rate limiting works**:
> 1. For each user and action, we keep a counter in Redis
> 2. Each request increments the counter
> 3. If the counter exceeds the limit, we reject the request
> 4. The counter automatically expires after the time window

### Transaction Logging

```rust
/// Logs a financial transaction with non-repudiation
async fn log_transaction(
    transaction: &Transaction,
    user_id: &str,
    request_id: &str,
) -> Result<(), AppError> {
    // Create a hash of the transaction for non-repudiation
    let transaction_json = serde_json::to_string(transaction)?;
    let transaction_hash = sha256::digest(transaction_json.as_bytes());
    
    // Create audit record
    let audit = AuditRecord {
        event_id: Uuid::new_v4().to_string(),
        user_id: user_id.to_string(),
        action: "create_transaction".to_string(),
        resource_id: transaction.id.clone(),
        resource_type: "transaction".to_string(),
        timestamp: Utc::now(),
        previous_state: None,
        new_state: Some(transaction_json),
        request_id: request_id.to_string(),
        hash: transaction_hash,
    };
    
    // Store audit record
    store_audit_record(&audit).await?;
    
    // Log for immediate visibility
    tracing::info!(
        user_id = %user_id,
        transaction_id = %transaction.id,
        amount = %transaction.amount,
        hash = %transaction_hash,
        "Transaction created"
    );
    
    Ok(())
}

// Usage example
async fn create_transaction(
    transaction: Transaction,
    user_id: &str,
    request_id: &str,
) -> Result<Response<Body>, AppError> {
    // Process the transaction
    // ...
    
    // Log the transaction for audit purposes
    log_transaction(&transaction, user_id, request_id).await?;
    
    // Return response
    // ...
}
```

> 💡 **Why transaction logging matters**: In financial applications, it's crucial to have a complete, tamper-evident record of all transactions. The hash value ensures that any changes to the transaction data can be detected.

## 📊 Compliance Documentation

Maintain documentation for each compliance requirement:

| Document | Purpose | Update Frequency |
|----------|---------|------------------|
| **Security Controls Mapping** | Maps application controls to regulatory requirements | Quarterly |
| **Risk Assessment** | Identifies and evaluates potential risks | Annually |
| **Penetration Testing Results** | Documents security testing findings | Annually |
| **Compliance Certifications** | Official compliance documentation | As required |
| **Audit Logs Retention Policy** | Defines how long audit logs are kept | Annually |

> 💡 **For beginners**: Good documentation is essential for compliance. During an audit, you'll need to prove that your application meets all regulatory requirements.

## 🔄 Regular Security Reviews

Schedule regular security reviews:

| Review Type | Frequency | Purpose |
|-------------|-----------|---------|
| **Dependency Updates** | Monthly | Keep dependencies up-to-date with security patches |
| **Security Assessments** | Quarterly | Review security controls and identify gaps |
| **Penetration Testing** | Annually | Test application for vulnerabilities |
| **Disaster Recovery Testing** | Bi-annually | Ensure recovery procedures work as expected |

> 💡 **Why regular reviews matter**: Security is not a one-time effort. Regular reviews help you stay ahead of new threats and ensure your security controls remain effective. 