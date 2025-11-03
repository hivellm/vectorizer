//! Database module for Vectorizer

pub mod auto_save;
mod collection;
pub mod collection_normalization;

#[cfg(feature = "hive-gpu")]
pub mod hive_gpu_collection;

#[cfg(feature = "hive-gpu")]
pub mod gpu_detection;

pub mod optimized_hnsw;
mod vector_store;

pub use auto_save::AutoSaveManager;
pub use collection::Collection;
pub use collection_normalization::CollectionNormalizationHelper;

#[cfg(feature = "hive-gpu")]
pub use gpu_detection::{GpuBackendType, GpuDetector, GpuInfo};

pub use optimized_hnsw::{OptimizedHnswConfig, OptimizedHnswIndex};
pub use vector_store::{CollectionType, VectorStore};
