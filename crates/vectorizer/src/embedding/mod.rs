//! Embedding generation module for converting text to vectors
//!
//! # Layout
//!
//! The sparse / dense / native-model providers live in sibling files
//! under `providers/`. This module only declares the
//! `EmbeddingProvider` trait and the submodule wiring; concrete
//! implementations are re-exported so external callers still see
//! `crate::embedding::{TfIdfEmbedding, Bm25Embedding, SvdEmbedding,
//! BertEmbedding, MiniLmEmbedding, BagOfWordsEmbedding,
//! CharNGramEmbedding, EmbeddingManager}` unchanged.

use std::path::Path;

use crate::error::{Result, VectorizerError};

// Real models implementation (only when real-models feature is enabled)
#[cfg(feature = "real-models")]
pub mod candle_models;

/// Trait for embedding providers
pub trait EmbeddingProvider: Send + Sync {
    /// Generate embeddings for a batch of texts
    fn embed_batch(&self, texts: &[&str]) -> Result<Vec<Vec<f32>>>;

    /// Generate embedding for a single text
    fn embed(&self, text: &str) -> Result<Vec<f32>> {
        let results = self.embed_batch(&[text])?;
        results
            .into_iter()
            .next()
            .ok_or_else(|| VectorizerError::Other("Failed to generate embedding".to_string()))
    }

    /// Get the dimension of embeddings produced by this provider
    fn dimension(&self) -> usize;

    /// Persist this provider's vocabulary (if any) to a JSON file.
    ///
    /// Default implementation returns an error — providers that have no
    /// vocabulary to serialize (e.g. embedding models that bake weights into
    /// the binary) leave this unimplemented. Sparse providers like BM25,
    /// TF-IDF, CharNGram, and BagOfWords override it.
    ///
    /// This replaces an older `downcast_ref` if-chain in
    /// `EmbeddingManager::save_vocabulary_json` that forced every new
    /// provider to be enumerated in two places.
    fn save_vocabulary_json(&self, _path: &Path) -> Result<()> {
        Err(VectorizerError::Other(
            "Provider does not support vocabulary saving".to_string(),
        ))
    }

    /// Cast to Any for downcasting (mutable)
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any;

    /// Cast to Any for downcasting (immutable)
    fn as_any(&self) -> &dyn std::any::Any;
}

pub mod providers;
pub use providers::{
    BagOfWordsEmbedding, BertEmbedding, Bm25Embedding, CharNGramEmbedding, EmbeddingManager,
    MiniLmEmbedding, SvdEmbedding, TfIdfEmbedding,
};

// Real models module
pub mod real_models;

// Performance modules
#[cfg(feature = "tokenizers")]
pub mod fast_tokenizer;

#[cfg(feature = "onnx-models")]
pub mod onnx_models;

pub mod cache;

// Re-export real models
pub use cache::{CacheConfig, EmbeddingCache};
// Re-export performance modules
#[cfg(feature = "tokenizers")]
pub use fast_tokenizer::{FastTokenizer, FastTokenizerConfig};
#[cfg(feature = "onnx-models")]
pub use onnx_models::{OnnxConfig, OnnxEmbedder, OnnxModelType, PoolingStrategy};
pub use real_models::{RealModelEmbedder, RealModelType};
