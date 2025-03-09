//! # Modern Conversational Financial Advisor
//! 
//! A human-level conversational financial advisor using emerging AI techniques.
//! 
//! ## Core Modules
//! 
//! - **conversation**: Core conversation management and context tracking
//! - **financial_entities**: Financial entity extraction and management
//! - **financial_advisor**: Advanced NLP and financial knowledge capabilities
//! - **portfolio**: Portfolio management and investment tracking
//! - **factor_model**: Factor model analysis and asset allocation
//! - **performance_calculator**: Performance calculation and analysis
//! - **common**: Common utilities and shared resources
//! - **visualization**: Visualization tools and utilities

// Core modules
pub mod conversation;
pub mod financial_entities;
pub mod financial_advisor;
pub mod portfolio;
pub mod factor_model;
pub mod performance_calculator;
pub mod common;
pub mod visualization;

// Re-export key types for easier access
pub use conversation::{Message, Conversation};
pub use conversation::context_manager::{ContextManager, ContextManagerConfig, ConversationState, ImportanceLevel, ConversationTopic};
pub use conversation::topic_detector::{TopicDetector, FinancialTopicCategory};
pub use financial_entities::FinancialEntity;
pub use financial_entities::entity_extractor::{EntityExtractor, FinancialEntityType};

/// Version of the library
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Creates a new conversational financial advisor with default settings
pub fn create_conversational_advisor() -> ConversationalAdvisor {
    ConversationalAdvisor::new()
}

/// Main conversational advisor struct that coordinates all components
#[derive(Debug)]
pub struct ConversationalAdvisor {
    /// Context manager for maintaining conversation context
    context_manager: conversation::context_manager::ContextManager,
    /// Topic detector for identifying financial topics
    topic_detector: conversation::topic_detector::TopicDetector,
    /// Entity extractor for identifying financial entities
    entity_extractor: financial_entities::entity_extractor::EntityExtractor,
    /// Enhanced NLP service for advanced processing (optional)
    enhanced_nlp: Option<std::sync::Arc<financial_advisor::nlp::EnhancedHybridService>>,
    /// Conversation manager for maintaining conversation state (optional)
    conversation_manager: Option<std::sync::Arc<tokio::sync::Mutex<financial_advisor::nlp::ConversationManager>>>,
    /// Knowledge retriever for providing relevant information (optional)
    knowledge_retriever: Option<std::sync::Arc<financial_advisor::nlp::KnowledgeRetriever>>,
}

impl ConversationalAdvisor {
    /// Creates a new conversational advisor with default settings
    pub fn new() -> Self {
        Self {
            context_manager: conversation::context_manager::ContextManager::new(),
            topic_detector: conversation::topic_detector::TopicDetector::new(),
            entity_extractor: financial_entities::entity_extractor::EntityExtractor::new(),
            enhanced_nlp: None,
            conversation_manager: None,
            knowledge_retriever: None,
        }
    }
    
    /// Creates a new conversational advisor with enhanced NLP capabilities
    pub async fn new_enhanced() -> anyhow::Result<Self> {
        use financial_advisor::nlp::{
            EnhancedHybridService, 
            EnhancedHybridConfig, 
            ConversationManager,
            KnowledgeRetriever,
            KnowledgeRetrieverConfig,
            bedrock::{BedrockNlpConfig, BedrockModelProvider},
            embeddings::TitanEmbeddingConfig,
        };
        
        use std::sync::Arc;
        use tokio::sync::Mutex;
        
        // Initialize AWS config
        let aws_config = aws_config::defaults(aws_config::BehaviorVersion::v2024_03_28())
            .load()
            .await;
            
        // Create Bedrock client
        let bedrock_client = aws_sdk_bedrockruntime::Client::new(&aws_config);
        
        // Create NLP config - using Claude 3 Sonnet for best performance
        let nlp_config = BedrockNlpConfig {
            provider: BedrockModelProvider::Claude,
            version: "3-sonnet-20240229-v1:0".to_string(),
            temperature: 0.2,
            max_tokens: 4096,
            top_p: 0.9,
            top_k: 250,
            stop_sequences: vec![],
        };
        
        // Create embedding config
        let embedding_config = TitanEmbeddingConfig::default();
        
        // Create enhanced hybrid service
        let enhanced_nlp = EnhancedHybridService::new_with_embeddings(
            bedrock_client.clone(),
            nlp_config,
            embedding_config,
        ).with_config(EnhancedHybridConfig {
            use_llm_for_responses: true,
            use_embeddings_for_intents: true,
            use_conversation_context: true,
            use_knowledge_retrieval: true,
            ..EnhancedHybridConfig::default()
        });
        
        // Create knowledge retriever with default config
        let knowledge_retriever = Arc::new(KnowledgeRetriever::new(KnowledgeRetrieverConfig::default()));
        
        // Add knowledge retriever to enhanced NLP
        let enhanced_nlp = enhanced_nlp.with_knowledge_retriever(knowledge_retriever.clone());
        
        // Create conversation manager
        let conversation_manager = Arc::new(Mutex::new(
            ConversationManager::new("default_user")
        ));
        
        // Create context manager with persistence
        let context_config = ContextManagerConfig {
            max_context_messages: 30,
            max_context_tokens: 6000,
            max_segments: 15,
            min_segment_importance: 0.3,
            persistence_path: Some("conversations/context.json".to_string()),
            auto_persist: true,
        };
        
        let context_manager = conversation::context_manager::ContextManager::with_config(context_config);
        
        Ok(Self {
            context_manager,
            topic_detector: conversation::topic_detector::TopicDetector::new(),
            entity_extractor: financial_entities::entity_extractor::EntityExtractor::new(),
            enhanced_nlp: Some(Arc::new(enhanced_nlp)),
            conversation_manager: Some(conversation_manager),
            knowledge_retriever: Some(knowledge_retriever),
        })
    }
    
    /// Processes a user message and returns a response
    pub fn process_message(&mut self, message: &str) -> String {
        // Create a message object
        let user_message = conversation::Message::from_user(message);
        
        // Detect topics in the message
        let topics = self.topic_detector.detect_topics(&user_message);
        
        // Extract entities from the message
        let entities = self.entity_extractor.extract_entities(&user_message);
        
        // Add the message to the context manager
        self.context_manager.add_message(user_message.clone());
        
        // Get relevant context from the context manager
        let _relevant_context = self.context_manager.get_relevant_context();
        
        // Check for topic shifts
        let _topic_shifts = self.topic_detector.get_topic_shifts();
        let latest_shift = self.topic_detector.get_latest_topic_shift();
        
        // In a real implementation, this would use an LLM to generate a response
        // For now, we'll return a simple response based on the detected topics and entities
        
        if let Some(shift) = latest_shift {
            if shift.message_index == self.topic_detector.get_message_counter() {
                // A topic shift just occurred
                if let Some(previous) = &shift.previous_topic {
                    format!("I see we're shifting from discussing {} to {}. How can I help you with that?", 
                        previous, shift.new_topic)
                } else {
                    format!("I see we're now focusing on {}. How can I help you with that?", 
                        shift.new_topic)
                }
            } else if !topics.is_empty() {
                let topic_name = &topics[0].name;
                format!("I see you're interested in {}. How can I help you with that?", topic_name)
            } else if !entities.is_empty() {
                let entity_name = &entities[0].name;
                let entity_type = &entities[0].entity_type;
                
                if let Some(value) = entities[0].value {
                    format!("I see you mentioned your {} with a value of ${:.2}. Would you like to discuss this further?", entity_name, value)
                } else {
                    format!("I see you mentioned {}. Would you like to discuss this {}?", entity_name, entity_type)
                }
            } else {
                "How can I assist you with your financial planning today?".to_string()
            }
        } else if !topics.is_empty() {
            let topic_name = &topics[0].name;
            format!("I see you're interested in {}. How can I help you with that?", topic_name)
        } else if !entities.is_empty() {
            let entity_name = &entities[0].name;
            let entity_type = &entities[0].entity_type;
            
            if let Some(value) = entities[0].value {
                format!("I see you mentioned your {} with a value of ${:.2}. Would you like to discuss this further?", entity_name, value)
            } else {
                format!("I see you mentioned {}. Would you like to discuss this {}?", entity_name, entity_type)
            }
        } else {
            "How can I assist you with your financial planning today?".to_string()
        }
    }
    
    /// Processes a user message asynchronously using enhanced NLP capabilities
    pub async fn process_message_enhanced(&self, message: &str) -> anyhow::Result<String> {
        if let Some(enhanced_nlp) = &self.enhanced_nlp {
            if let Some(conversation_manager) = &self.conversation_manager {
                // Lock the conversation manager
                let mut conversation_manager = conversation_manager.lock().await;
                
                // Get relevant context from the context manager
                let _relevant_context = self.context_manager.get_relevant_context();
                
                // Check for topic shifts
                let _topic_shifts = self.topic_detector.get_topic_shifts();
                let latest_shift = self.topic_detector.get_latest_topic_shift();
                
                // Create context text with topic information
                let mut context_parts = Vec::new();
                
                // Add relevant messages
                let messages_context = _relevant_context.iter()
                    .map(|msg| format!("{}: {}", msg.role, msg.content))
                    .collect::<Vec<String>>()
                    .join("\n");
                
                if !messages_context.is_empty() {
                    context_parts.push(format!("Previous conversation:\n{}", messages_context));
                }
                
                // Add topic information
                let current_topics = self.context_manager.get_current_topics();
                if !current_topics.is_empty() {
                    let topics_str = current_topics.iter()
                        .map(|t| format!("{} (importance: {})", t.name, t.importance.to_score()))
                        .collect::<Vec<String>>()
                        .join(", ");
                    
                    context_parts.push(format!("Current topics: {}", topics_str));
                }
                
                // Add topic shift information
                if let Some(shift) = latest_shift {
                    if let Some(previous) = &shift.previous_topic {
                        context_parts.push(format!("Topic shift: {} -> {}", previous, shift.new_topic));
                    } else {
                        context_parts.push(format!("New topic: {}", shift.new_topic));
                    }
                }
                
                // Combine all context parts
                let context_text = context_parts.join("\n\n");
                
                // Add the user query with context to the conversation manager
                let query_with_context = if !context_text.is_empty() {
                    format!("{}\n\nContext from previous conversation:\n{}", message, context_text)
                } else {
                    message.to_string()
                };
                
                conversation_manager.add_user_query(&query_with_context);
                
                // Process the conversation turn
                let turn = enhanced_nlp.process_conversation_turn(
                    &mut conversation_manager, 
                    None
                ).await?;
                
                // Return the response
                if let Some(response) = &turn.response {
                    Ok(response.response_text.clone())
                } else {
                    Err(anyhow::anyhow!("No response generated"))
                }
            } else {
                // If we don't have a conversation manager, use the enhanced NLP directly
                // Get relevant context from the context manager
                let _relevant_context = self.context_manager.get_relevant_context();
                
                // Check for topic shifts
                let _topic_shifts = self.topic_detector.get_topic_shifts();
                let latest_shift = self.topic_detector.get_latest_topic_shift();
                
                // Create context text with topic information
                let mut context_parts = Vec::new();
                
                // Add relevant messages
                let messages_context = _relevant_context.iter()
                    .map(|msg| format!("{}: {}", msg.role, msg.content))
                    .collect::<Vec<String>>()
                    .join("\n");
                
                if !messages_context.is_empty() {
                    context_parts.push(format!("Previous conversation:\n{}", messages_context));
                }
                
                // Add topic information
                let current_topics = self.context_manager.get_current_topics();
                if !current_topics.is_empty() {
                    let topics_str = current_topics.iter()
                        .map(|t| format!("{} (importance: {})", t.name, t.importance.to_score()))
                        .collect::<Vec<String>>()
                        .join(", ");
                    
                    context_parts.push(format!("Current topics: {}", topics_str));
                }
                
                // Add topic shift information
                if let Some(shift) = latest_shift {
                    if let Some(previous) = &shift.previous_topic {
                        context_parts.push(format!("Topic shift: {} -> {}", previous, shift.new_topic));
                    } else {
                        context_parts.push(format!("New topic: {}", shift.new_topic));
                    }
                }
                
                // Combine all context parts
                let context_text = context_parts.join("\n\n");
                
                // Create a temporary conversation manager for this request
                let mut temp_conversation_manager = financial_advisor::nlp::ConversationManager::new("temp_user");
                
                // Add the user query with context to the conversation manager
                let query_with_context = if !context_text.is_empty() {
                    format!("{}\n\nContext from previous conversation:\n{}", message, context_text)
                } else {
                    message.to_string()
                };
                
                temp_conversation_manager.add_user_query(&query_with_context);
                
                // Process the query with the temporary conversation manager
                let response = enhanced_nlp.process_query(
                    &query_with_context, 
                    Some(&temp_conversation_manager), 
                    None
                ).await?;
                
                Ok(response.response_text)
            }
        } else {
            // Fall back to the simple response generation
            // Create a clone of self to avoid borrowing issues
            let mut advisor_clone = ConversationalAdvisor {
                context_manager: self.context_manager.clone(),
                topic_detector: self.topic_detector.clone(),
                entity_extractor: self.entity_extractor.clone(),
                enhanced_nlp: None,
                conversation_manager: None,
                knowledge_retriever: None,
            };
            Ok(advisor_clone.process_message(message))
        }
    }
    
    /// Gets the current conversation state
    pub fn get_conversation_state(&self) -> conversation::context_manager::ConversationState {
        self.context_manager.get_state()
    }
    
    /// Gets all detected topics
    pub fn get_topics(&self) -> Vec<conversation::context_manager::ConversationTopic> {
        self.topic_detector.get_active_topics()
    }
    
    /// Gets all extracted entities
    pub fn get_entities(&self) -> Vec<financial_entities::FinancialEntity> {
        self.entity_extractor.get_all_entities()
    }
    
    /// Returns the currently detected topics in the conversation
    pub fn get_detected_topics(&self) -> Vec<conversation::context_manager::ConversationTopic> {
        self.context_manager.get_current_topics()
    }
    
    /// Returns the currently detected entities in the conversation
    pub fn get_detected_entities(&self) -> Vec<financial_entities::FinancialEntity> {
        self.context_manager.get_entities()
    }
    
    /// Persists the current conversation context to a file
    pub fn persist_context(&self, path: &str) -> Result<(), std::io::Error> {
        self.context_manager.persist_to_file(path)
    }
    
    /// Loads conversation context from a file
    pub fn load_context(&mut self, path: &str) -> Result<(), std::io::Error> {
        let loaded_context = conversation::context_manager::ContextManager::load_from_file(path)?;
        self.context_manager = loaded_context;
        Ok(())
    }
    
    /// Sets the conversation ID for persistence
    pub fn set_conversation_id(&mut self, id: &str) {
        self.context_manager.set_conversation_id(id);
    }
    
    /// Gets the conversation ID
    pub fn get_conversation_id(&self) -> &str {
        self.context_manager.get_conversation_id()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_conversational_advisor_creation() {
        let advisor = create_conversational_advisor();
        assert_eq!(advisor.get_conversation_state(), conversation::context_manager::ConversationState::Introduction);
    }
    
    #[test]
    fn test_process_message() {
        let mut advisor = create_conversational_advisor();
        let response = advisor.process_message("I'm planning for retirement");
        
        // The response should mention retirement
        assert!(response.contains("retirement") || response.contains("Retirement"));
    }
} 