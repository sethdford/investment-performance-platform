use std::collections::{HashMap, VecDeque};
use std::cmp::Ordering;
use uuid::Uuid;
use chrono::Utc;
use serde::{Serialize, Deserialize};

use crate::conversation::Message;
use crate::conversation::context_manager::{ConversationTopic, ImportanceLevel};

/// Categories of financial topics
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

impl FinancialTopicCategory {
    /// Get a display name for the category
    pub fn display_name(&self) -> String {
        match self {
            Self::RetirementPlanning => "Retirement Planning".to_string(),
            Self::Investment => "Investment".to_string(),
            Self::TaxPlanning => "Tax Planning".to_string(),
            Self::EstatePlanning => "Estate Planning".to_string(),
            Self::Insurance => "Insurance".to_string(),
            Self::Budgeting => "Budgeting".to_string(),
            Self::DebtManagement => "Debt Management".to_string(),
            Self::EducationPlanning => "Education Planning".to_string(),
            Self::HomeOwnership => "Home Ownership".to_string(),
            Self::IncomeGeneration => "Income Generation".to_string(),
            Self::CharitableGiving => "Charitable Giving".to_string(),
            Self::HealthcarePlanning => "Healthcare Planning".to_string(),
            Self::RiskManagement => "Risk Management".to_string(),
            Self::AssetAllocation => "Asset Allocation".to_string(),
            Self::Other(name) => name.clone(),
        }
    }
    
    /// Get a description for the category
    pub fn description(&self) -> String {
        match self {
            Self::RetirementPlanning => "Planning for retirement and retirement accounts".to_string(),
            Self::Investment => "Investment strategies and options".to_string(),
            Self::TaxPlanning => "Tax strategies and implications".to_string(),
            Self::EstatePlanning => "Estate planning and inheritance".to_string(),
            Self::Insurance => "Insurance policies and coverage".to_string(),
            Self::Budgeting => "Budgeting and expense management".to_string(),
            Self::DebtManagement => "Debt management and loans".to_string(),
            Self::EducationPlanning => "Education funding and planning".to_string(),
            Self::HomeOwnership => "Home buying, mortgages, and ownership".to_string(),
            Self::IncomeGeneration => "Income sources and generation".to_string(),
            Self::CharitableGiving => "Charitable donations and giving strategies".to_string(),
            Self::HealthcarePlanning => "Healthcare costs and planning".to_string(),
            Self::RiskManagement => "Financial risk management".to_string(),
            Self::AssetAllocation => "Asset allocation and portfolio management".to_string(),
            Self::Other(name) => format!("Discussion about {}", name),
        }
    }
    
    /// Get the default importance level for this category
    pub fn default_importance(&self) -> ImportanceLevel {
        match self {
            Self::RetirementPlanning => ImportanceLevel::High,
            Self::Investment => ImportanceLevel::High,
            Self::TaxPlanning => ImportanceLevel::High,
            Self::EstatePlanning => ImportanceLevel::Medium,
            Self::Insurance => ImportanceLevel::Medium,
            Self::Budgeting => ImportanceLevel::Medium,
            Self::DebtManagement => ImportanceLevel::High,
            Self::EducationPlanning => ImportanceLevel::Medium,
            Self::HomeOwnership => ImportanceLevel::Medium,
            Self::IncomeGeneration => ImportanceLevel::High,
            Self::CharitableGiving => ImportanceLevel::Low,
            Self::HealthcarePlanning => ImportanceLevel::Medium,
            Self::RiskManagement => ImportanceLevel::Medium,
            Self::AssetAllocation => ImportanceLevel::High,
            Self::Other(_) => ImportanceLevel::Low,
        }
    }
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
    /// Threshold for detecting a topic shift (0.0 to 1.0)
    pub topic_shift_threshold: f32,
    /// Number of messages to consider for topic shift detection
    pub topic_shift_window: usize,
    /// Minimum message length for topic detection
    pub min_message_length: usize,
}

impl Default for TopicDetectorConfig {
    fn default() -> Self {
        Self {
            min_confidence: 0.3,
            max_topics: 10,
            use_llm: false,
            use_rule_based_fallback: true,
            topic_shift_threshold: 0.5,
            topic_shift_window: 5,
            min_message_length: 10,
        }
    }
}

/// Represents a shift in conversation topics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TopicShift {
    /// Unique identifier for the topic shift
    pub id: String,
    /// Message index where the shift occurred
    pub message_index: usize,
    /// Previous dominant topic
    pub previous_topic: Option<String>,
    /// New dominant topic
    pub new_topic: String,
    /// Confidence score for the shift detection (0.0 to 1.0)
    pub confidence: f32,
    /// Timestamp when the shift was detected
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// Detects and tracks topics in a conversation
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
    /// Recent topic distributions for detecting shifts
    recent_topic_distributions: VecDeque<HashMap<String, f32>>,
    /// Detected topic shifts
    topic_shifts: Vec<TopicShift>,
}

impl TopicDetector {
    /// Creates a new topic detector with default settings
    pub fn new() -> Self {
        let config = TopicDetectorConfig::default();
        let topic_shift_window = config.topic_shift_window;
        
        let mut detector = Self {
            config,
            topic_keywords: HashMap::new(),
            active_topics: HashMap::new(),
            message_counter: 0,
            recent_topic_distributions: VecDeque::with_capacity(topic_shift_window),
            topic_shifts: Vec::new(),
        };
        
        detector.initialize_topic_keywords();
        detector
    }
    
    /// Creates a new topic detector with custom settings
    pub fn with_config(config: TopicDetectorConfig) -> Self {
        let topic_shift_window = config.topic_shift_window;
        
        let mut detector = Self {
            config,
            topic_keywords: HashMap::new(),
            active_topics: HashMap::new(),
            message_counter: 0,
            recent_topic_distributions: VecDeque::with_capacity(topic_shift_window),
            topic_shifts: Vec::new(),
        };
        
        detector.initialize_topic_keywords();
        detector
    }
    
    /// Detect topics in a message
    pub fn detect_topics(&mut self, message: &Message) -> Vec<ConversationTopic> {
        // Increment message counter
        self.message_counter += 1;
        
        // Skip topic detection for system messages
        if message.role == "system" {
            return Vec::new();
        }
        
        // Skip if message is too short
        if message.content.len() < self.config.min_message_length {
            return Vec::new();
        }
        
        // Detect topics using rule-based detection
        // In the future, we can add LLM-based detection when async support is available
        let detected_topics = self.detect_topics_rule_based(message);
        
        // Update active topics
        for topic in &detected_topics {
            self.update_active_topic(topic.clone());
        }
        
        // Update topic distributions for shift detection
        self.update_topic_distributions(&detected_topics);
        
        // Detect topic shifts
        self.detect_topic_shift();
        
        // Prune topics if needed
        self.prune_topics();
        
        detected_topics
    }
    
    /// Gets all active topics
    pub fn get_active_topics(&self) -> Vec<ConversationTopic> {
        let mut topics: Vec<ConversationTopic> = self.active_topics.values().cloned().collect();
        self.sort_topics_by_importance(&mut topics);
        topics
    }
    
    /// Gets topics by category
    pub fn get_topics_by_category(&self, category: &FinancialTopicCategory) -> Vec<ConversationTopic> {
        let category_name = category.display_name();
        self.active_topics.values()
            .filter(|t| t.name == category_name)
            .cloned()
            .collect()
    }
    
    /// Gets all detected topic shifts
    pub fn get_topic_shifts(&self) -> &[TopicShift] {
        &self.topic_shifts
    }
    
    /// Gets the latest topic shift
    pub fn get_latest_topic_shift(&self) -> Option<&TopicShift> {
        self.topic_shifts.last()
    }
    
    /// Gets the current message counter
    pub fn get_message_counter(&self) -> usize {
        self.message_counter
    }
    
    /// Sorts topics by importance (descending)
    fn sort_topics_by_importance(&self, topics: &mut [ConversationTopic]) {
        topics.sort_by(|a, b| {
            let a_score = a.calculate_overall_score();
            let b_score = b.calculate_overall_score();
            b_score.partial_cmp(&a_score).unwrap_or(Ordering::Equal)
        });
    }
    
    /// Initializes the known topic keywords
    fn initialize_topic_keywords(&mut self) {
        // Retirement planning keywords
        self.topic_keywords.insert(
            FinancialTopicCategory::RetirementPlanning,
            vec![
                "retirement".to_string(),
                "401k".to_string(),
                "ira".to_string(),
                "pension".to_string(),
                "social security".to_string(),
                "retire".to_string(),
                "retirement account".to_string(),
                "retirement plan".to_string(),
                "roth".to_string(),
                "traditional ira".to_string(),
                "401(k)".to_string(),
                "403(b)".to_string(),
                "sep ira".to_string(),
                "simple ira".to_string(),
                "annuity".to_string(),
            ]
        );
        
        // Investment keywords
        self.topic_keywords.insert(
            FinancialTopicCategory::Investment,
            vec![
                "invest".to_string(),
                "investment".to_string(),
                "stock".to_string(),
                "bond".to_string(),
                "etf".to_string(),
                "mutual fund".to_string(),
                "portfolio".to_string(),
                "dividend".to_string(),
                "return".to_string(),
                "market".to_string(),
                "equity".to_string(),
                "securities".to_string(),
                "index fund".to_string(),
                "growth".to_string(),
                "value".to_string(),
                "capital gain".to_string(),
                "yield".to_string(),
                "diversification".to_string(),
            ]
        );
        
        // Tax planning keywords
        self.topic_keywords.insert(
            FinancialTopicCategory::TaxPlanning,
            vec![
                "tax".to_string(),
                "taxes".to_string(),
                "deduction".to_string(),
                "credit".to_string(),
                "irs".to_string(),
                "capital gain".to_string(),
                "tax-loss".to_string(),
                "tax bracket".to_string(),
                "tax return".to_string(),
                "tax planning".to_string(),
                "tax-advantaged".to_string(),
                "tax-deferred".to_string(),
                "tax-exempt".to_string(),
                "tax-free".to_string(),
                "withholding".to_string(),
                "estimated tax".to_string(),
            ]
        );
        
        // Estate planning keywords
        self.topic_keywords.insert(
            FinancialTopicCategory::EstatePlanning,
            vec![
                "estate".to_string(),
                "will".to_string(),
                "trust".to_string(),
                "inheritance".to_string(),
                "beneficiary".to_string(),
                "legacy".to_string(),
                "estate tax".to_string(),
                "probate".to_string(),
                "executor".to_string(),
                "power of attorney".to_string(),
                "living will".to_string(),
                "healthcare directive".to_string(),
                "gift tax".to_string(),
                "heir".to_string(),
            ]
        );
        
        // Insurance keywords
        self.topic_keywords.insert(
            FinancialTopicCategory::Insurance,
            vec![
                "insurance".to_string(),
                "coverage".to_string(),
                "premium".to_string(),
                "policy".to_string(),
                "life insurance".to_string(),
                "health insurance".to_string(),
                "disability insurance".to_string(),
                "long-term care".to_string(),
                "auto insurance".to_string(),
                "home insurance".to_string(),
                "umbrella policy".to_string(),
                "deductible".to_string(),
                "claim".to_string(),
                "beneficiary".to_string(),
                "term life".to_string(),
                "whole life".to_string(),
            ]
        );
        
        // Budgeting keywords
        self.topic_keywords.insert(
            FinancialTopicCategory::Budgeting,
            vec![
                "budget".to_string(),
                "saving".to_string(),
                "expense".to_string(),
                "spend".to_string(),
                "income".to_string(),
                "emergency fund".to_string(),
                "cash flow".to_string(),
                "financial plan".to_string(),
                "spending plan".to_string(),
                "track expenses".to_string(),
                "discretionary".to_string(),
                "necessary expenses".to_string(),
                "fixed expenses".to_string(),
                "variable expenses".to_string(),
                "zero-based budget".to_string(),
            ]
        );
        
        // Debt management keywords
        self.topic_keywords.insert(
            FinancialTopicCategory::DebtManagement,
            vec![
                "debt".to_string(),
                "loan".to_string(),
                "mortgage".to_string(),
                "credit card".to_string(),
                "interest rate".to_string(),
                "refinance".to_string(),
                "student loan".to_string(),
                "consolidation".to_string(),
                "debt snowball".to_string(),
                "debt avalanche".to_string(),
                "balance transfer".to_string(),
                "principal".to_string(),
                "amortization".to_string(),
                "credit score".to_string(),
                "credit report".to_string(),
                "debt-to-income".to_string(),
            ]
        );
        
        // Education planning keywords
        self.topic_keywords.insert(
            FinancialTopicCategory::EducationPlanning,
            vec![
                "college".to_string(),
                "education".to_string(),
                "tuition".to_string(),
                "student loan".to_string(),
                "529 plan".to_string(),
                "coverdell".to_string(),
                "financial aid".to_string(),
                "scholarship".to_string(),
                "grant".to_string(),
                "fafsa".to_string(),
                "student debt".to_string(),
                "college fund".to_string(),
                "education savings".to_string(),
                "college planning".to_string(),
            ]
        );
        
        // Home ownership keywords
        self.topic_keywords.insert(
            FinancialTopicCategory::HomeOwnership,
            vec![
                "home".to_string(),
                "house".to_string(),
                "mortgage".to_string(),
                "property".to_string(),
                "real estate".to_string(),
                "down payment".to_string(),
                "closing costs".to_string(),
                "refinance".to_string(),
                "equity".to_string(),
                "home equity".to_string(),
                "heloc".to_string(),
                "property tax".to_string(),
                "homeowner's insurance".to_string(),
                "pmi".to_string(),
                "escrow".to_string(),
            ]
        );
        
        // Income generation keywords
        self.topic_keywords.insert(
            FinancialTopicCategory::IncomeGeneration,
            vec![
                "income".to_string(),
                "salary".to_string(),
                "wage".to_string(),
                "earn".to_string(),
                "passive income".to_string(),
                "side hustle".to_string(),
                "business".to_string(),
                "self-employed".to_string(),
                "freelance".to_string(),
                "rental income".to_string(),
                "dividend income".to_string(),
                "interest income".to_string(),
                "royalty".to_string(),
                "commission".to_string(),
                "bonus".to_string(),
            ]
        );
        
        // Add more categories as needed
    }
    
    /// Detects topics using rule-based keyword matching
    fn detect_topics_rule_based(&self, message: &Message) -> Vec<ConversationTopic> {
        // Skip topic detection for system messages
        if message.role == "system" {
            return Vec::new();
        }
        
        // Skip if message is too short
        if message.content.len() < self.config.min_message_length {
            return Vec::new();
        }
        
        let content = message.content.to_lowercase();
        let mut detected_topics = Vec::new();
        let message_index = self.message_counter;
        
        // Check for each known financial topic
        for (topic_category, keywords) in &self.topic_keywords {
            // Check if any keyword is present in the message
            let matches: Vec<&str> = keywords.iter()
                .filter(|&k| content.contains(k))
                .map(|s| s.as_str())
                .collect();
                
            if !matches.is_empty() {
                // Determine importance based on number of matches and specific keywords
                let importance = if matches.len() >= 3 {
                    ImportanceLevel::High
                } else if matches.len() >= 2 {
                    ImportanceLevel::Medium
                } else {
                    ImportanceLevel::Low
                };
                
                // Create a description based on the topic
                let topic_name = topic_category.display_name();
                let description = topic_category.description();
                
                // Create a new topic
                let mut topic = ConversationTopic::new(
                    &topic_name,
                    &description,
                    importance,
                    message_index
                );
                
                // Set relevance based on number of matches
                topic.relevance_score = 0.5 + (matches.len() as f32 * 0.1).min(0.5);
                topic.recency_score = 1.0;
                
                detected_topics.push(topic);
            }
        }
        
        detected_topics
    }
    
    /// Updates an active topic or adds a new one
    fn update_active_topic(&mut self, topic: ConversationTopic) {
        if let Some(existing_topic) = self.active_topics.get_mut(&topic.name) {
            // Update the existing topic
            existing_topic.update_mention(self.message_counter, self.message_counter);
            
            // Update importance if the new topic has higher importance
            if topic.importance.to_score() > existing_topic.importance.to_score() {
                existing_topic.importance = topic.importance;
            }
            
            // Update relevance score if the new one is higher
            if topic.relevance_score > existing_topic.relevance_score {
                existing_topic.relevance_score = topic.relevance_score;
            }
            
            // Add any new related entities
            for entity in topic.related_entities {
                if !existing_topic.related_entities.iter().any(|e| e.id == entity.id) {
                    existing_topic.related_entities.push(entity);
                }
            }
        } else {
            // Add the new topic
            self.active_topics.insert(topic.name.clone(), topic);
            
            // Prune topics if we exceed the maximum
            self.prune_topics();
        }
    }
    
    /// Updates the recent topic distributions based on detected topics
    fn update_topic_distributions(&mut self, detected_topics: &[ConversationTopic]) {
        // Create a new distribution
        let mut distribution = HashMap::new();
        
        // Add scores for each detected topic
        for topic in detected_topics {
            let score = topic.calculate_overall_score();
            distribution.insert(topic.name.clone(), score);
        }
        
        // Add scores for active topics that weren't detected in this message
        for (name, topic) in &self.active_topics {
            if !distribution.contains_key(name) {
                // Add with a decayed score
                let decayed_score = topic.calculate_overall_score() * 0.8;
                if decayed_score >= self.config.min_confidence {
                    distribution.insert(name.clone(), decayed_score);
                }
            }
        }
        
        // Add the distribution to the recent list
        self.recent_topic_distributions.push_back(distribution);
        
        // Prune if needed
        while self.recent_topic_distributions.len() > self.config.topic_shift_window {
            self.recent_topic_distributions.pop_front();
        }
    }
    
    /// Detects topic shifts in the conversation
    fn detect_topic_shift(&mut self) {
        // Need at least 2 distributions to detect a shift
        if self.recent_topic_distributions.len() < 2 {
            return;
        }
        
        // Get the current and previous distributions
        let current_dist = self.recent_topic_distributions.back().unwrap();
        let prev_dist = self.recent_topic_distributions.iter().rev().nth(1).unwrap();
        
        // Find the dominant topic in each distribution
        let current_dominant = current_dist.iter()
            .max_by(|a, b| a.1.partial_cmp(b.1).unwrap_or(Ordering::Equal))
            .map(|(k, _)| k.clone());
            
        let prev_dominant = prev_dist.iter()
            .max_by(|a, b| a.1.partial_cmp(b.1).unwrap_or(Ordering::Equal))
            .map(|(k, _)| k.clone());
        
        // Check if the dominant topic has changed
        if let (Some(current), Some(prev)) = (&current_dominant, &prev_dominant) {
            if current != prev {
                // Calculate the confidence of the shift
                let current_score = current_dist.get(current).unwrap_or(&0.0);
                let prev_score = prev_dist.get(prev).unwrap_or(&0.0);
                
                // Only consider it a shift if the scores are significant
                if *current_score >= self.config.min_confidence && *prev_score >= self.config.min_confidence {
                    // Create a new topic shift
                    let shift = TopicShift {
                        id: Uuid::new_v4().to_string(),
                        message_index: self.message_counter,
                        previous_topic: Some(prev.clone()),
                        new_topic: current.clone(),
                        confidence: (*current_score + *prev_score) / 2.0,
                        timestamp: Utc::now(),
                    };
                    
                    self.topic_shifts.push(shift);
                }
            }
        } else if let Some(current) = current_dominant {
            // First dominant topic (no previous)
            let current_score = current_dist.get(&current).unwrap_or(&0.0);
            
            if *current_score >= self.config.min_confidence {
                // Create a new topic shift
                let shift = TopicShift {
                    id: Uuid::new_v4().to_string(),
                    message_index: self.message_counter,
                    previous_topic: None,
                    new_topic: current.clone(),
                    confidence: *current_score,
                    timestamp: Utc::now(),
                };
                
                self.topic_shifts.push(shift);
            }
        }
    }
    
    /// Prunes topics if we exceed the maximum
    fn prune_topics(&mut self) {
        if self.active_topics.len() <= self.config.max_topics {
            return;
        }
        
        // Get all topics sorted by overall score (descending)
        let mut topics: Vec<(String, f32)> = self.active_topics.iter()
            .map(|(name, topic)| (name.clone(), topic.calculate_overall_score()))
            .collect();
            
        topics.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(Ordering::Equal));
        
        // Keep only the top N topics
        let topics_to_keep: Vec<String> = topics.iter()
            .take(self.config.max_topics)
            .map(|(name, _)| name.clone())
            .collect();
            
        // Remove topics that aren't in the keep list
        self.active_topics.retain(|name, _| topics_to_keep.contains(name));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::conversation::Message;
    
    fn create_test_message(content: &str) -> Message {
        Message {
            id: "test".to_string(),
            role: "user".to_string(),
            content: content.to_string(),
            timestamp: Utc::now(),
            metadata: std::collections::HashMap::new(),
        }
    }
    
    #[test]
    fn test_topic_detector_creation() {
        let detector = TopicDetector::new();
        assert_eq!(detector.config.max_topics, 10);
        assert!(!detector.topic_keywords.is_empty());
    }
    
    #[test]
    fn test_rule_based_detection() {
        let mut detector = TopicDetector::new();
        
        // Test retirement detection
        let message = create_test_message("I want to plan for my retirement with a 401k");
        
        let topics = detector.detect_topics(&message);
        
        assert!(!topics.is_empty());
        assert!(topics.iter().any(|t| t.name.contains("Retirement")));
        
        // Test investment detection
        let message = create_test_message("I want to invest in stocks and bonds");
        
        let topics = detector.detect_topics(&message);
        
        assert!(!topics.is_empty());
        assert!(topics.iter().any(|t| t.name.contains("Investment")));
    }
    
    #[test]
    fn test_topic_shift_detection() {
        let mut detector = TopicDetector::new();
        
        // Start with retirement topic
        let message1 = create_test_message("I want to plan for my retirement with a 401k");
        detector.detect_topics(&message1);
        
        // Shift to investment topic
        let message2 = create_test_message("I want to invest in stocks and bonds");
        detector.detect_topics(&message2);
        
        // Shift to tax topic
        let message3 = create_test_message("What about tax implications of my investments?");
        detector.detect_topics(&message3);
        
        // Should have detected at least one topic shift
        assert!(!detector.topic_shifts.is_empty());
    }
    
    #[test]
    fn test_topic_pruning() {
        let mut config = TopicDetectorConfig::default();
        config.max_topics = 2; // Only keep 2 topics
        
        let mut detector = TopicDetector::with_config(config);
        
        // Add 3 different topics
        let message1 = create_test_message("I want to plan for my retirement with a 401k");
        detector.detect_topics(&message1);
        
        let message2 = create_test_message("I want to invest in stocks and bonds");
        detector.detect_topics(&message2);
        
        let message3 = create_test_message("What about tax implications of my investments?");
        detector.detect_topics(&message3);
        
        // Should have pruned to only 2 topics
        assert!(detector.active_topics.len() <= 2);
    }
} 