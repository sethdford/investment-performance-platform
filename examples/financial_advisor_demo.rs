use std::io::{self, Write};
use investment_management::create_conversational_advisor;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("=== Modern Conversational Financial Advisor ===");
    println!("Initializing AI components...");
    
    // Try to create an enhanced advisor with LLM capabilities
    let advisor_result = investment_management::ConversationalAdvisor::new_enhanced().await;
    
    let mut use_enhanced = false;
    let mut advisor = match advisor_result {
        Ok(enhanced_advisor) => {
            println!("✅ Enhanced NLP capabilities initialized successfully!");
            println!("Using Claude 3 Sonnet for advanced conversational capabilities.");
            use_enhanced = true;
            enhanced_advisor
        }
        Err(e) => {
            println!("⚠️ Could not initialize enhanced NLP capabilities: {}", e);
            println!("Falling back to basic conversation capabilities.");
            create_conversational_advisor()
        }
    };
    
    println!("\n🤖 Financial Advisor is ready to assist you!");
    println!("Type your financial questions or statements. Type 'exit' to quit.\n");
    
    loop {
        print!("You > ");
        io::stdout().flush().unwrap();
        
        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();
        
        let input = input.trim();
        
        if input.to_lowercase() == "exit" {
            break;
        }
        
        // Process the user input
        let response = if use_enhanced {
            match advisor.process_message_enhanced(input).await {
                Ok(response) => response,
                Err(e) => {
                    eprintln!("Error processing message: {}", e);
                    "I'm sorry, I encountered an error processing your request.".to_string()
                }
            }
        } else {
            advisor.process_message(input)
        };
        
        // Display the response
        println!("\nAdvisor > {}\n", response);
        
        // Get and display detected topics if not using enhanced mode
        if !use_enhanced {
            let topics = advisor.get_detected_topics();
            if !topics.is_empty() {
                println!("📊 Detected Topics:");
                for topic in topics {
                    println!("  - {} (Importance: {:.2})", topic.name, topic.importance as u8);
                }
                println!();
            }
            
            // Get and display detected entities
            let entities = advisor.get_detected_entities();
            if !entities.is_empty() {
                println!("🔍 Detected Entities:");
                for entity in entities {
                    if let Some(value) = entity.value {
                        println!("  - {} ({:?}): ${:.2}", entity.name, entity.entity_type, value);
                    } else {
                        println!("  - {} ({:?})", entity.name, entity.entity_type);
                    }
                }
                println!();
            }
        }
    }
    
    println!("Thank you for using the Modern Conversational Financial Advisor!");
    Ok(())
} 