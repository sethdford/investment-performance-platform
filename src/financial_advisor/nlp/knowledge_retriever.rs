use anyhow::{Result, anyhow};
use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use std::sync::Arc;
use tracing::{info, debug};

use super::embeddings::EmbeddingService;
use super::rule_based::FinancialQueryIntent;

/// Knowledge source type
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum KnowledgeSourceType {
    /// Financial concepts
    FinancialConcept,
    
    /// Market data
    MarketData,
    
    /// Regulatory information
    RegulatoryInfo,
    
    /// Tax rules
    TaxRules,
    
    /// Investment strategies
    InvestmentStrategy,
    
    /// Financial planning
    FinancialPlanning,
    
    /// Retirement planning
    RetirementPlanning,
    
    /// Estate planning
    EstatePlanning,
    
    /// Insurance planning
    InsurancePlanning,
    
    /// Education planning
    EducationPlanning,
}

/// Knowledge item
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnowledgeItem {
    /// Unique ID
    pub id: String,
    
    /// Title
    pub title: String,
    
    /// Content
    pub content: String,
    
    /// Source type
    pub source_type: KnowledgeSourceType,
    
    /// Tags
    pub tags: Vec<String>,
    
    /// Related intents
    pub related_intents: Vec<FinancialQueryIntent>,
    
    /// Embedding vector (if available)
    #[serde(skip_serializing, skip_deserializing)]
    pub embedding: Option<Vec<f32>>,
}

/// Knowledge retriever configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnowledgeRetrieverConfig {
    /// Maximum number of items to retrieve
    pub max_items: usize,
    
    /// Similarity threshold
    pub similarity_threshold: f32,
    
    /// Whether to use embeddings for retrieval
    pub use_embeddings: bool,
}

impl Default for KnowledgeRetrieverConfig {
    fn default() -> Self {
        Self {
            max_items: 5,
            similarity_threshold: 0.7,
            use_embeddings: true,
        }
    }
}

/// Knowledge retriever
#[derive(Debug)]
pub struct KnowledgeRetriever {
    /// Knowledge items
    items: Vec<KnowledgeItem>,
    
    /// Embedding service (optional)
    embedding_service: Option<Arc<EmbeddingService>>,
    
    /// Configuration
    config: KnowledgeRetrieverConfig,
    
    /// Intent to knowledge mapping
    intent_to_knowledge: HashMap<FinancialQueryIntent, Vec<usize>>,
}

impl KnowledgeRetriever {
    /// Create a new knowledge retriever
    pub fn new(config: KnowledgeRetrieverConfig) -> Self {
        Self {
            items: Vec::new(),
            embedding_service: None,
            config,
            intent_to_knowledge: HashMap::new(),
        }
    }
    
    /// Set the embedding service
    pub fn with_embedding_service(mut self, service: Arc<EmbeddingService>) -> Self {
        self.embedding_service = Some(service);
        self
    }
    
    /// Add a knowledge item
    pub fn add_item(&mut self, item: KnowledgeItem) -> Result<()> {
        let item_index = self.items.len();
        
        // Update intent to knowledge mapping
        for intent in &item.related_intents {
            self.intent_to_knowledge
                .entry(intent.clone())
                .or_insert_with(Vec::new)
                .push(item_index);
        }
        
        self.items.push(item);
        Ok(())
    }
    
    /// Retrieve knowledge items by intent
    pub fn retrieve_by_intent(&self, intent: &FinancialQueryIntent) -> Vec<&KnowledgeItem> {
        if let Some(indices) = self.intent_to_knowledge.get(intent) {
            indices.iter()
                .filter_map(|&idx| self.items.get(idx))
                .collect()
        } else {
            Vec::new()
        }
    }
    
    /// Retrieve knowledge items by query using embeddings
    pub async fn retrieve_by_query(&self, query: &str) -> Result<Vec<&KnowledgeItem>> {
        if let Some(embedding_service) = &self.embedding_service {
            if self.config.use_embeddings {
                // Generate embedding for the query
                let query_embedding = embedding_service.generate_embedding(query).await?;
                
                // Calculate similarity with all items that have embeddings
                let mut similarities: Vec<(usize, f32)> = Vec::new();
                
                // Log the query for debugging
                info!("Retrieving knowledge for query: {}", query);
                
                // Process items in batches to avoid memory issues with large knowledge bases
                for (idx, item) in self.items.iter().enumerate() {
                    if let Some(embedding) = &item.embedding {
                        let similarity = cosine_similarity(&query_embedding, embedding);
                        
                        // Only include items above the threshold
                        if similarity >= self.config.similarity_threshold {
                            similarities.push((idx, similarity));
                            debug!("Item {} matched with similarity {:.4}: {}", 
                                   item.id, similarity, item.title);
                        }
                    }
                }
                
                // Sort by similarity (descending)
                similarities.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
                
                // Return the top items
                let result: Vec<&KnowledgeItem> = similarities.iter()
                    .take(self.config.max_items)
                    .filter_map(|(idx, _)| self.items.get(*idx))
                    .collect();
                
                info!("Retrieved {} knowledge items for query", result.len());
                
                return Ok(result);
            }
        }
        
        // Fallback to keyword-based search if embeddings are not available
        let normalized_query = query.to_lowercase();
        let words: Vec<&str> = normalized_query.split_whitespace().collect();
        
        // Calculate relevance scores based on keyword matching
        let mut scores: Vec<(usize, usize)> = self.items.iter().enumerate()
            .map(|(idx, item)| {
                let title_score = words.iter()
                    .filter(|word| item.title.to_lowercase().contains(*word))
                    .count() * 3; // Title matches are weighted higher
                
                let content_score = words.iter()
                    .filter(|word| item.content.to_lowercase().contains(*word))
                    .count();
                
                let tag_score = words.iter()
                    .filter(|word| item.tags.iter().any(|tag| tag.to_lowercase().contains(*word)))
                    .count() * 2; // Tag matches are weighted higher
                
                (idx, title_score + content_score + tag_score)
            })
            .filter(|(_, score)| *score > 0) // Only include items with non-zero scores
            .collect();
        
        // Sort by score (descending)
        scores.sort_by(|a, b| b.1.cmp(&a.1));
        
        // Return the top items
        let result: Vec<&KnowledgeItem> = scores.iter()
            .take(self.config.max_items)
            .filter_map(|(idx, _)| self.items.get(*idx))
            .collect();
        
        info!("Retrieved {} knowledge items using keyword search", result.len());
        
        Ok(result)
    }
    
    /// Retrieve knowledge items by tags
    pub fn retrieve_by_tags(&self, tags: &[String]) -> Vec<&KnowledgeItem> {
        self.items.iter()
            .filter(|item| {
                tags.iter().any(|tag| item.tags.contains(tag))
            })
            .collect()
    }
    
    /// Retrieve knowledge items by source type
    pub fn retrieve_by_source_type(&self, source_type: &KnowledgeSourceType) -> Vec<&KnowledgeItem> {
        self.items.iter()
            .filter(|item| item.source_type == *source_type)
            .collect()
    }
    
    /// Generate embeddings for all knowledge items
    pub async fn generate_embeddings(&mut self) -> Result<()> {
        if let Some(embedding_service) = &self.embedding_service {
            for item in &mut self.items {
                if item.embedding.is_none() {
                    let text = format!("{}: {}", item.title, item.content);
                    let embedding = embedding_service.generate_embedding(&text).await?;
                    item.embedding = Some(embedding);
                }
            }
            Ok(())
        } else {
            Err(anyhow!("Embedding service not available"))
        }
    }
    
    /// Get all knowledge items
    pub fn get_all_items(&self) -> &[KnowledgeItem] {
        &self.items
    }
    
    /// Get a knowledge item by ID
    pub fn get_item_by_id(&self, id: &str) -> Option<&KnowledgeItem> {
        self.items.iter().find(|item| item.id == id)
    }
}

/// Calculate cosine similarity between two vectors
fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    if a.len() != b.len() || a.is_empty() {
        return 0.0;
    }
    
    let dot_product: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
    
    let magnitude_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
    let magnitude_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();
    
    if magnitude_a == 0.0 || magnitude_b == 0.0 {
        return 0.0;
    }
    
    dot_product / (magnitude_a * magnitude_b)
}

#[cfg(test)]
mod tests {
    use super::*;
    
    fn create_test_knowledge_item(id: &str, title: &str, content: &str, source_type: KnowledgeSourceType, intents: Vec<FinancialQueryIntent>) -> KnowledgeItem {
        KnowledgeItem {
            id: id.to_string(),
            title: title.to_string(),
            content: content.to_string(),
            source_type,
            tags: vec![],
            related_intents: intents,
            embedding: None,
        }
    }
    
    #[test]
    fn test_retrieve_by_intent() {
        let mut retriever = KnowledgeRetriever::new(KnowledgeRetrieverConfig::default());
        
        retriever.add_item(create_test_knowledge_item(
            "1",
            "Portfolio Diversification",
            "Diversification is a risk management strategy that mixes a wide variety of investments within a portfolio.",
            KnowledgeSourceType::InvestmentStrategy,
            vec![FinancialQueryIntent::AssetAllocation, FinancialQueryIntent::RiskAssessment],
        )).unwrap();
        
        retriever.add_item(create_test_knowledge_item(
            "2",
            "Retirement Planning Basics",
            "Retirement planning is the process of determining retirement income goals and the actions necessary to achieve those goals.",
            KnowledgeSourceType::RetirementPlanning,
            vec![FinancialQueryIntent::RetirementPlanning],
        )).unwrap();
        
        let results = retriever.retrieve_by_intent(&FinancialQueryIntent::RetirementPlanning);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].id, "2");
        
        let results = retriever.retrieve_by_intent(&FinancialQueryIntent::AssetAllocation);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].id, "1");
        
        let results = retriever.retrieve_by_intent(&FinancialQueryIntent::TaxOptimization);
        assert_eq!(results.len(), 0);
    }
    
    #[test]
    fn test_retrieve_by_source_type() {
        let mut retriever = KnowledgeRetriever::new(KnowledgeRetrieverConfig::default());
        
        retriever.add_item(create_test_knowledge_item(
            "1",
            "Portfolio Diversification",
            "Diversification is a risk management strategy that mixes a wide variety of investments within a portfolio.",
            KnowledgeSourceType::InvestmentStrategy,
            vec![FinancialQueryIntent::AssetAllocation],
        )).unwrap();
        
        retriever.add_item(create_test_knowledge_item(
            "2",
            "Retirement Planning Basics",
            "Retirement planning is the process of determining retirement income goals and the actions necessary to achieve those goals.",
            KnowledgeSourceType::RetirementPlanning,
            vec![FinancialQueryIntent::RetirementPlanning],
        )).unwrap();
        
        let results = retriever.retrieve_by_source_type(&KnowledgeSourceType::RetirementPlanning);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].id, "2");
        
        let results = retriever.retrieve_by_source_type(&KnowledgeSourceType::InvestmentStrategy);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].id, "1");
        
        let results = retriever.retrieve_by_source_type(&KnowledgeSourceType::TaxRules);
        assert_eq!(results.len(), 0);
    }
    
    #[test]
    fn test_cosine_similarity() {
        let a = vec![1.0, 2.0, 3.0];
        let b = vec![1.0, 2.0, 3.0];
        let similarity = cosine_similarity(&a, &b);
        assert!((similarity - 1.0).abs() < 1e-6, "Expected similarity close to 1.0, got {}", similarity);
        
        let a = vec![1.0, 0.0, 0.0];
        let b = vec![0.0, 1.0, 0.0];
        assert_eq!(cosine_similarity(&a, &b), 0.0);
        
        let a = vec![1.0, 1.0, 0.0];
        let b = vec![1.0, 0.0, 1.0];
        assert!(cosine_similarity(&a, &b) > 0.0 && cosine_similarity(&a, &b) < 1.0);
    }
} 