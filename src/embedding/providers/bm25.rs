//! BM25 embedding provider (sparse, probabilistic).
//!
//! Extracted from the monolithic `embedding/mod.rs` by
//! phase4_split-interleaved-embedding-providers. Note: a different
//! `BM25Provider` type lives in `src/embedding/bm25.rs` — they are not
//! the same implementation.

use std::collections::HashMap;
use std::fs;
use std::path::Path;

use tracing::warn;

use crate::embedding::EmbeddingProvider;
use crate::error::{Result, VectorizerError};

#[derive(Debug)]
pub struct Bm25Embedding {
    dimension: usize,
    vocabulary: HashMap<String, usize>,
    doc_freq: HashMap<String, usize>, // Document frequency for each term
    doc_lengths: Vec<usize>,          // Length of each document
    avg_doc_length: f32,              // Average document length
    total_docs: usize,                // Total number of documents
    k1: f32,                          // BM25 parameter (typically 1.5)
    b: f32,                           // BM25 parameter (typically 0.75)
}
impl Bm25Embedding {
    /// Create a new BM25 embedding provider
    pub fn new(dimension: usize) -> Self {
        Self {
            dimension,
            vocabulary: HashMap::new(),
            doc_freq: HashMap::new(),
            doc_lengths: Vec::new(),
            avg_doc_length: 0.0,
            total_docs: 0,
            k1: 1.5, // Standard BM25 k1 parameter
            b: 0.75, // Standard BM25 b parameter
        }
    }

    /// Get the vocabulary size
    pub fn vocabulary_size(&self) -> usize {
        self.vocabulary.len()
    }

    /// Extract vocabulary data for restoration
    pub fn extract_vocabulary_data(
        &self,
    ) -> (
        HashMap<String, usize>,
        HashMap<String, usize>,
        Vec<usize>,
        f32,
        usize,
    ) {
        (
            self.vocabulary.clone(),
            self.doc_freq.clone(),
            self.doc_lengths.clone(),
            self.avg_doc_length,
            self.total_docs,
        )
    }

    /// Restore vocabulary data
    pub fn restore_vocabulary_data(
        &mut self,
        vocabulary: HashMap<String, usize>,
        doc_freq: HashMap<String, usize>,
        doc_lengths: Vec<usize>,
        avg_doc_length: f32,
        total_docs: usize,
    ) {
        self.vocabulary = vocabulary;
        self.doc_freq = doc_freq;
        self.doc_lengths = doc_lengths;
        self.avg_doc_length = avg_doc_length;
        self.total_docs = total_docs;
    }

    /// Save vocabulary to a JSON file (tokenizer)
    pub fn save_vocabulary_json<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let path_ref = path.as_ref();
        let data = serde_json::json!({
            "type": "bm25",
            "dimension": self.dimension,
            "vocabulary": self.vocabulary,
            "doc_freq": self.doc_freq,
            "doc_lengths": self.doc_lengths,
            "avg_doc_length": self.avg_doc_length,
            "total_docs": self.total_docs,
        });
        let json = serde_json::to_string_pretty(&data).map_err(|e| {
            VectorizerError::Other(format!("Failed to serialize vocabulary: {}", e))
        })?;
        fs::write(path_ref, json).map_err(|e| {
            VectorizerError::Other(format!(
                "Failed to write vocabulary file {}: {}",
                path_ref.display(),
                e
            ))
        })?;
        Ok(())
    }

    /// Load vocabulary from a JSON file (tokenizer)
    pub fn load_vocabulary_json<P: AsRef<Path>>(&mut self, path: P) -> Result<()> {
        let path_ref = path.as_ref();
        let content = fs::read_to_string(path_ref).map_err(|e| {
            VectorizerError::Other(format!(
                "Failed to read vocabulary file {}: {}",
                path_ref.display(),
                e
            ))
        })?;
        let v: serde_json::Value = serde_json::from_str(&content).map_err(|e| {
            VectorizerError::Other(format!(
                "Failed to parse vocabulary JSON {}: {}",
                path_ref.display(),
                e
            ))
        })?;

        // Validate type
        if let Some(t) = v.get("type").and_then(|x| x.as_str()) {
            if t != "bm25" {
                return Err(VectorizerError::Other(format!(
                    "Tokenizer type mismatch: expected bm25, found {}",
                    t
                )));
            }
        }

        // Extract fields
        let dimension = v
            .get("dimension")
            .and_then(|x| x.as_u64())
            .unwrap_or(self.dimension as u64) as usize;
        let vocabulary: HashMap<String, usize> = v
            .get("vocabulary")
            .and_then(|x| serde_json::from_value(x.clone()).ok())
            .ok_or_else(|| {
                VectorizerError::Other("Missing or invalid 'vocabulary' field".to_string())
            })?;
        let doc_freq: HashMap<String, usize> = v
            .get("doc_freq")
            .and_then(|x| serde_json::from_value(x.clone()).ok())
            .ok_or_else(|| {
                VectorizerError::Other("Missing or invalid 'doc_freq' field".to_string())
            })?;
        let doc_lengths: Vec<usize> = v
            .get("doc_lengths")
            .and_then(|x| serde_json::from_value(x.clone()).ok())
            .unwrap_or_default();
        let avg_doc_length: f32 = v
            .get("avg_doc_length")
            .and_then(|x| x.as_f64())
            .unwrap_or(0.0) as f32;
        let total_docs: usize = v.get("total_docs").and_then(|x| x.as_u64()).unwrap_or(0) as usize;

        self.dimension = dimension;
        self.vocabulary = vocabulary;
        self.doc_freq = doc_freq;
        self.doc_lengths = doc_lengths;
        self.avg_doc_length = avg_doc_length;
        self.total_docs = total_docs;
        Ok(())
    }

    /// Build vocabulary and document statistics from a corpus of texts
    pub fn build_vocabulary(&mut self, texts: &[String]) {
        let mut word_counts: HashMap<String, usize> = HashMap::new();
        let mut doc_frequencies: HashMap<String, usize> = HashMap::new();

        // Process each document
        for text in texts {
            let tokens = self.tokenize(text);
            let doc_length = tokens.len();
            self.doc_lengths.push(doc_length);

            let mut unique_terms = std::collections::HashSet::new();
            for token in &tokens {
                *word_counts.entry(token.clone()).or_insert(0) += 1;
                unique_terms.insert(token.clone());
            }

            // Update document frequencies
            for term in unique_terms {
                *doc_frequencies.entry(term).or_insert(0) += 1;
            }
        }

        self.total_docs = texts.len();
        self.avg_doc_length =
            self.doc_lengths.iter().sum::<usize>() as f32 / self.total_docs as f32;

        // Build vocabulary and sort by frequency for deterministic results
        let mut word_freq: Vec<(String, usize)> = word_counts.into_iter().collect();
        word_freq.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));

        // Take top N terms based on dimension
        for (i, (word, _)) in word_freq.into_iter().enumerate().take(self.dimension) {
            let df = *doc_frequencies.get(&word).unwrap_or(&0);
            self.vocabulary.insert(word.clone(), i);
            self.doc_freq.insert(word, df);
        }

        // Vocabulary construction completed silently
    }

    /// Tokenize text into words (simple whitespace splitting)
    fn tokenize(&self, text: &str) -> Vec<String> {
        text.to_lowercase()
            .split_whitespace()
            .map(|s| s.trim_matches(|c: char| !c.is_alphanumeric()).to_string())
            .filter(|s| !s.is_empty())
            .collect()
    }

    /// Calculate BM25 score for a term in a document
    fn bm25_score(&self, term_freq: usize, doc_length: usize, doc_freq: usize) -> f32 {
        if doc_freq == 0 {
            return 0.0;
        }

        let idf =
            ((self.total_docs as f32 - doc_freq as f32 + 0.5) / (doc_freq as f32 + 0.5) + 1.0).ln();

        let tf = term_freq as f32 * (self.k1 + 1.0)
            / (term_freq as f32
                + self.k1 * (1.0 - self.b + self.b * doc_length as f32 / self.avg_doc_length));

        idf * tf
    }

    /// Fallback hash-based embedding when vocabulary is empty or no matches found
    fn fallback_hash_embedding(&self, text: &str) -> Vec<f32> {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        // Use text hash as seed, or use dimension as seed if text is empty
        let seed = if text.is_empty() {
            // Use dimension as seed for empty text to ensure non-zero
            self.dimension as u64
        } else {
            let mut hasher = DefaultHasher::new();
            text.hash(&mut hasher);
            hasher.finish()
        };

        // Generate pseudo-random but deterministic embedding
        let mut embedding = Vec::with_capacity(self.dimension);
        for i in 0..self.dimension {
            // Simple LCG-like generator seeded by text hash
            let value =
                ((seed.wrapping_mul(1103515245).wrapping_add(12345 + i as u64)) % 65536) as f32;
            embedding.push((value / 32768.0) - 1.0); // Normalize to [-1, 1]
        }

        // L2 normalize - ensure norm is never zero
        let norm = embedding.iter().map(|x| x * x).sum::<f32>().sqrt();
        if norm > 0.0 {
            for value in &mut embedding {
                *value /= norm;
            }
        } else {
            // If somehow norm is zero (shouldn't happen), set first element to 1.0
            warn!("BM25 fallback embedding had zero norm, setting first element to 1.0");
            if !embedding.is_empty() {
                embedding[0] = 1.0;
            }
        }

        // Final check: ensure at least one non-zero value
        let has_non_zero = embedding.iter().any(|&v| v != 0.0);
        if !has_non_zero {
            warn!("BM25 fallback embedding still all zeros, forcing non-zero values");
            for (i, v) in embedding.iter_mut().enumerate() {
                *v = (i as f32 / self.dimension as f32) * 0.1; // Small non-zero values
            }
            // Normalize again
            let norm = embedding.iter().map(|x| x * x).sum::<f32>().sqrt();
            if norm > 0.0 {
                for value in &mut embedding {
                    *value /= norm;
                }
            }
        }

        embedding
    }
}
impl EmbeddingProvider for Bm25Embedding {
    fn embed_batch(&self, texts: &[&str]) -> Result<Vec<Vec<f32>>> {
        texts.iter().map(|text| self.embed(text)).collect()
    }

    fn embed(&self, text: &str) -> Result<Vec<f32>> {
        let tokens = self.tokenize(text);
        let doc_length = tokens.len();

        // Debug: Log tokenization for queries (only in trace level)
        if text.len() < 100 { // Only log short queries to avoid spam
            //trace!("Query '{}' -> tokens: {:?}", text, tokens);
        }

        // Count term frequencies in this document
        let mut term_freq: HashMap<String, usize> = HashMap::new();
        for token in tokens {
            *term_freq.entry(token).or_insert(0) += 1;
        }

        // If vocabulary is empty, use fallback immediately
        if self.vocabulary.is_empty() {
            // Safely truncate text for logging (handle Unicode properly)
            let preview = if text.len() > 100 {
                // Find the last char boundary before 100 bytes
                let mut boundary = 100;
                while boundary > 0 && !text.is_char_boundary(boundary) {
                    boundary -= 1;
                }
                &text[..boundary]
            } else {
                text
            };
            warn!(
                "BM25 vocabulary is empty for text '{}' (first {} chars), using hash-based fallback",
                preview,
                text.chars().count().min(100)
            );
            let fallback = self.fallback_hash_embedding(text);
            // Normalize fallback
            let norm: f32 = fallback.iter().map(|x| x * x).sum::<f32>().sqrt();
            if norm > 0.0 {
                let mut normalized = fallback;
                for v in &mut normalized {
                    *v /= norm;
                }
                return Ok(normalized);
            }
            return Ok(fallback);
        }

        // Calculate BM25 scores for each term in vocabulary
        let mut embedding = vec![0.0; self.dimension];
        let mut _matched_terms = 0;
        for (term, &vocab_index) in &self.vocabulary {
            if vocab_index >= self.dimension {
                continue;
            }

            let tf = *term_freq.get(term).unwrap_or(&0);
            let df = *self.doc_freq.get(term).unwrap_or(&0);

            if tf > 0 {
                embedding[vocab_index] = self.bm25_score(tf, doc_length, df);
                _matched_terms += 1;
            }
        }

        // If embedding is all zeros (no vocab matches), build deterministic feature-hashed embedding from tokens
        let non_zero_count = embedding.iter().filter(|&&x| x != 0.0).count();
        if non_zero_count == 0 {
            // Safely truncate text for logging (handle Unicode properly)
            let preview = if text.len() > 100 {
                // Find the last char boundary before 100 bytes
                let mut boundary = 100;
                while boundary > 0 && !text.is_char_boundary(boundary) {
                    boundary -= 1;
                }
                &text[..boundary]
            } else {
                text
            };
            warn!(
                "BM25 produced all-zero embedding for text '{}' (vocab size: {}), using hash-based fallback",
                preview,
                self.vocabulary.len()
            );
            use std::collections::hash_map::DefaultHasher;
            use std::hash::{Hash, Hasher};

            // Feature-hashing for OOV tokens to guarantee non-zero vector while preserving input dependence
            let mut hashed_embedding = vec![0.0f32; self.dimension];
            for (token, tf) in term_freq {
                let mut hasher = DefaultHasher::new();
                token.hash(&mut hasher);
                let idx = (hasher.finish() as usize) % self.dimension;
                // Use TF with a mild scaling to avoid domination
                hashed_embedding[idx] += tf as f32;
            }

            // If still zero (e.g., empty text), fall back to text-hash embedding
            let nz = hashed_embedding.iter().any(|&v| v != 0.0);
            let mut final_embedding = if nz {
                hashed_embedding
            } else {
                self.fallback_hash_embedding(text)
            };

            // Normalize
            let norm: f32 = final_embedding.iter().map(|x| x * x).sum::<f32>().sqrt();
            if norm > 0.0 {
                for v in &mut final_embedding {
                    *v /= norm;
                }
            }

            // Downgrade severity: this is expected for OOV queries
            return Ok(final_embedding);
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
        Bm25Embedding::save_vocabulary_json(self, path)
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}
