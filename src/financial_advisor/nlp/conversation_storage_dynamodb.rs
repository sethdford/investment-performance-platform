use std::collections::HashMap;
use std::sync::Arc;
use std::time::SystemTime;

use anyhow::{Result, anyhow};
use async_trait::async_trait;
use aws_sdk_dynamodb::{Client, types::AttributeValue};
use chrono::{DateTime, Utc};
use serde_json::Value;

use super::conversation_storage::{Conversation, ConversationMessage, ConversationStorageTrait};
use crate::financial_advisor::nlp::{MessageRole, FinancialQueryIntent};

/// DynamoDB implementation for conversation storage
pub struct DynamoDbConversationStorage {
    /// DynamoDB client
    client: Arc<Client>,
    
    /// Table name for conversations
    conversations_table: String,
    
    /// Table name for messages
    messages_table: String,
}

impl DynamoDbConversationStorage {
    /// Create a new DynamoDB conversation storage
    pub fn new(client: Arc<Client>, conversations_table: String, messages_table: String) -> Self {
        Self {
            client,
            conversations_table,
            messages_table,
        }
    }
    
    /// Initialize the DynamoDB tables
    pub async fn initialize(&self) -> Result<()> {
        // Check if tables exist
        let tables = self.client.list_tables().send().await?;
        
        // Check if table exists
        let table_names = tables.table_names();
        let table_names_vec: Vec<String> = table_names.to_vec();
        
        // Create conversations table if it doesn't exist
        if !table_names_vec.contains(&self.conversations_table) {
            self.create_conversations_table().await?;
        }
        
        // Create messages table if it doesn't exist
        if !table_names_vec.contains(&self.messages_table) {
            self.create_messages_table().await?;
        }
        
        Ok(())
    }
    
    /// Create the conversations table
    async fn create_conversations_table(&self) -> Result<()> {
        let key_schema = aws_sdk_dynamodb::types::KeySchemaElement::builder()
            .attribute_name("id")
            .key_type(aws_sdk_dynamodb::types::KeyType::Hash)
            .build();
            
        let id_attr_def = aws_sdk_dynamodb::types::AttributeDefinition::builder()
            .attribute_name("id")
            .attribute_type(aws_sdk_dynamodb::types::ScalarAttributeType::S)
            .build();
            
        let user_id_attr_def = aws_sdk_dynamodb::types::AttributeDefinition::builder()
            .attribute_name("user_id")
            .attribute_type(aws_sdk_dynamodb::types::ScalarAttributeType::S)
            .build();
            
        let user_id_key_schema = aws_sdk_dynamodb::types::KeySchemaElement::builder()
            .attribute_name("user_id")
            .key_type(aws_sdk_dynamodb::types::KeyType::Hash)
            .build();
            
        let projection = aws_sdk_dynamodb::types::Projection::builder()
            .projection_type(aws_sdk_dynamodb::types::ProjectionType::All)
            .build();
            
        // Create separate provisioned throughput instances for each use
        let gsi_throughput = aws_sdk_dynamodb::types::ProvisionedThroughput::builder()
            .read_capacity_units(5)
            .write_capacity_units(5)
            .build();
            
        let table_throughput = aws_sdk_dynamodb::types::ProvisionedThroughput::builder()
            .read_capacity_units(5)
            .write_capacity_units(5)
            .build();
            
        let gsi = aws_sdk_dynamodb::types::GlobalSecondaryIndex::builder()
            .index_name("UserIdIndex")
            .key_schema(user_id_key_schema?)
            .projection(projection)
            .provisioned_throughput(gsi_throughput?)
            .build()?;
            
        self.client.create_table()
            .table_name(&self.conversations_table)
            .key_schema(key_schema?)
            .attribute_definitions(id_attr_def?)
            .attribute_definitions(user_id_attr_def?)
            .global_secondary_indexes(gsi)
            .provisioned_throughput(table_throughput?)
            .send()
            .await?;
        
        Ok(())
    }
    
    /// Create the messages table
    async fn create_messages_table(&self) -> Result<()> {
        let id_attr_def = aws_sdk_dynamodb::types::AttributeDefinition::builder()
            .attribute_name("id")
            .attribute_type(aws_sdk_dynamodb::types::ScalarAttributeType::S)
            .build();
            
        let key_schema = aws_sdk_dynamodb::types::KeySchemaElement::builder()
            .attribute_name("id")
            .key_type(aws_sdk_dynamodb::types::KeyType::Hash)
            .build();
            
        let conv_id_attr_def = aws_sdk_dynamodb::types::AttributeDefinition::builder()
            .attribute_name("conversation_id")
            .attribute_type(aws_sdk_dynamodb::types::ScalarAttributeType::S)
            .build();
            
        let conv_id_key_schema = aws_sdk_dynamodb::types::KeySchemaElement::builder()
            .attribute_name("conversation_id")
            .key_type(aws_sdk_dynamodb::types::KeyType::Hash)
            .build();
            
        let projection = aws_sdk_dynamodb::types::Projection::builder()
            .projection_type(aws_sdk_dynamodb::types::ProjectionType::All)
            .build();
            
        // Create separate provisioned throughput instances for each use
        let gsi_throughput = aws_sdk_dynamodb::types::ProvisionedThroughput::builder()
            .read_capacity_units(5)
            .write_capacity_units(5)
            .build();
            
        let table_throughput = aws_sdk_dynamodb::types::ProvisionedThroughput::builder()
            .read_capacity_units(5)
            .write_capacity_units(5)
            .build();
            
        let gsi = aws_sdk_dynamodb::types::GlobalSecondaryIndex::builder()
            .index_name("ConversationIdIndex")
            .key_schema(conv_id_key_schema?)
            .projection(projection)
            .provisioned_throughput(gsi_throughput?)
            .build()?;
            
        self.client.create_table()
            .table_name(&self.messages_table)
            .key_schema(key_schema?)
            .attribute_definitions(id_attr_def?)
            .attribute_definitions(conv_id_attr_def?)
            .global_secondary_indexes(gsi)
            .provisioned_throughput(table_throughput?)
            .send()
            .await?;
        
        Ok(())
    }
    
    /// Save a conversation to DynamoDB
    pub async fn save_conversation(&self, conversation: &Conversation) -> Result<()> {
        // Convert the conversation to DynamoDB attributes
        let mut item = HashMap::new();
        item.insert("id".to_string(), AttributeValue::S(conversation.id.clone()));
        item.insert("user_id".to_string(), AttributeValue::S(conversation.user_id.clone()));
        
        // Convert client data to JSON if available
        if let Some(client_data) = &conversation.client_data {
            if let Ok(json) = serde_json::to_string(client_data) {
                item.insert("client_data".to_string(), AttributeValue::S(json));
            }
        }
        
        // Convert DateTime to timestamp
        let created_at_secs = conversation.created_at.timestamp();
        item.insert("created_at".to_string(), AttributeValue::N(created_at_secs.to_string()));
        
        let updated_at_secs = conversation.updated_at.timestamp();
        item.insert("updated_at".to_string(), AttributeValue::N(updated_at_secs.to_string()));
        
        // Convert tags to a set
        if !conversation.tags.is_empty() {
            item.insert("tags".to_string(), AttributeValue::Ss(conversation.tags.clone()));
        }
        
        // Add summary if available
        if let Some(summary) = &conversation.summary {
            item.insert("summary".to_string(), AttributeValue::S(summary.clone()));
        }
        
        // Save the conversation
        self.client.put_item()
            .table_name(&self.conversations_table)
            .set_item(Some(item))
            .send()
            .await?;
        
        // Save the messages
        for message in &conversation.messages {
            self.save_message(message, &conversation.id).await?;
        }
        
        Ok(())
    }
    
    /// Save a message to DynamoDB
    async fn save_message(&self, message: &ConversationMessage, conversation_id: &str) -> Result<()> {
        // Create the item
        let mut item = HashMap::new();
        
        // Add ID
        item.insert("id".to_string(), AttributeValue::S(message.id.clone()));
        
        // Add conversation ID
        item.insert("conversation_id".to_string(), AttributeValue::S(conversation_id.to_string()));
        
        // Convert DateTime to timestamp
        let timestamp_secs = message.timestamp.timestamp();
        item.insert("timestamp".to_string(), AttributeValue::N(timestamp_secs.to_string()));
        
        // Convert role to string
        let role_str = message.role.clone();
        item.insert("role".to_string(), AttributeValue::S(role_str));
        
        // Add content
        let content = message.content.clone();
        item.insert("content".to_string(), AttributeValue::S(content));
        
        // Add intent if available
        if let Some(intent_str) = &message.intent {
            item.insert("intent".to_string(), AttributeValue::S(intent_str.clone()));
        }
        
        // Add intent confidence if available
        if let Some(confidence) = message.intent_confidence {
            item.insert("intent_confidence".to_string(), AttributeValue::N(confidence.to_string()));
        }
        
        // Add metadata if available
        if let Some(metadata) = &message.metadata {
            if !metadata.is_empty() {
                let metadata_map: HashMap<String, AttributeValue> = metadata
                    .iter()
                    .map(|(k, v)| (k.clone(), AttributeValue::S(v.clone())))
                    .collect();
                item.insert("metadata".to_string(), AttributeValue::M(metadata_map));
            }
        }
        
        // Save the item
        self.client.put_item()
            .table_name(&self.messages_table)
            .set_item(Some(item))
            .send()
            .await?;
        
        Ok(())
    }
    
    /// Load a conversation from DynamoDB
    pub async fn load_conversation(&self, conversation_id: &str) -> Result<Option<Conversation>> {
        // Query the conversations table
        let result = self.client.query()
            .table_name(&self.conversations_table)
            .key_condition_expression("id = :id")
            .expression_attribute_values(":id", AttributeValue::S(conversation_id.to_string()))
            .send()
            .await?;
        
        // Get the items
        let items = result.items.clone().unwrap_or_default();
        
        if items.is_empty() {
            return Ok(None);
        }
        
        // Check if the conversation exists
        let item = items[0].clone();
        
        // Extract the user ID
        let user_id = item.get("user_id")
            .and_then(|v| v.as_s().ok())
            .ok_or_else(|| anyhow!("Invalid user ID for conversation: {}", conversation_id))?
            .clone();
        
        // Extract the client data if available
        let client_data = item.get("client_data")
            .and_then(|v| v.as_s().ok())
            .and_then(|s| serde_json::from_str(s).ok());
        
        // Extract the timestamps
        let created_at = item.get("created_at")
            .and_then(|v| v.as_n().ok())
            .and_then(|n| n.parse::<i64>().ok())
            .map(|secs| SystemTime::UNIX_EPOCH + std::time::Duration::from_secs(secs as u64))
            .unwrap_or_else(SystemTime::now);
        
        let updated_at = item.get("updated_at")
            .and_then(|v| v.as_n().ok())
            .and_then(|n| n.parse::<i64>().ok())
            .map(|secs| SystemTime::UNIX_EPOCH + std::time::Duration::from_secs(secs as u64))
            .unwrap_or_else(SystemTime::now);
        
        // Extract the tags
        let tags = item.get("tags")
            .and_then(|v| v.as_ss().ok())
            .map(|ss| ss.to_vec())
            .unwrap_or_default();
        
        // Extract the summary
        let summary = item.get("summary")
            .and_then(|v| v.as_s().ok())
            .map(|s| s.clone());
        
        // Load the messages
        let messages = self.load_messages(conversation_id).await?;
        
        // Create the conversation
        let conversation = Conversation {
            id: conversation_id.to_string(),
            user_id,
            client_data,
            messages,
            created_at: chrono::DateTime::<Utc>::from(created_at),
            updated_at: chrono::DateTime::<Utc>::from(updated_at),
            tags,
            summary,
            metadata: HashMap::new(),
        };
        
        Ok(Some(conversation))
    }
    
    /// Load messages for a conversation from DynamoDB
    pub async fn load_messages(&self, conversation_id: &str) -> Result<Vec<ConversationMessage>> {
        // Query the messages table
        let result = self.client.query()
            .table_name(&self.messages_table)
            .index_name("ConversationIdIndex")
            .key_condition_expression("conversation_id = :id")
            .expression_attribute_values(":id", AttributeValue::S(conversation_id.to_string()))
            .send()
            .await?;
        
        // Get the items
        let items = result.items.clone().unwrap_or_default();
        
        // Convert items to messages
        let mut messages = Vec::new();
        
        for item in items {
            // Extract the ID
            let id = item.get("id")
                .and_then(|v| v.as_s().ok())
                .ok_or_else(|| anyhow!("Invalid message ID"))?;
                
            // Extract the timestamp
            let timestamp_str = item.get("timestamp")
                .and_then(|v| v.as_n().ok())
                .ok_or_else(|| anyhow!("Invalid message timestamp"))?;
                
            let timestamp_secs = timestamp_str.parse::<i64>()
                .map_err(|_| anyhow!("Invalid timestamp format"))?;
                
            let timestamp = chrono::DateTime::<Utc>::from_timestamp(timestamp_secs, 0)
                .ok_or_else(|| anyhow!("Invalid timestamp value"))?;
                
            // Extract the role
            let role_str = item.get("role")
                .and_then(|v| v.as_s().ok())
                .ok_or_else(|| anyhow!("Invalid message role"))?;
                
            // Extract the content
            let content = item.get("content")
                .and_then(|v| v.as_s().ok())
                .ok_or_else(|| anyhow!("Invalid message content"))?;
                
            // Extract the intent if available
            let intent_str = item.get("intent")
                .and_then(|v| v.as_s().ok())
                .map(|s| s.to_string());
                
            // Extract the intent confidence if available
            let intent_confidence = item.get("intent_confidence")
                .and_then(|v| v.as_n().ok())
                .and_then(|n| n.parse::<f64>().ok());
                
            // Extract the metadata if available
            let metadata = item.get("metadata")
                .and_then(|v| v.as_m().ok())
                .map(|m| {
                    m.iter()
                        .filter_map(|(k, v)| {
                            v.as_s().ok().map(|s| (k.clone(), s.clone()))
                        })
                        .collect::<HashMap<String, String>>()
                });
                
            // Create the message
            let message = ConversationMessage {
                id: id.to_string(),
                timestamp,
                role: role_str.to_string(),
                content: content.to_string(),
                intent: intent_str,
                intent_confidence,
                metadata,
            };
            
            messages.push(message);
        }
        
        Ok(messages)
    }
    
    /// Delete a conversation from DynamoDB
    pub async fn delete_conversation(&self, conversation_id: &str) -> Result<()> {
        // Delete the conversation
        self.client.delete_item()
            .table_name(&self.conversations_table)
            .key("id", AttributeValue::S(conversation_id.to_string()))
            .send()
            .await?;
        
        // Delete the messages
        self.delete_messages(conversation_id).await?;
        
        Ok(())
    }
    
    /// Delete messages for a conversation from DynamoDB
    async fn delete_messages(&self, conversation_id: &str) -> Result<()> {
        // Query the messages
        let result = self.client.query()
            .table_name(&self.messages_table)
            .index_name("ConversationIdIndex")
            .key_condition_expression("conversation_id = :conversation_id")
            .expression_attribute_values(":conversation_id", AttributeValue::S(conversation_id.to_string()))
            .send()
            .await?;
        
        let items = result.items.clone().unwrap_or_default();
        
        // Delete each message
        for item in items {
            if let Some(id) = item.get("id").and_then(|v| v.as_s().ok()) {
                self.client.delete_item()
                    .table_name(&self.messages_table)
                    .key("id", AttributeValue::S(id.clone()))
                    .send()
                    .await?;
            }
        }
        
        Ok(())
    }
    
    /// List conversations for a user
    pub async fn list_conversations(&self, user_id: &str, limit: Option<i32>) -> Result<Vec<Conversation>> {
        // Query the conversations table
        let mut query = self.client.query()
            .table_name(&self.conversations_table)
            .index_name("user_id-index")
            .key_condition_expression("user_id = :user_id")
            .expression_attribute_values(":user_id", AttributeValue::S(user_id.to_string()));
        
        // Apply limit if specified
        if let Some(limit_val) = limit {
            query = query.limit(limit_val);
        }
        
        let result = query.send().await?;
        
        // Get the items
        let items = result.items.clone().unwrap_or_default();
        
        // Convert the items to conversations
        let mut conversations = Vec::new();
        for item in items {
            // Extract the conversation ID
            let conversation_id = item.get("conversation_id")
                .and_then(|v| v.as_s().ok())
                .ok_or_else(|| anyhow!("Missing conversation_id in DynamoDB item"))?;
            
            // Load the conversation
            if let Some(conversation) = self.load_conversation(conversation_id).await? {
                conversations.push(conversation);
            }
        }
        
        Ok(conversations)
    }
    
    /// Search conversations for a user in DynamoDB
    pub async fn search_conversations(&self, _query: &str) -> Result<Vec<Conversation>> {
        // For simplicity, we'll just list all conversations and filter them
        // In a real implementation, you would use DynamoDB's query capabilities
        
        // List all conversations
        let result = self.client.scan()
            .table_name(&self.conversations_table)
            .send()
            .await?;
        
        // Get the items
        let items = result.items.clone().unwrap_or_default();
        
        // Convert the items to conversations
        let mut conversations = Vec::new();
        for item in items {
            // Extract the conversation ID
            let conversation_id = item.get("conversation_id")
                .and_then(|v| v.as_s().ok())
                .ok_or_else(|| anyhow!("Missing conversation_id in DynamoDB item"))?;
            
            // Load the conversation
            if let Some(conversation) = self.load_conversation(conversation_id).await? {
                conversations.push(conversation);
            }
        }
        
        Ok(conversations)
    }
}

#[async_trait]
impl ConversationStorageTrait for DynamoDbConversationStorage {
    fn save_conversation<'a>(&'a self, conversation: &'a Conversation) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<()>> + Send + 'a>> {
        Box::pin(async move {
            // Reuse the existing implementation
            self.save_conversation(conversation).await
        })
    }
    
    fn load_conversation<'a>(&'a self, conversation_id: &'a str) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<Option<Conversation>>> + Send + 'a>> {
        Box::pin(async move {
            // Reuse the existing implementation
            self.load_conversation(conversation_id).await
        })
    }
    
    fn delete_conversation<'a>(&'a self, conversation_id: &'a str) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<()>> + Send + 'a>> {
        Box::pin(async move {
            // Reuse the existing implementation
            self.delete_conversation(conversation_id).await
        })
    }
    
    fn list_conversations<'a>(&'a self, user_id: &'a str, limit: Option<i32>) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<Vec<Conversation>>> + Send + 'a>> {
        Box::pin(async move {
            // Reuse the existing implementation
            self.list_conversations(user_id, limit).await
        })
    }
    
    fn search_conversations<'a>(&'a self, query: &'a str) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<Vec<Conversation>>> + Send + 'a>> {
        Box::pin(async move {
            // Reuse the existing implementation
            self.search_conversations(query).await
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use aws_sdk_dynamodb::config::{Region, Credentials};
    
    async fn create_test_client() -> Client {
        // Create a local DynamoDB client for testing
        let credentials = Credentials::new(
            "test",
            "test",
            None,
            None,
            "test",
        );
        
        let config = aws_sdk_dynamodb::config::Builder::new()
            .region(Region::new("us-east-1"))
            .endpoint_url("http://localhost:8000")
            .credentials_provider(credentials)
            .build();
        
        Client::from_conf(config)
    }
    
    #[tokio::test]
    async fn test_dynamodb_storage() -> Result<()> {
        // Create a test client
        let client = create_test_client().await;
        
        // Create a DynamoDB storage
        let storage = DynamoDbConversationStorage::new(
            Arc::new(client),
            "test_conversations".to_string(),
            "test_messages".to_string(),
        );
        
        // Initialize the storage
        storage.initialize().await.unwrap();
        
        // Create a conversation
        let mut conversation = Conversation {
            id: "test_conversation".to_string(),
            user_id: "test_user".to_string(),
            client_data: None,
            messages: Vec::new(),
            created_at: SystemTime::now().into(),
            updated_at: SystemTime::now().into(),
            tags: vec!["test".to_string()],
            summary: Some("Test conversation".to_string()),
            metadata: HashMap::new(),
        };
        
        // Add a message
        let message = ConversationMessage {
            id: "test_message".to_string(),
            timestamp: SystemTime::now().into(),
            role: MessageRole::User.to_string(),
            content: "Hello, world!".to_string(),
            intent: Some(FinancialQueryIntent::Greeting.to_string()),
            intent_confidence: Some(0.95),
            metadata: Some(HashMap::new()),
        };
        
        conversation.messages.push(message);
        
        // Save the conversation
        storage.save_conversation(&conversation).await.unwrap();
        
        // Verify the loaded conversation
        let loaded_conversation = storage.load_conversation("test_conversation").await?;
        assert!(loaded_conversation.is_some());
        let loaded_conversation = loaded_conversation.unwrap();
        assert_eq!(loaded_conversation.id, conversation.id);
        assert_eq!(loaded_conversation.user_id, conversation.user_id);
        assert_eq!(loaded_conversation.messages.len(), conversation.messages.len());
        assert_eq!(loaded_conversation.messages[0].content, conversation.messages[0].content);
        
        // Delete the conversation
        storage.delete_conversation("test_conversation").await.unwrap();
        
        // Verify the conversation was deleted
        let loaded_conversation = storage.load_conversation("test_conversation").await?;
        assert!(loaded_conversation.is_none());
        
        Ok(())
    }
} 