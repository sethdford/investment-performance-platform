use anyhow::{Result, anyhow};
use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Arc;

use super::bedrock::BedrockNlpClient;
use super::conversation_storage::ConversationStorageTrait;
use super::conversation_summary::{
    EntityExtractor, ImportanceScorer, FinancialEntity, FinancialDecision
};

/// LLM-based entity extractor
pub struct LlmEntityExtractor {
    /// Bedrock NLP client
    bedrock_client: Arc<BedrockNlpClient>,
    
    /// Conversation storage
    conversation_storage: Arc<dyn ConversationStorageTrait>,
}

impl LlmEntityExtractor {
    /// Create a new LLM entity extractor
    pub fn new(
        bedrock_client: Arc<BedrockNlpClient>,
        conversation_storage: Arc<dyn ConversationStorageTrait>,
    ) -> Self {
        Self {
            bedrock_client,
            conversation_storage,
        }
    }
    
    /// Format conversation turns into text for the prompt
    async fn format_conversation_text(&self, conversation_id: &str) -> Result<String> {
        let conversation = self.conversation_storage.load_conversation(conversation_id).await?
            .ok_or_else(|| anyhow!("Conversation not found"))?;
        
        let mut formatted_text = String::new();
        
        for message in conversation.messages.iter() {
            match message.role.as_str() {
                "user" => {
                    formatted_text.push_str(&format!("Human: {}\n\n", message.content));
                }
                "assistant" => {
                    formatted_text.push_str(&format!("Assistant: {}\n\n", message.content));
                }
                _ => {
                    formatted_text.push_str(&format!("{}: {}\n\n", message.role, message.content));
                }
            }
        }
        
        Ok(formatted_text)
    }

    // Fix the unused prompt variable
    fn extract_entities_from_text(&self, text: &str) -> Result<Vec<FinancialEntity>> {
        // Implementation
        let _prompt = format!(
            "Extract financial entities from the following text:\n\n{}",
            text
        );
        
        // Return a placeholder result
        Ok(Vec::new())
    }
}

#[async_trait]
impl EntityExtractor for LlmEntityExtractor {
    fn extract_entities<'a>(
        &'a self,
        conversation_id: &'a str,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<Vec<FinancialEntity>>> + Send + 'a>> {
        Box::pin(async move {
            let conversation_text = self.format_conversation_text(conversation_id).await?;
            
            let _prompt = format!(
                r#"Extract all financial entities from the following conversation. 
                Return the result as a JSON array of objects with the following structure:
                [
                    {{
                        "entity_type": "Account|Asset|Liability|Goal|Income|Expense|Tax|Insurance|Investment|Retirement|Other",
                        "name": "entity name",
                        "attributes": {{"key1": "value1", "key2": "value2"}},
                        "importance": 0.5
                    }}
                ]
                
                Conversation:
                {}"#,
                conversation_text
            );
            
            // Return a placeholder result
            Ok(Vec::new())
        })
    }
    
    fn extract_entities_from_messages<'a>(
        &'a self,
        _message_ids: &'a [String],
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<Vec<FinancialEntity>>> + Send + 'a>> {
        Box::pin(async move {
            // Return a placeholder result with a sample entity
            let entities = vec![
                FinancialEntity {
                    entity_type: super::conversation_summary::FinancialEntityType::Investment,
                    name: "Stock Portfolio".to_string(),
                    attributes: {
                        let mut map = HashMap::new();
                        map.insert("allocation".to_string(), "70% stocks, 30% bonds".to_string());
                        map
                    },
                    importance: 0.7,
                },
            ];
            
            Ok(entities)
        })
    }
}

/// Rule-based importance scorer
pub struct RuleBasedImportanceScorer {
    /// Importance rules
    rules: Vec<ImportanceRule>,
}

/// Importance rule
struct ImportanceRule {
    /// Entity type to match
    entity_type: Option<super::conversation_summary::FinancialEntityType>,
    
    /// Entity name pattern to match
    name_pattern: Option<String>,
    
    /// Attribute key to match
    attribute_key: Option<String>,
    
    /// Attribute value pattern to match
    attribute_value_pattern: Option<String>,
    
    /// Importance score to assign
    importance: f32,
}

impl ImportanceRule {
    /// Check if an entity matches this rule
    fn matches(&self, entity: &FinancialEntity) -> bool {
        // Check entity type if specified
        if let Some(ref rule_type) = self.entity_type {
            if !matches_entity_type(rule_type, &entity.entity_type) {
                return false;
            }
        }
        
        // Check name pattern if specified
        if let Some(ref pattern) = self.name_pattern {
            if !entity.name.to_lowercase().contains(&pattern.to_lowercase()) {
                return false;
            }
        }
        
        // Check attribute key and value if specified
        if let (Some(ref key), Some(ref value_pattern)) = (&self.attribute_key, &self.attribute_value_pattern) {
            if let Some(value) = entity.attributes.get(key) {
                if !value.to_lowercase().contains(&value_pattern.to_lowercase()) {
                    return false;
                }
            } else {
                return false;
            }
        } else if let Some(ref key) = self.attribute_key {
            if !entity.attributes.contains_key(key) {
                return false;
            }
        }
        
        true
    }
}

impl RuleBasedImportanceScorer {
    /// Create a new rule-based importance scorer with default rules
    pub fn new() -> Self {
        let mut rules = Vec::new();
        
        // High importance for retirement accounts
        rules.push(ImportanceRule {
            entity_type: Some(super::conversation_summary::FinancialEntityType::Account),
            name_pattern: Some("401(k)".to_string()),
            attribute_key: None,
            attribute_value_pattern: None,
            importance: 0.9,
        });
        
        rules.push(ImportanceRule {
            entity_type: Some(super::conversation_summary::FinancialEntityType::Account),
            name_pattern: Some("IRA".to_string()),
            attribute_key: None,
            attribute_value_pattern: None,
            importance: 0.9,
        });
        
        // High importance for high-value assets
        rules.push(ImportanceRule {
            entity_type: Some(super::conversation_summary::FinancialEntityType::Asset),
            name_pattern: None,
            attribute_key: Some("value".to_string()),
            attribute_value_pattern: Some("\\$[0-9,]+,000".to_string()),
            importance: 0.9,
        });
        
        // High importance for retirement goals
        rules.push(ImportanceRule {
            entity_type: Some(super::conversation_summary::FinancialEntityType::Goal),
            name_pattern: Some("retirement".to_string()),
            attribute_key: None,
            attribute_value_pattern: None,
            importance: 0.9,
        });
        
        // Medium importance for other goals
        rules.push(ImportanceRule {
            entity_type: Some(super::conversation_summary::FinancialEntityType::Goal),
            name_pattern: None,
            attribute_key: None,
            attribute_value_pattern: None,
            importance: 0.6,
        });
        
        // Medium importance for income
        rules.push(ImportanceRule {
            entity_type: Some(super::conversation_summary::FinancialEntityType::Income),
            name_pattern: None,
            attribute_key: None,
            attribute_value_pattern: None,
            importance: 0.7,
        });
        
        // Medium importance for expenses
        rules.push(ImportanceRule {
            entity_type: Some(super::conversation_summary::FinancialEntityType::Expense),
            name_pattern: None,
            attribute_key: None,
            attribute_value_pattern: None,
            importance: 0.6,
        });
        
        // High importance for tax strategies
        rules.push(ImportanceRule {
            entity_type: Some(super::conversation_summary::FinancialEntityType::Tax),
            name_pattern: None,
            attribute_key: None,
            attribute_value_pattern: None,
            importance: 0.8,
        });
        
        Self { rules }
    }
    
    /// Check if an entity matches a rule
    fn matches_rule(&self, entity: &FinancialEntity, rule: &ImportanceRule) -> bool {
        // Check entity type
        if let Some(ref rule_type) = rule.entity_type {
            if !matches_entity_type(rule_type, &entity.entity_type) {
                return false;
            }
        }
        
        // Check name pattern
        if let Some(ref pattern) = rule.name_pattern {
            if !entity.name.to_lowercase().contains(&pattern.to_lowercase()) {
                return false;
            }
        }
        
        // Check attribute key and value
        if let Some(ref key) = rule.attribute_key {
            if !entity.attributes.contains_key(key) {
                return false;
            }
            
            if let Some(ref value_pattern) = rule.attribute_value_pattern {
                if let Some(value) = entity.attributes.get(key) {
                    if !value.to_lowercase().contains(&value_pattern.to_lowercase()) {
                        return false;
                    }
                } else {
                    return false;
                }
            }
        }
        
        true
    }
}

/// Check if two entity types match
fn matches_entity_type(
    rule_type: &super::conversation_summary::FinancialEntityType,
    entity_type: &super::conversation_summary::FinancialEntityType,
) -> bool {
    match (rule_type, entity_type) {
        (
            super::conversation_summary::FinancialEntityType::Other(rule_name),
            super::conversation_summary::FinancialEntityType::Other(entity_name),
        ) => rule_name.to_lowercase() == entity_name.to_lowercase(),
        (a, b) => std::mem::discriminant(a) == std::mem::discriminant(b),
    }
}

#[async_trait]
impl ImportanceScorer for RuleBasedImportanceScorer {
    fn score_entities<'a>(
        &'a self,
        entities: &'a [FinancialEntity],
        _conversation_id: &'a str,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<Vec<FinancialEntity>>> + Send + 'a>> {
        Box::pin(async move {
            let mut scored_entities = Vec::new();
            
            for entity in entities {
                let mut max_importance: f32 = 0.5; // Default importance
                
                // Apply rules to determine importance
                for rule in &self.rules {
                    if rule.matches(entity) {
                        max_importance = max_importance.max(rule.importance);
                    }
                }
                
                // Create a new entity with the updated importance
                let mut scored_entity = entity.clone();
                scored_entity.importance = max_importance;
                scored_entities.push(scored_entity);
            }
            
            Ok(scored_entities)
        })
    }
    
    fn score_decisions<'a>(
        &'a self,
        decisions: &'a [FinancialDecision],
        _conversation_id: &'a str,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<Vec<FinancialDecision>>> + Send + 'a>> {
        Box::pin(async move {
            // This is a simplified implementation that doesn't modify decisions
            // A more sophisticated implementation would apply rules to decisions
            let scored_decisions = decisions.to_vec();
            Ok(scored_decisions)
        })
    }
} 