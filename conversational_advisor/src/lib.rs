pub mod conversation;
pub mod financial_entities;

/// Re-export key types for easier access
pub use conversation::{Message, Conversation};
pub use conversation::context_manager::{ContextManager, ConversationState, ImportanceLevel};
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
}

impl ConversationalAdvisor {
    /// Creates a new conversational advisor with default settings
    pub fn new() -> Self {
        Self {
            context_manager: conversation::context_manager::ContextManager::new(),
            topic_detector: conversation::topic_detector::TopicDetector::new(),
            entity_extractor: financial_entities::entity_extractor::EntityExtractor::new(),
        }
    }
    
    /// Processes a user message and returns a response
    pub fn process_message(&mut self, message: &str) -> String {
        // Create a message object
        let user_message = conversation::Message::from_user(message);
        
        // Add the message to the context manager
        self.context_manager.add_message(user_message.clone());
        
        // Detect topics in the message
        let topics = self.topic_detector.detect_topics(&user_message);
        
        // Extract entities from the message
        let entities = self.entity_extractor.extract_entities(&user_message);
        
        // Get the relevant context for generating a response
        let _context = self.context_manager.get_relevant_context();
        
        // In a real implementation, this would use an LLM to generate a response
        // For now, we'll return a simple response based on the detected topics and entities
        
        if !topics.is_empty() {
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
        
        // Test with a message about retirement
        let response = advisor.process_message("I'm planning for retirement and want to save $500,000");
        assert!(!response.is_empty());
        
        // Check that topics were detected
        let topics = advisor.get_topics();
        assert!(!topics.is_empty());
        
        // Check that entities might be detected (but don't require it)
        let entities = advisor.get_entities();
        println!("Detected entities: {:?}", entities);
        
        // Test with a message about a 401k
        let response = advisor.process_message("I have a 401k with $100,000");
        assert!(!response.is_empty());
        
        // Now we should definitely have entities
        let entities = advisor.get_entities();
        assert!(!entities.is_empty());
    }
}
