#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::time::SystemTime;
    
    use crate::tax::tlh::AlgorithmicTLHConfig;
    use crate::cloud::aws::tlh_alerts::{AwsTlhAlertConfig, AwsTlhAlertService, MarketPrediction};
    use crate::cloud::aws::mock::{
        MockAwsService, MockBedrockClientMock, MockSageMakerClientMock, MockSnsClientMock,
        MockLambdaClientMock, MockDynamoDbClientMock, MockCloudWatchClientMock, MockEventBridgeClientMock
    };
    use crate::portfolio::model::{
        UnifiedManagedAccount, PortfolioSleeve, PortfolioHolding, TaxOptimizationSettings, ESGScreeningParams
    };
    
    // Helper function to create a test account with holdings
    fn create_test_account() -> UnifiedManagedAccount {
        // Create holdings with some losses for tax-loss harvesting
        let holdings = vec![
            PortfolioHolding {
                security_id: "AAPL".to_string(),
                quantity: 100.0,
                market_value: 15000.0,
                cost_basis: 18000.0,
                purchase_date: "2023-01-15".to_string(),
                unrealized_gain_loss: -3000.0,
                unrealized_gain_loss_pct: -0.167,
                sector: "Technology".to_string(),
                asset_class: "Equity".to_string(),
                tax_lots: vec![],
            },
            PortfolioHolding {
                security_id: "MSFT".to_string(),
                quantity: 50.0,
                market_value: 17500.0,
                cost_basis: 15000.0,
                purchase_date: "2023-02-10".to_string(),
                unrealized_gain_loss: 2500.0,
                unrealized_gain_loss_pct: 0.167,
                sector: "Technology".to_string(),
                asset_class: "Equity".to_string(),
                tax_lots: vec![],
            },
            PortfolioHolding {
                security_id: "AMZN".to_string(),
                quantity: 30.0,
                market_value: 9000.0,
                cost_basis: 12000.0,
                purchase_date: "2023-03-05".to_string(),
                unrealized_gain_loss: -3000.0,
                unrealized_gain_loss_pct: -0.25,
                sector: "Consumer Discretionary".to_string(),
                asset_class: "Equity".to_string(),
                tax_lots: vec![],
            },
        ];
        
        let sleeve = PortfolioSleeve {
            id: "sleeve-1".to_string(),
            name: "Core Equity".to_string(),
            holdings,
            target_allocation: 1.0,
            current_allocation: 1.0,
            sleeve_type: "Equity".to_string(),
            manager: "Internal".to_string(),
            strategy: "Core".to_string(),
        };
        
        // Create tax optimization settings
        let tax_settings = Some(TaxOptimizationSettings {
            enable_tax_loss_harvesting: true,
            annual_tax_budget: 10000.0,
            prioritize_loss_harvesting: true,
            defer_short_term_gains: true,
            short_term_tax_rate: 0.35,
            long_term_tax_rate: 0.15,
            state_tax_rate: 0.05,
            minimum_tax_benefit: 100.0,
        });
        
        // Create ESG criteria
        let esg_criteria = Some(ESGScreeningParams {
            min_esg_score: 50.0,
            exclude_sectors: vec!["Tobacco".to_string(), "Weapons".to_string()],
            prioritize_factors: vec!["Climate Change".to_string(), "Diversity".to_string()],
            custom_screens: vec![],
        });
        
        UnifiedManagedAccount {
            id: "test-account-123".to_string(),
            name: "Test Account".to_string(),
            owner: "Test User".to_string(),
            sleeves: vec![sleeve],
            cash_balance: 10000.0,
            total_market_value: 51500.0,
            created_at: "2023-01-01".to_string(),
            updated_at: "2023-06-01".to_string(),
            tax_settings,
            esg_criteria,
        }
    }
    
    // Helper function to create a market prediction
    fn create_market_prediction(security_id: &str, price_change: f64, confidence: f64) -> MarketPrediction {
        MarketPrediction {
            security_id: security_id.to_string(),
            predicted_price_change: price_change,
            confidence,
            horizon_hours: 24,
            timestamp: SystemTime::now(),
            model: "test-model".to_string(),
            features: HashMap::new(),
        }
    }
    
    #[tokio::test]
    async fn test_scan_for_opportunities_with_mocks() {
        // Create test account
        let account = create_test_account();
        
        // Configure the AWS TLH Alert System
        let aws_config = AwsTlhAlertConfig {
            region: "us-east-1".to_string(),
            use_bedrock: true,
            bedrock_model_id: Some("anthropic.claude-3-sonnet-20240229-v1:0".to_string()),
            use_sagemaker: false,
            sagemaker_endpoint: None,
            sns_topic_arn: Some("arn:aws:sns:us-east-1:123456789012:TaxLossHarvestingAlerts".to_string()),
            lambda_function_arn: None,
            dynamodb_table: Some("TaxLossHarvestingAlerts".to_string()),
            cloudwatch_namespace: Some("TaxLossHarvesting".to_string()),
            eventbridge_bus_name: Some("default".to_string()),
            alert_frequency_seconds: 60,
            confidence_threshold: 0.7,
            real_time_market_data: true,
            backtesting_mode: false,
        };
        
        // Configure the Algorithmic TLH Service
        let tlh_config = AlgorithmicTLHConfig::default();
        
        // Create mock Bedrock client
        let mut bedrock_mock = MockBedrockClientMock::new();
        
        // Setup expectations for the Bedrock client
        // ... (setup expectations here)
        
        // Create mock DynamoDB client
        let mut dynamodb_mock = MockDynamoDbClientMock::new();
        
        // Setup expectations for the DynamoDB client
        // ... (setup expectations here)
        
        // Create mock SNS client
        let mut sns_mock = MockSnsClientMock::new();
        
        // Setup expectations for the SNS client
        // ... (setup expectations here)
        
        // Create the AWS TLH Alert Service with mock clients
        let mut alert_service = AwsTlhAlertService::with_mock_clients(
            aws_config,
            tlh_config,
            Some(Box::new(bedrock_mock)),
            None, // No SageMaker client
            Some(Box::new(sns_mock)),
            None, // No Lambda client
            Some(Box::new(dynamodb_mock)),
            None, // No CloudWatch client
            None, // No EventBridge client
        );
        
        // Scan for opportunities
        let alerts = alert_service.scan_for_opportunities(&account).await.unwrap();
        
        // Verify that alerts were generated
        assert!(!alerts.is_empty(), "No alerts were generated");
        
        // ... (rest of the test)
    }
    
    #[tokio::test]
    async fn test_process_alert() {
        // Configure the AWS TLH Alert System
        let aws_config = AwsTlhAlertConfig {
            region: "us-east-1".to_string(),
            use_bedrock: true,
            bedrock_model_id: Some("anthropic.claude-3-sonnet-20240229-v1:0".to_string()),
            use_sagemaker: false,
            sagemaker_endpoint: None,
            sns_topic_arn: Some("arn:aws:sns:us-east-1:123456789012:TaxLossHarvestingAlerts".to_string()),
            lambda_function_arn: Some("arn:aws:lambda:us-east-1:123456789012:function:ProcessTLHAlert".to_string()),
            dynamodb_table: Some("TaxLossHarvestingAlerts".to_string()),
            cloudwatch_namespace: Some("TaxLossHarvesting".to_string()),
            eventbridge_bus_name: Some("default".to_string()),
            alert_frequency_seconds: 60,
            confidence_threshold: 0.7,
            real_time_market_data: true,
            backtesting_mode: false,
        };
        
        // Configure the Algorithmic TLH Service
        let tlh_config = AlgorithmicTLHConfig::default();
        
        // Create mock AWS service
        let mut mock_aws = MockAwsService::new();
        
        // Setup mocks for all AWS services
        mock_aws.setup_dynamodb_store_alert();
        mock_aws.setup_sns_publish();
        mock_aws.setup_lambda_invoke();
        mock_aws.setup_cloudwatch_metrics();
        mock_aws.setup_eventbridge_events();
        
        // Create the AWS TLH Alert Service
        let alert_service = AwsTlhAlertService::new(aws_config, tlh_config).await;
        
        // Create a test alert
        let alert = crate::cloud::aws_tlh_alerts::TlhOpportunityAlert {
            id: "test-alert-123".to_string(),
            account_id: "test-account-123".to_string(),
            security_id: "AAPL".to_string(),
            current_price: 150.0,
            cost_basis: 180.0,
            unrealized_loss: -3000.0,
            unrealized_loss_percentage: -0.167,
            estimated_tax_savings: 1050.0,
            priority: 2,
            recommended_action: "Sell AAPL and replace with MSFT".to_string(),
            replacement_securities: vec!["MSFT".to_string()],
            market_prediction: Some(create_market_prediction("AAPL", -0.05, 0.85)),
            timestamp: SystemTime::now(),
            expiration: SystemTime::now(),
        };
        
        // Process the alert
        let result = alert_service.process_alert(&alert).await;
        
        // Verify that the alert was processed successfully
        assert!(result.is_ok(), "Alert processing failed: {:?}", result.err());
    }
    
    #[tokio::test]
    async fn test_get_bedrock_prediction() {
        // Configure the AWS TLH Alert System
        let aws_config = AwsTlhAlertConfig {
            region: "us-east-1".to_string(),
            use_bedrock: true,
            bedrock_model_id: Some("anthropic.claude-3-sonnet-20240229-v1:0".to_string()),
            use_sagemaker: false,
            sagemaker_endpoint: None,
            sns_topic_arn: None,
            lambda_function_arn: None,
            dynamodb_table: None,
            cloudwatch_namespace: None,
            eventbridge_bus_name: None,
            alert_frequency_seconds: 60,
            confidence_threshold: 0.7,
            real_time_market_data: true,
            backtesting_mode: false,
        };
        
        // Configure the Algorithmic TLH Service
        let tlh_config = AlgorithmicTLHConfig::default();
        
        // Create mock AWS service
        let mut mock_aws = MockAwsService::new();
        
        // Setup mock for Bedrock client
        let expected_prediction = create_market_prediction("AAPL", -0.05, 0.85);
        mock_aws.setup_bedrock_prediction("AAPL", expected_prediction.clone());
        
        // Create the AWS TLH Alert Service
        let alert_service = AwsTlhAlertService::new(aws_config, tlh_config).await;
        
        // Get prediction
        let prediction_result = alert_service.get_bedrock_prediction("AAPL").await;
        
        // Verify that the prediction was retrieved successfully
        assert!(prediction_result.is_ok(), "Prediction retrieval failed: {:?}", prediction_result.err());
        
        // Verify that the prediction matches the expected prediction
        let prediction = prediction_result.unwrap();
        assert!(prediction.is_some(), "No prediction was returned");
        
        let prediction = prediction.unwrap();
        assert_eq!(prediction.security_id, "AAPL", "Prediction security ID should be AAPL");
        assert_eq!(prediction.predicted_price_change, -0.05, "Prediction price change should be -0.05");
        assert_eq!(prediction.confidence, 0.85, "Prediction confidence should be 0.85");
    }
} 