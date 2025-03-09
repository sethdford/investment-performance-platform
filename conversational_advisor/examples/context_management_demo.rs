use std::io::{self, Write};
use std::path::Path;
use conversational_advisor::conversation::context_manager::{ContextManager, ContextManagerConfig, ImportanceLevel};
use conversational_advisor::conversation::Message;
use conversational_advisor::financial_entities::{FinancialEntity, entity_extractor::FinancialEntityType};

fn main() {
    println!("=== Enhanced Context Management Demo ===");
    println!("This demo showcases the enhanced context management capabilities:");
    println!("1. Context window management");
    println!("2. Context relevance scoring");
    println!("3. Context persistence");
    println!();
    
    // Create a directory for persistence
    let persistence_dir = "context_demo";
    std::fs::create_dir_all(persistence_dir).unwrap_or_else(|_| {
        println!("Note: Could not create persistence directory. Context will not be saved.");
    });
    
    // Create a context manager with custom configuration
    let config = ContextManagerConfig {
        max_context_messages: 10,
        max_context_tokens: 2000,
        max_segments: 5,
        min_segment_importance: 0.3,
        persistence_path: Some(format!("{}/demo_context.json", persistence_dir)),
        auto_persist: true,
    };
    
    let mut context_manager = ContextManager::with_config(config);
    
    // Add some initial messages and entities to simulate a conversation
    simulate_initial_conversation(&mut context_manager);
    
    // Main interaction loop
    loop {
        println!("\nOptions:");
        println!("1. Add a message");
        println!("2. View active context");
        println!("3. View relevant context");
        println!("4. View topics");
        println!("5. View segments");
        println!("6. Save context");
        println!("7. Load context");
        println!("8. Exit");
        print!("\nEnter your choice: ");
        io::stdout().flush().unwrap();
        
        let mut choice = String::new();
        io::stdin().read_line(&mut choice).unwrap();
        let choice = choice.trim();
        
        match choice {
            "1" => add_message(&mut context_manager),
            "2" => view_active_context(&context_manager),
            "3" => view_relevant_context(&context_manager),
            "4" => view_topics(&context_manager),
            "5" => view_segments(&context_manager),
            "6" => save_context(&context_manager, persistence_dir),
            "7" => {
                context_manager = load_context(persistence_dir).unwrap_or_else(|_| {
                    println!("Could not load context. Using current context.");
                    context_manager
                });
            },
            "8" => {
                println!("Exiting demo. Thank you!");
                break;
            },
            _ => println!("Invalid choice. Please try again."),
        }
    }
}

fn simulate_initial_conversation(context_manager: &mut ContextManager) {
    println!("Simulating initial conversation...");
    
    // Add some messages
    let messages = [
        "Hello, I need help with my retirement planning.",
        "I'm 45 years old and want to retire at 65.",
        "I currently have $250,000 in my 401(k).",
        "I also have a Roth IRA with about $50,000.",
        "My current annual income is $120,000.",
        "I'm contributing 10% of my salary to my 401(k).",
        "My employer matches 5% of my contributions.",
    ];
    
    for message in messages.iter() {
        let user_message = Message::from_user(message);
        context_manager.add_message(user_message);
        
        // Simulate assistant response
        let response = format!("I understand. Let me help you with that information about {}.", 
            if message.contains("retirement") {
                "retirement planning"
            } else if message.contains("401(k)") {
                "your 401(k)"
            } else if message.contains("Roth IRA") {
                "your Roth IRA"
            } else if message.contains("income") {
                "your income"
            } else if message.contains("contributing") {
                "your contributions"
            } else {
                "your financial situation"
            }
        );
        
        let assistant_message = Message::from_assistant(&response);
        context_manager.add_message(assistant_message);
    }
    
    println!("Initial conversation simulated with {} messages.", messages.len() * 2);
}

fn add_message(context_manager: &mut ContextManager) {
    print!("Enter your message: ");
    io::stdout().flush().unwrap();
    
    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();
    let input = input.trim();
    
    if !input.is_empty() {
        // Create and add user message
        let user_message = Message::from_user(input);
        context_manager.add_message(user_message);
        
        // Simulate topic and entity detection
        detect_topics_and_entities(context_manager, input);
        
        // Simulate assistant response
        let response = "Thank you for sharing that information. I've updated your financial profile.";
        let assistant_message = Message::from_assistant(response);
        context_manager.add_message(assistant_message);
        
        println!("Message added to context.");
    } else {
        println!("Empty message not added.");
    }
}

fn detect_topics_and_entities(context_manager: &mut ContextManager, input: &str) {
    // This is a simplified simulation of topic and entity detection
    // In a real implementation, this would use NLP to extract topics and entities
    
    // Check for retirement-related topics
    if input.to_lowercase().contains("retire") || input.to_lowercase().contains("retirement") {
        println!("Detected retirement planning topic.");
    }
    
    // Check for investment-related topics
    if input.to_lowercase().contains("invest") || input.to_lowercase().contains("stock") || 
       input.to_lowercase().contains("bond") || input.to_lowercase().contains("fund") {
        println!("Detected investment topic.");
    }
    
    // Check for tax-related topics
    if input.to_lowercase().contains("tax") || input.to_lowercase().contains("deduction") {
        println!("Detected tax planning topic.");
    }
    
    // Check for account-related entities
    if input.to_lowercase().contains("401k") || input.to_lowercase().contains("401(k)") {
        println!("Detected 401(k) account entity.");
    }
    
    if input.to_lowercase().contains("ira") || input.to_lowercase().contains("roth") {
        println!("Detected IRA account entity.");
    }
    
    // Check for dollar amounts
    let dollar_regex = regex::Regex::new(r"\$(\d{1,3}(,\d{3})*|\d+)(\.\d+)?").unwrap();
    if let Some(captures) = dollar_regex.captures(input) {
        if let Some(amount_match) = captures.get(0) {
            println!("Detected dollar amount: {}", amount_match.as_str());
        }
    }
}

fn view_active_context(context_manager: &ContextManager) {
    let active_context = context_manager.get_active_context();
    
    println!("\n=== Active Context ({} messages) ===", active_context.len());
    
    if active_context.is_empty() {
        println!("No messages in active context.");
    } else {
        for (i, message) in active_context.iter().enumerate() {
            let role = if message.is_user() {
                "User"
            } else if message.is_assistant() {
                "Assistant"
            } else {
                "System"
            };
            
            println!("{}. [{}]: {}", i + 1, role, message.content);
        }
    }
}

fn view_relevant_context(context_manager: &ContextManager) {
    let relevant_context = context_manager.get_relevant_context();
    
    println!("\n=== Relevant Context ({} messages) ===", relevant_context.len());
    
    if relevant_context.is_empty() {
        println!("No messages in relevant context.");
    } else {
        for (i, message) in relevant_context.iter().enumerate() {
            let role = if message.is_user() {
                "User"
            } else if message.is_assistant() {
                "Assistant"
            } else {
                "System"
            };
            
            println!("{}. [{}]: {}", i + 1, role, message.content);
        }
    }
}

fn view_topics(context_manager: &ContextManager) {
    let topics = context_manager.get_current_topics();
    
    println!("\n=== Current Topics ({} topics) ===", topics.len());
    
    if topics.is_empty() {
        println!("No topics detected.");
    } else {
        for (i, topic) in topics.iter().enumerate() {
            println!("{}. {} (Importance: {:?}, Score: {:.2})", 
                i + 1, 
                topic.name, 
                topic.importance,
                topic.calculate_overall_score());
        }
    }
}

fn view_segments(context_manager: &ContextManager) {
    // This is a simplified view since we don't have direct access to segments
    // In a real implementation, we might add a method to get segments
    
    println!("\n=== Context Segments ===");
    println!("Context is organized into segments based on topic shifts and importance.");
    println!("Segments are used to prioritize relevant information when generating responses.");
    println!("The most relevant segments are included in the context window based on:");
    println!("- Recency (how recently the segment was created)");
    println!("- Relevance (how relevant the segment is to the current conversation)");
    println!("- Importance (how important the information in the segment is)");
}

fn save_context(context_manager: &ContextManager, persistence_dir: &str) {
    let path = format!("{}/manual_save.json", persistence_dir);
    
    match context_manager.persist_to_file(&path) {
        Ok(_) => println!("Context saved successfully to {}", path),
        Err(e) => println!("Error saving context: {}", e),
    }
}

fn load_context(persistence_dir: &str) -> Result<ContextManager, std::io::Error> {
    let path = format!("{}/manual_save.json", persistence_dir);
    
    if Path::new(&path).exists() {
        match ContextManager::load_from_file(&path) {
            Ok(manager) => {
                println!("Context loaded successfully from {}", path);
                Ok(manager)
            },
            Err(e) => {
                println!("Error loading context: {}", e);
                Err(e)
            }
        }
    } else {
        println!("No saved context found at {}", path);
        Err(std::io::Error::new(std::io::ErrorKind::NotFound, "No saved context found"))
    }
} 