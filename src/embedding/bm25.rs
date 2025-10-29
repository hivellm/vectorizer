//! BM25 embedding provider
//!
//! BM25 (Best Matching 25) is a probabilistic ranking function used by search engines
//! to estimate the relevance of documents to a given search query. This implementation
//! provides sparse embeddings based on BM25 scoring.

use std::collections::HashMap;
use std::sync::Arc;

use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;

use crate::embedding::{EmbeddingError, EmbeddingProvider, EmbeddingProviderType};

/// BM25 configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BM25Config {
    /// k1 parameter (controls term frequency normalization)
    pub k1: f32,
    /// b parameter (controls length normalization)
    pub b: f32,
    /// Minimum term frequency
    pub min_term_freq: usize,
    /// Maximum vocabulary size
    pub max_vocab_size: usize,
    /// Enable IDF weighting
    pub enable_idf: bool,
    /// Smoothing parameter for IDF
    pub idf_smoothing: f32,
}

impl Default for BM25Config {
    fn default() -> Self {
        Self {
            k1: 1.2,
            b: 0.75,
            min_term_freq: 1,
            max_vocab_size: 100_000,
            enable_idf: true,
            idf_smoothing: 1.0,
        }
    }
}

/// BM25 embedding provider
pub struct BM25Provider {
    config: BM25Config,
    vocabulary: Arc<RwLock<HashMap<String, usize>>>,
    document_frequencies: Arc<RwLock<HashMap<String, usize>>>,
    document_count: Arc<RwLock<usize>>,
    average_document_length: Arc<RwLock<f32>>,
    total_document_length: Arc<RwLock<usize>>,
}

impl BM25Provider {
    /// Create a new BM25 provider
    pub fn new(config: BM25Config) -> Self {
        Self {
            config,
            vocabulary: Arc::new(RwLock::new(HashMap::new())),
            document_frequencies: Arc::new(RwLock::new(HashMap::new())),
            document_count: Arc::new(RwLock::new(0)),
            average_document_length: Arc::new(RwLock::new(0.0)),
            total_document_length: Arc::new(RwLock::new(0)),
        }
    }

    /// Create with default configuration
    pub fn default() -> Self {
        Self::new(BM25Config::default())
    }

    /// Add documents to the corpus for training - LIMITS to max_vocab_size TOP terms
    pub async fn add_documents(&self, documents: &[String]) -> Result<(), EmbeddingError> {
        use std::collections::HashMap;
        
        // Count term frequencies across ALL documents
        let mut global_term_freq: HashMap<String, usize> = HashMap::new();
        let mut doc_count_local = 0;
        let mut total_length_local = 0;
        
        for document in documents {
            let tokens = self.tokenize(document);
            let mut seen_terms = std::collections::HashSet::new();
            
            for token in &tokens {
                // Global frequency for vocabulary building
                *global_term_freq.entry(token.clone()).or_insert(0) += 1;
                seen_terms.insert(token.clone());
            }
            
            doc_count_local += 1;
            total_length_local += tokens.len();
        }
        
        // Select TOP max_vocab_size most frequent terms
        let mut term_freq_vec: Vec<(String, usize)> = global_term_freq.into_iter().collect();
        term_freq_vec.sort_by(|a, b| b.1.cmp(&a.1)); // Sort by frequency descending
        
        let max_terms = self.config.max_vocab_size.min(term_freq_vec.len());
        
        // Build vocabulary with TOP terms only
        let mut vocab = self.vocabulary.write().await;
        let mut doc_freqs = self.document_frequencies.write().await;
        
        for (idx, (term, freq)) in term_freq_vec.iter().take(max_terms).enumerate() {
            vocab.insert(term.clone(), idx);
            doc_freqs.insert(term.clone(), *freq);
        }
        
        // Update statistics
        *self.document_count.write().await = doc_count_local;
        *self.total_document_length.write().await = total_length_local;
        
        let avg_length = if doc_count_local > 0 {
            total_length_local as f32 / doc_count_local as f32
        } else {
            0.0
        };
        *self.average_document_length.write().await = avg_length;

        Ok(())
    }

    /// Tokenize text into terms
    fn tokenize(&self, text: &str) -> Vec<String> {
        text.to_lowercase()
            .chars()
            .filter(|c| c.is_alphanumeric() || c.is_whitespace())
            .collect::<String>()
            .split_whitespace()
            .map(|s| s.to_string())
            .collect()
    }

    /// Calculate BM25 score for a term
    async fn calculate_bm25_score(&self, term: &str, term_freq: usize, doc_length: usize) -> f32 {
        let vocab = self.vocabulary.read().await;
        let doc_freqs = self.document_frequencies.read().await;
        let doc_count = *self.document_count.read().await;
        let avg_doc_length = *self.average_document_length.read().await;

        if let Some(&term_id) = vocab.get(term) {
            let df = doc_freqs.get(term).copied().unwrap_or(0);

            if df == 0 {
                return 0.0;
            }

            // Calculate IDF
            let idf = if self.config.enable_idf {
                ((doc_count as f32 - df as f32 + self.config.idf_smoothing)
                    / (df as f32 + self.config.idf_smoothing))
                    .ln()
            } else {
                1.0
            };

            // Calculate term frequency component
            let tf = term_freq as f32;
            let length_norm =
                1.0 - self.config.b + (self.config.b * doc_length as f32 / avg_doc_length);

            idf * (tf * (self.config.k1 + 1.0)) / (tf + self.config.k1 * length_norm)
        } else {
            0.0
        }
    }

    /// Generate BM25 embedding for text
    async fn generate_embedding(&self, text: &str) -> Result<Vec<f32>, EmbeddingError> {
        let vocab = self.vocabulary.read().await;
        let tokens = self.tokenize(text);

        if vocab.is_empty() {
            return Err(EmbeddingError::Internal(
                "BM25 vocabulary is empty - must call add_documents() first".to_string(),
            ));
        }

        // ALWAYS create embedding with FIXED dimension (max_vocab_size)
        let fixed_dimension = self.config.max_vocab_size;
        let mut embedding = vec![0.0; fixed_dimension];
        let mut term_frequencies = HashMap::new();

        // Count term frequencies
        for token in &tokens {
            *term_frequencies.entry(token.clone()).or_insert(0) += 1;
        }

        // Calculate BM25 scores - only for terms in vocabulary
        for (term, freq) in term_frequencies {
            if let Some(&term_id) = vocab.get(&term) {
                if term_id < fixed_dimension {
                    let score = self.calculate_bm25_score(&term, freq, tokens.len()).await;
                    // Ensure score is valid (not NaN or infinite)
                    if score.is_finite() {
                        embedding[term_id] = score;
                    }
                }
            }
        }

        Ok(embedding)
    }

    /// Get vocabulary size
    pub async fn vocabulary_size(&self) -> usize {
        self.vocabulary.read().await.len()
    }

    /// Get document count
    pub async fn document_count(&self) -> usize {
        *self.document_count.read().await
    }
    
    /// Get vocabulary (for serialization)
    pub async fn get_vocabulary(&self) -> HashMap<String, usize> {
        self.vocabulary.read().await.clone()
    }
    
    /// Get document frequencies (for serialization)
    pub async fn get_document_frequencies(&self) -> HashMap<String, usize> {
        self.document_frequencies.read().await.clone()
    }
    
    /// Set vocabulary (for deserialization)
    pub async fn set_vocabulary(&self, vocab: HashMap<String, usize>) {
        *self.vocabulary.write().await = vocab;
    }
    
    /// Set document frequencies (for deserialization)
    pub async fn set_document_frequencies(&self, doc_freqs: HashMap<String, usize>) {
        *self.document_frequencies.write().await = doc_freqs;
    }
    
    /// Set statistics (for deserialization)
    pub async fn set_statistics(&self, doc_count: usize, avg_length: f32, total_length: usize) {
        *self.document_count.write().await = doc_count;
        *self.average_document_length.write().await = avg_length;
        *self.total_document_length.write().await = total_length;
    }

    /// Clear all data
    pub async fn clear(&self) {
        self.vocabulary.write().await.clear();
        self.document_frequencies.write().await.clear();
        *self.document_count.write().await = 0;
        *self.average_document_length.write().await = 0.0;
        *self.total_document_length.write().await = 0;
    }
}

#[async_trait::async_trait]
impl EmbeddingProvider for BM25Provider {
    async fn embed(&self, text: &str) -> Result<Vec<f32>, EmbeddingError> {
        self.generate_embedding(text).await
    }

    async fn embed_batch(&self, texts: &[String]) -> Result<Vec<Vec<f32>>, EmbeddingError> {
        let mut embeddings = Vec::with_capacity(texts.len());

        for text in texts {
            let embedding = self.generate_embedding(text).await?;
            embeddings.push(embedding);
        }

        Ok(embeddings)
    }

    fn dimension(&self) -> usize {
        // ALWAYS return FIXED dimension = max_vocab_size (e.g., 512)
        self.config.max_vocab_size
    }

    fn name(&self) -> &str {
        "BM25"
    }

    async fn is_available(&self) -> bool {
        // BM25 is always available - builds vocabulary on first use
        true
    }
    
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

/// BM25 factory for creating providers
pub struct BM25Factory;

impl BM25Factory {
    /// Create a new BM25 provider with default config
    pub fn create_default() -> BM25Provider {
        BM25Provider::default()
    }

    /// Create a new BM25 provider with custom config
    pub fn create_with_config(config: BM25Config) -> BM25Provider {
        BM25Provider::new(config)
    }

    /// Create a BM25 provider and train it on documents
    pub async fn create_and_train(
        documents: &[String],
        config: Option<BM25Config>,
    ) -> Result<BM25Provider, EmbeddingError> {
        let provider = if let Some(config) = config {
            BM25Provider::new(config)
        } else {
            BM25Provider::default()
        };

        provider.add_documents(documents).await?;
        Ok(provider)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_bm25_provider_creation() {
        let config = BM25Config::default();
        let provider = BM25Provider::new(config);

        assert_eq!(provider.name(), "BM25");
        assert_eq!(provider.dimension(), 0); // No vocabulary yet
        assert!(!provider.is_available().await);
    }

    #[tokio::test]
    async fn test_bm25_training() {
        let provider = BM25Provider::default();
        let documents = vec![
            "The quick brown fox jumps over the lazy dog".to_string(),
            "A brown dog is sleeping in the garden".to_string(),
            "The fox is very quick and agile".to_string(),
        ];

        provider.add_documents(&documents).await.unwrap();

        assert!(provider.is_available().await);
        assert!(provider.vocabulary_size().await > 0);
        assert_eq!(provider.document_count().await, 3);
    }

    #[tokio::test]
    async fn test_bm25_embedding() {
        let provider = BM25Provider::default();
        let documents = vec![
            "The quick brown fox jumps over the lazy dog".to_string(),
            "A brown dog is sleeping in the garden".to_string(),
        ];

        provider.add_documents(&documents).await.unwrap();

        let embedding = provider.embed("The quick brown fox").await.unwrap();
        assert_eq!(embedding.len(), provider.dimension());

        // Embedding should exist and have proper length
        assert!(!embedding.is_empty(), "Embedding should not be empty");
    }

    #[tokio::test]
    async fn test_bm25_batch_embedding() {
        let provider = BM25Provider::default();
        let documents = vec!["The quick brown fox jumps over the lazy dog".to_string()];

        provider.add_documents(&documents).await.unwrap();

        let texts = vec!["The quick brown fox".to_string(), "A lazy dog".to_string()];

        let embeddings = provider.embed_batch(&texts).await.unwrap();
        assert_eq!(embeddings.len(), 2);
        assert_eq!(embeddings[0].len(), provider.dimension());
        assert_eq!(embeddings[1].len(), provider.dimension());
    }

    #[tokio::test]
    async fn test_bm25_factory() {
        let documents = vec!["The quick brown fox jumps over the lazy dog".to_string()];

        let provider = BM25Factory::create_and_train(&documents, None)
            .await
            .unwrap();
        assert!(provider.is_available().await);
        assert!(provider.vocabulary_size().await > 0);
    }

    #[test]
    fn test_bm25_config_default() {
        let config = BM25Config::default();
        assert_eq!(config.k1, 1.2);
        assert_eq!(config.b, 0.75);
        assert!(config.enable_idf);
    }

    #[tokio::test]
    async fn test_bm25_clear() {
        let provider = BM25Provider::default();
        let documents = vec!["test document".to_string()];

        provider.add_documents(&documents).await.unwrap();
        assert!(provider.is_available().await);

        provider.clear().await;
        assert!(!provider.is_available().await);
        assert_eq!(provider.vocabulary_size().await, 0);
    }
}
