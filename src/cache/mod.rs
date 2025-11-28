//! Query caching module for improved search performance.
//!
//! This module provides an LRU (Least Recently Used) cache for query results,
//! significantly improving performance for repeated queries.

pub mod query_cache;

pub use query_cache::{QueryCache, QueryCacheConfig, QueryKey};
