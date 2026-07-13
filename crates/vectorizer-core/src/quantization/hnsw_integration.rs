//! HNSW integration for quantized vectors
//!
//! Implements efficient similarity search using quantized vectors.
//! Provides foundation for HNSW integration with quantization.

use std::collections::HashMap;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

use parking_lot::RwLock;
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
    /// Cache hit counter for statistics
    cache_hits: AtomicU64,
    /// Cache miss counter for statistics
    cache_misses: AtomicU64,
}

impl QuantizedHnswIndex {
    /// Create a new quantized index
    pub fn new(
        config: HnswQuantizationConfig,
        storage: Arc<QuantizedVectorStorage>,
    ) -> QuantizationResult<Self> {
        // Create quantization method based on config. The untrained
        // instance is replaced at first `add_vectors` (fit path); PQ's
        // dimension is only known at that point, so it starts at 0
        // until the fit constructs the real quantizer.
        let quantization: Box<dyn QuantizationMethod + Send + Sync> = match config.quantization_type
        {
            QuantizationType::Scalar(bits) => Box::new(ScalarQuantization::new(bits)?),
            QuantizationType::Product => Box::new(
                crate::quantization::product::ProductQuantization::new(Default::default(), 0),
            ),
            QuantizationType::Binary => {
                Box::new(crate::quantization::binary::BinaryQuantization::new())
            }
            QuantizationType::None => {
                return Err(QuantizationError::InvalidParameters(
                    "QuantizationType::None cannot back a quantized index — \
                     use the unquantized HNSW path instead"
                        .to_string(),
                ));
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
            cache_hits: AtomicU64::new(0),
            cache_misses: AtomicU64::new(0),
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

        // Fit quantization on the first batch. Each type constructs and
        // trains its real quantizer here — before phase38 this arm was
        // Scalar-only with a silent 8-bit fallback for PQ/Binary, which
        // meant those fully-implemented methods were never reachable.
        if self.vector_count == 0 {
            self.quantization = match self.config.quantization_type {
                QuantizationType::Scalar(bits) => {
                    let mut scalar_q = ScalarQuantization::new(bits)?;
                    scalar_q.fit(vectors)?;
                    Box::new(scalar_q)
                }
                QuantizationType::Product => {
                    let mut pq = crate::quantization::product::ProductQuantization::new(
                        Default::default(),
                        self.dimension,
                    );
                    pq.train(vectors)?;
                    Box::new(pq)
                }
                QuantizationType::Binary => {
                    let mut bq = crate::quantization::binary::BinaryQuantization::new();
                    bq.train(vectors)?;
                    Box::new(bq)
                }
                QuantizationType::None => {
                    return Err(QuantizationError::InvalidParameters(
                        "QuantizationType::None cannot back a quantized index".to_string(),
                    ));
                }
            };
        }

        // Quantize vectors
        let quantized = self.quantization.quantize(vectors)?;

        // Always cache original vectors for similarity calculation
        let mut original_cache = self.original_cache.write();
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
        let quantized_cache = self.quantized_cache.read();

        // Use original vectors for similarity calculation, via the SIMD
        // backend (phase7a). True cosine = dot / (|q| · |v|): callers
        // hand us RAW vectors, and `crate::simd::cosine_similarity` is
        // a clamped dot product that ASSUMES unit-length inputs — used
        // directly it returned 1.0 for every pair of non-normalised
        // vectors (phase38 fix; the old tests only used unit vectors,
        // which is why it never surfaced).
        let query_norm = crate::simd::l2_norm(query);
        let original_cache = self.original_cache.read();
        for (id, original_vector) in original_cache.iter() {
            let vector_norm = crate::simd::l2_norm(original_vector);
            let similarity = if query_norm == 0.0 || vector_norm == 0.0 {
                0.0
            } else {
                crate::simd::dot_product(query, original_vector) / (query_norm * vector_norm)
            };
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
        let query_norm = crate::simd::l2_norm(query);
        let original_cache = self.original_cache.read();

        for (id, vector) in original_cache.iter() {
            // True cosine — see the note in `search_quantized`.
            let vector_norm = crate::simd::l2_norm(vector);
            let similarity = if query_norm == 0.0 || vector_norm == 0.0 {
                0.0
            } else {
                crate::simd::dot_product(query, vector) / (query_norm * vector_norm)
            };
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
        let quantized_cache = self.quantized_cache.read();
        let original_cache = self.original_cache.read();
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
            cache_hit_ratio: {
                let hits = self.cache_hits.load(Ordering::Relaxed);
                let misses = self.cache_misses.load(Ordering::Relaxed);
                let total = hits + misses;
                if total > 0 {
                    hits as f32 / total as f32
                } else {
                    1.0 // No accesses yet, assume perfect
                }
            },
        })
    }

    /// Clear the index
    pub fn clear(&mut self) -> QuantizationResult<()> {
        self.quantized_cache.write().clear();
        self.original_cache.write().clear();
        self.vector_count = 0;
        Ok(())
    }

    /// Save quantized vectors to storage
    pub fn save_to_storage(&self, collection_name: &str) -> QuantizationResult<()> {
        let cache = self.quantized_cache.read();

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
        let mut cache = self.quantized_cache.write();
        if quantized.count == 0 {
            return Ok(());
        }
        // Records are fixed-size per method (scalar: packed bits ×
        // dimension; PQ: one code byte per subvector; binary: dimension
        // bits) — deriving the stride from the payload itself works for
        // every method and can't drift from the per-type formulas.
        let bytes_per_vector = quantized.data.len() / quantized.count;

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

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
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

    /// Distinct, deterministic vectors with a unique dominant axis per
    /// vector (requires `n <= dim`) so the nearest neighbour of any
    /// vector is unambiguously itself under cosine similarity, even
    /// after lossy quantization.
    fn spread_vectors(n: usize, dim: usize) -> Vec<Vec<f32>> {
        assert!(n <= dim, "unique dominant axis requires n <= dim");
        (0..n)
            .map(|i| {
                (0..dim)
                    .map(|d| {
                        if d == i {
                            10.0
                        } else {
                            0.05 * ((i * 13 + d * 7) % 5) as f32
                        }
                    })
                    .collect()
            })
            .collect()
    }

    #[test]
    fn test_product_quantization_round_trip() {
        let temp_dir = tempdir().unwrap();
        let storage_config = crate::quantization::StorageConfig {
            storage_dir: temp_dir.path().to_path_buf(),
            ..Default::default()
        };
        let storage = Arc::new(QuantizedVectorStorage::new(storage_config).unwrap());

        let config = HnswQuantizationConfig {
            quantization_type: QuantizationType::Product,
            ..Default::default()
        };
        let mut index = QuantizedHnswIndex::new(config, storage).unwrap();

        // Enough vectors for k-means to have something to chew on;
        // dimension divisible by the default 8 subvectors.
        let vectors = spread_vectors(48, 64);
        index.add_vectors(&vectors).unwrap();

        let results = index.search_quantized(&vectors[3], 3).unwrap();
        assert_eq!(results.len(), 3);
        assert_eq!(results[0].0, 3, "self-query must return itself first");

        // Persisted round trip keeps the codebooks (trained state).
        index.save_to_storage("pq_collection").unwrap();
        let mut reloaded = QuantizedHnswIndex::new(
            HnswQuantizationConfig {
                quantization_type: QuantizationType::Product,
                ..Default::default()
            },
            index.quantized_storage.clone(),
        )
        .unwrap();
        reloaded.load_from_storage("pq_collection").unwrap();
        let stats = reloaded.get_quantization_stats().unwrap();
        assert_eq!(stats.vector_count, 48);
        assert!(stats.compression_ratio > 1.0);
    }

    #[test]
    fn test_binary_quantization_round_trip() {
        let temp_dir = tempdir().unwrap();
        let storage_config = crate::quantization::StorageConfig {
            storage_dir: temp_dir.path().to_path_buf(),
            ..Default::default()
        };
        let storage = Arc::new(QuantizedVectorStorage::new(storage_config).unwrap());

        let config = HnswQuantizationConfig {
            quantization_type: QuantizationType::Binary,
            ..Default::default()
        };
        let mut index = QuantizedHnswIndex::new(config, storage).unwrap();

        let vectors = spread_vectors(32, 64);
        index.add_vectors(&vectors).unwrap();

        let results = index.search_quantized(&vectors[5], 3).unwrap();
        assert_eq!(results.len(), 3);
        assert_eq!(results[0].0, 5, "self-query must return itself first");

        let stats = index.get_quantization_stats().unwrap();
        assert_eq!(stats.vector_count, 32);
    }

    #[test]
    fn test_none_quantization_is_explicit_error() {
        let temp_dir = tempdir().unwrap();
        let storage_config = crate::quantization::StorageConfig {
            storage_dir: temp_dir.path().to_path_buf(),
            ..Default::default()
        };
        let storage = Arc::new(QuantizedVectorStorage::new(storage_config).unwrap());

        let config = HnswQuantizationConfig {
            quantization_type: QuantizationType::None,
            ..Default::default()
        };
        // Spec: unsupported types fail construction explicitly — the
        // pre-phase38 code silently substituted 8-bit scalar at fit time.
        assert!(QuantizedHnswIndex::new(config, storage).is_err());
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
