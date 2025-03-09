use std::io::{self, Write};
use std::path::PathBuf;
use std::fs::{self, File, OpenOptions};
use std::io::{BufReader, BufWriter};
use chrono::{DateTime, Utc};
use clap::{Parser, Subcommand};
use colored::*;
use investment_management::{ConversationalAdvisor, Message, Conversation};
use serde::{Serialize, Deserialize};
use anyhow::{Result, Context, anyhow};
use tracing::{info, warn, error, Level};
use tracing_subscriber;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// Optional conversation ID to load
    #[arg(short, long)]
    conversation_id: Option<String>,
    
    /// Path to save conversations
    #[arg(short, long, default_value = "conversations")]
    save_path: PathBuf,
    
    /// Use basic mode (no LLM)
    #[arg(short, long)]
    basic: bool,
    
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// List saved conversations
    List,
    
    /// Delete a conversation
    Delete {
        /// Conversation ID to delete
        id: String,
    },
}

#[derive(Serialize, Deserialize)]
struct SavedConversation {
    conversation: Conversation,
    last_updated: DateTime<Utc>,
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt::init();
    
    // Parse command line arguments
    let cli = Cli::parse();
    
    // Create save directory if it doesn't exist
    fs::create_dir_all(&cli.save_path)
        .context(format!("Failed to create save directory at {}", cli.save_path.display()))?;
    
    // Handle subcommands
    if let Some(cmd) = cli.command {
        match cmd {
            Commands::List => {
                list_conversations(&cli.save_path)?;
                return Ok(());
            }
            Commands::Delete { id } => {
                delete_conversation(&cli.save_path, &id)?;
                return Ok(());
            }
        }
    }
    
    // Initialize the advisor
    let mut advisor = if cli.basic {
        info!("Using basic mode (no LLM capabilities)");
        println!("{}", "Using basic mode (no LLM capabilities)".yellow());
        investment_management::create_conversational_advisor()
    } else {
        info!("Initializing AI components...");
        println!("{}", "Initializing AI components...".cyan());
        match ConversationalAdvisor::new_enhanced().await {
            Ok(enhanced_advisor) => {
                info!("Enhanced NLP capabilities initialized successfully!");
                println!("{}", "✅ Enhanced NLP capabilities initialized successfully!".green());
                println!("{}", "Using Claude 3 Sonnet for advanced conversational capabilities.".green());
                enhanced_advisor
            }
            Err(e) => {
                error!("Could not initialize enhanced NLP capabilities: {}", e);
                eprintln!("{}: {}", "⚠️ Could not initialize enhanced NLP capabilities".yellow(), e);
                println!("{}", "Falling back to basic conversation capabilities.".yellow());
                investment_management::create_conversational_advisor()
            }
        }
    };
    
    // Load or create conversation
    let mut conversation = if let Some(id) = &cli.conversation_id {
        // Set the conversation ID for the advisor
        advisor.set_conversation_id(id);
        
        // Try to load the conversation context
        let context_path = format!("{}/context_{}.json", cli.save_path.display(), id);
        if std::path::Path::new(&context_path).exists() {
            match advisor.load_context(&context_path) {
                Ok(_) => {
                    info!("Loaded conversation context: {}", id);
                    println!("{}: {}", "Loaded conversation context".green(), id);
                }
                Err(e) => {
                    warn!("Could not load conversation context: {}", e);
                    println!("{}: {}", "Could not load conversation context".yellow(), e);
                }
            }
        }
        
        // Load the conversation history
        load_conversation(&cli.save_path, id)
            .unwrap_or_else(|e| {
                warn!("Conversation not found: {} - {}", id, e);
                println!("{}: {} - {}", "Conversation not found".yellow(), id, e);
                println!("{}", "Creating new conversation".green());
                let mut new_conversation = Conversation::new("user");
                new_conversation.id = id.clone();
                new_conversation
            })
    } else {
        // Generate a new conversation ID
        let new_id = uuid::Uuid::new_v4().to_string();
        info!("Creating new conversation with ID: {}", new_id);
        advisor.set_conversation_id(&new_id);
        
        let mut conversation = Conversation::new("user");
        conversation.id = new_id;
        conversation
    };
    
    // Print welcome message
    print_welcome_message(&conversation);
    
    // Add system messages to conversation if it's new
    if conversation.messages.is_empty() {
        let system_message = Message::from_system(
            "I am a modern financial advisor that can help with financial planning, \
            investment strategies, retirement planning, and other financial topics. \
            I'll provide personalized guidance based on your financial situation."
        );
        conversation.add_message(system_message);
    }
    
    // Print conversation history
    if !conversation.messages.is_empty() {
        println!("{}", "\n--- Conversation History ---".cyan());
        for message in &conversation.messages {
            if message.is_system() {
                continue;
            }
            
            if message.is_user() {
                println!("{} {}", "You >".blue().bold(), message.content);
            } else {
                println!("{} {}", "Advisor >".green().bold(), message.content);
            }
        }
        println!("{}", "--- End of History ---\n".cyan());
    }
    
    // Main conversation loop
    let use_enhanced = !cli.basic;
    loop {
        print!("{} ", "You >".blue().bold());
        io::stdout().flush()?;
        
        let mut input = String::new();
        io::stdin().read_line(&mut input)
            .context("Failed to read user input")?;
        
        let input = input.trim();
        
        if input.is_empty() {
            continue;
        }
        
        if input.to_lowercase() == "exit" || input.to_lowercase() == "quit" {
            break;
        }
        
        if input.to_lowercase() == "help" {
            print_help();
            continue;
        }
        
        if input.to_lowercase() == "save" {
            // Save conversation history
            match save_conversation(&cli.save_path, &conversation) {
                Ok(_) => println!("{} {}", "Conversation saved with ID:".green(), conversation.id),
                Err(e) => println!("{}: {}", "Error saving conversation".red(), e),
            }
            
            // Save context
            let context_path = format!("{}/context_{}.json", cli.save_path.display(), conversation.id);
            match advisor.persist_context(&context_path) {
                Ok(_) => println!("{} {}", "Context saved with ID:".green(), conversation.id),
                Err(e) => println!("{}: {}", "Error saving context".red(), e),
            }
            
            continue;
        }
        
        if input.to_lowercase() == "topics" {
            display_topics(&advisor);
            continue;
        }
        
        if input.to_lowercase() == "entities" {
            display_entities(&advisor);
            continue;
        }
        
        // Create user message
        let user_message = Message::from_user(input);
        conversation.add_message(user_message.clone());
        
        // Process the message
        let response = if use_enhanced {
            match advisor.process_message_enhanced(&user_message.content).await {
                Ok(response) => response,
                Err(e) => {
                    error!("Error processing message: {}", e);
                    eprintln!("{}: {}", "Error processing message".red(), e);
                    "I'm sorry, I encountered an error processing your request. Please try again or rephrase your question.".to_string()
                }
            }
        } else {
            advisor.process_message(&user_message.content)
        };
        
        // Create assistant message and add to conversation
        let assistant_message = Message::from_assistant(&response);
        conversation.add_message(assistant_message);
        
        // Display the response
        println!("{} {}", "Advisor >".green().bold(), response);
        
        // Auto-save conversation
        if let Err(e) = save_conversation(&cli.save_path, &conversation) {
            warn!("Failed to auto-save conversation: {}", e);
        }
        
        // Auto-save context
        let context_path = format!("{}/context_{}.json", cli.save_path.display(), conversation.id);
        if let Err(e) = advisor.persist_context(&context_path) {
            warn!("Failed to auto-save context: {}", e);
        }
    }
    
    // Save conversation on exit
    match save_conversation(&cli.save_path, &conversation) {
        Ok(_) => {
            info!("Conversation saved with ID: {}", conversation.id);
            println!("{} {}", "Conversation saved with ID:".green(), conversation.id);
        },
        Err(e) => {
            error!("Failed to save conversation: {}", e);
            eprintln!("{}: {}", "Error saving conversation".red(), e);
        }
    }
    
    // Save context on exit
    let context_path = format!("{}/context_{}.json", cli.save_path.display(), conversation.id);
    match advisor.persist_context(&context_path) {
        Ok(_) => {
            info!("Context saved with ID: {}", conversation.id);
            println!("{} {}", "Context saved with ID:".green(), conversation.id);
        },
        Err(e) => {
            error!("Failed to save context: {}", e);
            eprintln!("{}: {}", "Error saving context".red(), e);
        }
    }
    
    println!("{}", "Thank you for using the Modern Conversational Financial Advisor!".cyan());
    
    Ok(())
}

fn print_welcome_message(conversation: &Conversation) {
    println!("{}", "=================================================".cyan());
    println!("{}", "  Modern Conversational Financial Advisor".cyan().bold());
    println!("{}", "=================================================".cyan());
    println!("Conversation ID: {}", conversation.id.green());
    println!("Type {} for commands, {} to exit", "help".yellow(), "exit".yellow());
    println!("{}", "=================================================".cyan());
}

fn print_help() {
    println!("{}", "\nAvailable commands:".cyan());
    println!("  {} - Exit the application", "exit".yellow());
    println!("  {} - Save the current conversation", "save".yellow());
    println!("  {} - Display detected topics", "topics".yellow());
    println!("  {} - Display detected entities", "entities".yellow());
    println!("  {} - Display this help message", "help".yellow());
    println!();
}

fn display_topics(advisor: &ConversationalAdvisor) {
    let topics = advisor.get_detected_topics();
    
    if topics.is_empty() {
        println!("{}", "No topics detected in the current conversation.".yellow());
        return;
    }
    
    println!("{}", "\n📊 Detected Topics:".cyan());
    for (i, topic) in topics.iter().enumerate() {
        println!("  {}. {} (Importance: {:?}, Score: {:.2})", 
            i + 1,
            topic.name.yellow(), 
            topic.importance,
            topic.calculate_overall_score());
    }
    println!();
}

fn display_entities(advisor: &ConversationalAdvisor) {
    let entities = advisor.get_detected_entities();
    
    if entities.is_empty() {
        println!("{}", "No entities detected in the current conversation.".yellow());
        return;
    }
    
    println!("{}", "\n🔍 Detected Entities:".cyan());
    for (i, entity) in entities.iter().enumerate() {
        if let Some(value) = entity.value {
            println!("  {}. {} ({}): {}", 
                i + 1,
                entity.name.yellow(), 
                format!("{:?}", entity.entity_type).blue(),
                format!("${:.2}", value).green());
        } else {
            println!("  {}. {} ({})", 
                i + 1,
                entity.name.yellow(), 
                format!("{:?}", entity.entity_type).blue());
        }
    }
    println!();
}

fn list_conversations(save_path: &PathBuf) -> Result<()> {
    let entries = fs::read_dir(save_path)
        .context(format!("Failed to read directory: {}", save_path.display()))?;
    
    let mut found = false;
    println!("{}", "Saved conversations:".cyan());
    
    for entry in entries {
        let entry = entry.context("Failed to read directory entry")?;
        let path = entry.path();
        
        if path.is_file() && path.extension().map_or(false, |ext| ext == "json") {
            if let Some(filename) = path.file_name() {
                let filename_str = filename.to_string_lossy();
                if filename_str.starts_with("conversation_") && !filename_str.starts_with("context_") {
                    found = true;
                    
                    // Extract ID from filename
                    let id = filename_str
                        .strip_prefix("conversation_")
                        .and_then(|s| s.strip_suffix(".json"))
                        .unwrap_or("unknown");
                    
                    // Try to load the conversation to get metadata
                    match load_conversation(save_path, id) {
                        Ok(conversation) => {
                            let message_count = conversation.messages.len();
                            let last_message = conversation.get_last_message()
                                .map(|m| m.timestamp.format("%Y-%m-%d %H:%M:%S").to_string())
                                .unwrap_or_else(|| "Unknown".to_string());
                            
                            println!("  ID: {}, Messages: {}, Last updated: {}", 
                                id.green(), 
                                message_count, 
                                last_message);
                        },
                        Err(e) => {
                            println!("  ID: {} (Error loading conversation: {})", id.yellow(), e);
                        }
                    }
                }
            }
        }
    }
    
    if !found {
        println!("  No saved conversations found.");
    }
    
    Ok(())
}

fn save_conversation(save_path: &PathBuf, conversation: &Conversation) -> Result<()> {
    let saved = SavedConversation {
        conversation: conversation.clone(),
        last_updated: Utc::now(),
    };
    
    let path = save_path.join(format!("conversation_{}.json", conversation.id));
    let file = File::create(&path)
        .context(format!("Failed to create file: {}", path.display()))?;
    let writer = BufWriter::new(file);
    
    serde_json::to_writer_pretty(writer, &saved)
        .context(format!("Failed to write conversation to file: {}", path.display()))?;
    
    Ok(())
}

fn load_conversation(save_path: &PathBuf, id: &str) -> Result<Conversation> {
    let path = save_path.join(format!("conversation_{}.json", id));
    let file = File::open(&path)
        .context(format!("Failed to open conversation file: {}", path.display()))?;
    let reader = BufReader::new(file);
    
    let saved: SavedConversation = serde_json::from_reader(reader)
        .context(format!("Failed to parse conversation file: {}", path.display()))?;
    
    Ok(saved.conversation)
}

fn delete_conversation(save_path: &PathBuf, id: &str) -> Result<()> {
    let conversation_path = save_path.join(format!("conversation_{}.json", id));
    let context_path = save_path.join(format!("context_{}.json", id));
    
    let mut deleted = false;
    
    if conversation_path.exists() {
        fs::remove_file(&conversation_path)
            .context(format!("Failed to delete conversation file: {}", conversation_path.display()))?;
        deleted = true;
        println!("Deleted conversation file: {}", conversation_path.display());
    }
    
    if context_path.exists() {
        fs::remove_file(&context_path)
            .context(format!("Failed to delete context file: {}", context_path.display()))?;
        deleted = true;
        println!("Deleted context file: {}", context_path.display());
    }
    
    if deleted {
        println!("{} {}", "Successfully deleted conversation:".green(), id);
    } else {
        println!("{} {}", "No files found for conversation:".yellow(), id);
    }
    
    Ok(())
} 