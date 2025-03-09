use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use anyhow::{Result, anyhow};
use async_trait::async_trait;
use aws_sdk_dynamodb::Client;
use aws_sdk_dynamodb::types::{
    AttributeDefinition, AttributeValue, GlobalSecondaryIndex, KeySchemaElement,
    KeyType, Projection, ProjectionType, ProvisionedThroughput, ScalarAttributeType
};
use chrono::{DateTime, Utc};

use super::conversation_summary::{ConversationSummary, SummaryStorage};

/// In-memory storage for conversation summaries
pub struct InMemorySummaryStorage {
    /// Map of summary ID to summary
    summaries: Arc<Mutex<HashMap<String, ConversationSummary>>>,
    
    /// Map of conversation ID to summary IDs
    conversation_summaries: Arc<Mutex<HashMap<String, Vec<String>>>>,
}

impl InMemorySummaryStorage {
    /// Create a new in-memory summary storage
    pub fn new() -> Self {
        Self {
            summaries: Arc::new(Mutex::new(HashMap::new())),
            conversation_summaries: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}

#[async_trait]
impl SummaryStorage for InMemorySummaryStorage {
    fn save_summary<'a>(&'a self, summary: &'a ConversationSummary) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<()>> + Send + 'a>> {
        Box::pin(async move {
            let mut summaries = self.summaries.lock()
                .map_err(|_| anyhow!("Failed to lock summaries"))?;
            summaries.insert(summary.id.clone(), summary.clone());
            
            // Update the conversation summaries map
            let mut conversation_summaries = self.conversation_summaries.lock()
                .map_err(|_| anyhow!("Failed to lock conversation_summaries"))?;
            
            let summary_ids = conversation_summaries
                .entry(summary.conversation_id.clone())
                .or_insert_with(Vec::new);
            
            if !summary_ids.contains(&summary.id) {
                summary_ids.push(summary.id.clone());
            }
            
            Ok(())
        })
    }
    
    fn get_summary<'a>(&'a self, summary_id: &'a str) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<Option<ConversationSummary>>> + Send + 'a>> {
        Box::pin(async move {
            let summaries = self.summaries.lock()
                .map_err(|_| anyhow!("Failed to lock summaries"))?;
            Ok(summaries.get(summary_id).cloned())
        })
    }
    
    fn list_summaries<'a>(&'a self, conversation_id: &'a str) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<Vec<ConversationSummary>>> + Send + 'a>> {
        Box::pin(async move {
            let conversation_summaries = self.conversation_summaries.lock()
                .map_err(|_| anyhow!("Failed to lock conversation_summaries"))?;
            
            let summary_ids = match conversation_summaries.get(conversation_id) {
                Some(ids) => ids.clone(),
                None => return Ok(Vec::new()),
            };
            
            let summaries = self.summaries.lock()
                .map_err(|_| anyhow!("Failed to lock summaries"))?;
            
            let mut result = Vec::new();
            for id in summary_ids {
                if let Some(summary) = summaries.get(&id) {
                    result.push(summary.clone());
                }
            }
            
            // Sort by version, descending
            result.sort_by(|a, b| b.version.cmp(&a.version));
            
            Ok(result)
        })
    }
    
    fn delete_summary<'a>(&'a self, summary_id: &'a str) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<()>> + Send + 'a>> {
        Box::pin(async move {
            let mut summaries = self.summaries.lock()
                .map_err(|_| anyhow!("Failed to lock summaries"))?;
            
            // Get the conversation ID before removing the summary
            let conversation_id = match summaries.get(summary_id) {
                Some(summary) => summary.conversation_id.clone(),
                None => return Ok(()), // Summary doesn't exist, nothing to do
            };
            
            // Remove the summary
            summaries.remove(summary_id);
            
            // Update the conversation summaries map
            let mut conversation_summaries = self.conversation_summaries.lock()
                .map_err(|_| anyhow!("Failed to lock conversation_summaries"))?;
            
            if let Some(summary_ids) = conversation_summaries.get_mut(&conversation_id) {
                summary_ids.retain(|id| id != summary_id);
            }
            
            Ok(())
        })
    }
}

/// DynamoDB storage for conversation summaries
pub struct DynamoDbSummaryStorage {
    /// DynamoDB client
    client: Client,
    
    /// Table name for summaries
    table_name: String,
}

impl DynamoDbSummaryStorage {
    /// Create a new DynamoDB summary storage
    pub fn new(client: Client, table_name: String) -> Self {
        Self {
            client,
            table_name,
        }
    }
    
    /// Initialize the DynamoDB table
    pub async fn initialize(&self) -> Result<()> {
        // Check if the table exists
        let tables = self.client.list_tables().send().await?;
        
        let table_names = tables.table_names();
        let table_names_vec: Vec<String> = table_names.to_vec();
        
        if table_names_vec.contains(&self.table_name) {
            return Ok(());
        }
        
        // Create the table
        self.client.create_table()
            .table_name(&self.table_name)
            .key_schema(
                KeySchemaElement::builder()
                    .attribute_name("id")
                    .key_type(KeyType::Hash)
                    .build()?,
            )
            .attribute_definitions(
                AttributeDefinition::builder()
                    .attribute_name("id")
                    .attribute_type(ScalarAttributeType::S)
                    .build()?,
            )
            .attribute_definitions(
                AttributeDefinition::builder()
                    .attribute_name("conversation_id")
                    .attribute_type(ScalarAttributeType::S)
                    .build()?,
            )
            .global_secondary_indexes(
                GlobalSecondaryIndex::builder()
                    .index_name("conversation_id-index")
                    .key_schema(
                        KeySchemaElement::builder()
                            .attribute_name("conversation_id")
                            .key_type(KeyType::Hash)
                            .build()?,
                    )
                    .projection(
                        Projection::builder()
                            .projection_type(ProjectionType::All)
                            .build(),
                    )
                    .provisioned_throughput(
                        ProvisionedThroughput::builder()
                            .read_capacity_units(5)
                            .write_capacity_units(5)
                            .build()?,
                    )
                    .build()?,
            )
            .provisioned_throughput(
                ProvisionedThroughput::builder()
                    .read_capacity_units(5)
                    .write_capacity_units(5)
                    .build()?,
            )
            .send()
            .await?;
        
        Ok(())
    }

    /// Create the table
    async fn create_table(&self) -> Result<()> {
        // Create the table
        self.client.create_table()
            .table_name(&self.table_name)
            .key_schema(
                KeySchemaElement::builder()
                    .attribute_name("id")
                    .key_type(KeyType::Hash)
                    .build()?,
            )
            .attribute_definitions(
                AttributeDefinition::builder()
                    .attribute_name("id")
                    .attribute_type(ScalarAttributeType::S)
                    .build()?,
            )
            .attribute_definitions(
                AttributeDefinition::builder()
                    .attribute_name("conversation_id")
                    .attribute_type(ScalarAttributeType::S)
                    .build()?,
            )
            .global_secondary_indexes(
                GlobalSecondaryIndex::builder()
                    .index_name("conversation_id-index")
                    .key_schema(
                        KeySchemaElement::builder()
                            .attribute_name("conversation_id")
                            .key_type(KeyType::Hash)
                            .build()?,
                    )
                    .projection(
                        Projection::builder()
                            .projection_type(ProjectionType::All)
                            .build(),
                    )
                    .provisioned_throughput(
                        ProvisionedThroughput::builder()
                            .read_capacity_units(5)
                            .write_capacity_units(5)
                            .build()?,
                    )
                    .build()?,
            )
            .provisioned_throughput(
                ProvisionedThroughput::builder()
                    .read_capacity_units(5)
                    .write_capacity_units(5)
                    .build()?,
            )
            .send()
            .await?;
        
        Ok(())
    }
}

#[async_trait]
impl SummaryStorage for DynamoDbSummaryStorage {
    fn save_summary<'a>(&'a self, summary: &'a ConversationSummary) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<()>> + Send + 'a>> {
        Box::pin(async move {
            // Convert the summary to a DynamoDB item
            let mut item = HashMap::new();
            
            // Add the required fields
            item.insert("id".to_string(), AttributeValue::S(summary.id.clone()));
            item.insert("conversation_id".to_string(), AttributeValue::S(summary.conversation_id.clone()));
            
            // Note: summary_type field is not present in ConversationSummary struct
            // Using a default value of "Standard"
            item.insert("summary_type".to_string(), AttributeValue::S("Standard".to_string()));
            
            item.insert("content".to_string(), AttributeValue::S(summary.content.clone()));
            item.insert("version".to_string(), AttributeValue::N(summary.version.to_string()));
            item.insert("created_at".to_string(), AttributeValue::S(summary.created_at.to_rfc3339()));
            item.insert("updated_at".to_string(), AttributeValue::S(summary.updated_at.to_rfc3339()));
            item.insert("last_message_id".to_string(), AttributeValue::S(summary.last_message_id.clone()));
            
            // Put the item in the table
            self.client.put_item()
                .table_name(&self.table_name)
                .set_item(Some(item))
                .send()
                .await?;
            
            Ok(())
        })
    }
    
    fn get_summary<'a>(&'a self, summary_id: &'a str) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<Option<ConversationSummary>>> + Send + 'a>> {
        Box::pin(async move {
            // Get the item from the table
            let result = self.client.get_item()
                .table_name(&self.table_name)
                .key("id", AttributeValue::S(summary_id.to_string()))
                .send()
                .await?;
            
            // Convert the item to a summary
            if let Some(item) = result.item() {
                let summary = self.convert_item_to_summary(item)?;
                Ok(Some(summary))
            } else {
                Ok(None)
            }
        })
    }
    
    fn list_summaries<'a>(&'a self, conversation_id: &'a str) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<Vec<ConversationSummary>>> + Send + 'a>> {
        Box::pin(async move {
            // Query the table using the GSI
            let result = self.client.query()
                .table_name(&self.table_name)
                .index_name("conversation_id-index")
                .key_condition_expression("conversation_id = :conversation_id")
                .expression_attribute_values(
                    ":conversation_id",
                    AttributeValue::S(conversation_id.to_string()),
                )
                .send()
                .await?;
            
            // Convert the items to summaries
            let mut summaries = Vec::new();
            
            // Check if result.items is Some and not empty
            if let Some(items) = &result.items {
                for item in items {
                    // Process each item individually
                    let summary = self.convert_item_to_summary(item)?;
                    summaries.push(summary);
                }
            }
            
            Ok(summaries)
        })
    }
    
    fn delete_summary<'a>(&'a self, summary_id: &'a str) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<()>> + Send + 'a>> {
        Box::pin(async move {
            // Delete the item from the table
            self.client.delete_item()
                .table_name(&self.table_name)
                .key("id", AttributeValue::S(summary_id.to_string()))
                .send()
                .await?;
            
            Ok(())
        })
    }
}

impl DynamoDbSummaryStorage {
    /// Convert a DynamoDB item to a ConversationSummary
    fn convert_item_to_summary(&self, item: &HashMap<String, AttributeValue>) -> Result<ConversationSummary> {
        let id = item.get("id")
            .and_then(|v| v.as_s().ok())
            .ok_or_else(|| anyhow!("Missing id field"))?
            .clone();
        
        let conversation_id = item.get("conversation_id")
            .and_then(|v| v.as_s().ok())
            .ok_or_else(|| anyhow!("Missing conversation_id field"))?
            .clone();
        
        // Note: We're ignoring the summary_type field since it's not in the ConversationSummary struct
        
        let content = item.get("content")
            .and_then(|v| v.as_s().ok())
            .or_else(|| item.get("text").and_then(|v| v.as_s().ok()))
            .ok_or_else(|| anyhow!("Missing content/text field"))?
            .clone();
        
        let version = item.get("version")
            .and_then(|v| v.as_n().ok())
            .and_then(|n| n.parse::<u32>().ok())
            .unwrap_or(1);
        
        let created_at_str = item.get("created_at")
            .and_then(|v| v.as_s().ok())
            .ok_or_else(|| anyhow!("Missing created_at field"))?;
        
        let created_at = DateTime::parse_from_rfc3339(created_at_str)
            .map_err(|e| anyhow!("Invalid created_at format: {}", e))?
            .with_timezone(&Utc);
        
        let updated_at_str = item.get("updated_at")
            .and_then(|v| v.as_s().ok())
            .unwrap_or(created_at_str);
        
        let updated_at = DateTime::parse_from_rfc3339(updated_at_str)
            .map_err(|e| anyhow!("Invalid updated_at format: {}", e))?
            .with_timezone(&Utc);
        
        let last_message_id = item.get("last_message_id")
            .and_then(|v| v.as_s().ok())
            .unwrap_or(&"".to_string())
            .clone();
        
        // For simplicity, we're not parsing the complex nested structures
        // In a real implementation, you would parse these from the DynamoDB item
        
        Ok(ConversationSummary {
            id,
            conversation_id,
            content,
            version,
            created_at,
            updated_at,
            last_message_id,
            key_entities: Vec::new(),
            key_decisions: Vec::new(),
            topics: Vec::new(),
        })
    }
}

/// DynamoDB-based conversation summary storage
pub struct DynamoDbConversationSummaryStorage {
    /// DynamoDB client
    client: Client,
    
    /// Table name for summaries
    table_name: String,
}

impl DynamoDbConversationSummaryStorage {
    /// Create a new DynamoDB conversation summary storage
    pub fn new(client: Client, table_name: String) -> Self {
        Self {
            client,
            table_name,
        }
    }
    
    /// Ensure the table exists
    pub async fn ensure_table_exists(&self) -> Result<()> {
        // List tables
        let tables = self.client.list_tables().send().await?;
        
        // Check if table exists
        let table_names = tables.table_names();
        let table_names_vec: Vec<String> = table_names.to_vec();
        
        if !table_names_vec.contains(&self.table_name) {
            // Create table
            self.create_table().await?;
        }
        
        Ok(())
    }

    /// Create the table
    async fn create_table(&self) -> Result<()> {
        // Create the table
        self.client.create_table()
            .table_name(&self.table_name)
            .key_schema(
                KeySchemaElement::builder()
                    .attribute_name("id")
                    .key_type(KeyType::Hash)
                    .build()?,
            )
            .attribute_definitions(
                AttributeDefinition::builder()
                    .attribute_name("id")
                    .attribute_type(ScalarAttributeType::S)
                    .build()?,
            )
            .attribute_definitions(
                AttributeDefinition::builder()
                    .attribute_name("conversation_id")
                    .attribute_type(ScalarAttributeType::S)
                    .build()?,
            )
            .global_secondary_indexes(
                GlobalSecondaryIndex::builder()
                    .index_name("conversation_id-index")
                    .key_schema(
                        KeySchemaElement::builder()
                            .attribute_name("conversation_id")
                            .key_type(KeyType::Hash)
                            .build()?,
                    )
                    .projection(
                        Projection::builder()
                            .projection_type(ProjectionType::All)
                            .build(),
                    )
                    .provisioned_throughput(
                        ProvisionedThroughput::builder()
                            .read_capacity_units(5)
                            .write_capacity_units(5)
                            .build()?,
                    )
                    .build()?,
            )
            .provisioned_throughput(
                ProvisionedThroughput::builder()
                    .read_capacity_units(5)
                    .write_capacity_units(5)
                    .build()?,
            )
            .send()
            .await?;
        
        Ok(())
    }
} 