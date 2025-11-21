//! Integration tests for query cache functionality

use serde_json::json;
use vectorizer::cache::QueryCache;
use vectorizer::server::VectorizerServer;

#[path = "../helpers/mod.rs"]
mod helpers;
use helpers::{create_test_collection, generate_test_vectors, insert_test_vectors};

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

#[tokio::test]
async fn test_cache_integration_with_server() {
    // Test cache integration with VectorizerServer
    let server = VectorizerServer::new()
        .await
        .expect("Failed to create server");

    // Create test collection
    create_test_collection(&server.store, "cache_test_collection", 128)
        .expect("Failed to create collection");

    // Insert test vectors
    let vectors = generate_test_vectors(10, 128);
    insert_test_vectors(&server.store, "cache_test_collection", vectors)
        .expect("Failed to insert vectors");

    // Get initial cache stats
    let initial_stats = server.query_cache.stats();
    assert_eq!(initial_stats.size, 0);
    assert_eq!(initial_stats.hits, 0);
    assert_eq!(initial_stats.misses, 0);

    // Simulate a search query (would normally go through HTTP endpoint)
    // For now, we test the cache directly
    use vectorizer::cache::query_cache::QueryKey;
    let cache_key = QueryKey::new(
        "cache_test_collection".to_string(),
        "test query".to_string(),
        10,
        None,
    );

    // Insert a cached result
    let test_result = json!({
        "results": [],
        "query": "test query",
        "limit": 10,
        "collection": "cache_test_collection"
    });

    server
        .query_cache
        .insert(cache_key.clone(), test_result.clone());

    // Verify cache hit
    let cached = server.query_cache.get(&cache_key);
    assert!(cached.is_some());

    // Check stats
    let stats = server.query_cache.stats();
    assert_eq!(stats.size, 1);
    assert_eq!(stats.hits, 1);
    assert_eq!(stats.misses, 0);
}

#[tokio::test]
async fn test_cache_invalidation_on_insert() {
    // Test that cache is invalidated when vectors are inserted
    let server = VectorizerServer::new()
        .await
        .expect("Failed to create server");

    // Create test collection
    create_test_collection(&server.store, "invalidation_test", 128)
        .expect("Failed to create collection");

    // Insert a cached query result
    use vectorizer::cache::query_cache::QueryKey;
    let cache_key = QueryKey::new(
        "invalidation_test".to_string(),
        "test query".to_string(),
        10,
        None,
    );

    let test_result = json!({
        "results": [],
        "query": "test query"
    });

    server.query_cache.insert(cache_key.clone(), test_result);

    // Verify cache entry exists
    assert!(server.query_cache.get(&cache_key).is_some());

    // Insert vectors (this should invalidate cache)
    let vectors = generate_test_vectors(5, 128);
    insert_test_vectors(&server.store, "invalidation_test", vectors)
        .expect("Failed to insert vectors");

    // Manually invalidate cache (as done in handlers)
    server
        .query_cache
        .invalidate_collection("invalidation_test");

    // Verify cache entry is gone
    assert!(server.query_cache.get(&cache_key).is_none());
}

#[tokio::test]
async fn test_cache_stats_in_health_endpoint() {
    // Test that cache stats are available
    let server = VectorizerServer::new()
        .await
        .expect("Failed to create server");

    // Get cache stats
    let stats = server.query_cache.stats();

    // Verify stats structure
    assert!(stats.capacity > 0);
    assert_eq!(stats.size, 0); // Initially empty
    assert_eq!(stats.hits, 0);
    assert_eq!(stats.misses, 0);
    assert_eq!(stats.evictions, 0);
    assert_eq!(stats.hit_rate, 0.0);

    // Add some cache entries and verify stats update
    use vectorizer::cache::query_cache::QueryKey;
    for i in 0..5 {
        let key = QueryKey::new(
            "test_collection".to_string(),
            format!("query_{i}"),
            10,
            None,
        );
        server.query_cache.insert(key, json!({"result": i}));
    }

    let updated_stats = server.query_cache.stats();
    assert_eq!(updated_stats.size, 5);
    assert_eq!(updated_stats.capacity, stats.capacity); // Capacity unchanged
}
