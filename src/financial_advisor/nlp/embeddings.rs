use anyhow::{Result, anyhow, Context};
use aws_sdk_bedrockruntime::Client as BedrockRuntimeClient;
use aws_sdk_bedrockruntime::primitives::Blob;
use serde::Serialize;
use std::collections::HashMap;
use std::sync::Arc;
use tracing::{warn, error};
use lru::LruCache;
use std::sync::Mutex;
use std::num::NonZeroUsize;
use aws_sdk_bedrockruntime::error::ProvideErrorMetadata;
use chrono::Utc;

use super::rule_based::{FinancialQueryIntent, EntityType};
use super::types::ClientData;

/// Configuration for the Titan embedding model
#[derive(Debug, Clone)]
pub struct TitanEmbeddingConfig {
    /// Maximum number of items to keep in the cache
    pub max_cache_size: usize,
    /// Time-to-live for cache entries in seconds
    pub cache_ttl_seconds: i64,
    /// Model ID to use for embeddings
    pub model_id: String,
}

impl Default for TitanEmbeddingConfig {
    fn default() -> Self {
        Self {
            max_cache_size: 1000,
            cache_ttl_seconds: 3600, // 1 hour
            model_id: "amazon.titan-embed-text-v1".to_string(),
        }
    }
}

/// Cached embedding with timestamp
#[derive(Debug, Clone)]
struct CachedEmbedding {
    /// The embedding vector
    embedding: Vec<f32>,
    /// Timestamp when the embedding was cached (Unix timestamp)
    timestamp: i64,
}

/// Embedding service for financial queries
pub struct EmbeddingService {
    /// Bedrock runtime client
    client: BedrockRuntimeClient,
    
    /// Configuration
    config: TitanEmbeddingConfig,
    
    /// Intent embeddings cache
    intent_embeddings: HashMap<FinancialQueryIntent, Vec<f32>>,
    
    /// Entity type embeddings cache
    entity_embeddings: HashMap<EntityType, Vec<f32>>,
    
    /// Embedding cache
    embedding_cache: Arc<Mutex<LruCache<String, CachedEmbedding>>>,
}

impl Clone for EmbeddingService {
    fn clone(&self) -> Self {
        Self {
            client: self.client.clone(),
            config: self.config.clone(),
            intent_embeddings: self.intent_embeddings.clone(),
            entity_embeddings: self.entity_embeddings.clone(),
            embedding_cache: self.embedding_cache.clone(),
        }
    }
}

impl EmbeddingService {
    /// Create a new embedding service
    pub fn new(client: BedrockRuntimeClient, config: TitanEmbeddingConfig) -> Self {
        let cache_size = NonZeroUsize::new(config.max_cache_size).unwrap_or(NonZeroUsize::new(1).unwrap());
        let embedding_cache = Arc::new(Mutex::new(LruCache::new(cache_size)));
        
        Self {
            client,
            config,
            intent_embeddings: HashMap::new(),
            entity_embeddings: HashMap::new(),
            embedding_cache,
        }
    }
    
    /// Initialize intent embeddings
    pub async fn initialize_intent_embeddings(&mut self) -> Result<()> {
        // Generate embeddings for each intent description
        for intent in [
            (FinancialQueryIntent::PortfolioPerformance, "How is my portfolio performing"),
            (FinancialQueryIntent::AssetAllocation, "What is my current asset allocation"),
            (FinancialQueryIntent::RetirementPlanning, "When can I retire"),
            (FinancialQueryIntent::TaxOptimization, "How can I reduce my taxes"),
            (FinancialQueryIntent::GoalProgress, "Am I on track for my financial goals"),
        ].iter() {
            let embedding = self.generate_embedding(intent.1).await?;
            self.intent_embeddings.insert(intent.0.clone(), embedding);
        }
        
        Ok(())
    }
    
    /// Generate an embedding for a single text
    pub async fn generate_embedding(&self, text: &str) -> Result<Vec<f32>> {
        // Check cache first
        let cache_key = format!("emb:{}", text);
        
        {
            let mut cache = self.embedding_cache.lock().unwrap();
            if let Some(cached_embedding) = cache.get(&cache_key) {
                // Check if the cached embedding is still valid
                let now = Utc::now().timestamp();
                if now - cached_embedding.timestamp < self.config.cache_ttl_seconds {
                    return Ok(cached_embedding.embedding.clone());
                }
            }
        }
        
        // Not in cache or expired, generate new embedding
        #[cfg(not(test))]
        let embedding = self.invoke_embedding_model(text).await?;
        
        #[cfg(test)]
        let embedding = {
            // In test mode, use our mock implementation
            use crate::financial_advisor::nlp::embeddings::mocks::mock_embedding_response;
            mock_embedding_response(text)
        };
        
        // Store in cache
        {
            let mut cache = self.embedding_cache.lock().unwrap();
            let cached_item = CachedEmbedding {
                embedding: embedding.clone(),
                timestamp: Utc::now().timestamp(),
            };
            
            cache.put(cache_key, cached_item);
            
            // Prune cache if it exceeds max size
            if cache.len() > self.config.max_cache_size {
                let oldest_key = cache.iter()
                    .min_by_key(|(_, item)| item.timestamp)
                    .map(|(k, _)| k.clone());
                
                if let Some(key) = oldest_key {
                    cache.pop(&key);
                }
            }
        }
        
        Ok(embedding)
    }
    
    /// Generate embeddings for a batch of texts
    pub async fn generate_embeddings_batch(&self, texts: &[String]) -> Result<Vec<Vec<f32>>> {
        let mut results = Vec::with_capacity(texts.len());
        let mut texts_to_process = Vec::new();
        let mut indices_to_process = Vec::new();
        
        // Check cache for each text
        {
            let mut cache = self.embedding_cache.lock().unwrap();
            for (i, text) in texts.iter().enumerate() {
                let cache_key = format!("emb:{}", text);
                
                if let Some(cached_embedding) = cache.get(&cache_key) {
                    // Check if the cached embedding is still valid
                    let now = Utc::now().timestamp();
                    if now - cached_embedding.timestamp < self.config.cache_ttl_seconds {
                        results.push(cached_embedding.embedding.clone());
                        continue;
                    }
                }
                
                // Not in cache or expired, add to processing list
                texts_to_process.push(text.clone());
                indices_to_process.push(i);
            }
        }
        
        // If all embeddings were in cache, return early
        if texts_to_process.is_empty() {
            return Ok(results);
        }
        
        // Process texts not in cache
        #[cfg(not(test))]
        let new_embeddings = self.batch_invoke_embedding_model(&texts_to_process).await?;
        
        #[cfg(test)]
        let new_embeddings = {
            // In test mode, use our mock implementation
            use crate::financial_advisor::nlp::embeddings::mocks::mock_embedding_response;
            texts_to_process.iter()
                .map(|text| mock_embedding_response(text))
                .collect::<Vec<_>>()
        };
        
        // Store new embeddings in cache
        {
            let mut cache = self.embedding_cache.lock().unwrap();
            for (i, text) in texts_to_process.iter().enumerate() {
                let cache_key = format!("emb:{}", text);
                let cached_item = CachedEmbedding {
                    embedding: new_embeddings[i].clone(),
                    timestamp: Utc::now().timestamp(),
                };
                
                cache.put(cache_key, cached_item);
            }
            
            // Prune cache if it exceeds max size
            while cache.len() > self.config.max_cache_size {
                let oldest_key = cache.iter()
                    .min_by_key(|(_, item)| item.timestamp)
                    .map(|(k, _)| k.clone());
                
                if let Some(key) = oldest_key {
                    cache.pop(&key);
                } else {
                    break;
                }
            }
        }
        
        // Merge cached and new embeddings in the original order
        let mut final_results = vec![Vec::new(); texts.len()];
        
        // First, place the cached embeddings
        for (i, embedding) in results.into_iter().enumerate() {
            final_results[i] = embedding;
        }
        
        // Then, place the new embeddings
        for (i, embedding) in new_embeddings.into_iter().enumerate() {
            let original_index = indices_to_process[i];
            final_results[original_index] = embedding;
        }
        
        Ok(final_results)
    }
    
    /// Find the most similar intent for a query
    pub async fn find_similar_intent(&self, query: &str) -> Result<(FinancialQueryIntent, f64)> {
        let query_embedding = self.generate_embedding(query).await?;
        
        let mut best_intent = FinancialQueryIntent::Unknown;
        let mut best_similarity = 0.0;
        
        for (intent, embedding) in &self.intent_embeddings {
            let similarity = cosine_similarity(&query_embedding, embedding);
            
            if similarity > best_similarity {
                best_similarity = similarity;
                best_intent = intent.clone();
            }
        }
        
        Ok((best_intent, best_similarity as f64))
    }
    
    /// Find similar client profiles
    pub async fn find_similar_clients(&self, client_data: &ClientData, all_clients: &[ClientData]) -> Result<Vec<(String, f32)>> {
        // Generate embedding for client profile
        let client_desc = format!(
            "Client with portfolio value ${}, asset allocation: {}% stocks, {}% bonds, {}% cash",
            client_data.portfolio.as_ref().map_or(0.0, |p| p.total_value),
            client_data.portfolio.as_ref().map_or(0.0, |p| p.asset_allocation.iter()
                .find(|a| a.asset_class == "Stocks").map_or(0.0, |a| a.allocation_percentage * 100.0)),
            client_data.portfolio.as_ref().map_or(0.0, |p| p.asset_allocation.iter()
                .find(|a| a.asset_class == "Bonds").map_or(0.0, |a| a.allocation_percentage * 100.0)),
            client_data.portfolio.as_ref().map_or(0.0, |p| p.asset_allocation.iter()
                .find(|a| a.asset_class == "Cash").map_or(0.0, |a| a.allocation_percentage * 100.0))
        );
        
        let client_embedding = self.generate_embedding(&client_desc).await?;
        
        // Find similar clients
        let mut similarities = Vec::new();
        
        for other_client in all_clients {
            if other_client.client_id == client_data.client_id {
                continue;
            }
            
            let other_desc = format!(
                "Client with portfolio value ${}, asset allocation: {}% stocks, {}% bonds, {}% cash",
                other_client.portfolio.as_ref().map_or(0.0, |p| p.total_value),
                other_client.portfolio.as_ref().map_or(0.0, |p| p.asset_allocation.iter()
                    .find(|a| a.asset_class == "Stocks").map_or(0.0, |a| a.allocation_percentage * 100.0)),
                other_client.portfolio.as_ref().map_or(0.0, |p| p.asset_allocation.iter()
                    .find(|a| a.asset_class == "Bonds").map_or(0.0, |a| a.allocation_percentage * 100.0)),
                other_client.portfolio.as_ref().map_or(0.0, |p| p.asset_allocation.iter()
                    .find(|a| a.asset_class == "Cash").map_or(0.0, |a| a.allocation_percentage * 100.0))
            );
            
            let other_embedding = self.generate_embedding(&other_desc).await?;
            let similarity = cosine_similarity(&client_embedding, &other_embedding);
            
            similarities.push((other_client.client_id.clone(), similarity));
        }
        
        // Sort by similarity (descending)
        similarities.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        
        Ok(similarities)
    }

    /// Update the embedding cache with new embeddings
    /// 
    /// This method allows for incremental updates to the embedding cache,
    /// which is useful when new content is added to the knowledge base.
    pub fn update_cache(&self, texts: &[String], embeddings: &[Vec<f32>]) {
        if texts.len() != embeddings.len() {
            return;
        }
        
        let mut cache = self.embedding_cache.lock().unwrap();
        for (i, text) in texts.iter().enumerate() {
            let cache_key = format!("emb:{}", text);
            cache.put(cache_key, CachedEmbedding {
                embedding: embeddings[i].clone(),
                timestamp: Utc::now().timestamp(),
            });
        }
        
        // Prune cache if needed
        while cache.len() > self.config.max_cache_size {
            // Find the oldest entry
            if let Some(oldest_key) = cache.iter()
                .min_by_key(|(_, item)| item.timestamp)
                .map(|(k, _)| k.clone()) {
                cache.pop(&oldest_key);
            } else {
                break;
            }
        }
    }
    
    /// Invalidate specific entries in the embedding cache
    /// 
    /// This is useful when content changes and embeddings need to be regenerated.
    pub fn invalidate_embeddings(&self, texts: &[String]) -> Result<()> {
        let mut cache = self.embedding_cache.lock().unwrap();
        for text in texts {
            let cache_key = format!("emb:{}", text);
            cache.pop(&cache_key);
        }
        
        Ok(())
    }
    
    /// Get cache statistics
    /// 
    /// Returns the number of items in the cache and the cache capacity.
    pub fn get_cache_stats(&self) -> (usize, usize) {
        let cache = self.embedding_cache.lock().unwrap();
        (cache.len(), cache.cap().get())
    }

    /// Invoke the embedding model to generate an embedding for a single text
    async fn invoke_embedding_model(&self, text: &str) -> Result<Vec<f32>> {
        // Create the request
        let request = TitanEmbeddingRequest {
            input_text: text.to_string(),
        };
        
        let body = serde_json::to_vec(&request)
            .context("Failed to serialize embedding request")?;
        
        // Create the invoke model request
        let invoke_request = self.client
            .invoke_model()
            .model_id(&self.config.model_id)
            .content_type("application/json")
            .accept("application/json")
            .body(Blob::new(body));
        
        // Execute the request with retries
        let max_retries = 3;
        let mut last_error = None;
        
        for retry in 0..=max_retries {
            if retry > 0 {
                let backoff_duration = std::time::Duration::from_millis(100 * 2u64.pow(retry as u32));
                warn!("Retrying embedding API call (attempt {}/{}) after {}ms", 
                      retry, max_retries, backoff_duration.as_millis());
                tokio::time::sleep(backoff_duration).await;
            }
            
            match invoke_request.clone().send().await {
                Ok(response) => {
                    // Parse the response
                    let response_bytes = response.body.as_ref();
                    
                    // Parse the response
                    let response_json: serde_json::Value = match serde_json::from_slice(response_bytes) {
                        Ok(json) => json,
                        Err(e) => {
                            error!("Failed to parse embedding response: {}", e);
                            error!("Response bytes: {:?}", String::from_utf8_lossy(response_bytes));
                            last_error = Some(anyhow!("Failed to parse embedding response: {}", e));
                            continue;
                        }
                    };
                    
                    // Extract the embedding
                    let embedding = match response_json["embedding"].as_array() {
                        Some(array) => {
                            array.iter()
                                .filter_map(|v| v.as_f64().map(|f| f as f32))
                                .collect::<Vec<f32>>()
                        },
                        None => {
                            error!("No embedding field in response: {:?}", response_json);
                            last_error = Some(anyhow!("No embedding field in response"));
                            continue;
                        }
                    };
                    
                    if embedding.is_empty() {
                        error!("Empty embedding returned");
                        last_error = Some(anyhow!("Empty embedding returned"));
                        continue;
                    }
                    
                    // Normalize the embedding
                    let mut norm = 0.0;
                    for val in &embedding {
                        norm += val * val;
                    }
                    norm = norm.sqrt();
                    
                    let mut normalized_embedding = embedding;
                    if norm > 0.0 {
                        for val in &mut normalized_embedding {
                            *val /= norm;
                        }
                    }
                    
                    return Ok(normalized_embedding);
                },
                Err(e) => {
                    warn!("Embedding API call failed (attempt {}/{}): {}", retry, max_retries, e);
                    last_error = Some(anyhow!("Failed to invoke embedding model: {}", e));
                    
                    // Check if we should retry based on error type
                    let should_retry = match e.code() {
                        Some(code) if code == "ThrottlingException" => true,
                        Some(code) if code.starts_with("5") => true, // 5xx errors
                        Some(code) if code.starts_with("4") => false, // 4xx errors (client errors)
                        _ => true, // Unknown errors, retry
                    };
                    
                    if !should_retry {
                        return Err(anyhow!("Client error from embedding API: {}", e));
                    }
                    
                    // Retry on server errors and network errors
                    continue;
                }
            }
        }
        
        // If we get here, all retries failed
        Err(last_error.unwrap_or_else(|| anyhow!("Failed to generate embedding after {} retries", max_retries)))
    }
    
    /// Batch invoke the embedding model for multiple texts
    async fn batch_invoke_embedding_model(&self, texts: &[String]) -> Result<Vec<Vec<f32>>> {
        // Process texts in parallel with limited concurrency
        let semaphore = Arc::new(tokio::sync::Semaphore::new(5)); // Max 5 concurrent requests
        let mut tasks = Vec::with_capacity(texts.len());
        
        for text in texts {
            let text_clone = text.clone();
            let semaphore_clone = semaphore.clone();
            let self_clone = self.clone();
            
            let task = tokio::spawn(async move {
                let _permit = semaphore_clone.acquire().await.unwrap();
                self_clone.invoke_embedding_model(&text_clone).await
            });
            
            tasks.push(task);
        }
        
        // Wait for all tasks to complete
        let mut results = Vec::with_capacity(texts.len());
        for task in tasks {
            match task.await {
                Ok(Ok(embedding)) => {
                    results.push(embedding);
                },
                Ok(Err(e)) => {
                    return Err(anyhow!("Failed to generate embedding in batch: {}", e));
                },
                Err(e) => {
                    return Err(anyhow!("Task failed: {}", e));
                }
            }
        }
        
        Ok(results)
    }
}

/// Titan embedding request
#[derive(Debug, Serialize)]
struct TitanEmbeddingRequest {
    input_text: String,
}

/// Calculate cosine similarity between two vectors
fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    if a.len() != b.len() {
        return 0.0;
    }
    
    let mut dot_product = 0.0;
    let mut a_norm = 0.0;
    let mut b_norm = 0.0;
    
    for i in 0..a.len() {
        dot_product += a[i] * b[i];
        a_norm += a[i] * a[i];
        b_norm += b[i] * b[i];
    }
    
    a_norm = a_norm.sqrt();
    b_norm = b_norm.sqrt();
    
    if a_norm == 0.0 || b_norm == 0.0 {
        return 0.0;
    }
    
    dot_product / (a_norm * b_norm)
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_cosine_similarity() {
        let a: Vec<f32> = vec![1.0, 0.0, 0.0];
        let b: Vec<f32> = vec![0.0, 1.0, 0.0];
        let similarity = cosine_similarity(&a, &b);
        assert_eq!(similarity, 0.0);
    }
    
    #[tokio::test]
    async fn test_batch_embedding_generation() {
        // Create a mock Bedrock client
        let client = create_mock_bedrock_client();
        
        // Create the embedding service
        let config = TitanEmbeddingConfig {
            max_cache_size: 10,
            cache_ttl_seconds: 3600,
            ..Default::default()
        };
        let service = EmbeddingService::new(client, config);
        
        // Test batch embedding generation
        let texts = vec![
            "How is my portfolio performing?".to_string(),
            "What is my current asset allocation?".to_string(),
            "When can I retire?".to_string(),
        ];
        
        // Generate embeddings
        let embeddings = service.generate_embeddings_batch(&texts).await.unwrap();
        
        // Verify results
        assert_eq!(embeddings.len(), 3);
        for embedding in &embeddings {
            assert!(!embedding.is_empty());
        }
        
        // Check cache stats
        let (cache_size, _) = service.get_cache_stats();
        assert_eq!(cache_size, 3);
        
        // Generate again to test cache hit
        let embeddings2 = service.generate_embeddings_batch(&texts).await.unwrap();
        
        // Verify results are the same
        assert_eq!(embeddings.len(), embeddings2.len());
        for i in 0..embeddings.len() {
            assert_eq!(embeddings[i], embeddings2[i]);
        }
        
        // Test invalidation
        service.invalidate_embeddings(&[texts[0].clone()]).unwrap();
        
        // Check cache stats
        let (cache_size, _) = service.get_cache_stats();
        assert_eq!(cache_size, 2);
        
        // Test incremental update
        service.update_cache(&[texts[0].clone()], &[service.generate_embedding(&texts[0]).await.unwrap()]);
        
        // Check cache stats
        let (cache_size, _) = service.get_cache_stats();
        assert_eq!(cache_size, 3);
    }
    
    // Helper function to create a mock Bedrock client
    fn create_mock_bedrock_client() -> BedrockRuntimeClient {
        // Create a mock client that returns fixed responses for testing
        use aws_sdk_bedrockruntime::operation::invoke_model::{InvokeModelError, InvokeModelOutput};
        use aws_smithy_runtime_api::client::result::SdkError;
        use aws_smithy_types::body::SdkBody;
        use aws_smithy_types::byte_stream::ByteStream;
        use std::io::Cursor;
        
        // Create a custom config with a test endpoint
        let config = aws_sdk_bedrockruntime::config::Builder::new()
            .region(aws_sdk_bedrockruntime::config::Region::new("us-east-1"))
            .behavior_version(aws_sdk_bedrockruntime::config::BehaviorVersion::v2023_11_09())
            .endpoint_url("http://localhost:8000") // Use a fake endpoint that won't be called
            .build();
        
        // Create the client with the test config
        let client = BedrockRuntimeClient::from_conf(config);
        
        // Use the client's test utilities to override the invoke_model operation
        // This is a simplified approach - in a real test, you'd use a proper mocking framework
        
        client
    }
}

// For testing purposes, we'll create a mock implementation of the invoke_model response
#[cfg(test)]
mod mocks {
    use super::*;
    use aws_sdk_bedrockruntime::operation::invoke_model::{InvokeModelOutput};
    use aws_smithy_types::body::SdkBody;
    use aws_smithy_types::byte_stream::ByteStream;
    use std::io::Cursor;
    
    // Mock response generator for embedding API
    pub fn mock_embedding_response(input_text: &str) -> Vec<f32> {
        // Generate a deterministic embedding based on the input text
        // This is just for testing - not a real embedding algorithm
        let mut result = Vec::with_capacity(1536);
        let seed = input_text.chars().fold(0, |acc, c| acc + c as u32) as f32;
        
        for i in 0..1536 {
            // Generate a pseudo-random value based on the seed and position
            let val = ((seed + i as f32).sin() + 1.0) / 2.0;
            result.push(val);
        }
        
        result
    }
} 