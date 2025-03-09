use std::collections::HashMap;
use serde::{Serialize, Deserialize};
use crate::conversation::Message;
use crate::financial_entities::FinancialEntity;
use regex::Regex;

/// Financial entity types
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
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
            min_confidence: 0.6,
            use_llm: true,
            use_rule_based_fallback: true,
            normalize_values: true,
        }
    }
}

/// Extracts and tracks financial entities from conversations
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
    
    /// Creates a new entity extractor with custom settings
    pub fn with_config(config: EntityExtractorConfig) -> Self {
        let mut extractor = Self {
            config,
            entity_patterns: HashMap::new(),
            entities: HashMap::new(),
        };
        
        extractor.initialize_entity_patterns();
        extractor
    }
    
    /// Extracts entities from a message
    pub fn extract_entities(&mut self, message: &Message) -> Vec<FinancialEntity> {
        let mut extracted_entities = Vec::new();
        
        // Try LLM-based extraction if enabled
        if self.config.use_llm {
            extracted_entities = self.extract_entities_with_llm(message);
        }
        
        // Fall back to rule-based extraction if needed
        if extracted_entities.is_empty() && self.config.use_rule_based_fallback {
            extracted_entities = self.extract_entities_rule_based(message);
        }
        
        // Update the entities collection
        for entity in extracted_entities.clone() {
            self.update_entity(entity);
        }
        
        extracted_entities
    }
    
    /// Gets all extracted entities
    pub fn get_all_entities(&self) -> Vec<FinancialEntity> {
        self.entities.values().cloned().collect()
    }
    
    /// Gets entities by type
    pub fn get_entities_by_type(&self, entity_type: &FinancialEntityType) -> Vec<FinancialEntity> {
        let type_str = match entity_type {
            FinancialEntityType::Account => "Account",
            FinancialEntityType::Asset => "Asset",
            FinancialEntityType::Liability => "Liability",
            FinancialEntityType::Income => "Income",
            FinancialEntityType::Expense => "Expense",
            FinancialEntityType::Goal => "Goal",
            FinancialEntityType::Institution => "Institution",
            FinancialEntityType::Product => "Product",
            FinancialEntityType::Tax => "Tax",
            FinancialEntityType::Insurance => "Insurance",
            FinancialEntityType::Person => "Person",
            FinancialEntityType::Other(s) => s,
        };
        
        self.entities
            .values()
            .filter(|e| e.entity_type == type_str)
            .cloned()
            .collect()
    }
    
    /// Gets an entity by ID
    pub fn get_entity_by_id(&self, id: &str) -> Option<FinancialEntity> {
        self.entities.get(id).cloned()
    }
    
    /// Gets entities by name
    pub fn get_entities_by_name(&self, name: &str) -> Vec<FinancialEntity> {
        self.entities
            .values()
            .filter(|e| e.name.to_lowercase().contains(&name.to_lowercase()))
            .cloned()
            .collect()
    }
    
    /// Initializes the entity patterns
    fn initialize_entity_patterns(&mut self) {
        // Account patterns
        self.entity_patterns.insert(
            FinancialEntityType::Account,
            vec![
                r"401[kK]".to_string(),
                r"IRA".to_string(),
                r"Roth".to_string(),
                r"checking account".to_string(),
                r"savings account".to_string(),
                r"brokerage account".to_string(),
                r"HSA".to_string(),
                r"529".to_string(),
            ],
        );
        
        // Asset patterns
        self.entity_patterns.insert(
            FinancialEntityType::Asset,
            vec![
                r"house".to_string(),
                r"home".to_string(),
                r"property".to_string(),
                r"real estate".to_string(),
                r"car".to_string(),
                r"vehicle".to_string(),
                r"stock".to_string(),
                r"bond".to_string(),
                r"ETF".to_string(),
                r"mutual fund".to_string(),
                r"gold".to_string(),
                r"silver".to_string(),
                r"crypto".to_string(),
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
            ],
        );
        
        // Add more entity types and patterns as needed
        // This is a simplified version for demonstration purposes
    }
    
    /// Extracts entities using an LLM
    fn extract_entities_with_llm(&self, _message: &Message) -> Vec<FinancialEntity> {
        // In a real implementation, this would call an LLM to extract entities
        // For now, return an empty vector
        Vec::new()
    }
    
    /// Extracts entities using rule-based methods
    fn extract_entities_rule_based(&self, message: &Message) -> Vec<FinancialEntity> {
        let mut extracted_entities = Vec::new();
        let content = message.content.to_lowercase();
        
        // Check each entity type's patterns against the message content
        for (entity_type, patterns) in &self.entity_patterns {
            for pattern in patterns {
                let regex_pattern = format!(r"(?i)\b{}\b", pattern);
                if let Ok(regex) = Regex::new(&regex_pattern) {
                    if regex.is_match(&content) {
                        // Extract the matched text
                        let captures = regex.captures(&content);
                        if let Some(cap) = captures {
                            let entity_name = cap[0].to_string();
                            
                            // Determine the entity type string
                            let type_str = match entity_type {
                                FinancialEntityType::Account => "Account",
                                FinancialEntityType::Asset => "Asset",
                                FinancialEntityType::Liability => "Liability",
                                FinancialEntityType::Income => "Income",
                                FinancialEntityType::Expense => "Expense",
                                FinancialEntityType::Goal => "Goal",
                                FinancialEntityType::Institution => "Institution",
                                FinancialEntityType::Product => "Product",
                                FinancialEntityType::Tax => "Tax",
                                FinancialEntityType::Insurance => "Insurance",
                                FinancialEntityType::Person => "Person",
                                FinancialEntityType::Other(s) => s,
                            };
                            
                            // Try to extract a value associated with the entity
                            let value = self.extract_value(&content, &entity_name);
                            
                            // Create the entity
                            let entity = if let Some(val) = value {
                                FinancialEntity::with_value(&entity_name, type_str, val)
                            } else {
                                FinancialEntity::new(&entity_name, type_str)
                            };
                            
                            extracted_entities.push(entity);
                        }
                    }
                }
            }
        }
        
        extracted_entities
    }
    
    /// Extracts a numeric value associated with an entity
    fn extract_value(&self, content: &str, _entity_name: &str) -> Option<f64> {
        // Look for currency values near the entity name
        let currency_regex = Regex::new(r"\$\s*(\d{1,3}(,\d{3})*(\.\d+)?|\d+(\.\d+)?)").unwrap();
        
        // Find all currency values in the content
        let mut values = Vec::new();
        for cap in currency_regex.captures_iter(content) {
            let value_str = cap[1].replace(",", "");
            if let Ok(value) = value_str.parse::<f64>() {
                values.push(value);
            }
        }
        
        // If we found values, return the one closest to the entity name
        // This is a simplified approach; in a real implementation, we would use
        // more sophisticated NLP techniques to associate values with entities
        if !values.is_empty() {
            // For simplicity, just return the first value
            return Some(values[0]);
        }
        
        // Look for numeric values with units
        let numeric_regex = Regex::new(r"(\d{1,3}(,\d{3})*(\.\d+)?|\d+(\.\d+)?)\s*(dollars|USD|k|thousand|million|billion)").unwrap();
        
        for cap in numeric_regex.captures_iter(content) {
            let value_str = cap[1].replace(",", "");
            if let Ok(mut value) = value_str.parse::<f64>() {
                // Apply multiplier based on unit
                let unit = cap[5].to_lowercase();
                if unit == "k" || unit == "thousand" {
                    value *= 1_000.0;
                } else if unit == "million" {
                    value *= 1_000_000.0;
                } else if unit == "billion" {
                    value *= 1_000_000_000.0;
                }
                
                return Some(value);
            }
        }
        
        None
    }
    
    /// Updates an entity or adds it if it's new
    fn update_entity(&mut self, entity: FinancialEntity) {
        for (_id, existing_entity) in &mut self.entities {
            // If we already have this entity, update it
            if existing_entity.name.to_lowercase() == entity.name.to_lowercase() &&
               existing_entity.entity_type == entity.entity_type {
                // Update last mentioned
                existing_entity.update_last_mentioned();
                
                // Update value if the new entity has one
                if let Some(value) = entity.value {
                    existing_entity.set_value(value);
                }
                
                // Merge metadata
                for (key, value) in entity.metadata {
                    existing_entity.add_metadata(&key, &value);
                }
                
                return;
            }
        }
        
        // If we get here, it's a new entity
        self.entities.insert(entity.id.clone(), entity);
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
    fn test_entity_extractor_creation() {
        let extractor = EntityExtractor::new();
        assert_eq!(extractor.get_all_entities().len(), 0);
        assert!(extractor.entity_patterns.contains_key(&FinancialEntityType::Account));
        assert!(extractor.entity_patterns.contains_key(&FinancialEntityType::Asset));
    }
    
    #[test]
    fn test_rule_based_extraction() {
        let mut extractor = EntityExtractor::new();
        
        // Test account entity
        let message = create_test_message("I have a 401k with $50,000 in it");
        let entities = extractor.extract_entities(&message);
        
        assert!(!entities.is_empty());
        assert_eq!(entities[0].name, "401k");
        assert_eq!(entities[0].entity_type, "Account");
        assert_eq!(entities[0].value, Some(50000.0));
        
        // Test asset entity
        let message = create_test_message("My house is worth $500,000");
        let entities = extractor.extract_entities(&message);
        
        assert!(!entities.is_empty());
        assert_eq!(entities[0].name, "house");
        assert_eq!(entities[0].entity_type, "Asset");
        assert_eq!(entities[0].value, Some(500000.0));
        
        // Test liability entity
        let message = create_test_message("I have a mortgage of $300,000");
        let entities = extractor.extract_entities(&message);
        
        assert!(!entities.is_empty());
        assert_eq!(entities[0].name, "mortgage");
        assert_eq!(entities[0].entity_type, "Liability");
        assert_eq!(entities[0].value, Some(300000.0));
    }
    
    #[test]
    fn test_entity_updating() {
        let mut extractor = EntityExtractor::new();
        
        // Add an entity
        let message1 = create_test_message("I have a 401k");
        extractor.extract_entities(&message1);
        
        // Update the entity with a value
        let message2 = create_test_message("My 401k has $75,000 in it");
        extractor.extract_entities(&message2);
        
        // Check that the entity was updated, not duplicated
        let entities = extractor.get_all_entities();
        assert_eq!(entities.len(), 1);
        assert_eq!(entities[0].name, "401k");
        assert_eq!(entities[0].value, Some(75000.0));
    }
} 