//! Collection — root module for the `Collection` type.
//!
//! The `Collection` struct and its trait impls live here. Behaviour is
//! split across sibling files by concern to keep any one file reviewable
//! in isolation:
//!
//! - [`data`] — insert / insert_batch / update / delete / get_vector / search / hybrid_search
//! - [`index`] — HNSW construction, dump/load, fast batch load
//! - [`persistence`] — cache load, memory accounting, vector enumeration
//! - [`graph`] — enable_graph, populate_graph_if_empty, graph accessors
//! - [`quantization`] — SQ quantize/dequantize, PQ train + encode, requantize migration
//!
//! Constructors and trivial accessors stay in this file.

// Internal data-layout file: public fields are self-documenting; the
// blanket allow keeps `cargo doc -W missing-docs` clean without padding
// every field with a tautological `///` comment. See
// phase4_enforce-public-api-docs.
#![allow(missing_docs)]

use std::collections::HashMap;
use std::sync::Arc;

use dashmap::DashMap;
use parking_lot::{Mutex, RwLock};

use super::graph_relationship_discovery::GraphRelationshipHelper;
use super::optimized_hnsw::{OptimizedHnswConfig, OptimizedHnswIndex};
use super::payload_index::PayloadIndex;
use super::storage_backend::VectorStorageBackend;
use crate::error::{Result, VectorizerError};
use crate::models::{CollectionConfig, CollectionMetadata, SparseVectorIndex, StorageType, Vector};

mod data;
mod graph;
mod index;
mod persistence;
mod quantization;

/// A collection of vectors with an associated HNSW index
#[derive(Clone, Debug)]
pub struct Collection {
    /// Collection name
    pub(super) name: String,
    /// Collection configuration
    pub(super) config: CollectionConfig,
    /// Owner ID (tenant/user ID for multi-tenancy in HiveHub cluster mode)
    /// None for standalone mode, Some(uuid) for cluster mode
    pub(super) owner_id: Option<uuid::Uuid>,
    /// Vector storage (Memory or Mmap)
    pub(super) vectors: VectorStorageBackend,
    /// Quantized vector storage (only used when quantization is enabled)
    /// Uses 75% less memory than Vec<f32> (1 byte vs 4 bytes per dimension)
    pub(super) quantized_vectors: Arc<Mutex<HashMap<String, crate::models::QuantizedVector>>>,
    /// Vector IDs in insertion order (for persistence consistency)
    pub(super) vector_order: Arc<RwLock<Vec<String>>>,
    /// HNSW index for similarity search
    pub(super) index: Arc<RwLock<OptimizedHnswIndex>>,
    /// Embedding type used for this collection
    pub(super) embedding_type: Arc<RwLock<String>>,
    /// Set of unique document IDs (for counting documents)
    pub(super) document_ids: Arc<DashMap<String, ()>>,
    /// Persistent vector count (maintains count even when vectors are unloaded)
    pub(super) vector_count: Arc<RwLock<usize>>,
    /// Payload index for efficient filtering
    pub(super) payload_index: Arc<PayloadIndex>,
    /// Sparse vector index for sparse vector search
    pub(super) sparse_index: Arc<RwLock<SparseVectorIndex>>,
    /// Product Quantization instance (optional, only when PQ is enabled)
    pub(super) pq_quantizer: Arc<RwLock<Option<crate::quantization::product::ProductQuantization>>>,
    /// Creation timestamp
    pub(super) created_at: chrono::DateTime<chrono::Utc>,
    /// Last update timestamp
    pub(super) updated_at: Arc<RwLock<chrono::DateTime<chrono::Utc>>>,
    /// Graph for relationship tracking (optional, enabled via config)
    pub(super) graph: Option<Arc<super::graph::Graph>>,
}

impl GraphRelationshipHelper for Collection {
    fn search_similar_vectors(
        &self,
        query_vector: &[f32],
        limit: usize,
    ) -> Result<Vec<(String, f32)>> {
        let results = self.search(query_vector, limit)?;
        Ok(results.into_iter().map(|r| (r.id, r.score)).collect())
    }

    fn get_vector(&self, vector_id: &str) -> Result<Vector> {
        self.get_vector(vector_id)
    }
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

    /// Create a new collection with a specific owner (for HiveHub cluster mode)
    pub fn new_with_owner(name: String, config: CollectionConfig, owner_id: uuid::Uuid) -> Self {
        Self::new_with_owner_and_embedding(name, config, Some(owner_id), "bm25".to_string())
    }

    /// Create a new collection with specific embedding type
    pub fn new_with_embedding_type(
        name: String,
        config: CollectionConfig,
        embedding_type: String,
    ) -> Self {
        Self::new_with_owner_and_embedding(name, config, None, embedding_type)
    }

    /// Create a new collection with owner and embedding type
    pub fn new_with_owner_and_embedding(
        name: String,
        config: CollectionConfig,
        owner_id: Option<uuid::Uuid>,
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

        // SAFE: OptimizedHnswIndex::new only fails on a zero/negative dimension
        // or invalid HnswConfig — both impossible here because `dimension`
        // is u32-validated by CollectionConfig and `optimized_config` is
        // built from already-validated fields. A failure indicates a logic
        // bug in this constructor, not a runtime condition.
        #[allow(clippy::expect_used)]
        let index = OptimizedHnswIndex::new(config.dimension, optimized_config)
            .expect("Failed to create optimized HNSW index");
        let now = chrono::Utc::now();

        // Initialize payload index with common fields
        let payload_index = Arc::new(PayloadIndex::new());

        // Auto-index common payload fields
        payload_index.add_index_config(super::payload_index::PayloadIndexConfig::new(
            "file_path".to_string(),
            super::payload_index::PayloadIndexType::Keyword,
        ));
        payload_index.add_index_config(super::payload_index::PayloadIndexConfig::new(
            "chunk_index".to_string(),
            super::payload_index::PayloadIndexType::Integer,
        ));

        // Initialize sparse vector index
        let sparse_index = Arc::new(RwLock::new(SparseVectorIndex::new()));

        // Initialize vector storage
        let vectors = match config.storage_type.unwrap_or(StorageType::Memory) {
            StorageType::Memory => VectorStorageBackend::new_memory(),
            StorageType::Mmap => {
                // Use a standard path for mmap files: ./data/{name}.mmap
                // In a real implementation, the data directory should be passed in
                let data_dir = std::path::Path::new("./data");
                if !data_dir.exists() {
                    let _ = std::fs::create_dir_all(data_dir);
                }
                let path = data_dir.join(format!("{}.mmap", name));

                // SAFE-ish: This is a real I/O call that can legitimately
                // fail (permissions, disk full, corrupt file). Long-term we
                // should propagate the error via Collection::new ->
                // Result<Self> — tracked under
                // phase4_enforce-no-unwrap-policy item 2.4 follow-up. Until
                // then, an mmap failure surfaces as a startup panic, which
                // matches the prior behaviour and is preferable to a silent
                // fallback to in-memory storage on a path the operator
                // explicitly chose.
                #[allow(clippy::expect_used)]
                let storage =
                    crate::storage::mmap::MmapVectorStorage::open(&path, config.dimension)
                        .expect("Failed to initialize mmap storage");
                VectorStorageBackend::new_mmap(storage)
            }
        };

        let graph_enabled = config.graph.as_ref().map(|g| g.enabled).unwrap_or(false);
        let collection_name = name.clone();
        let graph = if graph_enabled {
            Some(Arc::new(super::graph::Graph::new(collection_name)))
        } else {
            None
        };

        Self {
            name,
            config,
            owner_id,
            vectors,
            quantized_vectors: Arc::new(Mutex::new(HashMap::new())),
            vector_order: Arc::new(RwLock::new(Vec::new())),
            index: Arc::new(RwLock::new(index)),
            embedding_type: Arc::new(RwLock::new(embedding_type)),
            document_ids: Arc::new(DashMap::new()),
            vector_count: Arc::new(RwLock::new(0)),
            payload_index,
            sparse_index,
            pq_quantizer: Arc::new(RwLock::new(None)),
            created_at: now,
            updated_at: Arc::new(RwLock::new(now)),
            graph,
        }
    }

    /// Get the owner ID (tenant/user ID for multi-tenancy)
    pub fn owner_id(&self) -> Option<uuid::Uuid> {
        self.owner_id
    }

    /// Set the owner ID (used when loading from persistence or updating ownership)
    pub fn set_owner_id(&mut self, owner_id: Option<uuid::Uuid>) {
        self.owner_id = owner_id;
    }

    /// Check if this collection belongs to a specific owner
    pub fn belongs_to(&self, owner_id: &uuid::Uuid) -> bool {
        self.owner_id.map(|id| &id == owner_id).unwrap_or(false)
    }

    /// Get collection metadata
    pub fn metadata(&self) -> CollectionMetadata {
        CollectionMetadata {
            name: self.name.clone(),
            tenant_id: self.owner_id.map(|id| id.to_string()),
            created_at: self.created_at,
            updated_at: *self.updated_at.read(),
            vector_count: *self.vector_count.read(),
            document_count: self.document_ids.len(),
            config: self.config.clone(),
        }
    }

    /// Get the number of unique documents in this collection
    pub fn document_count(&self) -> usize {
        self.document_ids.len()
    }

    /// Get the embedding type used for this collection
    pub fn get_embedding_type(&self) -> String {
        self.embedding_type.read().clone()
    }

    /// Set the embedding type for this collection
    pub fn set_embedding_type(&self, embedding_type: String) {
        *self.embedding_type.write() = embedding_type;
    }

    /// Get the number of vectors in the collection
    pub fn vector_count(&self) -> usize {
        // Use persistent vector count (maintains count even when vectors are unloaded)
        *self.vector_count.read()
    }
}

#[cfg(test)]
#[path = "../collection_tests.rs"]
mod tests;
