//! `EmbeddingManager` — multi-provider registry + dispatch facade.
//! Extracted from the monolithic `embedding/mod.rs` under
//! phase4_split-embedding-providers. No behavior change.

use std::any::Any;
use std::collections::HashMap;
use std::path::Path;

use crate::embedding::EmbeddingProvider;
use crate::error::{Result, VectorizerError};

pub struct EmbeddingManager {
    providers: HashMap<String, Box<dyn EmbeddingProvider>>,
    default_provider: Option<String>,
}

impl EmbeddingManager {
    /// Create a new embedding manager
    pub fn new() -> Self {
        Self {
            providers: HashMap::new(),
            default_provider: None,
        }
    }

    /// Register an embedding provider
    pub fn register_provider(&mut self, name: String, provider: Box<dyn EmbeddingProvider>) {
        if self.default_provider.is_none() {
            self.default_provider = Some(name.clone());
        }
        self.providers.insert(name, provider);
    }

    /// Set the default provider
    pub fn set_default_provider(&mut self, name: &str) -> Result<()> {
        if self.providers.contains_key(name) {
            self.default_provider = Some(name.to_string());
            Ok(())
        } else {
            Err(VectorizerError::Other(format!(
                "Provider '{}' not found",
                name
            )))
        }
    }

    /// Get a provider by name
    pub fn get_provider(&self, name: &str) -> Result<&dyn EmbeddingProvider> {
        self.providers
            .get(name)
            .map(|p| p.as_ref())
            .ok_or_else(|| VectorizerError::Other(format!("Provider '{}' not found", name)))
    }

    /// Get a mutable provider by name
    pub fn get_provider_mut(&mut self, name: &str) -> Option<&mut Box<dyn EmbeddingProvider>> {
        self.providers.get_mut(name)
    }

    /// Get the default provider
    pub fn get_default_provider(&self) -> Result<&dyn EmbeddingProvider> {
        let provider_name = self
            .default_provider
            .as_ref()
            .ok_or_else(|| VectorizerError::Other("No default provider set".to_string()))?;

        self.get_provider(provider_name)
    }

    /// Get the default provider name
    pub fn get_default_provider_name(&self) -> Option<&str> {
        self.default_provider.as_deref()
    }

    /// Embed text using the default provider
    pub fn embed(&self, text: &str) -> Result<Vec<f32>> {
        self.get_default_provider()?.embed(text)
    }

    /// Embed batch of texts using the default provider
    pub fn embed_batch(&self, texts: &[&str]) -> Result<Vec<Vec<f32>>> {
        self.get_default_provider()?.embed_batch(texts)
    }

    /// Embed text using a specific provider by name
    pub fn embed_with_provider(&self, provider_name: &str, text: &str) -> Result<Vec<f32>> {
        let provider = self.get_provider(provider_name)?;
        provider.embed(text)
    }

    /// Embed batch of texts using a specific provider by name
    pub fn embed_batch_with_provider(
        &self,
        texts: &[&str],
        provider_name: &str,
    ) -> Result<Vec<Vec<f32>>> {
        self.get_provider(provider_name)?.embed_batch(texts)
    }

    /// Get the dimension of a specific provider
    pub fn get_provider_dimension(&self, provider_name: &str) -> Result<usize> {
        Ok(self.get_provider(provider_name)?.dimension())
    }

    /// List all available provider names
    pub fn list_providers(&self) -> Vec<String> {
        self.providers.keys().cloned().collect()
    }

    /// Check if a provider exists
    pub fn has_provider(&self, provider_name: &str) -> bool {
        self.providers.contains_key(provider_name)
    }

    /// Save vocabulary for a specific provider.
    ///
    /// Dispatches through `EmbeddingProvider::save_vocabulary_json`; providers
    /// that don't override the trait default return an error. Adding a new
    /// vocabulary-bearing provider is a single-file change (the impl block),
    /// not a two-file change as before.
    pub fn save_vocabulary_json<P: AsRef<Path>>(&self, provider_name: &str, path: P) -> Result<()> {
        let provider = self.get_provider(provider_name)?;
        provider.save_vocabulary_json(path.as_ref()).map_err(|e| {
            // Wrap the generic "not supported" message with the provider name
            // so the HTTP/MCP error stays actionable.
            VectorizerError::Other(format!("Provider '{}': {}", provider_name, e))
        })
    }
}

impl Default for EmbeddingManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;
    use crate::embedding::{
        BagOfWordsEmbedding, Bm25Embedding, CharNGramEmbedding, SvdEmbedding, TfIdfEmbedding,
    };

    #[test]
    fn test_tfidf_embedding() {
        let mut tfidf = TfIdfEmbedding::new(10);

        let corpus = vec![
            "machine learning is great",
            "deep learning is better",
            "vector databases store embeddings",
            "embeddings represent text as vectors",
        ];

        tfidf.build_vocabulary(&corpus);

        let embedding = tfidf.embed("machine learning vectors").unwrap();
        assert_eq!(embedding.len(), 10);

        // Check normalization
        let norm: f32 = embedding.iter().map(|x| x * x).sum::<f32>().sqrt();
        assert!((norm - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_bag_of_words() {
        let mut bow = BagOfWordsEmbedding::new(5);

        let corpus = vec!["hello world", "hello machine learning", "world of vectors"];

        bow.build_vocabulary(&corpus);

        let embedding = bow.embed("hello world").unwrap();
        assert_eq!(embedding.len(), 5);

        // Should have non-zero values for "hello" and "world"
        assert!(embedding.iter().any(|&x| x > 0.0));
    }

    #[test]
    fn test_char_ngram() {
        let mut ngram = CharNGramEmbedding::new(10, 3);

        let corpus = vec!["hello", "world", "hello world"];

        ngram.build_vocabulary(&corpus);

        let embedding = ngram.embed("hello").unwrap();
        assert_eq!(embedding.len(), 10);

        // Check normalization
        let norm: f32 = embedding.iter().map(|x| x * x).sum::<f32>().sqrt();
        assert!((norm - 1.0).abs() < 1e-6 || norm == 0.0);
    }

    #[test]
    fn test_embedding_manager() {
        let mut manager = EmbeddingManager::new();

        let tfidf = Box::new(TfIdfEmbedding::new(10));
        let bow = Box::new(BagOfWordsEmbedding::new(5));

        manager.register_provider("tfidf".to_string(), tfidf);
        manager.register_provider("bow".to_string(), bow);

        manager.set_default_provider("tfidf").unwrap();

        let provider = manager.get_provider("tfidf").unwrap();
        assert_eq!(provider.dimension(), 10);

        let default_provider = manager.get_default_provider().unwrap();
        assert_eq!(default_provider.dimension(), 10);
    }

    #[test]
    fn save_vocabulary_dispatches_through_trait_for_bm25() {
        let mut manager = EmbeddingManager::new();
        let mut bm25 = Bm25Embedding::new(32);
        let corpus: Vec<String> = vec!["hello world".into(), "machine learning".into()];
        bm25.build_vocabulary(&corpus);
        manager.register_provider("bm25".to_string(), Box::new(bm25));

        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("bm25.json");

        manager
            .save_vocabulary_json("bm25", &path)
            .expect("trait dispatch to BM25 override succeeds");

        let body = std::fs::read_to_string(&path).unwrap();
        assert!(
            body.contains("\"type\": \"bm25\""),
            "expected BM25 vocabulary JSON, got: {body}"
        );
    }

    #[test]
    fn save_vocabulary_errors_for_provider_without_override() {
        // SVD provider inherits the trait default, which returns an error.
        let mut manager = EmbeddingManager::new();
        manager.register_provider("svd".to_string(), Box::new(SvdEmbedding::new(16, 16)));

        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("svd.json");
        let err = manager.save_vocabulary_json("svd", &path).unwrap_err();
        let msg = err.to_string();
        assert!(
            msg.contains("svd") && msg.contains("does not support vocabulary saving"),
            "expected provider-aware error for SVD, got: {msg}"
        );
    }
}
