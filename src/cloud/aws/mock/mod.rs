use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::SystemTime;

use aws_sdk_bedrock::Client as BedrockClient;
use aws_sdk_bedrock::error::SdkError;
use aws_sdk_bedrockruntime::operation::invoke_model::InvokeModelError;
use aws_sdk_bedrockruntime::operation::invoke_model::InvokeModelOutput;
use aws_sdk_bedrockruntime::primitives::Blob;
use aws_sdk_sagemaker::Client as SageMakerClient;
use aws_sdk_sagemaker::error::SdkError as SageMakerSdkError;
use aws_sdk_sagemakerruntime::operation::invoke_endpoint::InvokeEndpointError;
use aws_sdk_sagemakerruntime::operation::invoke_endpoint::InvokeEndpointOutput;
use aws_sdk_sns::Client as SnsClient;
use aws_sdk_sns::error::SdkError as SnsSdkError;
use aws_sdk_sns::operation::publish::PublishError;
use aws_sdk_sns::operation::publish::PublishOutput;
use aws_sdk_lambda::Client as LambdaClient;
use aws_sdk_lambda::error::SdkError as LambdaSdkError;
use aws_sdk_lambda::operation::invoke::InvokeError;
use aws_sdk_lambda::operation::invoke::InvokeOutput;
use aws_sdk_dynamodb::Client as DynamoDbClient;
use aws_sdk_dynamodb::error::SdkError as DynamoDbSdkError;
use aws_sdk_dynamodb::operation::put_item::PutItemError;
use aws_sdk_dynamodb::operation::put_item::PutItemOutput;
use aws_sdk_cloudwatch::Client as CloudWatchClient;
use aws_sdk_cloudwatch::error::SdkError as CloudWatchSdkError;
use aws_sdk_cloudwatch::operation::put_metric_data::PutMetricDataError;
use aws_sdk_cloudwatch::operation::put_metric_data::PutMetricDataOutput;
use aws_sdk_eventbridge::Client as EventBridgeClient;
use aws_sdk_eventbridge::error::SdkError as EventBridgeSdkError;
use aws_sdk_eventbridge::operation::put_events::PutEventsError;
use aws_sdk_eventbridge::operation::put_events::PutEventsOutput;

use mockall::mock;
use mockall::predicate::*;
use serde_json::json;

use crate::cloud::aws::tlh_alerts::{MarketPrediction, TlhOpportunityAlert};

// Define request types for AWS SDK v2
type InvokeModelRequest = aws_sdk_bedrockruntime::operation::invoke_model::builders::InvokeModelFluentBuilder;
type InvokeEndpointRequest = aws_sdk_sagemakerruntime::operation::invoke_endpoint::builders::InvokeEndpointFluentBuilder;
type PublishRequest = aws_sdk_sns::operation::publish::builders::PublishFluentBuilder;
type InvocationRequest = aws_sdk_lambda::operation::invoke::builders::InvokeFluentBuilder;
type PutItemRequest = aws_sdk_dynamodb::operation::put_item::builders::PutItemFluentBuilder;
type PutMetricDataRequest = aws_sdk_cloudwatch::operation::put_metric_data::builders::PutMetricDataFluentBuilder;
type PutEventsRequest = aws_sdk_eventbridge::operation::put_events::builders::PutEventsFluentBuilder;

// Mock for BedrockClient
mock! {
    pub BedrockClientMock {}

    #[async_trait::async_trait]
    impl BedrockClientTrait for BedrockClientMock {
        async fn invoke_model(&self, request: InvokeModelRequest) 
            -> std::result::Result<InvokeModelOutput, SdkError<InvokeModelError>>;
    }
}

// Mock for SageMakerClient
mock! {
    pub SageMakerClientMock {}

    #[async_trait::async_trait]
    impl SageMakerClientTrait for SageMakerClientMock {
        async fn invoke_endpoint(&self, request: InvokeEndpointRequest) 
            -> std::result::Result<InvokeEndpointOutput, SageMakerSdkError<InvokeEndpointError>>;
    }
}

// Mock for SnsClient
mock! {
    pub SnsClientMock {}

    #[async_trait::async_trait]
    impl SnsClientTrait for SnsClientMock {
        async fn publish(&self, request: PublishRequest) 
            -> std::result::Result<PublishOutput, SnsSdkError<PublishError>>;
    }
}

// Mock for LambdaClient
mock! {
    pub LambdaClientMock {}

    #[async_trait::async_trait]
    impl LambdaClientTrait for LambdaClientMock {
        async fn invoke(&self, request: InvocationRequest) 
            -> std::result::Result<InvokeOutput, LambdaSdkError<InvokeError>>;
    }
}

// Mock for DynamoDbClient
mock! {
    pub DynamoDbClientMock {}

    #[async_trait::async_trait]
    impl DynamoDbClientTrait for DynamoDbClientMock {
        async fn put_item(&self, request: PutItemRequest) 
            -> std::result::Result<PutItemOutput, DynamoDbSdkError<PutItemError>>;
    }
}

// Mock for CloudWatchClient
mock! {
    pub CloudWatchClientMock {}

    #[async_trait::async_trait]
    impl CloudWatchClientTrait for CloudWatchClientMock {
        async fn put_metric_data(&self, request: PutMetricDataRequest) 
            -> std::result::Result<PutMetricDataOutput, CloudWatchSdkError<PutMetricDataError>>;
    }
}

// Mock for EventBridgeClient
mock! {
    pub EventBridgeClientMock {}

    #[async_trait::async_trait]
    impl EventBridgeClientTrait for EventBridgeClientMock {
        async fn put_events(&self, request: PutEventsRequest) 
            -> std::result::Result<PutEventsOutput, EventBridgeSdkError<PutEventsError>>;
    }
}

// Trait definitions for AWS clients
#[async_trait::async_trait]
pub trait BedrockClientTrait {
    async fn invoke_model(&self, request: InvokeModelRequest) 
        -> std::result::Result<InvokeModelOutput, SdkError<InvokeModelError>>;
}

#[async_trait::async_trait]
pub trait SageMakerClientTrait {
    async fn invoke_endpoint(&self, request: InvokeEndpointRequest) 
        -> std::result::Result<InvokeEndpointOutput, SageMakerSdkError<InvokeEndpointError>>;
}

#[async_trait::async_trait]
pub trait SnsClientTrait {
    async fn publish(&self, request: PublishRequest) 
        -> std::result::Result<PublishOutput, SnsSdkError<PublishError>>;
}

#[async_trait::async_trait]
pub trait LambdaClientTrait {
    async fn invoke(&self, request: InvocationRequest) 
        -> std::result::Result<InvokeOutput, LambdaSdkError<InvokeError>>;
}

#[async_trait::async_trait]
pub trait DynamoDbClientTrait {
    async fn put_item(&self, request: PutItemRequest) 
        -> std::result::Result<PutItemOutput, DynamoDbSdkError<PutItemError>>;
}

#[async_trait::async_trait]
pub trait CloudWatchClientTrait {
    async fn put_metric_data(&self, request: PutMetricDataRequest) 
        -> std::result::Result<PutMetricDataOutput, CloudWatchSdkError<PutMetricDataError>>;
}

#[async_trait::async_trait]
pub trait EventBridgeClientTrait {
    async fn put_events(&self, request: PutEventsRequest) 
        -> std::result::Result<PutEventsOutput, EventBridgeSdkError<PutEventsError>>;
}

// Implement the traits for the real AWS clients
#[async_trait::async_trait]
impl BedrockClientTrait for BedrockClient {
    async fn invoke_model(&self, request: InvokeModelRequest) 
        -> std::result::Result<InvokeModelOutput, SdkError<InvokeModelError>> {
        request.send().await
    }
}

#[async_trait::async_trait]
impl SageMakerClientTrait for SageMakerClient {
    async fn invoke_endpoint(&self, request: InvokeEndpointRequest) 
        -> std::result::Result<InvokeEndpointOutput, SageMakerSdkError<InvokeEndpointError>> {
        request.send().await
    }
}

#[async_trait::async_trait]
impl SnsClientTrait for SnsClient {
    async fn publish(&self, request: PublishRequest) 
        -> std::result::Result<PublishOutput, SnsSdkError<PublishError>> {
        request.send().await
    }
}

#[async_trait::async_trait]
impl LambdaClientTrait for LambdaClient {
    async fn invoke(&self, request: InvocationRequest) 
        -> std::result::Result<InvokeOutput, LambdaSdkError<InvokeError>> {
        request.send().await
    }
}

#[async_trait::async_trait]
impl DynamoDbClientTrait for DynamoDbClient {
    async fn put_item(&self, request: PutItemRequest) 
        -> std::result::Result<PutItemOutput, DynamoDbSdkError<PutItemError>> {
        request.send().await
    }
}

#[async_trait::async_trait]
impl CloudWatchClientTrait for CloudWatchClient {
    async fn put_metric_data(&self, request: PutMetricDataRequest) 
        -> std::result::Result<PutMetricDataOutput, CloudWatchSdkError<PutMetricDataError>> {
        request.send().await
    }
}

#[async_trait::async_trait]
impl EventBridgeClientTrait for EventBridgeClient {
    async fn put_events(&self, request: PutEventsRequest) 
        -> std::result::Result<PutEventsOutput, EventBridgeSdkError<PutEventsError>> {
        request.send().await
    }
}

// Mock AWS Service for testing
pub struct MockAwsService {
    pub bedrock_client: Option<MockBedrockClientMock>,
    pub sagemaker_client: Option<MockSageMakerClientMock>,
    pub sns_client: Option<MockSnsClientMock>,
    pub lambda_client: Option<MockLambdaClientMock>,
    pub dynamodb_client: Option<MockDynamoDbClientMock>,
    pub cloudwatch_client: Option<MockCloudWatchClientMock>,
    pub eventbridge_client: Option<MockEventBridgeClientMock>,
    pub market_predictions: Arc<Mutex<HashMap<String, MarketPrediction>>>,
    pub alerts: Arc<Mutex<Vec<TlhOpportunityAlert>>>,
}

impl Default for MockAwsService {
    fn default() -> Self {
        Self {
            bedrock_client: Some(MockBedrockClientMock::new()),
            sagemaker_client: Some(MockSageMakerClientMock::new()),
            sns_client: Some(MockSnsClientMock::new()),
            lambda_client: Some(MockLambdaClientMock::new()),
            dynamodb_client: Some(MockDynamoDbClientMock::new()),
            cloudwatch_client: Some(MockCloudWatchClientMock::new()),
            eventbridge_client: Some(MockEventBridgeClientMock::new()),
            market_predictions: Arc::new(Mutex::new(HashMap::new())),
            alerts: Arc::new(Mutex::new(Vec::new())),
        }
    }
}

impl MockAwsService {
    pub fn new() -> Self {
        Self::default()
    }

    // Setup mock for Bedrock client to return a specific prediction
    pub fn setup_bedrock_prediction(&mut self, security_id: &str, prediction: MarketPrediction) {
        if let Some(ref mut client) = self.bedrock_client {
            let predictions = Arc::clone(&self.market_predictions);
            let security_id = security_id.to_string();
            
            client.expect_invoke_model()
                .returning(move |_| {
                    let mut predictions = predictions.lock().unwrap();
                    predictions.insert(security_id.clone(), prediction.clone());
                    
                    // Create a mock response from Bedrock
                    let response_json = json!({
                        "content": [{
                            "text": format!(
                                "Based on my analysis, here's the prediction for {}: \
                                 {{\
                                 \"predictedPriceChange\": {}, \
                                 \"confidence\": {}, \
                                 \"rationale\": \"Market conditions suggest this movement.\"\
                                 }}", 
                                security_id, 
                                prediction.predicted_price_change,
                                prediction.confidence
                            )
                        }]
                    });
                    
                    let response_bytes = serde_json::to_vec(&response_json).unwrap();
                    
                    let output = InvokeModelOutput::builder()
                        .body(Blob::new(response_bytes))
                        .build()
                        .expect("Failed to build InvokeModelOutput");
                    
                    Ok(output)
                });
        }
    }
    
    // Setup mock for SageMaker client to return a specific prediction
    pub fn setup_sagemaker_prediction(&mut self, security_id: &str, prediction: MarketPrediction) {
        if let Some(ref mut client) = self.sagemaker_client {
            let predictions = Arc::clone(&self.market_predictions);
            let security_id = security_id.to_string();
            
            client.expect_invoke_endpoint()
                .returning(move |_| {
                    let mut predictions = predictions.lock().unwrap();
                    predictions.insert(security_id.clone(), prediction.clone());
                    
                    // Create a mock response from SageMaker
                    let response_json = json!({
                        "predicted_price_change": prediction.predicted_price_change,
                        "confidence": prediction.confidence,
                        "horizon_hours": prediction.horizon_hours
                    });
                    
                    let response_bytes = serde_json::to_vec(&response_json).unwrap();
                    
                    Ok(InvokeEndpointOutput::builder()
                        .body(Blob::new(response_bytes))
                        .build())
                });
        }
    }
    
    // Setup mock for DynamoDB client to store alerts
    pub fn setup_dynamodb_store_alert(&mut self) {
        if let Some(ref mut client) = self.dynamodb_client {
            let alerts = Arc::clone(&self.alerts);
            
            client.expect_put_item()
                .returning(move |_request| {
                    // Create a simple alert record for testing
                    let alert = TlhOpportunityAlert {
                        id: "test-alert-id".to_string(),
                        account_id: "test-account".to_string(),
                        security_id: "test-security".to_string(),
                        current_price: 100.0,
                        cost_basis: 120.0,
                        unrealized_loss: -20.0,
                        unrealized_loss_percentage: -0.167,
                        estimated_tax_savings: 7.0,
                        priority: 2,
                        recommended_action: "Sell and replace".to_string(),
                        replacement_securities: vec!["REPLACEMENT".to_string()],
                        market_prediction: None,
                        timestamp: SystemTime::now(),
                        expiration: SystemTime::now(),
                    };
                    
                    let mut alerts = alerts.lock().unwrap();
                    alerts.push(alert);
                    
                    Ok(PutItemOutput::builder().build())
                });
        }
    }
    
    // Setup mock for SNS client
    pub fn setup_sns_publish(&mut self) {
        if let Some(ref mut client) = self.sns_client {
            client.expect_publish()
                .returning(|_| {
                    Ok(PublishOutput::builder()
                        .message_id("test-message-id")
                        .build())
                });
        }
    }
    
    // Setup mock for Lambda client
    pub fn setup_lambda_invoke(&mut self) {
        if let Some(ref mut client) = self.lambda_client {
            client.expect_invoke()
                .returning(|_| {
                    Ok(InvokeOutput::builder()
                        .status_code(200)
                        .build())
                });
        }
    }
    
    // Setup mock for CloudWatch client
    pub fn setup_cloudwatch_metrics(&mut self) {
        if let Some(ref mut client) = self.cloudwatch_client {
            client.expect_put_metric_data()
                .returning(|_| {
                    Ok(PutMetricDataOutput::builder().build())
                });
        }
    }
    
    // Setup mock for EventBridge client
    pub fn setup_eventbridge_events(&mut self) {
        if let Some(ref mut client) = self.eventbridge_client {
            client.expect_put_events()
                .returning(|_| {
                    Ok(PutEventsOutput::builder().build())
                });
        }
    }
    
    // Get stored market predictions
    pub fn get_market_predictions(&self) -> HashMap<String, MarketPrediction> {
        self.market_predictions.lock().unwrap().clone()
    }
    
    // Get stored alerts
    pub fn get_alerts(&self) -> Vec<TlhOpportunityAlert> {
        self.alerts.lock().unwrap().clone()
    }
} 