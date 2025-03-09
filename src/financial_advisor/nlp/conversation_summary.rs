use std::collections::HashMap;
use anyhow::{Result, anyhow};
use chrono::{DateTime, Utc};
use serde::{Serialize, Deserialize};
use uuid::Uuid;


/// Types of financial entities that can be extracted
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum FinancialEntityType {
    Account,
    Asset,
    Liability,
    Goal,
    Income,
    Expense,
    Tax,
    Insurance,
    Investment,
    Retirement,
    Other(String),
}

/// Represents a financial entity mentioned in a conversation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FinancialEntity {
    /// Type of financial entity
    pub entity_type: FinancialEntityType,
    
    /// Name or identifier of the entity
    pub name: String,
    
    /// Additional attributes of the entity
    pub attributes: HashMap<String, String>,
    
    /// Importance score (0.0 to 1.0)
    pub importance: f32,
}

/// Status of a financial decision
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum DecisionStatus {
    Proposed,
    Agreed,
    Rejected,
    Pending,
    Completed,
}

/// Types of financial decisions
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum DecisionType {
    Investment,
    Withdrawal,
    AssetAllocation,
    TaxStrategy,
    RetirementPlanning,
    InsuranceChange,
    BudgetAdjustment,
    DebtManagement,
    Other(String),
}

/// Represents a financial decision mentioned in a conversation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FinancialDecision {
    /// Type of decision
    pub decision_type: DecisionType,
    
    /// Description of the decision
    pub description: String,
    
    /// Status of the decision
    pub status: DecisionStatus,
    
    /// Related financial entities
    pub related_entities: Vec<String>,
    
    /// Importance score (0.0 to 1.0)
    pub importance: f32,
}

/// Represents a topic discussed in a conversation
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ConversationTopic {
    /// Name of the topic
    pub name: String,
    
    /// Confidence score (0.0 to 1.0)
    pub confidence: f32,
    
    /// Message IDs where this topic was discussed
    pub message_ids: Vec<String>,
}

/// Types of summaries that can be generated
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum SummaryType {
    /// Brief overview (1-2 sentences)
    Brief,
    
    /// Standard summary (1-2 paragraphs)
    Standard,
    
    /// Detailed summary with all key points
    Detailed,
    
    /// Focus on specific financial topic
    TopicFocused(String),
}

/// Represents a summary of a conversation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversationSummary {
    /// Unique ID for the summary
    pub id: String,
    
    /// ID of the conversation this summary is for
    pub conversation_id: String,
    
    /// The actual summary text
    pub content: String,
    
    /// Version of the summary (increments with updates)
    pub version: u32,
    
    /// When the summary was created
    pub created_at: DateTime<Utc>,
    
    /// When the summary was last updated
    pub updated_at: DateTime<Utc>,
    
    /// ID of the last message included in this summary
    pub last_message_id: String,
    
    /// Key financial entities mentioned in the summary
    pub key_entities: Vec<FinancialEntity>,
    
    /// Key decisions or actions mentioned in the summary
    pub key_decisions: Vec<FinancialDecision>,
    
    /// Topics covered in the conversation
    pub topics: Vec<ConversationTopic>,
}

impl ConversationSummary {
    /// Create a new conversation summary
    pub fn new(conversation_id: String, content: String, last_message_id: String) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4().to_string(),
            conversation_id,
            content,
            version: 1,
            created_at: now,
            updated_at: now,
            last_message_id,
            key_entities: Vec::new(),
            key_decisions: Vec::new(),
            topics: Vec::new(),
        }
    }
    
    /// Add a key entity to the summary
    pub fn add_entity(&mut self, entity: FinancialEntity) {
        self.key_entities.push(entity);
    }
    
    /// Add a key decision to the summary
    pub fn add_decision(&mut self, decision: FinancialDecision) {
        self.key_decisions.push(decision);
    }
    
    /// Add a topic to the summary
    pub fn add_topic(&mut self, topic: ConversationTopic) {
        self.topics.push(topic);
    }
    
    /// Update the summary content and increment version
    pub fn update_content(&mut self, content: String, last_message_id: String) {
        self.content = content;
        self.last_message_id = last_message_id;
        self.version += 1;
        self.updated_at = Utc::now();
    }
}

/// Generates summaries for conversations
pub trait SummaryGenerator: Send + Sync {
    /// Generate a new summary for a conversation
    fn generate_summary<'a>(
        &'a self,
        conversation_id: &'a str,
        summary_type: SummaryType,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<ConversationSummary>> + Send + 'a>>;
    
    /// Update an existing summary with new messages
    fn update_summary<'a>(
        &'a self,
        summary_id: &'a str,
        new_message_ids: &'a [String],
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<ConversationSummary>> + Send + 'a>>;
    
    /// Generate a summary focused on specific topics
    fn generate_topic_summary<'a>(
        &'a self,
        conversation_id: &'a str,
        topics: &'a [String],
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<ConversationSummary>> + Send + 'a>>;
}

/// Trait for storing and retrieving conversation summaries
pub trait SummaryStorage: Send + Sync {
    /// Save a summary
    fn save_summary<'a>(&'a self, summary: &'a ConversationSummary) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<()>> + Send + 'a>>;
    
    /// Get a summary by ID
    fn get_summary<'a>(&'a self, summary_id: &'a str) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<Option<ConversationSummary>>> + Send + 'a>>;
    
    /// List summaries for a conversation
    fn list_summaries<'a>(&'a self, conversation_id: &'a str) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<Vec<ConversationSummary>>> + Send + 'a>>;
    
    /// Delete a summary
    fn delete_summary<'a>(&'a self, summary_id: &'a str) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<()>> + Send + 'a>>;
}

/// Extracts financial entities from conversations
pub trait EntityExtractor: Send + Sync {
    /// Extract entities from a conversation
    fn extract_entities<'a>(
        &'a self,
        conversation_id: &'a str,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<Vec<FinancialEntity>>> + Send + 'a>>;
    
    /// Extract entities from specific messages
    fn extract_entities_from_messages<'a>(
        &'a self,
        message_ids: &'a [String],
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<Vec<FinancialEntity>>> + Send + 'a>>;
}

/// Scores the importance of entities and decisions
pub trait ImportanceScorer: Send + Sync {
    /// Score the importance of entities
    fn score_entities<'a>(
        &'a self,
        entities: &'a [FinancialEntity],
        conversation_id: &'a str,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<Vec<FinancialEntity>>> + Send + 'a>>;
    
    /// Score the importance of decisions
    fn score_decisions<'a>(
        &'a self,
        decisions: &'a [FinancialDecision],
        conversation_id: &'a str,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<Vec<FinancialDecision>>> + Send + 'a>>;
}

/// Manages conversation summaries
pub struct SummaryManager {
    generator: Box<dyn SummaryGenerator>,
    storage: Box<dyn SummaryStorage>,
    entity_extractor: Box<dyn EntityExtractor>,
    importance_scorer: Box<dyn ImportanceScorer>,
}

impl SummaryManager {
    /// Create a new summary manager
    pub fn new(
        generator: Box<dyn SummaryGenerator>,
        storage: Box<dyn SummaryStorage>,
        entity_extractor: Box<dyn EntityExtractor>,
        importance_scorer: Box<dyn ImportanceScorer>,
    ) -> Self {
        Self {
            generator,
            storage,
            entity_extractor,
            importance_scorer,
        }
    }
    
    /// Get or create a summary for a conversation
    pub async fn get_or_create_summary(
        &self,
        conversation_id: &str,
        summary_type: SummaryType,
    ) -> Result<ConversationSummary> {
        // Check if we already have a summary
        let summaries = self.storage.list_summaries(conversation_id).await?;
        if let Some(summary) = summaries.first() {
            return Ok(summary.clone());
        }
        
        // Generate a new summary
        let summary = self.generator.generate_summary(conversation_id, summary_type).await?;
        
        // Extract and score entities
        let entities = self.entity_extractor.extract_entities(conversation_id).await?;
        let scored_entities = self.importance_scorer.score_entities(&entities, conversation_id).await?;
        
        // Save the summary
        let mut enhanced_summary = summary;
        for entity in scored_entities {
            enhanced_summary.add_entity(entity);
        }
        
        self.storage.save_summary(&enhanced_summary).await?;
        
        Ok(enhanced_summary)
    }
    
    /// Update a summary with new conversation messages
    pub async fn update_summary(
        &self,
        conversation_id: &str,
    ) -> Result<ConversationSummary> {
        // Get the existing summary
        let summaries = self.storage.list_summaries(conversation_id).await?;
        let summary = summaries.first().ok_or_else(|| anyhow!("No summary found for conversation"))?;
        
        // Update the summary
        let updated_summary = self.generator.update_summary(&summary.id, &[]).await?;
        
        // Save the updated summary
        self.storage.save_summary(&updated_summary).await?;
        
        Ok(updated_summary)
    }
    
    /// Get a summary by ID
    pub async fn get_summary(
        &self,
        summary_id: &str,
    ) -> Result<Option<ConversationSummary>> {
        self.storage.get_summary(summary_id).await
    }
    
    /// List summaries for a conversation
    pub async fn list_summaries(
        &self,
        conversation_id: &str,
    ) -> Result<Vec<ConversationSummary>> {
        self.storage.list_summaries(conversation_id).await
    }
    
    /// Delete a summary
    pub async fn delete_summary(
        &self,
        summary_id: &str,
    ) -> Result<()> {
        self.storage.delete_summary(summary_id).await
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SummaryConfidence {
    pub overall: f32,
    pub confidence: f32,
} 