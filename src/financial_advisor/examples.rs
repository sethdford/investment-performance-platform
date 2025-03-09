use crate::financial_advisor::{FinancialAdvisorService, NotificationPreferences, FinancialAdvisorEventType, RiskProfileResponse, FinancialAdvisorConfig};
use crate::financial_advisor::streaming_handler::FinancialAdvisorEventHandler;
use crate::factor_model::FactorModelApi;
use crate::portfolio::rebalancing::{PortfolioRebalancingService, Portfolio, PortfolioHolding, CashFlow, CashFlowType};
// TODO: Uncomment when performance_calculator module is available
use crate::performance_calculator::calculations::streaming::{StreamingProcessor, StreamingConfig, StreamingEvent};
use std::collections::HashMap;
use std::sync::Arc;
use chrono::Utc;
use anyhow::Result;
use tracing::info;

// Placeholder for StreamingEvent until performance_calculator module is available
// #[derive(Debug, Clone)]
// pub struct StreamingEvent {
//     /// Event type
//     pub event_type: String,
//     
//     /// Entity ID
//     pub entity_id: String,
//     
//     /// Event payload
//     pub payload: serde_json::Value,
//     
//     /// Timestamp
//     pub timestamp: chrono::DateTime<chrono::Utc>,
// }

// Placeholder for StreamingProcessor until performance_calculator module is available
// pub struct StreamingProcessor {
//     // In a real implementation, this would be a proper streaming processor
// }

// impl StreamingProcessor {
//     pub fn new() -> Self {
//         Self {}
//     }
//     
//     pub async fn submit_event(&self, event: StreamingEvent) -> Result<()> {
//         // In a real implementation, this would submit the event to a streaming processor
//         Ok(())
//     }
// }

// Placeholder for StreamingConfig until performance_calculator module is available
// pub struct StreamingConfig {
//     // In a real implementation, this would be a proper streaming config
// }

// impl Default for StreamingConfig {
//     fn default() -> Self {
//         Self {}
//     }
// }

/// Run a real-time financial advisor example
pub async fn run_real_time_financial_advisor_example() -> Result<()> {
    // Create dependencies
    let factor_model_api = FactorModelApi::new();
    let rebalancing_service = PortfolioRebalancingService::new(factor_model_api.clone());
    
    // Create financial advisor service
    let config = FinancialAdvisorConfig::default();
    let advisor_service = FinancialAdvisorService::new(config, Some(rebalancing_service)).await?;
    
    // Set notification preferences
    let preferences = NotificationPreferences {
        email_enabled: true,
        email_address: Some("user@example.com".to_string()),
        push_enabled: false,
        device_token: None,
        sms_enabled: false,
        phone_number: None,
        min_priority: 2,
        event_types: vec![
            FinancialAdvisorEventType::PortfolioDrift,
            FinancialAdvisorEventType::TaxLossHarvesting,
            FinancialAdvisorEventType::MarketVolatility,
        ],
    };
    
    advisor_service.set_notification_preferences("user123", preferences).await?;
    
    // Create a test portfolio
    let portfolio = create_test_portfolio();
    
    // Process a cash flow
    let cash_flow = CashFlow {
        amount: 10000.0,
        date: "2023-01-15".to_string(),
        flow_type: CashFlowType::Deposit,
    };
    
    println!("Processing cash flow...");
    let recommendation = advisor_service.handle_cash_flow(&portfolio, &cash_flow, "user123").await?;
    if let Some(rec) = recommendation {
        println!("Cash flow recommendation: {}", rec.title);
        println!("Description: {}", rec.description);
        if let Some(trades) = &rec.recommended_trades {
            println!("Recommended trades:");
            for trade in trades {
                println!("  {} {} ${:.2}", 
                    if trade.is_buy { "Buy" } else { "Sell" },
                    trade.security_id,
                    trade.amount);
            }
        }
    } else {
        println!("No recommendation generated for cash flow");
    }
    
    // Check for portfolio drift
    println!("\nChecking portfolio drift...");
    let recommendation = advisor_service.check_portfolio_drift(&portfolio, "user123").await?;
    if let Some(rec) = recommendation {
        println!("Portfolio drift recommendation: {}", rec.title);
        println!("Description: {}", rec.description);
        if let Some(trades) = &rec.recommended_trades {
            println!("Recommended trades:");
            for trade in trades {
                println!("  {} {} ${:.2}", 
                    if trade.is_buy { "Buy" } else { "Sell" },
                    trade.security_id,
                    trade.amount);
            }
        }
    } else {
        println!("No recommendation generated for portfolio drift");
    }
    
    // Check for tax loss harvesting opportunities
    println!("\nChecking tax loss harvesting opportunities...");
    let recommendation = advisor_service.check_tax_loss_harvesting(&portfolio, "user123").await?;
    if let Some(rec) = recommendation {
        println!("Tax loss harvesting recommendation: {}", rec.title);
        println!("Description: {}", rec.description);
        if let Some(trades) = &rec.recommended_trades {
            println!("Recommended trades:");
            for trade in trades {
                println!("  {} {} ${:.2}", 
                    if trade.is_buy { "Buy" } else { "Sell" },
                    trade.security_id,
                    trade.amount);
            }
        }
    } else {
        println!("No recommendation generated for tax loss harvesting");
    }
    
    // Get recent recommendations
    println!("\nGetting recent recommendations...");
    let recommendations = advisor_service.get_recent_recommendations(&portfolio.id).await?;
    println!("Found {} recent recommendations", recommendations.len());
    for rec in recommendations {
        println!("  {} ({:?})", rec.title, rec.recommendation_type);
    }
    
    Ok(())
}

/// Create a test portfolio
fn create_test_portfolio() -> Portfolio {
    Portfolio {
        id: "portfolio-123".to_string(),
        name: "Test Portfolio".to_string(),
        total_market_value: 100000.0,
        cash_balance: 5000.0,
        holdings: vec![
            PortfolioHolding {
                security_id: "VTI".to_string(),
                market_value: 30000.0,
                weight: 0.3,
                target_weight: 0.4,
                cost_basis: 25000.0,
                purchase_date: "2022-01-01".to_string(),
                factor_exposures: HashMap::new(),
            },
            PortfolioHolding {
                security_id: "BND".to_string(),
                market_value: 20000.0,
                weight: 0.2,
                target_weight: 0.3,
                cost_basis: 22000.0,
                purchase_date: "2022-01-01".to_string(),
                factor_exposures: HashMap::new(),
            },
            PortfolioHolding {
                security_id: "VEA".to_string(),
                market_value: 25000.0,
                weight: 0.25,
                target_weight: 0.2,
                cost_basis: 28000.0,
                purchase_date: "2022-01-01".to_string(),
                factor_exposures: HashMap::new(),
            },
            PortfolioHolding {
                security_id: "VWO".to_string(),
                market_value: 20000.0,
                weight: 0.2,
                target_weight: 0.1,
                cost_basis: 18000.0,
                purchase_date: "2022-01-01".to_string(),
                factor_exposures: HashMap::new(),
            },
        ],
    }
}

/// Create a test event
fn create_event(event_type: &str, entity_id: &str, payload: serde_json::Value) -> StreamingEvent {
    // Convert the JSON Value to a HashMap
    let mut payload_map = std::collections::HashMap::new();
    if let serde_json::Value::Object(map) = payload {
        for (key, value) in map {
            payload_map.insert(key, value);
        }
    }
    
    StreamingEvent {
        id: uuid::Uuid::new_v4().to_string(),
        timestamp: chrono::Utc::now(),
        event_type: event_type.to_string(),
        source: "test".to_string(),
        entity_id: entity_id.to_string(),
        payload: payload_map,
    }
}

/// Run a risk profiling example
pub fn run_risk_profiling_example() -> Result<()> {
    println!("Starting Risk Profiling Example");
    
    // Create dependencies
    let factor_model_api = FactorModelApi::new();
    let rebalancing_service = PortfolioRebalancingService::new(factor_model_api.clone());
    
    // Create financial advisor service
    let config = FinancialAdvisorConfig::default();
    let advisor_service = tokio::runtime::Runtime::new()
        .unwrap()
        .block_on(async {
            FinancialAdvisorService::new(config, Some(rebalancing_service)).await.unwrap()
        });
    
    // Get the risk profiling service
    let risk_profiling_service = advisor_service.get_risk_profiling_service();
    
    // Get the comprehensive risk profile questionnaire
    let questionnaire = risk_profiling_service.get_questionnaire("comprehensive_risk_profile")
        .expect("Questionnaire not found");
    
    println!("\nRisk Profiling Questionnaire: {}", questionnaire.name);
    println!("Description: {}", questionnaire.description);
    println!("\nQuestions:");
    
    // Print all questions and options
    for (i, question) in questionnaire.questions.iter().enumerate() {
        println!("\n{}. {} (Category: {})", i + 1, question.text, question.category);
        println!("   Options:");
        for option in &question.options {
            println!("     {}. {} (Risk Score: {})", option.value, option.text, option.risk_score);
        }
    }
    
    // Create some sample responses
    let responses = vec![
        RiskProfileResponse {
            question_id: "time_horizon".to_string(),
            response_value: 4,
            comments: None,
            timestamp: Utc::now(),
        },
        RiskProfileResponse {
            question_id: "investment_knowledge".to_string(),
            response_value: 3,
            comments: None,
            timestamp: Utc::now(),
        },
        RiskProfileResponse {
            question_id: "loss_tolerance".to_string(),
            response_value: 2,
            comments: None,
            timestamp: Utc::now(),
        },
        RiskProfileResponse {
            question_id: "income_stability".to_string(),
            response_value: 4,
            comments: None,
            timestamp: Utc::now(),
        },
        RiskProfileResponse {
            question_id: "emergency_fund".to_string(),
            response_value: 5,
            comments: None,
            timestamp: Utc::now(),
        },
    ];
    
    // Calculate risk tolerance
    let risk_tolerance = risk_profiling_service.calculate_risk_tolerance(&responses, "comprehensive_risk_profile")?;
    
    println!("\nCalculated Risk Tolerance: {:?}", risk_tolerance);
    
    // Detect behavioral biases
    let biases = risk_profiling_service.detect_behavioral_biases(&responses);
    
    println!("\nDetected Behavioral Biases:");
    for bias in biases {
        println!("  - {:?}", bias);
    }
    
    Ok(())
}

/// Example demonstrating the use of the NLP module for financial queries
pub fn nlp_example() {
    use crate::financial_advisor::nlp::FinancialNlpService;
    
    
    println!("Financial Advisor NLP Example");
    println!("============================");
    
    // Create a new NLP service
    let nlp_service = FinancialNlpService::new();
    
    // Example queries
    let queries = vec![
        "How is my portfolio performing this year?",
        "What is my current asset allocation?",
        "Am I on track for retirement?",
        "How can I reduce my taxes?",
        "When can I retire?",
        "What are my monthly expenses?",
        "What should I invest in?",
        "How risky is my portfolio?",
        "What's happening in the markets?",
        "Can you explain dollar cost averaging?",
    ];
    
    // Process each query
    for query in queries {
        println!("\nQuery: {}", query);
        
        // Process the query
        match nlp_service.process_query(query) {
            Ok(processed) => {
                // Print the recognized intent
                println!("Intent: {:?} (confidence: {:.2})", processed.intent, processed.intent_confidence);
                
                // Print extracted entities
                if !processed.entities.is_empty() {
                    println!("Entities:");
                    for entity in &processed.entities {
                        println!("  - {:?}: {} (confidence: {:.2})", 
                                entity.entity_type, entity.value, entity.confidence);
                    }
                } else {
                    println!("No entities extracted");
                }
                
                // Generate and print a response
                let response = nlp_service.generate_response(&processed);
                println!("Response: {}", response);
            },
            Err(e) => {
                println!("Error processing query: {}", e);
            }
        }
    }
    
    println!("\nInteractive Mode (type 'exit' to quit)");
    println!("-------------------------------------");
    
    // Interactive mode
    loop {
        print!("\nEnter a financial question: ");
        std::io::Write::flush(&mut std::io::stdout()).unwrap();
        
        let mut input = String::new();
        std::io::stdin().read_line(&mut input).unwrap();
        
        let input = input.trim();
        if input.to_lowercase() == "exit" {
            break;
        }
        
        // Process the query
        match nlp_service.process_query(input) {
            Ok(processed) => {
                // Print the recognized intent
                println!("Intent: {:?} (confidence: {:.2})", processed.intent, processed.intent_confidence);
                
                // Generate and print a response
                let response = nlp_service.generate_response(&processed);
                println!("Response: {}", response);
            },
            Err(e) => {
                println!("Error processing query: {}", e);
            }
        }
    }
}

/// Example of using the financial advisor with streaming data
pub async fn streaming_example() -> Result<()> {
    // Create a financial advisor service
    let config = FinancialAdvisorConfig::default();
    let advisor_service = FinancialAdvisorService::new(config, None).await?;
    
    // Create a streaming processor
    let streaming_config = StreamingConfig::default();
    let advisor_handler_box = Box::new(FinancialAdvisorEventHandler::new(Arc::new(advisor_service.clone())));
    let _streaming_processor = StreamingProcessor::new(
        streaming_config,
        advisor_handler_box,
    );
    
    // In a real implementation, we would register handlers and start the processor
    // For this example, we'll just return success
    info!("Streaming processor created successfully");
    
    Ok(())
} 