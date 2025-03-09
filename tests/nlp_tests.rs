use investment_management::financial_advisor::nlp::{
    FinancialNlpService, FinancialQueryIntent, EntityType,
    EnhancedHybridConfig
};

// ============================================================================
// Rule-Based NLP Service Tests
// ============================================================================

#[test]
fn test_portfolio_performance_intent() {
    let nlp_service = FinancialNlpService::new();
    
    let queries = vec![
        "How is my portfolio performing?",
        "What's my portfolio return this year?",
        "How well have my investments done?",
        "What was my performance last quarter?",
    ];
    
    for query in queries {
        let processed = nlp_service.process_query(query).unwrap();
        assert_eq!(processed.intent, FinancialQueryIntent::PortfolioPerformance);
    }
}

#[test]
fn test_asset_allocation_intent() {
    let nlp_service = FinancialNlpService::new();
    
    let queries = vec![
        "What is my current asset allocation?",
        "How are my investments allocated?",
        "Show me my portfolio breakdown",
        "What's my exposure to stocks?",
    ];
    
    for query in queries {
        let processed = nlp_service.process_query(query).unwrap();
        assert_eq!(processed.intent, FinancialQueryIntent::AssetAllocation);
    }
}

#[test]
fn test_greeting_intent() {
    let nlp_service = FinancialNlpService::new();
    
    let queries = vec![
        "Hi",
        "Hello",
        "Hey there",
        "Good morning"
    ];
    
    for query in queries {
        let processed = nlp_service.process_query(query).unwrap();
        println!("Query: '{}', Intent: {:?}", query, processed.intent);
        assert_eq!(processed.intent, FinancialQueryIntent::Greeting);
    }
}

#[test]
fn test_entity_extraction() {
    let nlp_service = FinancialNlpService::new();
    
    // Test time period extraction
    let query1 = "How did my portfolio perform last year?";
    let processed1 = nlp_service.process_query(query1).unwrap();
    assert!(processed1.entities.iter().any(|e| 
        e.entity_type == EntityType::TimePeriod && e.value == "last year"
    ));
    
    // Test account type extraction
    let query2 = "What's the balance in my 401k?";
    let processed2 = nlp_service.process_query(query2).unwrap();
    assert!(processed2.entities.iter().any(|e| 
        e.entity_type == EntityType::AccountType && e.value == "401k"
    ));
    
    // Test amount extraction
    let query3 = "Can I save $500 per month for retirement?";
    let processed3 = nlp_service.process_query(query3).unwrap();
    assert!(processed3.entities.iter().any(|e| 
        e.entity_type == EntityType::Amount && e.value == "$500"
    ));
    
    // Test goal extraction
    let query4 = "Am I on track for retirement?";
    let processed4 = nlp_service.process_query(query4).unwrap();
    assert!(processed4.entities.iter().any(|e| 
        e.entity_type == EntityType::Goal && e.value == "retirement"
    ));
}

#[test]
fn test_response_generation() {
    let nlp_service = FinancialNlpService::new();
    
    // Test portfolio performance response
    let query1 = "How is my portfolio performing?";
    let processed1 = nlp_service.process_query(query1).unwrap();
    let response1 = nlp_service.generate_response(&processed1);
    assert!(!response1.is_empty());
    assert!(response1.contains("portfolio"));
    
    // Test asset allocation response
    let query2 = "What is my current asset allocation?";
    let processed2 = nlp_service.process_query(query2).unwrap();
    let response2 = nlp_service.generate_response(&processed2);
    assert!(!response2.is_empty());
    assert!(response2.contains("asset allocation"));
    
    // Test greeting response
    let query3 = "Hi";
    let processed3 = nlp_service.process_query(query3).unwrap();
    let response3 = nlp_service.generate_response(&processed3);
    assert!(!response3.is_empty());
    assert!(response3.contains("Hello"));
    assert!(!response3.contains("portfolio"));
    
    // Test unknown intent response
    let query4 = "What's the weather like today?";
    let processed4 = nlp_service.process_query(query4).unwrap();
    let response4 = nlp_service.generate_response(&processed4);
    assert!(!response4.is_empty());
    assert!(response4.contains("not sure"));
}

#[test]
fn test_multiple_entities() {
    let nlp_service = FinancialNlpService::new();
    
    let query = "How did my 401k perform last year with a $10,000 investment?";
    let processed = nlp_service.process_query(query).unwrap();
    
    // Should extract time period, account type, and amount
    assert!(processed.entities.iter().any(|e| 
        e.entity_type == EntityType::TimePeriod && e.value == "last year"
    ));
    
    assert!(processed.entities.iter().any(|e| 
        e.entity_type == EntityType::AccountType && e.value == "401k"
    ));
    
    assert!(processed.entities.iter().any(|e| 
        e.entity_type == EntityType::Amount && e.value == "$10,000"
    ));
}

// ============================================================================
// Enhanced Hybrid Service Tests
// ============================================================================

#[test]
fn test_enhanced_hybrid_config() {
    let config = EnhancedHybridConfig::default();
    assert_eq!(config.rule_based_confidence_threshold, 0.7);
    assert!(config.use_llm_for_responses);
    assert!(config.use_embeddings_for_intents);
    assert!(config.use_conversation_context);
    assert!(config.use_knowledge_retrieval);
} 