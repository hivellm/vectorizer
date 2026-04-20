//! BM25 embedding provider (sparse, probabilistic).
//!
//! Extracted from the monolithic `embedding/mod.rs` by
//! phase4_split-interleaved-embedding-providers. Note: a different
//! `BM25Provider` type lives in `src/embedding/bm25.rs` — they are not
//! the same implementation.

// Internal data-layout file: public fields are self-documenting; the
// blanket allow keeps `cargo doc -W missing-docs` clean without padding
// every field with a tautological `///` comment. See
// phase4_enforce-public-api-docs.
#![allow(missing_docs)]

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

        // Generate a pseudo-random but deterministic embedding. Previously
        // this computed `seed.wrapping_mul(1103515245).wrapping_add(12345
        // + i as u64) % 65536`, which kept `seed * 1103515245` fixed
        // across all dimensions and only added `12345 + i`. After `% 65536`
        // that produced 512 consecutive integers differing by 1, which
        // L2-normalize to ~1/sqrt(dimension) uniformly — the exact
        // pathological behavior probe 2.1 observed (~0.0436 across every
        // component). Iterate the LCG state per dimension so each
        // component is an independent step of the generator.
        let mut embedding = Vec::with_capacity(self.dimension);
        let mut state = seed;
        for _ in 0..self.dimension {
            state = state
                .wrapping_mul(6364136223846793005)
                .wrapping_add(1442695040888963407);
            let value = ((state >> 33) & 0xFFFF) as f32;
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

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used)]

    use super::*;

    fn cosine(a: &[f32], b: &[f32]) -> f32 {
        let dot: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
        let na = a.iter().map(|x| x * x).sum::<f32>().sqrt();
        let nb = b.iter().map(|x| x * x).sum::<f32>().sqrt();
        dot / (na * nb)
    }

    /// Regression test for phase8_investigate-uniform-embeddings (finding F4).
    ///
    /// The pre-fix LCG in `fallback_hash_embedding` produced
    /// `~1/sqrt(dim)` on every component for every text, which collapsed
    /// all embeddings to the same uniform vector and made pairwise
    /// cosine similarity ≈ 1.0 across different inputs. This test locks
    /// the post-fix invariant: distinct inputs produce distinct vectors
    /// with pairwise cosine similarity well below 0.95.
    #[test]
    fn fallback_embedding_is_not_uniform_across_distinct_inputs() {
        let bm25 = Bm25Embedding::new(512);
        let texts = [
            "completely different first sentence about oranges",
            "another totally unrelated topic about submarines",
            "third short idea discussing ancient stone tablets",
            "fourth concept centered on jazz improvisation",
            "fifth note on the winter migration of arctic terns",
        ];

        let embeddings: Vec<Vec<f32>> = texts
            .iter()
            .map(|t| bm25.fallback_hash_embedding(t))
            .collect();

        // Each vector should cover both signs (pre-fix produced a
        // monotonic linear sequence that after L2-normalize landed on
        // a constant).
        for (i, v) in embeddings.iter().enumerate() {
            let has_positive = v.iter().any(|&x| x > 0.01);
            let has_negative = v.iter().any(|&x| x < -0.01);
            assert!(
                has_positive && has_negative,
                "embedding {i} lacks sign variation (pos={has_positive}, neg={has_negative})"
            );
        }

        // Pairwise cosine similarity must stay well below 1.0 — the
        // pre-fix bug made every pair effectively identical.
        for i in 0..embeddings.len() {
            for j in (i + 1)..embeddings.len() {
                let s = cosine(&embeddings[i], &embeddings[j]);
                assert!(
                    s.abs() < 0.95,
                    "pairwise similarity too high for {i}-{j}: {s}"
                );
            }
        }

        // A single vector's components must not be uniform. Compute
        // the std dev — pre-fix fallback had std dev ≈ 0, post-fix
        // should land well above 0.01.
        for (i, v) in embeddings.iter().enumerate() {
            let mean = v.iter().sum::<f32>() / v.len() as f32;
            let var = v.iter().map(|x| (x - mean).powi(2)).sum::<f32>() / v.len() as f32;
            let std = var.sqrt();
            assert!(
                std > 0.01,
                "embedding {i} std dev {std} looks uniform (pre-fix would be ~0)"
            );
        }
    }
}
