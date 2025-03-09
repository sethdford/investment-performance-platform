use anyhow::Result;
use chrono::Utc;
use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use uuid::Uuid;

use super::conversation_manager::{ConversationManager, ConversationTurn, ConversationGoal, ConversationGoalType};
use super::conversation_storage::{ConversationStorage, Conversation, ConversationMessage, MessageRole};
use super::rule_based::{FinancialQueryIntent, ProcessedQuery};
use super::types::{NlpResponse, NlpResponseSource, NlpConfidenceLevel};
use crate::financial_advisor::nlp::{ConversationStorageConfig, StorageType};
use tempfile::tempdir;

/// Persistent conversation manager that uses the conversation storage
pub struct PersistentConversationManager {
    /// Conversation manager
    manager: ConversationManager,
    
    /// Conversation storage
    storage: Arc<ConversationStorage>,
    
    /// Whether the conversation has been persisted
    persisted: bool,
}

impl PersistentConversationManager {
    /// Create a new persistent conversation manager
    pub fn new(client_id: &str, storage: Arc<ConversationStorage>) -> Self {
        let manager = ConversationManager::new(client_id);
        
        Self {
            manager,
            storage,
            persisted: false,
        }
    }
    
    /// Load a conversation from storage
    pub async fn load(conversation_id: &str, storage: Arc<ConversationStorage>) -> Result<Self> {
        // Load the conversation from storage
        let stored_conversation = storage.get_conversation(conversation_id).await?;
        
        // Convert to ConversationManager
        let manager = Self::convert_from_stored(stored_conversation)?;
        
        Ok(Self {
            manager,
            storage,
            persisted: true,
        })
    }
    
    /// Convert a stored conversation to a ConversationManager
    fn convert_from_stored(stored: Conversation) -> Result<ConversationManager> {
        // Create a new conversation manager
        let mut manager = ConversationManager::new(&stored.user_id);
        
        // Set the ID to match the stored conversation
        manager.id = stored.id;
        
        // Convert messages to turns
        let mut turns = VecDeque::new();
        let mut current_query: Option<String> = None;
        let mut current_intent: Option<FinancialQueryIntent> = None;
        let mut current_intent_confidence: Option<f64> = None;
        
        // Group messages by user/assistant pairs
        for message in stored.messages {
            match message.role.as_str() {
                "user" => {
                    // If we already have a query, add the previous turn
                    if current_query.is_some() {
                        // Add the previous turn without a response
                        let turn = Self::create_turn(
                            current_query.take().unwrap(),
                            current_intent.take(),
                            current_intent_confidence.take(),
                            None,
                        );
                        turns.push_back(turn);
                    }
                    
                    // Store the current query
                    current_query = Some(message.content);
                    // Convert from Option<String> to Option<FinancialQueryIntent>
                    current_intent = message.intent.map(|intent_str| {
                        // Try to parse the string into FinancialQueryIntent or default to Unknown
                        match intent_str.as_str() {
                            "PortfolioPerformance" => FinancialQueryIntent::PortfolioPerformance,
                            "AssetAllocation" => FinancialQueryIntent::AssetAllocation,
                            "InvestmentRecommendation" => FinancialQueryIntent::InvestmentRecommendation,
                            "RiskAssessment" => FinancialQueryIntent::RiskAssessment,
                            "MarketInformation" => FinancialQueryIntent::MarketInformation,
                            "SustainableInvesting" => FinancialQueryIntent::SustainableInvesting,
                            "RetirementPlanning" => FinancialQueryIntent::RetirementPlanning,
                            "AccountInformation" => FinancialQueryIntent::AccountInformation,
                            "TaxOptimization" => FinancialQueryIntent::TaxOptimization,
                            "DebtManagement" => FinancialQueryIntent::DebtManagement,
                            "Greeting" => FinancialQueryIntent::Greeting,
                            "Farewell" => FinancialQueryIntent::Farewell,
                            "Help" => FinancialQueryIntent::Help,
                            _ => FinancialQueryIntent::Unknown,
                        }
                    });
                    current_intent_confidence = message.intent_confidence;
                }
                "assistant" => {
                    // If we have a query, add a turn with this response
                    if let Some(query) = current_query.take() {
                        // Create a response
                        let response = NlpResponse {
                            query: query.clone(),
                            intent: current_intent.take().unwrap_or(FinancialQueryIntent::Unknown),
                            confidence: current_intent_confidence.take().unwrap_or(0.0),
                            processed_query: None,
                            response_text: message.content,
                            source: NlpResponseSource::Hybrid,
                            confidence_level: NlpConfidenceLevel::from_score(current_intent_confidence.unwrap_or(0.0)),
                            is_uncertain: false,
                            explanation: None,
                        };
                        
                        // Add the turn
                        let turn = Self::create_turn(query, None, None, Some(response));
                        turns.push_back(turn);
                    } else {
                        // Orphaned assistant message, add it as a system message
                        let response = NlpResponse {
                            query: "".to_string(),
                            intent: FinancialQueryIntent::Unknown,
                            confidence: 0.0,
                            processed_query: None,
                            response_text: message.content,
                            source: NlpResponseSource::Hybrid,
                            confidence_level: NlpConfidenceLevel::Low,
                            is_uncertain: false,
                            explanation: None,
                        };
                        
                        // Add the turn with an empty query
                        let turn = Self::create_turn("".to_string(), None, None, Some(response));
                        turns.push_back(turn);
                    }
                }
                "system" => {
                    // System messages are ignored for now
                }
                _ => {
                    // Handle unknown role types
                }
            }
        }
        
        // If we still have a query without a response, add it
        if let Some(query) = current_query {
            let turn = Self::create_turn(
                query,
                current_intent,
                current_intent_confidence,
                None,
            );
            turns.push_back(turn);
        }
        
        // Set the history
        // Note: This is a private field, so we'd need to modify ConversationManager to expose a setter
        // For now, we'll recreate the history by adding each turn
        for turn in turns {
            let response_clone = turn.response.clone();
            manager.add_turn(turn.query.clone(), response_clone.unwrap_or_else(|| NlpResponse {
                query: turn.query.clone(),
                intent: FinancialQueryIntent::Unknown,
                confidence: 0.0,
                processed_query: None,
                response_text: "".to_string(),
                source: NlpResponseSource::Hybrid,
                confidence_level: NlpConfidenceLevel::Low,
                is_uncertain: false,
                explanation: None,
            }));
        }
        
        // Set client data if available
        if let Some(client_data) = stored.client_data {
            manager = manager.with_client_data(Arc::new(client_data));
        }
        
        // Set created_at and updated_at
        // Note: These are public fields, but they use chrono::DateTime<Utc>
        // We can just copy them directly since they're already in the right format
        let created_at = stored.created_at;
        let updated_at = stored.updated_at;
        
        manager.created_at = created_at;
        manager.updated_at = updated_at;
        
        Ok(manager)
    }
    
    /// Create a conversation turn
    fn create_turn(
        query: String,
        intent: Option<FinancialQueryIntent>,
        intent_confidence: Option<f64>,
        response: Option<NlpResponse>,
    ) -> ConversationTurn {
        let mut turn = ConversationTurn::new(query);
        
        // Set the processed query if we have an intent
        if let Some(intent) = intent {
            let processed_query = ProcessedQuery {
                original_text: turn.query.clone(),
                normalized_text: turn.query.to_lowercase(),
                intent,
                intent_confidence: intent_confidence.unwrap_or(0.0),
                entities: Vec::new(), // We don't store entities in the message
            };
            turn = turn.with_processed_query(processed_query);
        }
        
        // Set the response if available
        if let Some(response) = response {
            turn = turn.with_response(response);
        }
        
        turn
    }
    
    /// Convert the conversation manager to a stored conversation
    fn convert_to_stored(&self) -> Result<Conversation> {
        // Create messages from turns
        let mut messages = Vec::new();
        
        // Add tags based on intents
        let mut tags = Vec::new();
        
        for turn in self.manager.get_history() {
            if let Some(processed_query) = &turn.processed_query {
                let intent_tag = format!("intent:{}", processed_query.intent);
                if !tags.contains(&intent_tag) {
                    tags.push(intent_tag);
                }
            }
            
            // Add user message
            let user_message = ConversationMessage {
                id: Uuid::new_v4().to_string(),
                timestamp: chrono::DateTime::<Utc>::from_timestamp(
                    turn.timestamp.timestamp(),
                    0,
                ).unwrap_or_else(|| Utc::now()),
                role: MessageRole::User.to_string(),
                content: turn.query.clone(),
                intent: turn.processed_query.as_ref().map(|pq| pq.intent.to_string()),
                intent_confidence: turn.processed_query.as_ref().map(|pq| pq.intent_confidence),
                metadata: Some(HashMap::new()),
            };
            messages.push(user_message);
            
            // Add assistant message if available
            if let Some(response) = &turn.response {
                let assistant_message = ConversationMessage {
                    id: Uuid::new_v4().to_string(),
                    timestamp: chrono::DateTime::<Utc>::from_timestamp(
                        turn.timestamp.timestamp(),
                        0,
                    ).unwrap_or_else(|| Utc::now()),
                    role: MessageRole::Assistant.to_string(),
                    content: response.response_text.clone(),
                    intent: None,
                    intent_confidence: None,
                    metadata: Some(HashMap::new()),
                };
                messages.push(assistant_message);
            }
        }
        
        // Create a new conversation
        Ok(Conversation {
            id: self.manager.id.clone(),
            user_id: self.manager.client_id.clone(),
            client_data: None,
            messages,
            created_at: self.manager.created_at,
            updated_at: self.manager.updated_at,
            tags,
            summary: None,
            metadata: HashMap::new(),
        })
    }
    
    /// Save the conversation to storage
    pub async fn save(&mut self) -> Result<()> {
        // Convert to stored conversation
        let conversation = self.convert_to_stored()?;
        
        // Save to storage
        self.storage.save_conversation(&conversation).await?;
        
        // Mark as persisted
        self.persisted = true;
        
        Ok(())
    }
    
    /// Delete the conversation from storage
    pub async fn delete(&self) -> Result<()> {
        // Delete from storage
        self.storage.delete_conversation(&self.manager.id).await?;
        
        Ok(())
    }
    
    /// Get the conversation manager
    pub fn get_manager(&self) -> &ConversationManager {
        &self.manager
    }
    
    /// Get the conversation manager (mutable)
    pub fn get_manager_mut(&mut self) -> &mut ConversationManager {
        &mut self.manager
    }
    
    /// Add a user query to the conversation
    pub async fn add_user_query(&mut self, query: &str) -> Result<ConversationTurn> {
        // Add the query to the manager
        let turn = self.manager.add_user_query(query);
        let turn_clone = turn.clone();
        
        // Save the conversation
        let conversation = self.convert_to_stored()?;
        self.storage.save_conversation(&conversation).await?;
        self.persisted = true;
        
        Ok(turn_clone)
    }
    
    /// Update the current turn with a processed query
    pub async fn update_current_turn_with_processed_query(&mut self, processed_query: ProcessedQuery) -> Result<()> {
        // Update the manager
        self.manager.update_current_turn_with_processed_query(processed_query)?;
        
        // Save the conversation
        let conversation = self.convert_to_stored()?;
        self.storage.save_conversation(&conversation).await?;
        self.persisted = true;
        
        Ok(())
    }
    
    /// Update the current turn with a response
    pub async fn update_current_turn_with_response(&mut self, response: NlpResponse) -> Result<()> {
        // Update the manager
        self.manager.update_current_turn_with_response(response)?;
        
        // Save the conversation
        let conversation = self.convert_to_stored()?;
        self.storage.save_conversation(&conversation).await?;
        self.persisted = true;
        
        Ok(())
    }
    
    /// Add a goal to the conversation
    pub async fn add_goal(&mut self, goal_type: ConversationGoalType) -> Result<ConversationGoal> {
        // Add the goal to the manager
        let goal = self.manager.add_goal(goal_type);
        let goal_clone = goal.clone();
        
        // Save the conversation
        let conversation = self.convert_to_stored()?;
        self.storage.save_conversation(&conversation).await?;
        self.persisted = true;
        
        Ok(goal_clone)
    }
    
    /// List conversations for a user
    pub async fn list_conversations(&self, user_id: &str, limit: Option<i32>, _offset: Option<i32>) -> Result<Vec<Conversation>> {
        // List conversations from storage
        self.storage.list_conversations(user_id, limit).await
    }
    
    /// Search conversations
    pub async fn search_conversations(&self, query: &str, _limit: Option<usize>) -> Result<Vec<Conversation>> {
        // Search conversations from storage
        self.storage.search_conversations(query).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::conversation::context_manager::ContextManager;
    use std::path::PathBuf;
    use tempfile::tempdir;
    
    // A simple storage implementation for testing context managers
    struct TestContextManagerStorage {
        config: ConversationStorageConfig,
        managers: HashMap<String, ContextManager>,
    }
    
    impl TestContextManagerStorage {
        fn new(config: ConversationStorageConfig) -> Self {
            Self {
                config,
                managers: HashMap::new(),
            }
        }
        
        async fn save_context_manager(&self, id: &str, manager: &ContextManager) -> Result<()> {
            let mut managers = self.managers.clone();
            managers.insert(id.to_string(), manager.clone());
            Ok(())
        }
        
        async fn load_context_manager(&self, id: &str) -> Result<ContextManager> {
            self.managers.get(id)
                .cloned()
                .ok_or_else(|| anyhow::anyhow!("Context manager not found: {}", id))
        }
        
        async fn delete_context_manager(&self, id: &str) -> Result<()> {
            let mut managers = self.managers.clone();
            managers.remove(id);
            Ok(())
        }
    }
    
    #[tokio::test]
    async fn test_file_storage() {
        // Create a temporary directory for testing
        let temp_dir = tempdir().unwrap();
        let base_path = temp_dir.path().to_path_buf();
        
        // Create a storage config
        let config = ConversationStorageConfig {
            storage_type: StorageType::File,
            base_directory: Some(base_path.to_string_lossy().to_string()),
            database_url: None,
            dynamodb_tables: None,
            max_memory_conversations: 10,
            enable_summarization: false,
        };
        
        // Create a conversation manager storage
        let storage = TestContextManagerStorage::new(config);
        
        // Create a conversation manager
        let manager_id = "test-manager";
        
        // Create a context manager
        let context_manager = ContextManager::new();
        
        // Save the context manager
        storage.save_context_manager(manager_id, &context_manager).await.unwrap();
        
        // Load the context manager
        let loaded_manager = storage.load_context_manager(manager_id).await.unwrap();
        
        // Check that the loaded manager matches the original
        assert_eq!(loaded_manager.get_conversation_id(), context_manager.get_conversation_id());
        
        // Delete the context manager
        storage.delete_context_manager(manager_id).await.unwrap();
        
        // Check that the manager was deleted
        let result = storage.load_context_manager(manager_id).await;
        assert!(result.is_err());
    }
} 