//! Integration tests for cache system

#[cfg(test)]
mod integration_tests {
    use crate::normalization::cache::{CacheConfig, CacheManager};
    use crate::normalization::ContentHash;
    use tempfile::tempdir;
    use tokio::time::{sleep, Duration};

    #[tokio::test]
    async fn test_multi_tier_cache_flow() {
        let dir = tempdir().unwrap();
        let config = CacheConfig {
            hot_cache_size: 1024,
            warm_store_path: dir.path().join("warm"),
            cold_store_path: dir.path().join("cold"),
            compression_level: 3,
            enable_metrics: true,
        };

        let mut cache = CacheManager::new(config).unwrap();

        let hash = ContentHash::from_bytes([1u8; 32]);
        let text = "Multi-tier test content";

        // Store in cache (should go to all tiers)
        cache.put_normalized(hash, text).await.unwrap();

        // Retrieve (should hit hot cache)
        let result = cache.get_normalized(&hash).await.unwrap();
        assert_eq!(result.as_deref(), Some(text));

        let stats = cache.stats();
        assert_eq!(stats.hot_hits, 1);

        // Clear hot cache and retrieve (should hit warm)
        cache.hot_cache.clear();
        let result = cache.get_normalized(&hash).await.unwrap();
        assert_eq!(result.as_deref(), Some(text));

        let stats = cache.stats();
        assert_eq!(stats.warm_hits, 1);

        // Clear warm and retrieve (should hit cold)
        cache.warm_store.clear().await.unwrap();
        let result = cache.get_normalized(&hash).await.unwrap();
        assert_eq!(result.as_deref(), Some(text));

        let stats = cache.stats();
        assert_eq!(stats.cold_hits, 1);
    }

    #[tokio::test]
    async fn test_concurrent_cache_access() {
        let dir = tempdir().unwrap();
        let config = CacheConfig {
            warm_store_path: dir.path().join("warm"),
            cold_store_path: dir.path().join("cold"),
            ..Default::default()
        };

        let cache = CacheManager::new(config).unwrap();
        let cache = std::sync::Arc::new(tokio::sync::RwLock::new(cache));

        // Spawn multiple concurrent tasks
        let mut handles = vec![];

        for i in 0..10 {
            let cache_clone = cache.clone();
            let handle = tokio::spawn(async move {
                let mut hash_bytes = [0u8; 32];
                hash_bytes[0] = i;
                let hash = ContentHash::from_bytes(hash_bytes);
                let text = format!("Concurrent text {}", i);

                // Write
                {
                    let mut cache = cache_clone.write().await;
                    cache.put_normalized(hash, &text).await.unwrap();
                }

                // Small delay
                sleep(Duration::from_millis(10)).await;

                // Read
                {
                    let cache = cache_clone.read().await;
                    let result = cache.get_normalized(&hash).await.unwrap();
                    assert_eq!(result.as_deref(), Some(text.as_str()));
                }
            });

            handles.push(handle);
        }

        // Wait for all tasks
        for handle in handles {
            handle.await.unwrap();
        }
    }

    #[tokio::test]
    async fn test_cache_eviction_lfu() {
        let dir = tempdir().unwrap();
        let config = CacheConfig {
            hot_cache_size: 500, // Small size to force eviction
            warm_store_path: dir.path().join("warm"),
            cold_store_path: dir.path().join("cold"),
            compression_level: 3,
            enable_metrics: true,
        };

        let mut cache = CacheManager::new(config).unwrap();

        // Add multiple entries
        for i in 0..10u8 {
            let mut hash_bytes = [0u8; 32];
            hash_bytes[0] = i;
            let hash = ContentHash::from_bytes(hash_bytes);
            cache
                .put_normalized(hash, &format!("Entry {}", i))
                .await
                .unwrap();
        }

        // Access some entries more frequently
        let mut hash_bytes = [0u8; 32];
        hash_bytes[0] = 0;
        let popular_hash = ContentHash::from_bytes(hash_bytes);

        for _ in 0..10 {
            let _ = cache.get_normalized(&popular_hash).await;
        }

        // Manual eviction
        let evicted = cache.evict_lfu(5);
        assert!(evicted > 0);

        // Popular entry should still be in hot cache
        cache.hot_cache.get(&popular_hash).is_some();
    }

    #[tokio::test]
    async fn test_cache_persistence_across_restart() {
        let dir = tempdir().unwrap();

        let hash = ContentHash::from_bytes([42u8; 32]);
        let text = "Persistent cache data";

        // First instance: write data
        {
            let config = CacheConfig {
                warm_store_path: dir.path().join("warm"),
                cold_store_path: dir.path().join("cold"),
                ..Default::default()
            };

            let mut cache = CacheManager::new(config).unwrap();
            cache.put_normalized(hash, text).await.unwrap();
        }

        // Second instance: read data (simulating restart)
        {
            let config = CacheConfig {
                warm_store_path: dir.path().join("warm"),
                cold_store_path: dir.path().join("cold"),
                ..Default::default()
            };

            let cache = CacheManager::new(config).unwrap();

            // Should be able to retrieve from warm/cold store
            let result = cache.get_normalized(&hash).await.unwrap();
            assert_eq!(result.as_deref(), Some(text));
        }
    }

    #[tokio::test]
    async fn test_cache_metrics_accuracy() {
        let dir = tempdir().unwrap();
        let config = CacheConfig {
            warm_store_path: dir.path().join("warm"),
            cold_store_path: dir.path().join("cold"),
            enable_metrics: true,
            ..Default::default()
        };

        let mut cache = CacheManager::new(config).unwrap();

        // Perform various operations
        let hash1 = ContentHash::from_bytes([1u8; 32]);
        let hash2 = ContentHash::from_bytes([2u8; 32]);

        // Writes
        cache.put_normalized(hash1, "text1").await.unwrap();
        cache.put_normalized(hash2, "text2").await.unwrap();

        // Hits
        let _ = cache.get_normalized(&hash1).await;
        let _ = cache.get_normalized(&hash2).await;

        // Misses
        let hash3 = ContentHash::from_bytes([3u8; 32]);
        let _ = cache.get_normalized(&hash3).await;

        let stats = cache.stats();

        assert_eq!(stats.total_writes, 2);
        assert_eq!(stats.total_hits, 2);
        assert_eq!(stats.total_misses, 1);
        assert!((stats.hit_rate - 0.666).abs() < 0.01);
    }

    #[tokio::test]
    async fn test_cache_compression_effectiveness() {
        let dir = tempdir().unwrap();
        let config = CacheConfig {
            warm_store_path: dir.path().join("warm"),
            cold_store_path: dir.path().join("cold"),
            compression_level: 10, // High compression
            enable_metrics: true,
        };

        let mut cache = CacheManager::new(config).unwrap();

        // Store highly compressible data
        let hash = ContentHash::from_bytes([99u8; 32]);
        let text = "a".repeat(10000);

        cache.put_normalized(hash, &text).await.unwrap();

        // Check compression stats
        let stats = cache.cold_store.compression_stats();

        assert!(stats.compression_ratio > 5.0); // Should compress very well
        assert!(stats.space_saved > 8000);
    }

    #[tokio::test]
    async fn test_cache_clear_all_tiers() {
        let dir = tempdir().unwrap();
        let config = CacheConfig {
            warm_store_path: dir.path().join("warm"),
            cold_store_path: dir.path().join("cold"),
            ..Default::default()
        };

        let mut cache = CacheManager::new(config).unwrap();

        // Add data
        for i in 0..5u8 {
            let mut hash_bytes = [0u8; 32];
            hash_bytes[0] = i;
            let hash = ContentHash::from_bytes(hash_bytes);
            cache
                .put_normalized(hash, &format!("Data {}", i))
                .await
                .unwrap();
        }

        // Clear all
        cache.clear().await.unwrap();

        // Verify all cleared
        for i in 0..5u8 {
            let mut hash_bytes = [0u8; 32];
            hash_bytes[0] = i;
            let hash = ContentHash::from_bytes(hash_bytes);
            let result = cache.get_normalized(&hash).await.unwrap();
            assert!(result.is_none());
        }
    }
}

