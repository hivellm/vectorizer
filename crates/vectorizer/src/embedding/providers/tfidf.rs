//! TF-IDF embedding provider.
//!
//! Extracted from the monolithic `embedding/mod.rs` by
//! phase4_split-interleaved-embedding-providers. The `SvdEmbedding`
//! provider sits on top of this one — see `svd.rs`.

use std::collections::HashMap;
use std::fs;
use std::path::Path;

use crate::embedding::EmbeddingProvider;
use crate::error::{Result, VectorizerError};

/// Simple TF-IDF based embedding provider for demonstration
#[derive(Debug)]
pub struct TfIdfEmbedding {
    dimension: usize,
    vocabulary: HashMap<String, usize>,
    idf_weights: Vec<f32>,
}
impl TfIdfEmbedding {
    /// Create a new TF-IDF embedding provider
    pub fn new(dimension: usize) -> Self {
        Self {
            dimension,
            vocabulary: HashMap::new(),
            idf_weights: vec![1.0; dimension],
        }
    }

    /// Build vocabulary from a corpus of texts
    pub fn build_vocabulary(&mut self, texts: &[&str]) {
        let mut word_counts: HashMap<String, usize> = HashMap::new();
        let mut doc_frequencies: HashMap<String, usize> = HashMap::new();

        for text in texts {
            let words = self.tokenize(text);
            let mut seen_words = std::collections::HashSet::new();

            for word in words {
                *word_counts.entry(word.clone()).or_insert(0) += 1;

                if seen_words.insert(word.clone()) {
                    *doc_frequencies.entry(word).or_insert(0) += 1;
                }
            }
        }

        // Compute a combined TF-IDF based score for vocabulary selection
        // score(word) = term_frequency(word) * idf(word)
        // This promotes salient (rare but informative) terms into the vocabulary
        let total_docs = texts.len() as f32;

        let mut scored_terms: Vec<(String, f32)> = doc_frequencies
            .iter()
            .map(|(word, &df)| {
                let tf_count = *word_counts.get(word).unwrap_or(&0) as f32;
                // Use natural log idf; guard df>=1
                let idf = if df > 0 {
                    (total_docs / (df as f32)).ln().max(0.0)
                } else {
                    0.0
                };
                (word.clone(), tf_count * idf)
            })
            .collect();

        // Sort by score descending, tie-break alphabetically for determinism
        scored_terms.sort_by(|a, b| {
            b.1.partial_cmp(&a.1)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| a.0.cmp(&b.0))
        });

        self.vocabulary.clear();
        self.idf_weights.clear();

        for (i, (word, _score)) in scored_terms.iter().take(self.dimension).enumerate() {
            self.vocabulary.insert(word.clone(), i);

            let df = *doc_frequencies.get(word).unwrap_or(&1) as f32;
            let idf = (total_docs / df).ln().max(0.0);
            self.idf_weights.push(idf);
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

    fn compute_tf(&self, text: &str) -> HashMap<String, f32> {
        let words = self.tokenize(text);
        let total_words = words.len() as f32;

        let mut word_counts: HashMap<String, usize> = HashMap::new();
        for word in words {
            *word_counts.entry(word).or_insert(0) += 1;
        }

        word_counts
            .into_iter()
            .map(|(word, count)| (word, count as f32 / total_words))
            .collect()
    }
}

impl EmbeddingProvider for TfIdfEmbedding {
    fn embed_batch(&self, texts: &[&str]) -> Result<Vec<Vec<f32>>> {
        texts.iter().map(|text| self.embed(text)).collect()
    }

    fn embed(&self, text: &str) -> Result<Vec<f32>> {
        let tf = self.compute_tf(text);
        let mut embedding = vec![0.0; self.dimension];

        let mut _matched_terms = 0;
        for (word, tf_value) in tf {
            if let Some(&idx) = self.vocabulary.get(&word) {
                if idx < self.dimension {
                    let idf = self.idf_weights.get(idx).unwrap_or(&1.0);
                    embedding[idx] = tf_value * idf;
                    _matched_terms += 1;
                }
            }
        }

        // Check if embedding is all zeros (fallback to hash-based embedding)
        let non_zero_count = embedding.iter().filter(|&&x| x != 0.0).count();
        if non_zero_count == 0 {
            return Ok(self.fallback_hash_embedding(text));
        }

        // Normalize the embedding
        let norm: f32 = embedding.iter().map(|x| x * x).sum::<f32>().sqrt();
        if norm > 0.0 {
            for value in &mut embedding {
                *value /= norm;
            }
        }

        Ok(embedding)
    }

    fn dimension(&self) -> usize {
        self.dimension
    }

    fn save_vocabulary_json(&self, path: &Path) -> Result<()> {
        TfIdfEmbedding::save_vocabulary_json(self, path)
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl TfIdfEmbedding {
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

    /// Save TF-IDF vocabulary/tokenizer JSON
    pub fn save_vocabulary_json<P: AsRef<std::path::Path>>(&self, path: P) -> Result<()> {
        let data = serde_json::json!({
            "type": "tfidf",
            "dimension": self.dimension,
            "vocabulary": self.vocabulary,
            "idf_weights": self.idf_weights,
        });
        let json = serde_json::to_string_pretty(&data).map_err(|e| {
            VectorizerError::Other(format!("Failed to serialize TF-IDF vocab: {}", e))
        })?;
        std::fs::write(path.as_ref(), json).map_err(|e| {
            VectorizerError::Other(format!(
                "Failed to write TF-IDF vocab {}: {}",
                path.as_ref().display(),
                e
            ))
        })?;
        Ok(())
    }

    /// Load TF-IDF vocabulary/tokenizer JSON
    pub fn load_vocabulary_json<P: AsRef<std::path::Path>>(&mut self, path: P) -> Result<()> {
        let content = std::fs::read_to_string(path.as_ref()).map_err(|e| {
            VectorizerError::Other(format!(
                "Failed to read TF-IDF vocab {}: {}",
                path.as_ref().display(),
                e
            ))
        })?;
        let v: serde_json::Value = serde_json::from_str(&content).map_err(|e| {
            VectorizerError::Other(format!(
                "Failed to parse TF-IDF vocab {}: {}",
                path.as_ref().display(),
                e
            ))
        })?;
        let t = v.get("type").and_then(|x| x.as_str()).unwrap_or("");
        if t != "tfidf" {
            return Err(VectorizerError::Other(format!(
                "Tokenizer type mismatch: expected tfidf, found {}",
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
        self.idf_weights = v
            .get("idf_weights")
            .and_then(|x| serde_json::from_value(x.clone()).ok())
            .unwrap_or_default();
        Ok(())
    }
}
