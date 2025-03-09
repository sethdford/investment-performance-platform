use investment_management::{create_conversational_advisor, Message};

fn main() {
    println!("=================================================");
    println!("  Minimal Conversational Financial Advisor Demo");
    println!("=================================================");
    
    // Create a basic advisor (no enhanced NLP)
    let mut advisor = create_conversational_advisor();
    
    // Process a simple message
    let response = advisor.process_message("I want to save for retirement");
    
    // Display the response
    println!("User > I want to save for retirement");
    println!("Advisor > {}", response);
    
    // Display detected topics
    let topics = advisor.get_detected_topics();
    if !topics.is_empty() {
        println!("\nDetected Topics:");
        for topic in topics {
            println!("  - {} (Importance: {:?})", topic.name, topic.importance);
        }
    }
    
    // Display detected entities
    let entities = advisor.get_detected_entities();
    if !entities.is_empty() {
        println!("\nDetected Entities:");
        for entity in entities {
            if let Some(value) = entity.value {
                println!("  - {} ({:?}): ${:.2}", entity.name, entity.entity_type, value);
            } else {
                println!("  - {} ({:?})", entity.name, entity.entity_type);
            }
        }
    }
    
    println!("\n=================================================");
} 