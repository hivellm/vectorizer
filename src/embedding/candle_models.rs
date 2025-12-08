//! Real BERT and MiniLM embeddings using Candle
//!
//! This module provides actual BERT and MiniLM implementations using the Candle framework.
//! Only available when the "real-models" feature is enabled.

use std::path::PathBuf;
use std::sync::Arc;

use candle_core::{DType, Device, IndexOp, Tensor};
use candle_nn::VarBuilder;
use candle_transformers::models::bert::{BertModel, Config as BertConfig, DTYPE};
use hf_hub::api::sync::Api;
use hf_hub::{Repo, RepoType};
use parking_lot::RwLock;
use tokenizers::Tokenizer;
use tracing::{debug, error, info};

use crate::error::{Result, VectorizerError};

/// Real BERT embedding model using Candle
pub struct RealBertEmbedding {
    /// Model configuration
    config: BertConfig,
    /// Tokenizer
    tokenizer: Arc<Tokenizer>,
    /// BERT model
    model: Arc<RwLock<BertModel>>,
    /// Compute device (CPU or CUDA)
    device: Device,
    /// Model dimension
    dimension: usize,
    /// Maximum sequence length
    max_seq_len: usize,
}

impl std::fmt::Debug for RealBertEmbedding {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RealBertEmbedding")
            .field("dimension", &self.dimension)
            .field("max_seq_len", &self.max_seq_len)
            .field("device", &format!("{:?}", self.device))
            .finish()
    }
}

/// Real MiniLM embedding model using Candle
pub struct RealMiniLmEmbedding {
    /// Model configuration (MiniLM uses BERT architecture)
    config: BertConfig,
    /// Tokenizer
    tokenizer: Arc<Tokenizer>,
    /// MiniLM model (using BertModel)
    model: Arc<RwLock<BertModel>>,
    /// Compute device
    device: Device,
    /// Model dimension
    dimension: usize,
    /// Maximum sequence length
    max_seq_len: usize,
}

impl std::fmt::Debug for RealMiniLmEmbedding {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RealMiniLmEmbedding")
            .field("dimension", &self.dimension)
            .field("max_seq_len", &self.max_seq_len)
            .field("device", &format!("{:?}", self.device))
            .finish()
    }
}

impl RealBertEmbedding {
    /// Load BERT model from HuggingFace Hub
    ///
    /// # Arguments
    /// * `model_id` - HuggingFace model identifier (e.g., "bert-base-uncased")
    /// * `use_gpu` - Whether to use GPU acceleration if available
    ///
    /// # Returns
    /// * `Result<Self>` - Loaded BERT embedding model or error
    pub fn load_model(model_id: &str, use_gpu: bool) -> Result<Self> {
        info!("Loading BERT model: {}", model_id);

        // Setup device (CPU or CUDA)
        let device = if use_gpu && candle_core::utils::cuda_is_available() {
            Device::new_cuda(0).map_err(|e| {
                VectorizerError::Other(format!("Failed to initialize CUDA device: {}", e))
            })?
        } else {
            Device::Cpu
        };

        debug!("Using device: {:?}", device);

        // Download model files from HuggingFace Hub
        let api = Api::new().map_err(|e| {
            VectorizerError::Other(format!("Failed to initialize HuggingFace API: {}", e))
        })?;

        let repo = api.repo(Repo::new(model_id.to_string(), RepoType::Model));

        // Download config.json
        let config_path = repo.get("config.json").map_err(|e| {
            VectorizerError::Other(format!("Failed to download config.json: {}", e))
        })?;

        // Download model weights
        let weights_path = repo
            .get("pytorch_model.bin")
            .or_else(|_| repo.get("model.safetensors"))
            .map_err(|e| {
                VectorizerError::Other(format!("Failed to download model weights: {}", e))
            })?;

        // Download tokenizer
        let tokenizer_path = repo
            .get("tokenizer.json")
            .map_err(|e| VectorizerError::Other(format!("Failed to download tokenizer: {}", e)))?;

        // Load configuration
        let config: BertConfig =
            serde_json::from_str(&std::fs::read_to_string(config_path).map_err(|e| {
                VectorizerError::Other(format!("Failed to read config file: {}", e))
            })?)
            .map_err(|e| VectorizerError::Other(format!("Failed to parse config: {}", e)))?;

        let dimension = config.hidden_size;

        // Load tokenizer
        let tokenizer = Tokenizer::from_file(tokenizer_path)
            .map_err(|e| VectorizerError::Other(format!("Failed to load tokenizer: {}", e)))?;

        // Load model weights
        let vb = if weights_path.extension().and_then(|s| s.to_str()) == Some("safetensors") {
            unsafe {
                VarBuilder::from_mmaped_safetensors(&[weights_path], DTYPE, &device).map_err(
                    |e| VectorizerError::Other(format!("Failed to load safetensors: {}", e)),
                )?
            }
        } else {
            VarBuilder::from_pth(&weights_path, DTYPE, &device).map_err(|e| {
                VectorizerError::Other(format!("Failed to load PyTorch weights: {}", e))
            })?
        };

        // Initialize BERT model
        let model = BertModel::load(vb, &config).map_err(|e| {
            VectorizerError::Other(format!("Failed to initialize BERT model: {}", e))
        })?;

        info!(
            "Successfully loaded BERT model '{}' with dimension {}",
            model_id, dimension
        );

        Ok(Self {
            config,
            tokenizer: Arc::new(tokenizer),
            model: Arc::new(RwLock::new(model)),
            device,
            dimension,
            max_seq_len: 512, // BERT default
        })
    }

    /// Generate embeddings for a batch of texts
    pub fn embed_batch(&self, texts: &[&str]) -> Result<Vec<Vec<f32>>> {
        if texts.is_empty() {
            return Ok(Vec::new());
        }

        let mut all_embeddings = Vec::with_capacity(texts.len());

        // Process each text
        for text in texts {
            let embedding = self.embed_single(text)?;
            all_embeddings.push(embedding);
        }

        Ok(all_embeddings)
    }

    /// Generate embedding for a single text
    fn embed_single(&self, text: &str) -> Result<Vec<f32>> {
        // Tokenize input
        let encoding = self
            .tokenizer
            .encode(text, true)
            .map_err(|e| VectorizerError::Other(format!("Tokenization failed: {}", e)))?;

        let tokens = encoding.get_ids();

        // Truncate to max sequence length
        let tokens: Vec<u32> = tokens.iter().take(self.max_seq_len).copied().collect();

        // Create input tensors
        let token_ids = Tensor::new(&tokens[..], &self.device)
            .map_err(|e| VectorizerError::Other(format!("Failed to create token tensor: {}", e)))?;

        let token_ids = token_ids
            .unsqueeze(0)
            .map_err(|e| VectorizerError::Other(format!("Failed to add batch dimension: {}", e)))?;

        let token_type_ids = token_ids.zeros_like().map_err(|e| {
            VectorizerError::Other(format!("Failed to create token type IDs: {}", e))
        })?;

        // Run forward pass
        let model = self.model.read();
        let embeddings = model
            .forward(&token_ids, &token_type_ids, None)
            .map_err(|e| VectorizerError::Other(format!("BERT forward pass failed: {}", e)))?;

        // Use [CLS] token embedding (first token)
        let cls_embedding = embeddings
            .i((0, 0))
            .map_err(|e| VectorizerError::Other(format!("Failed to extract CLS token: {}", e)))?;

        // Convert to Vec<f32>
        let embedding_vec: Vec<f32> = cls_embedding.to_vec1().map_err(|e| {
            VectorizerError::Other(format!("Failed to convert embedding to vector: {}", e))
        })?;

        Ok(embedding_vec)
    }

    /// Get embedding dimension
    pub fn dimension(&self) -> usize {
        self.dimension
    }
}

impl RealMiniLmEmbedding {
    /// Load MiniLM model from HuggingFace Hub
    ///
    /// # Arguments
    /// * `model_id` - HuggingFace model identifier (e.g., "sentence-transformers/all-MiniLM-L6-v2")
    /// * `use_gpu` - Whether to use GPU acceleration if available
    ///
    /// # Returns
    /// * `Result<Self>` - Loaded MiniLM embedding model or error
    pub fn load_model(model_id: &str, use_gpu: bool) -> Result<Self> {
        info!("Loading MiniLM model: {}", model_id);

        // Setup device (CPU or CUDA)
        let device = if use_gpu && candle_core::utils::cuda_is_available() {
            Device::new_cuda(0).map_err(|e| {
                VectorizerError::Other(format!("Failed to initialize CUDA device: {}", e))
            })?
        } else {
            Device::Cpu
        };

        debug!("Using device: {:?}", device);

        // Download model files from HuggingFace Hub
        let api = Api::new().map_err(|e| {
            VectorizerError::Other(format!("Failed to initialize HuggingFace API: {}", e))
        })?;

        let repo = api.repo(Repo::new(model_id.to_string(), RepoType::Model));

        // Download config.json
        let config_path = repo.get("config.json").map_err(|e| {
            VectorizerError::Other(format!("Failed to download config.json: {}", e))
        })?;

        // Download model weights (try safetensors first, then pytorch)
        let weights_path = repo
            .get("model.safetensors")
            .or_else(|_| repo.get("pytorch_model.bin"))
            .map_err(|e| {
                VectorizerError::Other(format!("Failed to download model weights: {}", e))
            })?;

        // Download tokenizer
        let tokenizer_path = repo
            .get("tokenizer.json")
            .map_err(|e| VectorizerError::Other(format!("Failed to download tokenizer: {}", e)))?;

        // Load configuration
        let config: BertConfig =
            serde_json::from_str(&std::fs::read_to_string(config_path).map_err(|e| {
                VectorizerError::Other(format!("Failed to read config file: {}", e))
            })?)
            .map_err(|e| VectorizerError::Other(format!("Failed to parse config: {}", e)))?;

        let dimension = config.hidden_size;

        // Load tokenizer
        let tokenizer = Tokenizer::from_file(tokenizer_path)
            .map_err(|e| VectorizerError::Other(format!("Failed to load tokenizer: {}", e)))?;

        // Load model weights
        let vb = if weights_path.extension().and_then(|s| s.to_str()) == Some("safetensors") {
            unsafe {
                VarBuilder::from_mmaped_safetensors(&[weights_path], DTYPE, &device).map_err(
                    |e| VectorizerError::Other(format!("Failed to load safetensors: {}", e)),
                )?
            }
        } else {
            VarBuilder::from_pth(&weights_path, DTYPE, &device).map_err(|e| {
                VectorizerError::Other(format!("Failed to load PyTorch weights: {}", e))
            })?
        };

        // Initialize model (MiniLM uses BERT architecture)
        let model = BertModel::load(vb, &config).map_err(|e| {
            VectorizerError::Other(format!("Failed to initialize MiniLM model: {}", e))
        })?;

        info!(
            "Successfully loaded MiniLM model '{}' with dimension {}",
            model_id, dimension
        );

        Ok(Self {
            config,
            tokenizer: Arc::new(tokenizer),
            model: Arc::new(RwLock::new(model)),
            device,
            dimension,
            max_seq_len: 512,
        })
    }

    /// Generate embeddings for a batch of texts
    pub fn embed_batch(&self, texts: &[&str]) -> Result<Vec<Vec<f32>>> {
        if texts.is_empty() {
            return Ok(Vec::new());
        }

        let mut all_embeddings = Vec::with_capacity(texts.len());

        // Process each text
        for text in texts {
            let embedding = self.embed_single(text)?;
            all_embeddings.push(embedding);
        }

        Ok(all_embeddings)
    }

    /// Generate embedding for a single text with mean pooling
    fn embed_single(&self, text: &str) -> Result<Vec<f32>> {
        // Tokenize input
        let encoding = self
            .tokenizer
            .encode(text, true)
            .map_err(|e| VectorizerError::Other(format!("Tokenization failed: {}", e)))?;

        let tokens = encoding.get_ids();
        let attention_mask = encoding.get_attention_mask();

        // Truncate to max sequence length
        let tokens: Vec<u32> = tokens.iter().take(self.max_seq_len).copied().collect();
        let attention_mask: Vec<u32> = attention_mask
            .iter()
            .take(self.max_seq_len)
            .copied()
            .collect();

        // Create input tensors
        let token_ids = Tensor::new(&tokens[..], &self.device)
            .map_err(|e| VectorizerError::Other(format!("Failed to create token tensor: {}", e)))?;

        let token_ids = token_ids
            .unsqueeze(0)
            .map_err(|e| VectorizerError::Other(format!("Failed to add batch dimension: {}", e)))?;

        let token_type_ids = token_ids.zeros_like().map_err(|e| {
            VectorizerError::Other(format!("Failed to create token type IDs: {}", e))
        })?;

        // Run forward pass
        let model = self.model.read();
        let embeddings = model
            .forward(&token_ids, &token_type_ids, None)
            .map_err(|e| VectorizerError::Other(format!("MiniLM forward pass failed: {}", e)))?;

        // Mean pooling (average all token embeddings, weighted by attention mask)
        let attention_tensor = Tensor::new(&attention_mask[..], &self.device)
            .map_err(|e| {
                VectorizerError::Other(format!("Failed to create attention mask tensor: {}", e))
            })?
            .unsqueeze(0)
            .map_err(|e| {
                VectorizerError::Other(format!("Failed to add batch dimension to mask: {}", e))
            })?
            .unsqueeze(2)
            .map_err(|e| {
                VectorizerError::Other(format!("Failed to expand attention mask: {}", e))
            })?;

        let attention_expanded = attention_tensor.to_dtype(embeddings.dtype()).map_err(|e| {
            VectorizerError::Other(format!("Failed to convert attention mask dtype: {}", e))
        })?;

        let sum_embeddings = (embeddings * &attention_expanded)
            .map_err(|e| VectorizerError::Other(format!("Failed to apply attention mask: {}", e)))?
            .sum(1)
            .map_err(|e| VectorizerError::Other(format!("Failed to sum embeddings: {}", e)))?;

        let sum_mask = attention_expanded
            .sum(1)
            .map_err(|e| VectorizerError::Other(format!("Failed to sum attention mask: {}", e)))?
            .clamp(1e-9, f64::MAX)
            .map_err(|e| VectorizerError::Other(format!("Failed to clamp mask sum: {}", e)))?;

        let mean_pooled = (sum_embeddings / sum_mask).map_err(|e| {
            VectorizerError::Other(format!("Failed to compute mean pooling: {}", e))
        })?;

        // Extract embedding vector
        let embedding = mean_pooled
            .i(0)
            .map_err(|e| VectorizerError::Other(format!("Failed to extract embedding: {}", e)))?;

        // Convert to Vec<f32>
        let embedding_vec: Vec<f32> = embedding.to_vec1().map_err(|e| {
            VectorizerError::Other(format!("Failed to convert embedding to vector: {}", e))
        })?;

        Ok(embedding_vec)
    }

    /// Get embedding dimension
    pub fn dimension(&self) -> usize {
        self.dimension
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[ignore = "Requires downloading models from HuggingFace"]
    fn test_bert_model_loading() {
        let model = RealBertEmbedding::load_model("bert-base-uncased", false);
        assert!(model.is_ok());

        let model = model.unwrap();
        assert_eq!(model.dimension(), 768);
    }

    #[test]
    #[ignore = "Requires downloading models from HuggingFace"]
    fn test_bert_embedding_generation() {
        let model = RealBertEmbedding::load_model("bert-base-uncased", false).unwrap();

        let text = "This is a test sentence.";
        let embedding = model.embed_single(text);
        assert!(embedding.is_ok());

        let embedding = embedding.unwrap();
        assert_eq!(embedding.len(), 768);
    }

    #[test]
    #[ignore = "Requires downloading models from HuggingFace"]
    fn test_minilm_model_loading() {
        let model =
            RealMiniLmEmbedding::load_model("sentence-transformers/all-MiniLM-L6-v2", false);
        assert!(model.is_ok());

        let model = model.unwrap();
        assert_eq!(model.dimension(), 384);
    }

    #[test]
    #[ignore = "Requires downloading models from HuggingFace"]
    fn test_minilm_embedding_generation() {
        let model =
            RealMiniLmEmbedding::load_model("sentence-transformers/all-MiniLM-L6-v2", false)
                .unwrap();

        let text = "This is a test sentence.";
        let embedding = model.embed_single(text);
        assert!(embedding.is_ok());

        let embedding = embedding.unwrap();
        assert_eq!(embedding.len(), 384);
    }

    #[test]
    #[ignore = "Requires downloading models from HuggingFace"]
    fn test_batch_embedding() {
        let model =
            RealMiniLmEmbedding::load_model("sentence-transformers/all-MiniLM-L6-v2", false)
                .unwrap();

        let texts = vec!["First sentence.", "Second sentence.", "Third sentence."];
        let embeddings = model.embed_batch(&texts);
        assert!(embeddings.is_ok());

        let embeddings = embeddings.unwrap();
        assert_eq!(embeddings.len(), 3);
        assert_eq!(embeddings[0].len(), 384);
        assert_eq!(embeddings[1].len(), 384);
        assert_eq!(embeddings[2].len(), 384);
    }
}
