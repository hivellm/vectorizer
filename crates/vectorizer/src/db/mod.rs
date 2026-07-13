//! Database module for Vectorizer
//!
//! ## Lock convention
//!
//! `db/` mixes two lock libraries on purpose, not by accident — pick the
//! right one for the critical section you're writing:
//!
//! - **`parking_lot::{Mutex, RwLock}`** is the default for every short,
//!   synchronous critical section (no `.await` while the guard is held).
//!   It is faster than `std::sync` and, unlike `tokio::sync`, panics
//!   loudly if you try to hold a guard across an `.await` point instead
//!   of silently blocking the executor thread. Almost all of `db/` uses
//!   this (e.g. `VectorStore`'s internal maps, `DashMap` shard guards).
//! - **`tokio::sync::{Mutex, RwLock}`** is reserved for the few call sites
//!   that genuinely hold a guard across an `.await` — currently
//!   `auto_save.rs` and `wal_integration.rs`. Both modules run inside the
//!   async auto-save / WAL-flush tasks and need the guard to survive
//!   asynchronous I/O, which a `parking_lot` guard cannot do safely.
//!
//! Do not introduce a third lock type, and do not reach for
//! `tokio::sync` outside the two sanctioned modules above — if a new
//! critical section needs to cross an `.await`, either restructure it to
//! avoid holding the guard across the await, or extend this list
//! (deliberately, with a comment) rather than silently mixing lock
//! libraries per call site.

pub mod async_indexing;
pub mod auto_save;
pub mod backpressure;
mod collection;
pub mod collection_normalization;
pub mod graph;
pub mod graph_relationship_discovery;
pub mod hybrid_search;
pub mod payload_index;
pub mod storage_backend;
pub mod ttl_reaper;
pub mod upsert_queue;

#[cfg(feature = "hive-gpu")]
pub mod hive_gpu_collection;

#[cfg(feature = "hive-gpu")]
pub mod gpu_detection;

pub mod distributed_sharded_collection;
pub mod multi_tenancy;
pub mod optimized_hnsw;
pub mod raft;
pub mod shard_topology;
pub mod sharded_collection;
pub mod sharding;
pub mod vector_store;
mod wal_integration;

pub use async_indexing::{AsyncIndexManager, IndexBuildProgress, IndexBuildStatus};
pub use auto_save::AutoSaveManager;
pub use backpressure::{BackpressureGuard, BackpressurePermit};
pub use collection::{Collection, VectorCountSample};
pub use collection_normalization::CollectionNormalizationHelper;
pub use distributed_sharded_collection::DistributedShardedCollection;
#[cfg(feature = "hive-gpu")]
pub use gpu_detection::{GpuBackendType, GpuDetector, GpuInfo};
pub use graph::{Edge, Graph, Node, RelationshipType};
pub use graph_relationship_discovery::{
    DiscoveryStats, GraphRelationshipHelper, discover_edges_for_collection,
    discover_edges_for_node, discover_similarity_relationships,
};
pub use hybrid_search::{HybridScoringAlgorithm, HybridSearchConfig, HybridSearchResult};
pub use multi_tenancy::{
    MultiTenancyManager, TenantId, TenantMetadata, TenantOperation, TenantQuotas, TenantUsage,
    TenantUsageUpdate,
};
pub use optimized_hnsw::{OptimizedHnswConfig, OptimizedHnswIndex};
pub use raft::{
    LogEntry, LogIndex, NodeId, RaftConfig, RaftNode, RaftRole, RaftState, RaftStateMachine, Term,
};
pub use sharding::{ConsistentHashRing, ShardId, ShardRebalancer, ShardRouter};
pub use ttl_reaper::{DEFAULT_REAPER_INTERVAL_SECS, TtlReaper};
pub use upsert_queue::{AdmissionError, AdmissionStatus, UpsertQueue, UpsertTicket};
pub use vector_store::{CollectionType, VectorStore};
