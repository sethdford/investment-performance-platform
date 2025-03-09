use anyhow::Result;
use chrono::Utc;
use uuid::Uuid;
use std::collections::HashMap;

/// A simple conversation message
#[derive(Debug, Clone)]
struct Message {
    id: String,
    timestamp: chrono::DateTime<Utc>,
    role: String,
    content: String,
}

impl Message {
    fn new(role: &str, content: &str) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            timestamp: Utc::now(),
            role: role.to_string(),
            content: content.to_string(),
        }
    }
}

/// A simple conversation
#[derive(Debug, Clone)]
struct Conversation {
    id: String,
    user_id: String,
    messages: Vec<Message>,
    created_at: chrono::DateTime<Utc>,
    updated_at: chrono::DateTime<Utc>,
}

impl Conversation {
    fn new(user_id: &str) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            user_id: user_id.to_string(),
            messages: Vec::new(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }
    
    fn add_message(&mut self, role: &str, content: &str) -> &Message {
        let message = Message::new(role, content);
        self.messages.push(message);
        self.updated_at = Utc::now();
        self.messages.last().unwrap()
    }
}

/// A simple in-memory conversation storage
struct ConversationStorage {
    conversations: HashMap<String, Conversation>,
}

impl ConversationStorage {
    fn new() -> Self {
        Self {
            conversations: HashMap::new(),
        }
    }
    
    fn save_conversation(&mut self, conversation: Conversation) {
        self.conversations.insert(conversation.id.clone(), conversation);
    }
    
    fn get_conversation(&self, conversation_id: &str) -> Option<&Conversation> {
        self.conversations.get(conversation_id)
    }
    
    fn delete_conversation(&mut self, conversation_id: &str) -> bool {
        self.conversations.remove(conversation_id).is_some()
    }
    
    fn list_conversations(&self, user_id: &str) -> Vec<&Conversation> {
        self.conversations.values()
            .filter(|c| c.user_id == user_id)
            .collect()
    }
    
    fn search_conversations(&self, query: &str) -> Vec<&Conversation> {
        self.conversations.values()
            .filter(|c| c.messages.iter().any(|m| m.content.contains(query)))
            .collect()
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    println!("Running simple conversation demo...");
    
    // Create a new conversation storage
    let mut storage = ConversationStorage::new();
    
    // Create a new conversation
    let user_id = "test_user";
    let mut conversation = Conversation::new(user_id);
    println!("Created conversation with ID: {}", conversation.id);
    
    // Add some messages to the conversation
    conversation.add_message("user", "Hello, I need help with retirement planning.");
    conversation.add_message("assistant", "I'd be happy to help with your retirement planning. What's your current age and when would you like to retire?");
    conversation.add_message("user", "I'm 35 and I'd like to retire at 65.");
    
    // Save the conversation
    storage.save_conversation(conversation.clone());
    
    // Get the conversation
    let conversation_id = conversation.id.clone();
    if let Some(loaded_conversation) = storage.get_conversation(&conversation_id) {
        println!("Loaded conversation with {} messages", loaded_conversation.messages.len());
        
        // Print the conversation
        println!("\nConversation history:");
        for message in &loaded_conversation.messages {
            println!("{}: {}", message.role, message.content);
        }
    }
    
    // List all conversations for the user
    let conversations = storage.list_conversations(user_id);
    println!("\nFound {} conversations for user {}", conversations.len(), user_id);
    
    // Search for conversations containing "retirement"
    let search_results = storage.search_conversations("retirement");
    println!("\nFound {} conversations containing 'retirement'", search_results.len());
    
    // Delete the conversation
    let deleted = storage.delete_conversation(&conversation_id);
    println!("\nDeleted conversation with ID {}: {}", conversation_id, deleted);
    
    // Try to get the deleted conversation
    match storage.get_conversation(&conversation_id) {
        Some(_) => println!("Conversation still exists (unexpected)"),
        None => println!("Conversation was successfully deleted"),
    }
    
    println!("\nSimple conversation demo completed successfully!");
    Ok(())
} 