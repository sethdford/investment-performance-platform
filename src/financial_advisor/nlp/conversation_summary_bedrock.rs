use anyhow::{Result, anyhow, Context};
use async_trait::async_trait;
use std::sync::Arc;
use chrono::Utc;

use super::bedrock::BedrockNlpClient;
use super::conversation_storage::ConversationStorageTrait;
use super::conversation_summary::{
    ConversationSummary, SummaryGenerator, SummaryType, 
    FinancialEntity, FinancialEntityType, FinancialDecision, 
    DecisionType, DecisionStatus, ConversationTopic
};

/// Prompt templates for different summary types
const SUMMARY_PROMPT_TEMPLATES: &[(&str, &str)] = &[
    ("brief", r#"You are an expert financial advisor assistant. Your task is to create a very brief summary (1-2 sentences) of a conversation between a financial advisor and a client.

Focus on:
1. Key financial details (accounts, assets, liabilities, income, expenses)
2. Financial goals mentioned
3. Decisions made or actions agreed upon

Conversation:
{conversation_text}

Please provide a brief summary that captures the essential financial information and context."#),

    ("standard", r#"You are an expert financial advisor assistant. Your task is to create a concise summary of a conversation between a financial advisor and a client.

Focus on:
1. Key financial details (accounts, assets, liabilities, income, expenses)
2. Financial goals mentioned
3. Decisions made or actions agreed upon
4. Important questions or concerns raised

Conversation:
{conversation_text}

Please provide a summary that captures the essential financial information and context."#),

    ("detailed", r#"You are an expert financial advisor assistant. Your task is to create a detailed summary of a conversation between a financial advisor and a client.

Focus on:
1. Key financial details (accounts, assets, liabilities, income, expenses)
2. Financial goals mentioned
3. Decisions made or actions agreed upon
4. Important questions or concerns raised
5. Timeline of discussion topics
6. All significant financial information mentioned

Conversation:
{conversation_text}

Please provide a detailed summary that captures all important financial information and context."#),

    ("topic_focused", r#"You are an expert financial advisor assistant. Your task is to create a summary of a conversation between a financial advisor and a client, focusing specifically on the topic of {topic}.

Focus on:
1. Key financial details related to {topic}
2. Goals mentioned related to {topic}
3. Decisions made or actions agreed upon related to {topic}
4. Important questions or concerns raised about {topic}

Conversation:
{conversation_text}

Please provide a summary that captures the essential information and context related to {topic}."#),

    ("update", r#"You are an expert financial advisor assistant. Your task is to update an existing summary with new information from a conversation between a financial advisor and a client.

Existing summary:
{existing_summary}

New conversation content:
{new_conversation_text}

Please update the summary to incorporate the new information while preserving the important context from the existing summary. Focus on:
1. Key financial details (accounts, assets, liabilities, income, expenses)
2. Financial goals mentioned
3. Decisions made or actions agreed upon
4. Important questions or concerns raised

Please provide the updated summary."#),

    ("entity_extraction", r#"You are an expert financial advisor assistant. Your task is to extract key financial entities from a conversation between a financial advisor and a client.

Conversation:
{conversation_text}

Please extract and list all financial entities mentioned in the conversation. For each entity, provide:
1. Entity type (Account, Asset, Liability, Goal, Income, Expense, Tax, Insurance, Investment, Retirement, Other)
2. Entity name or identifier
3. Any attributes mentioned (amount, rate, term, etc.)
4. Estimated importance (High, Medium, Low)

Format your response as a JSON array of objects with the following structure:
[
  {
    "entity_type": "Account",
    "name": "401(k)",
    "attributes": {
      "balance": "$250,000",
      "contribution_rate": "10%"
    },
    "importance": "High"
  }
]"#),

    ("decision_extraction", r#"You are an expert financial advisor assistant. Your task is to extract key financial decisions from a conversation between a financial advisor and a client.

Conversation:
{conversation_text}

Please extract and list all financial decisions mentioned in the conversation. For each decision, provide:
1. Decision type (Investment, Withdrawal, AssetAllocation, TaxStrategy, RetirementPlanning, InsuranceChange, BudgetAdjustment, DebtManagement, Other)
2. Description of the decision
3. Status (Proposed, Agreed, Rejected, Pending, Completed)
4. Related entities
5. Estimated importance (High, Medium, Low)

Format your response as a JSON array of objects with the following structure:
[
  {
    "decision_type": "Investment",
    "description": "Increase 401(k) contribution from 6% to 10%",
    "status": "Agreed",
    "related_entities": ["401(k)"],
    "importance": "High"
  }
]"#),

    ("topic_extraction", r#"You are an expert financial advisor assistant. Your task is to identify the main topics discussed in a conversation between a financial advisor and a client.

Conversation:
{conversation_text}

Please identify and list all financial topics discussed in the conversation. For each topic, provide:
1. Topic name
2. Confidence level (High, Medium, Low)
3. Message IDs where this topic was discussed (if available)

Format your response as a JSON array of objects with the following structure:
[
  {
    "name": "Retirement Planning",
    "confidence": "High",
    "message_ids": []
  }
]"#),
];

/// Bedrock-based summary generator
pub struct BedrockSummaryGenerator {
    /// Bedrock NLP client
    bedrock_client: Arc<BedrockNlpClient>,
    
    /// Conversation storage
    conversation_storage: Arc<dyn ConversationStorageTrait>,
}

impl BedrockSummaryGenerator {
    /// Create a new Bedrock summary generator
    pub fn new(
        bedrock_client: Arc<BedrockNlpClient>,
        conversation_storage: Arc<dyn ConversationStorageTrait>,
    ) -> Self {
        Self {
            bedrock_client,
            conversation_storage,
        }
    }
    
    /// Get the prompt template for a summary type
    fn get_prompt_template(&self, summary_type: SummaryType) -> Result<String> {
        match summary_type {
            SummaryType::Brief => Ok(SUMMARY_PROMPT_TEMPLATES
                .iter()
                .find(|(name, _)| *name == "brief")
                .map(|(_, template)| template.to_string())
                .ok_or_else(|| anyhow!("Brief summary template not found"))?),
            
            SummaryType::Standard => Ok(SUMMARY_PROMPT_TEMPLATES
                .iter()
                .find(|(name, _)| *name == "standard")
                .map(|(_, template)| template.to_string())
                .ok_or_else(|| anyhow!("Standard summary template not found"))?),
            
            SummaryType::Detailed => Ok(SUMMARY_PROMPT_TEMPLATES
                .iter()
                .find(|(name, _)| *name == "detailed")
                .map(|(_, template)| template.to_string())
                .ok_or_else(|| anyhow!("Detailed summary template not found"))?),
            
            SummaryType::TopicFocused(topic) => {
                let template = SUMMARY_PROMPT_TEMPLATES
                    .iter()
                    .find(|(name, _)| *name == "topic_focused")
                    .map(|(_, template)| template.to_string())
                    .ok_or_else(|| anyhow!("Topic-focused summary template not found"))?;
                
                Ok(template.replace("{topic}", &topic))
            }
        }
    }
    
    /// Helper method to format conversation messages for the prompt
    async fn format_conversation_text(&self, conversation_id: &str) -> Result<String> {
        let conversation = self.conversation_storage.load_conversation(conversation_id)
            .await
            .context("Failed to get conversation")?
            .ok_or_else(|| anyhow!("No conversation found for ID"))?;
        
        if conversation.messages.is_empty() {
            return Err(anyhow!("No messages found for conversation"));
        }
        
        let formatted_messages = conversation.messages.iter()
            .map(|msg| format!("{}: {}", msg.role, msg.content))
            .collect::<Vec<_>>()
            .join("\n\n");
        
        Ok(formatted_messages)
    }
    
    /// Extract entities from conversation text using Bedrock
    async fn extract_entities(&self, conversation_text: &str) -> Result<Vec<FinancialEntity>> {
        let template = SUMMARY_PROMPT_TEMPLATES
            .iter()
            .find(|(name, _)| *name == "entity_extraction")
            .map(|(_, template)| template.to_string())
            .ok_or_else(|| anyhow!("Entity extraction template not found"))?;
        
        let prompt = template.replace("{conversation_text}", conversation_text);
        
        let response = self.bedrock_client.generate_text(&prompt).await?;
        
        // Parse the JSON response
        let entities_json = serde_json::from_str::<Vec<serde_json::Value>>(&response)
            .context("Failed to parse entity extraction response as JSON")?;
        
        let mut entities = Vec::new();
        
        for entity_json in entities_json {
            let entity_type = match entity_json["entity_type"].as_str() {
                Some("Account") => FinancialEntityType::Account,
                Some("Asset") => FinancialEntityType::Asset,
                Some("Liability") => FinancialEntityType::Liability,
                Some("Goal") => FinancialEntityType::Goal,
                Some("Income") => FinancialEntityType::Income,
                Some("Expense") => FinancialEntityType::Expense,
                Some("Tax") => FinancialEntityType::Tax,
                Some("Insurance") => FinancialEntityType::Insurance,
                Some("Investment") => FinancialEntityType::Investment,
                Some("Retirement") => FinancialEntityType::Retirement,
                Some(other) => FinancialEntityType::Other(other.to_string()),
                None => continue,
            };
            
            let name = match entity_json["name"].as_str() {
                Some(name) => name.to_string(),
                None => continue,
            };
            
            let mut attributes = std::collections::HashMap::new();
            if let Some(attrs) = entity_json["attributes"].as_object() {
                for (key, value) in attrs {
                    if let Some(value_str) = value.as_str() {
                        attributes.insert(key.clone(), value_str.to_string());
                    }
                }
            }
            
            let importance = match entity_json["importance"].as_str() {
                Some("High") => 0.9,
                Some("Medium") => 0.6,
                Some("Low") => 0.3,
                _ => 0.5,
            };
            
            entities.push(FinancialEntity {
                entity_type,
                name,
                attributes,
                importance,
            });
        }
        
        Ok(entities)
    }
    
    /// Extract decisions from conversation text using Bedrock
    async fn extract_decisions(&self, conversation_text: &str) -> Result<Vec<FinancialDecision>> {
        let template = SUMMARY_PROMPT_TEMPLATES
            .iter()
            .find(|(name, _)| *name == "decision_extraction")
            .map(|(_, template)| template.to_string())
            .ok_or_else(|| anyhow!("Decision extraction template not found"))?;
        
        let prompt = template.replace("{conversation_text}", conversation_text);
        
        let response = self.bedrock_client.generate_text(&prompt).await?;
        
        // Parse the JSON response
        let decisions_json = serde_json::from_str::<Vec<serde_json::Value>>(&response)
            .context("Failed to parse decision extraction response as JSON")?;
        
        let mut decisions = Vec::new();
        
        for decision_json in decisions_json {
            let decision_type = match decision_json["decision_type"].as_str() {
                Some("Investment") => DecisionType::Investment,
                Some("Withdrawal") => DecisionType::Withdrawal,
                Some("AssetAllocation") => DecisionType::AssetAllocation,
                Some("TaxStrategy") => DecisionType::TaxStrategy,
                Some("RetirementPlanning") => DecisionType::RetirementPlanning,
                Some("InsuranceChange") => DecisionType::InsuranceChange,
                Some("BudgetAdjustment") => DecisionType::BudgetAdjustment,
                Some("DebtManagement") => DecisionType::DebtManagement,
                Some(other) => DecisionType::Other(other.to_string()),
                None => continue,
            };
            
            let description = match decision_json["description"].as_str() {
                Some(desc) => desc.to_string(),
                None => continue,
            };
            
            let status = match decision_json["status"].as_str() {
                Some("Proposed") => DecisionStatus::Proposed,
                Some("Agreed") => DecisionStatus::Agreed,
                Some("Rejected") => DecisionStatus::Rejected,
                Some("Pending") => DecisionStatus::Pending,
                Some("Completed") => DecisionStatus::Completed,
                _ => DecisionStatus::Proposed,
            };
            
            let mut related_entities = Vec::new();
            if let Some(entities) = decision_json["related_entities"].as_array() {
                for entity in entities {
                    if let Some(entity_str) = entity.as_str() {
                        related_entities.push(entity_str.to_string());
                    }
                }
            }
            
            let importance = match decision_json["importance"].as_str() {
                Some("High") => 0.9,
                Some("Medium") => 0.6,
                Some("Low") => 0.3,
                _ => 0.5,
            };
            
            decisions.push(FinancialDecision {
                decision_type,
                description,
                status,
                related_entities,
                importance,
            });
        }
        
        Ok(decisions)
    }
    
    /// Extract topics from conversation text using Bedrock
    async fn extract_topics(&self, conversation_text: &str) -> Result<Vec<ConversationTopic>> {
        let template = SUMMARY_PROMPT_TEMPLATES
            .iter()
            .find(|(name, _)| *name == "topic_extraction")
            .map(|(_, template)| template.to_string())
            .ok_or_else(|| anyhow!("Topic extraction template not found"))?;
        
        let prompt = template.replace("{conversation_text}", conversation_text);
        
        let response = self.bedrock_client.generate_text(&prompt).await?;
        
        // Parse the JSON response
        let topics_json = serde_json::from_str::<Vec<serde_json::Value>>(&response)
            .context("Failed to parse topic extraction response as JSON")?;
        
        let mut topics = Vec::new();
        
        for topic_json in topics_json {
            let name = match topic_json["name"].as_str() {
                Some(name) => name.to_string(),
                None => continue,
            };
            
            let confidence = match topic_json["confidence"].as_str() {
                Some("High") => 0.9,
                Some("Medium") => 0.6,
                Some("Low") => 0.3,
                _ => 0.5,
            };
            
            let mut message_ids = Vec::new();
            if let Some(ids) = topic_json["message_ids"].as_array() {
                for id in ids {
                    if let Some(id_str) = id.as_str() {
                        message_ids.push(id_str.to_string());
                    }
                }
            }
            
            topics.push(ConversationTopic {
                name,
                confidence,
                message_ids,
            });
        }
        
        Ok(topics)
    }
}

#[async_trait]
impl SummaryGenerator for BedrockSummaryGenerator {
    fn generate_summary<'a>(
        &'a self,
        conversation_id: &'a str,
        summary_type: SummaryType,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<ConversationSummary>> + Send + 'a>> {
        Box::pin(async move {
            // Get the conversation
            let conversation = self.conversation_storage.load_conversation(conversation_id).await?
                .ok_or_else(|| anyhow!("Conversation not found"))?;
            
            if conversation.messages.is_empty() {
                return Err(anyhow!("No messages found for conversation"));
            }
            
            // Format the conversation text
            let conversation_text = self.format_conversation_text(conversation_id).await?;
            
            // Get the appropriate prompt template
            let template_str = match summary_type {
                SummaryType::Brief => {
                    SUMMARY_PROMPT_TEMPLATES.iter()
                        .find(|(name, _)| *name == "brief")
                        .map(|(_, template)| (*template).to_string())
                        .ok_or_else(|| anyhow!("Brief summary template not found"))?
                },
                SummaryType::Standard => {
                    SUMMARY_PROMPT_TEMPLATES.iter()
                        .find(|(name, _)| *name == "standard")
                        .map(|(_, template)| (*template).to_string())
                        .ok_or_else(|| anyhow!("Standard summary template not found"))?
                },
                SummaryType::Detailed => {
                    SUMMARY_PROMPT_TEMPLATES.iter()
                        .find(|(name, _)| *name == "detailed")
                        .map(|(_, template)| (*template).to_string())
                        .ok_or_else(|| anyhow!("Detailed summary template not found"))?
                },
                SummaryType::TopicFocused(topic) => {
                    // For topic-focused summaries, we'll use a custom prompt
                    let topic_template = SUMMARY_PROMPT_TEMPLATES.iter()
                        .find(|(name, _)| *name == "topic")
                        .map(|(_, template)| (*template).to_string())
                        .ok_or_else(|| anyhow!("Topic summary template not found"))?;
                    
                    // Replace the topic placeholder
                    topic_template.replace("{topic}", &topic)
                }
            };
            
            // Replace placeholders in the prompt
            let prompt = template_str.replace("{conversation_text}", &conversation_text);
            
            // Call the LLM to generate the summary
            let summary_text = self.bedrock_client.generate_text(&prompt).await?;
            
            // Get the last message ID
            let last_message_id = conversation.messages.last()
                .map(|msg| msg.id.clone())
                .ok_or_else(|| anyhow!("No messages found"))?;
            
            // Create the summary
            let mut summary = ConversationSummary::new(
                conversation_id.to_string(),
                summary_text,
                last_message_id,
            );
            
            // Extract and add entities
            let entities = self.extract_entities(&conversation_text).await?;
            for entity in entities {
                summary.add_entity(entity);
            }
            
            // Extract and add decisions
            let decisions = self.extract_decisions(&conversation_text).await?;
            for decision in decisions {
                summary.add_decision(decision);
            }
            
            // Extract and add topics
            let topics = self.extract_topics(&conversation_text).await?;
            for topic in topics {
                summary.add_topic(topic);
            }
            
            Ok(summary)
        })
    }
    
    fn update_summary<'a>(
        &'a self,
        summary_id: &'a str,
        _new_message_ids: &'a [String],
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<ConversationSummary>> + Send + 'a>> {
        Box::pin(async move {
            // Extract conversation_id from summary_id (format: "conversation_id-summary")
            let parts: Vec<&str> = summary_id.split('-').collect();
            if parts.len() < 2 {
                return Err(anyhow!("Invalid summary ID format"));
            }
            let conversation_id = parts[0];
            
            // Load the conversation to get all messages
            let conversation = self.conversation_storage.load_conversation(conversation_id).await?
                .ok_or_else(|| anyhow!("Conversation not found"))?;
            
            // Get the last message ID
            let last_message_id = conversation.messages.last()
                .map(|msg| msg.id.clone())
                .ok_or_else(|| anyhow!("No messages found"))?;
            
            // Generate a new summary
            // For simplicity, we're regenerating the entire summary
            // A more sophisticated implementation would update only the relevant parts
            let summary_type = SummaryType::Standard; // Default to standard summary
            let mut summary = self.generate_summary(conversation_id, summary_type).await?;
            
            // Update the ID to match the original summary ID
            summary.id = summary_id.to_string();
            summary.last_message_id = last_message_id;
            
            Ok(summary)
        })
    }
    
    fn generate_topic_summary<'a>(
        &'a self,
        conversation_id: &'a str,
        topics: &'a [String],
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<ConversationSummary>> + Send + 'a>> {
        Box::pin(async move {
            // Format the conversation text
            let conversation_text = self.format_conversation_text(conversation_id).await?;
            
            // Load the conversation to get all messages
            let conversation = self.conversation_storage.load_conversation(conversation_id).await?
                .ok_or_else(|| anyhow!("Conversation not found"))?;
            
            // Get the last message ID
            let last_message_id = conversation.messages.last()
                .map(|msg| msg.id.clone())
                .ok_or_else(|| anyhow!("No messages found"))?;
            
            // Create a topic-focused prompt
            let topics_str = topics.join(", ");
            let prompt = format!(
                r#"You are an expert financial advisor assistant. Your task is to create a focused summary of a conversation between a financial advisor and a client, specifically about the following topics: {}.

Focus on:
1. Information related to these specific topics
2. Financial details relevant to these topics
3. Decisions or actions related to these topics

Conversation:
{}

Please provide a focused summary that captures the essential information about these topics."#,
                topics_str, conversation_text
            );
            
            // Generate the summary text using Bedrock
            let summary_text = self.bedrock_client.generate_text(&prompt)
                .await
                .context("Failed to generate summary text")?;
            
            // Extract entities, decisions, and topics
            let entities = self.extract_entities(&conversation_text).await?;
            let decisions = self.extract_decisions(&conversation_text).await?;
            let extracted_topics = self.extract_topics(&conversation_text).await?;
            
            // Filter to only include topics that were requested
            let filtered_topics = extracted_topics.into_iter()
                .filter(|topic| topics.iter().any(|t| topic.name.to_lowercase().contains(&t.to_lowercase())))
                .collect();
            
            // Create the summary
            let summary = ConversationSummary {
                id: format!("{}-topic-summary", conversation_id),
                conversation_id: conversation_id.to_string(),
                content: summary_text,
                version: 1,
                created_at: Utc::now(),
                updated_at: Utc::now(),
                last_message_id,
                key_entities: entities,
                key_decisions: decisions,
                topics: filtered_topics,
            };
            
            Ok(summary)
        })
    }
} 