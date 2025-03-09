use std::collections::HashMap;
use std::time::SystemTime;
use tracing::{debug, instrument};
#[cfg(not(test))]
use aws_sdk_bedrock::Client as BedrockClient;
#[cfg(not(test))]
use aws_sdk_bedrock::config::Region;
#[cfg(not(test))]
use aws_sdk_eventbridge::Client as EventBridgeClient;
#[cfg(not(test))]
use aws_sdk_sagemaker::Client as SageMakerClient;
#[cfg(not(test))]
use aws_sdk_sns::Client as SnsClient;
#[cfg(not(test))]
use aws_sdk_lambda::Client as LambdaClient;
#[cfg(not(test))]
use aws_sdk_dynamodb::Client as DynamoDbClient;
#[cfg(not(test))]
use aws_sdk_cloudwatch::Client as CloudWatchClient;
use serde::{Deserialize, Serialize};
use crate::algorithmic_tlh::{AlgorithmicTLHConfig, AlgorithmicTLHService, TLHPerformanceReport};
use crate::model_portfolio::UnifiedManagedAccount;

// Add these imports for the trait-based approach
#[cfg(test)]
use crate::cloud::aws::mock::{
    BedrockClientTrait, SageMakerClientTrait, SnsClientTrait, LambdaClientTrait,
    DynamoDbClientTrait, CloudWatchClientTrait, EventBridgeClientTrait
};

/// Configuration for AWS TLH Alert System
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AwsTlhAlertConfig {
    /// AWS Region
    pub region: String,
    /// Whether to use Amazon Bedrock for AI predictions
    pub use_bedrock: bool,
    /// Bedrock model ID to use (e.g., "anthropic.claude-3-sonnet-20240229-v1:0")
    pub bedrock_model_id: Option<String>,
    /// Whether to use SageMaker for custom ML models
    pub use_sagemaker: bool,
    /// SageMaker endpoint name
    pub sagemaker_endpoint: Option<String>,
    /// SNS topic ARN for alerts
    pub sns_topic_arn: Option<String>,
    /// Lambda function ARN for custom processing
    pub lambda_function_arn: Option<String>,
    /// DynamoDB table for storing alert history
    pub dynamodb_table: Option<String>,
    /// CloudWatch namespace for metrics
    pub cloudwatch_namespace: Option<String>,
    /// EventBridge event bus name
    pub eventbridge_bus_name: Option<String>,
    /// Alert frequency in seconds
    pub alert_frequency_seconds: u64,
    /// Confidence threshold for alerts (0.0 to 1.0)
    pub confidence_threshold: f64,
    /// Whether to enable real-time market data integration
    pub real_time_market_data: bool,
    /// Whether to enable backtesting mode
    pub backtesting_mode: bool,
}

impl Default for AwsTlhAlertConfig {
    fn default() -> Self {
        Self {
            region: "us-east-1".to_string(),
            use_bedrock: false,
            bedrock_model_id: None,
            use_sagemaker: false,
            sagemaker_endpoint: None,
            sns_topic_arn: None,
            lambda_function_arn: None,
            dynamodb_table: None,
            cloudwatch_namespace: None,
            eventbridge_bus_name: None,
            alert_frequency_seconds: 3600,
            confidence_threshold: 0.7,
            real_time_market_data: false,
            backtesting_mode: false,
        }
    }
}

/// Market prediction from AI/ML models
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketPrediction {
    /// Security identifier
    pub security_id: String,
    /// Predicted price change (percentage)
    pub predicted_price_change: f64,
    /// Prediction confidence (0.0 to 1.0)
    pub confidence: f64,
    /// Prediction horizon (in hours)
    pub horizon_hours: u32,
    /// Timestamp of the prediction
    pub timestamp: SystemTime,
    /// Model used for prediction
    pub model: String,
    /// Features used in the prediction
    pub features: HashMap<String, f64>,
}

/// Tax-loss harvesting opportunity alert
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TlhOpportunityAlert {
    /// Alert ID
    pub id: String,
    /// Account ID
    pub account_id: String,
    /// Security identifier
    pub security_id: String,
    /// Current price
    pub current_price: f64,
    /// Cost basis
    pub cost_basis: f64,
    /// Unrealized loss amount
    pub unrealized_loss: f64,
    /// Unrealized loss percentage
    pub unrealized_loss_percentage: f64,
    /// Estimated tax savings
    pub estimated_tax_savings: f64,
    /// Alert priority (1-5, with 1 being highest)
    pub priority: u8,
    /// Recommended action
    pub recommended_action: String,
    /// Recommended replacement securities
    pub replacement_securities: Vec<String>,
    /// Market prediction
    pub market_prediction: Option<MarketPrediction>,
    /// Timestamp of the alert
    pub timestamp: SystemTime,
    /// Expiration time of the alert
    pub expiration: SystemTime,
}

/// AWS TLH Alert Service
#[allow(dead_code)]
pub struct AwsTlhAlertService {
    /// Configuration
    config: AwsTlhAlertConfig,
    /// Bedrock client for AI predictions
    #[cfg(not(test))]
    bedrock_client: Option<BedrockClient>,
    #[cfg(test)]
    bedrock_client: Option<Box<dyn BedrockClientTrait>>,
    /// SageMaker client for custom ML models
    #[cfg(not(test))]
    sagemaker_client: Option<SageMakerClient>,
    #[cfg(test)]
    sagemaker_client: Option<Box<dyn SageMakerClientTrait>>,
    /// SNS client for notifications
    #[cfg(not(test))]
    sns_client: Option<SnsClient>,
    #[cfg(test)]
    sns_client: Option<Box<dyn SnsClientTrait>>,
    /// Lambda client for custom processing
    #[cfg(not(test))]
    lambda_client: Option<LambdaClient>,
    #[cfg(test)]
    lambda_client: Option<Box<dyn LambdaClientTrait>>,
    /// DynamoDB client for storing alert history
    #[cfg(not(test))]
    dynamodb_client: Option<DynamoDbClient>,
    #[cfg(test)]
    dynamodb_client: Option<Box<dyn DynamoDbClientTrait>>,
    /// CloudWatch client for metrics
    #[cfg(not(test))]
    cloudwatch_client: Option<CloudWatchClient>,
    #[cfg(test)]
    cloudwatch_client: Option<Box<dyn CloudWatchClientTrait>>,
    /// EventBridge client for events
    #[cfg(not(test))]
    eventbridge_client: Option<EventBridgeClient>,
    #[cfg(test)]
    eventbridge_client: Option<Box<dyn EventBridgeClientTrait>>,
    /// Algorithmic TLH service
    tlh_service: AlgorithmicTLHService,
    /// Alert history
    alert_history: HashMap<String, Vec<TlhOpportunityAlert>>,
    /// Market predictions
    market_predictions: HashMap<String, MarketPrediction>,
    /// Last scan time
    last_scan_time: SystemTime,
}

impl AwsTlhAlertService {
    /// Create a new AWS TLH Alert Service
    #[instrument(skip(config, tlh_config), level = "debug")]
    pub async fn new(config: AwsTlhAlertConfig, tlh_config: AlgorithmicTLHConfig) -> Self {
        #[cfg(not(test))]
        let region = Region::new(config.region.clone());
        
        // Create a shared SDK config
        #[cfg(not(test))]
        let sdk_config = aws_config::defaults(aws_config::BehaviorVersion::latest())
            .region(region)
            .load()
            .await;
        
        // Initialize clients with the shared config
        #[cfg(not(test))]
        let bedrock_client = if config.use_bedrock {
            Some(BedrockClient::new(&sdk_config))
        } else {
            None
        };
        
        #[cfg(not(test))]
        let sagemaker_client = if config.use_sagemaker {
            Some(SageMakerClient::new(&sdk_config))
        } else {
            None
        };
        
        #[cfg(not(test))]
        let sns_client = if config.sns_topic_arn.is_some() {
            Some(SnsClient::new(&sdk_config))
        } else {
            None
        };
        
        #[cfg(not(test))]
        let lambda_client = if config.lambda_function_arn.is_some() {
            Some(LambdaClient::new(&sdk_config))
        } else {
            None
        };
        
        #[cfg(not(test))]
        let dynamodb_client = if config.dynamodb_table.is_some() {
            Some(DynamoDbClient::new(&sdk_config))
        } else {
            None
        };
        
        #[cfg(not(test))]
        let cloudwatch_client = if config.cloudwatch_namespace.is_some() {
            Some(CloudWatchClient::new(&sdk_config))
        } else {
            None
        };
        
        #[cfg(not(test))]
        let eventbridge_client = if config.eventbridge_bus_name.is_some() {
            Some(EventBridgeClient::new(&sdk_config))
        } else {
            None
        };
        
        // For test environment, we'll set these to None and let tests inject mocks
        #[cfg(test)]
        let bedrock_client: Option<Box<dyn BedrockClientTrait>> = None;
        
        #[cfg(test)]
        let sagemaker_client: Option<Box<dyn SageMakerClientTrait>> = None;
        
        #[cfg(test)]
        let sns_client: Option<Box<dyn SnsClientTrait>> = None;
        
        #[cfg(test)]
        let lambda_client: Option<Box<dyn LambdaClientTrait>> = None;
        
        #[cfg(test)]
        let dynamodb_client: Option<Box<dyn DynamoDbClientTrait>> = None;
        
        #[cfg(test)]
        let cloudwatch_client: Option<Box<dyn CloudWatchClientTrait>> = None;
        
        #[cfg(test)]
        let eventbridge_client: Option<Box<dyn EventBridgeClientTrait>> = None;
        
        Self {
            config,
            bedrock_client,
            sagemaker_client,
            sns_client,
            lambda_client,
            dynamodb_client,
            cloudwatch_client,
            eventbridge_client,
            tlh_service: AlgorithmicTLHService::new(tlh_config),
            alert_history: HashMap::new(),
            market_predictions: HashMap::new(),
            last_scan_time: SystemTime::now(),
        }
    }
    
    /// Get alert history for an account
    #[instrument(skip(self), fields(account_id = %account_id))]
    pub fn get_alert_history(&self, account_id: &str) -> Vec<TlhOpportunityAlert> {
        debug!(account_id = %account_id, "Getting alert history");
        
        let alerts = self.alert_history.get(account_id).cloned().unwrap_or_default();
        
        debug!(
            account_id = %account_id,
            alert_count = alerts.len(),
            "Retrieved alert history"
        );
        
        alerts
    }
    
    /// Get performance metrics
    #[instrument(skip(self), fields(account_id = %account_id))]
    pub fn get_performance_metrics(&self, account_id: &str) -> TLHPerformanceReport {
        debug!(account_id = %account_id, "Getting performance metrics");
        
        // Get the TLH performance report for the account
        let report = self.tlh_service.analyze_tlh_performance(&UnifiedManagedAccount {
            id: account_id.to_string(),
            name: "".to_string(),
            owner: "".to_string(),
            sleeves: Vec::new(),
            cash_balance: 0.0,
            total_market_value: 0.0,
            created_at: "".to_string(),
            updated_at: "".to_string(),
            tax_settings: None,
            esg_criteria: None,
        });
        
        debug!(
            account_id = %account_id,
            tax_savings = report.total_tax_savings,
            harvested_losses = report.total_harvested_losses,
            harvest_count = report.harvest_count,
            "Retrieved performance metrics"
        );
        
        // Return the report
        report
    }
} 