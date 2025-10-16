//! Collection implementation for storing vectors

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use dashmap::DashMap;
use parking_lot::RwLock;
use tracing::{debug, info, warn};

use super::optimized_hnsw::{OptimizedHnswConfig, OptimizedHnswIndex};
use crate::error::{Result, VectorizerError};
use crate::models::{
    CollectionConfig, CollectionMetadata, DistanceMetric, SearchResult, Vector, vector_utils,
};

/// A collection of vectors with an associated HNSW index
#[derive(Clone, Debug)]
pub struct Collection {
    /// Collection name
    name: String,
    /// Collection configuration
    config: CollectionConfig,
    /// Vector storage (quantized for memory efficiency when SQ enabled)
    vectors: Arc<Mutex<HashMap<String, Vector>>>,
    /// Quantized vector storage (only used when quantization is enabled)
    /// Uses 75% less memory than Vec<f32> (1 byte vs 4 bytes per dimension)
    quantized_vectors: Arc<Mutex<HashMap<String, crate::models::QuantizedVector>>>,
    /// Vector IDs in insertion order (for persistence consistency)
    vector_order: Arc<RwLock<Vec<String>>>,
    /// HNSW index for similarity search
    index: Arc<RwLock<OptimizedHnswIndex>>,
    /// Embedding type used for this collection
    embedding_type: Arc<RwLock<String>>,
    /// Set of unique document IDs (for counting documents)
    document_ids: Arc<DashMap<String, ()>>,
    /// Persistent vector count (maintains count even when vectors are unloaded)
    vector_count: Arc<RwLock<usize>>,
    /// Creation timestamp
    created_at: chrono::DateTime<chrono::Utc>,
    /// Last update timestamp
    updated_at: Arc<RwLock<chrono::DateTime<chrono::Utc>>>,
}

impl Collection {
    /// Get collection name
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Get collection config
    pub fn config(&self) -> &CollectionConfig {
        &self.config
    }

    /// Create a new collection
    pub fn new(name: String, config: CollectionConfig) -> Self {
        Self::new_with_embedding_type(name, config, "bm25".to_string())
    }

    /// Create a new collection with specific embedding type
    pub fn new_with_embedding_type(
        name: String,
        config: CollectionConfig,
        embedding_type: String,
    ) -> Self {
        // Convert HnswConfig to OptimizedHnswConfig
        let optimized_config = OptimizedHnswConfig {
            max_connections: config.hnsw_config.m,
            max_connections_0: config.hnsw_config.m * 2,
            ef_construction: config.hnsw_config.ef_construction,
            seed: config.hnsw_config.seed,
            distance_metric: config.metric,
            parallel: true,
            initial_capacity: 100_000,
            batch_size: 1000,
        };

        let index = OptimizedHnswIndex::new(config.dimension, optimized_config)
            .expect("Failed to create optimized HNSW index");
        let now = chrono::Utc::now();

        Self {
            name,
            config,
            vectors: Arc::new(Mutex::new(HashMap::new())),
            quantized_vectors: Arc::new(Mutex::new(HashMap::new())),
            vector_order: Arc::new(RwLock::new(Vec::new())),
            index: Arc::new(RwLock::new(index)),
            embedding_type: Arc::new(RwLock::new(embedding_type)),
            document_ids: Arc::new(DashMap::new()),
            vector_count: Arc::new(RwLock::new(0)),
            created_at: now,
            updated_at: Arc::new(RwLock::new(now)),
        }
    }

    /// Get collection metadata
    pub fn metadata(&self) -> CollectionMetadata {
        CollectionMetadata {
            name: self.name.clone(),
            created_at: self.created_at,
            updated_at: *self.updated_at.read(),
            vector_count: *self.vector_count.read(),
            document_count: self.document_ids.len(),
            config: self.config.clone(),
        }
    }

    /// Get the embedding type used for this collection
    pub fn get_embedding_type(&self) -> String {
        self.embedding_type.read().clone()
    }

    /// Set the embedding type for this collection
    pub fn set_embedding_type(&self, embedding_type: String) {
        *self.embedding_type.write() = embedding_type;
    }

    /// Insert a batch of vectors
    pub fn insert_batch(&self, vectors: Vec<Vector>) -> Result<()> {
        // Validate dimensions
        for vector in &vectors {
            if vector.dimension() != self.config.dimension {
                return Err(VectorizerError::InvalidDimension {
                    expected: self.config.dimension,
                    got: vector.dimension(),
                });
            }
        }

        // Insert vectors and update index
        let vectors_len = vectors.len();
        let index = self.index.write();
        let mut vector_order = self.vector_order.write();
        for mut vector in vectors {
            let id = vector.id.clone();
            let mut data = vector.data.clone();

            // Normalize vector for cosine similarity
            if matches!(self.config.metric, DistanceMetric::Cosine) {
                data = vector_utils::normalize_vector(&data);
                vector.data = data.clone(); // Update stored vector to normalized version
            }

            // Extract document ID from payload for tracking unique documents
            if let Some(payload) = &vector.payload {
                if let Some(file_path) = payload.data.get("file_path") {
                    if let Some(file_path_str) = file_path.as_str() {
                        self.document_ids.insert(file_path_str.to_string(), ());
                    }
                }
            }

            // Apply quantization if enabled - store in quantized format to save memory
            if matches!(
                self.config.quantization,
                crate::models::QuantizationConfig::SQ { bits: 8 }
            ) {
                // Store as quantized vector (75% memory reduction)
                let quantized_vector = crate::models::QuantizedVector::from_vector(vector.clone());
                debug!(
                    "Storing quantized vector '{}' ({} bytes instead of {})",
                    id,
                    quantized_vector.memory_size(),
                    data.len() * 4
                );
                self.quantized_vectors
                    .lock()
                    .unwrap()
                    .insert(id.clone(), quantized_vector);

                // Don't store full precision vector to save memory
                // It will be reconstructed on-demand from quantized version
            } else {
                // Store full precision vector
                self.vectors.lock().unwrap().insert(id.clone(), vector);
            }

            // Track insertion order for persistence consistency
            vector_order.push(id.clone());

            // Add to index (using full precision for search accuracy)
            index.add(id.clone(), data.clone())?;
        }

        // Update vector count
        *self.vector_count.write() += vectors_len;

        // Update timestamp
        *self.updated_at.write() = chrono::Utc::now();

        Ok(())
    }

    /// Insert a single vector
    pub fn insert(&self, vector: Vector) -> Result<()> {
        self.insert_batch(vec![vector])
    }

    /// Update a vector
    pub fn update(&self, mut vector: Vector) -> Result<()> {
        // Validate dimension
        if vector.dimension() != self.config.dimension {
            return Err(VectorizerError::InvalidDimension {
                expected: self.config.dimension,
                got: vector.dimension(),
            });
        }

        let id = vector.id.clone();
        let mut data = vector.data.clone();

        // Check if vector exists
        if !self.vectors.lock().unwrap().contains_key(&id) {
            return Err(VectorizerError::VectorNotFound(id));
        }

        // Normalize vector for cosine similarity
        if matches!(self.config.metric, DistanceMetric::Cosine) {
            data = vector_utils::normalize_vector(&data);
            vector.data = data.clone(); // Update stored vector to normalized version
        }

        // Update vector
        self.vectors.lock().unwrap().insert(id.clone(), vector);

        // Update index
        let index = self.index.write();
        index.update(&id, &data)?;

        // Update timestamp
        *self.updated_at.write() = chrono::Utc::now();

        Ok(())
    }

    /// Delete a vector
    pub fn delete(&self, vector_id: &str) -> Result<()> {
        // Remove from storage (both quantized and full precision)
        let found = if matches!(
            self.config.quantization,
            crate::models::QuantizationConfig::SQ { bits: 8 }
        ) {
            self.quantized_vectors
                .lock()
                .unwrap()
                .remove(vector_id)
                .is_some()
        } else {
            self.vectors.lock().unwrap().remove(vector_id).is_some()
        };

        if !found {
            return Err(VectorizerError::VectorNotFound(vector_id.to_string()));
        }

        // Remove from order tracking
        let mut vector_order = self.vector_order.write();
        vector_order.retain(|id| id != vector_id);

        // Remove from index
        let index = self.index.write();
        index.remove(vector_id)?;

        // Update vector count
        *self.vector_count.write() -= 1;

        // Update timestamp
        *self.updated_at.write() = chrono::Utc::now();

        Ok(())
    }

    /// Get a vector by ID
    pub fn get_vector(&self, vector_id: &str) -> Result<Vector> {
        // If quantization is enabled, get from quantized storage
        if matches!(
            self.config.quantization,
            crate::models::QuantizationConfig::SQ { bits: 8 }
        ) {
            let quantized_vector = self
                .quantized_vectors
                .lock()
                .unwrap()
                .get(vector_id)
                .cloned()
                .ok_or_else(|| VectorizerError::VectorNotFound(vector_id.to_string()))?;

            // Dequantize on-demand (only when needed for API response)
            let mut vector = quantized_vector.to_vector();

            // Normalize payload content (fix line endings from legacy data)
            if let Some(ref mut payload) = vector.payload {
                payload.normalize();
            }

            return Ok(vector);
        }

        // Otherwise get from full precision storage
        let vector = self
            .vectors
            .lock()
            .unwrap()
            .get(vector_id)
            .cloned()
            .ok_or_else(|| VectorizerError::VectorNotFound(vector_id.to_string()))?;

        // Normalize payload content (fix line endings from legacy data)
        let mut normalized_vector = vector;
        if let Some(ref mut payload) = normalized_vector.payload {
            payload.normalize();
        }

        Ok(normalized_vector)
    }

    /// Search for similar vectors
    pub fn search(&self, query_vector: &[f32], k: usize) -> Result<Vec<SearchResult>> {
        // Validate dimension
        if query_vector.len() != self.config.dimension {
            return Err(VectorizerError::InvalidDimension {
                expected: self.config.dimension,
                got: query_vector.len(),
            });
        }

        // Normalize query vector for cosine similarity
        let search_vector = if matches!(self.config.metric, DistanceMetric::Cosine) {
            vector_utils::normalize_vector(query_vector)
        } else {
            query_vector.to_vec()
        };

        // Search in index
        let index = self.index.read();
        let neighbors = index.search(&search_vector, k)?;

        // Build results - check quantized storage first if quantization is enabled
        let mut results = Vec::with_capacity(neighbors.len());
        let use_quantization = matches!(
            self.config.quantization,
            crate::models::QuantizationConfig::SQ { bits: 8 }
        );

        for (id, score) in neighbors {
            let vector = if use_quantization {
                // Get from quantized storage and dequantize on-demand
                if let Some(quantized) = self.quantized_vectors.lock().unwrap().get(&id) {
                    quantized.to_vector()
                } else {
                    continue; // Vector not found
                }
            } else {
                // Get from full precision storage
                if let Some(v) = self.vectors.lock().unwrap().get(&id) {
                    v.clone()
                } else {
                    continue; // Vector not found
                }
            };

            // Normalize payload content (fix line endings from legacy data)
            let normalized_payload = vector.payload.as_ref().map(|p| p.normalized());

            results.push(SearchResult {
                id: id.clone(),
                score,
                vector: Some(vector.data.clone()),
                payload: normalized_payload,
            });
        }

        Ok(results)
    }

    /// Get the number of vectors in the collection
    pub fn vector_count(&self) -> usize {
        // Count from whichever storage is being used
        if matches!(
            self.config.quantization,
            crate::models::QuantizationConfig::SQ { bits: 8 }
        ) {
            self.quantized_vectors.lock().unwrap().len()
        } else {
            self.vectors.lock().unwrap().len()
        }
    }

    /// Requantize existing vectors if quantization is enabled (parallel processing)
    /// Migrates vectors from full precision to quantized storage
    pub fn requantize_existing_vectors(&self) -> Result<()> {
        use rayon::prelude::*;

        if matches!(
            self.config.quantization,
            crate::models::QuantizationConfig::SQ { bits: 8 }
        ) {
            debug!(
                "Migrating existing vectors to quantized storage in collection '{}'",
                self.name
            );

            let mut vectors = self.vectors.lock().unwrap();
            let vector_count = vectors.len();

            if vector_count == 0 {
                return Ok(());
            }

            // Convert all vectors to quantized format in parallel
            let quantized: Vec<(String, crate::models::QuantizedVector)> = vectors
                .par_iter()
                .map(|(id, vector)| {
                    let qv = crate::models::QuantizedVector::from_vector(vector.clone());
                    (id.clone(), qv)
                })
                .collect();

            // Move to quantized storage
            let mut quantized_storage = self.quantized_vectors.lock().unwrap();
            for (id, qv) in quantized {
                quantized_storage.insert(id, qv);
            }

            // Clear full precision storage to free memory
            vectors.clear();
            drop(vectors); // Explicitly drop to free memory immediately

            info!(
                "✅ Migrated {} vectors to quantized storage (~75% memory reduction)",
                vector_count
            );
        }

        Ok(())
    }

    /// Quantize a vector using scalar quantization (8-bit)
    fn quantize_vector(&self, vector: &[f32], bits: u8) -> Result<Vec<u8>> {
        let max_val = 2_u32.pow(bits as u32) - 1;
        let mut quantized = Vec::with_capacity(vector.len());

        for &val in vector {
            // Normalize to [0, 1] range (assuming vectors are normalized to [-1, 1])
            let normalized = (val + 1.0) / 2.0;
            let normalized_clamped = normalized.clamp(0.0, 1.0);
            let quantized_val = (normalized_clamped * max_val as f32) as u8;
            quantized.push(quantized_val);
        }

        Ok(quantized)
    }

    /// Dequantize a vector from scalar quantization (8-bit)
    fn dequantize_vector(&self, quantized: &[u8], bits: u8) -> Result<Vec<f32>> {
        let max_val = 2_u32.pow(bits as u32) - 1;
        let mut dequantized = Vec::with_capacity(quantized.len());

        for &val in quantized {
            // Denormalize from [0, 1] range back to [-1, 1]
            let normalized = val as f32 / max_val as f32;
            let denormalized = normalized * 2.0 - 1.0;
            dequantized.push(denormalized);
        }

        Ok(dequantized)
    }

    /// Estimate memory usage in bytes with quantization support
    pub fn estimated_memory_usage(&self) -> usize {
        let vector_count = self.vectors.lock().unwrap().len();
        let dimension = self.config.dimension;

        // Check if quantization is enabled in config
        let quantization_enabled = matches!(
            self.config.quantization,
            crate::models::QuantizationConfig::SQ { bits: 8 }
        );

        if quantization_enabled {
            // Calculate memory usage for quantized vectors (4x compression with SQ-8bit)
            let mut total_memory = 0;
            let mut quantized_vectors = 0;
            let mut unquantized_vectors = 0;

            for vector in self.vectors.lock().unwrap().iter() {
                // Base overhead for Vector struct
                total_memory += std::mem::size_of::<Vector>();

                // Check if vector is quantized (data cleared)
                let is_quantized = vector.1.data.is_empty();

                if is_quantized {
                    // Vector is quantized - minimal memory usage
                    total_memory += dimension; // 1 byte per dimension for quantized data
                    quantized_vectors += 1;
                } else {
                    // Vector not yet quantized - use f32 data size
                    total_memory += std::mem::size_of::<f32>() * dimension;
                    unquantized_vectors += 1;
                }

                // Payload overhead
                if let Some(payload) = &vector.1.payload {
                    total_memory += std::mem::size_of_val(payload);
                }
            }

            // Debug logging for memory analysis
            if vector_count > 0 {
                let quantization_ratio = quantized_vectors as f32 / vector_count as f32;
                debug!(
                    "🔍 [MEMORY ANALYSIS] Collection '{}': {}/{} vectors quantized ({:.1}%), total_memory: {} bytes",
                    self.name,
                    quantized_vectors,
                    vector_count,
                    quantization_ratio * 100.0,
                    total_memory
                );
            }

            total_memory
        } else {
            // Standard memory usage without quantization
            let vector_size = std::mem::size_of::<f32>() * dimension;
            let entry_overhead = std::mem::size_of::<String>() + std::mem::size_of::<Vector>();
            let total_per_vector = vector_size + entry_overhead;

            vector_count * total_per_vector
        }
    }

    /// Fast load from cache without building HNSW index (index built lazily on first search)
    pub fn load_from_cache(
        &self,
        persisted_vectors: Vec<crate::persistence::PersistedVector>,
    ) -> Result<()> {
        use crate::persistence::PersistedVector;

        debug!(
            "Fast loading {} vectors into collection '{}' (lazy index)",
            persisted_vectors.len(),
            self.name
        );

        // Convert persisted vectors to runtime vectors
        let mut runtime_vectors = Vec::with_capacity(persisted_vectors.len());
        for pv in persisted_vectors {
            runtime_vectors.push(pv.into_runtime_with_payload()?);
        }

        debug!("Loaded {} vectors from cache", runtime_vectors.len());

        // Use fast load for runtime vectors
        self.fast_load_vectors(runtime_vectors)?;

        // Apply quantization automatically after loading if enabled
        if matches!(
            self.config.quantization,
            crate::models::QuantizationConfig::SQ { bits: 8 }
        ) {
            debug!(
                "Applying automatic quantization to loaded vectors in collection '{}'",
                self.name
            );
            self.requantize_existing_vectors()?;
        }

        debug!(
            "Fast loaded {} vectors into collection '{}' with HNSW index",
            self.vectors.lock().unwrap().len(),
            self.name
        );
        Ok(())
    }

    pub fn load_from_cache_with_hnsw_dump(
        &self,
        persisted_vectors: Vec<crate::persistence::PersistedVector>,
        hnsw_dump_path: Option<&std::path::Path>,
        hnsw_basename: Option<&str>,
    ) -> Result<()> {
        use crate::persistence::PersistedVector;

        info!(
            "🚀 [CACHE LOAD] Loading {} vectors into collection '{}' from cache (HNSW dump path: {:?})",
            persisted_vectors.len(),
            self.name,
            hnsw_dump_path
        );

        // Try to load HNSW index from dump first
        let hnsw_loaded = if let (Some(path), Some(basename)) = (hnsw_dump_path, hnsw_basename) {
            match self.load_hnsw_index_from_dump(path, basename) {
                Ok(_) => {
                    info!(
                        "Successfully loaded HNSW index from dump for collection '{}'",
                        self.name
                    );
                    true
                }
                Err(e) => false,
            }
        } else {
            false
        };

        // Convert persisted vectors to runtime vectors
        let mut runtime_vectors = Vec::with_capacity(persisted_vectors.len());
        for pv in persisted_vectors {
            runtime_vectors.push(pv.into_runtime_with_payload()?);
        }

        // IMPORTANT: Do NOT apply quantization here - it will clear vector data
        // and prevent proper re-persistence. Quantization should only be applied
        // in search operations, not in storage.
        // The original code was clearing vector.data after loading from cache,
        // which caused re-saved .bin files to be empty.

        debug!(
            "Loaded {} vectors without applying quantization (preserving data for persistence)",
            runtime_vectors.len()
        );

        if hnsw_loaded {
            // HNSW index already loaded, just load vectors into memory
            debug!(
                "Loading {} vectors into memory (HNSW index already loaded)",
                runtime_vectors.len()
            );
            self.load_vectors_into_memory(runtime_vectors)?;
        } else {
            // Build HNSW index from scratch
            debug!("Building HNSW index from {} vectors", runtime_vectors.len());
            self.fast_load_vectors(runtime_vectors)?;
        }

        debug!(
            "Loaded {} vectors into collection '{}' {}",
            self.vectors.lock().unwrap().len(),
            self.name,
            if hnsw_loaded {
                "(from dump)"
            } else {
                "(rebuilt)"
            }
        );
        Ok(())
    }

    /// Load vectors into memory without building HNSW index (assumes index is already loaded)
    pub fn load_vectors_into_memory(&self, vectors: Vec<Vector>) -> Result<()> {
        let vectors_len = vectors.len();
        let mut vector_order = self.vector_order.write();

        for vector in vectors {
            let id = vector.id.clone();

            // Extract document ID from payload for tracking unique documents
            if let Some(payload) = &vector.payload {
                if let Some(file_path) = payload.data.get("file_path") {
                    if let Some(file_path_str) = file_path.as_str() {
                        self.document_ids.insert(file_path_str.to_string(), ());
                    }
                }
            }

            // Store vector
            self.vectors.lock().unwrap().insert(id.clone(), vector);

            // Track insertion order
            vector_order.push(id.clone());
        }

        // Update vector count
        *self.vector_count.write() += vectors_len;

        // Update timestamp
        *self.updated_at.write() = chrono::Utc::now();

        debug!(
            "Loaded {} vectors into memory for collection '{}'",
            vector_order.len(),
            self.name
        );
        Ok(())
    }

    /// Fast load vectors with HNSW index building
    pub fn fast_load_vectors(&self, vectors: Vec<Vector>) -> Result<()> {
        let vectors_len = vectors.len();
        debug!(
            "Fast loading {} vectors into collection '{}' with HNSW index",
            vectors_len, self.name
        );

        let mut vector_order = self.vector_order.write();
        let index = self.index.write();

        // Prepare vectors for batch insertion
        let mut batch_vectors = Vec::with_capacity(vectors_len);

        for mut vector in vectors {
            let id = vector.id.clone();

            // Extract document ID from payload for tracking unique documents
            if let Some(payload) = &vector.payload {
                if let Some(file_path) = payload.data.get("file_path") {
                    if let Some(file_path_str) = file_path.as_str() {
                        self.document_ids.insert(file_path_str.to_string(), ());
                    }
                }
            }

            // Vector is already normalized by into_runtime_with_payload if needed

            // CRITICAL FIX: Apply quantization if enabled (same as insert_batch does)
            // This ensures vectors are stored consistently whether loaded from disk or inserted fresh
            if matches!(
                self.config.quantization,
                crate::models::QuantizationConfig::SQ { bits: 8 }
            ) {
                // Store as quantized vector (75% memory reduction)
                let quantized_vector = crate::models::QuantizedVector::from_vector(vector.clone());
                debug!("Storing quantized vector '{}' during fast load", id);
                self.quantized_vectors
                    .lock()
                    .unwrap()
                    .insert(id.clone(), quantized_vector);

                // Don't store full precision vector to save memory
            } else {
                // Store full precision vector
                self.vectors
                    .lock()
                    .unwrap()
                    .insert(id.clone(), vector.clone());
            }

            // Add to batch for HNSW index (using full precision for search accuracy)
            batch_vectors.push((id.clone(), vector.data.clone()));

            // Track insertion order
            vector_order.push(id.clone());
        }

        // Batch insert into HNSW index
        index.batch_add(batch_vectors)?;

        // Update vector count
        *self.vector_count.write() += vectors_len;

        // Update timestamp
        *self.updated_at.write() = chrono::Utc::now();

        debug!(
            "Fast loaded {} vectors into collection '{}' with HNSW index",
            vector_order.len(),
            self.name
        );
        Ok(())
    }

    /// Get all vectors in the collection (for persistence)
    /// Returns vectors in insertion order to maintain HNSW index consistency
    pub fn get_all_vectors(&self) -> Vec<Vector> {
        let vector_order = self.vector_order.read();

        // If quantization is enabled, get from quantized storage
        if matches!(
            self.config.quantization,
            crate::models::QuantizationConfig::SQ { bits: 8 }
        ) {
            let quantized = self.quantized_vectors.lock().unwrap();
            vector_order
                .iter()
                .filter_map(|id| quantized.get(id).map(|qv| qv.to_vector()))
                .collect()
        } else {
            // Get from full precision storage
            let vectors = self.vectors.lock().unwrap();
            vector_order
                .iter()
                .filter_map(|id| vectors.get(id).cloned())
                .collect()
        }
    }

    /// Dump the HNSW index to files for faster reloading
    pub fn dump_hnsw_index<P: AsRef<std::path::Path>>(&self, path: P) -> Result<String> {
        let basename = format!("{}_hnsw", self.name);
        (*self.index.write()).file_dump(path, &basename)?;
        Ok(basename)
    }

    /// Load HNSW index from dump files
    pub fn load_hnsw_index_from_dump<P: AsRef<std::path::Path>>(
        &self,
        path: P,
        basename: &str,
    ) -> Result<()> {
        (*self.index.write()).load_from_dump(path, basename)
    }

    /// Calculate approximate memory usage of the collection
    pub fn calculate_memory_usage(&self) -> (usize, usize, usize) {
        let mut index_size = 0;
        let mut payload_size = 0;
        let mut total_size = 0;

        // Check if quantization is enabled
        let use_quantization = matches!(
            self.config.quantization,
            crate::models::QuantizationConfig::SQ { bits: 8 }
        );

        if use_quantization {
            // Calculate from quantized storage
            let quantized_vectors = self.quantized_vectors.lock().unwrap();
            let vector_count = quantized_vectors.len();

            for (id, qvector) in quantized_vectors.iter() {
                // Vector ID size
                let id_size = id.len();

                // Quantized vector data size (u8 = 1 byte per element)
                let vector_data_size = qvector.quantized_data.len();

                // Quantization params (2 f32 values)
                let quant_params_size = std::mem::size_of::<f32>() * 2;

                // Payload size (approximate JSON serialization size)
                let vector_payload_size = if let Some(ref payload) = qvector.payload {
                    match serde_json::to_string(&payload.data) {
                        Ok(json_str) => json_str.len(),
                        Err(_) => 0,
                    }
                } else {
                    0
                };

                // Total size for this quantized vector
                let vector_total_size =
                    id_size + vector_data_size + quant_params_size + vector_payload_size;

                index_size += id_size + vector_data_size + quant_params_size;
                payload_size += vector_payload_size;
                total_size += vector_total_size;
            }

            // Add HNSW index overhead (approximate)
            let index_overhead = vector_count * 100;
            index_size += index_overhead;
            total_size += index_overhead;
        } else {
            // Calculate from full precision storage
            let vectors = self.vectors.lock().unwrap();
            let vector_count = vectors.len();

            for (id, vector) in vectors.iter() {
                // Vector ID size
                let id_size = id.len();

                // Vector data size (f32 = 4 bytes per element)
                let vector_data_size = vector.data.len() * 4;

                // Payload size (approximate JSON serialization size)
                let vector_payload_size = if let Some(ref payload) = vector.payload {
                    match serde_json::to_string(&payload.data) {
                        Ok(json_str) => json_str.len(),
                        Err(_) => 0,
                    }
                } else {
                    0
                };

                // Total size for this vector
                let vector_total_size = id_size + vector_data_size + vector_payload_size;

                index_size += id_size + vector_data_size;
                payload_size += vector_payload_size;
                total_size += vector_total_size;
            }

            // Add HNSW index overhead (approximate)
            let index_overhead = vector_count * 100;
            index_size += index_overhead;
            total_size += index_overhead;
        }

        (index_size, payload_size, total_size)
    }

    /// Get collection size information in a formatted way
    pub fn get_size_info(&self) -> (String, String, String) {
        let (index_size, payload_size, total_size) = self.calculate_memory_usage();

        let format_bytes = |bytes: usize| -> String {
            if bytes >= 1024 * 1024 {
                format!("{:.1} MB", bytes as f64 / (1024.0 * 1024.0))
            } else if bytes >= 1024 {
                format!("{:.1} KB", bytes as f64 / 1024.0)
            } else {
                format!("{} B", bytes)
            }
        };

        (
            format_bytes(index_size),
            format_bytes(payload_size),
            format_bytes(total_size),
        )
    }

    /// Dump HNSW index to centralized cache directory for faster future loading
    pub fn dump_hnsw_index_for_cache<P: AsRef<std::path::Path>>(
        &self,
        _project_path: P,
    ) -> Result<()> {
        use tracing::{debug, info, warn};

        // Get the vectorizer root directory (where config.yml is located)
        let current_dir = std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."));
        let cache_dir = current_dir.join("data");

        if !cache_dir.exists() {
            std::fs::create_dir_all(&cache_dir)?;
        }

        let basename = format!("{}_hnsw", self.name);

        // Check if index has vectors
        let index_len = (*self.index.read()).len();

        if index_len == 0 {
            warn!(
                "⚠️ COLLECTION DUMP WARNING: Index is empty for collection '{}'",
                self.name
            );
            return Err(VectorizerError::IndexError(format!(
                "Index is empty for collection '{}'",
                self.name
            )));
        }

        (*self.index.write()).file_dump(&cache_dir, &basename)?;
        info!(
            "✅ Successfully dumped HNSW index for collection '{}' to centralized cache",
            self.name
        );
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{DistanceMetric, HnswConfig};

    fn create_test_collection() -> Collection {
        let config = CollectionConfig {
            dimension: 3,
            metric: DistanceMetric::Euclidean,
            hnsw_config: HnswConfig::default(),
            quantization: crate::models::QuantizationConfig::None,
            compression: Default::default(),
            normalization: None,
        };
        Collection::new("test".to_string(), config)
    }

    #[test]
    fn test_insert_and_get_vector() {
        let collection = create_test_collection();

        let vector = Vector::new("v1".to_string(), vec![1.0, 2.0, 3.0]);
        collection.insert(vector.clone()).unwrap();

        let retrieved = collection.get_vector("v1").unwrap();
        assert_eq!(retrieved.id, "v1");
        assert_eq!(retrieved.data, vec![1.0, 2.0, 3.0]);
    }

    #[test]
    fn test_dimension_validation() {
        let collection = create_test_collection();

        // Wrong dimension
        let vector = Vector::new("v1".to_string(), vec![1.0, 2.0]); // 2D instead of 3D
        let result = collection.insert(vector);

        assert!(matches!(
            result,
            Err(VectorizerError::InvalidDimension {
                expected: 3,
                got: 2
            })
        ));
    }

    #[test]
    fn test_update_vector() {
        let collection = create_test_collection();

        // Insert original
        let vector = Vector::new("v1".to_string(), vec![1.0, 2.0, 3.0]);
        collection.insert(vector).unwrap();

        // Update
        let updated = Vector::new("v1".to_string(), vec![4.0, 5.0, 6.0]);
        collection.update(updated).unwrap();

        // Verify
        let retrieved = collection.get_vector("v1").unwrap();
        assert_eq!(retrieved.data, vec![4.0, 5.0, 6.0]);
    }

    #[test]
    fn test_delete_vector() {
        let collection = create_test_collection();

        // Insert and delete
        let vector = Vector::new("v1".to_string(), vec![1.0, 2.0, 3.0]);
        collection.insert(vector).unwrap();
        assert_eq!(collection.vector_count(), 1);

        collection.delete("v1").unwrap();
        assert_eq!(collection.vector_count(), 0);

        // Try to get deleted vector
        let result = collection.get_vector("v1");
        assert!(matches!(result, Err(VectorizerError::VectorNotFound(_))));
    }
}
