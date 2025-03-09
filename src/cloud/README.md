# Cloud Integration Module

This module provides cloud integration capabilities for the investment platform, allowing for scalable, cloud-native deployment of various investment platform features.

## AWS Tax-Loss Harvesting Alert System

The AWS Tax-Loss Harvesting (TLH) Alert System is a cloud-based solution that leverages AWS services to provide real-time monitoring and alerting for tax-loss harvesting opportunities. It combines the power of the platform's algorithmic TLH capabilities with AWS's scalable infrastructure and AI services.

### Key Features

- **Real-time Monitoring**: Continuously scans portfolios for tax-loss harvesting opportunities based on configurable criteria.
- **AI-Driven Predictions**: Uses Amazon Bedrock or SageMaker to predict market movements and optimize harvesting timing.
- **Multi-Channel Alerts**: Delivers alerts through SNS, Lambda, EventBridge, and more.
- **Comprehensive Metrics**: Records detailed metrics in CloudWatch for monitoring and analysis.
- **Historical Tracking**: Stores alert history in DynamoDB for audit and analysis.
- **Customizable Configuration**: Highly configurable to meet specific requirements and risk profiles.

### Architecture

The system is built on several AWS services:

- **Amazon Bedrock**: For AI-driven market predictions using large language models.
- **Amazon SageMaker**: For custom machine learning models (alternative to Bedrock).
- **Amazon SNS**: For sending notifications to users and systems.
- **AWS Lambda**: For custom processing of alerts and integration with other systems.
- **Amazon DynamoDB**: For storing alert history and configuration.
- **Amazon CloudWatch**: For monitoring and metrics.
- **Amazon EventBridge**: For event-driven architecture and integration.

### Getting Started

To use the AWS TLH Alert System, you need to:

1. Configure the AWS services in your AWS account.
2. Create an `AwsTlhAlertConfig` with your AWS configuration.
3. Initialize the `AwsTlhAlertService` with your configuration.
4. Start monitoring for opportunities.

See the example in `examples/aws_tlh_alerts_example.rs` for a complete implementation.

### Example Usage

```rust
// Configure the AWS TLH Alert System
let aws_config = AwsTlhAlertConfig {
    region: "us-east-1".to_string(),
    use_bedrock: true,
    bedrock_model_id: Some("anthropic.claude-3-sonnet-20240229-v1:0".to_string()),
    sns_topic_arn: Some("arn:aws:sns:us-east-1:123456789012:TaxLossHarvestingAlerts".to_string()),
    dynamodb_table: Some("TaxLossHarvestingAlerts".to_string()),
    cloudwatch_namespace: Some("TaxLossHarvesting".to_string()),
    alert_frequency_seconds: 300, // Check every 5 minutes
    confidence_threshold: 0.75,
    real_time_market_data: true,
    ..AwsTlhAlertConfig::default()
};

// Configure the Algorithmic TLH Service
let tlh_config = AlgorithmicTLHConfig::default();

// Create the AWS TLH Alert Service
let mut alert_service = AwsTlhAlertService::new(aws_config, tlh_config).await;

// Start monitoring for opportunities
alert_service.start_monitoring(&account).await?;
```

### Alert Processing

When an opportunity is detected, the system:

1. Creates an alert with details about the opportunity.
2. Stores the alert in DynamoDB for historical tracking.
3. Sends a notification via SNS to subscribers.
4. Invokes a Lambda function for custom processing.
5. Sends an event to EventBridge for integration with other systems.
6. Records metrics in CloudWatch for monitoring and analysis.

### Customization

The system is highly customizable through the `AwsTlhAlertConfig` struct, allowing you to:

- Choose between Bedrock and SageMaker for AI predictions.
- Configure alert frequency and confidence thresholds.
- Enable or disable specific AWS services.
- Set up custom processing through Lambda functions.
- Define custom metrics and monitoring in CloudWatch.

### Future Enhancements

Planned enhancements for the AWS TLH Alert System include:

- Integration with AWS Step Functions for complex workflows.
- Support for AWS Comprehend for sentiment analysis of market news.
- Integration with AWS Forecast for time-series forecasting.
- Support for AWS AppSync for real-time GraphQL APIs.
- Enhanced security with AWS IAM and AWS KMS.

## Testing Framework

The AWS TLH Alert System includes a comprehensive testing framework that allows for thorough testing without requiring actual AWS credentials or making real API calls.

### Key Features

- **Trait-Based Design**: Each AWS service client has a corresponding trait that defines its interface.
- **Mock Implementations**: Mock implementations of these traits using the `mockall` crate.
- **Test-Specific Service Implementation**: The `AwsTlhAlertService` has test-specific implementations that accept mock clients.
- **Test Utilities**: Helper functions for creating test data and setting up mock expectations.

### Example Test

```rust
#[tokio::test]
async fn test_get_bedrock_prediction() {
    // Configure the AWS TLH Alert System
    let aws_config = AwsTlhAlertConfig {
        // ... configuration ...
    };
    
    // Configure the Algorithmic TLH Service
    let tlh_config = AlgorithmicTLHConfig::default();
    
    // Create mock Bedrock client
    let mut bedrock_mock = MockBedrockClientMock::new();
    
    // Set up expectations for the Bedrock client
    bedrock_mock.expect_invoke_model()
        .returning(|_| {
            // Create a mock response
            // ...
        });
    
    // Create the AWS TLH Alert Service with mock clients
    let alert_service = AwsTlhAlertService::with_mock_clients(
        aws_config,
        tlh_config,
        Some(Box::new(bedrock_mock)),
        // ... other mock clients ...
    );
    
    // Get prediction
    let prediction_result = alert_service.get_bedrock_prediction("AAPL").await;
    
    // Verify the prediction
    assert!(prediction_result.is_ok());
    // ... more assertions ...
}
```

For more information on testing, see [TESTING.md](TESTING.md). 