//! Collection implementation for storing vectors

use crate::{
    error::{Result, VectorizerError},
    models::{
        CollectionConfig, CollectionMetadata, DistanceMetric, SearchResult, Vector, vector_utils,
    },
};
use dashmap::DashMap;
use parking_lot::RwLock;
use std::sync::Arc;
use tracing::{debug, info, warn};

use super::optimized_hnsw::{OptimizedHnswConfig, OptimizedHnswIndex};

/// A collection of vectors with an associated HNSW index
#[derive(Clone, Debug)]
pub struct Collection {
    /// Collection name
    name: String,
    /// Collection configuration
    config: CollectionConfig,
    /// Vector storage
    vectors: Arc<DashMap<String, Vector>>,
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
            vectors: Arc::new(DashMap::new()),
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

            // Store vector
            self.vectors.insert(id.clone(), vector);

            // Track insertion order for persistence consistency
            vector_order.push(id.clone());

            // Add to index
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
        if !self.vectors.contains_key(&id) {
            return Err(VectorizerError::VectorNotFound(id));
        }

        // Normalize vector for cosine similarity
        if matches!(self.config.metric, DistanceMetric::Cosine) {
            data = vector_utils::normalize_vector(&data);
            vector.data = data.clone(); // Update stored vector to normalized version
        }

        // Update vector
        self.vectors.insert(id.clone(), vector);

        // Update index
        let index = self.index.write();
        index.update(&id, &data)?;

        // Update timestamp
        *self.updated_at.write() = chrono::Utc::now();

        Ok(())
    }

    /// Delete a vector
    pub fn delete(&self, vector_id: &str) -> Result<()> {
        // Remove from storage
        self.vectors
            .remove(vector_id)
            .ok_or_else(|| VectorizerError::VectorNotFound(vector_id.to_string()))?;

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
        self.vectors
            .get(vector_id)
            .map(|entry| entry.value().clone())
            .ok_or_else(|| VectorizerError::VectorNotFound(vector_id.to_string()))
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

        // Build results
        let mut results = Vec::with_capacity(neighbors.len());
        for (id, score) in neighbors {
            if let Some(vector) = self.vectors.get(&id) {
                results.push(SearchResult {
                    id: id.clone(),
                    score,
                    vector: Some(vector.data.clone()),
                    payload: vector.payload.clone(),
                });
            }
        }

        Ok(results)
    }

    /// Get the number of vectors in the collection
    pub fn vector_count(&self) -> usize {
        self.vectors.len()
    }

    /// Estimate memory usage in bytes
    pub fn estimated_memory_usage(&self) -> usize {
        let vector_size = std::mem::size_of::<f32>() * self.config.dimension;
        let entry_overhead = std::mem::size_of::<String>() + std::mem::size_of::<Vector>();
        let total_per_vector = vector_size + entry_overhead;

        self.vectors.len() * total_per_vector
    }

    /// Fast load from cache without building HNSW index (index built lazily on first search)
    pub fn load_from_cache(&self, persisted_vectors: Vec<crate::persistence::PersistedVector>) -> Result<()> {
        use crate::persistence::PersistedVector;

        debug!("Fast loading {} vectors into collection '{}' (lazy index)", persisted_vectors.len(), self.name);

        // Convert persisted vectors to runtime vectors
        let mut runtime_vectors = Vec::with_capacity(persisted_vectors.len());
        for pv in persisted_vectors {
            runtime_vectors.push(pv.into_runtime_with_payload()?);
        }

        // Use fast load for runtime vectors
        self.fast_load_vectors(runtime_vectors)?;

        debug!("Fast loaded {} vectors into collection '{}' with HNSW index", self.vectors.len(), self.name);
        Ok(())
    }

    pub fn load_from_cache_with_hnsw_dump(&self, persisted_vectors: Vec<crate::persistence::PersistedVector>, hnsw_dump_path: Option<&std::path::Path>, hnsw_basename: Option<&str>) -> Result<()> {
        use crate::persistence::PersistedVector;

        debug!("Loading {} vectors into collection '{}' from cache", persisted_vectors.len(), self.name);

        // Try to load HNSW index from dump first
        let hnsw_loaded = if let (Some(path), Some(basename)) = (hnsw_dump_path, hnsw_basename) {
            match self.load_hnsw_index_from_dump(path, basename) {
                Ok(_) => {
                    info!("Successfully loaded HNSW index from dump for collection '{}'", self.name);
                    true
                }
                Err(e) => {
                    false
                }
            }
        } else {
            false
        };

        // Convert persisted vectors to runtime vectors
        let mut runtime_vectors = Vec::with_capacity(persisted_vectors.len());
        for pv in persisted_vectors {
            runtime_vectors.push(pv.into_runtime_with_payload()?);
        }

        if hnsw_loaded {
            // HNSW index already loaded, just load vectors into memory
            debug!("Loading {} vectors into memory (HNSW index already loaded)", runtime_vectors.len());
            self.load_vectors_into_memory(runtime_vectors)?;
        } else {
            // Build HNSW index from scratch
            debug!("Building HNSW index from {} vectors", runtime_vectors.len());
            self.fast_load_vectors(runtime_vectors)?;
        }

        debug!("Loaded {} vectors into collection '{}' {}", self.vectors.len(), self.name,
               if hnsw_loaded { "(from dump)" } else { "(rebuilt)" });
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
            self.vectors.insert(id.clone(), vector);

            // Track insertion order
            vector_order.push(id.clone());
        }

        // Update vector count
        *self.vector_count.write() += vectors_len;

        // Update timestamp
        *self.updated_at.write() = chrono::Utc::now();

        debug!("Loaded {} vectors into memory for collection '{}'", vector_order.len(), self.name);
        Ok(())
    }

    /// Fast load vectors with HNSW index building
    pub fn fast_load_vectors(&self, vectors: Vec<Vector>) -> Result<()> {
        let vectors_len = vectors.len();
        debug!("Fast loading {} vectors into collection '{}' with HNSW index", vectors_len, self.name);

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

            // Store vector
            self.vectors.insert(id.clone(), vector.clone());

            // Add to batch for HNSW index
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

        debug!("Fast loaded {} vectors into collection '{}' with HNSW index", vector_order.len(), self.name);
        Ok(())
    }

    /// Get all vectors in the collection (for persistence)
    /// Returns vectors in insertion order to maintain HNSW index consistency
    pub fn get_all_vectors(&self) -> Vec<Vector> {
        let vector_order = self.vector_order.read();
        vector_order
            .iter()
            .filter_map(|id| self.vectors.get(id))
            .map(|entry| entry.value().clone())
            .collect()
    }

    /// Dump the HNSW index to files for faster reloading
    pub fn dump_hnsw_index<P: AsRef<std::path::Path>>(&self, path: P) -> Result<String> {
        let basename = format!("{}_hnsw", self.name);
        (*self.index.write()).file_dump(path, &basename)?;
        Ok(basename)
    }

    /// Load HNSW index from dump files
    pub fn load_hnsw_index_from_dump<P: AsRef<std::path::Path>>(&self, path: P, basename: &str) -> Result<()> {
        (*self.index.write()).load_from_dump(path, basename)
    }


    /// Dump HNSW index to cache directory for faster future loading
    pub fn dump_hnsw_index_for_cache<P: AsRef<std::path::Path>>(&self, project_path: P) -> Result<()> {
        use tracing::{debug, info, warn};
        
        let cache_dir = project_path.as_ref().join(".vectorizer");
        if !cache_dir.exists() {
            std::fs::create_dir_all(&cache_dir)?;
        }

        let basename = format!("{}_hnsw", self.name);
        
        info!("🔍 COLLECTION DUMP DEBUG: Starting HNSW dump for collection '{}'", self.name);
        info!("🔍 COLLECTION DUMP DEBUG: Cache directory: {}", cache_dir.display());
        info!("🔍 COLLECTION DUMP DEBUG: Basename: {}", basename);
        info!("🔍 COLLECTION DUMP DEBUG: Collection has {} vectors, {} documents", 
              self.vector_count(), self.document_ids.len());
        
        // Check if index has vectors
        let index_len = (*self.index.read()).len();
        info!("🔍 COLLECTION DUMP DEBUG: Index length: {}", index_len);
        
        if index_len == 0 {
            warn!("⚠️ COLLECTION DUMP WARNING: Index is empty for collection '{}'", self.name);
            return Err(VectorizerError::IndexError(format!("Index is empty for collection '{}'", self.name)));
        }
        
        debug!("🔍 COLLECTION DUMP DEBUG: Calling index.file_dump...");
        (*self.index.write()).file_dump(&cache_dir, &basename)?;
        info!("✅ Successfully dumped HNSW index for collection '{}' to cache", self.name);
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
            quantization: None,
            compression: Default::default(),
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
