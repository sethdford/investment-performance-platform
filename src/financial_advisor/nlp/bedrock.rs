use anyhow::{Result, anyhow, Context};
use aws_sdk_bedrockruntime::Client as BedrockRuntimeClient;
use aws_sdk_bedrockruntime::error::ProvideErrorMetadata;
use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use tracing::{warn, error};
use tokio::time::Duration;

use super::rule_based::FinancialQueryIntent;
use super::types::{ValidatedLlmResponse, ClientData, ValidatedEntity};

/// Bedrock model provider
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BedrockModelProvider {
    /// Amazon Titan
    Titan,
    
    /// Anthropic Claude
    Claude,
    
    /// AI21 Labs Jurassic
    AI21,
    
    /// Cohere Command
    Cohere,
    
    /// Meta Llama
    Llama,
}

impl BedrockModelProvider {
    /// Get the model ID for a provider and version
    pub fn model_id(&self, version: &str) -> String {
        match self {
            BedrockModelProvider::Titan => format!("amazon.titan-{}", version),
            BedrockModelProvider::Claude => format!("anthropic.claude-{}", version),
            BedrockModelProvider::AI21 => format!("ai21.j2-{}", version),
            BedrockModelProvider::Cohere => format!("cohere.command-{}", version),
            BedrockModelProvider::Llama => format!("meta.llama-{}", version),
        }
    }
}

/// Bedrock NLP client configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BedrockNlpConfig {
    /// Model provider
    pub provider: BedrockModelProvider,
    
    /// Model version
    pub version: String,
    
    /// Temperature (0.0-1.0)
    pub temperature: f64,
    
    /// Maximum tokens to generate
    pub max_tokens: u32,
    
    /// Top-p sampling
    pub top_p: f64,
    
    /// Top-k sampling
    pub top_k: u32,
    
    /// Stop sequences
    pub stop_sequences: Vec<String>,
}

impl Default for BedrockNlpConfig {
    fn default() -> Self {
        Self {
            provider: BedrockModelProvider::Claude,
            version: "v2".to_string(),
            temperature: 0.2,
            max_tokens: 1000,
            top_p: 0.9,
            top_k: 250,
            stop_sequences: vec!["\n\nHuman:".to_string()],
        }
    }
}

/// Bedrock NLP client
pub struct BedrockNlpClient {
    /// Bedrock runtime client
    client: BedrockRuntimeClient,
    
    /// Configuration
    config: BedrockNlpConfig,
    
    /// Prompt templates
    prompt_templates: HashMap<String, String>,
}

impl BedrockNlpClient {
    /// Create a new Bedrock NLP client
    pub fn new(client: BedrockRuntimeClient, config: BedrockNlpConfig) -> Self {
        let mut prompt_templates = HashMap::new();
        
        // Add default prompt templates
        prompt_templates.insert(
            "intent_classification".to_string(),
            include_str!("prompts/intent_classification.txt").to_string(),
        );
        
        prompt_templates.insert(
            "entity_extraction".to_string(),
            include_str!("prompts/entity_extraction.txt").to_string(),
        );
        
        prompt_templates.insert(
            "response_generation".to_string(),
            include_str!("prompts/response_generation.txt").to_string(),
        );
        
        Self {
            client,
            config,
            prompt_templates,
        }
    }
    
    /// Process a query with the Bedrock NLP client
    pub async fn process_query(&self, _query: &str) -> Result<ValidatedLlmResponse> {
        // In a real implementation, this would call the Bedrock API
        // For now, we'll return a mock response
        
        let intent = FinancialQueryIntent::PortfolioPerformance;
        let intent_confidence = 0.85;
        
        Ok(ValidatedLlmResponse {
            intent,
            intent_confidence,
            entities: vec![],
            is_uncertain: false,
            response_text: Some("Your portfolio is up 5% this year.".to_string()),
            explanation: None,
        })
    }
    
    /// Classify the intent of a query
    pub async fn classify_intent(&self, query: &str, _context: &str) -> Result<ValidatedLlmResponse> {
        // In a real implementation, this would call the Bedrock API with the intent classification prompt
        // For now, we'll return a mock response
        
        let intent = if query.contains("portfolio") || query.contains("performance") {
            FinancialQueryIntent::PortfolioPerformance
        } else if query.contains("retire") || query.contains("retirement") {
            FinancialQueryIntent::RetirementPlanning
        } else if query.contains("tax") {
            FinancialQueryIntent::TaxOptimization
        } else if query.contains("asset") || query.contains("allocation") {
            FinancialQueryIntent::AssetAllocation
        } else if query.contains("goal") || query.contains("progress") {
            FinancialQueryIntent::GoalProgress
        } else {
            FinancialQueryIntent::FinancialEducation
        };
        
        let intent_confidence = 0.85;
        
        Ok(ValidatedLlmResponse {
            intent,
            intent_confidence,
            entities: vec![],
            is_uncertain: false,
            response_text: None,
            explanation: None,
        })
    }
    
    /// Generate a response for a query
    pub async fn generate_response(
        &self, 
        query: &str, 
        intent: &FinancialQueryIntent, 
        entities: &[ValidatedEntity], 
        client_data: Option<&ClientData>, 
        context: &str
    ) -> Result<String> {
        // Build a prompt for the LLM that includes all relevant information
        let mut prompt = String::new();
        
        // Add system instructions
        prompt.push_str("You are a professional financial advisor assistant. Provide helpful, accurate, and personalized financial advice.\n\n");
        
        // Add context if available
        if !context.is_empty() {
            prompt.push_str("Conversation context:\n");
            prompt.push_str(context);
            prompt.push_str("\n\n");
        }
        
        // Add client data if available
        if let Some(data) = client_data {
            prompt.push_str("Client information:\n");
            
            if let Some(name) = &data.client_name {
                prompt.push_str(&format!("Name: {}\n", name));
            }
            
            if let Some(portfolio) = &data.portfolio {
                prompt.push_str(&format!("Portfolio value: ${:.2}\n", portfolio.total_value));
                
                if !portfolio.asset_allocation.is_empty() {
                    prompt.push_str("Asset allocation:\n");
                    for asset in &portfolio.asset_allocation {
                        prompt.push_str(&format!("- {}: {:.1}%\n", asset.asset_class, asset.allocation_percentage * 100.0));
                    }
                }
            }
            
            if let Some(goals) = &data.goals {
                if !goals.is_empty() {
                    prompt.push_str("Financial goals:\n");
                    for goal in goals {
                        prompt.push_str(&format!("- {}: ${:.2}", goal.goal_type, goal.target_amount));
                        prompt.push_str(&format!(" by {}", goal.target_date));
                        prompt.push_str("\n");
                    }
                }
            }
            
            if let Some(cash_flow) = &data.cash_flow {
                prompt.push_str(&format!("Monthly income: ${:.2}\n", cash_flow.monthly_income));
                prompt.push_str(&format!("Monthly expenses: ${:.2}\n", cash_flow.monthly_expenses));
            }
            
            prompt.push_str("\n");
        }
        
        // Add recognized intent
        prompt.push_str(&format!("Recognized intent: {:?}\n\n", intent));
        
        // Add extracted entities
        if !entities.is_empty() {
            prompt.push_str("Extracted entities:\n");
            for entity in entities {
                prompt.push_str(&format!("- {:?}: {} (confidence: {:.2})\n", 
                    entity.entity_type, entity.value, entity.confidence));
            }
            prompt.push_str("\n");
        }
        
        // Add the user query
        prompt.push_str(&format!("User query: {}\n\n", query));
        
        // Add specific instructions based on intent
        match intent {
            FinancialQueryIntent::Greeting => {
                prompt.push_str("Respond with a friendly greeting. ");
                prompt.push_str("Introduce yourself as a financial advisor assistant and ask how you can help with financial planning.\n\n");
            },
            FinancialQueryIntent::PortfolioPerformance => {
                prompt.push_str("Provide a detailed analysis of the client's portfolio performance. ");
                prompt.push_str("Include information about returns, asset allocation, and any recommendations for improvement.\n\n");
            },
            FinancialQueryIntent::RetirementPlanning => {
                prompt.push_str("Provide retirement planning advice. ");
                prompt.push_str("Consider the client's current savings, retirement goals, and time horizon.\n\n");
            },
            FinancialQueryIntent::TaxOptimization => {
                prompt.push_str("Provide tax optimization strategies. ");
                prompt.push_str("Consider the client's income, investments, and tax situation.\n\n");
            },
            FinancialQueryIntent::GoalProgress => {
                prompt.push_str("Provide an analysis of the client's progress toward their financial goals. ");
                prompt.push_str("Consider their current savings, investment strategy, and time horizon.\n\n");
            },
            FinancialQueryIntent::Unknown => {
                // Check if the query is asking for personal information
                if query.to_lowercase().contains("my name") || query.to_lowercase().contains("who am i") {
                    prompt.push_str("The user is asking for personal information that you don't have access to. ");
                    prompt.push_str("Politely explain that you don't have access to their personal information and redirect the conversation to financial topics. ");
                    prompt.push_str("Offer to help with financial planning or investment questions instead.\n\n");
                } else {
                    prompt.push_str("The user's query doesn't match any specific financial intent. ");
                    prompt.push_str("Politely explain that you're designed to help with financial planning and investment questions. ");
                    prompt.push_str("Ask them to rephrase their question or suggest some financial topics you can help with.\n\n");
                }
            },
            _ => {
                prompt.push_str(&format!("Provide helpful information about {:?}. ", intent));
                prompt.push_str("Be specific, accurate, and tailored to the client's situation if possible.\n\n");
            }
        }
        
        prompt.push_str("Response:");
        
        // Call the LLM to generate a response
        let response = self.invoke_model(prompt).await?;
        
        Ok(response.trim().to_string())
    }
    
    /// Create a prompt from a template
    fn create_prompt(&self, template_name: &str, variables: &[(String, String)]) -> Result<String> {
        let template = self.prompt_templates.get(template_name)
            .ok_or_else(|| anyhow!("Prompt template not found: {}", template_name))?;
        
        let mut prompt = template.clone();
        for (key, value) in variables {
            prompt = prompt.replace(&format!("{{{}}}", key), value);
        }
        
        Ok(prompt)
    }
    
    /// Invoke the Bedrock model
    async fn invoke_model(&self, prompt: String) -> Result<String> {
        use aws_sdk_bedrockruntime::primitives::Blob;
        
        // Create the request body based on the provider
        let body = match self.config.provider {
            BedrockModelProvider::Claude => {
                let request = ClaudeRequest {
                    prompt,
                    max_tokens_to_sample: self.config.max_tokens,
                    temperature: self.config.temperature,
                    top_p: self.config.top_p,
                    top_k: self.config.top_k,
                    stop_sequences: self.config.stop_sequences.clone(),
                };
                serde_json::to_vec(&request).context("Failed to serialize Claude request")?
            },
            _ => {
                let request = GenericRequest {
                    prompt,
                    max_tokens: self.config.max_tokens,
                    temperature: self.config.temperature,
                    top_p: self.config.top_p,
                    stop: self.config.stop_sequences.clone(),
                };
                serde_json::to_vec(&request).context("Failed to serialize generic request")?
            }
        };
        
        // Get the model ID
        let model_id = self.config.provider.model_id(&self.config.version);
        
        // Create the invoke model request
        let invoke_request = self.client
            .invoke_model()
            .model_id(model_id)
            .content_type("application/json")
            .accept("application/json")
            .body(Blob::new(body));
        
        // Execute the request with retries
        let max_retries = 3;
        let mut last_error = None;
        
        for retry in 0..=max_retries {
            if retry > 0 {
                let backoff_duration = Duration::from_millis(100 * 2u64.pow(retry as u32));
                warn!("Retrying Bedrock API call (attempt {}/{}) after {}ms", 
                      retry, max_retries, backoff_duration.as_millis());
                tokio::time::sleep(backoff_duration).await;
            }
            
            match invoke_request.clone().send().await {
                Ok(response) => {
                    // Parse the response
                    let response_bytes = response.body.as_ref();
                    
                    // Parse the response based on the provider
                    return match self.config.provider {
                        BedrockModelProvider::Claude => {
                            let claude_response: ClaudeResponse = match serde_json::from_slice(response_bytes) {
                                Ok(resp) => resp,
                                Err(e) => {
                                    error!("Failed to parse Claude response: {}", e);
                                    error!("Response bytes: {:?}", String::from_utf8_lossy(response_bytes));
                                    return Err(anyhow!("Failed to parse Claude response: {}", e));
                                }
                            };
                            Ok(claude_response.completion)
                        },
                        _ => {
                            let generic_response: GenericResponse = match serde_json::from_slice(response_bytes) {
                                Ok(resp) => resp,
                                Err(e) => {
                                    error!("Failed to parse generic response: {}", e);
                                    error!("Response bytes: {:?}", String::from_utf8_lossy(response_bytes));
                                    return Err(anyhow!("Failed to parse generic response: {}", e));
                                }
                            };
                            Ok(generic_response.text)
                        }
                    };
                },
                Err(e) => {
                    warn!("Bedrock API call failed (attempt {}/{}): {}", retry, max_retries, e);
                    last_error = Some(anyhow!("Failed to invoke Bedrock model: {}", e));
                    
                    // Check if we should retry based on error type
                    let should_retry = match e.code() {
                        Some(code) if code == "ThrottlingException" => true,
                        Some(code) if code.starts_with("5") => true, // 5xx errors
                        Some(code) if code.starts_with("4") => false, // 4xx errors (client errors)
                        _ => true, // Unknown errors, retry
                    };
                    
                    if !should_retry {
                        return Err(anyhow!("Client error from Bedrock API: {}", e));
                    }
                    
                    // Retry on server errors and network errors
                    continue;
                }
            }
        }
        
        // If we get here, all retries failed
        Err(last_error.unwrap_or_else(|| anyhow!("Failed to invoke Bedrock model after {} retries", max_retries)))
    }
    
    /// Generate text using the Bedrock model
    pub async fn generate_text(&self, prompt: &str) -> Result<String> {
        self.invoke_model(prompt.to_string()).await
    }
}

/// Claude request format
#[derive(Debug, Serialize)]
struct ClaudeRequest {
    /// Prompt text
    prompt: String,
    
    /// Maximum tokens to generate
    max_tokens_to_sample: u32,
    
    /// Temperature (0.0-1.0)
    temperature: f64,
    
    /// Top-p sampling
    top_p: f64,
    
    /// Top-k sampling
    top_k: u32,
    
    /// Stop sequences
    stop_sequences: Vec<String>,
}

/// Generic request format for other providers
#[derive(Debug, Serialize)]
struct GenericRequest {
    /// Prompt text
    prompt: String,
    
    /// Maximum tokens to generate
    max_tokens: u32,
    
    /// Temperature (0.0-1.0)
    temperature: f64,
    
    /// Top-p sampling
    top_p: f64,
    
    /// Stop sequences
    stop: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct ClaudeResponse {
    completion: String,
}

#[derive(Debug, Deserialize)]
struct GenericResponse {
    text: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use aws_sdk_bedrockruntime::config::Config;
    
    #[tokio::test]
    async fn test_bedrock_client() {
        // Create a Bedrock client
        let config = aws_config::load_from_env().await;
        
        let client = BedrockRuntimeClient::new(&config);
        
        // Create a Bedrock NLP client
        let bedrock_config = BedrockNlpConfig {
            provider: BedrockModelProvider::Claude,
            version: "2.0".to_string(),
            temperature: 0.7,
            max_tokens: 500,
            top_p: 0.9,
            top_k: 250,
            stop_sequences: vec!["\n\nHuman:".to_string()],
        };
        
        let _bedrock_client = BedrockNlpClient::new(client, bedrock_config);
        
        // Note: We're not actually making API calls in this test
        // as it would require AWS credentials
    }
} 