//! `BagOfWordsEmbedding` provider — extracted from the monolithic
//! `embedding/mod.rs` under phase4_split-embedding-providers. No
//! behavior change; the struct and impls are byte-for-byte the same.

use std::any::Any;
use std::collections::HashMap;
use std::fs;
use std::path::Path;

use serde::{Deserialize, Serialize};
use tracing::warn;

use crate::embedding::EmbeddingProvider;
use crate::error::{Result, VectorizerError};

pub struct BagOfWordsEmbedding {
    dimension: usize,
    vocabulary: HashMap<String, usize>,
}

impl BagOfWordsEmbedding {
    /// Create a new Bag-of-Words embedding provider
    pub fn new(dimension: usize) -> Self {
        Self {
            dimension,
            vocabulary: HashMap::new(),
        }
    }

    /// Build vocabulary from texts
    pub fn build_vocabulary(&mut self, texts: &[&str]) {
        let mut word_counts: HashMap<String, usize> = HashMap::new();

        for text in texts {
            let words = self.tokenize(text);
            for word in words {
                *word_counts.entry(word).or_insert(0) += 1;
            }
        }

        // Select top words by frequency, with alphabetical tie-breaking for determinism
        let mut word_freq: Vec<(String, usize)> = word_counts.into_iter().collect();
        word_freq.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));

        self.vocabulary.clear();
        for (i, (word, _)) in word_freq.iter().take(self.dimension).enumerate() {
            self.vocabulary.insert(word.clone(), i);
        }
    }

    fn tokenize(&self, text: &str) -> Vec<String> {
        text.to_lowercase()
            .split_whitespace()
            .filter(|w| w.len() > 2)
            .map(|w| w.trim_matches(|c: char| !c.is_alphanumeric()).to_string())
            .filter(|w| !w.is_empty())
            .collect()
    }
}

impl EmbeddingProvider for BagOfWordsEmbedding {
    fn embed(&self, text: &str) -> Result<Vec<f32>> {
        let words = self.tokenize(text);
        let mut embedding = vec![0.0; self.dimension];

        for word in words {
            if let Some(&idx) = self.vocabulary.get(&word) {
                embedding[idx] += 1.0;
            }
        }

        // Check if embedding is all zeros
        let non_zero_count = embedding.iter().filter(|&&x| x != 0.0).count();
        if non_zero_count == 0 {
            warn!(
                "WARNING: BagOfWordsEmbedding produced all-zero embedding for '{}'",
                text
            );
            warn!("Vocabulary size: {}", self.vocabulary.len());

            // Fallback: Generate a simple hash-based embedding to ensure non-zero vector
            warn!("Using fallback hash-based embedding");
            return Ok(self.fallback_hash_embedding(text));
        }

        // Normalize
        let norm: f32 = embedding.iter().map(|x| x * x).sum::<f32>().sqrt();
        if norm > 0.0 {
            for value in &mut embedding {
                *value /= norm;
            }
        }

        Ok(embedding)
    }

    fn embed_batch(&self, texts: &[&str]) -> Result<Vec<Vec<f32>>> {
        texts.iter().map(|text| self.embed(text)).collect()
    }

    fn dimension(&self) -> usize {
        self.dimension
    }

    fn save_vocabulary_json(&self, path: &Path) -> Result<()> {
        BagOfWordsEmbedding::save_vocabulary_json(self, path)
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl BagOfWordsEmbedding {
    /// Fallback hash-based embedding when vocabulary is empty or no matches found
    fn fallback_hash_embedding(&self, text: &str) -> Vec<f32> {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        text.hash(&mut hasher);
        let seed = hasher.finish();

        // Generate pseudo-random but deterministic embedding
        let mut embedding = Vec::with_capacity(self.dimension);
        for i in 0..self.dimension {
            // Simple LCG-like generator seeded by text hash
            let value =
                ((seed.wrapping_mul(1103515245).wrapping_add(12345 + i as u64)) % 65536) as f32;
            embedding.push((value / 32768.0) - 1.0); // Normalize to [-1, 1]
        }

        // L2 normalize
        let norm = embedding.iter().map(|x| x * x).sum::<f32>().sqrt();
        for value in &mut embedding {
            *value /= norm;
        }

        embedding
    }

    /// Save BagOfWords vocabulary/tokenizer JSON
    pub fn save_vocabulary_json<P: AsRef<std::path::Path>>(&self, path: P) -> Result<()> {
        let data = serde_json::json!({
            "type": "bagofwords",
            "dimension": self.dimension,
            "vocabulary": self.vocabulary,
        });
        let json = serde_json::to_string_pretty(&data)
            .map_err(|e| VectorizerError::Other(format!("Failed to serialize BoW vocab: {}", e)))?;
        std::fs::write(path.as_ref(), json).map_err(|e| {
            VectorizerError::Other(format!(
                "Failed to write BoW vocab {}: {}",
                path.as_ref().display(),
                e
            ))
        })?;
        Ok(())
    }

    /// Load BagOfWords vocabulary/tokenizer JSON
    pub fn load_vocabulary_json<P: AsRef<std::path::Path>>(&mut self, path: P) -> Result<()> {
        let content = std::fs::read_to_string(path.as_ref()).map_err(|e| {
            VectorizerError::Other(format!(
                "Failed to read BoW vocab {}: {}",
                path.as_ref().display(),
                e
            ))
        })?;
        let v: serde_json::Value = serde_json::from_str(&content).map_err(|e| {
            VectorizerError::Other(format!(
                "Failed to parse BoW vocab {}: {}",
                path.as_ref().display(),
                e
            ))
        })?;
        let t = v.get("type").and_then(|x| x.as_str()).unwrap_or("");
        if t != "bagofwords" {
            return Err(VectorizerError::Other(format!(
                "Tokenizer type mismatch: expected bagofwords, found {}",
                t
            )));
        }
        self.dimension = v
            .get("dimension")
            .and_then(|x| x.as_u64())
            .unwrap_or(self.dimension as u64) as usize;
        self.vocabulary = v
            .get("vocabulary")
            .and_then(|x| serde_json::from_value(x.clone()).ok())
            .unwrap_or_default();
        Ok(())
    }
}
