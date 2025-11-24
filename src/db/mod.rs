//! Database module for Vectorizer

pub mod async_indexing;
pub mod auto_save;
mod collection;
pub mod collection_normalization;
pub mod graph;
mod graph_relationship_discovery;
pub mod hybrid_search;
pub mod payload_index;
pub mod storage_backend;

#[cfg(feature = "hive-gpu")]
pub mod hive_gpu_collection;

#[cfg(feature = "hive-gpu")]
pub mod gpu_detection;

pub mod multi_tenancy;
pub mod optimized_hnsw;
pub mod raft;
pub mod sharded_collection;
pub mod distributed_sharded_collection;
pub mod sharding;
pub mod vector_store;
mod wal_integration;

pub use async_indexing::{AsyncIndexManager, IndexBuildProgress, IndexBuildStatus};
pub use auto_save::AutoSaveManager;
pub use collection::Collection;
pub use collection_normalization::CollectionNormalizationHelper;
pub use graph::{Edge, Graph, Node, RelationshipType};
#[cfg(feature = "hive-gpu")]
pub use gpu_detection::{GpuBackendType, GpuDetector, GpuInfo};
pub use hybrid_search::{HybridScoringAlgorithm, HybridSearchConfig, HybridSearchResult};
pub use multi_tenancy::{
    MultiTenancyManager, TenantId, TenantMetadata, TenantOperation, TenantQuotas, TenantUsage,
    TenantUsageUpdate,
};
pub use optimized_hnsw::{OptimizedHnswConfig, OptimizedHnswIndex};
pub use raft::{
    LogEntry, LogIndex, NodeId, RaftConfig, RaftNode, RaftRole, RaftState, RaftStateMachine, Term,
};
pub use distributed_sharded_collection::DistributedShardedCollection;
pub use sharding::{ConsistentHashRing, ShardId, ShardRebalancer, ShardRouter};
pub use vector_store::{CollectionType, VectorStore};
