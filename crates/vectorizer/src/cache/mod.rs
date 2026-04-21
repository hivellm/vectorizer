//! Query caching module for improved search performance.
//!
//! This module provides an LRU (Least Recently Used) cache for query results,
//! significantly improving performance for repeated queries.

pub mod memory_manager;
pub mod query_cache;

pub use memory_manager::{
    AllocationResult, CacheMemoryManager, CacheMemoryManagerConfig, CacheMemoryStats,
    get_global_cache_memory_manager, init_global_cache_memory_manager,
};
pub use query_cache::{QueryCache, QueryCacheConfig, QueryKey};
