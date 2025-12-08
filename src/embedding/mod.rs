//! Embedding generation module for converting text to vectors

use std::collections::HashMap;
use std::fs;
use std::path::Path;

use tracing::warn;

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

    /// Cast to Any for downcasting (mutable)
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any;

    /// Cast to Any for downcasting (immutable)
    fn as_any(&self) -> &dyn std::any::Any;
}

/// Simple TF-IDF based embedding provider for demonstration
#[derive(Debug)]
pub struct TfIdfEmbedding {
    dimension: usize,
    vocabulary: HashMap<String, usize>,
    idf_weights: Vec<f32>,
}

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

#[derive(Debug)]
pub struct SvdEmbedding {
    /// The target reduced dimension
    reduced_dimension: usize,
    /// TF-IDF embedding for base transformation
    tfidf: TfIdfEmbedding,
    /// SVD transformation matrix (V^T truncated to reduced_dimension)
    transformation_matrix: Option<ndarray::Array2<f32>>,
    /// Whether SVD has been fitted
    fitted: bool,
}

#[derive(Debug)]
pub struct BertEmbedding {
    /// BERT model dimension (768 for BERT-base, 384 for BERT-small, etc.)
    dimension: usize,
    /// Maximum sequence length
    #[allow(dead_code)]
    max_seq_len: usize,
    /// Whether the model is loaded (placeholder for actual BERT integration)
    loaded: bool,
    /// Real BERT model (only when real-models feature is enabled)
    #[cfg(feature = "real-models")]
    real_model: Option<candle_models::RealBertEmbedding>,
}

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
}

impl SvdEmbedding {
    /// Create a new SVD embedding provider
    pub fn new(reduced_dimension: usize, vocabulary_size: usize) -> Self {
        Self {
            reduced_dimension,
            tfidf: TfIdfEmbedding::new(vocabulary_size),
            transformation_matrix: None,
            fitted: false,
        }
    }

    /// Fit a simple linear transformation (simplified SVD approximation)
    pub fn fit_svd(&mut self, texts: &[&str]) -> Result<()> {
        // First, build TF-IDF vocabulary
        self.tfidf.build_vocabulary(texts);

        // Create a simple transformation matrix using hash-based pseudo-random orthogonal vectors
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let vocab_size = self.tfidf.dimension;
        let mut transformation_matrix =
            ndarray::Array2::<f32>::zeros((self.reduced_dimension, vocab_size));

        // Generate transformation matrix using seeded random values
        let mut hasher = DefaultHasher::new();
        texts.hash(&mut hasher);
        let base_seed = hasher.finish();

        for i in 0..self.reduced_dimension {
            // Create a vector for this dimension
            let mut vector = Vec::with_capacity(vocab_size);

            for j in 0..vocab_size {
                // Generate pseudo-random value seeded by dimension and position
                let seed = base_seed.wrapping_add((i as u64 * 1000) + j as u64);
                let value = ((seed.wrapping_mul(1103515245) % 65536) as f32 / 32768.0) - 1.0;
                vector.push(value);
            }

            // Orthogonalize with previous vectors (simplified Gram-Schmidt)
            for k in 0..i {
                let prev_row = transformation_matrix.row(k);
                let dot_product: f32 = vector.iter().zip(prev_row.iter()).map(|(a, b)| a * b).sum();
                let norm_sq: f32 = prev_row.iter().map(|x| x * x).sum();

                if norm_sq > 0.0 {
                    let projection = dot_product / norm_sq;
                    for j in 0..vocab_size {
                        vector[j] -= projection * prev_row[j];
                    }
                }
            }

            // Normalize the vector
            let norm: f32 = vector.iter().map(|x| x * x).sum::<f32>().sqrt();
            if norm > 0.0 {
                for j in 0..vocab_size {
                    vector[j] /= norm;
                }
            }

            // Store in matrix
            for j in 0..vocab_size {
                transformation_matrix[[i, j]] = vector[j];
            }
        }

        self.transformation_matrix = Some(transformation_matrix);
        self.fitted = true;

        Ok(())
    }
}

impl EmbeddingProvider for SvdEmbedding {
    fn embed_batch(&self, texts: &[&str]) -> Result<Vec<Vec<f32>>> {
        texts.iter().map(|text| self.embed(text)).collect()
    }

    fn embed(&self, text: &str) -> Result<Vec<f32>> {
        if !self.fitted {
            return Err(VectorizerError::Other(
                "SVD embedding not fitted. Call fit_svd first.".to_string(),
            ));
        }

        // Get TF-IDF embedding
        let tfidf_embedding = self.tfidf.embed(text)?;

        // Apply transformation: result = tfidf_vector * V^T_reduced
        let vt = self.transformation_matrix.as_ref().unwrap();
        let mut result = vec![0.0f32; self.reduced_dimension];

        // Manual matrix multiplication for simplicity
        for i in 0..self.reduced_dimension {
            for j in 0..tfidf_embedding.len() {
                result[i] += tfidf_embedding[j] * vt[[i, j]];
            }
        }

        Ok(result)
    }

    fn dimension(&self) -> usize {
        self.reduced_dimension
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl BertEmbedding {
    /// Create a new BERT embedding provider
    /// dimension: 768 for BERT-base, 384 for BERT-small, etc.
    pub fn new(dimension: usize) -> Self {
        Self {
            dimension,
            max_seq_len: 512,
            loaded: false,
            #[cfg(feature = "real-models")]
            real_model: None,
        }
    }

    /// Load BERT model
    ///
    /// When "real-models" feature is enabled, this loads the actual BERT model from HuggingFace.
    /// Otherwise, it just marks the model as loaded (uses placeholder embeddings).
    pub fn load_model(&mut self) -> Result<()> {
        self.load_model_with_id("bert-base-uncased", false)
    }

    /// Load BERT model with custom model ID
    ///
    /// # Arguments
    /// * `model_id` - HuggingFace model ID (e.g., "bert-base-uncased")
    /// * `use_gpu` - Whether to use GPU acceleration if available
    pub fn load_model_with_id(&mut self, model_id: &str, use_gpu: bool) -> Result<()> {
        #[cfg(feature = "real-models")]
        {
            use tracing::info;
            info!("Loading real BERT model: {}", model_id);
            let model = candle_models::RealBertEmbedding::load_model(model_id, use_gpu)?;
            self.dimension = model.dimension();
            self.real_model = Some(model);
            self.loaded = true;
            Ok(())
        }

        #[cfg(not(feature = "real-models"))]
        {
            warn!(
                "real-models feature not enabled. Using placeholder embeddings for BERT. Model ID '{}' ignored.",
                model_id
            );
            let _ = use_gpu; // Suppress unused warning
            self.loaded = true;
            Ok(())
        }
    }

    /// Simple hash-based embedding simulation (placeholder)
    fn simple_hash_embedding(&self, text: &str) -> Vec<f32> {
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
}

impl EmbeddingProvider for BertEmbedding {
    fn embed_batch(&self, texts: &[&str]) -> Result<Vec<Vec<f32>>> {
        if !self.loaded {
            return Err(VectorizerError::Other(
                "BERT model not loaded. Call load_model first.".to_string(),
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
                "BERT model not loaded. Call load_model first.".to_string(),
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

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

/// Simple Bag-of-Words embedding provider
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

/// Character n-gram based embedding provider
pub struct CharNGramEmbedding {
    dimension: usize,
    n: usize,
    ngram_map: HashMap<String, usize>,
}

impl CharNGramEmbedding {
    /// Create a new character n-gram embedding provider
    pub fn new(dimension: usize, n: usize) -> Self {
        Self {
            dimension,
            n,
            ngram_map: HashMap::new(),
        }
    }

    /// Build n-gram vocabulary from texts
    pub fn build_vocabulary(&mut self, texts: &[&str]) {
        let mut ngram_counts: HashMap<String, usize> = HashMap::new();

        for text in texts {
            let ngrams = self.extract_ngrams(text);
            for ngram in ngrams {
                *ngram_counts.entry(ngram).or_insert(0) += 1;
            }
        }

        // Select top n-grams by frequency, with alphabetical tie-breaking for determinism
        let mut ngram_freq: Vec<(String, usize)> = ngram_counts.into_iter().collect();
        ngram_freq.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));

        self.ngram_map.clear();
        for (i, (ngram, _)) in ngram_freq.iter().take(self.dimension).enumerate() {
            self.ngram_map.insert(ngram.clone(), i);
        }
    }

    fn extract_ngrams(&self, text: &str) -> Vec<String> {
        let text = text.to_lowercase();
        let chars: Vec<char> = text.chars().collect();

        if chars.len() < self.n {
            return vec![text];
        }

        let mut ngrams = Vec::new();
        for i in 0..=(chars.len() - self.n) {
            let ngram: String = chars[i..i + self.n].iter().collect();
            ngrams.push(ngram);
        }

        ngrams
    }
}

impl EmbeddingProvider for CharNGramEmbedding {
    fn embed(&self, text: &str) -> Result<Vec<f32>> {
        let ngrams = self.extract_ngrams(text);
        let mut embedding = vec![0.0; self.dimension];

        for ngram in ngrams {
            if let Some(&idx) = self.ngram_map.get(&ngram) {
                embedding[idx] += 1.0;
            }
        }

        // Check if embedding is all zeros
        let non_zero_count = embedding.iter().filter(|&&x| x != 0.0).count();
        if non_zero_count == 0 {
            warn!(
                "WARNING: CharNGramEmbedding produced all-zero embedding for '{}'",
                text
            );
            warn!("N-gram map size: {}", self.ngram_map.len());

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

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl CharNGramEmbedding {
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

    /// Save CharNGram tokenizer JSON
    pub fn save_vocabulary_json<P: AsRef<std::path::Path>>(&self, path: P) -> Result<()> {
        let data = serde_json::json!({
            "type": "charngram",
            "dimension": self.dimension,
            "n": self.n,
            "ngram_map": self.ngram_map,
        });
        let json = serde_json::to_string_pretty(&data).map_err(|e| {
            VectorizerError::Other(format!("Failed to serialize CharNGram vocab: {}", e))
        })?;
        std::fs::write(path.as_ref(), json).map_err(|e| {
            VectorizerError::Other(format!(
                "Failed to write CharNGram vocab {}: {}",
                path.as_ref().display(),
                e
            ))
        })?;
        Ok(())
    }

    /// Load CharNGram tokenizer JSON
    pub fn load_vocabulary_json<P: AsRef<std::path::Path>>(&mut self, path: P) -> Result<()> {
        let content = std::fs::read_to_string(path.as_ref()).map_err(|e| {
            VectorizerError::Other(format!(
                "Failed to read CharNGram vocab {}: {}",
                path.as_ref().display(),
                e
            ))
        })?;
        let v: serde_json::Value = serde_json::from_str(&content).map_err(|e| {
            VectorizerError::Other(format!(
                "Failed to parse CharNGram vocab {}: {}",
                path.as_ref().display(),
                e
            ))
        })?;
        let t = v.get("type").and_then(|x| x.as_str()).unwrap_or("");
        if t != "charngram" {
            return Err(VectorizerError::Other(format!(
                "Tokenizer type mismatch: expected charngram, found {}",
                t
            )));
        }
        self.dimension = v
            .get("dimension")
            .and_then(|x| x.as_u64())
            .unwrap_or(self.dimension as u64) as usize;
        self.n = v.get("n").and_then(|x| x.as_u64()).unwrap_or(self.n as u64) as usize;
        self.ngram_map = v
            .get("ngram_map")
            .and_then(|x| serde_json::from_value(x.clone()).ok())
            .unwrap_or_default();
        Ok(())
    }
}

/// Manager for embedding providers
pub struct EmbeddingManager {
    providers: HashMap<String, Box<dyn EmbeddingProvider>>,
    default_provider: Option<String>,
}

impl EmbeddingManager {
    /// Create a new embedding manager
    pub fn new() -> Self {
        Self {
            providers: HashMap::new(),
            default_provider: None,
        }
    }

    /// Register an embedding provider
    pub fn register_provider(&mut self, name: String, provider: Box<dyn EmbeddingProvider>) {
        if self.default_provider.is_none() {
            self.default_provider = Some(name.clone());
        }
        self.providers.insert(name, provider);
    }

    /// Set the default provider
    pub fn set_default_provider(&mut self, name: &str) -> Result<()> {
        if self.providers.contains_key(name) {
            self.default_provider = Some(name.to_string());
            Ok(())
        } else {
            Err(VectorizerError::Other(format!(
                "Provider '{}' not found",
                name
            )))
        }
    }

    /// Get a provider by name
    pub fn get_provider(&self, name: &str) -> Result<&dyn EmbeddingProvider> {
        self.providers
            .get(name)
            .map(|p| p.as_ref())
            .ok_or_else(|| VectorizerError::Other(format!("Provider '{}' not found", name)))
    }

    /// Get a mutable provider by name
    pub fn get_provider_mut(&mut self, name: &str) -> Option<&mut Box<dyn EmbeddingProvider>> {
        self.providers.get_mut(name)
    }

    /// Get the default provider
    pub fn get_default_provider(&self) -> Result<&dyn EmbeddingProvider> {
        let provider_name = self
            .default_provider
            .as_ref()
            .ok_or_else(|| VectorizerError::Other("No default provider set".to_string()))?;

        self.get_provider(provider_name)
    }

    /// Get the default provider name
    pub fn get_default_provider_name(&self) -> Option<&str> {
        self.default_provider.as_deref()
    }

    /// Embed text using the default provider
    pub fn embed(&self, text: &str) -> Result<Vec<f32>> {
        self.get_default_provider()?.embed(text)
    }

    /// Embed batch of texts using the default provider
    pub fn embed_batch(&self, texts: &[&str]) -> Result<Vec<Vec<f32>>> {
        self.get_default_provider()?.embed_batch(texts)
    }

    /// Embed text using a specific provider by name
    pub fn embed_with_provider(&self, provider_name: &str, text: &str) -> Result<Vec<f32>> {
        let provider = self.get_provider(provider_name)?;
        provider.embed(text)
    }

    /// Embed batch of texts using a specific provider by name
    pub fn embed_batch_with_provider(
        &self,
        texts: &[&str],
        provider_name: &str,
    ) -> Result<Vec<Vec<f32>>> {
        self.get_provider(provider_name)?.embed_batch(texts)
    }

    /// Get the dimension of a specific provider
    pub fn get_provider_dimension(&self, provider_name: &str) -> Result<usize> {
        Ok(self.get_provider(provider_name)?.dimension())
    }

    /// List all available provider names
    pub fn list_providers(&self) -> Vec<String> {
        self.providers.keys().cloned().collect()
    }

    /// Check if a provider exists
    pub fn has_provider(&self, provider_name: &str) -> bool {
        self.providers.contains_key(provider_name)
    }

    /// Save vocabulary for a specific provider
    pub fn save_vocabulary_json<P: AsRef<Path>>(&self, provider_name: &str, path: P) -> Result<()> {
        let provider = self.get_provider(provider_name)?;

        // Try to downcast to specific embedding types that have save_vocabulary_json
        if let Some(bm25) = provider.as_any().downcast_ref::<Bm25Embedding>() {
            bm25.save_vocabulary_json(path)
        } else if let Some(tfidf) = provider.as_any().downcast_ref::<TfIdfEmbedding>() {
            tfidf.save_vocabulary_json(path)
        } else if let Some(char_ngram) = provider.as_any().downcast_ref::<CharNGramEmbedding>() {
            char_ngram.save_vocabulary_json(path)
        } else if let Some(bow) = provider.as_any().downcast_ref::<BagOfWordsEmbedding>() {
            bow.save_vocabulary_json(path)
        } else {
            Err(VectorizerError::Other(format!(
                "Provider '{}' does not support vocabulary saving",
                provider_name
            )))
        }
    }
}

impl Default for EmbeddingManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tfidf_embedding() {
        let mut tfidf = TfIdfEmbedding::new(10);

        let corpus = vec![
            "machine learning is great",
            "deep learning is better",
            "vector databases store embeddings",
            "embeddings represent text as vectors",
        ];

        tfidf.build_vocabulary(&corpus);

        let embedding = tfidf.embed("machine learning vectors").unwrap();
        assert_eq!(embedding.len(), 10);

        // Check normalization
        let norm: f32 = embedding.iter().map(|x| x * x).sum::<f32>().sqrt();
        assert!((norm - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_bag_of_words() {
        let mut bow = BagOfWordsEmbedding::new(5);

        let corpus = vec!["hello world", "hello machine learning", "world of vectors"];

        bow.build_vocabulary(&corpus);

        let embedding = bow.embed("hello world").unwrap();
        assert_eq!(embedding.len(), 5);

        // Should have non-zero values for "hello" and "world"
        assert!(embedding.iter().any(|&x| x > 0.0));
    }

    #[test]
    fn test_char_ngram() {
        let mut ngram = CharNGramEmbedding::new(10, 3);

        let corpus = vec!["hello", "world", "hello world"];

        ngram.build_vocabulary(&corpus);

        let embedding = ngram.embed("hello").unwrap();
        assert_eq!(embedding.len(), 10);

        // Check normalization
        let norm: f32 = embedding.iter().map(|x| x * x).sum::<f32>().sqrt();
        assert!((norm - 1.0).abs() < 1e-6 || norm == 0.0);
    }

    #[test]
    fn test_embedding_manager() {
        let mut manager = EmbeddingManager::new();

        let tfidf = Box::new(TfIdfEmbedding::new(10));
        let bow = Box::new(BagOfWordsEmbedding::new(5));

        manager.register_provider("tfidf".to_string(), tfidf);
        manager.register_provider("bow".to_string(), bow);

        manager.set_default_provider("tfidf").unwrap();

        let provider = manager.get_provider("tfidf").unwrap();
        assert_eq!(provider.dimension(), 10);

        let default_provider = manager.get_default_provider().unwrap();
        assert_eq!(default_provider.dimension(), 10);
    }
}

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

// TfIdfEmbedding is already public in this module
