use std::io::{self, Write};
use conversational_advisor::create_conversational_advisor;

fn main() {
    println!("=== Conversational Financial Advisor Demo ===");
    println!("Type 'exit' to quit the demo.");
    println!("Enter your financial questions or statements below:");
    println!();
    
    // Create a conversational financial advisor
    let mut advisor = create_conversational_advisor();
    
    loop {
        // Print prompt
        print!("> ");
        io::stdout().flush().unwrap();
        
        // Read user input
        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();
        let input = input.trim();
        
        // Check if user wants to exit
        if input.to_lowercase() == "exit" {
            println!("Thank you for using the Conversational Financial Advisor. Goodbye!");
            break;
        }
        
        // Process the message and get a response
        let response = advisor.process_message(input);
        
        // Print the response
        println!("Financial Advisor: {}", response);
        println!();
        
        // Print detected topics and entities (for demonstration purposes)
        let topics = advisor.get_topics();
        let entities = advisor.get_entities();
        
        if !topics.is_empty() {
            println!("Detected Topics:");
            for topic in topics {
                println!("  - {} (Importance: {:?})", topic.name, topic.importance);
            }
            println!();
        }
        
        if !entities.is_empty() {
            println!("Detected Entities:");
            for entity in entities {
                let value_str = match entity.value {
                    Some(value) => format!("${:.2}", value),
                    None => "N/A".to_string(),
                };
                println!("  - {} ({}): {}", entity.name, entity.entity_type, value_str);
            }
            println!();
        }
    }
} 