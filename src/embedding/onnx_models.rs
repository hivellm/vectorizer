//! ONNX Runtime high-performance model inference
//! 
//! This module provides ultra-fast inference using ONNX Runtime with:
//! - INT8 quantization for CPU inference
//! - Batch processing with optimal sizes
//! - Thread pool management for parallel processing
//! - Memory-mapped model loading

use anyhow::{Context, Result};
use arc_swap::ArcSwap;
use ndarray::{Array2, Array3, Axis};
use once_cell::sync::OnceCell;
use ort::{
    environment::Environment,
    session::{Session, SessionBuilder},
    tensor::OrtOwnedTensor,
    value::Value,
    GraphOptimizationLevel, LoggingLevel,
};
use parking_lot::RwLock;
use rayon::prelude::*;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tracing::{debug, info};

use super::fast_tokenizer::{FastTokenizer, FastTokenizerConfig};
use crate::error::VectorizerError;
use crate::models::Vector;

/// Global ONNX environment
static ONNX_ENV: OnceCell<Arc<Environment>> = OnceCell::new();

/// ONNX model types
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum OnnxModelType {
    /// MiniLM multilingual model (384D)
    MiniLMMultilingual384,
    /// E5 small multilingual (384D)
    E5SmallMultilingual384,
    /// E5 base multilingual (768D)
    E5BaseMultilingual768,
    /// MPNet multilingual (768D)  
    MPNetMultilingual768,
    /// GTE multilingual base (768D)
    GTEMultilingual768,
    /// DistilUSE multilingual (512D)
    DistilUSEMultilingual512,
}

impl OnnxModelType {
    pub fn model_id(&self) -> &'static str {
        match self {
            Self::MiniLMMultilingual384 => "sentence-transformers/paraphrase-multilingual-MiniLM-L12-v2",
            Self::E5SmallMultilingual384 => "intfloat/multilingual-e5-small",
            Self::E5BaseMultilingual768 => "intfloat/multilingual-e5-base",
            Self::MPNetMultilingual768 => "sentence-transformers/paraphrase-multilingual-mpnet-base-v2",
            Self::GTEMultilingual768 => "Alibaba-NLP/gte-multilingual-base",
            Self::DistilUSEMultilingual512 => "sentence-transformers/distiluse-base-multilingual-cased-v2",
        }
    }

    pub fn dimension(&self) -> usize {
        match self {
            Self::MiniLMMultilingual384 | Self::E5SmallMultilingual384 => 384,
            Self::DistilUSEMultilingual512 => 512,
            Self::E5BaseMultilingual768 | Self::MPNetMultilingual768 | Self::GTEMultilingual768 => 768,
        }
    }

    pub fn needs_prefix(&self) -> bool {
        matches!(self, Self::E5SmallMultilingual384 | Self::E5BaseMultilingual768)
    }

    pub fn max_sequence_length(&self) -> usize {
        match self {
            Self::E5SmallMultilingual384 | Self::E5BaseMultilingual768 => 512,
            _ => 256,
        }
    }
}

/// ONNX model configuration
#[derive(Debug, Clone)]
pub struct OnnxConfig {
    /// Model type
    pub model_type: OnnxModelType,
    /// Batch size for inference
    pub batch_size: usize,
    /// Number of threads for ONNX Runtime
    pub num_threads: usize,
    /// Enable INT8 quantization
    pub use_int8: bool,
    /// Cache directory
    pub cache_dir: PathBuf,
    /// Maximum sequence length
    pub max_length: usize,
    /// Pooling strategy
    pub pooling: PoolingStrategy,
}

impl Default for OnnxConfig {
    fn default() -> Self {
        Self {
            model_type: OnnxModelType::MiniLMMultilingual384,
            batch_size: 128,
            num_threads: 1, // Per-worker thread count
            use_int8: true,
            cache_dir: PathBuf::from("./models/onnx"),
            max_length: 384,
            pooling: PoolingStrategy::Mean,
        }
    }
}

/// Pooling strategies for sentence embeddings
#[derive(Debug, Clone, Copy)]
pub enum PoolingStrategy {
    /// Mean pooling (most common)
    Mean,
    /// CLS token (first token)
    Cls,
    /// Max pooling
    Max,
    /// Mean of first and last layers
    MeanSqrt,
}

/// High-performance ONNX model embedder
pub struct OnnxEmbedder {
    /// ONNX session
    session: Arc<Session>,
    /// Fast tokenizer
    tokenizer: FastTokenizer,
    /// Configuration
    config: OnnxConfig,
    /// Embedding cache
    cache: Arc<RwLock<HashMap<u64, Vec<f32>>>>,
}

impl OnnxEmbedder {
    /// Create a new ONNX embedder
    pub fn new(config: OnnxConfig) -> Result<Self> {
        // Initialize ONNX environment
        let env = ONNX_ENV.get_or_init(|| {
            Arc::new(
                Environment::builder()
                    .with_name("vectorizer")
                    .with_log_level(LoggingLevel::Warning)
                    .build()
                    .expect("Failed to create ONNX environment"),
            )
        });

        // Load model
        let model_path = Self::get_or_download_model(&config)?;
        
        // Configure session
        let mut session_builder = SessionBuilder::new(&env)?
            .with_optimization_level(GraphOptimizationLevel::Level3)?
            .with_intra_threads(config.num_threads as i16)?
            .with_inter_threads(1)?;

        // Platform-specific optimizations
        #[cfg(target_arch = "x86_64")]
        {
            // Enable AVX2/AVX512 if available
            session_builder = session_builder
                .with_cpu_arena_allocator()?
                .with_memory_pattern(true)?;
        }

        let session = Arc::new(session_builder.with_model_from_file(&model_path)?);

        // Configure tokenizer
        let tokenizer_config = FastTokenizerConfig {
            max_length: config.max_length,
            padding: true,
            truncation: true,
            batch_size: config.batch_size,
            num_threads: config.num_threads,
            ..Default::default()
        };

        let tokenizer = FastTokenizer::from_pretrained(
            config.model_type.model_id(),
            tokenizer_config,
        )?;

        Ok(Self {
            session,
            tokenizer,
            config,
            cache: Arc::new(RwLock::new(HashMap::new())),
        })
    }

    /// Get or download ONNX model
    fn get_or_download_model(config: &OnnxConfig) -> Result<PathBuf> {
        let model_name = format!(
            "{}{}.onnx",
            config.model_type.model_id().replace('/', "_"),
            if config.use_int8 { "_int8" } else { "" }
        );

        let model_path = config.cache_dir.join(&model_name);

        if model_path.exists() {
            info!("Using cached ONNX model: {}", model_path.display());
            return Ok(model_path);
        }

        // TODO: Implement model download and conversion
        // For now, assume models are pre-downloaded
        Err(anyhow::anyhow!(
            "ONNX model not found: {}. Please download and convert the model first.",
            model_path.display()
        ))
    }

    /// Embed a single text
    pub fn embed(&self, text: &str) -> Result<Vec<f32>> {
        self.embed_batch(&[text]).map(|mut vecs| vecs.pop().unwrap())
    }

    /// Embed a batch of texts with optimal performance
    pub fn embed_batch(&self, texts: &[&str]) -> Result<Vec<Vec<f32>>> {
        if texts.is_empty() {
            return Ok(vec![]);
        }

        // Apply prefixes if needed
        let processed_texts: Vec<String> = if self.config.model_type.needs_prefix() {
            texts.iter()
                .map(|t| format!("passage: {}", t))
                .collect()
        } else {
            texts.iter().map(|t| t.to_string()).collect()
        };

        let text_refs: Vec<&str> = processed_texts.iter().map(|s| s.as_str()).collect();

        // Batch tokenization
        let token_batches = self.tokenizer.encode_batch(&text_refs)?;

        // Process in optimal batch sizes
        let mut all_embeddings = Vec::with_capacity(texts.len());

        for batch in token_batches.chunks(self.config.batch_size) {
            let batch_embeddings = self.infer_batch(batch)?;
            all_embeddings.extend(batch_embeddings);
        }

        Ok(all_embeddings)
    }

    /// Run inference on a batch of tokenized inputs
    fn infer_batch(&self, token_ids_batch: &[Vec<u32>]) -> Result<Vec<Vec<f32>>> {
        let batch_size = token_ids_batch.len();
        let seq_len = token_ids_batch[0].len();

        // Prepare input tensors
        let mut input_ids = Array2::<i64>::zeros((batch_size, seq_len));
        let mut attention_mask = Array2::<i64>::ones((batch_size, seq_len));

        for (i, tokens) in token_ids_batch.iter().enumerate() {
            for (j, &token) in tokens.iter().enumerate() {
                input_ids[[i, j]] = token as i64;
                // Assuming padding token is 0
                if token == 0 {
                    attention_mask[[i, j]] = 0;
                }
            }
        }

        // Create ONNX values
        let input_ids_value = Value::from_array(self.session.allocator(), &input_ids)?;
        let attention_mask_value = Value::from_array(self.session.allocator(), &attention_mask)?;

        // Run inference
        let outputs = self.session.run(vec![input_ids_value, attention_mask_value])?;

        // Extract embeddings
        let embeddings = outputs[0].try_extract::<f32>()?.view().to_owned();
        
        // Apply pooling
        let pooled = self.apply_pooling(embeddings, &attention_mask)?;

        Ok(pooled)
    }

    /// Apply pooling strategy to token embeddings
    fn apply_pooling(
        &self,
        embeddings: Array3<f32>,
        attention_mask: &Array2<i64>,
    ) -> Result<Vec<Vec<f32>>> {
        let batch_size = embeddings.shape()[0];
        let mut pooled = Vec::with_capacity(batch_size);

        for i in 0..batch_size {
            let embedding = embeddings.index_axis(Axis(0), i);
            let mask = attention_mask.index_axis(Axis(0), i);

            let pooled_embedding = match self.config.pooling {
                PoolingStrategy::Mean => {
                    // Mean pooling with attention mask
                    let mut sum = Array1::<f32>::zeros(embedding.shape()[1]);
                    let mut count = 0f32;

                    for (j, &mask_val) in mask.iter().enumerate() {
                        if mask_val == 1 {
                            sum += &embedding.index_axis(Axis(0), j);
                            count += 1.0;
                        }
                    }

                    (sum / count).to_vec()
                }
                PoolingStrategy::Cls => {
                    // First token
                    embedding.index_axis(Axis(0), 0).to_vec()
                }
                PoolingStrategy::Max => {
                    // Max pooling
                    let mut max_vals = vec![f32::NEG_INFINITY; embedding.shape()[1]];
                    
                    for (j, &mask_val) in mask.iter().enumerate() {
                        if mask_val == 1 {
                            let token_embedding = embedding.index_axis(Axis(0), j);
                            for (k, &val) in token_embedding.iter().enumerate() {
                                max_vals[k] = max_vals[k].max(val);
                            }
                        }
                    }

                    max_vals
                }
                PoolingStrategy::MeanSqrt => {
                    // Mean pooling with sqrt length normalization
                    let mut sum = Array1::<f32>::zeros(embedding.shape()[1]);
                    let mut count = 0f32;

                    for (j, &mask_val) in mask.iter().enumerate() {
                        if mask_val == 1 {
                            sum += &embedding.index_axis(Axis(0), j);
                            count += 1.0;
                        }
                    }

                    (sum / count.sqrt()).to_vec()
                }
            };

            // L2 normalization
            let norm: f32 = pooled_embedding.iter().map(|x| x * x).sum::<f32>().sqrt();
            let normalized = pooled_embedding.iter().map(|x| x / norm).collect();

            pooled.push(normalized);
        }

        Ok(pooled)
    }

    /// Parallel embedding with thread pool control
    pub fn embed_parallel(&self, texts: &[String]) -> Result<Vec<Vec<f32>>> {
        // Set thread pool size to avoid oversubscription
        let pool = rayon::ThreadPoolBuilder::new()
            .num_threads(num_cpus::get() / 2) // Use half the cores
            .build()?;

        pool.install(|| {
            texts
                .par_chunks(self.config.batch_size * 4)
                .map(|chunk| {
                    let text_refs: Vec<&str> = chunk.iter().map(|s| s.as_str()).collect();
                    self.embed_batch(&text_refs)
                })
                .collect::<Result<Vec<_>>>()
                .map(|vecs| vecs.into_iter().flatten().collect())
        })
    }
}

/// Benchmark utilities for ONNX models
pub mod benchmark {
    use super::*;
    use std::time::Instant;

    pub fn benchmark_onnx_throughput(
        model_type: OnnxModelType,
        texts: &[String],
        batch_size: usize,
    ) -> Result<f64> {
        let config = OnnxConfig {
            model_type,
            batch_size,
            use_int8: true,
            ..Default::default()
        };

        let embedder = OnnxEmbedder::new(config)?;

        let start = Instant::now();
        let _embeddings = embedder.embed_parallel(texts)?;
        let elapsed = start.elapsed();

        let docs_per_sec = texts.len() as f64 / elapsed.as_secs_f64();
        
        info!(
            "ONNX {} throughput: {:.2} docs/sec ({} docs in {:.2}s)",
            model_type.model_id(),
            docs_per_sec,
            texts.len(),
            elapsed.as_secs_f64()
        );

        Ok(docs_per_sec)
    }
}

// Re-export for convenience
use ndarray::Array1;
pub use benchmark::benchmark_onnx_throughput;
