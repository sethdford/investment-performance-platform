# Investment Management Platform API Reference

This document provides a comprehensive reference for the Investment Management Platform API, including client usage, available endpoints, and example code.

## Table of Contents

1. [API Client](#api-client)
2. [Portfolio Management API](#portfolio-management-api)
3. [Tax Optimization API](#tax-optimization-api)
4. [Household Management API](#household-management-api)
5. [Charitable Giving API](#charitable-giving-api)
6. [Risk Analysis API](#risk-analysis-api)
7. [Performance Analysis API](#performance-analysis-api)
8. [Error Handling](#error-handling)
9. [Authentication and Authorization](#authentication-and-authorization)

## API Client

The platform provides a Rust client for interacting with the API. The client handles authentication, request formatting, and response parsing.

### Creating a Client

```rust
use investment_management::api::Client;

// Create a new API client
let client = Client::new();

// Create a client with custom configuration
let client = Client::new_with_config(ClientConfig {
    base_url: "https://api.example.com".to_string(),
    timeout: Duration::from_secs(30),
    max_retries: 3,
    ..Default::default()
});
```

### Client Configuration

The client can be configured with the following options:

```rust
pub struct ClientConfig {
    /// Base URL for the API
    pub base_url: String,
    /// Request timeout
    pub timeout: Duration,
    /// Maximum number of retries
    pub max_retries: u32,
    /// Retry backoff strategy
    pub retry_strategy: RetryStrategy,
    /// Authentication configuration
    pub auth_config: AuthConfig,
}
```

## Portfolio Management API

The Portfolio Management API provides endpoints for managing investment portfolios, including model portfolios, accounts, and rebalancing.

### Model Portfolios API

#### Creating a Model Portfolio

```rust
// Create a model portfolio
let model = client.models.create(CreateModelParams {
    name: "Technology Growth".to_string(),
    securities: [
        ("AAPL".to_string(), 0.25),
        ("MSFT".to_string(), 0.25),
        ("AMZN".to_string(), 0.25),
        ("GOOGL".to_string(), 0.25),
    ].iter().cloned().collect(),
    model_type: ModelType::Direct,
    ..Default::default()
}).unwrap();
```

#### Getting a Model Portfolio

```rust
// Get a model portfolio by ID
let model = client.models.get("model-123").unwrap();
```

#### Listing Model Portfolios

```rust
// List all model portfolios
let models = client.models.list(None).unwrap();

// List model portfolios with filtering
let models = client.models.list(Some(ListModelsParams {
    model_type: Some(ModelType::Direct),
    limit: Some(10),
    offset: Some(0),
    ..Default::default()
})).unwrap();
```

#### Updating a Model Portfolio

```rust
// Update a model portfolio
let updated_model = client.models.update("model-123", UpdateModelParams {
    name: Some("Technology Growth V2".to_string()),
    securities: Some([
        ("AAPL".to_string(), 0.20),
        ("MSFT".to_string(), 0.20),
        ("AMZN".to_string(), 0.20),
        ("GOOGL".to_string(), 0.20),
        ("TSLA".to_string(), 0.20),
    ].iter().cloned().collect()),
    ..Default::default()
}).unwrap();
```

#### Deleting a Model Portfolio

```rust
// Delete a model portfolio
client.models.delete("model-123").unwrap();
```

### Accounts API

#### Creating an Account

```rust
// Create an account
let account = client.accounts.create(CreateAccountParams {
    name: "John Doe's Tech Portfolio".to_string(),
    owner: "John Doe".to_string(),
    model_id: "model-123".to_string(),
    initial_investment: 100000.0,
    ..Default::default()
}).unwrap();
```

#### Getting an Account

```rust
// Get an account by ID
let account = client.accounts.get("account-123").unwrap();
```

#### Listing Accounts

```rust
// List all accounts
let accounts = client.accounts.list(None).unwrap();

// List accounts with filtering
let accounts = client.accounts.list(Some(ListAccountsParams {
    owner: Some("John Doe".to_string()),
    limit: Some(10),
    offset: Some(0),
    ..Default::default()
})).unwrap();
```

#### Updating an Account

```rust
// Update an account
let updated_account = client.accounts.update("account-123", UpdateAccountParams {
    name: Some("John Doe's Tech Portfolio V2".to_string()),
    model_id: Some("model-456".to_string()),
    ..Default::default()
}).unwrap();
```

#### Deleting an Account

```rust
// Delete an account
client.accounts.delete("account-123").unwrap();
```

### Rebalancing API

#### Generating Rebalance Trades

```rust
// Generate rebalance trades
let trades = client.accounts.generate_rebalance_trades(
    "account-123",
    Some(RebalanceParams {
        max_trades: Some(5),
        tax_aware: true,
        min_trade_amount: Some(1000.0),
        drift_threshold: Some(0.02),
    })
).unwrap();
```

#### Executing Trades

```rust
// Execute trades
let executed_trades = client.trades.execute(
    "account-123",
    trades.iter().map(|t| t.id.clone()).collect(),
    None
).unwrap();
```

## Tax Optimization API

The Tax Optimization API provides endpoints for tax-efficient portfolio management.

### Tax-Loss Harvesting API

#### Generating Tax-Loss Harvesting Trades

```rust
// Generate tax-loss harvesting trades
let tlh_trades = client.accounts.generate_algorithmic_tlh_trades(
    "account-123",
    Some(TaxOptimizationParams {
        _enable_tax_loss_harvesting: true,
        _tax_loss_harvesting_threshold: Some(1000.0),
        _wash_sale_window_days: Some(30),
        ..Default::default()
    })
).unwrap();
```

### Tax-Efficient Asset Location API

#### Generating Asset Location Recommendations

```rust
// Generate asset location recommendations
let recommendations = client.households.generate_asset_location_recommendations(
    "household-123"
).unwrap();
```

## Household Management API

The Household Management API provides endpoints for managing household-level financial planning.

### Household API

#### Creating a Household

```rust
// Create a household
let household = client.households.create_household(
    "Smith Family",
    "John Smith"
).unwrap();
```

#### Adding a Household Member

```rust
// Add a household member
let member = client.households.add_member(
    &mut household,
    "Jane Smith",
    MemberRelationship::Spouse
).unwrap();
```

#### Adding an Account to a Household

```rust
// Add an account to a household
client.households.add_account(
    &mut household,
    account,
    vec![member.id.clone()],
    AccountTaxType::Taxable
).unwrap();
```

### Financial Goals API

#### Adding a Financial Goal

```rust
// Add a financial goal
let goal = client.households.add_financial_goal(
    &mut household,
    FinancialGoal {
        id: "goal-123".to_string(),
        name: "Retirement".to_string(),
        goal_type: GoalType::Retirement,
        target_amount: 1000000.0,
        current_amount: 100000.0,
        target_date: NaiveDate::from_ymd_opt(2050, 1, 1).unwrap(),
        status: GoalStatus::Active,
        priority: 1,
        created_at: NaiveDate::from_ymd_opt(2023, 1, 1).unwrap(),
        updated_at: NaiveDate::from_ymd_opt(2023, 1, 1).unwrap(),
        contributions: Vec::new(),
        linked_accounts: Vec::new(),
    }
).unwrap();
```

#### Tracking Goal Progress

```rust
// Track goal progress
let progress = client.households.track_goal_progress(
    &household,
    "goal-123"
).unwrap();
```

### Withdrawal Planning API

#### Generating a Withdrawal Plan

```rust
// Generate a withdrawal plan
let plan = client.households.generate_tax_efficient_withdrawal_plan(
    &household,
    50000.0,
    WithdrawalTimeframe::Annual
).unwrap();
```

## Charitable Giving API

The Charitable Giving API provides endpoints for managing charitable giving strategies.

### Charity API

#### Creating a Charity

```rust
// Create a charity
let charity = client.households.create_charity(
    &mut household,
    "American Red Cross".to_string(),
    Some("12-3456789".to_string()),
    "Humanitarian".to_string(),
    true,
    None
);
```

### Charitable Vehicle API

#### Creating a Charitable Vehicle

```rust
// Create a charitable vehicle
let vehicle = client.households.create_charitable_vehicle(
    &mut household,
    "Family Donor Advised Fund".to_string(),
    CharitableVehicleType::DonorAdvisedFund,
    None,
    100000.0,
    10000.0,
    5000.0,
    vec![("charity-123".to_string(), 1.0)],
    None
).unwrap();
```

### Donation API

#### Recording a Donation

```rust
// Record a donation
let donation = client.households.record_donation(
    &mut household,
    "charity-123".to_string(),
    Some("vehicle-123".to_string()),
    5000.0,
    NaiveDate::from_ymd_opt(2023, 9, 1).unwrap(),
    "Cash".to_string(),
    None,
    None,
    5000.0,
    2023,
    true,
    None
).unwrap();
```

#### Analyzing Charitable Tax Impact

```rust
// Analyze charitable tax impact
let tax_impact = client.households.analyze_charitable_tax_impact(
    &household,
    2023,
    0.0
);
```

#### Generating Donation Strategies

```rust
// Generate donation strategies
let strategies = client.households.generate_donation_strategies(
    &household
);
```

## Risk Analysis API

The Risk Analysis API provides endpoints for analyzing portfolio risk.

### Portfolio Risk Analysis API

#### Analyzing Portfolio Risk

```rust
// Analyze portfolio risk
let risk_analysis = client.accounts.analyze_risk(
    "account-123"
).unwrap();
```

### Household Risk Analysis API

#### Analyzing Household Risk

```rust
// Analyze household risk
let risk_analysis = client.households.analyze_household_risk(
    &household
).unwrap();
```

## Performance Analysis API

The Performance Analysis API provides endpoints for analyzing investment performance.

### Performance Metrics API

#### Calculating Performance Metrics

```rust
// Calculate performance metrics
let metrics = client.accounts.calculate_performance_metrics(
    "account-123",
    Some(PerformanceParams {
        start_date: Some(NaiveDate::from_ymd_opt(2023, 1, 1).unwrap()),
        end_date: Some(NaiveDate::from_ymd_opt(2023, 12, 31).unwrap()),
        metrics: Some(vec![
            PerformanceMetric::TWR,
            PerformanceMetric::MWR,
            PerformanceMetric::Volatility,
            PerformanceMetric::SharpeRatio,
        ]),
        ..Default::default()
    })
).unwrap();
```

### Performance Attribution API

#### Calculating Performance Attribution

```rust
// Calculate performance attribution
let attribution = client.accounts.calculate_performance_attribution(
    "account-123",
    Some(AttributionParams {
        start_date: Some(NaiveDate::from_ymd_opt(2023, 1, 1).unwrap()),
        end_date: Some(NaiveDate::from_ymd_opt(2023, 12, 31).unwrap()),
        attribution_type: Some(AttributionType::Factor),
        ..Default::default()
    })
).unwrap();
```

## Error Handling

The API client returns `Result<T, Error>` for all operations, where `Error` is an enum that represents different types of errors that can occur.

```rust
pub enum Error {
    /// API error with status code and message
    Api { status: u16, message: String },
    /// Network error
    Network(String),
    /// Serialization/deserialization error
    Serialization(String),
    /// Validation error
    Validation(String),
    /// Authentication error
    Authentication(String),
    /// Authorization error
    Authorization(String),
    /// Rate limit exceeded
    RateLimit { reset_after: Duration },
    /// Internal server error
    Internal(String),
    /// Unknown error
    Unknown(String),
}
```

Example error handling:

```rust
match client.models.get("model-123") {
    Ok(model) => {
        // Handle successful response
        println!("Model: {:?}", model);
    }
    Err(err) => match err {
        Error::Api { status, message } => {
            // Handle API error
            println!("API error ({}): {}", status, message);
        }
        Error::Network(msg) => {
            // Handle network error
            println!("Network error: {}", msg);
        }
        Error::Authentication(msg) => {
            // Handle authentication error
            println!("Authentication error: {}", msg);
        }
        // Handle other error types
        _ => println!("Other error: {:?}", err),
    }
}
```

## Authentication and Authorization

The API client supports multiple authentication methods:

### API Key Authentication

```rust
let client = Client::new_with_config(ClientConfig {
    auth_config: AuthConfig::ApiKey {
        key: "your-api-key".to_string(),
    },
    ..Default::default()
});
```

### OAuth 2.0 Authentication

```rust
let client = Client::new_with_config(ClientConfig {
    auth_config: AuthConfig::OAuth {
        client_id: "your-client-id".to_string(),
        client_secret: "your-client-secret".to_string(),
        token_url: "https://auth.example.com/token".to_string(),
        scope: Some("read write".to_string()),
    },
    ..Default::default()
});
```

### JWT Authentication

```rust
let client = Client::new_with_config(ClientConfig {
    auth_config: AuthConfig::Jwt {
        token: "your-jwt-token".to_string(),
    },
    ..Default::default()
});
```

### AWS IAM Authentication

```rust
let client = Client::new_with_config(ClientConfig {
    auth_config: AuthConfig::AwsIam {
        region: "us-west-2".to_string(),
        service: "execute-api".to_string(),
    },
    ..Default::default()
});
``` 