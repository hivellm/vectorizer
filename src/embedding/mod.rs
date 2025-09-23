//! Embedding generation module for converting text to vectors

use crate::error::{Result, VectorizerError};
use std::collections::HashMap;

/// Trait for embedding providers
pub trait EmbeddingProvider: Send + Sync {
    /// Generate embeddings for a batch of texts
    fn embed_batch(&self, texts: &[&str]) -> Result<Vec<Vec<f32>>>;
    
    /// Generate embedding for a single text
    fn embed(&self, text: &str) -> Result<Vec<f32>> {
        let results = self.embed_batch(&[text])?;
        results.into_iter().next()
            .ok_or_else(|| VectorizerError::Other("Failed to generate embedding".to_string()))
    }
    
    /// Get the dimension of embeddings produced by this provider
    fn dimension(&self) -> usize;
}

/// Simple TF-IDF based embedding provider for demonstration
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
        
        // Select top words by frequency
        let mut word_freq: Vec<(String, usize)> = word_counts.into_iter().collect();
        word_freq.sort_by(|a, b| b.1.cmp(&a.1));
        
        self.vocabulary.clear();
        self.idf_weights.clear();
        
        let total_docs = texts.len() as f32;
        
        for (i, (word, _)) in word_freq.iter().take(self.dimension).enumerate() {
            self.vocabulary.insert(word.clone(), i);
            
            let doc_freq = doc_frequencies.get(word).unwrap_or(&1);
            let idf = (total_docs / (*doc_freq as f32)).ln();
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
        
        word_counts.into_iter()
            .map(|(word, count)| (word, count as f32 / total_words))
            .collect()
    }
}

impl EmbeddingProvider for TfIdfEmbedding {
    fn embed_batch(&self, texts: &[&str]) -> Result<Vec<Vec<f32>>> {
        texts.iter()
            .map(|text| self.embed(text))
            .collect()
    }
    
    fn embed(&self, text: &str) -> Result<Vec<f32>> {
        let tf = self.compute_tf(text);
        let mut embedding = vec![0.0; self.dimension];
        
        for (word, tf_value) in tf {
            if let Some(&idx) = self.vocabulary.get(&word) {
                if idx < self.dimension {
                    let idf = self.idf_weights.get(idx).unwrap_or(&1.0);
                    embedding[idx] = tf_value * idf;
                }
            }
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
        
        // Select top words by frequency
        let mut word_freq: Vec<(String, usize)> = word_counts.into_iter().collect();
        word_freq.sort_by(|a, b| b.1.cmp(&a.1));
        
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
        texts.iter()
            .map(|text| self.embed(text))
            .collect()
    }
    
    fn dimension(&self) -> usize {
        self.dimension
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
        
        // Select top n-grams
        let mut ngram_freq: Vec<(String, usize)> = ngram_counts.into_iter().collect();
        ngram_freq.sort_by(|a, b| b.1.cmp(&a.1));
        
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
        texts.iter()
            .map(|text| self.embed(text))
            .collect()
    }
    
    fn dimension(&self) -> usize {
        self.dimension
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
            Err(VectorizerError::Other(format!("Provider '{}' not found", name)))
        }
    }
    
    /// Get a provider by name
    pub fn get_provider(&self, name: &str) -> Result<&dyn EmbeddingProvider> {
        self.providers
            .get(name)
            .map(|p| p.as_ref())
            .ok_or_else(|| VectorizerError::Other(format!("Provider '{}' not found", name)))
    }
    
    /// Get the default provider
    pub fn get_default_provider(&self) -> Result<&dyn EmbeddingProvider> {
        let provider_name = self.default_provider
            .as_ref()
            .ok_or_else(|| VectorizerError::Other("No default provider set".to_string()))?;
        
        self.get_provider(provider_name)
    }
    
    /// Embed text using the default provider
    pub fn embed(&self, text: &str) -> Result<Vec<f32>> {
        self.get_default_provider()?.embed(text)
    }
    
    /// Embed batch of texts using the default provider
    pub fn embed_batch(&self, texts: &[&str]) -> Result<Vec<Vec<f32>>> {
        self.get_default_provider()?.embed_batch(texts)
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
        
        let corpus = vec![
            "hello world",
            "hello machine learning",
            "world of vectors",
        ];
        
        bow.build_vocabulary(&corpus);
        
        let embedding = bow.embed("hello world").unwrap();
        assert_eq!(embedding.len(), 5);
        
        // Should have non-zero values for "hello" and "world"
        assert!(embedding.iter().any(|&x| x > 0.0));
    }
    
    #[test]
    fn test_char_ngram() {
        let mut ngram = CharNGramEmbedding::new(10, 3);
        
        let corpus = vec![
            "hello",
            "world",
            "hello world",
        ];
        
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
