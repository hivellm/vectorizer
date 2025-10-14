//! Ultra-fast tokenization with caching and batch processing
//!
//! This module provides high-performance tokenization using HuggingFace tokenizers
//! with intelligent caching, batch processing, and memory-mapped persistence.

use anyhow::Result;
use arc_swap::ArcSwap;
use once_cell::sync::OnceCell;
use parking_lot::RwLock;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tokenizers::{PaddingParams, PaddingStrategy, TruncationParams, tokenizer::Tokenizer};
use xxhash_rust::xxh3::xxh3_64;

/// Global tokenizer cache shared across the application
static TOKENIZER_CACHE: OnceCell<Arc<TokenizerCache>> = OnceCell::new();

/// Configuration for fast tokenization
#[derive(Debug, Clone)]
pub struct FastTokenizerConfig {
    /// Maximum sequence length
    pub max_length: usize,
    /// Padding strategy
    pub padding: bool,
    /// Truncation strategy
    pub truncation: bool,
    /// Stride for overlapping chunks
    pub stride: usize,
    /// Cache directory for tokenizers
    pub cache_dir: PathBuf,
    /// Enable batch tokenization
    pub batch_size: usize,
    /// Number of threads for tokenization
    pub num_threads: usize,
}

impl Default for FastTokenizerConfig {
    fn default() -> Self {
        Self {
            max_length: 384,
            padding: true,
            truncation: true,
            stride: 50,
            cache_dir: PathBuf::from("./models/tokenizers"),
            batch_size: 128,
            num_threads: num_cpus::get(),
        }
    }
}

/// Fast tokenizer with caching and batch processing
pub struct FastTokenizer {
    /// The actual tokenizer instance
    tokenizer: Arc<Tokenizer>,
    /// Configuration
    config: FastTokenizerConfig,
    /// Token cache for deduplication
    token_cache: Arc<RwLock<HashMap<u64, Vec<u32>>>>,
}

impl FastTokenizer {
    /// Create a new fast tokenizer from a model path
    pub fn from_pretrained(model_name: &str, config: FastTokenizerConfig) -> Result<Self> {
        // Get or create global cache
        let cache =
            TOKENIZER_CACHE.get_or_init(|| Arc::new(TokenizerCache::new(config.cache_dir.clone())));

        // Load tokenizer with caching
        let tokenizer = cache.get_or_load(model_name)?;

        // Configure tokenizer
        let mut tokenizer = (*tokenizer).clone();

        // Set truncation
        if config.truncation {
            tokenizer
                .with_truncation(Some(TruncationParams {
                    max_length: config.max_length,
                    ..Default::default()
                }))
                .map_err(|e| anyhow::anyhow!("Truncation error: {:?}", e))?;
        }

        // Set padding
        if config.padding {
            tokenizer.with_padding(Some(PaddingParams {
                strategy: PaddingStrategy::Fixed(config.max_length),
                ..Default::default()
            }));
        }

        Ok(Self {
            tokenizer: Arc::new(tokenizer),
            config,
            token_cache: Arc::new(RwLock::new(HashMap::new())),
        })
    }

    /// Tokenize a single text with caching
    pub fn encode(&self, text: &str) -> Result<Vec<u32>> {
        // Compute hash for caching
        let hash = xxh3_64(text.as_bytes());

        // Check cache
        {
            let cache = self.token_cache.read();
            if let Some(tokens) = cache.get(&hash) {
                return Ok(tokens.clone());
            }
        }

        // Tokenize
        let encoding = self
            .tokenizer
            .encode(text, false)
            .map_err(|e| anyhow::anyhow!("Tokenization error: {:?}", e))?;
        let tokens = encoding.get_ids().to_vec();

        // Update cache
        {
            let mut cache = self.token_cache.write();
            cache.insert(hash, tokens.clone());
        }

        Ok(tokens)
    }

    /// Batch tokenize multiple texts efficiently
    pub fn encode_batch(&self, texts: &[&str]) -> Result<Vec<Vec<u32>>> {
        if texts.is_empty() {
            return Ok(vec![]);
        }

        // Process in batches for better performance
        let mut all_tokens = Vec::with_capacity(texts.len());

        for chunk in texts.chunks(self.config.batch_size) {
            // Batch tokenization
            let encodings = self
                .tokenizer
                .encode_batch(chunk.to_vec(), false)
                .map_err(|e| anyhow::anyhow!("Batch tokenization error: {:?}", e))?;

            // Extract token IDs
            for encoding in encodings {
                all_tokens.push(encoding.get_ids().to_vec());
            }
        }

        Ok(all_tokens)
    }

    /// Tokenize with chunking for long documents
    pub fn encode_chunked(&self, text: &str) -> Result<Vec<Vec<u32>>> {
        let words: Vec<&str> = text.split_whitespace().collect();

        if words.is_empty() {
            return Ok(vec![]);
        }

        let mut chunks = Vec::new();
        let chunk_size = self.config.max_length - 50; // Leave room for special tokens
        let stride = self.config.stride;

        let mut start = 0;
        while start < words.len() {
            let end = std::cmp::min(start + chunk_size, words.len());
            let chunk_text = words[start..end].join(" ");

            let tokens = self.encode(&chunk_text)?;
            chunks.push(tokens);

            if end >= words.len() {
                break;
            }

            start += chunk_size - stride;
        }

        Ok(chunks)
    }

    /// Get vocabulary size
    pub fn vocab_size(&self) -> usize {
        self.tokenizer.get_vocab_size(true)
    }

    /// Clear token cache
    pub fn clear_cache(&self) {
        self.token_cache.write().clear();
    }
}

/// Tokenizer cache for reusing loaded tokenizers
struct TokenizerCache {
    cache_dir: PathBuf,
    tokenizers: ArcSwap<HashMap<String, Arc<Tokenizer>>>,
}

impl TokenizerCache {
    fn new(cache_dir: PathBuf) -> Self {
        std::fs::create_dir_all(&cache_dir).ok();

        Self {
            cache_dir,
            tokenizers: ArcSwap::from_pointee(HashMap::new()),
        }
    }

    fn get_or_load(&self, model_name: &str) -> Result<Arc<Tokenizer>> {
        // Check in-memory cache
        {
            let cache = self.tokenizers.load();
            if let Some(tokenizer) = cache.get(model_name) {
                return Ok(tokenizer.clone());
            }
        }

        // Try to load from disk cache
        let cache_path = self
            .cache_dir
            .join(format!("{}.json", model_name.replace('/', "_")));

        let tokenizer = if cache_path.exists() {
            // Load from cache
            Tokenizer::from_file(&cache_path)
                .map_err(|e| anyhow::anyhow!("Failed to load tokenizer from cache: {:?}", e))?
        } else {
            // For now, return error - user must download tokenizer manually
            return Err(anyhow::anyhow!(
                "Tokenizer not found at {}. Please download the tokenizer.json from HuggingFace and place it there.",
                cache_path.display()
            ));
        };

        let tokenizer = Arc::new(tokenizer);

        // Update in-memory cache
        self.tokenizers.rcu(|cache| {
            let mut new_cache = (**cache).clone();
            new_cache.insert(model_name.to_string(), tokenizer.clone());
            new_cache
        });

        Ok(tokenizer)
    }
}

/// Performance benchmarking utilities
pub mod benchmark {
    use super::*;
    use std::time::Instant;

    pub struct TokenizationBenchmark {
        tokenizer: FastTokenizer,
    }

    impl TokenizationBenchmark {
        pub fn new(model_name: &str) -> Result<Self> {
            let config = FastTokenizerConfig::default();
            let tokenizer = FastTokenizer::from_pretrained(model_name, config)?;
            Ok(Self { tokenizer })
        }

        pub fn benchmark_single(&self, texts: &[&str]) -> Result<f64> {
            let start = Instant::now();

            for text in texts {
                let _ = self.tokenizer.encode(text)?;
            }

            let elapsed = start.elapsed();
            let tokens_per_sec = texts.len() as f64 / elapsed.as_secs_f64();

            Ok(tokens_per_sec)
        }

        pub fn benchmark_batch(&self, texts: &[&str]) -> Result<f64> {
            let start = Instant::now();

            let _ = self.tokenizer.encode_batch(texts)?;

            let elapsed = start.elapsed();
            let tokens_per_sec = texts.len() as f64 / elapsed.as_secs_f64();

            Ok(tokens_per_sec)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fast_tokenizer_single() {
        let _config = FastTokenizerConfig {
            max_length: 256,
            ..Default::default()
        };

        // This test would require actual tokenizer files
        // For unit testing, we'd mock the tokenizer
    }

    #[test]
    fn test_batch_tokenization() {
        // Test batch vs single tokenization performance
    }

    #[test]
    fn test_chunking() {
        // Test document chunking logic
    }
}
