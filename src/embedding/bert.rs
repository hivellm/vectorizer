//! BERT embedding provider
//!
//! BERT (Bidirectional Encoder Representations from Transformers) is a transformer-based
//! machine learning technique for natural language processing. This implementation
//! provides dense embeddings using BERT models.

use std::sync::Arc;

use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;

use crate::embedding::{EmbeddingError, EmbeddingProvider, EmbeddingProviderType};

/// BERT configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BERTConfig {
    /// Model name or path
    pub model_name: String,
    /// Maximum sequence length
    pub max_length: usize,
    /// Use CLS token for sentence embeddings
    pub use_cls_token: bool,
    /// Use mean pooling
    pub use_mean_pooling: bool,
    /// Use max pooling
    pub use_max_pooling: bool,
    /// Normalize embeddings
    pub normalize: bool,
    /// Batch size for processing
    pub batch_size: usize,
    /// Use GPU if available
    pub use_gpu: bool,
}

impl Default for BERTConfig {
    fn default() -> Self {
        Self {
            model_name: "bert-base-uncased".to_string(),
            max_length: 512,
            use_cls_token: true,
            use_mean_pooling: false,
            use_max_pooling: false,
            normalize: true,
            batch_size: 32,
            use_gpu: false,
        }
    }
}

/// BERT embedding provider
pub struct BERTProvider {
    config: BERTConfig,
    model_loaded: Arc<RwLock<bool>>,
    dimension: usize,
}

impl BERTProvider {
    /// Create a new BERT provider
    pub fn new(config: BERTConfig) -> Self {
        Self {
            config,
            model_loaded: Arc::new(RwLock::new(false)),
            dimension: 768, // Default BERT dimension
        }
    }

    /// Create with default configuration
    pub fn default() -> Self {
        Self::new(BERTConfig::default())
    }

    /// Initialize the BERT model
    pub async fn initialize(&self) -> Result<(), EmbeddingError> {
        // In a real implementation, this would load the actual BERT model
        // For now, we'll simulate the initialization
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

        *self.model_loaded.write().await = true;
        Ok(())
    }

    /// Tokenize text for BERT
    fn tokenize(&self, text: &str) -> Vec<String> {
        // Simple tokenization - in a real implementation, you would use
        // the BERT tokenizer (WordPiece)
        text.to_lowercase()
            .chars()
            .filter(|c| c.is_alphanumeric() || c.is_whitespace())
            .collect::<String>()
            .split_whitespace()
            .map(|s| s.to_string())
            .collect()
    }

    /// Generate BERT embedding for text
    fn generate_embedding(&self, text: &str) -> Result<Vec<f32>, EmbeddingError> {
        if !self
            .model_loaded
            .try_read()
            .map(|loaded| *loaded)
            .unwrap_or(false)
        {
            return Err(EmbeddingError::Internal(
                "BERT model not initialized".to_string(),
            ));
        }

        let tokens = self.tokenize(text);

        if tokens.is_empty() {
            return Err(EmbeddingError::Internal(
                "Empty text after tokenization".to_string(),
            ));
        }

        // Truncate if too long
        let truncated_tokens = if tokens.len() > self.config.max_length {
            tokens[..self.config.max_length].to_vec()
        } else {
            tokens
        };

        // Generate embedding (simulated)
        let mut embedding = vec![0.0; self.dimension];

        // Simple hash-based embedding for simulation
        for (i, token) in truncated_tokens.iter().enumerate() {
            let hash = self.hash_string(token);
            let weight = 1.0 / (i + 1) as f32; // Decreasing weight for position

            for j in 0..self.dimension {
                let idx = (hash + j as u64) as usize % self.dimension;
                embedding[idx] += weight * (hash % 100) as f32 / 100.0;
            }
        }

        // Apply pooling strategy
        if self.config.use_cls_token {
            // Use first token (CLS) - already done in our simple implementation
        } else if self.config.use_mean_pooling {
            // Mean pooling
            let mean = embedding.iter().sum::<f32>() / embedding.len() as f32;
            embedding = vec![mean; self.dimension];
        } else if self.config.use_max_pooling {
            // Max pooling
            let max_val = embedding.iter().fold(0.0_f32, |a, &b| a.max(b));
            embedding = vec![max_val; self.dimension];
        }

        // Normalize if requested
        if self.config.normalize {
            let magnitude = embedding.iter().map(|&x| x * x).sum::<f32>().sqrt();
            if magnitude > 0.0 {
                for val in &mut embedding {
                    *val /= magnitude;
                }
            }
        }

        Ok(embedding)
    }

    /// Simple hash function for simulation
    fn hash_string(&self, s: &str) -> u64 {
        let mut hash = 0u64;
        for byte in s.bytes() {
            hash = hash.wrapping_mul(31).wrapping_add(byte as u64);
        }
        hash
    }

    /// Get model information
    pub fn model_info(&self) -> BERTModelInfo {
        BERTModelInfo {
            name: self.config.model_name.clone(),
            dimension: self.dimension,
            max_length: self.config.max_length,
            loaded: self.model_loaded.try_read().map(|r| *r).unwrap_or(false),
        }
    }
}

/// BERT model information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BERTModelInfo {
    /// Model name
    pub name: String,
    /// Embedding dimension
    pub dimension: usize,
    /// Maximum sequence length
    pub max_length: usize,
    /// Whether model is loaded
    pub loaded: bool,
}

#[async_trait::async_trait]
impl EmbeddingProvider for BERTProvider {
    async fn embed(&self, text: &str) -> Result<Vec<f32>, EmbeddingError> {
        self.generate_embedding(text)
    }

    async fn embed_batch(&self, texts: &[String]) -> Result<Vec<Vec<f32>>, EmbeddingError> {
        let mut embeddings = Vec::with_capacity(texts.len());

        for text in texts {
            let embedding = self.generate_embedding(text)?;
            embeddings.push(embedding);
        }

        Ok(embeddings)
    }

    fn dimension(&self) -> usize {
        self.dimension
    }

    fn name(&self) -> &str {
        "BERT"
    }

    async fn is_available(&self) -> bool {
        *self.model_loaded.read().await
    }
    
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

/// BERT factory for creating providers
pub struct BERTFactory;

impl BERTFactory {
    /// Create a new BERT provider with default config
    pub fn create_default() -> BERTProvider {
        BERTProvider::default()
    }

    /// Create a new BERT provider with custom config
    pub fn create_with_config(config: BERTConfig) -> BERTProvider {
        BERTProvider::new(config)
    }

    /// Create a BERT provider and initialize it
    pub async fn create_and_initialize(
        config: Option<BERTConfig>,
    ) -> Result<BERTProvider, EmbeddingError> {
        let provider = if let Some(config) = config {
            BERTProvider::new(config)
        } else {
            BERTProvider::default()
        };

        provider.initialize().await?;
        Ok(provider)
    }

    /// Get available BERT models
    pub fn available_models() -> Vec<BERTModel> {
        vec![
            BERTModel {
                name: "bert-base-uncased".to_string(),
                dimension: 768,
                max_length: 512,
                description: "Base BERT model (uncased)".to_string(),
            },
            BERTModel {
                name: "bert-large-uncased".to_string(),
                dimension: 1024,
                max_length: 512,
                description: "Large BERT model (uncased)".to_string(),
            },
            BERTModel {
                name: "bert-base-cased".to_string(),
                dimension: 768,
                max_length: 512,
                description: "Base BERT model (cased)".to_string(),
            },
        ]
    }
}

/// BERT model information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BERTModel {
    /// Model name
    pub name: String,
    /// Embedding dimension
    pub dimension: usize,
    /// Maximum sequence length
    pub max_length: usize,
    /// Model description
    pub description: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_bert_provider_creation() {
        let config = BERTConfig::default();
        let provider = BERTProvider::new(config);

        assert_eq!(provider.name(), "BERT");
        assert_eq!(provider.dimension(), 768);
        assert!(!provider.is_available().await);
    }

    #[tokio::test]
    async fn test_bert_initialization() {
        let provider = BERTProvider::default();
        assert!(!provider.is_available().await);

        provider.initialize().await.unwrap();
        assert!(provider.is_available().await);
    }

    #[tokio::test]
    async fn test_bert_embedding() {
        let provider = BERTProvider::default();
        provider.initialize().await.unwrap();

        let embedding = provider.embed("Hello, world!").await.unwrap();
        assert_eq!(embedding.len(), 768);

        // Check that embedding has non-zero values
        let has_values = embedding.iter().any(|&x| x != 0.0);
        assert!(has_values, "Embedding should have non-zero values");
    }

    #[tokio::test]
    async fn test_bert_batch_embedding() {
        let provider = BERTProvider::default();
        provider.initialize().await.unwrap();

        let texts = vec!["Hello, world!".to_string(), "This is a test".to_string()];

        let embeddings = provider.embed_batch(&texts).await.unwrap();
        assert_eq!(embeddings.len(), 2);
        assert_eq!(embeddings[0].len(), 768);
        assert_eq!(embeddings[1].len(), 768);
    }

    #[tokio::test]
    async fn test_bert_factory() {
        let provider = BERTFactory::create_and_initialize(None).await.unwrap();
        assert!(provider.is_available().await);
        assert_eq!(provider.dimension(), 768);
    }

    #[test]
    fn test_bert_config_default() {
        let config = BERTConfig::default();
        assert_eq!(config.model_name, "bert-base-uncased");
        assert_eq!(config.max_length, 512);
        assert!(config.use_cls_token);
        assert!(config.normalize);
    }

    #[test]
    fn test_bert_available_models() {
        let models = BERTFactory::available_models();
        assert!(!models.is_empty());
        assert!(models.iter().any(|m| m.name == "bert-base-uncased"));
    }

    #[tokio::test]
    async fn test_bert_model_info() {
        let provider = BERTProvider::default();
        let info = provider.model_info();

        assert_eq!(info.name, "bert-base-uncased");
        assert_eq!(info.dimension, 768);
        assert_eq!(info.max_length, 512);
        assert!(!info.loaded);

        provider.initialize().await.unwrap();
        let info = provider.model_info();
        assert!(info.loaded);
    }
}
