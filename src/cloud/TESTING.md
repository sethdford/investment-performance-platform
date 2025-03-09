# Testing the AWS TLH Alert System

This document provides guidance on testing the AWS Tax-Loss Harvesting (TLH) Alert System using the mock AWS clients.

## Overview

The AWS TLH Alert System is designed with testability in mind, using a trait-based approach that allows for easy mocking of AWS services. This enables comprehensive testing without requiring actual AWS credentials or making real API calls.

## Testing Framework

The testing framework consists of the following components:

1. **Trait Definitions**: Each AWS service client has a corresponding trait that defines its interface.
2. **Mock Implementations**: Mock implementations of these traits using the `mockall` crate.
3. **Test-Specific Service Implementation**: The `AwsTlhAlertService` has test-specific implementations that accept mock clients.
4. **Test Utilities**: Helper functions for creating test data and setting up mock expectations.

## Mock AWS Service

The `MockAwsService` class provides a convenient way to set up mock AWS clients for testing. It includes methods for:

- Setting up Bedrock predictions
- Setting up SageMaker predictions
- Setting up DynamoDB storage
- Setting up SNS notifications
- Setting up Lambda invocations
- Setting up CloudWatch metrics
- Setting up EventBridge events

## Writing Tests

### Basic Test Structure

```rust
#[tokio::test]
async fn test_some_functionality() {
    // 1. Create test data
    let account = create_test_account();
    
    // 2. Configure the AWS TLH Alert System
    let aws_config = AwsTlhAlertConfig {
        // ... configuration ...
    };
    
    // 3. Configure the Algorithmic TLH Service
    let tlh_config = AlgorithmicTLHConfig::default();
    
    // 4. Create mock clients
    let mut bedrock_mock = MockBedrockClientMock::new();
    // Set up expectations for the Bedrock client
    
    // 5. Create the AWS TLH Alert Service with mock clients
    let mut alert_service = AwsTlhAlertService::with_mock_clients(
        aws_config,
        tlh_config,
        Some(Box::new(bedrock_mock)),
        // ... other mock clients ...
    );
    
    // 6. Call the method being tested
    let result = alert_service.some_method().await;
    
    // 7. Verify the results
    assert!(result.is_ok());
    // ... more assertions ...
}
```

### Setting Up Mock Expectations

Mock expectations define how the mock clients should behave when called. For example:

```rust
let mut bedrock_mock = MockBedrockClientMock::new();
bedrock_mock.expect_invoke_model()
    .returning(|request| {
        // Create a mock response
        let response_json = json!({
            "content": [{
                "text": "Based on my analysis, here's the prediction for AAPL: {\"predictedPriceChange\": -0.05, \"confidence\": 0.85, \"rationale\": \"Market conditions suggest this movement.\"}"
            }]
        });
        
        let response_bytes = serde_json::to_vec(&response_json).unwrap();
        
        Ok(InvokeModelOutput::builder()
            .body(Blob::new(response_bytes))
            .build())
    });
```

### Testing Specific Scenarios

#### Testing Market Predictions

```rust
#[tokio::test]
async fn test_get_bedrock_prediction() {
    // ... setup ...
    
    // Set up mock for Bedrock client
    let expected_prediction = create_market_prediction("AAPL", -0.05, 0.85);
    bedrock_mock.expect_invoke_model()
        .returning(move |_| {
            // ... create mock response ...
        });
    
    // Get prediction
    let prediction_result = alert_service.get_bedrock_prediction("AAPL").await;
    
    // Verify the prediction
    assert!(prediction_result.is_ok());
    let prediction = prediction_result.unwrap().unwrap();
    assert_eq!(prediction.security_id, "AAPL");
    assert_eq!(prediction.predicted_price_change, -0.05);
    assert_eq!(prediction.confidence, 0.85);
}
```

#### Testing Alert Processing

```rust
#[tokio::test]
async fn test_process_alert() {
    // ... setup ...
    
    // Set up mocks for all AWS services
    dynamodb_mock.expect_put_item()
        .returning(|_| Ok(PutItemOutput::builder().build()));
    
    sns_mock.expect_publish()
        .returning(|_| Ok(PublishOutput::builder().message_id("test-id").build()));
    
    // ... more mock setups ...
    
    // Create a test alert
    let alert = TlhOpportunityAlert {
        // ... alert properties ...
    };
    
    // Process the alert
    let result = alert_service.process_alert(&alert).await;
    
    // Verify the result
    assert!(result.is_ok());
}
```

## Best Practices

1. **Test One Thing at a Time**: Each test should focus on testing a single functionality.
2. **Use Descriptive Test Names**: Test names should clearly indicate what is being tested.
3. **Set Up Minimal Mock Expectations**: Only set up the mock expectations that are necessary for the test.
4. **Verify All Relevant Outputs**: Make assertions about all relevant outputs of the method being tested.
5. **Test Edge Cases**: Test edge cases such as empty accounts, high volatility, etc.
6. **Test Error Handling**: Test how the system handles errors from AWS services.

## Example Tests

See `src/cloud/aws_tlh_alerts_test.rs` for example tests of the AWS TLH Alert System. 