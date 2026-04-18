//! `CollectionType` — the dispatch enum wrapping the four concrete
//! collection backends (`Cpu`, `HiveGpu`, `Sharded`, `DistributedSharded`)
//! so that `VectorStore` can hold a single uniform map.
//!
//! All shared methods (search, CRUD, metadata, memory accounting,
//! graph access) delegate into the variant's underlying type. Some
//! paths differ by backend — for example, `DistributedSharded` has no
//! synchronous `get_vector` because every read crosses the cluster
//! router — and those limitations are returned as explicit errors or
//! warnings rather than silent fallbacks.

use tracing::{debug, warn};

use crate::db::collection::Collection;
use crate::db::distributed_sharded_collection::DistributedShardedCollection;
#[cfg(feature = "hive-gpu")]
use crate::db::hive_gpu_collection::HiveGpuCollection;
use crate::db::sharded_collection::ShardedCollection;
use crate::error::{Result, VectorizerError};
use crate::models::{CollectionConfig, CollectionMetadata, SearchResult, Vector};

/// Enum to represent different collection types (CPU, GPU, or Sharded)
pub enum CollectionType {
    /// CPU-based collection
    Cpu(Collection),
    /// Hive-GPU collection (Metal, CUDA, WebGPU)
    #[cfg(feature = "hive-gpu")]
    HiveGpu(HiveGpuCollection),
    /// Sharded collection (distributed across multiple shards on single server)
    Sharded(ShardedCollection),
    /// Distributed sharded collection (distributed across multiple servers)
    DistributedSharded(DistributedShardedCollection),
}

impl std::fmt::Debug for CollectionType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CollectionType::Cpu(c) => write!(f, "CollectionType::Cpu({})", c.name()),
            #[cfg(feature = "hive-gpu")]
            CollectionType::HiveGpu(c) => write!(f, "CollectionType::HiveGpu({})", c.name()),
            CollectionType::Sharded(c) => write!(f, "CollectionType::Sharded({})", c.name()),
            CollectionType::DistributedSharded(c) => {
                write!(f, "CollectionType::DistributedSharded({})", c.name())
            }
        }
    }
}

impl CollectionType {
    /// Get collection name
    pub fn name(&self) -> &str {
        match self {
            CollectionType::Cpu(c) => c.name(),
            #[cfg(feature = "hive-gpu")]
            CollectionType::HiveGpu(c) => c.name(),
            CollectionType::Sharded(c) => c.name(),
            CollectionType::DistributedSharded(c) => c.name(),
        }
    }

    /// Get collection config
    pub fn config(&self) -> &CollectionConfig {
        match self {
            CollectionType::Cpu(c) => c.config(),
            #[cfg(feature = "hive-gpu")]
            CollectionType::HiveGpu(c) => c.config(),
            CollectionType::Sharded(c) => c.config(),
            CollectionType::DistributedSharded(c) => c.config(),
        }
    }

    /// Get owner ID (for multi-tenancy in HiveHub cluster mode)
    pub fn owner_id(&self) -> Option<uuid::Uuid> {
        match self {
            CollectionType::Cpu(c) => c.owner_id(),
            #[cfg(feature = "hive-gpu")]
            CollectionType::HiveGpu(c) => c.owner_id(),
            CollectionType::Sharded(c) => c.owner_id(),
            CollectionType::DistributedSharded(_) => None, // Distributed collections don't support multi-tenancy yet
        }
    }

    /// Check if this collection belongs to a specific owner
    pub fn belongs_to(&self, owner_id: &uuid::Uuid) -> bool {
        match self {
            CollectionType::Cpu(c) => c.belongs_to(owner_id),
            #[cfg(feature = "hive-gpu")]
            CollectionType::HiveGpu(c) => c.belongs_to(owner_id),
            CollectionType::Sharded(c) => c.belongs_to(owner_id),
            CollectionType::DistributedSharded(_) => false, // Distributed collections don't support multi-tenancy yet
        }
    }

    /// Add a vector to the collection
    pub fn add_vector(&mut self, _id: String, vector: Vector) -> Result<()> {
        match self {
            CollectionType::Cpu(c) => c.insert(vector),
            #[cfg(feature = "hive-gpu")]
            CollectionType::HiveGpu(c) => c.add_vector(vector).map(|_| ()),
            CollectionType::Sharded(c) => c.insert(vector),
            CollectionType::DistributedSharded(c) => {
                // Distributed collections require async operations
                // Use tokio runtime to execute async insert
                let rt = tokio::runtime::Runtime::new().map_err(|e| {
                    VectorizerError::Storage(format!("Failed to create runtime: {}", e))
                })?;
                rt.block_on(c.insert(vector))
            }
        }
    }

    /// Insert a batch of vectors (optimized for performance)
    pub fn insert_batch(&mut self, vectors: Vec<Vector>) -> Result<()> {
        match self {
            CollectionType::Cpu(c) => c.insert_batch(vectors),
            #[cfg(feature = "hive-gpu")]
            CollectionType::HiveGpu(c) => {
                // For Hive-GPU, use batch insertion
                c.add_vectors(vectors)?;
                Ok(())
            }
            CollectionType::Sharded(c) => c.insert_batch(vectors),
            CollectionType::DistributedSharded(c) => {
                // Distributed collections - use optimized batch insert
                let rt = tokio::runtime::Runtime::new().map_err(|e| {
                    VectorizerError::Storage(format!("Failed to create runtime: {}", e))
                })?;
                rt.block_on(c.insert_batch(vectors))
            }
        }
    }

    /// Search for similar vectors
    pub fn search(&self, query: &[f32], limit: usize) -> Result<Vec<SearchResult>> {
        match self {
            CollectionType::Cpu(c) => c.search(query, limit),
            #[cfg(feature = "hive-gpu")]
            CollectionType::HiveGpu(c) => c.search(query, limit),
            CollectionType::Sharded(c) => c.search(query, limit, None),
            CollectionType::DistributedSharded(c) => {
                // Distributed collections require async operations
                let rt = tokio::runtime::Runtime::new().map_err(|e| {
                    VectorizerError::Storage(format!("Failed to create runtime: {}", e))
                })?;
                rt.block_on(c.search(query, limit, None, None))
            }
        }
    }

    /// Perform hybrid search combining dense and sparse vectors
    pub fn hybrid_search(
        &self,
        query_dense: &[f32],
        query_sparse: Option<&crate::models::SparseVector>,
        config: crate::db::HybridSearchConfig,
    ) -> Result<Vec<SearchResult>> {
        match self {
            CollectionType::Cpu(c) => c.hybrid_search(query_dense, query_sparse, config),
            #[cfg(feature = "hive-gpu")]
            CollectionType::HiveGpu(_) => {
                // GPU collections don't support hybrid search yet
                // Fallback to dense search
                self.search(query_dense, config.final_k)
            }
            CollectionType::Sharded(c) => {
                // For sharded collections, use multi-shard hybrid search
                c.hybrid_search(query_dense, query_sparse, config, None)
            }
            CollectionType::DistributedSharded(c) => {
                // For distributed sharded collections, use distributed hybrid search
                let rt = tokio::runtime::Runtime::new().map_err(|e| {
                    VectorizerError::Storage(format!("Failed to create runtime: {}", e))
                })?;
                rt.block_on(c.hybrid_search(query_dense, query_sparse, config, None))
            }
        }
    }

    /// Get collection metadata
    pub fn metadata(&self) -> CollectionMetadata {
        match self {
            CollectionType::Cpu(c) => c.metadata(),
            #[cfg(feature = "hive-gpu")]
            CollectionType::HiveGpu(c) => c.metadata(),
            CollectionType::Sharded(c) => {
                // Create metadata for sharded collection
                CollectionMetadata {
                    name: c.name().to_string(),
                    tenant_id: None,
                    created_at: chrono::Utc::now(),
                    updated_at: chrono::Utc::now(),
                    vector_count: c.vector_count(),
                    document_count: c.document_count(),
                    config: c.config().clone(),
                }
            }
            CollectionType::DistributedSharded(c) => {
                // Create metadata for distributed sharded collection
                let rt = tokio::runtime::Runtime::new().unwrap_or_else(|_| {
                    tokio::runtime::Runtime::new().expect("Failed to create runtime")
                });
                let vector_count = rt.block_on(c.vector_count()).unwrap_or(0);
                // Use local document count for now (sync) - distributed count requires async
                let document_count = c.document_count();
                CollectionMetadata {
                    name: c.name().to_string(),
                    tenant_id: None,
                    created_at: chrono::Utc::now(),
                    updated_at: chrono::Utc::now(),
                    vector_count,
                    document_count,
                    config: c.config().clone(),
                }
            }
        }
    }

    /// Delete a vector from the collection
    pub fn delete_vector(&mut self, id: &str) -> Result<()> {
        match self {
            CollectionType::Cpu(c) => c.delete(id),
            #[cfg(feature = "hive-gpu")]
            CollectionType::HiveGpu(c) => c.remove_vector(id.to_string()),
            CollectionType::Sharded(c) => c.delete(id),
            CollectionType::DistributedSharded(_) => {
                // `DistributedShardedCollection::delete` is async by design
                // (it broadcasts to the shard-owning node). Synchronous
                // callers must go through the async cluster router instead.
                Err(VectorizerError::Storage(
                    "delete_vector is not supported synchronously on distributed \
                     collections; use the async cluster router"
                        .to_string(),
                ))
            }
        }
    }

    /// Update a vector atomically (faster than delete+add)
    pub fn update_vector(&mut self, vector: Vector) -> Result<()> {
        match self {
            CollectionType::Cpu(c) => c.update(vector),
            #[cfg(feature = "hive-gpu")]
            CollectionType::HiveGpu(c) => c.update(vector),
            CollectionType::Sharded(c) => c.update(vector),
            CollectionType::DistributedSharded(_) => Err(VectorizerError::Storage(
                "update_vector is not supported synchronously on distributed \
                 collections; use the async cluster router"
                    .to_string(),
            )),
        }
    }

    /// Get a vector by ID
    pub fn get_vector(&self, vector_id: &str) -> Result<Vector> {
        match self {
            CollectionType::Cpu(c) => c.get_vector(vector_id),
            #[cfg(feature = "hive-gpu")]
            CollectionType::HiveGpu(c) => c.get_vector_by_id(vector_id),
            CollectionType::Sharded(c) => c.get_vector(vector_id),
            CollectionType::DistributedSharded(_) => Err(VectorizerError::Storage(
                "get_vector is not supported synchronously on distributed \
                 collections; use the async cluster router"
                    .to_string(),
            )),
        }
    }

    /// Get the number of vectors in the collection
    ///
    /// For distributed collections this returns the locally-known document
    /// count (sync), not a cluster-wide total; callers that need the exact
    /// figure should use `DistributedShardedCollection::vector_count().await`.
    pub fn vector_count(&self) -> usize {
        match self {
            CollectionType::Cpu(c) => c.vector_count(),
            #[cfg(feature = "hive-gpu")]
            CollectionType::HiveGpu(c) => c.vector_count(),
            CollectionType::Sharded(c) => c.vector_count(),
            CollectionType::DistributedSharded(c) => c.document_count(),
        }
    }

    /// Get the number of documents in the collection
    /// This may differ from vector_count if documents have multiple vectors
    pub fn document_count(&self) -> usize {
        match self {
            CollectionType::Cpu(c) => c.document_count(),
            #[cfg(feature = "hive-gpu")]
            CollectionType::HiveGpu(c) => c.vector_count(), // GPU collections treat vectors as documents
            CollectionType::Sharded(c) => c.document_count(),
            CollectionType::DistributedSharded(c) => c.document_count(),
        }
    }

    /// Get estimated memory usage
    pub fn estimated_memory_usage(&self) -> usize {
        match self {
            CollectionType::Cpu(c) => c.estimated_memory_usage(),
            #[cfg(feature = "hive-gpu")]
            CollectionType::HiveGpu(c) => c.estimated_memory_usage(),
            CollectionType::Sharded(c) => {
                // Sum memory usage from all shards
                c.shard_counts().values().sum::<usize>() * c.config().dimension * 4 // Rough estimate
            }
            CollectionType::DistributedSharded(c) => {
                // Rough estimate from the locally-known document count;
                // cluster-wide exact accounting would require an RPC.
                c.document_count() * c.config().dimension * 4
            }
        }
    }

    /// Get all vectors in the collection
    pub fn get_all_vectors(&self) -> Vec<Vector> {
        match self {
            CollectionType::Cpu(c) => c.get_all_vectors(),
            #[cfg(feature = "hive-gpu")]
            CollectionType::HiveGpu(c) => c.get_all_vectors(),
            CollectionType::Sharded(_) => {
                // Sharded collections don't support get_all_vectors efficiently
                // Return empty for now - could be implemented by querying all shards
                Vec::new()
            }
            CollectionType::DistributedSharded(_) => {
                // Same rationale as Sharded: requires iterating every shard and
                // every node; no synchronous path today.
                Vec::new()
            }
        }
    }

    /// Get embedding type
    pub fn get_embedding_type(&self) -> String {
        match self {
            CollectionType::Cpu(c) => c.get_embedding_type(),
            #[cfg(feature = "hive-gpu")]
            CollectionType::HiveGpu(c) => c.get_embedding_type(),
            CollectionType::Sharded(_) => "sharded".to_string(),
            CollectionType::DistributedSharded(_) => "distributed".to_string(),
        }
    }

    /// Get graph for this collection (if enabled)
    pub fn get_graph(&self) -> Option<&std::sync::Arc<crate::db::graph::Graph>> {
        match self {
            CollectionType::Cpu(c) => c.get_graph(),
            #[cfg(feature = "hive-gpu")]
            CollectionType::HiveGpu(_) => None, // GPU collections don't support graph yet
            CollectionType::Sharded(_) => None, // Sharded collections don't support graph yet
            CollectionType::DistributedSharded(_) => None, // Distributed collections don't support graph yet
        }
    }

    /// Requantize existing vectors if quantization is enabled
    pub fn requantize_existing_vectors(&self) -> Result<()> {
        match self {
            CollectionType::Cpu(c) => c.requantize_existing_vectors(),
            #[cfg(feature = "hive-gpu")]
            CollectionType::HiveGpu(c) => c.requantize_existing_vectors(),
            CollectionType::Sharded(c) => c.requantize_existing_vectors(),
            CollectionType::DistributedSharded(c) => c.requantize_existing_vectors(),
        }
    }

    /// Calculate approximate memory usage of the collection
    pub fn calculate_memory_usage(&self) -> (usize, usize, usize) {
        match self {
            CollectionType::Cpu(c) => c.calculate_memory_usage(),
            #[cfg(feature = "hive-gpu")]
            CollectionType::HiveGpu(c) => {
                // For Hive-GPU collections, return basic estimation
                let total = c.estimated_memory_usage();
                (total / 2, total / 2, total)
            }
            CollectionType::Sharded(c) => {
                let total = c.vector_count() * c.config().dimension * 4; // Rough estimate
                (total / 2, total / 2, total)
            }
            CollectionType::DistributedSharded(c) => {
                // Same rough estimate as Sharded; distributed collections
                // need a cluster-wide query for exact figures.
                let total = c.document_count() * c.config().dimension * 4;
                (total / 2, total / 2, total)
            }
        }
    }

    /// Get collection size information in a formatted way
    pub fn get_size_info(&self) -> (String, String, String) {
        match self {
            CollectionType::Cpu(c) => c.get_size_info(),
            #[cfg(feature = "hive-gpu")]
            CollectionType::HiveGpu(c) => {
                let total = c.estimated_memory_usage();
                let format_bytes = |bytes: usize| -> String {
                    if bytes >= 1024 * 1024 {
                        format!("{:.1} MB", bytes as f64 / (1024.0 * 1024.0))
                    } else if bytes >= 1024 {
                        format!("{:.1} KB", bytes as f64 / 1024.0)
                    } else {
                        format!("{} B", bytes)
                    }
                };
                let index_size = format_bytes(total / 2);
                let payload_size = format_bytes(total / 2);
                let total_size = format_bytes(total);
                (index_size, payload_size, total_size)
            }
            CollectionType::Sharded(c) => {
                let total = c.vector_count() * c.config().dimension * 4;
                let format_bytes = |bytes: usize| -> String {
                    if bytes >= 1024 * 1024 {
                        format!("{:.1} MB", bytes as f64 / (1024.0 * 1024.0))
                    } else if bytes >= 1024 {
                        format!("{:.1} KB", bytes as f64 / 1024.0)
                    } else {
                        format!("{} B", bytes)
                    }
                };
                let index_size = format_bytes(total / 2);
                let payload_size = format_bytes(total / 2);
                let total_size = format_bytes(total);
                (index_size, payload_size, total_size)
            }
            CollectionType::DistributedSharded(c) => {
                let total = c.document_count() * c.config().dimension * 4;
                let format_bytes = |bytes: usize| -> String {
                    if bytes >= 1024 * 1024 {
                        format!("{:.1} MB", bytes as f64 / (1024.0 * 1024.0))
                    } else if bytes >= 1024 {
                        format!("{:.1} KB", bytes as f64 / 1024.0)
                    } else {
                        format!("{} B", bytes)
                    }
                };
                let index_size = format_bytes(total / 2);
                let payload_size = format_bytes(total / 2);
                let total_size = format_bytes(total);
                (index_size, payload_size, total_size)
            }
        }
    }

    /// Set embedding type
    pub fn set_embedding_type(&mut self, embedding_type: String) {
        match self {
            CollectionType::Cpu(c) => c.set_embedding_type(embedding_type),
            #[cfg(feature = "hive-gpu")]
            CollectionType::HiveGpu(_) => {
                // Hive-GPU doesn't need to track embedding types
                debug!(
                    "Hive-GPU collections don't track embedding types: {}",
                    embedding_type
                );
            }
            CollectionType::Sharded(_) => {
                // Sharded collections don't track embedding types at top level
                debug!(
                    "Sharded collections don't track embedding types: {}",
                    embedding_type
                );
            }
            CollectionType::DistributedSharded(_) => {
                debug!(
                    "Distributed collections don't track embedding types: {}",
                    embedding_type
                );
            }
        }
    }

    /// Load HNSW index from dump
    pub fn load_hnsw_index_from_dump<P: AsRef<std::path::Path>>(
        &self,
        path: P,
        basename: &str,
    ) -> Result<()> {
        match self {
            CollectionType::Cpu(c) => c.load_hnsw_index_from_dump(path, basename),
            #[cfg(feature = "hive-gpu")]
            CollectionType::HiveGpu(_) => {
                warn!("Hive-GPU collections don't support HNSW dump loading yet");
                Ok(())
            }
            CollectionType::Sharded(_) => {
                warn!("Sharded collections don't support HNSW dump loading yet");
                Ok(())
            }
            CollectionType::DistributedSharded(_) => {
                warn!("Distributed collections don't support HNSW dump loading yet");
                Ok(())
            }
        }
    }

    /// Load vectors into memory
    pub fn load_vectors_into_memory(&self, vectors: Vec<Vector>) -> Result<()> {
        match self {
            CollectionType::Cpu(c) => c.load_vectors_into_memory(vectors),
            #[cfg(feature = "hive-gpu")]
            CollectionType::HiveGpu(_) => {
                warn!("Hive-GPU collections don't support vector loading into memory yet");
                Ok(())
            }
            CollectionType::Sharded(c) => {
                // Use batch insert for sharded collections
                c.insert_batch(vectors)
            }
            CollectionType::DistributedSharded(_) => {
                // DistributedSharded's insert is async and bulk load goes
                // through the cluster router; a synchronous in-memory load
                // isn't meaningful here.
                warn!(
                    "Distributed collections don't support synchronous \
                     load_vectors_into_memory; use the async insert path"
                );
                Ok(())
            }
        }
    }

    /// Fast load vectors
    pub fn fast_load_vectors(&mut self, vectors: Vec<Vector>) -> Result<()> {
        match self {
            CollectionType::Cpu(c) => c.fast_load_vectors(vectors),
            #[cfg(feature = "hive-gpu")]
            CollectionType::HiveGpu(c) => {
                // Use batch insertion for better performance
                c.add_vectors(vectors)?;
                Ok(())
            }
            CollectionType::Sharded(c) => {
                // Use batch insert for sharded collections
                c.insert_batch(vectors)
            }
            CollectionType::DistributedSharded(_) => {
                warn!(
                    "Distributed collections don't support synchronous \
                     fast_load_vectors; use the async insert path"
                );
                Ok(())
            }
        }
    }
}
