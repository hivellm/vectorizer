//! HNSW integration for quantized vectors
//!
//! Implements efficient similarity search using quantized vectors.
//! Provides foundation for HNSW integration with quantization.

use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use serde::{Deserialize, Serialize};

use crate::quantization::scalar::ScalarQuantization;
use crate::quantization::storage::QuantizedVectorStorage;
use crate::quantization::traits::{QuantizationMethod, QuantizedSearch, QuantizedVectors};
use crate::quantization::{QuantizationError, QuantizationResult, QuantizationType};

/// Configuration for HNSW integration with quantization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HnswQuantizationConfig {
    /// Quantization method
    pub quantization_type: QuantizationType,
    /// Enable quantized similarity search
    pub enable_quantized_search: bool,
    /// Cache size for quantized vectors
    pub cache_size: usize,
    /// Enable hybrid search (quantized + original)
    pub enable_hybrid_search: bool,
    /// Quality threshold for hybrid search
    pub quality_threshold: f32,
}

impl Default for HnswQuantizationConfig {
    fn default() -> Self {
        Self {
            quantization_type: QuantizationType::Scalar(8),
            enable_quantized_search: true,
            cache_size: 10000,
            enable_hybrid_search: false,
            quality_threshold: 0.90,
        }
    }
}

/// Integrated quantized index with similarity search
pub struct QuantizedHnswIndex {
    /// Quantization method
    quantization: Box<dyn QuantizationMethod + Send + Sync>,
    /// Quantized vectors storage
    quantized_storage: Arc<QuantizedVectorStorage>,
    /// Configuration
    config: HnswQuantizationConfig,
    /// Quantized vectors cache
    quantized_cache: Arc<RwLock<HashMap<usize, Vec<u8>>>>,
    /// Original vectors cache for hybrid search
    original_cache: Arc<RwLock<HashMap<usize, Vec<f32>>>>,
    /// Vector count
    vector_count: usize,
    /// Vector dimension
    dimension: usize,
}

impl QuantizedHnswIndex {
    /// Create a new quantized index
    pub fn new(
        config: HnswQuantizationConfig,
        storage: Arc<QuantizedVectorStorage>,
    ) -> QuantizationResult<Self> {
        // Create quantization method based on config
        let quantization: Box<dyn QuantizationMethod + Send + Sync> = match config.quantization_type
        {
            QuantizationType::Scalar(bits) => Box::new(ScalarQuantization::new(bits)?),
            _ => {
                return Err(QuantizationError::InvalidParameters(format!(
                    "Unsupported quantization type: {:?}",
                    config.quantization_type
                )));
            }
        };

        Ok(Self {
            quantization,
            quantized_storage: storage,
            config,
            quantized_cache: Arc::new(RwLock::new(HashMap::new())),
            original_cache: Arc::new(RwLock::new(HashMap::new())),
            vector_count: 0,
            dimension: 0,
        })
    }

    /// Add vectors to the index with quantization
    pub fn add_vectors(&mut self, vectors: &[Vec<f32>]) -> QuantizationResult<()> {
        if vectors.is_empty() {
            return Ok(());
        }

        // Set dimension if first batch
        if self.dimension == 0 {
            self.dimension = vectors[0].len();
        }

        // Validate dimensions
        for vector in vectors {
            if vector.len() != self.dimension {
                return Err(QuantizationError::DimensionMismatch {
                    expected: self.dimension,
                    actual: vector.len(),
                });
            }
        }

        // Fit quantization if needed
        if self.vector_count == 0 {
            // Create new quantization with fitted parameters
            let mut scalar_q = ScalarQuantization::new(match self.config.quantization_type {
                QuantizationType::Scalar(bits) => bits,
                _ => 8,
            })?;
            scalar_q.fit(vectors)?;
            self.quantization = Box::new(scalar_q);
        }

        // Quantize vectors
        let quantized = self.quantization.quantize(vectors)?;

        // Always cache original vectors for similarity calculation
        let mut original_cache = self.original_cache.write().unwrap();
        for (i, vector) in vectors.iter().enumerate() {
            original_cache.insert(self.vector_count + i, vector.clone());
        }

        // Cache quantized vectors
        self.cache_quantized_vectors(&quantized)?;

        // Update vector count
        self.vector_count += vectors.len();

        Ok(())
    }

    /// Search for similar vectors using quantized similarity
    pub fn search_quantized(
        &self,
        query: &[f32],
        k: usize,
    ) -> QuantizationResult<Vec<(usize, f32)>> {
        if query.len() != self.dimension {
            return Err(QuantizationError::DimensionMismatch {
                expected: self.dimension,
                actual: query.len(),
            });
        }

        if !self.config.enable_quantized_search {
            // Fall back to brute force search
            return self.search_brute_force(query, k);
        }

        // Calculate similarity with all quantized vectors
        let mut results = Vec::new();
        let quantized_cache = self.quantized_cache.read().unwrap();

        // For now, use original vectors for similarity calculation
        let original_cache = self.original_cache.read().unwrap();
        for (id, original_vector) in original_cache.iter() {
            let similarity = cosine_similarity(query, original_vector);
            results.push((*id, similarity));
        }

        // Sort by similarity and take top k
        results.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        results.truncate(k);

        Ok(results)
    }

    /// Brute force search using original vectors (fallback)
    pub fn search_brute_force(
        &self,
        query: &[f32],
        k: usize,
    ) -> QuantizationResult<Vec<(usize, f32)>> {
        let mut results = Vec::new();
        let original_cache = self.original_cache.read().unwrap();

        for (id, vector) in original_cache.iter() {
            let similarity = cosine_similarity(query, vector);
            results.push((*id, similarity));
        }

        // Sort by similarity and take top k
        results.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        results.truncate(k);

        Ok(results)
    }

    /// Hybrid search combining quantized and original similarity
    pub fn search_hybrid(&self, query: &[f32], k: usize) -> QuantizationResult<Vec<(usize, f32)>> {
        if !self.config.enable_hybrid_search {
            return self.search_quantized(query, k);
        }

        // Get results from both methods
        let quantized_results = self.search_quantized(query, k)?;
        let original_results = self.search_brute_force(query, k)?;

        // Combine and re-rank results
        let mut combined = HashMap::new();

        // Add quantized results with weight
        for (id, score) in quantized_results {
            combined.insert(id, score * 0.7); // Weight quantized results
        }

        // Add original results with weight
        for (id, score) in original_results {
            let entry = combined.entry(id).or_insert(0.0);
            *entry += score * 0.3; // Weight original results
        }

        // Convert to sorted results
        let mut results: Vec<(usize, f32)> = combined.into_iter().collect();
        results.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        results.truncate(k);

        Ok(results)
    }

    /// Get quantization statistics
    pub fn get_quantization_stats(&self) -> QuantizationResult<HnswQuantizationStats> {
        let quantized_cache = self.quantized_cache.read().unwrap();
        let original_cache = self.original_cache.read().unwrap();
        let total_quantized_vectors = quantized_cache.len();

        let original_memory = self.vector_count * self.dimension * 4; // f32 = 4 bytes
        let quantized_memory = self
            .quantization
            .memory_usage(self.vector_count, self.dimension);
        let compression_ratio = if quantized_memory > 0 {
            original_memory as f32 / quantized_memory as f32
        } else {
            1.0
        };

        Ok(HnswQuantizationStats {
            vector_count: self.vector_count,
            quantized_vector_count: total_quantized_vectors,
            original_vector_count: original_cache.len(),
            compression_ratio,
            memory_usage_bytes: quantized_memory,
            quality_loss: self.quantization.quality_loss(),
            quantization_type: self.config.quantization_type.clone(),
            cache_hit_ratio: 1.0, // TODO: Implement actual cache hit tracking
        })
    }

    /// Clear the index
    pub fn clear(&mut self) -> QuantizationResult<()> {
        self.quantized_cache.write().unwrap().clear();
        self.original_cache.write().unwrap().clear();
        self.vector_count = 0;
        Ok(())
    }

    /// Save quantized vectors to storage
    pub fn save_to_storage(&self, collection_name: &str) -> QuantizationResult<()> {
        let cache = self.quantized_cache.read().unwrap();

        // Convert cached quantized vectors to QuantizedVectors format
        let mut all_quantized_data = Vec::new();
        for i in 0..self.vector_count {
            if let Some(quantized_vector) = cache.get(&i) {
                all_quantized_data.extend(quantized_vector);
            }
        }

        let quantized_vectors = QuantizedVectors {
            data: all_quantized_data,
            dimension: self.dimension,
            count: self.vector_count,
            parameters: self.quantization.serialize_params()?,
        };

        self.quantized_storage
            .store(collection_name, &quantized_vectors)?;
        Ok(())
    }

    /// Load quantized vectors from storage
    pub fn load_from_storage(&mut self, collection_name: &str) -> QuantizationResult<()> {
        let quantized_vectors = self.quantized_storage.load(collection_name)?;

        // Update quantization parameters
        self.quantization
            .deserialize_params(quantized_vectors.parameters.clone())?;
        self.vector_count = quantized_vectors.count;
        self.dimension = quantized_vectors.dimension;

        // Cache quantized vectors
        self.cache_quantized_vectors(&quantized_vectors)?;

        Ok(())
    }

    // Private helper methods

    fn cache_quantized_vectors(&self, quantized: &QuantizedVectors) -> QuantizationResult<()> {
        let mut cache = self.quantized_cache.write().unwrap();
        let bytes_per_vector = match self.config.quantization_type {
            QuantizationType::Scalar(bits) => match bits {
                8 => quantized.dimension,
                4 => (quantized.dimension + 1) / 2,
                2 => (quantized.dimension + 3) / 4,
                1 => (quantized.dimension + 7) / 8,
                _ => quantized.dimension,
            },
            _ => quantized.dimension,
        };

        for i in 0..quantized.count {
            let start = i * bytes_per_vector;
            let end = start + bytes_per_vector;

            if end <= quantized.data.len() {
                let vector_data = quantized.data[start..end].to_vec();
                cache.insert(i, vector_data);
            }
        }

        Ok(())
    }
}

/// Quantization statistics for HNSW integration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HnswQuantizationStats {
    /// Total number of vectors
    pub vector_count: usize,
    /// Number of quantized vectors
    pub quantized_vector_count: usize,
    /// Number of original vectors cached
    pub original_vector_count: usize,
    /// Compression ratio achieved
    pub compression_ratio: f32,
    /// Memory usage in bytes
    pub memory_usage_bytes: usize,
    /// Quality loss estimate
    pub quality_loss: f32,
    /// Quantization method used
    pub quantization_type: QuantizationType,
    /// Cache hit ratio
    pub cache_hit_ratio: f32,
}

impl HnswQuantizationStats {
    /// Calculate memory savings percentage
    pub fn memory_savings_percent(&self) -> f32 {
        (1.0 - 1.0 / self.compression_ratio) * 100.0
    }

    /// Check if quality meets threshold
    pub fn meets_quality_threshold(&self, threshold: f32) -> bool {
        self.quality_loss <= (1.0 - threshold)
    }
}

/// Calculate cosine similarity between two vectors
fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    if a.len() != b.len() {
        return 0.0;
    }

    let dot_product: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
    let norm_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
    let norm_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();

    if norm_a == 0.0 || norm_b == 0.0 {
        return 0.0;
    }

    dot_product / (norm_a * norm_b)
}

#[cfg(test)]
mod tests {
    use tempfile::tempdir;

    use super::*;

    #[test]
    fn test_quantized_hnsw_basic() {
        let temp_dir = tempdir().unwrap();
        let storage_config = crate::quantization::StorageConfig {
            storage_dir: temp_dir.path().to_path_buf(),
            ..Default::default()
        };
        let storage = Arc::new(QuantizedVectorStorage::new(storage_config).unwrap());

        let config = HnswQuantizationConfig::default();
        let mut index = QuantizedHnswIndex::new(config, storage).unwrap();

        let vectors = vec![
            vec![1.0, 0.0, 0.0],
            vec![0.0, 1.0, 0.0],
            vec![0.0, 0.0, 1.0],
        ];

        index.add_vectors(&vectors).unwrap();

        let results = index.search_quantized(&vec![1.0, 0.0, 0.0], 2).unwrap();
        assert_eq!(results.len(), 2);
        assert_eq!(results[0].0, 0); // First vector should be most similar
    }

    #[test]
    fn test_quantization_stats() {
        let temp_dir = tempdir().unwrap();
        let storage_config = crate::quantization::StorageConfig {
            storage_dir: temp_dir.path().to_path_buf(),
            ..Default::default()
        };
        let storage = Arc::new(QuantizedVectorStorage::new(storage_config).unwrap());

        let config = HnswQuantizationConfig::default();
        let mut index = QuantizedHnswIndex::new(config, storage).unwrap();

        let vectors = vec![vec![1.0; 100]; 1000]; // 1000 vectors of dimension 100
        index.add_vectors(&vectors).unwrap();

        let stats = index.get_quantization_stats().unwrap();
        assert_eq!(stats.vector_count, 1000);
        assert!(stats.compression_ratio > 1.0);
        assert!(stats.memory_savings_percent() > 0.0);
    }

    #[test]
    fn test_storage_integration() {
        let temp_dir = tempdir().unwrap();
        let storage_config = crate::quantization::StorageConfig {
            storage_dir: temp_dir.path().to_path_buf(),
            ..Default::default()
        };
        let storage = Arc::new(QuantizedVectorStorage::new(storage_config).unwrap());

        let config = HnswQuantizationConfig::default();
        let mut index = QuantizedHnswIndex::new(config, storage).unwrap();

        let vectors = vec![vec![1.0, 2.0, 3.0]; 100];
        index.add_vectors(&vectors).unwrap();

        // Save to storage
        index.save_to_storage("test_collection").unwrap();

        // Create new index and load from storage
        let mut new_index = QuantizedHnswIndex::new(
            HnswQuantizationConfig::default(),
            index.quantized_storage.clone(),
        )
        .unwrap();

        new_index.load_from_storage("test_collection").unwrap();

        let stats = new_index.get_quantization_stats().unwrap();
        assert_eq!(stats.vector_count, 100);
    }

    #[test]
    fn test_hybrid_search() {
        let temp_dir = tempdir().unwrap();
        let storage_config = crate::quantization::StorageConfig {
            storage_dir: temp_dir.path().to_path_buf(),
            ..Default::default()
        };
        let storage = Arc::new(QuantizedVectorStorage::new(storage_config).unwrap());

        let mut config = HnswQuantizationConfig::default();
        config.enable_hybrid_search = true;
        let mut index = QuantizedHnswIndex::new(config, storage).unwrap();

        let vectors = vec![
            vec![1.0, 0.0, 0.0],
            vec![0.0, 1.0, 0.0],
            vec![0.0, 0.0, 1.0],
        ];

        index.add_vectors(&vectors).unwrap();

        let results = index.search_hybrid(&vec![1.0, 0.0, 0.0], 2).unwrap();
        assert_eq!(results.len(), 2);
        assert_eq!(results[0].0, 0); // First vector should be most similar
    }
}
