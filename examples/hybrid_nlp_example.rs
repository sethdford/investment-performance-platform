use investment_management::financial_advisor::nlp::{
    HybridNlpService, NlpResponseSource, NlpConfidenceLevel, FinancialQueryIntent
};
use aws_sdk_bedrockruntime::Client as BedrockRuntimeClient;
use aws_config::BehaviorVersion;
use investment_management::financial_advisor::nlp::bedrock::{BedrockNlpConfig, BedrockModelProvider};
use std::error::Error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    println!("Hybrid NLP Example");
    println!("=================");
    
    // Create a rule-based only service for demonstration
    let rule_based_service = HybridNlpService::new_rule_based_only();
    
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
    
    println!("\nRule-Based NLP Service");
    println!("---------------------");
    
    // Process each query with the rule-based service
    for query in &queries {
        println!("\nQuery: {}", query);
        
        // Process the query
        match rule_based_service.process_query(query, None).await {
            Ok(response) => {
                // Print the recognized intent
                println!("Intent: {:?} (confidence: {:?})", 
                         response.processed_query.as_ref().map(|pq| &pq.intent).unwrap_or(&FinancialQueryIntent::Unknown), 
                         response.confidence_level);
                
                // Print extracted entities
                if let Some(processed_query) = &response.processed_query {
                    if !processed_query.entities.is_empty() {
                        println!("Entities:");
                        for entity in &processed_query.entities {
                            println!("  - {:?}: {}", entity.entity_type, entity.value);
                        }
                    }
                }
                
                // Print the response
                println!("Response: {}", response.response_text);
                println!("Source: {:?}", response.source);
            },
            Err(e) => {
                println!("Error processing query: {}", e);
            }
        }
    }
    
    // Uncomment to test with Bedrock (requires AWS credentials)
    /*
    println!("\nHybrid NLP Service with Bedrock");
    println!("------------------------------");
    
    // Initialize AWS SDK
    let aws_config = aws_config::defaults(BehaviorVersion::latest())
        .region("us-east-1")
        .load()
        .await;
    
    // Create Bedrock client
    let bedrock_client = BedrockRuntimeClient::new(&aws_config);
    
    // Create Bedrock NLP config
    let bedrock_config = BedrockNlpConfig {
        provider: BedrockModelProvider::Claude,
        version: "v2".to_string(),
        temperature: 0.2,
        max_tokens: 1000,
        top_p: 0.9,
        top_k: 250,
        stop_sequences: vec!["\n\nHuman:".to_string()],
    };
    
    // Create hybrid service
    let hybrid_service = HybridNlpService::new_with_bedrock(bedrock_client, bedrock_config)
        .with_rule_based_confidence_threshold(0.7)
        .with_llm_for_responses(true);
    
    // Process each query with the hybrid service
    for query in &queries {
        println!("\nQuery: {}", query);
        
        // Process the query
        match hybrid_service.process_query(query, None).await {
            Ok(response) => {
                // Print the recognized intent
                println!("Intent: {:?} (confidence: {:?})", 
                         response.processed_query.as_ref().map(|pq| &pq.intent).unwrap_or(&FinancialQueryIntent::Unknown), 
                         response.confidence_level);
                
                // Print extracted entities
                if let Some(processed_query) = &response.processed_query {
                    if !processed_query.entities.is_empty() {
                        println!("Entities:");
                        for entity in &processed_query.entities {
                            println!("  - {:?}: {}", entity.entity_type, entity.value);
                        }
                    } else {
                        println!("No entities extracted");
                    }
                } else {
                    println!("No processed query available");
                }
                
                // Print the response
                println!("Response: {}", response.response_text);
                println!("Source: {:?}", response.source);
            },
            Err(e) => {
                println!("Error processing query: {}", e);
            }
        }
    }
    */
    
    println!("\nExample completed successfully!");
    
    Ok(())
} 