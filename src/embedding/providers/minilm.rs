//! MiniLM embedding provider. Real model gated behind `real-models`.
//!
//! Extracted from the monolithic `embedding/mod.rs` by
//! phase4_split-interleaved-embedding-providers.

use tracing::warn;

use crate::embedding::EmbeddingProvider;
#[cfg(feature = "real-models")]
use crate::embedding::candle_models;
use crate::error::{Result, VectorizerError};

#[derive(Debug)]
pub struct MiniLmEmbedding {
    /// MiniLM model dimension (384 typically)
    dimension: usize,
    /// Maximum sequence length
    #[allow(dead_code)]
    max_seq_len: usize,
    /// Whether the model is loaded
    loaded: bool,
    /// Real MiniLM model (only when real-models feature is enabled)
    #[cfg(feature = "real-models")]
    real_model: Option<candle_models::RealMiniLmEmbedding>,
}
impl MiniLmEmbedding {
    /// Create a new MiniLM embedding provider
    /// dimension: typically 384 for MiniLM models
    pub fn new(dimension: usize) -> Self {
        Self {
            dimension,
            max_seq_len: 256,
            loaded: false,
            #[cfg(feature = "real-models")]
            real_model: None,
        }
    }

    /// Load MiniLM model
    ///
    /// When "real-models" feature is enabled, this loads the actual MiniLM model from HuggingFace.
    /// Otherwise, it just marks the model as loaded (uses placeholder embeddings).
    pub fn load_model(&mut self) -> Result<()> {
        self.load_model_with_id("sentence-transformers/all-MiniLM-L6-v2", false)
    }

    /// Load MiniLM model with custom model ID
    ///
    /// # Arguments
    /// * `model_id` - HuggingFace model ID (e.g., "sentence-transformers/all-MiniLM-L6-v2")
    /// * `use_gpu` - Whether to use GPU acceleration if available
    pub fn load_model_with_id(&mut self, model_id: &str, use_gpu: bool) -> Result<()> {
        #[cfg(feature = "real-models")]
        {
            use tracing::info;
            info!("Loading real MiniLM model: {}", model_id);
            let model = candle_models::RealMiniLmEmbedding::load_model(model_id, use_gpu)?;
            self.dimension = model.dimension();
            self.real_model = Some(model);
            self.loaded = true;
            Ok(())
        }

        #[cfg(not(feature = "real-models"))]
        {
            warn!(
                "real-models feature not enabled. Using placeholder embeddings for MiniLM. Model ID '{}' ignored.",
                model_id
            );
            let _ = use_gpu; // Suppress unused warning
            self.loaded = true;
            Ok(())
        }
    }

    /// Simple hash-based embedding simulation (placeholder)
    fn simple_hash_embedding(&self, text: &str) -> Vec<f32> {
        // Similar to BERT but with different seed for variety
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        format!("minilm_{}", text).hash(&mut hasher);
        let seed = hasher.finish();

        let mut embedding = Vec::with_capacity(self.dimension);
        for i in 0..self.dimension {
            let value =
                ((seed.wrapping_mul(1103515245).wrapping_add(54321 + i as u64)) % 65536) as f32;
            embedding.push((value / 32768.0) - 1.0);
        }

        // L2 normalize
        let norm = embedding.iter().map(|x| x * x).sum::<f32>().sqrt();
        for value in &mut embedding {
            *value /= norm;
        }

        embedding
    }
}

impl EmbeddingProvider for MiniLmEmbedding {
    fn embed_batch(&self, texts: &[&str]) -> Result<Vec<Vec<f32>>> {
        if !self.loaded {
            return Err(VectorizerError::Other(
                "MiniLM model not loaded. Call load_model first.".to_string(),
            ));
        }

        #[cfg(feature = "real-models")]
        {
            if let Some(ref model) = self.real_model {
                return model.embed_batch(texts);
            }
        }

        // Fallback to placeholder
        texts.iter().map(|text| self.embed(text)).collect()
    }

    fn embed(&self, text: &str) -> Result<Vec<f32>> {
        if !self.loaded {
            return Err(VectorizerError::Other(
                "MiniLM model not loaded. Call load_model first.".to_string(),
            ));
        }

        #[cfg(feature = "real-models")]
        {
            if let Some(ref model) = self.real_model {
                return model.embed_batch(&[text]).and_then(|mut results| {
                    results.pop().ok_or_else(|| {
                        VectorizerError::Other("Failed to generate embedding".to_string())
                    })
                });
            }
        }

        // Fallback to placeholder
        Ok(self.simple_hash_embedding(text))
    }

    fn dimension(&self) -> usize {
        self.dimension
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}
