//! Real model implementations using Candle framework
//!
//! This module provides real transformer models for embeddings:
//! - MiniLM models (fast, efficient)
//! - BERT/MPNet models (more accurate)
//! - E5 models (optimized for retrieval)

#[cfg(feature = "candle-models")]
use candle_core::{Device, Tensor};
#[cfg(feature = "candle-models")]
use candle_nn::VarBuilder;
#[cfg(feature = "candle-models")]
use candle_transformers::models::bert::{BertModel, Config as BertConfig};
#[cfg(feature = "candle-models")]
use hf_hub::api::sync::ApiBuilder;
#[cfg(feature = "candle-models")]
use serde_json;
#[cfg(feature = "candle-models")]
use tokenizers::Tokenizer;

use super::EmbeddingProvider;
use crate::error::{Result, VectorizerError};

/// Available real models for download and use
#[derive(Debug, Clone, Copy)]
pub enum RealModelType {
    /// Fast multilingual MiniLM (384D)
    MiniLMMultilingual,
    /// DistilUSE multilingual (512D)
    DistilUseMultilingual,
    /// MPNet multilingual base (768D)
    MPNetMultilingualBase,
    /// E5 small multilingual (384D)
    E5SmallMultilingual,
    /// E5 base multilingual (768D)
    E5BaseMultilingual,
    /// Alibaba GTE multilingual base (768D)
    GTEMultilingualBase,
    /// LaBSE multilingual (768D)
    LaBSE,
}

impl RealModelType {
    /// Get HuggingFace model ID
    pub fn model_id(&self) -> &str {
        match self {
            RealModelType::MiniLMMultilingual => {
                "sentence-transformers/paraphrase-multilingual-MiniLM-L12-v2"
            }
            RealModelType::DistilUseMultilingual => {
                "sentence-transformers/distiluse-base-multilingual-cased-v2"
            }
            RealModelType::MPNetMultilingualBase => {
                "sentence-transformers/paraphrase-multilingual-mpnet-base-v2"
            }
            RealModelType::E5SmallMultilingual => "intfloat/multilingual-e5-small",
            RealModelType::E5BaseMultilingual => "intfloat/multilingual-e5-base",
            RealModelType::GTEMultilingualBase => "Alibaba-NLP/gte-multilingual-base",
            RealModelType::LaBSE => "sentence-transformers/LaBSE",
        }
    }

    /// Get expected embedding dimension
    pub fn dimension(&self) -> usize {
        match self {
            RealModelType::MiniLMMultilingual => 384,
            RealModelType::DistilUseMultilingual => 512,
            RealModelType::MPNetMultilingualBase => 768,
            RealModelType::E5SmallMultilingual => 384,
            RealModelType::E5BaseMultilingual => 768,
            RealModelType::GTEMultilingualBase => 768,
            RealModelType::LaBSE => 768,
        }
    }

    /// Check if model needs query/passage prefix (for E5 models)
    pub fn needs_prefix(&self) -> bool {
        matches!(
            self,
            RealModelType::E5SmallMultilingual | RealModelType::E5BaseMultilingual
        )
    }
}

/// Real transformer-based embedding model
#[cfg(feature = "candle-models")]
pub struct RealModelEmbedder {
    model: BertModel,
    tokenizer: Tokenizer,
    device: Device,
    model_type: RealModelType,
    #[allow(dead_code)]
    model_cache_dir: std::path::PathBuf,
}

/// Placeholder struct when Candle is not available
#[cfg(not(feature = "candle-models"))]
pub struct RealModelEmbedder {
    model_type: RealModelType,
    #[allow(dead_code)]
    dimension: usize,
}

#[cfg(feature = "candle-models")]
impl RealModelEmbedder {
    /// Create a new real model embedder
    pub fn new(model_type: RealModelType) -> Result<Self> {
        Self::new_with_cache(model_type, std::path::PathBuf::from("./models"))
    }

    /// Create a new real model embedder with custom cache directory
    pub fn new_with_cache(
        model_type: RealModelType,
        cache_dir: std::path::PathBuf,
    ) -> Result<Self> {
        let device = Device::Cpu; // Use CPU for now, can be extended to GPU
        let model_id = model_type.model_id();

        println!(
            "Loading model: {} (cache: {})",
            model_id,
            cache_dir.display()
        );

        // Create cache directory if it doesn't exist
        std::fs::create_dir_all(&cache_dir).map_err(|e| {
            VectorizerError::Other(format!("Failed to create cache directory: {}", e))
        })?;

        // Download model files with custom cache location
        let api = ApiBuilder::new()
            .with_progress(true)
            .with_cache_dir(cache_dir.clone())
            .build()
            .map_err(|e| VectorizerError::Other(format!("Failed to create HF API: {}", e)))?;
        let repo = api.model(model_id.to_string());

        // Load tokenizer
        let tokenizer_path = repo
            .get("tokenizer.json")
            .map_err(|e| VectorizerError::Other(format!("Failed to download tokenizer: {}", e)))?;
        let tokenizer = Tokenizer::from_file(tokenizer_path)
            .map_err(|e| VectorizerError::Other(format!("Failed to load tokenizer: {}", e)))?;

        // Load model config
        let config_path = repo
            .get("config.json")
            .map_err(|e| VectorizerError::Other(format!("Failed to download config: {}", e)))?;
        let config: BertConfig = serde_json::from_reader(
            std::fs::File::open(config_path)
                .map_err(|e| VectorizerError::Other(format!("Failed to open config: {}", e)))?,
        )
        .map_err(|e| VectorizerError::Other(format!("Failed to parse config: {}", e)))?;

        // Load model weights
        let weights_path = repo
            .get("pytorch_model.bin")
            .or_else(|_| repo.get("model.safetensors"))
            .map_err(|e| {
                VectorizerError::Other(format!("Failed to download model weights: {}", e))
            })?;

        let vb = if weights_path.extension().unwrap_or_default() == "safetensors" {
            // For safetensors, we need to load the tensors and create VarBuilder from them
            let tensors = candle_core::safetensors::load(weights_path, &device)?;
            VarBuilder::from_tensors(tensors, candle_core::DType::F32, &device)
        } else {
            VarBuilder::from_pth(weights_path, candle_core::DType::F32, &device)?
        };

        let model = BertModel::load(vb, &config)
            .map_err(|e| VectorizerError::Other(format!("Failed to load BERT model: {}", e)))?;

        println!(
            "Successfully loaded model: {} (cached in {})",
            model_id,
            cache_dir.display()
        );

        Ok(Self {
            model,
            tokenizer,
            device,
            model_type,
            model_cache_dir: cache_dir,
        })
    }

    /// Encode text and return embeddings
    fn encode(&self, texts: &[&str]) -> Result<Vec<Vec<f32>>> {
        let mut embeddings = Vec::new();

        for text in texts {
            // Add prefix for E5 models
            let processed_text = if self.model_type.needs_prefix() {
                format!("passage: {}", text)
            } else {
                text.to_string()
            };

            // Tokenize
            let tokens = self
                .tokenizer
                .encode(processed_text, true)
                .map_err(|e| VectorizerError::Other(format!("Tokenization failed: {}", e)))?;

            let input_ids = Tensor::new(tokens.get_ids(), &self.device)?.unsqueeze(0)?;
            let attention_mask =
                Tensor::new(tokens.get_attention_mask(), &self.device)?.unsqueeze(0)?;
            let token_type_ids = Tensor::new(tokens.get_type_ids(), &self.device)?.unsqueeze(0)?;

            // Forward pass - get the hidden states from BERT
            // The BERT model returns the last hidden state directly
            let hidden_states =
                self.model
                    .forward(&input_ids, &attention_mask, Some(&token_type_ids))?;

            // Mean pooling (simple average of token embeddings)
            // For sentence embeddings, we typically take the mean of all token embeddings
            let pooled = hidden_states.mean(1)?;

            // Normalize
            let norm = pooled.sqr()?.mean_all()?.sqrt()?;
            let normalized = pooled.broadcast_div(&norm)?;

            // Convert to Vec<f32>
            let embedding: Vec<f32> = normalized.squeeze(0)?.to_vec1()?;

            embeddings.push(embedding);
        }

        Ok(embeddings)
    }
}

#[cfg(feature = "candle-models")]
impl EmbeddingProvider for RealModelEmbedder {
    fn embed_batch(&self, texts: &[&str]) -> Result<Vec<Vec<f32>>> {
        self.encode(texts)
    }

    fn embed(&self, text: &str) -> Result<Vec<f32>> {
        let embeddings = self.encode(&[text])?;
        Ok(embeddings
            .into_iter()
            .next()
            .ok_or_else(|| VectorizerError::Other("No embedding generated".to_string()))?)
    }

    fn dimension(&self) -> usize {
        self.model_type.dimension()
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

#[cfg(not(feature = "candle-models"))]
impl RealModelEmbedder {
    pub fn new(model_type: RealModelType) -> Result<Self> {
        println!(
            "⚠️  Candle models feature not enabled. Using placeholder implementation for {}",
            model_type.model_id()
        );
        Ok(Self {
            model_type,
            dimension: model_type.dimension(),
        })
    }
}

#[cfg(not(feature = "candle-models"))]
impl EmbeddingProvider for RealModelEmbedder {
    fn embed_batch(&self, _texts: &[&str]) -> Result<Vec<Vec<f32>>> {
        Err(VectorizerError::Other(
            "Real models feature not enabled. Compile with --features real-models".to_string(),
        ))
    }

    fn embed(&self, _text: &str) -> Result<Vec<f32>> {
        Err(VectorizerError::Other(
            "Real models feature not enabled. Compile with --features real-models".to_string(),
        ))
    }

    fn dimension(&self) -> usize {
        self.model_type.dimension()
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_real_model_type_model_ids() {
        assert_eq!(
            RealModelType::MiniLMMultilingual.model_id(),
            "sentence-transformers/paraphrase-multilingual-MiniLM-L12-v2"
        );
        assert_eq!(
            RealModelType::DistilUseMultilingual.model_id(),
            "sentence-transformers/distiluse-base-multilingual-cased-v2"
        );
        assert_eq!(
            RealModelType::MPNetMultilingualBase.model_id(),
            "sentence-transformers/paraphrase-multilingual-mpnet-base-v2"
        );
        assert_eq!(
            RealModelType::E5SmallMultilingual.model_id(),
            "intfloat/multilingual-e5-small"
        );
        assert_eq!(
            RealModelType::E5BaseMultilingual.model_id(),
            "intfloat/multilingual-e5-base"
        );
        assert_eq!(
            RealModelType::GTEMultilingualBase.model_id(),
            "Alibaba-NLP/gte-multilingual-base"
        );
        assert_eq!(
            RealModelType::LaBSE.model_id(),
            "sentence-transformers/LaBSE"
        );
    }

    #[test]
    fn test_real_model_type_dimensions() {
        assert_eq!(RealModelType::MiniLMMultilingual.dimension(), 384);
        assert_eq!(RealModelType::DistilUseMultilingual.dimension(), 512);
        assert_eq!(RealModelType::MPNetMultilingualBase.dimension(), 768);
        assert_eq!(RealModelType::E5SmallMultilingual.dimension(), 384);
        assert_eq!(RealModelType::E5BaseMultilingual.dimension(), 768);
        assert_eq!(RealModelType::GTEMultilingualBase.dimension(), 768);
        assert_eq!(RealModelType::LaBSE.dimension(), 768);
    }

    #[test]
    fn test_real_model_type_needs_prefix() {
        assert!(!RealModelType::MiniLMMultilingual.needs_prefix());
        assert!(!RealModelType::DistilUseMultilingual.needs_prefix());
        assert!(!RealModelType::MPNetMultilingualBase.needs_prefix());
        assert!(RealModelType::E5SmallMultilingual.needs_prefix());
        assert!(RealModelType::E5BaseMultilingual.needs_prefix());
        assert!(!RealModelType::GTEMultilingualBase.needs_prefix());
        assert!(!RealModelType::LaBSE.needs_prefix());
    }

    #[test]
    fn test_real_model_embedder_creation_without_candle() {
        // Without candle-models feature, should still create placeholder
        let embedder = RealModelEmbedder::new(RealModelType::MiniLMMultilingual);
        assert!(embedder.is_ok());
    }

    #[test]
    fn test_real_model_embedder_dimension() {
        let embedder = RealModelEmbedder::new(RealModelType::MPNetMultilingualBase).unwrap();
        assert_eq!(embedder.dimension(), 768);
    }

    #[test]
    #[cfg(not(feature = "candle-models"))]
    fn test_real_model_embedder_embed_fails_without_candle() {
        let embedder = RealModelEmbedder::new(RealModelType::E5SmallMultilingual).unwrap();
        let result = embedder.embed("test text");

        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("not enabled"));
    }

    #[test]
    #[cfg(not(feature = "candle-models"))]
    fn test_real_model_embedder_embed_batch_fails_without_candle() {
        let embedder = RealModelType::LaBSE;
        let real_embedder = RealModelEmbedder::new(embedder).unwrap();
        let result = real_embedder.embed_batch(&["text1", "text2"]);

        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("not enabled"));
    }

    #[test]
    fn test_real_model_type_clone() {
        let model1 = RealModelType::MiniLMMultilingual;
        let model2 = model1;

        assert_eq!(model1.model_id(), model2.model_id());
        assert_eq!(model1.dimension(), model2.dimension());
    }

    #[test]
    fn test_real_model_type_debug() {
        let model = RealModelType::E5SmallMultilingual;
        let debug_str = format!("{:?}", model);

        assert!(!debug_str.is_empty());
        assert!(debug_str.contains("E5SmallMultilingual"));
    }
}
