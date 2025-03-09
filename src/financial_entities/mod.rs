use std::collections::HashMap;
use serde::{Serialize, Deserialize};
use chrono::{DateTime, Utc};

pub mod entity_extractor;

/// Represents a financial entity mentioned in a conversation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FinancialEntity {
    /// Unique identifier for the entity
    pub id: String,
    /// Name of the entity
    pub name: String,
    /// Type of the entity (e.g., "Account", "Asset", "Liability")
    pub entity_type: String,
    /// Numeric value associated with the entity, if any
    pub value: Option<f64>,
    /// Additional metadata about the entity
    pub metadata: HashMap<String, String>,
    /// When the entity was first mentioned
    pub first_mentioned: DateTime<Utc>,
    /// When the entity was last mentioned
    pub last_mentioned: DateTime<Utc>,
    /// Recency score (0.0 to 1.0) - higher means more recent
    pub recency_score: f32,
}

impl FinancialEntity {
    /// Creates a new financial entity
    pub fn new(name: &str, entity_type: &str) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            name: name.to_string(),
            entity_type: entity_type.to_string(),
            value: None,
            metadata: HashMap::new(),
            first_mentioned: Utc::now(),
            last_mentioned: Utc::now(),
            recency_score: 1.0, // New entities start with maximum recency
        }
    }
    
    /// Creates a new financial entity with a value
    pub fn with_value(name: &str, entity_type: &str, value: f64) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            name: name.to_string(),
            entity_type: entity_type.to_string(),
            value: Some(value),
            metadata: HashMap::new(),
            first_mentioned: Utc::now(),
            last_mentioned: Utc::now(),
            recency_score: 1.0, // New entities start with maximum recency
        }
    }
    
    /// Adds metadata to the entity
    pub fn add_metadata(&mut self, key: &str, value: &str) {
        self.metadata.insert(key.to_string(), value.to_string());
    }
    
    /// Updates the entity's value
    pub fn set_value(&mut self, value: f64) {
        self.value = Some(value);
    }
    
    /// Updates the last mentioned timestamp
    pub fn update_last_mentioned(&mut self) {
        self.last_mentioned = Utc::now();
    }
    
    /// Gets the entity's age in seconds
    pub fn age_seconds(&self) -> i64 {
        Utc::now().timestamp() - self.first_mentioned.timestamp()
    }
    
    /// Gets the time since last mentioned in seconds
    pub fn time_since_last_mentioned_seconds(&self) -> i64 {
        Utc::now().timestamp() - self.last_mentioned.timestamp()
    }
    
    /// Formats the entity's value as a currency string
    pub fn format_value_as_currency(&self) -> String {
        match self.value {
            Some(value) => format!("${:.2}", value),
            None => "N/A".to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread::sleep;
    use std::time::Duration;
    
    #[test]
    fn test_entity_creation() {
        let entity = FinancialEntity::new("401k", "Account");
        assert_eq!(entity.name, "401k");
        assert_eq!(entity.entity_type, "Account");
        assert_eq!(entity.value, None);
        assert!(entity.metadata.is_empty());
    }
    
    #[test]
    fn test_entity_with_value() {
        let entity = FinancialEntity::with_value("401k", "Account", 50000.0);
        assert_eq!(entity.name, "401k");
        assert_eq!(entity.entity_type, "Account");
        assert_eq!(entity.value, Some(50000.0));
        assert!(entity.metadata.is_empty());
    }
    
    #[test]
    fn test_add_metadata() {
        let mut entity = FinancialEntity::new("401k", "Account");
        entity.add_metadata("provider", "Fidelity");
        entity.add_metadata("account_type", "Traditional");
        
        assert_eq!(entity.metadata.get("provider"), Some(&"Fidelity".to_string()));
        assert_eq!(entity.metadata.get("account_type"), Some(&"Traditional".to_string()));
    }
    
    #[test]
    fn test_update_last_mentioned() {
        let mut entity = FinancialEntity::new("401k", "Account");
        let original_timestamp = entity.last_mentioned;
        
        // Sleep to ensure timestamp changes
        sleep(Duration::from_millis(10));
        
        entity.update_last_mentioned();
        assert!(entity.last_mentioned > original_timestamp);
    }
    
    #[test]
    fn test_format_value_as_currency() {
        let entity1 = FinancialEntity::with_value("401k", "Account", 50000.0);
        let entity2 = FinancialEntity::new("Roth IRA", "Account");
        
        assert_eq!(entity1.format_value_as_currency(), "$50000.00");
        assert_eq!(entity2.format_value_as_currency(), "N/A");
    }
} 