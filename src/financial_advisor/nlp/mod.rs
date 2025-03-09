mod rule_based;
pub mod bedrock;
mod hybrid;
mod validation;
mod types;
pub mod embeddings;
pub mod conversation_manager;
pub mod knowledge_retriever;
pub mod enhanced_hybrid;
pub mod reasoning;
mod debug_impls;
pub mod financial_knowledge_base;
pub mod conversation_storage;
pub mod conversation_storage_dynamodb;
pub mod conversation_manager_storage;
#[cfg(test)]
mod integration_tests;

// Conversation summarization
pub mod conversation_summary;
pub mod conversation_summary_bedrock;
pub mod conversation_summary_storage;
pub mod conversation_summary_extractors;

pub use rule_based::{FinancialNlpService, FinancialQueryIntent, EntityType, ProcessedQuery, ExtractedEntity};
pub use hybrid::HybridNlpService;
pub use types::{NlpResponse, ValidatedLlmResponse, NlpConfidenceLevel, NlpResponseSource, ClientData, PortfolioData, AssetAllocation, PerformanceData};
pub use embeddings::{EmbeddingService, TitanEmbeddingConfig};
pub use conversation_manager::{ConversationManager, ConversationTurn, ConversationGoal, ConversationGoalType, ConversationState, GoalStatus};
pub use knowledge_retriever::{KnowledgeRetriever, KnowledgeItem, KnowledgeSourceType, KnowledgeRetrieverConfig};
pub use enhanced_hybrid::{EnhancedHybridService, EnhancedHybridConfig};
pub use reasoning::{FinancialReasoningService, ReasoningServiceConfig, ReasoningChain, ReasoningStep, ReasoningStepType, FinancialScenario, FinancialRecommendation};
pub use financial_knowledge_base::{FinancialKnowledgeBase, FinancialKnowledgeBaseConfig, FinancialKnowledgeEntry, FinancialKnowledgeCategory, FinancialKnowledgeSource, create_default_knowledge_base};
pub use conversation_storage::{ConversationStorage, ConversationStorageConfig, StorageType, Conversation, ConversationMessage, MessageRole};
pub use conversation_storage_dynamodb::DynamoDbConversationStorage;
pub use conversation_manager_storage::PersistentConversationManager;

// Re-export conversation summarization types
pub use conversation_summary::{
    ConversationSummary, SummaryType, SummaryGenerator, SummaryStorage,
    EntityExtractor, ImportanceScorer, FinancialEntity, FinancialEntityType,
    FinancialDecision, DecisionType, DecisionStatus, ConversationTopic,
    SummaryManager,
};

// Re-export conversation summarization implementations
pub use conversation_summary_bedrock::BedrockSummaryGenerator;
pub use conversation_summary_storage::{InMemorySummaryStorage, DynamoDbSummaryStorage};
pub use conversation_summary_extractors::{LlmEntityExtractor, RuleBasedImportanceScorer}; 