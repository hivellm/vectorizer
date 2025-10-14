//! Hot cache implementation using LFU (Least Frequently Used) eviction
//!
//! This is the first tier of the cache hierarchy, storing recently accessed
//! normalized text in memory for ultra-fast access.

use parking_lot::RwLock;
use std::collections::HashMap;
use std::hash::Hash;
use std::sync::Arc;

/// Entry in LFU cache with frequency tracking
struct CacheEntry<V> {
    value: V,
    frequency: usize,
    size: usize,
}

/// LFU (Least Frequently Used) cache
pub struct LfuCache<K, V>
where
    K: Hash + Eq + Clone,
    V: Clone,
{
    cache: Arc<RwLock<HashMap<K, CacheEntry<V>>>>,
    max_size: usize,
    current_size: Arc<RwLock<usize>>,
}

impl<K, V> LfuCache<K, V>
where
    K: Hash + Eq + Clone,
    V: Clone,
{
    /// Create a new LFU cache with maximum size in bytes
    pub fn new(max_size: usize) -> Self {
        Self {
            cache: Arc::new(RwLock::new(HashMap::new())),
            max_size,
            current_size: Arc::new(RwLock::new(0)),
        }
    }

    /// Get value from cache, incrementing frequency
    pub fn get(&self, key: &K) -> Option<V> {
        let mut cache = self.cache.write();

        if let Some(entry) = cache.get_mut(key) {
            entry.frequency += 1;
            Some(entry.value.clone())
        } else {
            None
        }
    }

    /// Put value into cache
    pub fn put(&self, key: K, value: V) {
        let value_size = std::mem::size_of_val(&value);

        let mut cache = self.cache.write();
        let mut current_size = self.current_size.write();

        // If key exists, update it
        if let Some(entry) = cache.get_mut(&key) {
            let old_size = entry.size;
            entry.value = value;
            entry.size = value_size;
            entry.frequency += 1;
            *current_size = current_size.saturating_sub(old_size) + value_size;
            return;
        }

        // Evict if necessary
        while *current_size + value_size > self.max_size && !cache.is_empty() {
            // Find least frequently used
            let lfu_key = cache
                .iter()
                .min_by_key(|(_, entry)| entry.frequency)
                .map(|(k, _)| k.clone());

            if let Some(key_to_remove) = lfu_key {
                if let Some(removed) = cache.remove(&key_to_remove) {
                    *current_size = current_size.saturating_sub(removed.size);
                }
            } else {
                break;
            }
        }

        // Insert new entry
        cache.insert(
            key,
            CacheEntry {
                value,
                frequency: 1,
                size: value_size,
            },
        );
        *current_size += value_size;
    }

    /// Remove value from cache
    pub fn remove(&self, key: &K) -> Option<V> {
        let mut cache = self.cache.write();
        let mut current_size = self.current_size.write();

        cache.remove(key).map(|entry| {
            *current_size = current_size.saturating_sub(entry.size);
            entry.value
        })
    }

    /// Clear all entries
    pub fn clear(&self) {
        let mut cache = self.cache.write();
        let mut current_size = self.current_size.write();

        cache.clear();
        *current_size = 0;
    }

    /// Get current cache size in bytes
    pub fn size(&self) -> usize {
        *self.current_size.read()
    }

    /// Get number of entries
    pub fn len(&self) -> usize {
        self.cache.read().len()
    }

    /// Check if cache is empty
    pub fn is_empty(&self) -> bool {
        self.cache.read().is_empty()
    }

    /// Evict n least frequently used entries
    pub fn evict(&self, count: usize) -> usize {
        let mut cache = self.cache.write();
        let mut current_size = self.current_size.write();
        let mut evicted = 0;

        for _ in 0..count {
            if cache.is_empty() {
                break;
            }

            // Find least frequently used
            let lfu_key = cache
                .iter()
                .min_by_key(|(_, entry)| entry.frequency)
                .map(|(k, _)| k.clone());

            if let Some(key_to_remove) = lfu_key {
                if let Some(removed) = cache.remove(&key_to_remove) {
                    *current_size = current_size.saturating_sub(removed.size);
                    evicted += 1;
                }
            }
        }

        evicted
    }

    /// Get cache statistics
    pub fn stats(&self) -> LfuStats {
        let cache = self.cache.read();

        let total_frequency: usize = cache.values().map(|e| e.frequency).sum();
        let avg_frequency = if cache.is_empty() {
            0.0
        } else {
            total_frequency as f64 / cache.len() as f64
        };

        let max_frequency = cache.values().map(|e| e.frequency).max().unwrap_or(0);
        let min_frequency = cache.values().map(|e| e.frequency).min().unwrap_or(0);

        LfuStats {
            entries: cache.len(),
            total_size: *self.current_size.read(),
            max_size: self.max_size,
            avg_frequency,
            max_frequency,
            min_frequency,
        }
    }
}

/// LFU cache statistics
#[derive(Debug, Clone)]
pub struct LfuStats {
    pub entries: usize,
    pub total_size: usize,
    pub max_size: usize,
    pub avg_frequency: f64,
    pub max_frequency: usize,
    pub min_frequency: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lfu_basic() {
        let cache = LfuCache::new(1024);

        cache.put("key1", "value1".to_string());
        cache.put("key2", "value2".to_string());

        assert_eq!(cache.get(&"key1"), Some("value1".to_string()));
        assert_eq!(cache.get(&"key2"), Some("value2".to_string()));
        assert_eq!(cache.get(&"key3"), None);
    }

    #[test]
    fn test_lfu_eviction() {
        let cache = LfuCache::new(100);

        // Add entries
        for i in 0..10 {
            cache.put(format!("key{}", i), "x".repeat(20));
        }

        // Access some keys more frequently
        for _ in 0..5 {
            cache.get(&"key0".to_string());
        }

        // This should evict least frequently used
        cache.put("new_key".to_string(), "x".repeat(50));

        // key0 should still be there (high frequency)
        assert!(cache.get(&"key0".to_string()).is_some());
    }

    #[test]
    fn test_lfu_update() {
        let cache = LfuCache::new(1024);

        cache.put("key", "value1".to_string());
        cache.put("key", "value2".to_string());

        assert_eq!(cache.get(&"key"), Some("value2".to_string()));
        assert_eq!(cache.len(), 1);
    }

    #[test]
    fn test_lfu_clear() {
        let cache = LfuCache::new(1024);

        cache.put("key1", "value1".to_string());
        cache.put("key2", "value2".to_string());

        cache.clear();

        assert_eq!(cache.len(), 0);
        assert_eq!(cache.size(), 0);
    }

    #[test]
    fn test_lfu_remove() {
        let cache = LfuCache::new(1024);

        cache.put("key", "value".to_string());
        let removed = cache.remove(&"key");

        assert_eq!(removed, Some("value".to_string()));
        assert_eq!(cache.len(), 0);
    }

    #[test]
    fn test_lfu_stats() {
        let cache = LfuCache::new(1024);

        cache.put("key1", "value1".to_string());
        cache.put("key2", "value2".to_string());

        // Access key1 multiple times
        for _ in 0..5 {
            cache.get(&"key1");
        }

        let stats = cache.stats();
        assert_eq!(stats.entries, 2);
        assert!(stats.avg_frequency > 1.0);
    }

    #[test]
    fn test_lfu_frequency_tracking() {
        let cache = LfuCache::new(1024);

        cache.put("key", "value".to_string());

        // Access multiple times
        for _ in 0..10 {
            cache.get(&"key");
        }

        let stats = cache.stats();
        assert_eq!(stats.max_frequency, 11); // 1 from put + 10 from get
    }
}

