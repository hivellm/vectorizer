//! Database module for Vectorizer

pub mod async_indexing;
pub mod auto_save;
mod collection;
pub mod collection_normalization;
pub mod hybrid_search;
pub mod payload_index;
pub mod storage_backend;

#[cfg(feature = "hive-gpu")]
pub mod hive_gpu_collection;

#[cfg(feature = "hive-gpu")]
pub mod gpu_detection;

pub mod optimized_hnsw;
mod vector_store;
mod wal_integration;

pub use async_indexing::{AsyncIndexManager, IndexBuildProgress, IndexBuildStatus};
pub use auto_save::AutoSaveManager;
pub use collection::Collection;
pub use collection_normalization::CollectionNormalizationHelper;
#[cfg(feature = "hive-gpu")]
pub use gpu_detection::{GpuBackendType, GpuDetector, GpuInfo};
pub use hybrid_search::{HybridScoringAlgorithm, HybridSearchConfig, HybridSearchResult};
pub use optimized_hnsw::{OptimizedHnswConfig, OptimizedHnswIndex};
pub use vector_store::{CollectionType, VectorStore};
