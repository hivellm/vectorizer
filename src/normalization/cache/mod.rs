//! Multi-tier caching system for text normalization
//!
//! This module provides a three-tier cache architecture:
//! - **Hot Cache (Tier 1)**: In-memory LFU cache for recent normalized text
//! - **Warm Store (Tier 2)**: Memory-mapped disk storage for frequent access
//! - **Cold Store (Tier 3)**: Compressed blob storage for long-term persistence
//!
//! # Architecture
//!
//! ```text
//! ┌─────────────────────────────────────────────┐
//! │              Cache Manager                  │
//! ├─────────────────────────────────────────────┤
//! │                                             │
//! │  Request → Hot Cache (LFU) → Warm Store    │
//! │           (memory)        → Cold Store      │
//! │                              (compressed)   │
//! │                                             │
//! └─────────────────────────────────────────────┘
//! ```
//!
//! # Example
//!
//! ```no_run
//! use vectorizer::normalization::cache::{CacheManager, CacheConfig};
//! use vectorizer::normalization::ContentHash;
//!
//! # async fn example() -> anyhow::Result<()> {
//! let config = CacheConfig::default();
//! let mut cache = CacheManager::new(config)?;
//!
//! let hash = ContentHash::from_bytes([0u8; 32]);
//! let text = "Normalized text content";
//!
//! // Store in cache
//! cache.put_normalized(hash, text).await?;
//!
//! // Retrieve from cache
//! if let Some(cached) = cache.get_normalized(&hash).await? {
//!     println!("Cache hit: {}", cached);
//! }
//! # Ok(())
//! # }
//! ```

pub mod blob_store;
pub mod hot_cache;
pub mod metrics;
pub mod warm_store;

use crate::normalization::{ContentHash, VectorKey};
use anyhow::Result;
use std::path::PathBuf;
use std::sync::Arc;

pub use blob_store::BlobStore;
pub use hot_cache::LfuCache;
pub use metrics::{CacheMetrics, CacheStats};
pub use warm_store::WarmStore;

/// Cache configuration
#[derive(Debug, Clone)]
pub struct CacheConfig {
    /// Maximum size of hot cache in bytes
    pub hot_cache_size: usize,
    /// Path to warm store (mmap)
    pub warm_store_path: PathBuf,
    /// Path to cold store (blobs)
    pub cold_store_path: PathBuf,
    /// Compression level (1-22, higher = better compression)
    pub compression_level: i32,
    /// Enable cache metrics
    pub enable_metrics: bool,
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            hot_cache_size: 100 * 1024 * 1024, // 100 MB
            warm_store_path: PathBuf::from("data/cache/warm"),
            cold_store_path: PathBuf::from("data/cache/cold"),
            compression_level: 3, // Balanced compression
            enable_metrics: true,
        }
    }
}

/// Multi-tier cache manager
pub struct CacheManager {
    /// Hot cache (Tier 1 - Memory)
    hot_cache: Arc<LfuCache<ContentHash, String>>,
    /// Warm store (Tier 2 - Mmap)
    warm_store: Arc<WarmStore>,
    /// Cold store (Tier 3 - Compressed)
    cold_store: Arc<BlobStore>,
    /// Cache metrics
    metrics: Arc<CacheMetrics>,
    /// Configuration
    config: CacheConfig,
}

impl CacheManager {
    /// Create a new cache manager
    pub fn new(config: CacheConfig) -> Result<Self> {
        let hot_cache = Arc::new(LfuCache::new(config.hot_cache_size));
        let warm_store = Arc::new(WarmStore::new(&config.warm_store_path)?);
        let cold_store = Arc::new(BlobStore::new(
            &config.cold_store_path,
            config.compression_level,
        )?);
        let metrics = Arc::new(CacheMetrics::new(config.enable_metrics));

        Ok(Self {
            hot_cache,
            warm_store,
            cold_store,
            metrics,
            config,
        })
    }

    /// Get normalized text from cache
    pub async fn get_normalized(&self, hash: &ContentHash) -> Result<Option<String>> {
        // 1. Check hot cache
        if let Some(text) = self.hot_cache.get(hash) {
            self.metrics.record_hit("hot");
            return Ok(Some(text));
        }

        // 2. Check warm store
        if let Some(text) = self.warm_store.get(hash).await? {
            self.metrics.record_hit("warm");
            // Promote to hot cache
            self.hot_cache.put(*hash, text.clone());
            return Ok(Some(text));
        }

        // 3. Check cold store
        if let Some(data) = self.cold_store.get(hash).await? {
            self.metrics.record_hit("cold");
            let text = String::from_utf8(data)?;
            // Promote to warm and hot
            self.warm_store.put(*hash, &text).await?;
            self.hot_cache.put(*hash, text.clone());
            return Ok(Some(text));
        }

        self.metrics.record_miss();
        Ok(None)
    }

    /// Store normalized text in cache
    pub async fn put_normalized(&mut self, hash: ContentHash, text: &str) -> Result<()> {
        // Store in all tiers
        self.hot_cache.put(hash, text.to_string());
        self.warm_store.put(hash, text).await?;
        self.cold_store.put(hash, text.as_bytes()).await?;

        self.metrics.record_write();
        Ok(())
    }

    /// Get cache statistics
    pub fn stats(&self) -> CacheStats {
        self.metrics.stats()
    }

    /// Clear all caches
    pub async fn clear(&mut self) -> Result<()> {
        self.hot_cache.clear();
        self.warm_store.clear().await?;
        self.cold_store.clear().await?;
        Ok(())
    }

    /// Get cache size in bytes
    pub fn size(&self) -> usize {
        self.hot_cache.size()
    }

    /// Evict least frequently used items
    pub fn evict_lfu(&mut self, count: usize) -> usize {
        self.hot_cache.evict(count)
    }
}

#[cfg(test)]
mod tests;

#[cfg(test)]
mod unit_tests {
    use super::*;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_cache_manager_basic() {
        let dir = tempdir().unwrap();
        let config = CacheConfig {
            warm_store_path: dir.path().join("warm"),
            cold_store_path: dir.path().join("cold"),
            ..Default::default()
        };

        let mut cache = CacheManager::new(config).unwrap();

        let hash = ContentHash::from_bytes([1u8; 32]);
        let text = "Test normalized text";

        // Put in cache
        cache.put_normalized(hash, text).await.unwrap();

        // Get from cache (should hit hot cache)
        let retrieved = cache.get_normalized(&hash).await.unwrap();
        assert_eq!(retrieved.as_deref(), Some(text));

        // Clear hot cache and try again (should hit warm)
        cache.hot_cache.clear();
        let retrieved = cache.get_normalized(&hash).await.unwrap();
        assert_eq!(retrieved.as_deref(), Some(text));
    }

    #[tokio::test]
    async fn test_cache_miss() {
        let dir = tempdir().unwrap();
        let config = CacheConfig {
            warm_store_path: dir.path().join("warm"),
            cold_store_path: dir.path().join("cold"),
            ..Default::default()
        };

        let cache = CacheManager::new(config).unwrap();

        let hash = ContentHash::from_bytes([99u8; 32]);
        let result = cache.get_normalized(&hash).await.unwrap();
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn test_cache_stats() {
        let dir = tempdir().unwrap();
        let config = CacheConfig {
            warm_store_path: dir.path().join("warm"),
            cold_store_path: dir.path().join("cold"),
            ..Default::default()
        };

        let mut cache = CacheManager::new(config).unwrap();

        let hash = ContentHash::from_bytes([2u8; 32]);
        cache.put_normalized(hash, "test").await.unwrap();

        // Hit
        let _ = cache.get_normalized(&hash).await;

        // Miss
        let other_hash = ContentHash::from_bytes([3u8; 32]);
        let _ = cache.get_normalized(&other_hash).await;

        let stats = cache.stats();
        assert_eq!(stats.total_hits, 1);
        assert_eq!(stats.total_misses, 1);
        assert_eq!(stats.total_writes, 1);
    }
}

