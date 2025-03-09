use investment_management::{ConversationalAdvisor, Message};
use std::io::{self, Write};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("=================================================");
    println!("  Simple Conversational Financial Advisor Demo");
    println!("=================================================");
    println!("Type 'exit' to quit");
    println!("=================================================\n");
    
    // Create a basic advisor (no enhanced NLP)
    let mut advisor = investment_management::create_conversational_advisor();
    
    loop {
        print!("You > ");
        io::stdout().flush()?;
        
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        
        let input = input.trim();
        
        if input.is_empty() {
            continue;
        }
        
        if input.to_lowercase() == "exit" {
            break;
        }
        
        // Process the message
        let response = advisor.process_message(input);
        
        // Display the response
        println!("Advisor > {}", response);
        
        // Display detected topics and entities
        let topics = advisor.get_detected_topics();
        if !topics.is_empty() {
            println!("\nDetected Topics:");
            for topic in topics {
                println!("  - {} (Importance: {:?})", topic.name, topic.importance);
            }
        }
        
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
        
        println!();
    }
    
    println!("Thank you for using the Simple Conversational Financial Advisor!");
    
    Ok(())
} 