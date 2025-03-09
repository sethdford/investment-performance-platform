use std::io::{self, Write};
use std::path::Path;
use std::fs;
use std::collections::HashMap;
use colored::*;
use investment_management::conversation::context_manager::{ContextManager, ContextManagerConfig, ImportanceLevel, ConversationTopic, ConversationState};
use investment_management::conversation::Message;
use investment_management::financial_entities::{FinancialEntity, entity_extractor::FinancialEntityType};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    print_header();
    
    // Create a directory for persistence
    let persistence_dir = "context_demo";
    fs::create_dir_all(persistence_dir).unwrap_or_else(|e| {
        eprintln!("Warning: Could not create persistence directory: {}", e);
        eprintln!("Context will not be saved.");
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
        println!("\n{}", "Options:".cyan().bold());
        println!("  {}. Add a message", "1".green());
        println!("  {}. View active context", "2".green());
        println!("  {}. View relevant context", "3".green());
        println!("  {}. View topics", "4".green());
        println!("  {}. View segments", "5".green());
        println!("  {}. Save context", "6".green());
        println!("  {}. Load context", "7".green());
        println!("  {}. Add a topic manually", "8".green());
        println!("  {}. Change conversation state", "9".green());
        println!("  {}. Exit", "0".green());
        
        print!("\n{} ", "Enter your choice:".cyan());
        io::stdout().flush()?;
        
        let mut choice = String::new();
        io::stdin().read_line(&mut choice)?;
        let choice = choice.trim();
        
        match choice {
            "1" => add_message(&mut context_manager)?,
            "2" => view_active_context(&context_manager),
            "3" => view_relevant_context(&context_manager),
            "4" => view_topics(&context_manager),
            "5" => view_segments(&context_manager),
            "6" => save_context(&context_manager, persistence_dir)?,
            "7" => {
                context_manager = load_context(persistence_dir).unwrap_or_else(|e| {
                    eprintln!("Error loading context: {}", e);
                    println!("{}", "Could not load context. Using current context.".yellow());
                    context_manager
                });
            },
            "8" => add_topic_manually(&mut context_manager)?,
            "9" => change_state(&mut context_manager)?,
            "0" => {
                println!("{}", "Exiting demo. Thank you!".green());
                break;
            },
            _ => println!("{}", "Invalid choice. Please try again.".red()),
        }
    }
    
    Ok(())
}

fn print_header() {
    println!("{}", "=================================================".cyan());
    println!("{}", "  Enhanced Context Management Demo".cyan().bold());
    println!("{}", "=================================================".cyan());
    println!("This demo showcases the enhanced context management capabilities:");
    println!("  • {}", "Context window management".yellow());
    println!("  • {}", "Context relevance scoring".yellow());
    println!("  • {}", "Context persistence".yellow());
    println!("  • {}", "Topic and entity tracking".yellow());
    println!("{}", "=================================================".cyan());
}

fn simulate_initial_conversation(context_manager: &mut ContextManager) {
    println!("\n{}", "Simulating initial conversation...".cyan());
    
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
    
    // Add some topics manually to simulate topic detection
    let topics = [
        ("Retirement Planning", "Planning for retirement and retirement accounts", ImportanceLevel::High),
        ("401(k)", "Tax-advantaged retirement account", ImportanceLevel::Medium),
        ("Roth IRA", "Tax-free retirement account", ImportanceLevel::Medium),
        ("Income", "Current earnings and salary", ImportanceLevel::Medium),
        ("Employer Benefits", "Benefits provided by employer", ImportanceLevel::Low),
    ];
    
    // Create a map of topics for our simulation
    let mut topic_map = HashMap::new();
    for (i, (name, description, importance)) in topics.iter().enumerate() {
        let topic = create_sample_topic(name, description, importance, i);
        topic_map.insert(name.to_string(), topic);
    }
    
    // Add messages to the context manager
    for (i, message) in messages.iter().enumerate() {
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
        
        // Simulate updating topic mentions
        if let Some(topic) = topic_map.get_mut("Retirement Planning") {
            if message.contains("retire") || message.contains("retirement") {
                topic.update_mention(i, messages.len());
            }
        }
        
        if let Some(topic) = topic_map.get_mut("401(k)") {
            if message.contains("401(k)") {
                topic.update_mention(i, messages.len());
            }
        }
        
        if let Some(topic) = topic_map.get_mut("Roth IRA") {
            if message.contains("Roth IRA") {
                topic.update_mention(i, messages.len());
            }
        }
        
        if let Some(topic) = topic_map.get_mut("Income") {
            if message.contains("income") || message.contains("salary") {
                topic.update_mention(i, messages.len());
            }
        }
        
        if let Some(topic) = topic_map.get_mut("Employer Benefits") {
            if message.contains("employer") || message.contains("matches") {
                topic.update_mention(i, messages.len());
            }
        }
    }
    
    // Simulate adding entities to the context
    // In a real implementation, we would use the appropriate methods
    // Here we're just simulating for the demo
    println!("Adding financial entities to the conversation:");
    println!("  • {} (${:.2})", "401(k)".green(), 250000.0);
    println!("  • {} (${:.2})", "Roth IRA".green(), 50000.0);
    println!("  • {} (${:.2})", "Annual Income".green(), 120000.0);
    println!("  • {} (${:.2})", "401(k) Contribution".green(), 12000.0);
    println!("  • {} (${:.2})", "Employer Match".green(), 6000.0);
    
    println!("{} {} messages.", 
        "Initial conversation simulated with".green(),
        messages.len() * 2);
}

fn create_sample_topic(name: &str, description: &str, importance: &ImportanceLevel, i: usize) -> ConversationTopic {
    let topic = ConversationTopic::new(name, description, *importance, i);
    topic
}

fn add_message(context_manager: &mut ContextManager) -> io::Result<()> {
    print!("{} ", "Enter your message:".cyan());
    io::stdout().flush()?;
    
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    let input = input.trim();
    
    if !input.is_empty() {
        // Create and add user message
        let user_message = Message::from_user(input);
        context_manager.add_message(user_message);
        
        // Simulate topic and entity detection
        detect_topics_and_entities(input);
        
        // Simulate assistant response
        let response = generate_response(input);
        let assistant_message = Message::from_assistant(&response);
        context_manager.add_message(assistant_message);
        
        println!("{} {}", "Assistant:".green().bold(), response);
        println!("{}", "Message added to context.".green());
    } else {
        println!("{}", "Empty message not added.".yellow());
    }
    
    Ok(())
}

fn generate_response(input: &str) -> String {
    // Simple response generation based on input keywords
    if input.to_lowercase().contains("retire") || input.to_lowercase().contains("retirement") {
        "Based on your retirement goals, I recommend reviewing your current asset allocation and considering increasing your contributions if possible.".to_string()
    } else if input.to_lowercase().contains("401k") || input.to_lowercase().contains("401(k)") {
        "Your 401(k) is a valuable retirement vehicle. Make sure you're maximizing your employer match and consider your investment options carefully.".to_string()
    } else if input.to_lowercase().contains("ira") || input.to_lowercase().contains("roth") {
        "Roth IRAs offer tax-free growth and withdrawals in retirement, which can be a great complement to your pre-tax 401(k) savings.".to_string()
    } else if input.to_lowercase().contains("invest") || input.to_lowercase().contains("stock") || 
             input.to_lowercase().contains("bond") || input.to_lowercase().contains("fund") {
        "A diversified investment strategy is important. Consider a mix of stocks, bonds, and other assets based on your risk tolerance and time horizon.".to_string()
    } else if input.to_lowercase().contains("tax") || input.to_lowercase().contains("deduction") {
        "Tax planning is an important part of financial planning. There are several strategies we can discuss to optimize your tax situation.".to_string()
    } else {
        "Thank you for sharing that information. I've updated your financial profile and will take this into account for future recommendations.".to_string()
    }
}

fn detect_topics_and_entities(input: &str) {
    // This is a simplified simulation of topic and entity detection
    
    // Check for retirement-related topics
    if input.to_lowercase().contains("retire") || input.to_lowercase().contains("retirement") {
        println!("{}", "Detected retirement planning topic.".blue());
        // In a real implementation, we would add this topic to the context manager
        // For the demo, we'll just print that we detected it
    }
    
    // Check for investment-related topics
    if input.to_lowercase().contains("invest") || input.to_lowercase().contains("stock") || 
       input.to_lowercase().contains("bond") || input.to_lowercase().contains("fund") {
        println!("{}", "Detected investment topic.".blue());
        // In a real implementation, we would add this topic to the context manager
        // For the demo, we'll just print that we detected it
    }
    
    // Check for tax-related topics
    if input.to_lowercase().contains("tax") || input.to_lowercase().contains("deduction") {
        println!("{}", "Detected tax planning topic.".blue());
        // In a real implementation, we would add this topic to the context manager
        // For the demo, we'll just print that we detected it
    }
    
    // Check for account-related entities
    if input.to_lowercase().contains("401k") || input.to_lowercase().contains("401(k)") {
        println!("{}", "Detected 401(k) account entity.".blue());
        
        // Extract dollar amount if present
        let dollar_amount = extract_dollar_amount(input);
        
        if let Some(amount) = dollar_amount {
            println!("  {} ${:.2}", "Value:".blue(), amount);
            // In a real implementation, we would add this entity to the context manager
            // For the demo, we'll just print that we detected it
        }
    }
    
    if input.to_lowercase().contains("ira") || input.to_lowercase().contains("roth") {
        println!("{}", "Detected IRA account entity.".blue());
        
        // Extract dollar amount if present
        let dollar_amount = extract_dollar_amount(input);
        
        if let Some(amount) = dollar_amount {
            println!("  {} ${:.2}", "Value:".blue(), amount);
            // In a real implementation, we would add this entity to the context manager
            // For the demo, we'll just print that we detected it
        }
    }
}

fn extract_dollar_amount(input: &str) -> Option<f64> {
    let dollar_regex = regex::Regex::new(r"\$(\d{1,3}(,\d{3})*|\d+)(\.\d+)?").unwrap();
    if let Some(captures) = dollar_regex.captures(input) {
        if let Some(amount_match) = captures.get(0) {
            let amount_str = amount_match.as_str().replace(['$', ','], "");
            if let Ok(amount) = amount_str.parse::<f64>() {
                return Some(amount);
            }
        }
    }
    None
}

fn view_active_context(context_manager: &ContextManager) {
    let active_context = context_manager.get_active_context();
    
    println!("\n{}", format!("=== Active Context ({} messages) ===", active_context.len()).cyan().bold());
    
    if active_context.is_empty() {
        println!("{}", "No messages in active context.".yellow());
    } else {
        for (i, message) in active_context.iter().enumerate() {
            let role = if message.is_user() {
                "User".blue().bold()
            } else if message.is_assistant() {
                "Assistant".green().bold()
            } else {
                "System".yellow().bold()
            };
            
            println!("{}. [{}]: {}", i + 1, role, message.content);
        }
    }
}

fn view_relevant_context(context_manager: &ContextManager) {
    let relevant_context = context_manager.get_relevant_context();
    
    println!("\n{}", format!("=== Relevant Context ({} messages) ===", relevant_context.len()).cyan().bold());
    
    if relevant_context.is_empty() {
        println!("{}", "No messages in relevant context.".yellow());
    } else {
        for (i, message) in relevant_context.iter().enumerate() {
            let role = if message.is_user() {
                "User".blue().bold()
            } else if message.is_assistant() {
                "Assistant".green().bold()
            } else {
                "System".yellow().bold()
            };
            
            println!("{}. [{}]: {}", i + 1, role, message.content);
        }
    }
    
    println!("\n{}", "Note: Relevant context is a subset of the active context, prioritized by:".cyan());
    println!("  • {}", "Recency: More recent messages are prioritized".yellow());
    println!("  • {}", "Relevance: Messages related to current topics are prioritized".yellow());
    println!("  • {}", "Importance: Messages with important information are prioritized".yellow());
}

fn view_topics(context_manager: &ContextManager) {
    let topics = context_manager.get_current_topics();
    
    println!("\n{}", format!("=== Current Topics ({} topics) ===", topics.len()).cyan().bold());
    
    if topics.is_empty() {
        println!("{}", "No topics detected.".yellow());
    } else {
        for (i, topic) in topics.iter().enumerate() {
            let importance_color = match topic.importance {
                ImportanceLevel::Low => "Low".normal(),
                ImportanceLevel::Medium => "Medium".yellow(),
                ImportanceLevel::High => "High".bright_yellow(),
                ImportanceLevel::Critical => "Critical".red(),
            };
            
            println!("{}. {} (Importance: {}, Score: {:.2})", 
                i + 1, 
                topic.name.green(), 
                importance_color,
                topic.calculate_overall_score());
                
            println!("   Description: {}", topic.description);
            println!("   First mentioned: message #{}, Last mentioned: message #{}", 
                topic.first_mentioned, 
                topic.last_mentioned);
            
            if !topic.related_entities.is_empty() {
                println!("   Related entities: {}", topic.related_entities.iter()
                    .map(|e| e.name.clone())
                    .collect::<Vec<_>>()
                    .join(", "));
            }
            
            println!();
        }
    }
}

fn view_segments(context_manager: &ContextManager) {
    // This is a simplified view since we don't have direct access to segments
    
    println!("\n{}", "=== Context Segments ===".cyan().bold());
    println!("{}", "Context is organized into segments based on topic shifts and importance.".yellow());
    println!("{}", "Segments are used to prioritize relevant information when generating responses.".yellow());
    println!("\n{}", "The most relevant segments are included in the context window based on:".cyan());
    println!("  • {}", "Recency: How recently the segment was created".yellow());
    println!("  • {}", "Relevance: How relevant the segment is to the current conversation".yellow());
    println!("  • {}", "Importance: How important the information in the segment is".yellow());
    
    println!("\n{}", "Segment Structure:".cyan().bold());
    println!("  • {}", "Messages: A group of related messages".yellow());
    println!("  • {}", "Topics: Topics discussed in the segment".yellow());
    println!("  • {}", "Entities: Financial entities mentioned in the segment".yellow());
    println!("  • {}", "Importance Score: How important the segment is".yellow());
    println!("  • {}", "Recency Score: How recently the segment was created".yellow());
    println!("  • {}", "Relevance Score: How relevant the segment is to current context".yellow());
    println!("  • {}", "Token Count: Approximate number of tokens in the segment".yellow());
}

fn save_context(context_manager: &ContextManager, persistence_dir: &str) -> io::Result<()> {
    let path = format!("{}/manual_save.json", persistence_dir);
    
    match context_manager.persist_to_file(&path) {
        Ok(_) => {
            println!("{} {}", "Context saved successfully to".green(), path);
            Ok(())
        },
        Err(e) => {
            eprintln!("Error saving context: {}", e);
            Err(e)
        }
    }
}

fn load_context(persistence_dir: &str) -> Result<ContextManager, std::io::Error> {
    let path = format!("{}/manual_save.json", persistence_dir);
    
    if Path::new(&path).exists() {
        match ContextManager::load_from_file(&path) {
            Ok(manager) => {
                println!("{} {}", "Context loaded successfully from".green(), path);
                Ok(manager)
            },
            Err(e) => {
                eprintln!("Error loading context: {}", e);
                Err(e)
            }
        }
    } else {
        println!("{} {}", "No saved context found at".yellow(), path);
        Err(std::io::Error::new(std::io::ErrorKind::NotFound, "No saved context found"))
    }
}

fn add_topic_manually(context_manager: &mut ContextManager) -> io::Result<()> {
    println!("\n{}", "=== Add Topic Manually ===".cyan().bold());
    
    print!("{} ", "Enter topic name:".cyan());
    io::stdout().flush()?;
    let mut name = String::new();
    io::stdin().read_line(&mut name)?;
    let name = name.trim();
    
    if name.is_empty() {
        println!("{}", "Topic name cannot be empty.".red());
        return Ok(());
    }
    
    print!("{} ", "Enter topic description:".cyan());
    io::stdout().flush()?;
    let mut description = String::new();
    io::stdin().read_line(&mut description)?;
    let description = description.trim();
    
    println!("{}", "Select importance level:".cyan());
    println!("  1. Low");
    println!("  2. Medium");
    println!("  3. High");
    println!("  4. Critical");
    
    print!("{} ", "Enter choice (1-4):".cyan());
    io::stdout().flush()?;
    let mut importance_choice = String::new();
    io::stdin().read_line(&mut importance_choice)?;
    let importance_choice = importance_choice.trim();
    
    let importance = match importance_choice {
        "1" => ImportanceLevel::Low,
        "2" => ImportanceLevel::Medium,
        "3" => ImportanceLevel::High,
        "4" => ImportanceLevel::Critical,
        _ => {
            println!("{}", "Invalid choice. Using Medium importance.".yellow());
            ImportanceLevel::Medium
        }
    };
    
    // In a real implementation, we would add this topic to the context manager
    // For the demo, we'll just print that we added it
    println!("{} {} (Importance: {:?})", "Topic added:".green(), name, importance);
    
    Ok(())
}

fn read_line_trim() -> io::Result<String> {
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    Ok(input.trim().to_string())
}

fn change_state(context_manager: &mut ContextManager) -> io::Result<()> {
    println!("\n{}", "Change conversation state".blue().bold());
    println!("Current state: {:?}", context_manager.get_state());
    
    println!("\nSelect new state:");
    println!("1. Introduction");
    println!("2. InformationGathering");
    println!("3. TopicDiscussion");
    println!("4. AdvisingPhase");
    println!("5. QuestionAnswering");
    println!("6. Summarization");
    println!("7. Closing");
    
    let choice = read_line_trim()?;
    let state_choice = choice.parse::<usize>().unwrap_or(0);
    
    let new_state = match state_choice {
        1 => ConversationState::Introduction,
        2 => ConversationState::InformationGathering,
        3 => ConversationState::TopicDiscussion,
        4 => ConversationState::AdvisingPhase,
        5 => ConversationState::QuestionAnswering,
        6 => ConversationState::Summarization,
        7 => ConversationState::Closing,
        _ => {
            println!("Invalid choice");
            return Ok(());
        }
    };
    
    let state_for_display = new_state.clone();
    context_manager.set_state(new_state);
    
    println!("{} {:?}", "Conversation state changed to:".green(), state_for_display);
    Ok(())
} 