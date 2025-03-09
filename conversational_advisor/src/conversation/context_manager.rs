use std::collections::{HashMap, VecDeque};
use serde::{Serialize, Deserialize};
use crate::conversation::Message;
use crate::financial_entities::FinancialEntity;
use std::fs::{self, File};
use std::io::{BufReader, BufWriter};
use std::path::Path;
use uuid::Uuid;

/// Importance level for topics and context segments
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ImportanceLevel {
    Low,
    Medium,
    High,
    Critical,
}

impl ImportanceLevel {
    /// Convert importance level to a numeric score
    pub fn to_score(&self) -> f32 {
        match self {
            ImportanceLevel::Low => 0.25,
            ImportanceLevel::Medium => 0.5,
            ImportanceLevel::High => 0.75,
            ImportanceLevel::Critical => 1.0,
        }
    }
    
    /// Create from a numeric score
    pub fn from_score(score: f32) -> Self {
        if score < 0.3 {
            ImportanceLevel::Low
        } else if score < 0.6 {
            ImportanceLevel::Medium
        } else if score < 0.9 {
            ImportanceLevel::High
        } else {
            ImportanceLevel::Critical
        }
    }
}

/// Represents a topic in a conversation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversationTopic {
    /// Unique identifier for the topic
    pub id: String,
    /// Name of the topic
    pub name: String,
    /// Description of the topic
    pub description: String,
    /// Importance level of the topic
    pub importance: ImportanceLevel,
    /// When the topic was first mentioned (message index)
    pub first_mentioned: usize,
    /// When the topic was last mentioned (message index)
    pub last_mentioned: usize,
    /// Related financial entities
    pub related_entities: Vec<FinancialEntity>,
    /// Recency score (0.0 to 1.0) - higher means more recent
    pub recency_score: f32,
    /// Relevance score (0.0 to 1.0) - higher means more relevant to current context
    pub relevance_score: f32,
}

impl ConversationTopic {
    /// Create a new conversation topic
    pub fn new(name: &str, description: &str, importance: ImportanceLevel, message_index: usize) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            name: name.to_string(),
            description: description.to_string(),
            importance,
            first_mentioned: message_index,
            last_mentioned: message_index,
            related_entities: Vec::new(),
            recency_score: 1.0, // New topics start with maximum recency
            relevance_score: importance.to_score(), // Initial relevance based on importance
        }
    }
    
    /// Update the topic with a new mention
    pub fn update_mention(&mut self, message_index: usize, total_messages: usize) {
        self.last_mentioned = message_index;
        // Update recency score based on how recently it was mentioned
        if total_messages > 0 {
            self.recency_score = (message_index as f32) / (total_messages as f32);
        }
    }
    
    /// Add a related entity
    pub fn add_related_entity(&mut self, entity: FinancialEntity) {
        // Check if entity already exists
        if !self.related_entities.iter().any(|e| e.id == entity.id) {
            self.related_entities.push(entity);
        }
    }
    
    /// Calculate the overall importance score
    pub fn calculate_overall_score(&self) -> f32 {
        // Combine importance, recency, and relevance
        let importance_weight = 0.4;
        let recency_weight = 0.3;
        let relevance_weight = 0.3;
        
        (self.importance.to_score() * importance_weight) +
        (self.recency_score * recency_weight) +
        (self.relevance_score * relevance_weight)
    }
}

/// Represents a segment of the conversation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextSegment {
    /// Unique identifier for the segment
    pub id: String,
    /// Messages in this segment
    pub messages: Vec<Message>,
    /// Topics discussed in this segment
    pub topics: Vec<String>,
    /// Importance score (0.0 to 1.0)
    pub importance_score: f32,
    /// Financial entities mentioned in this segment
    pub entities: Vec<FinancialEntity>,
    /// Segment summary
    pub summary: Option<String>,
    /// Recency score (0.0 to 1.0) - higher means more recent
    pub recency_score: f32,
    /// Relevance score (0.0 to 1.0) - higher means more relevant to current context
    pub relevance_score: f32,
    /// Approximate token count for this segment
    pub token_count: usize,
}

impl ContextSegment {
    /// Create a new context segment
    pub fn new(messages: Vec<Message>, topics: Vec<String>, entities: Vec<FinancialEntity>) -> Self {
        let id = Uuid::new_v4().to_string();
        let token_count = Self::estimate_token_count(&messages);
        
        Self {
            id,
            messages,
            topics,
            importance_score: 0.5, // Default importance
            entities,
            summary: None,
            recency_score: 1.0, // New segments start with maximum recency
            relevance_score: 0.5, // Default relevance
            token_count,
        }
    }
    
    /// Estimate the token count for a set of messages
    fn estimate_token_count(messages: &[Message]) -> usize {
        // Simple estimation: 1 token ≈ 4 characters
        messages.iter()
            .map(|m| m.content.len() / 4 + 1)
            .sum()
    }
    
    /// Calculate the overall importance score
    pub fn calculate_overall_score(&self) -> f32 {
        // Combine importance, recency, and relevance
        let importance_weight = 0.4;
        let recency_weight = 0.3;
        let relevance_weight = 0.3;
        
        (self.importance_score * importance_weight) +
        (self.recency_score * recency_weight) +
        (self.relevance_score * relevance_weight)
    }
    
    /// Update the segment with a summary
    pub fn set_summary(&mut self, summary: String) {
        self.summary = Some(summary);
    }
    
    /// Update the segment's relevance score
    pub fn update_relevance(&mut self, current_topics: &[String], current_entities: &[FinancialEntity]) -> f32 {
        // Calculate topic overlap
        let topic_overlap = if !self.topics.is_empty() && !current_topics.is_empty() {
            let matching_topics = self.topics.iter()
                .filter(|t| current_topics.contains(t))
                .count();
            matching_topics as f32 / self.topics.len() as f32
        } else {
            0.0
        };
        
        // Calculate entity overlap
        let entity_overlap = if !self.entities.is_empty() && !current_entities.is_empty() {
            let matching_entities = self.entities.iter()
                .filter(|e1| current_entities.iter().any(|e2| e1.id == e2.id))
                .count();
            matching_entities as f32 / self.entities.len() as f32
        } else {
            0.0
        };
        
        // Update relevance score
        self.relevance_score = (topic_overlap * 0.6) + (entity_overlap * 0.4);
        self.relevance_score
    }
}

/// Configuration for the context manager
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextManagerConfig {
    /// Maximum number of messages to keep in active context
    pub max_context_messages: usize,
    /// Maximum number of tokens to keep in active context
    pub max_context_tokens: usize,
    /// Maximum number of segments to keep
    pub max_segments: usize,
    /// Minimum importance score for a segment to be included in relevant context
    pub min_segment_importance: f32,
    /// Path for persisting context (optional)
    pub persistence_path: Option<String>,
    /// Whether to automatically persist context on changes
    pub auto_persist: bool,
}

impl Default for ContextManagerConfig {
    fn default() -> Self {
        Self {
            max_context_messages: 20,
            max_context_tokens: 4000,
            max_segments: 10,
            min_segment_importance: 0.3,
            persistence_path: None,
            auto_persist: false,
        }
    }
}

/// Manages conversation context, including active messages, topics, and entities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextManager {
    /// Configuration for the context manager
    config: ContextManagerConfig,
    /// Active messages in the conversation (sliding window)
    active_messages: VecDeque<Message>,
    /// Segments of the conversation
    segments: Vec<ContextSegment>,
    /// Topics detected in the conversation
    topics: HashMap<String, ConversationTopic>,
    /// Financial entities mentioned in the conversation
    entities: HashMap<String, FinancialEntity>,
    /// Global conversation summary
    summary: Option<String>,
    /// Current conversation state
    state: ConversationState,
    /// Total message count (including pruned messages)
    total_message_count: usize,
    /// Conversation ID for persistence
    conversation_id: String,
}

/// State of the conversation
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConversationState {
    /// Initial greeting and purpose identification
    Introduction,
    /// Gathering information about the user's financial situation
    InformationGathering,
    /// Discussing specific financial topics
    TopicDiscussion,
    /// Providing recommendations or advice
    AdvisingPhase,
    /// Answering follow-up questions
    QuestionAnswering,
    /// Summarizing the conversation and next steps
    Summarization,
    /// Closing the conversation
    Closing,
}

impl ContextManager {
    /// Creates a new context manager with default settings
    pub fn new() -> Self {
        Self::with_config(ContextManagerConfig::default())
    }
    
    /// Creates a new context manager with custom settings
    pub fn with_config(config: ContextManagerConfig) -> Self {
        Self {
            config,
            active_messages: VecDeque::new(),
            segments: Vec::new(),
            topics: HashMap::new(),
            entities: HashMap::new(),
            summary: None,
            state: ConversationState::Introduction,
            total_message_count: 0,
            conversation_id: Uuid::new_v4().to_string(),
        }
    }
    
    /// Set the conversation ID
    pub fn set_conversation_id(&mut self, id: &str) {
        self.conversation_id = id.to_string();
    }
    
    /// Get the conversation ID
    pub fn get_conversation_id(&self) -> &str {
        &self.conversation_id
    }
    
    /// Adds a message to the context
    pub fn add_message(&mut self, message: Message) {
        // Add the message to the active context
        self.active_messages.push_back(message.clone());
        self.total_message_count += 1;
        
        // Update topics and entities based on the message
        self.update_topics_and_entities(&message);
        
        // Update the conversation state
        self.update_state(&message);
        
        // Check if we need to create a new segment
        self.check_and_create_segment();
        
        // Update relevance scores for all segments
        self.update_segment_relevance();
        
        // Prune the context if needed
        self.prune_context();
        
        // Auto-persist if enabled
        if self.config.auto_persist {
            if let Some(path) = &self.config.persistence_path {
                let _ = self.persist_to_file(path);
            }
        }
    }
    
    /// Gets the active context (all messages in the sliding window)
    pub fn get_active_context(&self) -> Vec<Message> {
        self.active_messages.iter().cloned().collect()
    }
    
    /// Gets the most relevant context for the current state
    pub fn get_relevant_context(&self) -> Vec<Message> {
        // If we have few messages, just return all of them
        if self.active_messages.len() <= self.config.max_context_messages / 2 {
            return self.get_active_context();
        }
        
        // Get current topics and entities
        let current_topics: Vec<String> = self.get_current_topics()
            .iter()
            .map(|t| t.name.clone())
            .collect();
            
        let current_entities: Vec<FinancialEntity> = self.get_entities();
        
        // Score segments by relevance to current context
        let mut scored_segments: Vec<(usize, f32)> = self.segments.iter()
            .enumerate()
            .map(|(i, segment)| {
                let mut segment_clone = segment.clone();
                let relevance = segment_clone.update_relevance(&current_topics, &current_entities);
                (i, segment.calculate_overall_score() * relevance)
            })
            .filter(|(_, score)| *score >= self.config.min_segment_importance)
            .collect();
        
        // Sort by score (descending)
        scored_segments.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        
        // Collect messages from top segments up to token limit
        let mut relevant_messages = Vec::new();
        let mut token_count = 0;
        
        // Always include the most recent messages
        let recent_messages: Vec<Message> = self.active_messages.iter()
            .rev()
            .take(5)
            .cloned()
            .collect();
            
        for msg in recent_messages.iter().rev() {
            relevant_messages.push(msg.clone());
            token_count += msg.content.len() / 4 + 1; // Simple token estimation
        }
        
        // Add messages from relevant segments
        for (segment_idx, _) in scored_segments {
            let segment = &self.segments[segment_idx];
            
            // Skip if adding this segment would exceed token limit
            if token_count + segment.token_count > self.config.max_context_tokens {
                continue;
            }
            
            // Add messages from this segment
            for msg in &segment.messages {
                // Skip if message is already included
                if !relevant_messages.iter().any(|m| m.id == msg.id) {
                    relevant_messages.push(msg.clone());
                    token_count += msg.content.len() / 4 + 1;
                }
            }
            
            // Stop if we've reached the token limit
            if token_count >= self.config.max_context_tokens {
                break;
            }
        }
        
        // Sort messages by timestamp
        relevant_messages.sort_by(|a, b| a.timestamp.cmp(&b.timestamp));
        
        relevant_messages
    }
    
    /// Gets the current topics
    pub fn get_current_topics(&self) -> Vec<ConversationTopic> {
        // Get all topics
        let mut topics: Vec<ConversationTopic> = self.topics.values().cloned().collect();
        
        // Sort by overall score (descending)
        topics.sort_by(|a, b| b.calculate_overall_score().partial_cmp(&a.calculate_overall_score()).unwrap_or(std::cmp::Ordering::Equal));
        
        // Return top topics
        topics
    }
    
    /// Gets all topics
    pub fn get_all_topics(&self) -> Vec<ConversationTopic> {
        self.topics.values().cloned().collect()
    }
    
    /// Gets all entities
    pub fn get_entities(&self) -> Vec<FinancialEntity> {
        self.entities.values().cloned().collect()
    }
    
    /// Gets the current conversation state
    pub fn get_state(&self) -> ConversationState {
        self.state.clone()
    }
    
    /// Sets the conversation state
    pub fn set_state(&mut self, state: ConversationState) {
        self.state = state;
    }
    
    /// Gets the conversation summary
    pub fn get_summary(&self) -> Option<String> {
        self.summary.clone()
    }
    
    /// Sets the conversation summary
    pub fn set_summary(&mut self, summary: String) {
        self.summary = Some(summary);
    }
    
    /// Updates topics and entities based on a message
    fn update_topics_and_entities(&mut self, _message: &Message) {
        // In a real implementation, this would use NLP to extract topics and entities
        // For now, this is a placeholder
        
        // Update recency scores for all topics
        for topic in self.topics.values_mut() {
            topic.recency_score *= 0.9; // Decay recency score for older topics
        }
    }
    
    /// Updates the conversation state based on a message
    fn update_state(&mut self, _message: &Message) {
        // In a real implementation, this would use the message content and context
        // to determine the appropriate state transition
        // For now, this is a placeholder
    }
    
    /// Checks if we need to create a new segment and does so if needed
    fn check_and_create_segment(&mut self) {
        // In a real implementation, this would use topic shifts, sentiment changes,
        // or other signals to determine when to create a new segment
        
        // For now, create a segment every 5 messages
        if self.active_messages.len() >= 5 && (self.segments.is_empty() || self.segments.last().unwrap().messages.len() >= 5) {
            let messages: Vec<Message> = self.active_messages.iter()
                .rev()
                .take(5)
                .cloned()
                .collect();
                
            let current_topics: Vec<String> = self.get_current_topics()
                .iter()
                .map(|t| t.name.clone())
                .collect();
                
            let current_entities = self.get_entities();
            
            let segment = ContextSegment::new(
                messages.into_iter().rev().collect(), // Reverse back to chronological order
                current_topics,
                current_entities
            );
            
            self.segments.push(segment);
            
            // Prune segments if needed
            while self.segments.len() > self.config.max_segments {
                // Remove the segment with the lowest overall score
                if self.segments.len() > 1 {
                    let mut lowest_idx = 0;
                    let mut lowest_score = self.segments[0].calculate_overall_score();
                    
                    for (i, segment) in self.segments.iter().enumerate().skip(1) {
                        let score = segment.calculate_overall_score();
                        if score < lowest_score {
                            lowest_score = score;
                            lowest_idx = i;
                        }
                    }
                    
                    self.segments.remove(lowest_idx);
                } else {
                    break;
                }
            }
        }
    }
    
    /// Update relevance scores for all segments
    fn update_segment_relevance(&mut self) {
        // Get current topics and entities
        let current_topics: Vec<String> = self.get_current_topics()
            .iter()
            .map(|t| t.name.clone())
            .collect();
            
        let current_entities = self.get_entities();
        
        // Update relevance for each segment
        for segment in &mut self.segments {
            segment.update_relevance(&current_topics, &current_entities);
        }
    }
    
    /// Prunes the context to stay within the maximum limits
    fn prune_context(&mut self) {
        // Remove messages if we exceed the maximum
        while self.active_messages.len() > self.config.max_context_messages {
            self.active_messages.pop_front();
        }
        
        // Calculate total tokens in active context
        let total_tokens: usize = self.active_messages.iter()
            .map(|m| m.content.len() / 4 + 1) // Simple token estimation
            .sum();
            
        // Remove oldest messages if we exceed token limit
        while total_tokens > self.config.max_context_tokens && !self.active_messages.is_empty() {
            if let Some(oldest) = self.active_messages.pop_front() {
                let tokens = oldest.content.len() / 4 + 1;
                // This is not entirely accurate as we're recalculating inside the loop,
                // but it's a simple approach for now
            }
        }
    }
    
    /// Persist the context manager to a file
    pub fn persist_to_file(&self, path: &str) -> Result<(), std::io::Error> {
        // Create directory if it doesn't exist
        if let Some(parent) = Path::new(path).parent() {
            fs::create_dir_all(parent)?;
        }
        
        // Open file for writing
        let file = File::create(path)?;
        let writer = BufWriter::new(file);
        
        // Serialize and write
        serde_json::to_writer(writer, self)?;
        
        Ok(())
    }
    
    /// Load the context manager from a file
    pub fn load_from_file(path: &str) -> Result<Self, std::io::Error> {
        // Open file for reading
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        
        // Deserialize
        let context_manager: Self = serde_json::from_reader(reader)?;
        
        Ok(context_manager)
    }
    
    /// Create a persistence path for a conversation
    pub fn create_persistence_path(base_dir: &str, conversation_id: &str) -> String {
        format!("{}/context_{}.json", base_dir, conversation_id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;
    
    #[test]
    fn test_context_manager_creation() {
        let manager = ContextManager::new();
        assert_eq!(manager.get_state(), ConversationState::Introduction);
        assert!(manager.get_active_context().is_empty());
    }
    
    #[test]
    fn test_add_message() {
        let mut manager = ContextManager::new();
        let message = Message::from_user("Hello");
        
        manager.add_message(message.clone());
        
        let active_context = manager.get_active_context();
        assert_eq!(active_context.len(), 1);
        assert_eq!(active_context[0].content, "Hello");
    }
    
    #[test]
    fn test_context_pruning() {
        let config = ContextManagerConfig {
            max_context_messages: 3,
            max_context_tokens: 1000,
            ..ContextManagerConfig::default()
        };
        let mut manager = ContextManager::with_config(config);
        
        // Add 5 messages
        for i in 0..5 {
            let message = Message::from_user(&format!("Message {}", i));
            manager.add_message(message);
        }
        
        // Should only keep the 3 most recent
        let active_context = manager.get_active_context();
        assert_eq!(active_context.len(), 3);
        assert_eq!(active_context[0].content, "Message 2");
        assert_eq!(active_context[1].content, "Message 3");
        assert_eq!(active_context[2].content, "Message 4");
    }
    
    #[test]
    fn test_importance_level_conversion() {
        assert_eq!(ImportanceLevel::Low.to_score(), 0.25);
        assert_eq!(ImportanceLevel::Medium.to_score(), 0.5);
        assert_eq!(ImportanceLevel::High.to_score(), 0.75);
        assert_eq!(ImportanceLevel::Critical.to_score(), 1.0);
        
        assert_eq!(ImportanceLevel::from_score(0.1), ImportanceLevel::Low);
        assert_eq!(ImportanceLevel::from_score(0.4), ImportanceLevel::Medium);
        assert_eq!(ImportanceLevel::from_score(0.7), ImportanceLevel::High);
        assert_eq!(ImportanceLevel::from_score(0.95), ImportanceLevel::Critical);
    }
    
    #[test]
    fn test_conversation_topic_scoring() {
        let mut topic = ConversationTopic::new(
            "Retirement", 
            "Planning for retirement", 
            ImportanceLevel::High, 
            5
        );
        
        // Initial score should be based on importance
        let initial_score = topic.calculate_overall_score();
        assert!(initial_score > 0.7);
        
        // Update with a new mention
        topic.update_mention(10, 20);
        
        // Score should reflect the update
        let updated_score = topic.calculate_overall_score();
        assert!(updated_score > 0.0);
    }
    
    #[test]
    fn test_context_persistence() {
        // Create a temporary directory
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("test_context.json").to_str().unwrap().to_string();
        
        // Create a context manager with persistence
        let config = ContextManagerConfig {
            persistence_path: Some(file_path.clone()),
            auto_persist: false,
            ..ContextManagerConfig::default()
        };
        let mut manager = ContextManager::with_config(config);
        
        // Add a message
        let message = Message::from_user("Test persistence");
        manager.add_message(message);
        
        // Persist to file
        manager.persist_to_file(&file_path).unwrap();
        
        // Load from file
        let loaded_manager = ContextManager::load_from_file(&file_path).unwrap();
        
        // Check that the loaded manager has the same message
        let loaded_context = loaded_manager.get_active_context();
        assert_eq!(loaded_context.len(), 1);
        assert_eq!(loaded_context[0].content, "Test persistence");
        
        // Clean up
        dir.close().unwrap();
    }
} 