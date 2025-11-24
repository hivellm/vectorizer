//! Query cache implementation using LRU eviction policy.

use std::hash::{Hash, Hasher};
use std::num::NonZeroUsize;
use std::sync::Arc;
use std::time::{Duration, Instant};

use lru::LruCache;
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};

/// Configuration for query cache
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryCacheConfig {
    /// Maximum number of queries to cache
    pub max_size: usize,
    /// Time-to-live for cached entries (in seconds)
    pub ttl_seconds: u64,
    /// Enable cache warmup on startup
    pub warmup_enabled: bool,
}

impl Default for QueryCacheConfig {
    fn default() -> Self {
        Self {
            max_size: 1000,
            ttl_seconds: 300, // 5 minutes
            warmup_enabled: false,
        }
    }
}

/// Key for caching queries
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QueryKey {
    /// Collection name
    pub collection: String,
    /// Query text
    pub query: String,
    /// Limit (max results)
    pub limit: usize,
    /// Similarity threshold
    pub threshold: Option<u32>, // Store as u32 (f64 * 1000) for hashing
}

impl QueryKey {
    /// Create a new query key
    pub fn new(collection: String, query: String, limit: usize, threshold: Option<f64>) -> Self {
        Self {
            collection,
            query,
            limit,
            threshold: threshold.map(|t| (t * 1000.0) as u32),
        }
    }

    /// Get threshold as f64
    pub fn threshold_f64(&self) -> Option<f64> {
        self.threshold.map(|t| t as f64 / 1000.0)
    }
}

impl Hash for QueryKey {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.collection.hash(state);
        self.query.hash(state);
        self.limit.hash(state);
        self.threshold.hash(state);
    }
}

/// Cached entry with TTL
#[derive(Debug, Clone)]
struct CachedEntry<T> {
    value: T,
    created_at: Instant,
    ttl: Duration,
}

impl<T> CachedEntry<T> {
    fn new(value: T, ttl: Duration) -> Self {
        Self {
            value,
            created_at: Instant::now(),
            ttl,
        }
    }

    fn is_expired(&self) -> bool {
        self.created_at.elapsed() > self.ttl
    }
}

/// Thread-safe LRU query cache
pub struct QueryCache<T: Clone> {
    cache: Arc<RwLock<LruCache<QueryKey, CachedEntry<T>>>>,
    ttl: Duration,
    hits: Arc<parking_lot::Mutex<u64>>,
    misses: Arc<parking_lot::Mutex<u64>>,
    evictions: Arc<parking_lot::Mutex<u64>>,
}

impl<T: Clone> QueryCache<T> {
    /// Create a new query cache with configuration
    pub fn new(config: QueryCacheConfig) -> Self {
        let capacity =
            NonZeroUsize::new(config.max_size).unwrap_or(NonZeroUsize::new(1000).unwrap());

        Self {
            cache: Arc::new(RwLock::new(LruCache::new(capacity))),
            ttl: Duration::from_secs(config.ttl_seconds),
            hits: Arc::new(parking_lot::Mutex::new(0)),
            misses: Arc::new(parking_lot::Mutex::new(0)),
            evictions: Arc::new(parking_lot::Mutex::new(0)),
        }
    }

    /// Get a cached query result
    pub fn get(&self, key: &QueryKey) -> Option<T> {
        let mut cache = self.cache.write();

        if let Some(entry) = cache.get(key) {
            if entry.is_expired() {
                // Entry expired, remove it
                cache.pop(key);
                *self.misses.lock() += 1;
                None
            } else {
                *self.hits.lock() += 1;
                Some(entry.value.clone())
            }
        } else {
            *self.misses.lock() += 1;
            None
        }
    }

    /// Insert a query result into the cache
    pub fn insert(&self, key: QueryKey, value: T) {
        let mut cache = self.cache.write();
        let entry = CachedEntry::new(value, self.ttl);

        if let Some(_evicted) = cache.push(key, entry) {
            *self.evictions.lock() += 1;
        }
    }

    /// Invalidate cache entries for a specific collection
    pub fn invalidate_collection(&self, collection: &str) {
        let mut cache = self.cache.write();
        let keys_to_remove: Vec<QueryKey> = cache
            .iter()
            .filter(|(k, _)| k.collection == collection)
            .map(|(k, _)| k.clone())
            .collect();

        for key in keys_to_remove {
            cache.pop(&key);
        }
    }

    /// Clear all cached entries
    pub fn clear(&self) {
        let mut cache = self.cache.write();
        cache.clear();
    }

    /// Get cache statistics
    pub fn stats(&self) -> CacheStats {
        let cache = self.cache.read();
        let hits = *self.hits.lock();
        let misses = *self.misses.lock();
        let evictions = *self.evictions.lock();

        CacheStats {
            size: cache.len(),
            capacity: cache.cap().get(),
            hits,
            misses,
            evictions,
            hit_rate: if hits + misses > 0 {
                hits as f64 / (hits + misses) as f64
            } else {
                0.0
            },
        }
    }

    /// Reset statistics
    pub fn reset_stats(&self) {
        *self.hits.lock() = 0;
        *self.misses.lock() = 0;
        *self.evictions.lock() = 0;
    }
}

/// Cache statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheStats {
    /// Current number of entries
    pub size: usize,
    /// Maximum capacity
    pub capacity: usize,
    /// Number of cache hits
    pub hits: u64,
    /// Number of cache misses
    pub misses: u64,
    /// Number of evictions
    pub evictions: u64,
    /// Hit rate (0.0 to 1.0)
    pub hit_rate: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_query_cache_creation() {
        let config = QueryCacheConfig::default();
        let cache: QueryCache<Vec<String>> = QueryCache::new(config);
        let stats = cache.stats();

        assert_eq!(stats.size, 0);
        assert_eq!(stats.capacity, 1000);
        assert_eq!(stats.hits, 0);
        assert_eq!(stats.misses, 0);
    }

    #[test]
    fn test_cache_insert_and_get() {
        let config = QueryCacheConfig::default();
        let cache: QueryCache<Vec<String>> = QueryCache::new(config);

        let key = QueryKey::new("test".to_string(), "hello".to_string(), 10, Some(0.5));
        let value = vec!["result1".to_string(), "result2".to_string()];

        cache.insert(key.clone(), value.clone());

        let retrieved = cache.get(&key);
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap(), value);

        let stats = cache.stats();
        assert_eq!(stats.hits, 1);
        assert_eq!(stats.misses, 0);
    }

    #[test]
    fn test_cache_miss() {
        let config = QueryCacheConfig::default();
        let cache: QueryCache<Vec<String>> = QueryCache::new(config);

        let key = QueryKey::new("test".to_string(), "hello".to_string(), 10, None);

        let retrieved = cache.get(&key);
        assert!(retrieved.is_none());

        let stats = cache.stats();
        assert_eq!(stats.hits, 0);
        assert_eq!(stats.misses, 1);
    }

    #[test]
    fn test_cache_eviction() {
        let config = QueryCacheConfig {
            max_size: 2,
            ttl_seconds: 300,
            warmup_enabled: false,
        };
        let cache: QueryCache<Vec<String>> = QueryCache::new(config);

        let key1 = QueryKey::new("test".to_string(), "query1".to_string(), 10, None);
        let key2 = QueryKey::new("test".to_string(), "query2".to_string(), 10, None);
        let key3 = QueryKey::new("test".to_string(), "query3".to_string(), 10, None);

        cache.insert(key1.clone(), vec!["result1".to_string()]);
        cache.insert(key2.clone(), vec!["result2".to_string()]);
        cache.insert(key3.clone(), vec!["result3".to_string()]);

        let stats = cache.stats();
        assert_eq!(stats.size, 2);
        assert_eq!(stats.evictions, 1);

        // key1 should be evicted (least recently used)
        assert!(cache.get(&key1).is_none());
        assert!(cache.get(&key2).is_some());
        assert!(cache.get(&key3).is_some());
    }

    #[test]
    fn test_cache_invalidation() {
        let config = QueryCacheConfig::default();
        let cache: QueryCache<Vec<String>> = QueryCache::new(config);

        let key1 = QueryKey::new("collection1".to_string(), "query1".to_string(), 10, None);
        let key2 = QueryKey::new("collection2".to_string(), "query2".to_string(), 10, None);

        cache.insert(key1.clone(), vec!["result1".to_string()]);
        cache.insert(key2.clone(), vec!["result2".to_string()]);

        cache.invalidate_collection("collection1");

        assert!(cache.get(&key1).is_none());
        assert!(cache.get(&key2).is_some());
    }

    #[test]
    fn test_cache_clear() {
        let config = QueryCacheConfig::default();
        let cache: QueryCache<Vec<String>> = QueryCache::new(config);

        let key1 = QueryKey::new("test".to_string(), "query1".to_string(), 10, None);
        let key2 = QueryKey::new("test".to_string(), "query2".to_string(), 10, None);

        cache.insert(key1.clone(), vec!["result1".to_string()]);
        cache.insert(key2.clone(), vec!["result2".to_string()]);

        cache.clear();

        let stats = cache.stats();
        assert_eq!(stats.size, 0);
    }

    #[test]
    fn test_cache_ttl_expiration() {
        let config = QueryCacheConfig {
            max_size: 1000,
            ttl_seconds: 0, // Expire immediately
            warmup_enabled: false,
        };
        let cache: QueryCache<Vec<String>> = QueryCache::new(config);

        let key = QueryKey::new("test".to_string(), "query".to_string(), 10, None);
        cache.insert(key.clone(), vec!["result".to_string()]);

        std::thread::sleep(Duration::from_millis(10));

        // Entry should be expired
        assert!(cache.get(&key).is_none());
    }

    #[test]
    fn test_query_key_hash() {
        let key1 = QueryKey::new("coll".to_string(), "query".to_string(), 10, Some(0.5));
        let key2 = QueryKey::new("coll".to_string(), "query".to_string(), 10, Some(0.5));
        let key3 = QueryKey::new("coll".to_string(), "query".to_string(), 20, Some(0.5));

        assert_eq!(key1, key2);
        assert_ne!(key1, key3);
    }

    #[test]
    fn test_hit_rate_calculation() {
        let config = QueryCacheConfig::default();
        let cache: QueryCache<Vec<String>> = QueryCache::new(config);

        let key = QueryKey::new("test".to_string(), "query".to_string(), 10, None);
        cache.insert(key.clone(), vec!["result".to_string()]);

        // 1 hit
        cache.get(&key);
        // 1 miss
        cache.get(&QueryKey::new(
            "test".to_string(),
            "other".to_string(),
            10,
            None,
        ));

        let stats = cache.stats();
        assert_eq!(stats.hit_rate, 0.5); // 1 hit out of 2 requests
    }
}
