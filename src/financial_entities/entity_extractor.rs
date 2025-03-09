use std::collections::HashMap;
use serde::{Serialize, Deserialize};
use uuid::Uuid;
use chrono::Utc;
use std::fmt;

use crate::conversation::Message;
use crate::financial_entities::FinancialEntity;

/// Represents the type of a financial entity
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum FinancialEntityType {
    Account,
    Asset,
    Liability,
    Income,
    Expense,
    Goal,
    Institution,
    Product,
    Tax,
    Insurance,
    Person,
    Other(String),
}

impl fmt::Display for FinancialEntityType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FinancialEntityType::Account => write!(f, "Account"),
            FinancialEntityType::Asset => write!(f, "Asset"),
            FinancialEntityType::Liability => write!(f, "Liability"),
            FinancialEntityType::Income => write!(f, "Income"),
            FinancialEntityType::Expense => write!(f, "Expense"),
            FinancialEntityType::Goal => write!(f, "Goal"),
            FinancialEntityType::Institution => write!(f, "Institution"),
            FinancialEntityType::Product => write!(f, "Product"),
            FinancialEntityType::Tax => write!(f, "Tax"),
            FinancialEntityType::Insurance => write!(f, "Insurance"),
            FinancialEntityType::Person => write!(f, "Person"),
            FinancialEntityType::Other(name) => write!(f, "Other({})", name),
        }
    }
}

/// Configuration for the entity extractor
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntityExtractorConfig {
    /// Minimum confidence score to consider an entity detected (0.0 to 1.0)
    pub min_confidence: f32,
    /// Whether to use LLM for entity extraction
    pub use_llm: bool,
    /// Whether to use rule-based extraction as fallback
    pub use_rule_based_fallback: bool,
    /// Whether to normalize entity values (e.g., convert "$5,000" to 5000.0)
    pub normalize_values: bool,
}

impl Default for EntityExtractorConfig {
    fn default() -> Self {
        Self {
            min_confidence: 0.7,
            use_llm: true,
            use_rule_based_fallback: true,
            normalize_values: true,
        }
    }
}

/// Extracts financial entities from conversations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntityExtractor {
    /// Configuration for the entity extractor
    config: EntityExtractorConfig,
    /// Known entity patterns by type
    entity_patterns: HashMap<FinancialEntityType, Vec<String>>,
    /// Extracted entities
    entities: HashMap<String, FinancialEntity>,
}

impl EntityExtractor {
    /// Creates a new entity extractor with default settings
    pub fn new() -> Self {
        let mut extractor = Self {
            config: EntityExtractorConfig::default(),
            entity_patterns: HashMap::new(),
            entities: HashMap::new(),
        };
        
        extractor.initialize_entity_patterns();
        extractor
    }
    
    /// Creates a new entity extractor with custom configuration
    pub fn with_config(config: EntityExtractorConfig) -> Self {
        let mut extractor = Self {
            config,
            entity_patterns: HashMap::new(),
            entities: HashMap::new(),
        };
        
        extractor.initialize_entity_patterns();
        extractor
    }
    
    /// Extracts entities from a message and updates the entity collection
    pub fn extract_entities(&mut self, message: &Message) -> Vec<FinancialEntity> {
        // Extract entities using LLM if enabled
        let mut extracted_entities = if self.config.use_llm {
            self.extract_entities_with_llm(message)
        } else {
            Vec::new()
        };
        
        // Use rule-based extraction as fallback if enabled and no entities extracted
        if extracted_entities.is_empty() && self.config.use_rule_based_fallback {
            extracted_entities = self.extract_entities_rule_based(message);
        }
        
        // Update entity collection
        for entity in &extracted_entities {
            self.update_entity(entity.clone());
        }
        
        // Return extracted entities
        extracted_entities
    }
    
    /// Gets all extracted entities
    pub fn get_all_entities(&self) -> Vec<FinancialEntity> {
        self.entities.values().cloned().collect()
    }
    
    /// Gets entities by type
    pub fn get_entities_by_type(&self, entity_type: &FinancialEntityType) -> Vec<FinancialEntity> {
        self.entities.values()
            .filter(|entity| {
                // In a real implementation, we would have a type field in FinancialEntity
                // For now, we'll check if the entity name contains the type name
                let type_name = match entity_type {
                    FinancialEntityType::Other(name) => name.clone(),
                    _ => format!("{:?}", entity_type),
                };
                
                entity.entity_type == type_name
            })
            .cloned()
            .collect()
    }
    
    /// Gets an entity by ID
    pub fn get_entity_by_id(&self, id: &str) -> Option<FinancialEntity> {
        self.entities.get(id).cloned()
    }
    
    /// Gets entities by name (partial match)
    pub fn get_entities_by_name(&self, name: &str) -> Vec<FinancialEntity> {
        let name_lower = name.to_lowercase();
        self.entities.values()
            .filter(|entity| entity.name.to_lowercase().contains(&name_lower))
            .cloned()
            .collect()
    }
    
    /// Initializes the entity patterns map
    fn initialize_entity_patterns(&mut self) {
        // Account patterns
        self.entity_patterns.insert(
            FinancialEntityType::Account,
            vec![
                r"401[k|K]".to_string(),
                r"IRA".to_string(),
                r"Roth".to_string(),
                r"checking account".to_string(),
                r"savings account".to_string(),
                r"brokerage account".to_string(),
                r"HSA".to_string(),
                r"529".to_string(),
                r"CD".to_string(),
                r"certificate of deposit".to_string(),
            ],
        );
        
        // Asset patterns
        self.entity_patterns.insert(
            FinancialEntityType::Asset,
            vec![
                r"stock".to_string(),
                r"bond".to_string(),
                r"ETF".to_string(),
                r"mutual fund".to_string(),
                r"real estate".to_string(),
                r"property".to_string(),
                r"home".to_string(),
                r"house".to_string(),
                r"investment property".to_string(),
                r"gold".to_string(),
                r"silver".to_string(),
                r"cryptocurrency".to_string(),
                r"bitcoin".to_string(),
                r"ethereum".to_string(),
            ],
        );
        
        // Liability patterns
        self.entity_patterns.insert(
            FinancialEntityType::Liability,
            vec![
                r"mortgage".to_string(),
                r"loan".to_string(),
                r"debt".to_string(),
                r"credit card".to_string(),
                r"student loan".to_string(),
                r"auto loan".to_string(),
                r"personal loan".to_string(),
                r"HELOC".to_string(),
                r"home equity".to_string(),
            ],
        );
        
        // Income patterns
        self.entity_patterns.insert(
            FinancialEntityType::Income,
            vec![
                r"salary".to_string(),
                r"wage".to_string(),
                r"income".to_string(),
                r"dividend".to_string(),
                r"interest".to_string(),
                r"rental income".to_string(),
                r"social security".to_string(),
                r"pension".to_string(),
                r"annuity".to_string(),
                r"bonus".to_string(),
                r"commission".to_string(),
            ],
        );
        
        // Add more entity types as needed...
    }
    
    /// Extracts entities using an LLM (placeholder for future implementation)
    fn extract_entities_with_llm(&self, _message: &Message) -> Vec<FinancialEntity> {
        // This would call an LLM to extract entities in a real implementation
        // For now, return an empty vector
        Vec::new()
    }
    
    /// Extracts entities using rule-based pattern matching
    fn extract_entities_rule_based(&self, message: &Message) -> Vec<FinancialEntity> {
        let content = message.content.to_lowercase();
        let mut entities = Vec::new();
        
        // Extract entities based on patterns
        for (entity_type, patterns) in &self.entity_patterns {
            for pattern in patterns {
                let pattern_lower = pattern.to_lowercase();
                
                if content.contains(&pattern_lower) {
                    // Extract value if available
                    let value = self.extract_value(&content, &pattern_lower);
                    
                    let entity = FinancialEntity {
                        id: Uuid::new_v4().to_string(),
                        name: pattern.clone(),
                        entity_type: entity_type.to_string(),
                        value,
                        metadata: HashMap::new(),
                        first_mentioned: Utc::now(),
                        last_mentioned: Utc::now(),
                        recency_score: 1.0, // New entities start with maximum recency
                    };
                    
                    entities.push(entity);
                }
            }
        }
        
        entities
    }
    
    /// Extracts a numeric value associated with an entity
    fn extract_value(&self, content: &str, entity_name: &str) -> Option<f64> {
        if !self.config.normalize_values {
            return None;
        }
        
        // Simple implementation to find a dollar amount near the entity name
        // In a real implementation, this would be much more sophisticated
        
        // Look for patterns like "$5,000" or "5000 dollars" near the entity name
        let entity_pos = match content.find(entity_name) {
            Some(pos) => pos,
            None => return None,
        };
        
        // Search for dollar amounts in a window around the entity
        let start = entity_pos.saturating_sub(50);
        let end = (entity_pos + entity_name.len() + 50).min(content.len());
        let window = &content[start..end];
        
        // Simple regex to find dollar amounts
        // In a real implementation, we would use a proper regex library
        for word in window.split_whitespace() {
            // Check for "$5,000" or "$5000" format
            if word.starts_with("$") {
                let numeric_part = word.trim_start_matches("$").replace(",", "");
                if let Ok(value) = numeric_part.parse::<f64>() {
                    return Some(value);
                }
            }
            
            // Check for "5000" format followed by "dollars"
            if let Ok(value) = word.replace(",", "").parse::<f64>() {
                // Check if "dollars" or "dollar" follows
                if window.contains("dollars") || window.contains("dollar") {
                    return Some(value);
                }
            }
        }
        
        None
    }
    
    /// Updates an entity or adds a new one
    fn update_entity(&mut self, entity: FinancialEntity) {
        // Check if we already have this entity (by name and type)
        for (_id, existing_entity) in &mut self.entities {
            if existing_entity.name == entity.name && existing_entity.entity_type == entity.entity_type {
                // Update the existing entity
                existing_entity.last_mentioned = entity.last_mentioned;
                
                // Update value if the new entity has a value
                if entity.value.is_some() {
                    existing_entity.value = entity.value;
                }
                
                // Merge metadata
                for (key, value) in entity.metadata {
                    existing_entity.metadata.insert(key, value);
                }
                
                return;
            }
        }
        
        // Add as a new entity
        self.entities.insert(entity.id.clone(), entity);
    }
    
    /// Updates entity recency scores
    fn update_entity_recency(&mut self) {
        // Decay recency scores for all entities
        for (_id, existing_entity) in &mut self.entities {
            // Decay the recency score (multiply by 0.9)
            existing_entity.recency_score *= 0.9;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    
    fn create_test_message(content: &str) -> Message {
        Message {
            id: Uuid::new_v4().to_string(),
            role: "user".to_string(),
            content: content.to_string(),
            timestamp: Utc::now(),
            metadata: HashMap::new(),
        }
    }
    
    #[test]
    fn test_entity_extractor_creation() {
        let extractor = EntityExtractor::new();
        assert_eq!(extractor.config.min_confidence, 0.7);
        assert!(extractor.config.use_llm);
        assert!(extractor.config.use_rule_based_fallback);
        assert!(extractor.config.normalize_values);
    }
    
    #[test]
    fn test_rule_based_extraction() {
        let mut extractor = EntityExtractor::with_config(EntityExtractorConfig {
            use_llm: false,
            ..Default::default()
        });
        
        // Test account entity extraction
        let message = create_test_message("I have a 401k account with $50,000 in it.");
        let entities = extractor.extract_entities(&message);
        
        assert!(!entities.is_empty());
        assert_eq!(entities[0].name, "401[k|K]");
        assert_eq!(entities[0].entity_type, "Account");
        assert_eq!(entities[0].value, Some(50000.0));
        
        // Test asset entity extraction
        let message = create_test_message("I own some stocks and ETFs in my portfolio.");
        let entities = extractor.extract_entities(&message);
        
        assert!(!entities.is_empty());
        assert!(entities.iter().any(|e| e.name == "stock" && e.entity_type == "Asset"));
        assert!(entities.iter().any(|e| e.name == "ETF" && e.entity_type == "Asset"));
        
        // Test liability entity extraction
        let message = create_test_message("I have a mortgage of $300,000 on my house.");
        let entities = extractor.extract_entities(&message);
        
        assert!(!entities.is_empty());
        assert_eq!(entities[0].name, "mortgage");
        assert_eq!(entities[0].entity_type, "Liability");
        assert_eq!(entities[0].value, Some(300000.0));
    }
    
    #[test]
    fn test_entity_updating() {
        let mut extractor = EntityExtractor::with_config(EntityExtractorConfig {
            use_llm: false,
            ..Default::default()
        });
        
        // Add an entity
        extractor.extract_entities(&create_test_message("I have a 401k with $50,000."));
        
        // Update the same entity
        extractor.extract_entities(&create_test_message("My 401k is now worth $60,000."));
        
        // Should have only one entity with the updated value
        let entities = extractor.get_all_entities();
        assert_eq!(entities.len(), 1);
        assert_eq!(entities[0].name, "401[k|K]");
        assert_eq!(entities[0].value, Some(60000.0));
    }
} 