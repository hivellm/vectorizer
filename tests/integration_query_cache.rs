//! Integration tests for query cache functionality

use serde_json::json;
use vectorizer::cache::QueryCache;

#[tokio::test]
async fn test_query_cache_integration() {
    // This test would require a full server setup
    // For now, we'll test the cache functionality directly

    let config = vectorizer::cache::QueryCacheConfig::default();
    let cache: QueryCache<serde_json::Value> = QueryCache::new(config);

    // Test cache key creation
    let key = vectorizer::cache::QueryKey::new(
        "test_collection".to_string(),
        "test query".to_string(),
        10,
        Some(0.5),
    );

    // Test cache insertion and retrieval
    let test_result = json!({
        "results": [
            {
                "id": "test_id",
                "score": 0.95,
                "vector": [0.1, 0.2, 0.3],
                "payload": {"text": "test"}
            }
        ],
        "query": "test query",
        "limit": 10,
        "collection": "test_collection"
    });

    cache.insert(key.clone(), test_result.clone());

    let cached_result = cache.get(&key);
    assert!(cached_result.is_some());
    assert_eq!(cached_result.unwrap(), test_result);

    // Test cache statistics
    let stats = cache.stats();
    assert_eq!(stats.size, 1);
    assert_eq!(stats.hits, 1);
    assert_eq!(stats.misses, 0);
    assert_eq!(stats.hit_rate, 1.0);
}

#[tokio::test]
async fn test_cache_invalidation() {
    let config = vectorizer::cache::QueryCacheConfig::default();
    let cache: QueryCache<serde_json::Value> = QueryCache::new(config);

    // Insert multiple entries for different collections
    let key1 =
        vectorizer::cache::QueryKey::new("collection1".to_string(), "query1".to_string(), 10, None);
    let key2 =
        vectorizer::cache::QueryKey::new("collection2".to_string(), "query2".to_string(), 10, None);

    cache.insert(key1.clone(), json!({"result": "collection1"}));
    cache.insert(key2.clone(), json!({"result": "collection2"}));

    // Verify both entries exist
    assert!(cache.get(&key1).is_some());
    assert!(cache.get(&key2).is_some());

    // Invalidate collection1
    cache.invalidate_collection("collection1");

    // Verify collection1 is invalidated but collection2 remains
    assert!(cache.get(&key1).is_none());
    assert!(cache.get(&key2).is_some());
}

#[tokio::test]
async fn test_cache_ttl_expiration() {
    let config = vectorizer::cache::QueryCacheConfig {
        max_size: 1000,
        ttl_seconds: 0, // Expire immediately for testing
        warmup_enabled: false,
    };
    let cache: QueryCache<serde_json::Value> = QueryCache::new(config);

    let key = vectorizer::cache::QueryKey::new("test".to_string(), "query".to_string(), 10, None);

    cache.insert(key.clone(), json!({"result": "test"}));

    // Wait a bit for expiration
    tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;

    // Entry should be expired
    assert!(cache.get(&key).is_none());
}

#[tokio::test]
async fn test_cache_lru_eviction() {
    let config = vectorizer::cache::QueryCacheConfig {
        max_size: 2,
        ttl_seconds: 300,
        warmup_enabled: false,
    };
    let cache: QueryCache<serde_json::Value> = QueryCache::new(config);

    let key1 = vectorizer::cache::QueryKey::new("test".to_string(), "query1".to_string(), 10, None);
    let key2 = vectorizer::cache::QueryKey::new("test".to_string(), "query2".to_string(), 10, None);
    let key3 = vectorizer::cache::QueryKey::new("test".to_string(), "query3".to_string(), 10, None);

    cache.insert(key1.clone(), json!({"result": "1"}));
    cache.insert(key2.clone(), json!({"result": "2"}));
    cache.insert(key3.clone(), json!({"result": "3"}));

    let stats = cache.stats();
    assert_eq!(stats.size, 2);
    assert_eq!(stats.evictions, 1);

    // key1 should be evicted (least recently used)
    assert!(cache.get(&key1).is_none());
    assert!(cache.get(&key2).is_some());
    assert!(cache.get(&key3).is_some());
}
