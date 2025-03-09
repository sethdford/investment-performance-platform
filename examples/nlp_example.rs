use investment_management::financial_advisor::nlp::{
    FinancialNlpService, FinancialQueryIntent, EntityType
};

fn main() {
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
    
    println!("\nExample completed successfully!");
} 