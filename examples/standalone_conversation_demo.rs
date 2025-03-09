use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};
use uuid::Uuid;

#[derive(Debug, Clone)]
enum MessageRole {
    User,
    Assistant,
    System,
}

impl MessageRole {
    fn as_str(&self) -> &'static str {
        match self {
            MessageRole::User => "user",
            MessageRole::Assistant => "assistant",
            MessageRole::System => "system",
        }
    }
}

#[derive(Debug, Clone)]
struct Message {
    id: String,
    conversation_id: String,
    timestamp: SystemTime,
    role: MessageRole,
    content: String,
    metadata: Option<HashMap<String, String>>,
}

impl Message {
    fn new(conversation_id: &str, role: MessageRole, content: String) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            conversation_id: conversation_id.to_string(),
            timestamp: SystemTime::now(),
            role,
            content,
            metadata: None,
        }
    }
}

#[derive(Debug, Clone)]
struct Conversation {
    id: String,
    user_id: String,
    messages: Vec<Message>,
    created_at: SystemTime,
    updated_at: SystemTime,
}

impl Conversation {
    fn new(user_id: &str) -> Self {
        let now = SystemTime::now();
        Self {
            id: Uuid::new_v4().to_string(),
            user_id: user_id.to_string(),
            messages: Vec::new(),
            created_at: now,
            updated_at: now,
        }
    }

    fn add_message(&mut self, role: MessageRole, content: String) -> &Message {
        let message = Message::new(&self.id, role, content);
        self.messages.push(message);
        self.updated_at = SystemTime::now();
        self.messages.last().unwrap()
    }
}

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

    fn get_conversation_mut(&mut self, conversation_id: &str) -> Option<&mut Conversation> {
        self.conversations.get_mut(conversation_id)
    }

    fn delete_conversation(&mut self, conversation_id: &str) -> bool {
        self.conversations.remove(conversation_id).is_some()
    }

    fn list_conversations(&self, user_id: &str) -> Vec<&Conversation> {
        self.conversations
            .values()
            .filter(|conv| conv.user_id == user_id)
            .collect()
    }

    fn search_conversations(&self, query: &str) -> Vec<&Conversation> {
        self.conversations
            .values()
            .filter(|conv| {
                conv.messages.iter().any(|msg| msg.content.contains(query))
            })
            .collect()
    }
}

fn format_time(time: SystemTime) -> String {
    let duration = time.duration_since(UNIX_EPOCH).unwrap_or_default();
    let secs = duration.as_secs();
    
    let (hours, remainder) = (secs / 3600, secs % 3600);
    let (minutes, seconds) = (remainder / 60, remainder % 60);
    
    format!("{:02}:{:02}:{:02}", hours % 24, minutes, seconds)
}

fn main() {
    println!("=== Standalone Conversation Storage Demo ===\n");
    
    // Create a new storage instance
    let mut storage = ConversationStorage::new();
    
    // Create a new conversation
    let user_id = "user123";
    let mut conversation = Conversation::new(user_id);
    let conversation_id = conversation.id.clone();
    
    println!("Created conversation with ID: {}", conversation_id);
    
    // Add messages to the conversation
    conversation.add_message(
        MessageRole::User,
        "I'm planning for retirement. Can you help me?".to_string(),
    );
    
    conversation.add_message(
        MessageRole::Assistant,
        "I'd be happy to help with your retirement planning. What's your current age and when do you plan to retire?".to_string(),
    );
    
    conversation.add_message(
        MessageRole::User,
        "I'm 35 and I want to retire at 65.".to_string(),
    );
    
    // Save the conversation
    storage.save_conversation(conversation);
    println!("Saved conversation with 3 messages\n");
    
    // Retrieve the conversation
    if let Some(conv) = storage.get_conversation(&conversation_id) {
        println!("Retrieved conversation:");
        println!("  ID: {}", conv.id);
        println!("  User ID: {}", conv.user_id);
        println!("  Created at: {}", format_time(conv.created_at));
        println!("  Updated at: {}", format_time(conv.updated_at));
        println!("  Messages:");
        
        for (i, msg) in conv.messages.iter().enumerate() {
            println!("    {}. [{}] {}: {}", 
                i + 1,
                format_time(msg.timestamp),
                msg.role.as_str(),
                msg.content
            );
        }
        println!();
    }
    
    // Add another message to the conversation
    if let Some(conv) = storage.get_conversation_mut(&conversation_id) {
        conv.add_message(
            MessageRole::Assistant,
            "Great! With 30 years until retirement, you have a good amount of time to build your nest egg. Let's discuss your current savings and investment strategy.".to_string(),
        );
        println!("Added a new message to the conversation\n");
    }
    
    // List all conversations for the user
    let user_conversations = storage.list_conversations(user_id);
    println!("Found {} conversation(s) for user {}", user_conversations.len(), user_id);
    
    // Search for conversations containing a specific query
    let search_query = "retirement";
    let search_results = storage.search_conversations(search_query);
    println!("Found {} conversation(s) containing '{}'", search_results.len(), search_query);
    
    // Delete the conversation
    if storage.delete_conversation(&conversation_id) {
        println!("Successfully deleted conversation {}", conversation_id);
    }
    
    // Verify deletion
    if storage.get_conversation(&conversation_id).is_none() {
        println!("Conversation no longer exists in storage");
    }
} 