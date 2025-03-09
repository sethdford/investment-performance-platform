use anyhow::{Result, anyhow};
use chrono::{DateTime, Utc};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::info;
use uuid::Uuid;
use serde::{Serialize, Deserialize};
use std::path::Path;
use std::fs::{self, File};
use std::io::{BufReader, BufWriter};
use aws_sdk_dynamodb::Client;

// Import DynamoDbConversationStorage
use super::conversation_storage_dynamodb::DynamoDbConversationStorage;

use super::rule_based::FinancialQueryIntent;
use super::types::ClientData;

/// Trait defining the interface for conversation storage implementations
pub trait ConversationStorageTrait: Send + Sync {
    /// Save a conversation
    fn save_conversation<'a>(&'a self, conversation: &'a Conversation) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<()>> + Send + 'a>>;
    
    /// Load a conversation by ID
    fn load_conversation<'a>(&'a self, conversation_id: &'a str) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<Option<Conversation>>> + Send + 'a>>;
    
    /// Delete a conversation by ID
    fn delete_conversation<'a>(&'a self, conversation_id: &'a str) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<()>> + Send + 'a>>;
    
    /// List conversations for a user
    fn list_conversations<'a>(&'a self, user_id: &'a str, limit: Option<i32>) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<Vec<Conversation>>> + Send + 'a>>;
    
    /// Search conversations
    fn search_conversations<'a>(&'a self, query: &'a str) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<Vec<Conversation>>> + Send + 'a>>;
}

/// Represents a message in a conversation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversationMessage {
    /// Unique ID for the message
    pub id: String,
    
    /// Timestamp when the message was created
    pub timestamp: DateTime<Utc>,
    
    /// Role of the message sender (user or assistant)
    pub role: String,
    
    /// Content of the message
    pub content: String,
    
    /// Detected intent (if available)
    pub intent: Option<String>,
    
    /// Confidence score for the intent detection (0.0 to 1.0)
    pub intent_confidence: Option<f64>,
    
    /// Metadata associated with the message
    pub metadata: Option<HashMap<String, String>>,
}

/// Role of the message sender
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum MessageRole {
    /// Message from the user
    User,
    
    /// Message from the assistant
    Assistant,
    
    /// System message
    System,
}

impl std::fmt::Display for MessageRole {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MessageRole::User => write!(f, "user"),
            MessageRole::Assistant => write!(f, "assistant"),
            MessageRole::System => write!(f, "system"),
        }
    }
}

/// Represents a conversation session
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Conversation {
    /// Unique ID for the conversation
    pub id: String,
    
    /// User ID associated with the conversation
    pub user_id: String,
    
    /// Client data associated with the conversation (if available)
    pub client_data: Option<ClientData>,
    
    /// Messages in the conversation
    pub messages: Vec<ConversationMessage>,
    
    /// Timestamp when the conversation was created
    pub created_at: DateTime<Utc>,
    
    /// Timestamp when the conversation was last updated
    pub updated_at: DateTime<Utc>,
    
    /// Tags associated with the conversation
    pub tags: Vec<String>,
    
    /// Summary of the conversation (if available)
    pub summary: Option<String>,
    
    /// Metadata associated with the conversation
    pub metadata: HashMap<String, String>,
}

/// Storage type for conversations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StorageType {
    /// In-memory storage
    Memory,
    
    /// File-based storage
    File,
    
    /// Database storage
    Database,
    
    /// DynamoDB storage
    DynamoDB,
}

/// Conversation storage configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversationStorageConfig {
    /// Storage type
    pub storage_type: StorageType,
    
    /// Base directory for file storage
    pub base_directory: Option<String>,
    
    /// Database URL for database storage
    pub database_url: Option<String>,
    
    /// DynamoDB table names (conversations_table, messages_table)
    pub dynamodb_tables: Option<(String, String)>,
    
    /// Maximum number of conversations to keep in memory
    pub max_memory_conversations: usize,
    
    /// Whether to enable conversation summarization
    pub enable_summarization: bool,
}

impl Default for ConversationStorageConfig {
    fn default() -> Self {
        Self {
            storage_type: StorageType::Memory,
            base_directory: None,
            database_url: None,
            dynamodb_tables: None,
            max_memory_conversations: 1000,
            enable_summarization: true,
        }
    }
}

/// Main conversation storage implementation
pub struct ConversationStorage {
    /// Configuration
    config: ConversationStorageConfig,
    
    /// Memory cache
    memory_cache: RwLock<HashMap<String, Conversation>>,
    
    /// DynamoDB storage
    dynamodb_storage: Option<DynamoDbConversationStorage>,
}

impl ConversationStorage {
    /// Create a new conversation storage with the given configuration
    pub fn new(config: ConversationStorageConfig) -> Self {
        let dynamodb_storage = match &config.storage_type {
            StorageType::DynamoDB => {
                if let Some((conversations_table, messages_table)) = &config.dynamodb_tables {
                    // Create AWS DynamoDB client
                    info!("Creating DynamoDB client for conversation storage");
                    let future = async {
                        let aws_config = aws_config::defaults(aws_config::BehaviorVersion::v2024_03_28()).load().await;
                        let client = Client::new(&aws_config);
                        Some(DynamoDbConversationStorage::new(
                            Arc::new(client),
                            conversations_table.clone(),
                            messages_table.clone(),
                        ))
                    };
                    
                    // Block on the future to get the client
                    // This is not ideal, but it's the simplest way to handle this
                    // in the constructor
                    let runtime = tokio::runtime::Runtime::new().unwrap();
                    runtime.block_on(future)
                } else {
                    None
                }
            },
            _ => None,
        };

        Self {
            config,
            memory_cache: RwLock::new(HashMap::new()),
            dynamodb_storage,
        }
    }
    
    /// Initialize the conversation storage
    pub async fn initialize(&self) -> Result<()> {
        match self.config.storage_type {
            StorageType::Memory => {
                info!("Initializing in-memory conversation storage");
                // No initialization needed for memory storage
                Ok(())
            }
            StorageType::File => {
                info!("Initializing file-based conversation storage");
                let base_dir = self.config.base_directory.as_ref()
                    .ok_or_else(|| anyhow!("Base directory is required for file storage"))?;
                
                // Create the base directory if it doesn't exist
                tokio::fs::create_dir_all(base_dir).await?;
                
                // Create the conversations directory
                let conversations_dir = Path::new(base_dir).join("conversations");
                if !conversations_dir.exists() {
                    fs::create_dir_all(&conversations_dir)?;
                }
                
                Ok(())
            }
            StorageType::Database => {
                info!("Initializing database conversation storage");
                let _db_url = self.config.database_url.as_ref()
                    .ok_or_else(|| anyhow!("Database URL is required for database storage"))?;
                
                // TODO: Initialize database connection
                // This would typically involve connecting to the database and creating tables if they don't exist
                
                Ok(())
            }
            StorageType::DynamoDB => {
                info!("Initializing DynamoDB conversation storage");
                if let Some(dynamodb_storage) = &self.dynamodb_storage {
                    dynamodb_storage.initialize().await?;
                    Ok(())
                } else {
                    Err(anyhow!("DynamoDB storage not initialized"))
                }
            }
        }
    }
    
    /// Create a new conversation
    pub async fn create_conversation(&self, user_id: &str, client_data: Option<ClientData>) -> Result<Conversation> {
        let now = Utc::now();
        let conversation = Conversation {
            id: Uuid::new_v4().to_string(),
            user_id: user_id.to_string(),
            client_data,
            messages: Vec::new(),
            created_at: now,
            updated_at: now,
            tags: Vec::new(),
            summary: None,
            metadata: HashMap::new(),
        };
        
        // Store the conversation
        self.save_conversation(&conversation).await?;
        
        Ok(conversation)
    }
    
    /// Add a message to a conversation
    pub async fn add_message(&self, conversation_id: &str, role: MessageRole, content: &str, 
                            intent: Option<FinancialQueryIntent>, intent_confidence: Option<f64>,
                            metadata: Option<HashMap<String, String>>) -> Result<ConversationMessage> {
        // Get the conversation
        let mut conversation = self.get_conversation(conversation_id).await?;
        
        // Create the message
        let message = ConversationMessage {
            id: Uuid::new_v4().to_string(),
            timestamp: Utc::now(),
            role: role.to_string(),
            content: content.to_string(),
            intent: intent.map(|i| i.to_string()),
            intent_confidence,
            metadata: Some(metadata.unwrap_or_default()),
        };
        
        // Add the message to the conversation
        conversation.messages.push(message.clone());
        conversation.updated_at = Utc::now();
        
        // Update the conversation summary if enabled
        if self.config.enable_summarization {
            conversation.summary = self.generate_summary(&conversation).await.ok();
        }
        
        // Save the updated conversation
        self.save_conversation(&conversation).await?;
        
        Ok(message)
    }
    
    /// Get a conversation by ID
    pub async fn get_conversation(&self, conversation_id: &str) -> Result<Conversation> {
        // First, try to get from memory cache
        if let Some(conversation) = self.get_from_memory_cache(conversation_id).await? {
            return Ok(conversation);
        }
        
        // If not in memory, try to load from persistent storage
        match self.config.storage_type {
            StorageType::Memory => {
                // If using memory storage and not found in cache, it doesn't exist
                Err(anyhow!("Conversation not found: {}", conversation_id))
            }
            StorageType::File => {
                self.load_conversation_from_file(conversation_id).await
            }
            StorageType::Database => {
                self.load_conversation_from_database(conversation_id).await
            }
            StorageType::DynamoDB => {
                self.load_conversation_from_dynamodb(conversation_id).await
            }
        }
    }
    
    /// Save a conversation
    pub async fn save_conversation(&self, conversation: &Conversation) -> Result<()> {
        match self.config.storage_type {
            StorageType::Memory => {
                // Save the conversation to memory
                let mut cache = self.memory_cache.write().await;
                cache.insert(conversation.id.clone(), conversation.clone());
                
                // Prune the cache if it exceeds the maximum size
                if cache.len() > self.config.max_memory_conversations {
                    let mut entries: Vec<_> = cache.iter().map(|(k, v)| (k.clone(), v.updated_at.clone())).collect();
                    entries.sort_by(|(_, a), (_, b)| a.cmp(b));
                    
                    let to_remove = entries.len() - self.config.max_memory_conversations;
                    let keys_to_remove: Vec<String> = entries.into_iter()
                        .take(to_remove)
                        .map(|(k, _)| k)
                        .collect();
                    
                    for key in keys_to_remove {
                        cache.remove(&key);
                    }
                }
                
                Ok(())
            }
            StorageType::File => {
                // Save the conversation to a file
                let base_dir = self.config.base_directory.as_ref()
                    .ok_or_else(|| anyhow!("Base directory is required for file storage"))?;
                
                let conversations_dir = Path::new(base_dir).join("conversations");
                let file_path = conversations_dir.join(format!("{}.json", conversation.id));
                
                let file = File::create(file_path)?;
                let writer = BufWriter::new(file);
                serde_json::to_writer_pretty(writer, conversation)?;
                
                Ok(())
            }
            StorageType::Database => {
                // Database storage is not implemented yet
                Err(anyhow!("Database storage is not implemented yet"))
            }
            StorageType::DynamoDB => {
                if let Some(dynamodb_storage) = &self.dynamodb_storage {
                    dynamodb_storage.save_conversation(conversation).await?;
                    Ok(())
                } else {
                    Err(anyhow!("DynamoDB storage not initialized"))
                }
            }
        }
    }
    
    /// Delete a conversation
    pub async fn delete_conversation(&self, conversation_id: &str) -> Result<()> {
        // Remove from memory cache
        self.remove_from_memory_cache(conversation_id).await?;
        
        // Remove from persistent storage if applicable
        match self.config.storage_type {
            StorageType::Memory => {
                // Already removed from memory cache
                Ok(())
            }
            StorageType::File => {
                self.delete_conversation_from_file(conversation_id).await
            }
            StorageType::Database => {
                self.delete_conversation_from_database(conversation_id).await
            }
            StorageType::DynamoDB => {
                // DynamoDB storage doesn't require deleting
                Ok(())
            }
        }
    }
    
    /// List conversations for a user
    pub async fn list_conversations(&self, user_id: &str, limit: Option<i32>) -> Result<Vec<Conversation>> {
        match self.config.storage_type {
            StorageType::Memory => {
                // Get conversations from memory
                let cache = self.memory_cache.read().await;
                
                // Filter by user ID and collect into a vector
                let mut conversations: Vec<Conversation> = cache.values()
                    .filter(|conv| conv.user_id == user_id)
                    .cloned()
                    .collect();
                
                // Sort by updated_at (most recent first)
                conversations.sort_by(|a, b| b.updated_at.cmp(&a.updated_at));
                
                // Apply limit if specified
                if let Some(limit) = limit {
                    conversations.truncate(limit as usize);
                }
                
                Ok(conversations)
            }
            StorageType::File => {
                self.list_conversations_from_file(user_id, limit).await
            }
            StorageType::Database => {
                self.list_conversations_from_database(user_id, limit).await
            }
            StorageType::DynamoDB => {
                if let Some(dynamodb_storage) = &self.dynamodb_storage {
                    dynamodb_storage.list_conversations(user_id, limit).await
                } else {
                    Err(anyhow!("DynamoDB storage not initialized"))
                }
            }
        }
    }
    
    /// Search conversations by query
    pub async fn search_conversations(&self, query: &str) -> Result<Vec<Conversation>> {
        match self.config.storage_type {
            StorageType::Memory => {
                // Get conversations from memory
                let cache = self.memory_cache.read().await;
                
                // Simple search: check if query is in any message content
                let mut conversations: Vec<Conversation> = cache.values()
                    .filter(|conv| {
                        conv.messages.iter().any(|msg| 
                            msg.content.to_lowercase().contains(&query.to_lowercase())
                        )
                    })
                    .cloned()
                    .collect();
                
                // Sort by updated_at (most recent first)
                conversations.sort_by(|a, b| b.updated_at.cmp(&a.updated_at));
                
                Ok(conversations)
            }
            StorageType::File => {
                self.search_conversations_in_file(query).await
            }
            StorageType::Database => {
                self.search_conversations_in_database(query).await
            }
            StorageType::DynamoDB => {
                if let Some(dynamodb_storage) = &self.dynamodb_storage {
                    // First get all conversations
                    let all_conversations = dynamodb_storage.list_conversations("", None).await?;
                    
                    // Then filter them based on the query
                    let matching_conversations: Vec<Conversation> = all_conversations
                        .into_iter()
                        .filter(|conversation| {
                            conversation.messages.iter().any(|message| {
                                message.content.to_lowercase().contains(&query.to_lowercase())
                            })
                        })
                        .collect();
                    
                    Ok(matching_conversations)
                } else {
                    Err(anyhow!("DynamoDB storage not initialized"))
                }
            }
        }
    }
    
    /// Generate a summary for a conversation
    async fn generate_summary(&self, conversation: &Conversation) -> Result<String> {
        // TODO: Implement conversation summarization
        // This would typically involve using an LLM to generate a summary of the conversation
        
        // For now, just return a simple summary
        let message_count = conversation.messages.len();
        let summary = format!("Conversation with {} messages", message_count);
        
        Ok(summary)
    }
    
    /// Get a conversation from the memory cache
    async fn get_from_memory_cache(&self, conversation_id: &str) -> Result<Option<Conversation>> {
        let cache = self.memory_cache.read().await;
        
        if let Some(conversation) = cache.get(conversation_id) {
            // Return a clone of the conversation
            Ok(Some(conversation.clone()))
        } else {
            Ok(None)
        }
    }
    
    /// Update the memory cache with a conversation
    async fn update_memory_cache(&self, conversation: &Conversation) -> Result<()> {
        let mut cache = self.memory_cache.write().await;
        cache.insert(conversation.id.clone(), conversation.clone());
        
        // Prune the cache if it exceeds the maximum size
        if cache.len() > self.config.max_memory_conversations {
            let mut entries: Vec<_> = cache.iter()
                .map(|(k, v)| (k.clone(), v.updated_at))
                .collect();
            entries.sort_by(|(_, a), (_, b)| a.cmp(b));
            
            let to_remove = entries.len() - self.config.max_memory_conversations;
            let keys_to_remove: Vec<String> = entries.into_iter()
                .take(to_remove)
                .map(|(k, _)| k)
                .collect();
            
            for key in keys_to_remove {
                cache.remove(&key);
            }
        }
        
        Ok(())
    }
    
    /// Remove a conversation from the memory cache
    async fn remove_from_memory_cache(&self, conversation_id: &str) -> Result<()> {
        let mut cache = self.memory_cache.write().await;
        cache.remove(conversation_id);
        Ok(())
    }
    
    /// Load a conversation from a file
    async fn load_conversation_from_file(&self, conversation_id: &str) -> Result<Conversation> {
        let base_dir = self.config.base_directory.as_ref()
            .ok_or_else(|| anyhow!("Base directory is required for file storage"))?;
        
        let conversations_dir = Path::new(base_dir).join("conversations");
        let file_path = conversations_dir.join(format!("{}.json", conversation_id));
        
        // Check if the file exists
        if !file_path.exists() {
            return Err(anyhow!("Conversation not found: {}", conversation_id));
        }
        
        // Read the file
        let file = File::open(file_path)?;
        let reader = BufReader::new(file);
        let conversation: Conversation = serde_json::from_reader(reader)?;
        
        // Update the memory cache
        self.update_memory_cache(&conversation).await?;
        
        Ok(conversation)
    }
    
    /// Save a conversation to a file
    async fn save_conversation_to_file(&self, conversation: &Conversation) -> Result<()> {
        let base_dir = self.config.base_directory.as_ref()
            .ok_or_else(|| anyhow!("Base directory is required for file storage"))?;
        
        let conversations_dir = Path::new(base_dir).join("conversations");
        let file_path = conversations_dir.join(format!("{}.json", conversation.id));
        
        // Serialize the conversation to JSON
        let file = File::create(file_path)?;
        let writer = BufWriter::new(file);
        serde_json::to_writer_pretty(writer, conversation)?;
        
        Ok(())
    }
    
    /// Delete a conversation from a file
    async fn delete_conversation_from_file(&self, conversation_id: &str) -> Result<()> {
        let base_dir = self.config.base_directory.as_ref()
            .ok_or_else(|| anyhow!("Base directory is required for file storage"))?;
        
        let conversations_dir = Path::new(base_dir).join("conversations");
        let file_path = conversations_dir.join(format!("{}.json", conversation_id));
        
        // Check if the file exists
        if !file_path.exists() {
            return Err(anyhow!("Conversation not found: {}", conversation_id));
        }
        
        // Delete the file
        fs::remove_file(file_path)?;
        
        Ok(())
    }
    
    /// List conversations from files
    async fn list_conversations_from_file(&self, user_id: &str, limit: Option<i32>) -> Result<Vec<Conversation>> {
        let base_dir = self.config.base_directory.as_ref()
            .ok_or_else(|| anyhow!("Base directory is required for file storage"))?;
        
        let conversations_dir = Path::new(base_dir).join("conversations");
        if !conversations_dir.exists() {
            return Ok(Vec::new());
        }
        
        let mut conversations = Vec::new();
        
        for entry in fs::read_dir(conversations_dir)? {
            let entry = entry?;
            let path = entry.path();
            
            if path.is_file() && path.extension().map_or(false, |ext| ext == "json") {
                let file = File::open(path)?;
                let reader = BufReader::new(file);
                let conversation: Conversation = serde_json::from_reader(reader)?;
                
                if conversation.user_id == user_id {
                    conversations.push(conversation);
                }
            }
        }
        
        // Sort by updated_at (newest first)
        conversations.sort_by(|a, b| b.updated_at.cmp(&a.updated_at));
        
        // Apply limit
        let limit_i32 = limit.map(|l| l as i32);
        let conversations = if let Some(limit) = limit_i32 {
            conversations[..std::cmp::min(limit as usize, conversations.len())].to_vec()
        } else {
            conversations.to_vec()
        };
        
        Ok(conversations)
    }
    
    /// Search conversations in file
    async fn search_conversations_in_file(&self, query: &str) -> Result<Vec<Conversation>> {
        let base_dir = self.config.base_directory.as_ref()
            .ok_or_else(|| anyhow!("Base directory not specified"))?;
        
        let conversations_dir = Path::new(base_dir).join("conversations");
        if !conversations_dir.exists() {
            return Ok(Vec::new());
        }
        
        let mut matching_conversations = Vec::new();
        let query = query.to_lowercase();
        
        for entry in fs::read_dir(&conversations_dir)? {
            let entry = entry?;
            let path = entry.path();
            
            if path.is_file() && path.extension().map_or(false, |ext| ext == "json") {
                let file = File::open(&path)?;
                let reader = BufReader::new(file);
                
                if let Ok(conversation) = serde_json::from_reader::<_, Conversation>(reader) {
                    // Check if any message contains the query
                    let contains_query = conversation.messages.iter().any(|message| {
                        message.content.to_lowercase().contains(&query)
                    });
                    
                    if contains_query {
                        matching_conversations.push(conversation);
                    }
                }
            }
        }
        
        // Sort by updated_at (newest first)
        matching_conversations.sort_by(|a, b| b.updated_at.cmp(&a.updated_at));
        
        Ok(matching_conversations)
    }
    
    /// Load a conversation from a database
    async fn load_conversation_from_database(&self, conversation_id: &str) -> Result<Conversation> {
        match &self.config.database_url {
            Some(db_url) => {
                if db_url.starts_with("dynamodb:") {
                    // Extract region and table names from the URL
                    // Format: dynamodb:region:conversations_table:messages_table
                    let parts: Vec<&str> = db_url.split(':').collect();
                    if parts.len() != 4 {
                        return Err(anyhow!("Invalid DynamoDB URL format. Expected: dynamodb:region:conversations_table:messages_table"));
                    }
                    
                    let region = parts[1];
                    let conversations_table = parts[2];
                    let messages_table = parts[3];
                    
                    // Create AWS config
                    let config = aws_config::defaults(aws_config::BehaviorVersion::v2024_03_28())
                        .region(aws_sdk_dynamodb::config::Region::new(region.to_string()))
                        .load()
                        .await;
                    
                    // Create DynamoDB client
                    let client = aws_sdk_dynamodb::Client::new(&config);
                    
                    // Create DynamoDB storage
                    let dynamodb_storage = DynamoDbConversationStorage::new(
                        std::sync::Arc::new(client),
                        conversations_table.to_string(),
                        messages_table.to_string(),
                    );
                    
                    // Load the conversation
                    dynamodb_storage.load_conversation(conversation_id).await?
                        .ok_or_else(|| anyhow!("Conversation not found"))
                } else {
                    Err(anyhow!("Unsupported database URL: {}", db_url))
                }
            },
            None => Err(anyhow!("Database URL is required for database storage")),
        }
    }
    
    /// Save a conversation to a database
    async fn save_conversation_to_database(&self, conversation: &Conversation) -> Result<()> {
        match &self.config.database_url {
            Some(db_url) => {
                if db_url.starts_with("dynamodb:") {
                    // Extract region and table names from the URL
                    // Format: dynamodb:region:conversations_table:messages_table
                    let parts: Vec<&str> = db_url.split(':').collect();
                    if parts.len() != 4 {
                        return Err(anyhow!("Invalid DynamoDB URL format. Expected: dynamodb:region:conversations_table:messages_table"));
                    }
                    
                    let region = parts[1];
                    let conversations_table = parts[2];
                    let messages_table = parts[3];
                    
                    // Create AWS config
                    let config = aws_config::defaults(aws_config::BehaviorVersion::v2024_03_28())
                        .region(aws_sdk_dynamodb::config::Region::new(region.to_string()))
                        .load()
                        .await;
                    
                    // Create DynamoDB client
                    let client = aws_sdk_dynamodb::Client::new(&config);
                    
                    // Create DynamoDB storage
                    let dynamodb_storage = DynamoDbConversationStorage::new(
                        std::sync::Arc::new(client),
                        conversations_table.to_string(),
                        messages_table.to_string(),
                    );
                    
                    // Save the conversation
                    dynamodb_storage.save_conversation(conversation).await
                } else {
                    Err(anyhow!("Unsupported database URL: {}", db_url))
                }
            },
            None => Err(anyhow!("Database URL is required for database storage")),
        }
    }
    
    /// Delete a conversation from a database
    async fn delete_conversation_from_database(&self, conversation_id: &str) -> Result<()> {
        match &self.config.database_url {
            Some(db_url) => {
                if db_url.starts_with("dynamodb:") {
                    // Extract region and table names from the URL
                    // Format: dynamodb:region:conversations_table:messages_table
                    let parts: Vec<&str> = db_url.split(':').collect();
                    if parts.len() != 4 {
                        return Err(anyhow!("Invalid DynamoDB URL format. Expected: dynamodb:region:conversations_table:messages_table"));
                    }
                    
                    let region = parts[1];
                    let conversations_table = parts[2];
                    let messages_table = parts[3];
                    
                    // Create AWS config
                    let config = aws_config::defaults(aws_config::BehaviorVersion::v2024_03_28())
                        .region(aws_sdk_dynamodb::config::Region::new(region.to_string()))
                        .load()
                        .await;
                    
                    // Create DynamoDB client
                    let client = aws_sdk_dynamodb::Client::new(&config);
                    
                    // Create DynamoDB storage
                    let dynamodb_storage = DynamoDbConversationStorage::new(
                        std::sync::Arc::new(client),
                        conversations_table.to_string(),
                        messages_table.to_string(),
                    );
                    
                    // Delete the conversation
                    dynamodb_storage.delete_conversation(conversation_id).await
                } else {
                    Err(anyhow!("Unsupported database URL: {}", db_url))
                }
            },
            None => Err(anyhow!("Database URL is required for database storage")),
        }
    }
    
    /// List conversations from a database
    async fn list_conversations_from_database(&self, user_id: &str, limit: Option<i32>) -> Result<Vec<Conversation>> {
        match &self.config.database_url {
            Some(db_url) => {
                if db_url.starts_with("dynamodb:") {
                    // Extract region and table names from the URL
                    // Format: dynamodb:region:conversations_table:messages_table
                    let parts: Vec<&str> = db_url.split(':').collect();
                    if parts.len() != 4 {
                        return Err(anyhow!("Invalid DynamoDB URL format. Expected: dynamodb:region:conversations_table:messages_table"));
                    }
                    
                    let region = parts[1];
                    let conversations_table = parts[2];
                    let messages_table = parts[3];
                    
                    // Create AWS config
                    let config = aws_config::defaults(aws_config::BehaviorVersion::v2024_03_28())
                        .region(aws_sdk_dynamodb::config::Region::new(region.to_string()))
                        .load()
                        .await;
                    
                    // Create DynamoDB client
                    let client = aws_sdk_dynamodb::Client::new(&config);
                    
                    // Create DynamoDB storage
                    let dynamodb_storage = DynamoDbConversationStorage::new(
                        std::sync::Arc::new(client),
                        conversations_table.to_string(),
                        messages_table.to_string(),
                    );
                    
                    // List conversations
                    dynamodb_storage.list_conversations(user_id, limit).await
                } else {
                    Err(anyhow!("Unsupported database URL: {}", db_url))
                }
            },
            None => Err(anyhow!("Database URL is required for database storage")),
        }
    }
    
    /// Search conversations in database
    async fn search_conversations_in_database(&self, _query: &str) -> Result<Vec<Conversation>> {
        let db_url = self.config.database_url.as_ref()
            .ok_or_else(|| anyhow!("Database URL not specified"))?;
        
        if db_url.starts_with("sqlite:") {
            // SQLite implementation would go here
            Err(anyhow!("SQLite search not implemented yet"))
        } else if db_url.starts_with("postgres:") {
            // PostgreSQL implementation would go here
            Err(anyhow!("PostgreSQL search not implemented yet"))
        } else {
            Err(anyhow!("Unsupported database URL: {}", db_url))
        }
    }
    
    /// Load a conversation from DynamoDB
    async fn load_conversation_from_dynamodb(&self, conversation_id: &str) -> Result<Conversation> {
        if let Some(dynamodb_storage) = &self.dynamodb_storage {
            let conversation_result = dynamodb_storage.load_conversation(conversation_id).await?;
            match conversation_result {
                Some(conversation) => Ok(conversation),
                None => Err(anyhow!("Conversation not found: {}", conversation_id))
            }
        } else {
            Err(anyhow!("DynamoDB storage not initialized"))
        }
    }
    
    /// List conversations from DynamoDB
    async fn list_conversations_from_dynamodb(&self, user_id: &str, limit: Option<i32>) -> Result<Vec<Conversation>> {
        if let Some(dynamodb_storage) = &self.dynamodb_storage {
            dynamodb_storage.list_conversations(user_id, limit).await
        } else {
            Err(anyhow!("DynamoDB storage not initialized"))
        }
    }
    
    /// Search conversations in DynamoDB
    async fn search_conversations_from_dynamodb(&self, query: &str) -> Result<Vec<Conversation>> {
        if let Some(dynamodb_storage) = &self.dynamodb_storage {
            dynamodb_storage.search_conversations(query).await
        } else {
            Err(anyhow!("DynamoDB storage not initialized"))
        }
    }
}

impl ConversationStorageTrait for ConversationStorage {
    fn save_conversation<'a>(&'a self, conversation: &'a Conversation) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<()>> + Send + 'a>> {
        Box::pin(async move {
            self.save_conversation(conversation).await
        })
    }
    
    fn load_conversation<'a>(&'a self, conversation_id: &'a str) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<Option<Conversation>>> + Send + 'a>> {
        Box::pin(async move {
            match self.get_conversation(conversation_id).await {
                Ok(conversation) => Ok(Some(conversation)),
                Err(e) => {
                    if e.to_string().contains("not found") {
                        Ok(None)
                    } else {
                        Err(e)
                    }
                }
            }
        })
    }
    
    fn delete_conversation<'a>(&'a self, conversation_id: &'a str) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<()>> + Send + 'a>> {
        Box::pin(async move {
            self.delete_conversation(conversation_id).await
        })
    }
    
    fn list_conversations<'a>(&'a self, user_id: &'a str, limit: Option<i32>) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<Vec<Conversation>>> + Send + 'a>> {
        Box::pin(async move {
            self.list_conversations(user_id, limit).await
        })
    }
    
    fn search_conversations<'a>(&'a self, query: &'a str) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<Vec<Conversation>>> + Send + 'a>> {
        Box::pin(async move {
            self.search_conversations(query).await
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;
    
    #[tokio::test]
    async fn test_memory_storage() {
        // Create a storage instance
        let config = ConversationStorageConfig {
            storage_type: StorageType::Memory,
            base_directory: None,
            database_url: None,
            dynamodb_tables: None,
            max_memory_conversations: 10,
            enable_summarization: false,
        };
        let storage = ConversationStorage::new(config);
        
        // Create a test conversation
        let mut conversation = Conversation {
            id: Uuid::new_v4().to_string(),
            user_id: "user123".to_string(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
            messages: Vec::new(),
            client_data: None,
            tags: Vec::new(),
            summary: None,
            metadata: HashMap::new(),
        };
        
        // Add a message
        let message = ConversationMessage {
            id: Uuid::new_v4().to_string(),
            timestamp: Utc::now(),
            role: "user".to_string(),
            content: "Tell me about retirement planning".to_string(),
            intent: None,
            intent_confidence: None,
            metadata: None,
        };
        conversation.messages.push(message);
        
        // Save the conversation
        storage.save_conversation(&conversation).await.unwrap();
        
        // Load the conversation
        let loaded = storage.load_conversation(&conversation.id).await.unwrap();
        assert!(loaded.is_some());
        assert_eq!(loaded.unwrap().id, conversation.id);
        
        // List conversations
        let conversations = storage.list_conversations("user123", None).await.unwrap();
        assert_eq!(conversations.len(), 1);
        assert_eq!(conversations[0].id, conversation.id);
        
        // Search conversations
        let search_results = storage.search_conversations("retirement").await.unwrap();
        assert_eq!(search_results.len(), 1);
        assert_eq!(search_results[0].id, conversation.id);
        
        // Delete the conversation
        storage.delete_conversation(&conversation.id).await.unwrap();
        
        // Verify it's gone
        let loaded = storage.load_conversation(&conversation.id).await.unwrap();
        assert!(loaded.is_none());
    }
    
    #[tokio::test]
    async fn test_memory_cache_expiration() {
        // Create a conversation storage with a very short TTL
        let config = ConversationStorageConfig {
            storage_type: StorageType::Memory,
            max_memory_conversations: 10,
            ..Default::default()
        };
        
        let storage = ConversationStorage::new(config);
        storage.initialize().await.unwrap();
        
        // Create a conversation
        let conversation = storage.create_conversation("user123", None).await.unwrap();
        
        // Wait for the TTL to expire
        tokio::time::sleep(Duration::from_secs(2)).await;
        
        // Try to get the conversation (should fail due to TTL expiration)
        let result = storage.get_conversation(&conversation.id).await;
        assert!(result.is_err());
    }
} 