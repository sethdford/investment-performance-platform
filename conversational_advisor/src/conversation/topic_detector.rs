use std::collections::HashMap;
use serde::{Serialize, Deserialize};
use crate::conversation::Message;
use crate::conversation::context_manager::{ConversationTopic, ImportanceLevel};

/// Financial topic categories
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum FinancialTopicCategory {
    RetirementPlanning,
    Investment,
    TaxPlanning,
    EstatePlanning,
    Insurance,
    Budgeting,
    DebtManagement,
    EducationPlanning,
    HomeOwnership,
    IncomeGeneration,
    CharitableGiving,
    HealthcarePlanning,
    RiskManagement,
    AssetAllocation,
    Other(String),
}

/// Configuration for the topic detector
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TopicDetectorConfig {
    /// Minimum confidence score to consider a topic detected (0.0 to 1.0)
    pub min_confidence: f32,
    /// Maximum number of topics to track
    pub max_topics: usize,
    /// Whether to use LLM for topic detection
    pub use_llm: bool,
    /// Whether to use rule-based detection as fallback
    pub use_rule_based_fallback: bool,
}

impl Default for TopicDetectorConfig {
    fn default() -> Self {
        Self {
            min_confidence: 0.6,
            max_topics: 10,
            use_llm: true,
            use_rule_based_fallback: true,
        }
    }
}

/// Detects and tracks financial topics in conversations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TopicDetector {
    /// Configuration for the topic detector
    config: TopicDetectorConfig,
    /// Known financial topics with their keywords
    topic_keywords: HashMap<FinancialTopicCategory, Vec<String>>,
    /// Currently active topics
    active_topics: HashMap<String, ConversationTopic>,
    /// Message counter for tracking when topics are mentioned
    message_counter: usize,
}

impl TopicDetector {
    /// Creates a new topic detector with default settings
    pub fn new() -> Self {
        let mut detector = Self {
            config: TopicDetectorConfig::default(),
            topic_keywords: HashMap::new(),
            active_topics: HashMap::new(),
            message_counter: 0,
        };
        
        detector.initialize_topic_keywords();
        detector
    }
    
    /// Creates a new topic detector with custom settings
    pub fn with_config(config: TopicDetectorConfig) -> Self {
        let mut detector = Self {
            config,
            topic_keywords: HashMap::new(),
            active_topics: HashMap::new(),
            message_counter: 0,
        };
        
        detector.initialize_topic_keywords();
        detector
    }
    
    /// Detects topics in a message
    pub fn detect_topics(&mut self, message: &Message) -> Vec<ConversationTopic> {
        self.message_counter += 1;
        
        let mut detected_topics = Vec::new();
        
        // Try LLM-based detection if enabled
        if self.config.use_llm {
            detected_topics = self.detect_topics_with_llm(message);
        }
        
        // Fall back to rule-based detection if needed
        if detected_topics.is_empty() && self.config.use_rule_based_fallback {
            detected_topics = self.detect_topics_rule_based(message);
        }
        
        // Update active topics
        for topic in detected_topics.clone() {
            self.update_active_topic(topic);
        }
        
        // Prune topics if we have too many
        self.prune_topics();
        
        detected_topics
    }
    
    /// Gets all active topics
    pub fn get_active_topics(&self) -> Vec<ConversationTopic> {
        self.active_topics.values().cloned().collect()
    }
    
    /// Gets topics by category
    pub fn get_topics_by_category(&self, category: &FinancialTopicCategory) -> Vec<ConversationTopic> {
        self.active_topics
            .values()
            .filter(|topic| {
                // Check if the topic name or description contains the category name
                let category_name = format!("{:?}", category);
                topic.name.contains(&category_name) || topic.description.contains(&category_name)
            })
            .cloned()
            .collect()
    }
    
    /// Initializes the topic keywords
    fn initialize_topic_keywords(&mut self) {
        // Retirement Planning
        self.topic_keywords.insert(
            FinancialTopicCategory::RetirementPlanning,
            vec![
                "retirement".to_string(),
                "401k".to_string(),
                "ira".to_string(),
                "pension".to_string(),
                "social security".to_string(),
                "retire".to_string(),
            ],
        );
        
        // Investment
        self.topic_keywords.insert(
            FinancialTopicCategory::Investment,
            vec![
                "invest".to_string(),
                "stock".to_string(),
                "bond".to_string(),
                "etf".to_string(),
                "mutual fund".to_string(),
                "portfolio".to_string(),
                "dividend".to_string(),
                "return".to_string(),
            ],
        );
        
        // Tax Planning
        self.topic_keywords.insert(
            FinancialTopicCategory::TaxPlanning,
            vec![
                "tax".to_string(),
                "deduction".to_string(),
                "credit".to_string(),
                "irs".to_string(),
                "income tax".to_string(),
                "capital gain".to_string(),
                "tax-loss harvesting".to_string(),
            ],
        );
        
        // Add more categories and keywords as needed
        // This is a simplified version for demonstration purposes
    }
    
    /// Detects topics using an LLM
    fn detect_topics_with_llm(&self, _message: &Message) -> Vec<ConversationTopic> {
        // In a real implementation, this would call an LLM to detect topics
        // For now, return an empty vector
        Vec::new()
    }
    
    /// Detects topics using rule-based methods
    fn detect_topics_rule_based(&self, message: &Message) -> Vec<ConversationTopic> {
        let mut detected_topics = Vec::new();
        
        // Check each category's keywords against the message content
        for (category, keywords) in &self.topic_keywords {
            for keyword in keywords {
                if message.content.to_lowercase().contains(&keyword.to_lowercase()) {
                    // Create a topic for this category
                    let topic_id = uuid::Uuid::new_v4().to_string();
                    let category_name = format!("{:?}", category);
                    
                    let topic = ConversationTopic {
                        id: topic_id,
                        name: category_name.clone(),
                        description: format!("Discussion about {}", category_name.to_lowercase()),
                        importance: ImportanceLevel::Medium, // Default importance
                        first_mentioned: self.message_counter,
                        last_mentioned: self.message_counter,
                        related_entities: Vec::new(),
                    };
                    
                    detected_topics.push(topic);
                    break; // Only add each category once
                }
            }
        }
        
        detected_topics
    }
    
    /// Updates an active topic or adds it if it's new
    fn update_active_topic(&mut self, topic: ConversationTopic) {
        for (_id, existing_topic) in &mut self.active_topics {
            // If we already have this topic category, update it
            if existing_topic.name == topic.name {
                // Update last mentioned
                existing_topic.last_mentioned = self.message_counter;
                
                // Merge related entities
                for entity in topic.related_entities {
                    if !existing_topic.related_entities.iter().any(|e| e.id == entity.id) {
                        existing_topic.related_entities.push(entity);
                    }
                }
                
                // Potentially update importance based on frequency or recency
                if topic.importance as u8 > existing_topic.importance.clone() as u8 {
                    existing_topic.importance = topic.importance.clone();
                }
                
                return;
            }
        }
        
        // If we get here, it's a new topic
        self.active_topics.insert(topic.id.clone(), topic);
    }
    
    /// Prunes topics to stay within the maximum limit
    fn prune_topics(&mut self) {
        if self.active_topics.len() <= self.config.max_topics {
            return;
        }
        
        // Calculate a score for each topic based on importance, recency, and frequency
        let mut topic_scores: Vec<(String, f32)> = self.active_topics
            .iter()
            .map(|(id, topic)| {
                let importance_score = match topic.importance {
                    ImportanceLevel::Low => 0.25,
                    ImportanceLevel::Medium => 0.5,
                    ImportanceLevel::High => 0.75,
                    ImportanceLevel::Critical => 1.0,
                };
                
                let recency_score = 1.0 - (self.message_counter - topic.last_mentioned) as f32 
                    / self.message_counter as f32;
                
                let frequency_score = (topic.last_mentioned - topic.first_mentioned) as f32 
                    / self.message_counter as f32;
                
                let score = importance_score * 0.5 + recency_score * 0.3 + frequency_score * 0.2;
                
                (id.clone(), score)
            })
            .collect();
        
        // Sort by score (descending)
        topic_scores.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        
        // Keep only the top N topics
        let topics_to_keep: Vec<String> = topic_scores
            .iter()
            .take(self.config.max_topics)
            .map(|(id, _)| id.clone())
            .collect();
        
        // Remove topics not in the keep list
        self.active_topics.retain(|id, _| topics_to_keep.contains(id));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    fn create_test_message(content: &str) -> Message {
        Message {
            id: uuid::Uuid::new_v4().to_string(),
            role: "user".to_string(),
            content: content.to_string(),
            timestamp: chrono::Utc::now(),
            metadata: HashMap::new(),
        }
    }
    
    #[test]
    fn test_topic_detector_creation() {
        let detector = TopicDetector::new();
        assert_eq!(detector.get_active_topics().len(), 0);
        assert!(detector.topic_keywords.contains_key(&FinancialTopicCategory::RetirementPlanning));
        assert!(detector.topic_keywords.contains_key(&FinancialTopicCategory::Investment));
    }
    
    #[test]
    fn test_rule_based_detection() {
        let mut detector = TopicDetector::new();
        
        // Test retirement topic
        let message = create_test_message("I want to plan for my retirement and set up a 401k");
        let topics = detector.detect_topics(&message);
        
        assert!(!topics.is_empty());
        assert_eq!(topics[0].name, "RetirementPlanning");
        
        // Test investment topic
        let message = create_test_message("I'm interested in investing in stocks and bonds");
        let topics = detector.detect_topics(&message);
        
        assert!(!topics.is_empty());
        assert_eq!(topics[0].name, "Investment");
        
        // Test multiple topics in one message
        let message = create_test_message("I want to invest my retirement funds in tax-efficient ways");
        let topics = detector.detect_topics(&message);
        
        // Should detect at least one topic
        assert!(!topics.is_empty());
    }
    
    #[test]
    fn test_topic_pruning() {
        let mut detector = TopicDetector::with_config(TopicDetectorConfig {
            max_topics: 2,
            ..TopicDetectorConfig::default()
        });
        
        // Add 3 topics
        detector.detect_topics(&create_test_message("I want to plan for retirement"));
        detector.detect_topics(&create_test_message("I need help with my taxes"));
        detector.detect_topics(&create_test_message("I want to invest in stocks"));
        
        // Should only keep 2 topics
        assert_eq!(detector.get_active_topics().len(), 2);
    }
} 