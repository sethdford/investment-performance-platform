use std::collections::HashMap;
use serde::{Serialize, Deserialize};
use chrono::{DateTime, Utc};

pub mod context_manager;
pub mod topic_detector;

/// Represents a message in a conversation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    /// Unique identifier for the message
    pub id: String,
    /// Role of the message sender (e.g., "user", "assistant", "system")
    pub role: String,
    /// Content of the message
    pub content: String,
    /// Timestamp when the message was created
    pub timestamp: DateTime<Utc>,
    /// Additional metadata about the message
    pub metadata: HashMap<String, String>,
}

impl Message {
    /// Creates a new message from a user
    pub fn from_user(content: &str) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            role: "user".to_string(),
            content: content.to_string(),
            timestamp: Utc::now(),
            metadata: HashMap::new(),
        }
    }
    
    /// Creates a new message from the assistant
    pub fn from_assistant(content: &str) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            role: "assistant".to_string(),
            content: content.to_string(),
            timestamp: Utc::now(),
            metadata: HashMap::new(),
        }
    }
    
    /// Creates a new system message
    pub fn from_system(content: &str) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            role: "system".to_string(),
            content: content.to_string(),
            timestamp: Utc::now(),
            metadata: HashMap::new(),
        }
    }
    
    /// Adds metadata to the message
    pub fn add_metadata(&mut self, key: &str, value: &str) {
        self.metadata.insert(key.to_string(), value.to_string());
    }
    
    /// Gets the message's age in seconds
    pub fn age_seconds(&self) -> i64 {
        Utc::now().timestamp() - self.timestamp.timestamp()
    }
    
    /// Checks if the message is from a user
    pub fn is_user(&self) -> bool {
        self.role == "user"
    }
    
    /// Checks if the message is from the assistant
    pub fn is_assistant(&self) -> bool {
        self.role == "assistant"
    }
    
    /// Checks if the message is a system message
    pub fn is_system(&self) -> bool {
        self.role == "system"
    }
}

/// Represents a conversation between a user and the financial advisor
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Conversation {
    /// Unique identifier for the conversation
    pub id: String,
    /// User identifier
    pub user_id: String,
    /// Messages in the conversation
    pub messages: Vec<Message>,
    /// Timestamp when the conversation was created
    pub created_at: DateTime<Utc>,
    /// Timestamp when the conversation was last updated
    pub updated_at: DateTime<Utc>,
    /// Additional metadata about the conversation
    pub metadata: HashMap<String, String>,
}

impl Conversation {
    /// Creates a new conversation for a user
    pub fn new(user_id: &str) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            user_id: user_id.to_string(),
            messages: Vec::new(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
            metadata: HashMap::new(),
        }
    }
    
    /// Adds a message to the conversation
    pub fn add_message(&mut self, message: Message) {
        self.messages.push(message);
        self.updated_at = Utc::now();
    }
    
    /// Gets the most recent message
    pub fn get_last_message(&self) -> Option<&Message> {
        self.messages.last()
    }
    
    /// Gets the number of messages in the conversation
    pub fn message_count(&self) -> usize {
        self.messages.len()
    }
    
    /// Gets the conversation duration in seconds
    pub fn duration_seconds(&self) -> i64 {
        if let Some(last_message) = self.get_last_message() {
            last_message.timestamp.timestamp() - self.created_at.timestamp()
        } else {
            0
        }
    }
    
    /// Adds metadata to the conversation
    pub fn add_metadata(&mut self, key: &str, value: &str) {
        self.metadata.insert(key.to_string(), value.to_string());
        self.updated_at = Utc::now();
    }
    
    /// Gets user messages only
    pub fn get_user_messages(&self) -> Vec<&Message> {
        self.messages.iter().filter(|m| m.is_user()).collect()
    }
    
    /// Gets assistant messages only
    pub fn get_assistant_messages(&self) -> Vec<&Message> {
        self.messages.iter().filter(|m| m.is_assistant()).collect()
    }
    
    /// Gets system messages only
    pub fn get_system_messages(&self) -> Vec<&Message> {
        self.messages.iter().filter(|m| m.is_system()).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread::sleep;
    use std::time::Duration;
    
    #[test]
    fn test_message_creation() {
        let user_msg = Message::from_user("Hello, I need financial advice");
        let assistant_msg = Message::from_assistant("I can help with that");
        let system_msg = Message::from_system("This is a system message");
        
        assert!(user_msg.is_user());
        assert!(assistant_msg.is_assistant());
        assert!(system_msg.is_system());
        
        assert_eq!(user_msg.content, "Hello, I need financial advice");
        assert_eq!(assistant_msg.content, "I can help with that");
        assert_eq!(system_msg.content, "This is a system message");
    }
    
    #[test]
    fn test_conversation_creation() {
        let conversation = Conversation::new("user123");
        assert_eq!(conversation.user_id, "user123");
        assert_eq!(conversation.messages.len(), 0);
        assert!(conversation.metadata.is_empty());
    }
    
    #[test]
    fn test_add_message() {
        let mut conversation = Conversation::new("user123");
        let message = Message::from_user("Hello");
        
        conversation.add_message(message);
        
        assert_eq!(conversation.message_count(), 1);
        assert!(conversation.get_last_message().unwrap().is_user());
        assert_eq!(conversation.get_last_message().unwrap().content, "Hello");
    }
    
    #[test]
    fn test_conversation_duration() {
        let mut conversation = Conversation::new("user123");
        
        // Add a message
        conversation.add_message(Message::from_user("Hello"));
        
        // Sleep to create a duration
        sleep(Duration::from_millis(10));
        
        // Add another message
        conversation.add_message(Message::from_assistant("Hi there"));
        
        // Duration should be positive
        assert!(conversation.duration_seconds() >= 0);
    }
    
    #[test]
    fn test_message_filtering() {
        let mut conversation = Conversation::new("user123");
        
        conversation.add_message(Message::from_user("Hello"));
        conversation.add_message(Message::from_assistant("Hi there"));
        conversation.add_message(Message::from_system("System message"));
        conversation.add_message(Message::from_user("How are you?"));
        
        assert_eq!(conversation.get_user_messages().len(), 2);
        assert_eq!(conversation.get_assistant_messages().len(), 1);
        assert_eq!(conversation.get_system_messages().len(), 1);
    }
} 