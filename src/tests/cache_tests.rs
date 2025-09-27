//! Tests for embedding cache functionality

use crate::embedding::{CacheConfig, EmbeddingCache};
use tempfile::tempdir;

#[test]
fn test_embedding_cache_initialization() {
    let temp_dir = tempdir().unwrap();
    let cache_dir = temp_dir.path().join("cache");

    let config = CacheConfig {
        cache_dir: cache_dir.clone(),
        max_size: 100 * 1024 * 1024, // 100MB
        use_mmap: false,
        prefix: "test".to_string(),
        num_shards: 4,
    };

    let _cache = EmbeddingCache::new(config).unwrap();
    assert!(cache_dir.exists());
}

#[test]
fn test_embedding_cache_miss() {
    let temp_dir = tempdir().unwrap();
    let cache_dir = temp_dir.path().join("cache");

    let config = CacheConfig {
        cache_dir,
        max_size: 100 * 1024 * 1024,
        use_mmap: true,
        prefix: "test".to_string(),
        num_shards: 4,
    };

    let cache = EmbeddingCache::new(config).unwrap();

    let text = "nonexistent text";
    let result = cache.get(text);
    assert!(result.is_none());
}

#[test]
fn test_embedding_cache_multiple_entries() {
    let temp_dir = tempdir().unwrap();
    let cache_dir = temp_dir.path().join("cache");

    let config = CacheConfig {
        cache_dir,
        max_size: 100 * 1024 * 1024,
        use_mmap: true,
        prefix: "test".to_string(),
        num_shards: 4,
    };

    let cache = EmbeddingCache::new(config).unwrap();

    let entries = vec![
        ("text1", vec![0.1, 0.2, 0.3]),
        ("text2", vec![0.4, 0.5, 0.6]),
        ("text3", vec![0.7, 0.8, 0.9]),
    ];

    // Put all entries
    for (text, embedding) in &entries {
        cache.put(text, embedding).unwrap();
    }

    // Verify all entries can be retrieved
    for (text, expected_embedding) in &entries {
        let retrieved = cache.get(text).unwrap();
        assert_eq!(retrieved, *expected_embedding);
    }
}

#[test]
fn test_embedding_cache_persistence() {
    let temp_dir = tempdir().unwrap();
    let cache_dir = temp_dir.path().join("cache");

    let config1 = CacheConfig {
        cache_dir: cache_dir.clone(),
        max_size: 100 * 1024 * 1024,
        use_mmap: false,
        prefix: "test".to_string(),
        num_shards: 4,
    };

    let cache1 = EmbeddingCache::new(config1).unwrap();
    let text = "persistent text";
    let embedding = vec![0.1, 0.2, 0.3, 0.4, 0.5];

    // Put in first cache instance
    cache1.put(text, &embedding).unwrap();

    // Drop first cache to ensure data is persisted
    drop(cache1);

    // Create new cache instance with same config
    let config2 = CacheConfig {
        cache_dir,
        max_size: 100 * 1024 * 1024,
        use_mmap: false,
        prefix: "test".to_string(),
        num_shards: 4,
    };

    let cache2 = EmbeddingCache::new(config2).unwrap();

    // Verify data persists across instances
    let retrieved = cache2.get(text).unwrap();
    assert_eq!(retrieved, embedding);
}
