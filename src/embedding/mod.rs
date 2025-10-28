//! Embedding providers module
//!
//! This module provides various embedding providers for generating vector
//! representations of text, including BM25, BERT, OpenAI, and other models.

pub mod bert;
pub mod bm25;
pub mod config;
pub mod openai;
pub mod provider;

// Re-export the actual types
pub use bert::{BERTFactory, BERTProvider as BertEmbedding, BERTProvider};
pub use bm25::{BM25Factory, BM25Provider as Bm25Embedding, BM25Provider};
pub use openai::{OpenAIFactory, OpenAIProvider as OpenAIEmbedding, OpenAIProvider};

// Create type aliases for missing embedding types
pub type TfIdfEmbedding = BM25Provider; // Use BM25 as TF-IDF alternative
pub type SvdEmbedding = BM25Provider; // Use BM25 as SVD alternative
pub type MiniLmEmbedding = BERTProvider; // Use BERT as MiniLM alternative
pub type BagOfWordsEmbedding = BM25Provider; // Use BM25 as BoW alternative
pub type CharNGramEmbedding = BM25Provider; // Use BM25 as CharNGram alternative

use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;

/// Embedding provider trait
#[async_trait::async_trait]
pub trait EmbeddingProvider: Send + Sync {
    /// Generate embeddings for a single text
    async fn embed(&self, text: &str) -> Result<Vec<f32>, EmbeddingError>;

    /// Generate embeddings for multiple texts
    async fn embed_batch(&self, texts: &[String]) -> Result<Vec<Vec<f32>>, EmbeddingError>;

    /// Get the dimension of embeddings produced by this provider
    fn dimension(&self) -> usize;

    /// Get the provider name
    fn name(&self) -> &str;

    /// Check if the provider is available
    async fn is_available(&self) -> bool;
    
    /// Downcast to Any for type-specific operations
    fn as_any(&self) -> &dyn std::any::Any;
}

/// Embedding configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbeddingConfig {
    /// Provider type
    pub provider: EmbeddingProviderType,
    /// Model name or identifier
    pub model: String,
    /// Dimension of embeddings
    pub dimension: usize,
    /// Maximum text length
    pub max_length: usize,
    /// Enable caching
    pub enable_caching: bool,
    /// Cache size (number of embeddings)
    pub cache_size: usize,
    /// Batch size for processing
    pub batch_size: usize,
    /// Timeout for requests (seconds)
    pub timeout_seconds: u64,
}

impl Default for EmbeddingConfig {
    fn default() -> Self {
        Self {
            provider: EmbeddingProviderType::BM25,
            model: "default".to_string(),
            dimension: 512,
            max_length: 512,
            enable_caching: true,
            cache_size: 10000,
            batch_size: 32,
            timeout_seconds: 30,
        }
    }
}

/// Types of embedding providers
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum EmbeddingProviderType {
    /// BM25 (sparse embeddings)
    BM25,
    /// BERT-based models
    BERT,
    /// OpenAI embeddings
    OpenAI,
    /// Sentence Transformers
    SentenceTransformers,
    /// Custom provider
    Custom(String),
}

impl std::fmt::Display for EmbeddingProviderType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EmbeddingProviderType::BM25 => write!(f, "BM25"),
            EmbeddingProviderType::BERT => write!(f, "BERT"),
            EmbeddingProviderType::OpenAI => write!(f, "OpenAI"),
            EmbeddingProviderType::SentenceTransformers => write!(f, "SentenceTransformers"),
            EmbeddingProviderType::Custom(name) => write!(f, "Custom({})", name),
        }
    }
}

/// Embedding error types
#[derive(Debug, thiserror::Error)]
pub enum EmbeddingError {
    #[error("Text too long: {length} > {max_length}")]
    TextTooLong { length: usize, max_length: usize },

    #[error("Provider not available: {0}")]
    ProviderNotAvailable(String),

    #[error("Model not found: {0}")]
    ModelNotFound(String),

    #[error("API error: {0}")]
    ApiError(String),

    #[error("Network error: {0}")]
    NetworkError(String),

    #[error("Timeout error: {0}")]
    TimeoutError(String),

    #[error("Invalid configuration: {0}")]
    InvalidConfiguration(String),

    #[error("Cache error: {0}")]
    CacheError(String),

    #[error("Internal error: {0}")]
    Internal(String),
}

/// Embedding result with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbeddingResult {
    /// The embedding vector
    pub embedding: Vec<f32>,
    /// Provider used
    pub provider: EmbeddingProviderType,
    /// Model used
    pub model: String,
    /// Processing time in milliseconds
    pub processing_time_ms: u64,
    /// Text length
    pub text_length: usize,
    /// Cache hit
    pub cache_hit: bool,
}

/// Embedding cache
pub struct EmbeddingCache {
    cache: Arc<RwLock<HashMap<String, Vec<f32>>>>,
    max_size: usize,
    hits: Arc<RwLock<usize>>,
    misses: Arc<RwLock<usize>>,
}

impl EmbeddingCache {
    /// Create a new embedding cache
    pub fn new(max_size: usize) -> Self {
        Self {
            cache: Arc::new(RwLock::new(HashMap::new())),
            max_size,
            hits: Arc::new(RwLock::new(0)),
            misses: Arc::new(RwLock::new(0)),
        }
    }

    /// Get embedding from cache
    pub async fn get(&self, key: &str) -> Option<Vec<f32>> {
        let mut cache = self.cache.write().await;
        if let Some(embedding) = cache.get(key) {
            *self.hits.write().await += 1;
            Some(embedding.clone())
        } else {
            *self.misses.write().await += 1;
            None
        }
    }

    /// Store embedding in cache
    pub async fn put(&self, key: String, embedding: Vec<f32>) {
        let mut cache = self.cache.write().await;

        // Remove oldest entries if cache is full
        if cache.len() >= self.max_size {
            let keys_to_remove: Vec<String> = cache
                .keys()
                .take(cache.len() - self.max_size + 1)
                .cloned()
                .collect();
            for key in keys_to_remove {
                cache.remove(&key);
            }
        }

        cache.insert(key, embedding);
    }

    /// Get cache statistics
    pub async fn stats(&self) -> CacheStats {
        let hits = *self.hits.read().await;
        let misses = *self.misses.read().await;
        let total = hits + misses;
        let hit_rate = if total > 0 {
            hits as f64 / total as f64
        } else {
            0.0
        };

        CacheStats {
            hits,
            misses,
            total,
            hit_rate,
            size: self.cache.read().await.len(),
            max_size: self.max_size,
        }
    }

    /// Clear cache
    pub async fn clear(&self) {
        self.cache.write().await.clear();
        *self.hits.write().await = 0;
        *self.misses.write().await = 0;
    }
}

/// Cache statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheStats {
    /// Number of cache hits
    pub hits: usize,
    /// Number of cache misses
    pub misses: usize,
    /// Total requests
    pub total: usize,
    /// Cache hit rate (0.0 to 1.0)
    pub hit_rate: f64,
    /// Current cache size
    pub size: usize,
    /// Maximum cache size
    pub max_size: usize,
}

/// Embedding manager for handling multiple providers
pub struct EmbeddingManager {
    providers: HashMap<EmbeddingProviderType, Arc<dyn EmbeddingProvider>>,
    default_provider: EmbeddingProviderType,
    cache: Option<EmbeddingCache>,
    config: EmbeddingConfig,
}

impl EmbeddingManager {
    /// Create a new embedding manager
    pub fn new(config: EmbeddingConfig) -> Self {
        let cache = if config.enable_caching {
            Some(EmbeddingCache::new(config.cache_size))
        } else {
            None
        };

        Self {
            providers: HashMap::new(),
            default_provider: config.provider.clone(),
            cache,
            config,
        }
    }

    /// Add a provider
    pub fn add_provider(
        &mut self,
        provider_type: EmbeddingProviderType,
        provider: Arc<dyn EmbeddingProvider>,
    ) {
        self.providers.insert(provider_type, provider);
    }

    /// Set default provider
    pub fn set_default_provider(&mut self, provider_type: EmbeddingProviderType) {
        self.default_provider = provider_type;
    }
    
    /// Get a specific provider
    pub fn get_provider(
        &self,
        provider_type: &EmbeddingProviderType,
    ) -> Option<&Arc<dyn EmbeddingProvider>> {
        self.providers.get(provider_type)
    }

    /// Generate embedding using default provider
    pub async fn embed(&self, text: &str) -> Result<EmbeddingResult, EmbeddingError> {
        self.embed_with_provider(text, &self.default_provider).await
    }

    /// Generate embedding using specific provider
    pub async fn embed_with_provider(
        &self,
        text: &str,
        provider_type: &EmbeddingProviderType,
    ) -> Result<EmbeddingResult, EmbeddingError> {
        let start_time = std::time::Instant::now();

        // Check text length
        if text.len() > self.config.max_length {
            return Err(EmbeddingError::TextTooLong {
                length: text.len(),
                max_length: self.config.max_length,
            });
        }

        // Check cache first
        let cache_key = format!("{}:{}:{}", provider_type, self.config.model, text);
        if let Some(ref cache) = self.cache {
            if let Some(cached_embedding) = cache.get(&cache_key).await {
                return Ok(EmbeddingResult {
                    embedding: cached_embedding,
                    provider: provider_type.clone(),
                    model: self.config.model.clone(),
                    processing_time_ms: start_time.elapsed().as_millis() as u64,
                    text_length: text.len(),
                    cache_hit: true,
                });
            }
        }

        // Get provider
        let provider = self
            .providers
            .get(provider_type)
            .ok_or_else(|| EmbeddingError::ProviderNotAvailable(provider_type.to_string()))?;

        // Check if provider is available
        if !provider.is_available().await {
            return Err(EmbeddingError::ProviderNotAvailable(
                provider_type.to_string(),
            ));
        }

        // Generate embedding
        let embedding = provider.embed(text).await?;

        // Store in cache
        if let Some(ref cache) = self.cache {
            cache.put(cache_key, embedding.clone()).await;
        }

        Ok(EmbeddingResult {
            embedding,
            provider: provider_type.clone(),
            model: self.config.model.clone(),
            processing_time_ms: start_time.elapsed().as_millis() as u64,
            text_length: text.len(),
            cache_hit: false,
        })
    }

    /// Generate embeddings for multiple texts
    pub async fn embed_batch(
        &self,
        texts: &[String],
    ) -> Result<Vec<EmbeddingResult>, EmbeddingError> {
        let mut results = Vec::with_capacity(texts.len());

        for text in texts {
            let result = self.embed(text).await?;
            results.push(result);
        }

        Ok(results)
    }

    /// Get cache statistics
    pub async fn cache_stats(&self) -> Option<CacheStats> {
        if let Some(cache) = &self.cache {
            Some(cache.stats().await)
        } else {
            None
        }
    }

    /// Clear cache
    pub async fn clear_cache(&self) {
        if let Some(ref cache) = self.cache {
            cache.clear().await;
        }
    }

    /// Get available providers
    pub fn available_providers(&self) -> Vec<EmbeddingProviderType> {
        self.providers.keys().cloned().collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct MockProvider {
        name: String,
        dimension: usize,
    }

    #[async_trait::async_trait]
    impl EmbeddingProvider for MockProvider {
        async fn embed(&self, _text: &str) -> Result<Vec<f32>, EmbeddingError> {
            Ok(vec![1.0; self.dimension])
        }

        async fn embed_batch(&self, texts: &[String]) -> Result<Vec<Vec<f32>>, EmbeddingError> {
            Ok(vec![vec![1.0; self.dimension]; texts.len()])
        }

        fn dimension(&self) -> usize {
            self.dimension
        }

        fn name(&self) -> &str {
            &self.name
        }

        async fn is_available(&self) -> bool {
            true
        }
    }

    #[tokio::test]
    async fn test_embedding_cache() {
        let cache = EmbeddingCache::new(10);

        // Test cache miss
        assert!(cache.get("test").await.is_none());

        // Test cache put and get
        cache.put("test".to_string(), vec![1.0, 2.0, 3.0]).await;
        assert_eq!(cache.get("test").await, Some(vec![1.0, 2.0, 3.0]));

        // Test cache stats
        let stats = cache.stats().await;
        assert_eq!(stats.hits, 1);
        assert_eq!(stats.misses, 1);
        assert_eq!(stats.hit_rate, 0.5);
    }

    #[tokio::test]
    async fn test_embedding_manager() {
        let config = EmbeddingConfig::default();
        let mut manager = EmbeddingManager::new(config);

        let provider = Arc::new(MockProvider {
            name: "test".to_string(),
            dimension: 512,
        });

        manager.add_provider(EmbeddingProviderType::BM25, provider);

        let result = manager.embed("test text").await.unwrap();
        assert_eq!(result.embedding.len(), 512);
        assert_eq!(result.provider, EmbeddingProviderType::BM25);
        assert!(!result.cache_hit);

        // Test cache hit
        let result2 = manager.embed("test text").await.unwrap();
        assert!(result2.cache_hit);
    }

    #[test]
    fn test_embedding_config_default() {
        let config = EmbeddingConfig::default();
        assert_eq!(config.provider, EmbeddingProviderType::BM25);
        assert_eq!(config.dimension, 512);
        assert!(config.enable_caching);
    }
}
