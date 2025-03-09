use std::collections::{HashMap, HashSet};
use std::time::{Duration, SystemTime};
use tracing::{debug, error, info, instrument, span, trace, warn, Level};
use aws_sdk_bedrock::Client as BedrockClient;
use aws_sdk_bedrock::config::Region;
use aws_sdk_bedrock::types::InvokeModelRequest;
use aws_sdk_eventbridge::Client as EventBridgeClient;
use aws_sdk_eventbridge::types::{PutEventsRequest, PutEventsRequestEntry};
use aws_sdk_sagemaker::Client as SageMakerClient;
use aws_sdk_sagemaker::types::{InvokeEndpointRequest, InvokeEndpointResponse};
use aws_sdk_sns::Client as SnsClient;
use aws_sdk_sns::types::PublishRequest;
use aws_sdk_lambda::Client as LambdaClient;
use aws_sdk_lambda::types::InvocationRequest;
use aws_sdk_dynamodb::Client as DynamoDbClient;
use aws_sdk_dynamodb::types::{AttributeValue, PutItemRequest};
use aws_sdk_cloudwatch::Client as CloudWatchClient;
use aws_sdk_cloudwatch::types::{Dimension, MetricDatum, PutMetricDataRequest, StandardUnit};
use serde::{Deserialize, Serialize};
use tokio::time;
use crate::algorithmic_tlh::{AlgorithmicTLHConfig, AlgorithmicTLHService, MarketData, TLHPerformanceReport};
use crate::model_portfolio::{UnifiedManagedAccount, TaxOptimizationSettings};
use crate::portfolio_rebalancing::RebalanceTrade;
use crate::error::{ApiError, ApiResult};

// Add these imports for the trait-based approach
#[cfg(test)]
use crate::cloud::mock_aws::{
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
            use_bedrock: true,
            bedrock_model_id: Some("anthropic.claude-3-sonnet-20240229-v1:0".to_string()),
            use_sagemaker: false,
            sagemaker_endpoint: None,
            sns_topic_arn: None,
            lambda_function_arn: None,
            dynamodb_table: Some("TaxLossHarvestingAlerts".to_string()),
            cloudwatch_namespace: Some("TaxLossHarvesting".to_string()),
            eventbridge_bus_name: Some("default".to_string()),
            alert_frequency_seconds: 300, // 5 minutes
            confidence_threshold: 0.75,
            real_time_market_data: true,
            backtesting_mode: false,
        }
    }
}

/// Market prediction from AI model
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
    #[cfg(not(test))]
    pub async fn new(config: AwsTlhAlertConfig, tlh_config: AlgorithmicTLHConfig) -> Self {
        let region = Region::new(config.region.clone());
        
        // Initialize AWS clients
        let bedrock_client = if config.use_bedrock {
            Some(BedrockClient::new(region.clone()))
        } else {
            None
        };
        
        let sagemaker_client = if config.use_sagemaker {
            Some(SageMakerClient::new(region.clone()))
        } else {
            None
        };
        
        let sns_client = if config.sns_topic_arn.is_some() {
            Some(SnsClient::new(region.clone()))
        } else {
            None
        };
        
        let lambda_client = if config.lambda_function_arn.is_some() {
            Some(LambdaClient::new(region.clone()))
        } else {
            None
        };
        
        let dynamodb_client = if config.dynamodb_table.is_some() {
            Some(DynamoDbClient::new(region.clone()))
        } else {
            None
        };
        
        let cloudwatch_client = if config.cloudwatch_namespace.is_some() {
            Some(CloudWatchClient::new(region.clone()))
        } else {
            None
        };
        
        let eventbridge_client = if config.eventbridge_bus_name.is_some() {
            Some(EventBridgeClient::new(region.clone()))
        } else {
            None
        };
        
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
    
    /// Create a new AWS TLH Alert Service for testing
    #[cfg(test)]
    pub async fn new(config: AwsTlhAlertConfig, tlh_config: AlgorithmicTLHConfig) -> Self {
        Self {
            config,
            bedrock_client: None,
            sagemaker_client: None,
            sns_client: None,
            lambda_client: None,
            dynamodb_client: None,
            cloudwatch_client: None,
            eventbridge_client: None,
            tlh_service: AlgorithmicTLHService::new(tlh_config),
            alert_history: HashMap::new(),
            market_predictions: HashMap::new(),
            last_scan_time: SystemTime::now(),
        }
    }
    
    /// Create a new AWS TLH Alert Service with mock clients for testing
    #[cfg(test)]
    pub fn with_mock_clients(
        config: AwsTlhAlertConfig,
        tlh_config: AlgorithmicTLHConfig,
        bedrock_client: Option<Box<dyn BedrockClientTrait>>,
        sagemaker_client: Option<Box<dyn SageMakerClientTrait>>,
        sns_client: Option<Box<dyn SnsClientTrait>>,
        lambda_client: Option<Box<dyn LambdaClientTrait>>,
        dynamodb_client: Option<Box<dyn DynamoDbClientTrait>>,
        cloudwatch_client: Option<Box<dyn CloudWatchClientTrait>>,
        eventbridge_client: Option<Box<dyn EventBridgeClientTrait>>,
    ) -> Self {
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
    
    /// Start monitoring for tax-loss harvesting opportunities
    #[instrument(skip(self, account), fields(account_id = %account.id))]
    pub async fn start_monitoring(&mut self, account: &UnifiedManagedAccount) -> ApiResult<()> {
        info!(
            account_id = %account.id,
            region = %self.config.region,
            alert_frequency = self.config.alert_frequency_seconds,
            "Starting AWS TLH Alert Service"
        );
        
        // Record metrics for monitoring start
        if let Some(ref client) = self.cloudwatch_client {
            if let Some(ref namespace) = self.config.cloudwatch_namespace {
                let request = PutMetricDataRequest::builder()
                    .namespace(namespace)
                    .metric_data(
                        MetricDatum::builder()
                            .metric_name("MonitoringStarted")
                            .unit(StandardUnit::Count)
                            .value(1.0)
                            .dimensions(
                                Dimension::builder()
                                    .name("AccountId")
                                    .value(&account.id)
                                    .build(),
                            )
                            .build(),
                    )
                    .build();
                
                match client.put_metric_data(request).await {
                    Ok(_) => debug!("Recorded monitoring start metric"),
                    Err(e) => warn!(error = %e, "Failed to record metric"),
                }
            }
        }
        
        // Main monitoring loop
        let mut interval = time::interval(Duration::from_secs(self.config.alert_frequency_seconds));
        
        loop {
            interval.tick().await;
            
            // Scan for opportunities
            let scan_span = span!(Level::INFO, "scan_for_opportunities", account_id = %account.id);
            let _enter = scan_span.enter();
            
            debug!(
                account_id = %account.id,
                "Scanning for TLH opportunities"
            );
            
            match self.scan_for_opportunities(account).await {
                Ok(alerts) => {
                    if !alerts.is_empty() {
                        info!(
                            account_id = %account.id,
                            alert_count = alerts.len(),
                            "Found TLH opportunities"
                        );
                        
                        // Process each alert
                        for alert in alerts {
                            let process_span = span!(
                                Level::INFO, 
                                "process_alert", 
                                alert_id = %alert.id,
                                security_id = %alert.security_id,
                                priority = alert.priority
                            );
                            let _enter = process_span.enter();
                            
                            debug!(
                                alert_id = %alert.id,
                                security_id = %alert.security_id,
                                unrealized_loss = alert.unrealized_loss,
                                estimated_tax_savings = alert.estimated_tax_savings,
                                "Processing TLH alert"
                            );
                            
                            if let Err(e) = self.process_alert(&alert).await {
                                error!(
                                    error = %e,
                                    alert_id = %alert.id,
                                    security_id = %alert.security_id,
                                    "Failed to process alert"
                                );
                            }
                        }
                    } else {
                        debug!(
                            account_id = %account.id,
                            "No TLH opportunities found"
                        );
                    }
                },
                Err(e) => {
                    error!(
                        error = %e,
                        account_id = %account.id,
                        "Error scanning for opportunities"
                    );
                    
                    // Record error metric
                    if let Some(ref client) = self.cloudwatch_client {
                        if let Some(ref namespace) = self.config.cloudwatch_namespace {
                            let request = PutMetricDataRequest::builder()
                                .namespace(namespace)
                                .metric_data(
                                    MetricDatum::builder()
                                        .metric_name("ScanErrors")
                                        .unit(StandardUnit::Count)
                                        .value(1.0)
                                        .dimensions(
                                            Dimension::builder()
                                                .name("AccountId")
                                                .value(&account.id)
                                                .build(),
                                        )
                                        .build(),
                                )
                                .build();
                            
                            let _ = client.put_metric_data(request).await;
                        }
                    }
                }
            }
            
            self.last_scan_time = SystemTime::now();
        }
    }
    
    /// Scan for tax-loss harvesting opportunities
    #[instrument(skip(self, account), fields(account_id = %account.id))]
    pub async fn scan_for_opportunities(&mut self, account: &UnifiedManagedAccount) -> ApiResult<Vec<TlhOpportunityAlert>> {
        debug!(
            account_id = %account.id,
            "Starting scan for TLH opportunities"
        );
        
        let mut alerts = Vec::new();
        
        // Update market predictions
        if self.config.use_bedrock || self.config.use_sagemaker {
            trace!("Updating market predictions");
            self.update_market_predictions(account).await?;
        }
        
        // Generate TLH trades using the algorithmic service
        let market_volatility = self.calculate_market_volatility().await?;
        debug!(
            volatility = market_volatility,
            "Calculated market volatility"
        );
        
        let tlh_trades = self.tlh_service.generate_real_time_tlh_trades(account, Some(market_volatility))?;
        debug!(
            trade_count = tlh_trades.len(),
            "Generated TLH trades"
        );
        
        // Convert trades to alerts with enhanced AI insights
        for trade in tlh_trades {
            if !trade.is_buy && trade.reason == crate::portfolio_rebalancing::TradeReason::TaxLossHarvesting {
                // This is a sell trade for tax-loss harvesting
                let security_id = trade.security_id.clone();
                let unrealized_loss = trade.tax_impact.unwrap_or(0.0);
                
                // Skip if loss is positive (this shouldn't happen for TLH trades)
                if unrealized_loss >= 0.0 {
                    trace!(
                        security_id = %security_id,
                        unrealized_loss = unrealized_loss,
                        "Skipping trade with non-negative unrealized loss"
                    );
                    continue;
                }
                
                // Get market prediction if available
                let market_prediction = self.market_predictions.get(&security_id).cloned();
                
                // Calculate priority based on tax savings and prediction confidence
                let tax_rate = 0.35; // Assume 35% tax rate
                let estimated_tax_savings = -unrealized_loss * tax_rate;
                let prediction_confidence = market_prediction.as_ref().map_or(0.5, |p| p.confidence);
                
                // Priority: 1 (highest) to 5 (lowest)
                let priority = if estimated_tax_savings > 10000.0 && prediction_confidence > 0.9 {
                    1
                } else if estimated_tax_savings > 5000.0 && prediction_confidence > 0.8 {
                    2
                } else if estimated_tax_savings > 1000.0 && prediction_confidence > 0.7 {
                    3
                } else if estimated_tax_savings > 500.0 {
                    4
                } else {
                    5
                };
                
                // Find replacement securities
                let replacements = self.tlh_service.find_replacement_securities(&security_id);
                let replacement_ids = replacements.iter()
                    .filter(|r| !r.recently_sold && r.correlation > 0.8) // Use hardcoded correlation threshold
                    .map(|r| r.security_id.clone())
                    .collect::<Vec<_>>();
                
                // Create alert
                let alert = TlhOpportunityAlert {
                    id: uuid::Uuid::new_v4().to_string(),
                    account_id: account.id.clone(),
                    security_id: security_id.clone(),
                    current_price: 0.0, // Would be populated from market data
                    cost_basis: 0.0,    // Would be calculated from tax lot data
                    unrealized_loss,
                    unrealized_loss_percentage: 0.0, // Would be calculated
                    estimated_tax_savings,
                    priority,
                    recommended_action: format!("Sell {} and replace with {}", 
                        security_id, 
                        replacement_ids.first().unwrap_or(&"similar security".to_string())),
                    replacement_securities: replacement_ids,
                    market_prediction,
                    timestamp: SystemTime::now(),
                    expiration: SystemTime::now() + Duration::from_secs(24 * 60 * 60), // 24 hours
                };
                
                debug!(
                    alert_id = %alert.id,
                    security_id = %alert.security_id,
                    unrealized_loss = alert.unrealized_loss,
                    estimated_tax_savings = alert.estimated_tax_savings,
                    priority = alert.priority,
                    "Created TLH alert"
                );
                
                // Add to alerts
                alerts.push(alert);
            }
        }
        
        // Sort alerts by priority
        alerts.sort_by_key(|a| a.priority);
        
        // Store in history
        self.alert_history.insert(account.id.clone(), alerts.clone());
        
        info!(
            account_id = %account.id,
            alert_count = alerts.len(),
            "Completed scan for TLH opportunities"
        );
        
        Ok(alerts)
    }
    
    /// Process a tax-loss harvesting opportunity alert
    #[instrument(skip(self), fields(alert_id = %alert.id, security_id = %alert.security_id))]
    async fn process_alert(&self, alert: &TlhOpportunityAlert) -> ApiResult<()> {
        debug!(
            alert_id = %alert.id,
            security_id = %alert.security_id,
            priority = alert.priority,
            "Processing TLH alert"
        );
        
        // 1. Store in DynamoDB
        if let Some(ref client) = self.dynamodb_client {
            if let Some(ref table) = self.config.dynamodb_table {
                debug!("Storing alert in DynamoDB");
                
                let request = PutItemRequest::builder()
                    .table_name(table)
                    .item("AlertId", AttributeValue::S(alert.id.clone()))
                    .item("AccountId", AttributeValue::S(alert.account_id.clone()))
                    .item("SecurityId", AttributeValue::S(alert.security_id.clone()))
                    .item("UnrealizedLoss", AttributeValue::N(alert.unrealized_loss.to_string()))
                    .item("EstimatedTaxSavings", AttributeValue::N(alert.estimated_tax_savings.to_string()))
                    .item("Priority", AttributeValue::N(alert.priority.to_string()))
                    .item("Timestamp", AttributeValue::N(alert.timestamp.duration_since(SystemTime::UNIX_EPOCH).unwrap().as_secs().to_string()))
                    .build();
                
                match client.put_item(request).await {
                    Ok(_) => debug!("Stored alert in DynamoDB"),
                    Err(e) => {
                        warn!(error = %e, "Failed to store alert in DynamoDB");
                        // Continue processing even if DynamoDB storage fails
                    }
                }
            }
        }
        
        // 2. Send SNS notification
        if let Some(ref client) = self.sns_client {
            if let Some(ref topic_arn) = self.config.sns_topic_arn {
                debug!(topic_arn = %topic_arn, "Sending SNS notification");
                
                let message = serde_json::to_string(alert).unwrap_or_else(|_| format!("Tax-loss harvesting opportunity for {}", alert.security_id));
                let subject = format!("TLH Alert: {} (Priority {})", alert.security_id, alert.priority);
                
                let request = PublishRequest::builder()
                    .topic_arn(topic_arn)
                    .message(message)
                    .subject(subject)
                    .build();
                
                match client.publish(request).await {
                    Ok(_) => debug!("Sent SNS notification"),
                    Err(e) => warn!(error = %e, "Failed to send SNS notification")
                }
            }
        }
        
        // 3. Invoke Lambda function
        if let Some(ref client) = self.lambda_client {
            if let Some(ref function_arn) = self.config.lambda_function_arn {
                debug!(function_arn = %function_arn, "Invoking Lambda function");
                
                let payload = serde_json::to_vec(alert).unwrap_or_default();
                
                let request = InvocationRequest::builder()
                    .function_name(function_arn)
                    .payload(payload.into())
                    .build();
                
                match client.invoke(request).await {
                    Ok(_) => debug!("Invoked Lambda function"),
                    Err(e) => warn!(error = %e, "Failed to invoke Lambda function")
                }
            }
        }
        
        // 4. Send event to EventBridge
        if let Some(ref client) = self.eventbridge_client {
            if let Some(ref bus_name) = self.config.eventbridge_bus_name {
                debug!(bus_name = %bus_name, "Sending EventBridge event");
                
                let detail = serde_json::to_string(alert).unwrap_or_default();
                
                let entry = PutEventsRequestEntry::builder()
                    .event_bus_name(bus_name)
                    .source("investment-platform.tlh-alerts")
                    .detail_type("TaxLossHarvestingOpportunity")
                    .detail(detail)
                    .build();
                
                let request = PutEventsRequest::builder()
                    .entries(entry)
                    .build();
                
                match client.put_events(request).await {
                    Ok(_) => debug!("Sent EventBridge event"),
                    Err(e) => warn!(error = %e, "Failed to send EventBridge event")
                }
            }
        }
        
        // 5. Record metrics
        if let Some(ref client) = self.cloudwatch_client {
            if let Some(ref namespace) = self.config.cloudwatch_namespace {
                debug!(namespace = %namespace, "Recording CloudWatch metrics");
                
                let request = PutMetricDataRequest::builder()
                    .namespace(namespace)
                    .metric_data(
                        MetricDatum::builder()
                            .metric_name("TLHOpportunities")
                            .unit(StandardUnit::Count)
                            .value(1.0)
                            .dimensions(
                                Dimension::builder()
                                    .name("SecurityId")
                                    .value(&alert.security_id)
                                    .build(),
                            )
                            .build(),
                    )
                    .metric_data(
                        MetricDatum::builder()
                            .metric_name("EstimatedTaxSavings")
                            .unit(StandardUnit::None)
                            .value(alert.estimated_tax_savings)
                            .dimensions(
                                Dimension::builder()
                                    .name("SecurityId")
                                    .value(&alert.security_id)
                                    .build(),
                            )
                            .build(),
                    )
                    .build();
                
                match client.put_metric_data(request).await {
                    Ok(_) => debug!("Recorded CloudWatch metrics"),
                    Err(e) => warn!(error = %e, "Failed to record CloudWatch metrics")
                }
            }
        }
        
        info!(
            alert_id = %alert.id,
            security_id = %alert.security_id,
            "Successfully processed TLH alert"
        );
        
        Ok(())
    }
    
    /// Update market predictions using AI models
    #[instrument(skip(self, account), fields(account_id = %account.id))]
    async fn update_market_predictions(&mut self, account: &UnifiedManagedAccount) -> ApiResult<()> {
        debug!(
            account_id = %account.id,
            "Updating market predictions"
        );
        
        // Get all securities in the account
        let mut securities = HashSet::new();
        
        for sleeve in &account.sleeves {
            for holding in &sleeve.holdings {
                securities.insert(holding.security_id.clone());
            }
        }
        
        debug!(
            security_count = securities.len(),
            "Found securities in account"
        );
        
        // Update predictions for each security
        for security_id in securities {
            if self.config.use_bedrock {
                trace!(
                    security_id = %security_id,
                    "Getting Bedrock prediction"
                );
                
                if let Some(prediction) = self.get_bedrock_prediction(&security_id).await? {
                    debug!(
                        security_id = %security_id,
                        predicted_change = prediction.predicted_price_change,
                        confidence = prediction.confidence,
                        "Received Bedrock prediction"
                    );
                    
                    self.market_predictions.insert(security_id, prediction);
                }
            } else if self.config.use_sagemaker {
                trace!(
                    security_id = %security_id,
                    "Getting SageMaker prediction"
                );
                
                if let Some(prediction) = self.get_sagemaker_prediction(&security_id).await? {
                    debug!(
                        security_id = %security_id,
                        predicted_change = prediction.predicted_price_change,
                        confidence = prediction.confidence,
                        "Received SageMaker prediction"
                    );
                    
                    self.market_predictions.insert(security_id, prediction);
                }
            }
        }
        
        info!(
            account_id = %account.id,
            prediction_count = self.market_predictions.len(),
            "Updated market predictions"
        );
        
        Ok(())
    }
    
    /// Get market prediction from Amazon Bedrock
    #[instrument(skip(self), fields(security_id = %security_id))]
    async fn get_bedrock_prediction(&self, security_id: &str) -> ApiResult<Option<MarketPrediction>> {
        if let Some(ref client) = self.bedrock_client {
            if let Some(ref model_id) = self.config.bedrock_model_id {
                debug!(
                    security_id = %security_id,
                    model_id = %model_id,
                    "Requesting Bedrock prediction"
                );
                
                // Construct prompt for Bedrock
                let prompt = format!(
                    "You are a financial AI assistant specializing in market predictions for tax-loss harvesting. \
                    Based on current market conditions, provide a prediction for the security {} over the next 24 hours. \
                    Format your response as a JSON object with the following fields: \
                    predictedPriceChange (percentage as a decimal), \
                    confidence (0.0 to 1.0), \
                    rationale (brief explanation).",
                    security_id
                );
                
                // Create request body for Claude model
                let body = serde_json::json!({
                    "anthropic_version": "bedrock-2023-05-31",
                    "max_tokens": 1000,
                    "messages": [
                        {
                            "role": "user",
                            "content": prompt
                        }
                    ]
                });
                
                // Invoke Bedrock model
                let request = InvokeModelRequest::builder()
                    .model_id(model_id)
                    .body(serde_json::to_string(&body).unwrap().into_bytes())
                    .build();
                
                match client.invoke_model(request).await {
                    Ok(response) => {
                        trace!("Received Bedrock response");
                        
                        // Parse response
                        if let Ok(response_str) = String::from_utf8(response.body.to_vec()) {
                            if let Ok(response_json) = serde_json::from_str::<serde_json::Value>(&response_str) {
                                // Extract content from Claude response
                                if let Some(content) = response_json["content"].as_array() {
                                    if let Some(text) = content.iter().find_map(|item| item["text"].as_str()) {
                                        // Extract JSON from text
                                        if let Some(json_start) = text.find('{') {
                                            if let Some(json_end) = text.rfind('}') {
                                                let json_str = &text[json_start..=json_end];
                                                if let Ok(prediction_data) = serde_json::from_str::<serde_json::Value>(json_str) {
                                                    // Create prediction
                                                    let prediction = MarketPrediction {
                                                        security_id: security_id.to_string(),
                                                        predicted_price_change: prediction_data["predictedPriceChange"].as_f64().unwrap_or(0.0),
                                                        confidence: prediction_data["confidence"].as_f64().unwrap_or(0.5),
                                                        horizon_hours: 24,
                                                        timestamp: SystemTime::now(),
                                                        model: model_id.clone(),
                                                        features: HashMap::new(),
                                                    };
                                                    
                                                    debug!(
                                                        security_id = %security_id,
                                                        predicted_change = prediction.predicted_price_change,
                                                        confidence = prediction.confidence,
                                                        "Successfully parsed Bedrock prediction"
                                                    );
                                                    
                                                    return Ok(Some(prediction));
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                        
                        warn!(
                            security_id = %security_id,
                            "Failed to parse Bedrock response"
                        );
                        Ok(None)
                    },
                    Err(e) => {
                        error!(
                            error = %e,
                            security_id = %security_id,
                            "Bedrock API error"
                        );
                        Ok(None)
                    }
                }
            } else {
                trace!(
                    security_id = %security_id,
                    "No Bedrock model ID configured"
                );
                Ok(None)
            }
        } else {
            trace!(
                security_id = %security_id,
                "No Bedrock client available"
            );
            Ok(None)
        }
    }
    
    /// Get market prediction from SageMaker
    #[instrument(skip(self), fields(security_id = %security_id))]
    async fn get_sagemaker_prediction(&self, security_id: &str) -> ApiResult<Option<MarketPrediction>> {
        if let Some(ref client) = self.sagemaker_client {
            if let Some(ref endpoint) = self.config.sagemaker_endpoint {
                debug!(
                    security_id = %security_id,
                    endpoint = %endpoint,
                    "Requesting SageMaker prediction"
                );
                
                // Create request payload
                let payload = serde_json::json!({
                    "security_id": security_id,
                    "timestamp": SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_secs()
                });
                
                // Invoke SageMaker endpoint
                let request = InvokeEndpointRequest::builder()
                    .endpoint_name(endpoint)
                    .body(serde_json::to_string(&payload).unwrap().into_bytes())
                    .build();
                
                match client.invoke_endpoint(request).await {
                    Ok(response) => {
                        trace!("Received SageMaker response");
                        
                        // Parse response
                        if let Ok(response_str) = String::from_utf8(response.body.to_vec()) {
                            if let Ok(prediction_data) = serde_json::from_str::<serde_json::Value>(&response_str) {
                                // Create prediction
                                let prediction = MarketPrediction {
                                    security_id: security_id.to_string(),
                                    predicted_price_change: prediction_data["predicted_price_change"].as_f64().unwrap_or(0.0),
                                    confidence: prediction_data["confidence"].as_f64().unwrap_or(0.5),
                                    horizon_hours: 24,
                                    timestamp: SystemTime::now(),
                                    model: endpoint.clone(),
                                    features: HashMap::new(),
                                };
                                
                                debug!(
                                    security_id = %security_id,
                                    predicted_change = prediction.predicted_price_change,
                                    confidence = prediction.confidence,
                                    "Successfully parsed SageMaker prediction"
                                );
                                
                                return Ok(Some(prediction));
                            }
                        }
                        
                        warn!(
                            security_id = %security_id,
                            "Failed to parse SageMaker response"
                        );
                        Ok(None)
                    },
                    Err(e) => {
                        error!(
                            error = %e,
                            security_id = %security_id,
                            "SageMaker API error"
                        );
                        Ok(None)
                    }
                }
            } else {
                trace!(
                    security_id = %security_id,
                    "No SageMaker endpoint configured"
                );
                Ok(None)
            }
        } else {
            trace!(
                security_id = %security_id,
                "No SageMaker client available"
            );
            Ok(None)
        }
    }
    
    /// Calculate current market volatility
    #[instrument(skip(self))]
    async fn calculate_market_volatility(&self) -> ApiResult<f64> {
        debug!("Calculating market volatility");
        
        // In a real implementation, this would use market data APIs or ML models
        // For now, return a moderate volatility value
        let volatility = 0.15;
        
        debug!(volatility = volatility, "Market volatility calculated");
        
        Ok(volatility)
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
        
        // In a real implementation, this would calculate metrics from alert history
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
            opportunities = report.total_opportunities,
            "Generated performance metrics"
        );
        
        report
    }
} 