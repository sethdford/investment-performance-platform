use std::fmt;

use super::bedrock::BedrockNlpClient;
use super::embeddings::EmbeddingService;
use super::rule_based::FinancialNlpService;

// Implement Debug for BedrockNlpClient
impl fmt::Debug for BedrockNlpClient {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("BedrockNlpClient")
            .field("config", &"<BedrockNlpConfig>")
            .finish()
    }
}

// Implement Debug for EmbeddingService
impl fmt::Debug for EmbeddingService {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("EmbeddingService")
            .field("config", &"<TitanEmbeddingConfig>")
            .finish()
    }
}

// Implement Debug for FinancialNlpService
impl fmt::Debug for FinancialNlpService {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("FinancialNlpService")
            .finish()
    }
} 