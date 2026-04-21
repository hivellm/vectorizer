//! Benchmarks for query cache performance

use criterion::{Criterion, black_box, criterion_group, criterion_main};
use serde_json::json;
use vectorizer::cache::{QueryCache, QueryCacheConfig, QueryKey};

fn benchmark_cache_insert(c: &mut Criterion) {
    let config = QueryCacheConfig::default();
    let cache: QueryCache<serde_json::Value> = QueryCache::new(config);

    c.bench_function("cache_insert", |b| {
        b.iter(|| {
            let key = QueryKey::new(
                black_box("test_collection".to_string()),
                black_box("test query".to_string()),
                black_box(10),
                black_box(Some(0.5)),
            );
            let value = json!({
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
            cache.insert(key, value);
        })
    });
}

fn benchmark_cache_get(c: &mut Criterion) {
    let config = QueryCacheConfig::default();
    let cache: QueryCache<serde_json::Value> = QueryCache::new(config);

    // Pre-populate cache
    for i in 0..100 {
        let key = QueryKey::new(
            format!("collection_{}", i % 10),
            format!("query_{i}"),
            10,
            Some(0.5),
        );
        let value = json!({
            "results": [{"id": format!("id_{}", i), "score": 0.95}],
            "query": format!("query_{i}"),
            "limit": 10,
            "collection": format!("collection_{}", i % 10)
        });
        cache.insert(key, value);
    }

    c.bench_function("cache_get", |b| {
        b.iter(|| {
            let key = QueryKey::new(
                black_box("collection_5".to_string()),
                black_box("query_50".to_string()),
                black_box(10),
                black_box(Some(0.5)),
            );
            let _result = cache.get(&key);
        })
    });
}

fn benchmark_cache_miss(c: &mut Criterion) {
    let config = QueryCacheConfig::default();
    let cache: QueryCache<serde_json::Value> = QueryCache::new(config);

    c.bench_function("cache_miss", |b| {
        b.iter(|| {
            let key = QueryKey::new(
                black_box("nonexistent_collection".to_string()),
                black_box("nonexistent query".to_string()),
                black_box(10),
                black_box(Some(0.5)),
            );
            let _result = cache.get(&key);
        })
    });
}

fn benchmark_cache_invalidation(c: &mut Criterion) {
    let config = QueryCacheConfig::default();
    let cache: QueryCache<serde_json::Value> = QueryCache::new(config);

    // Pre-populate cache with multiple collections
    for i in 0..100 {
        let key = QueryKey::new(
            format!("collection_{}", i % 5),
            format!("query_{i}"),
            10,
            Some(0.5),
        );
        let value = json!({
            "results": [{"id": format!("id_{}", i), "score": 0.95}],
            "query": format!("query_{i}"),
            "limit": 10,
            "collection": format!("collection_{}", i % 5)
        });
        cache.insert(key, value);
    }

    c.bench_function("cache_invalidation", |b| {
        b.iter(|| {
            cache.invalidate_collection(black_box("collection_2"));
        })
    });
}

fn benchmark_cache_stats(c: &mut Criterion) {
    let config = QueryCacheConfig::default();
    let cache: QueryCache<serde_json::Value> = QueryCache::new(config);

    // Pre-populate cache
    for i in 0..100 {
        let key = QueryKey::new(
            format!("collection_{}", i % 10),
            format!("query_{i}"),
            10,
            Some(0.5),
        );
        let value = json!({
            "results": [{"id": format!("id_{}", i), "score": 0.95}],
            "query": format!("query_{i}"),
            "limit": 10,
            "collection": format!("collection_{}", i % 10)
        });
        cache.insert(key, value);
    }

    c.bench_function("cache_stats", |b| {
        b.iter(|| {
            let _stats = cache.stats();
        })
    });
}

criterion_group!(
    benches,
    benchmark_cache_insert,
    benchmark_cache_get,
    benchmark_cache_miss,
    benchmark_cache_invalidation,
    benchmark_cache_stats
);
criterion_main!(benches);
