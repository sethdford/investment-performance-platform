# Investment Management Platform Security Guide

This guide outlines the security practices, threat models, and implementation details for the Investment Management Platform. It is intended for developers, security engineers, and system administrators working on the platform.

## Table of Contents

1. [Security Principles](#security-principles)
2. [Threat Model](#threat-model)
3. [Authentication and Authorization](#authentication-and-authorization)
4. [Data Protection](#data-protection)
5. [API Security](#api-security)
6. [Infrastructure Security](#infrastructure-security)
7. [Secure Development Practices](#secure-development-practices)
8. [Security Testing](#security-testing)
9. [Incident Response](#incident-response)
10. [Compliance](#compliance)
11. [Security Checklist](#security-checklist)

## Security Principles

The Investment Management Platform follows these core security principles:

1. **Defense in Depth**: Multiple layers of security controls are implemented throughout the system.
2. **Least Privilege**: Users and services have only the permissions necessary to perform their functions.
3. **Secure by Default**: Security is built into the platform from the beginning, not added as an afterthought.
4. **Zero Trust**: All access requests are verified regardless of source.
5. **Data Protection**: Sensitive financial data is protected at rest and in transit.
6. **Continuous Monitoring**: Security events are continuously monitored and analyzed.

## Threat Model

### Assets to Protect

1. **Customer Financial Data**: Portfolio information, account balances, transaction history
2. **Personal Identifiable Information (PII)**: Names, addresses, tax IDs
3. **Authentication Credentials**: Passwords, API keys, tokens
4. **Intellectual Property**: Proprietary algorithms, trading strategies
5. **System Integrity**: Ensuring accurate financial calculations and transactions

### Potential Threats

1. **Unauthorized Access**: Attackers gaining access to customer accounts or admin functions
2. **Data Breaches**: Theft of sensitive financial or personal data
3. **Financial Fraud**: Manipulation of transactions or portfolio data
4. **API Abuse**: Excessive API calls, scraping, or manipulation
5. **Insider Threats**: Malicious actions by employees or contractors
6. **Denial of Service**: Attacks that make the platform unavailable
7. **Supply Chain Attacks**: Compromised dependencies or third-party services

### STRIDE Analysis

| Threat | Description | Mitigations |
|--------|-------------|-------------|
| **Spoofing** | Impersonating users or services | Multi-factor authentication, strong session management, JWT with short expiration |
| **Tampering** | Unauthorized modification of data | Input validation, parameterized queries, digital signatures, audit logs |
| **Repudiation** | Denying actions performed | Comprehensive logging, audit trails, transaction signing |
| **Information Disclosure** | Unauthorized access to data | Encryption, access controls, data minimization, secure API design |
| **Denial of Service** | Making system unavailable | Rate limiting, auto-scaling, DDoS protection, resource quotas |
| **Elevation of Privilege** | Gaining higher access levels | Principle of least privilege, role-based access control, regular permission reviews |

## Authentication and Authorization

### Authentication Methods

The platform supports multiple authentication methods:

1. **Username/Password**: With strong password policies
2. **OAuth 2.0/OpenID Connect**: For integration with identity providers
3. **API Keys**: For service-to-service communication
4. **Multi-Factor Authentication (MFA)**: Required for administrative access

### Implementation

```rust
// Authentication middleware
pub async fn authenticate(
    req: Request<Body>,
    next: Next<Body>,
) -> Result<Response<Body>, Error> {
    // Extract and validate JWT token
    let token = extract_token_from_header(&req)?;
    let claims = validate_token(token)?;
    
    // Add user info to request extension
    let mut req = req;
    req.extensions_mut().insert(UserInfo::from(claims));
    
    // Continue to next middleware or handler
    Ok(next.run(req).await)
}

// Authorization middleware
pub async fn authorize(
    req: Request<Body>,
    next: Next<Body>,
    required_permissions: &[Permission],
) -> Result<Response<Body>, Error> {
    // Get user info from request extension
    let user_info = req.extensions().get::<UserInfo>()
        .ok_or(Error::Unauthorized("User not authenticated".to_string()))?;
    
    // Check if user has required permissions
    if !has_permissions(user_info, required_permissions) {
        return Err(Error::Forbidden("Insufficient permissions".to_string()));
    }
    
    // Continue to next middleware or handler
    Ok(next.run(req).await)
}
```

### Role-Based Access Control (RBAC)

The platform implements RBAC with the following roles:

1. **Admin**: Full access to all platform features
2. **Portfolio Manager**: Can manage portfolios and execute trades
3. **Analyst**: Read-only access to portfolios and analytics
4. **Client**: Access to own accounts and portfolios only
5. **Service**: Limited API access for specific services

### Permission Model

Permissions are granular and follow the format `resource:action`:

```rust
pub enum Permission {
    PortfolioView,
    PortfolioCreate,
    PortfolioUpdate,
    PortfolioDelete,
    TradeExecute,
    UserManage,
    ReportGenerate,
    // ...
}

// Role definitions with associated permissions
pub fn get_role_permissions(role: Role) -> Vec<Permission> {
    match role {
        Role::Admin => vec![
            Permission::PortfolioView,
            Permission::PortfolioCreate,
            Permission::PortfolioUpdate,
            Permission::PortfolioDelete,
            Permission::TradeExecute,
            Permission::UserManage,
            Permission::ReportGenerate,
            // ...
        ],
        Role::PortfolioManager => vec![
            Permission::PortfolioView,
            Permission::PortfolioCreate,
            Permission::PortfolioUpdate,
            Permission::TradeExecute,
            Permission::ReportGenerate,
            // ...
        ],
        // Other roles...
    }
}
```

## Data Protection

### Encryption at Rest

All sensitive data is encrypted at rest:

1. **Database Encryption**: Using AWS DynamoDB encryption or equivalent
2. **File Encryption**: For reports and documents using AES-256
3. **Secrets Management**: Using AWS Secrets Manager or HashiCorp Vault

### Encryption in Transit

All data is encrypted in transit:

1. **TLS 1.3**: For all HTTP communications
2. **API Gateway**: Enforcing HTTPS-only connections
3. **VPC**: For internal service-to-service communication

### Data Classification

Data is classified according to sensitivity:

1. **Public**: Information that can be freely shared
2. **Internal**: Information for internal use only
3. **Confidential**: Sensitive business information
4. **Restricted**: Highly sensitive customer financial data and PII

### Data Handling

```rust
// Example of handling sensitive data
pub struct SensitiveData {
    // Fields are private and can only be accessed through methods
    name: String,
    tax_id: EncryptedField,
    account_number: EncryptedField,
}

impl SensitiveData {
    // Constructor ensures data is encrypted
    pub fn new(name: String, tax_id: &str, account_number: &str) -> Result<Self, Error> {
        Ok(Self {
            name,
            tax_id: EncryptedField::encrypt(tax_id)?,
            account_number: EncryptedField::encrypt(account_number)?,
        })
    }
    
    // Methods for controlled access to sensitive fields
    pub fn tax_id_last_four(&self) -> Result<String, Error> {
        let decrypted = self.tax_id.decrypt()?;
        Ok(mask_except_last_four(&decrypted))
    }
    
    // Full access requires explicit permission check
    pub fn full_tax_id(&self, user_info: &UserInfo) -> Result<String, Error> {
        if !user_info.has_permission(Permission::ViewFullPII) {
            return Err(Error::Forbidden("Insufficient permissions".to_string()));
        }
        self.tax_id.decrypt()
    }
}
```

## API Security

### Input Validation

All API inputs are validated:

```rust
// Example of input validation using Serde and validator
#[derive(Deserialize, Validate)]
pub struct CreatePortfolioRequest {
    #[validate(length(min = 1, max = 100))]
    pub name: String,
    
    #[validate(length(min = 3, max = 3))]
    pub currency: String,
    
    #[validate(range(min = 0.0))]
    pub initial_investment: f64,
}

// Validation middleware
pub async fn validate_request<T: DeserializeOwned + Validate>(
    req: Request<Body>,
) -> Result<T, Error> {
    let body_bytes = hyper::body::to_bytes(req.into_body()).await?;
    let request: T = serde_json::from_slice(&body_bytes)?;
    
    // Validate the request
    request.validate()?;
    
    Ok(request)
}
```

### Rate Limiting

API rate limiting is implemented to prevent abuse:

```rust
// Rate limiting middleware
pub async fn rate_limit(
    req: Request<Body>,
    next: Next<Body>,
    rate_limit_config: RateLimitConfig,
) -> Result<Response<Body>, Error> {
    let client_id = extract_client_id(&req)?;
    
    // Check rate limit
    if is_rate_limited(client_id, &rate_limit_config).await? {
        return Err(Error::TooManyRequests("Rate limit exceeded".to_string()));
    }
    
    // Continue to next middleware or handler
    Ok(next.run(req).await)
}
```

### API Versioning

APIs are versioned to ensure backward compatibility:

```
https://api.example.com/v1/portfolios
https://api.example.com/v2/portfolios
```

### Security Headers

All API responses include security headers:

```rust
// Security headers middleware
pub async fn add_security_headers(
    req: Request<Body>,
    next: Next<Body>,
) -> Result<Response<Body>, Error> {
    let mut response = next.run(req).await;
    
    // Add security headers
    let headers = response.headers_mut();
    headers.insert("Strict-Transport-Security", "max-age=31536000; includeSubDomains".parse().unwrap());
    headers.insert("Content-Security-Policy", "default-src 'self'".parse().unwrap());
    headers.insert("X-Content-Type-Options", "nosniff".parse().unwrap());
    headers.insert("X-Frame-Options", "DENY".parse().unwrap());
    headers.insert("X-XSS-Protection", "1; mode=block".parse().unwrap());
    
    Ok(response)
}
```

## Infrastructure Security

### AWS Security Configuration

The platform uses AWS services with the following security configurations:

1. **IAM**: Least privilege policies for all services
2. **VPC**: Private subnets for databases and internal services
3. **Security Groups**: Restricted network access
4. **WAF**: Web application firewall for API Gateway
5. **CloudTrail**: Audit logging for all AWS API calls
6. **GuardDuty**: Threat detection and monitoring

### Deployment Security

Secure deployment practices include:

1. **Infrastructure as Code**: Using AWS CDK or Terraform with security checks
2. **Immutable Infrastructure**: No changes to running instances
3. **Secrets Rotation**: Regular rotation of credentials and secrets
4. **Vulnerability Scanning**: Pre-deployment scanning of container images
5. **Approval Workflows**: Required approvals for production deployments

### Example Terraform Configuration

```hcl
# Example of secure S3 bucket configuration
resource "aws_s3_bucket" "reports_bucket" {
  bucket = "investment-platform-reports"
  
  # Enable versioning
  versioning {
    enabled = true
  }
  
  # Enable server-side encryption
  server_side_encryption_configuration {
    rule {
      apply_server_side_encryption_by_default {
        sse_algorithm = "AES256"
      }
    }
  }
  
  # Block public access
  block_public_acls       = true
  block_public_policy     = true
  ignore_public_acls      = true
  restrict_public_buckets = true
}

# Secure IAM policy
resource "aws_iam_policy" "lambda_reports_policy" {
  name        = "lambda-reports-policy"
  description = "Policy for Lambda to access reports bucket"
  
  policy = jsonencode({
    Version = "2012-10-17"
    Statement = [
      {
        Action = [
          "s3:GetObject",
          "s3:PutObject",
        ]
        Effect   = "Allow"
        Resource = "${aws_s3_bucket.reports_bucket.arn}/*"
      }
    ]
  })
}
```

## Secure Development Practices

### Secure Coding Guidelines

1. **Input Validation**: Validate all inputs for type, length, format, and range
2. **Output Encoding**: Encode outputs to prevent injection attacks
3. **Error Handling**: Use generic error messages for users, detailed logs for debugging
4. **Memory Safety**: Leverage Rust's memory safety features
5. **Dependency Management**: Regularly update and audit dependencies

### Code Review Security Checklist

All code reviews should check for:

1. **Authentication**: Proper authentication for all endpoints
2. **Authorization**: Correct permission checks
3. **Input Validation**: Comprehensive validation of all inputs
4. **SQL Injection**: Use of parameterized queries or ORM
5. **Secrets Management**: No hardcoded secrets
6. **Error Handling**: Proper error handling without information leakage
7. **Logging**: Appropriate logging without sensitive data

### Dependency Management

```rust
// Example Cargo.toml with version pinning and security features
[dependencies]
tokio = { version = "1.28.0", features = ["full"] }
hyper = { version = "0.14.26", features = ["full"] }
aws-sdk-dynamodb = { version = "0.28.0", features = ["rt-tokio"] }
serde = { version = "1.0.163", features = ["derive"] }
serde_json = "1.0.96"
jsonwebtoken = "8.3.0"
argon2 = "0.5.0"
validator = { version = "0.16.0", features = ["derive"] }
tracing = "0.1.37"
```

## Security Testing

### SAST (Static Application Security Testing)

Static analysis tools are integrated into the CI/CD pipeline:

1. **Cargo Audit**: Checks for known vulnerabilities in dependencies
2. **Clippy**: Rust linter with security-related checks
3. **SonarQube**: Code quality and security analysis

### DAST (Dynamic Application Security Testing)

Dynamic testing is performed regularly:

1. **OWASP ZAP**: Automated scanning for common vulnerabilities
2. **API Fuzzing**: Testing API endpoints with unexpected inputs
3. **Penetration Testing**: Annual third-party penetration tests

### Example CI/CD Security Steps

```yaml
# GitHub Actions workflow with security checks
name: Security Checks

on:
  push:
    branches: [ main ]
  pull_request:
    branches: [ main ]

jobs:
  security:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      
      - name: Install Rust
        uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
          override: true
      
      - name: Cargo Audit
        uses: actions-rs/audit-check@v1
        with:
          token: ${{ secrets.GITHUB_TOKEN }}
      
      - name: Clippy
        uses: actions-rs/clippy-check@v1
        with:
          token: ${{ secrets.GITHUB_TOKEN }}
          args: -- -D warnings
      
      - name: OWASP Dependency Check
        uses: dependency-check/Dependency-Check_Action@main
        with:
          project: 'Investment Management Platform'
          path: '.'
          format: 'HTML'
          out: 'reports'
      
      - name: Upload Security Report
        uses: actions/upload-artifact@v3
        with:
          name: security-report
          path: reports/
```

## Incident Response

### Incident Response Plan

The platform has a defined incident response plan:

1. **Detection**: Monitoring and alerting for security events
2. **Analysis**: Determining the scope and impact of the incident
3. **Containment**: Limiting the damage of the incident
4. **Eradication**: Removing the cause of the incident
5. **Recovery**: Restoring systems to normal operation
6. **Post-Incident Review**: Learning from the incident

### Security Monitoring

The platform implements comprehensive security monitoring:

1. **CloudWatch Logs**: Centralized logging for all components
2. **CloudTrail**: Audit logging for AWS API calls
3. **GuardDuty**: Threat detection for AWS resources
4. **Custom Alerts**: Application-specific security alerts

### Example Monitoring Configuration

```rust
// Example of security event logging
pub fn log_security_event(
    event_type: SecurityEventType,
    user_id: Option<String>,
    resource: &str,
    action: &str,
    status: EventStatus,
    details: Option<serde_json::Value>,
) {
    let event = SecurityEvent {
        timestamp: Utc::now(),
        event_type,
        user_id,
        resource: resource.to_string(),
        action: action.to_string(),
        status,
        details,
        request_id: get_current_request_id(),
        source_ip: get_current_source_ip(),
    };
    
    // Log the event
    tracing::info!(
        event_type = ?event.event_type,
        user_id = ?event.user_id,
        resource = %event.resource,
        action = %event.action,
        status = ?event.status,
        request_id = %event.request_id,
        source_ip = %event.source_ip,
        "Security event"
    );
    
    // For high-severity events, send an alert
    if event.event_type.severity() >= Severity::High {
        send_security_alert(&event);
    }
}
```

## Compliance

### Regulatory Compliance

The platform is designed to comply with relevant financial regulations:

1. **SOC 2**: Security, availability, processing integrity, confidentiality, and privacy
2. **GDPR**: Data protection and privacy for EU citizens
3. **CCPA**: California Consumer Privacy Act
4. **FINRA**: Financial Industry Regulatory Authority rules
5. **SEC**: Securities and Exchange Commission requirements

### Compliance Controls

Key compliance controls include:

1. **Access Controls**: Strict access controls with audit trails
2. **Data Retention**: Policies for data retention and deletion
3. **Privacy Controls**: Data minimization and purpose limitation
4. **Audit Logging**: Comprehensive logging of all actions
5. **Regular Audits**: Internal and external security audits

### Example Compliance Implementation

```rust
// Example of implementing data retention policy
pub async fn apply_data_retention_policy() -> Result<(), Error> {
    // Get retention configuration
    let config = get_data_retention_config().await?;
    
    // Apply retention policy to transaction history
    let cutoff_date = Utc::now() - Duration::days(config.transaction_history_days as i64);
    archive_transactions_before_date(cutoff_date).await?;
    
    // Apply retention policy to audit logs
    let audit_cutoff_date = Utc::now() - Duration::days(config.audit_log_days as i64);
    archive_audit_logs_before_date(audit_cutoff_date).await?;
    
    // Apply retention policy to reports
    let report_cutoff_date = Utc::now() - Duration::days(config.report_days as i64);
    archive_reports_before_date(report_cutoff_date).await?;
    
    // Log completion of retention policy application
    log_security_event(
        SecurityEventType::DataRetention,
        None,
        "all",
        "apply_retention_policy",
        EventStatus::Success,
        Some(json!({
            "transaction_cutoff": cutoff_date,
            "audit_cutoff": audit_cutoff_date,
            "report_cutoff": report_cutoff_date,
        })),
    );
    
    Ok(())
}
```

## Security Checklist

### Pre-Deployment Security Checklist

Before deploying to production, verify:

1. **Authentication**: All endpoints require proper authentication
2. **Authorization**: Permission checks are in place for all actions
3. **Encryption**: Sensitive data is encrypted at rest and in transit
4. **Input Validation**: All inputs are properly validated
5. **Dependency Scanning**: No known vulnerabilities in dependencies
6. **Secrets Management**: No hardcoded secrets, proper secrets rotation
7. **Logging**: Appropriate logging without sensitive data
8. **Rate Limiting**: API rate limiting is configured
9. **Error Handling**: Proper error handling without information leakage
10. **Security Headers**: All required security headers are set

### Regular Security Tasks

Perform these security tasks regularly:

1. **Dependency Updates**: Update dependencies and check for vulnerabilities weekly
2. **Penetration Testing**: Conduct penetration testing annually
3. **Security Training**: Provide security training for developers bi-annually
4. **Access Review**: Review access permissions quarterly
5. **Incident Response Drill**: Practice incident response annually
6. **Compliance Audit**: Conduct compliance audit annually
7. **Threat Model Review**: Review and update threat model annually

## Conclusion

Security is a critical aspect of the Investment Management Platform. By following the practices outlined in this guide, we can protect sensitive financial data, maintain customer trust, and comply with regulatory requirements.

Remember that security is an ongoing process, not a one-time effort. Stay vigilant, keep up with security best practices, and continuously improve the platform's security posture. 