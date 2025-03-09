use std::collections::{HashMap, VecDeque};
use std::cmp::Ordering;
use serde::{Serialize, Deserialize};
use uuid::Uuid;
use std::fs::{self, File};
use std::io::{BufReader, BufWriter};
use std::path::Path;

use crate::conversation::Message;
use crate::conversation::topic_detector::TopicDetector;
use crate::financial_entities::FinancialEntity;

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
    /// Topic detector for identifying topics in messages
    #[serde(skip)]
    topic_detector: Option<TopicDetector>,
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

impl ConversationState {
    /// Returns a description of the state
    pub fn description(&self) -> &str {
        match self {
            ConversationState::Introduction => "Initial greeting and purpose identification",
            ConversationState::InformationGathering => "Gathering information about the user's financial situation",
            ConversationState::TopicDiscussion => "Discussing specific financial topics",
            ConversationState::AdvisingPhase => "Providing recommendations or advice",
            ConversationState::QuestionAnswering => "Answering follow-up questions",
            ConversationState::Summarization => "Summarizing the conversation and next steps",
            ConversationState::Closing => "Closing the conversation",
        }
    }
    
    /// Returns the next logical state in the conversation flow
    pub fn next_state(&self) -> Self {
        match self {
            ConversationState::Introduction => ConversationState::InformationGathering,
            ConversationState::InformationGathering => ConversationState::TopicDiscussion,
            ConversationState::TopicDiscussion => ConversationState::AdvisingPhase,
            ConversationState::AdvisingPhase => ConversationState::QuestionAnswering,
            ConversationState::QuestionAnswering => ConversationState::Summarization,
            ConversationState::Summarization => ConversationState::Closing,
            ConversationState::Closing => ConversationState::Closing, // Terminal state
        }
    }
    
    /// Returns the previous logical state in the conversation flow
    pub fn previous_state(&self) -> Self {
        match self {
            ConversationState::Introduction => ConversationState::Introduction, // Initial state
            ConversationState::InformationGathering => ConversationState::Introduction,
            ConversationState::TopicDiscussion => ConversationState::InformationGathering,
            ConversationState::AdvisingPhase => ConversationState::TopicDiscussion,
            ConversationState::QuestionAnswering => ConversationState::AdvisingPhase,
            ConversationState::Summarization => ConversationState::QuestionAnswering,
            ConversationState::Closing => ConversationState::Summarization,
        }
    }
}

/// Represents a state transition in the conversation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateTransition {
    /// Unique identifier for the transition
    pub id: String,
    /// Message index where the transition occurred
    pub message_index: usize,
    /// Previous state
    pub from_state: ConversationState,
    /// New state
    pub to_state: ConversationState,
    /// Reason for the transition
    pub reason: String,
    /// Confidence score for the transition (0.0 to 1.0)
    pub confidence: f32,
    /// Timestamp when the transition occurred
    pub timestamp: chrono::DateTime<chrono::Utc>,
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
            topic_detector: None,
        }
    }
    
    /// Creates a new context manager with a topic detector
    pub fn with_topic_detector(topic_detector: TopicDetector) -> Self {
        Self {
            config: ContextManagerConfig::default(),
            active_messages: VecDeque::new(),
            segments: Vec::new(),
            topics: HashMap::new(),
            entities: HashMap::new(),
            summary: None,
            state: ConversationState::Introduction,
            total_message_count: 0,
            conversation_id: Uuid::new_v4().to_string(),
            topic_detector: Some(topic_detector),
        }
    }
    
    /// Creates a new context manager with custom settings and a topic detector
    pub fn with_config_and_topic_detector(config: ContextManagerConfig, topic_detector: TopicDetector) -> Self {
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
            topic_detector: Some(topic_detector),
        }
    }
    
    /// Set the topic detector
    pub fn set_topic_detector(&mut self, topic_detector: TopicDetector) {
        self.topic_detector = Some(topic_detector);
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
        
        // Use the topic detector if available
        if let Some(topic_detector) = &mut self.topic_detector {
            let detected_topics = topic_detector.detect_topics(&message);
            
            // Update our topics based on the detected topics
            for topic in detected_topics {
                self.topics.insert(topic.name.clone(), topic);
            }
        } else {
            // Fallback to basic topic detection
            self.update_topics_and_entities(&message);
        }
        
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
        scored_segments.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(Ordering::Equal));
        
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
        topics.sort_by(|a, b| b.calculate_overall_score().partial_cmp(&a.calculate_overall_score()).unwrap_or(Ordering::Equal));
        
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
        // There are several strategies for creating new segments:
        // 1. Topic shift detection - create a segment when there's a significant topic shift
        // 2. Time-based segmentation - create a segment after a certain amount of time
        // 3. Message count - create a segment after a certain number of messages
        // 4. Sentiment change - create a segment when the sentiment changes significantly
        
        let mut should_create_segment = false;
        
        // Check for topic shifts
        if let Some(topic_detector) = &self.topic_detector {
            // Get the latest topic shift
            if let Some(latest_shift) = topic_detector.get_latest_topic_shift() {
                // Check if this shift happened in the current message
                if latest_shift.message_index == topic_detector.get_message_counter() {
                    // This is a new shift, create a segment
                    should_create_segment = true;
                }
            }
        }
        
        // Message count-based segmentation (fallback)
        // Create a segment every 5 messages or if there's a topic shift
        if !should_create_segment && self.active_messages.len() >= 5 {
            if self.segments.is_empty() || self.segments.last().unwrap().messages.len() >= 5 {
                should_create_segment = true;
            }
        }
        
        // Create a new segment if needed
        if should_create_segment {
            // Determine how many messages to include in the segment
            // For topic shifts, include messages since the last segment
            // For message count-based segmentation, include the last 5 messages
            let message_count = if self.segments.is_empty() {
                self.active_messages.len().min(5)
            } else {
                let last_segment_size = self.segments.last().unwrap().messages.len();
                (self.active_messages.len() - last_segment_size).min(5)
            };
            
            let messages: Vec<Message> = self.active_messages.iter()
                .rev()
                .take(message_count)
                .cloned()
                .collect();
                
            let current_topics: Vec<String> = self.get_current_topics()
                .iter()
                .map(|t| t.name.clone())
                .collect();
                
            let current_entities = self.get_entities();
            
            // Create a new segment
            let mut segment = ContextSegment::new(
                messages.into_iter().rev().collect(), // Reverse back to chronological order
                current_topics,
                current_entities
            );
            
            // Set importance based on topics
            let topic_importance = self.get_current_topics()
                .iter()
                .map(|t| t.importance.to_score())
                .max_by(|a, b| a.partial_cmp(b).unwrap_or(Ordering::Equal))
                .unwrap_or(0.5);
                
            segment.importance_score = topic_importance;
            
            // Generate a summary for the segment if possible
            if let Some(summary) = self.generate_segment_summary(&segment) {
                segment.set_summary(summary);
            }
            
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
    
    /// Generates a summary for a segment
    fn generate_segment_summary(&self, segment: &ContextSegment) -> Option<String> {
        // In a real implementation, this would use an LLM to generate a summary
        // For now, we'll create a simple summary based on the topics and entities
        
        if segment.topics.is_empty() && segment.entities.is_empty() {
            return None;
        }
        
        let mut summary = String::new();
        
        // Add topics to the summary
        if !segment.topics.is_empty() {
            summary.push_str("Topics discussed: ");
            summary.push_str(&segment.topics.join(", "));
        }
        
        // Add entities to the summary
        if !segment.entities.is_empty() {
            if !summary.is_empty() {
                summary.push_str(". ");
            }
            
            summary.push_str("Entities mentioned: ");
            
            let entity_summaries: Vec<String> = segment.entities.iter()
                .map(|e| {
                    if let Some(value) = e.value {
                        format!("{} (${:.2})", e.name, value)
                    } else {
                        e.name.clone()
                    }
                })
                .collect();
                
            summary.push_str(&entity_summaries.join(", "));
        }
        
        // Add a period at the end if needed
        if !summary.is_empty() && !summary.ends_with('.') {
            summary.push('.');
        }
        
        Some(summary)
    }
    
    /// Gets the topic detector if available
    fn get_topic_detector(&self) -> Option<&TopicDetector> {
        self.topic_detector.as_ref()
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
            if let Some(_oldest) = self.active_messages.pop_front() {
                // No need to track tokens here, we'll recalculate in the next loop iteration
                // This is simpler than trying to maintain an accurate count inside the loop
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
    use crate::conversation::Message;
    
    #[test]
    fn test_context_manager_creation() {
        let manager = ContextManager::new();
        assert_eq!(manager.active_messages.len(), 0);
        assert_eq!(manager.segments.len(), 0);
    }
    
    #[test]
    fn test_add_message() {
        let mut manager = ContextManager::new();
        let message = Message {
            id: "test".to_string(),
            role: "user".to_string(),
            content: "Hello, I need financial advice".to_string(),
            timestamp: chrono::Utc::now(),
            metadata: std::collections::HashMap::new(),
        };
        
        manager.add_message(message);
        assert_eq!(manager.active_messages.len(), 1);
    }
    
    #[test]
    fn test_context_pruning() {
        let mut config = ContextManagerConfig::default();
        config.max_context_messages = 3;
        
        let mut manager = ContextManager::with_config(config);
        
        // Add 5 messages
        for i in 0..5 {
            let message = Message {
                id: format!("test_{}", i),
                role: "user".to_string(),
                content: format!("Message {}", i),
                timestamp: chrono::Utc::now(),
                metadata: std::collections::HashMap::new(),
            };
            
            manager.add_message(message);
        }
        
        // Should have pruned to 3 messages
        assert_eq!(manager.active_messages.len(), 3);
        
        // The oldest messages should be pruned
        assert_eq!(manager.active_messages[0].content, "Message 2");
    }
    
    #[test]
    fn test_importance_level_conversion() {
        assert_eq!(ImportanceLevel::Low.to_score(), 0.25);
        assert_eq!(ImportanceLevel::Medium.to_score(), 0.5);
        assert_eq!(ImportanceLevel::High.to_score(), 0.75);
        assert_eq!(ImportanceLevel::Critical.to_score(), 1.0);
        
        assert_eq!(ImportanceLevel::from_score(0.2), ImportanceLevel::Low);
        assert_eq!(ImportanceLevel::from_score(0.4), ImportanceLevel::Medium);
        assert_eq!(ImportanceLevel::from_score(0.7), ImportanceLevel::High);
        assert_eq!(ImportanceLevel::from_score(0.95), ImportanceLevel::Critical);
    }
    
    #[test]
    fn test_conversation_topic_scoring() {
        let topic = ConversationTopic::new(
            "Retirement", 
            "Planning for retirement", 
            ImportanceLevel::High, 
            5
        );
        
        assert!(topic.calculate_overall_score() > 0.0);
        assert!(topic.calculate_overall_score() <= 1.0);
    }
    
    #[test]
    fn test_context_persistence() {
        let mut manager = ContextManager::new();
        let message = Message {
            id: "test".to_string(),
            role: "user".to_string(),
            content: "Hello, I need financial advice".to_string(),
            timestamp: chrono::Utc::now(),
            metadata: std::collections::HashMap::new(),
        };
        
        manager.add_message(message);
        
        // Test persistence to a temporary file
        let temp_dir = std::env::temp_dir();
        let file_path = temp_dir.join("test_context.json");
        let path_str = file_path.to_str().unwrap();
        
        let result = manager.persist_to_file(path_str);
        assert!(result.is_ok());
        
        // Test loading from the file
        let loaded_result = ContextManager::load_from_file(path_str);
        assert!(loaded_result.is_ok());
        
        let loaded_manager = loaded_result.unwrap();
        assert_eq!(loaded_manager.active_messages.len(), 1);
        
        // Clean up
        let _ = std::fs::remove_file(file_path);
    }
} 