//! End-to-end behavioural tests for the query cache.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

use vectorizer::cache::query_cache::{QueryCache, QueryCacheConfig, QueryKey};

fn cache() -> QueryCache<String> {
    QueryCache::new(QueryCacheConfig {
        max_size: 64,
        ttl_seconds: 60,
        warmup_enabled: false,
    })
}

#[test]
fn hit_rate_climbs_under_repeated_query_workload() {
    let cache = cache();
    let key = QueryKey::new("col".into(), "q".into(), 10, None);

    // First call is a miss (cache is empty), every subsequent call is a hit.
    cache.insert(key.clone(), "value".into());
    for _ in 0..50 {
        let v = cache.get(&key);
        assert_eq!(v.as_deref(), Some("value"));
    }
    let stats = cache.stats();
    assert_eq!(stats.hits, 50, "every repeat read after insert is a hit");
    assert_eq!(stats.misses, 0);
    assert!(
        stats.hit_rate > 0.99,
        "repeated-query hit_rate must approach 1.0; got {}",
        stats.hit_rate
    );
}

#[test]
fn invalidation_drops_only_targeted_collection() {
    let cache = cache();
    let key_a = QueryKey::new("alpha".into(), "q".into(), 10, None);
    let key_b = QueryKey::new("beta".into(), "q".into(), 10, None);

    cache.insert(key_a.clone(), "alpha-value".into());
    cache.insert(key_b.clone(), "beta-value".into());

    cache.invalidate_collection("alpha");

    assert!(
        cache.get(&key_a).is_none(),
        "alpha entry must be evicted by invalidate_collection"
    );
    assert_eq!(
        cache.get(&key_b).as_deref(),
        Some("beta-value"),
        "beta entry stays — invalidation is collection-scoped"
    );
}

#[test]
fn cached_or_compute_runs_compute_only_on_miss() {
    let cache = cache();
    let key = QueryKey::new("col".into(), "q".into(), 10, None);

    let compute_calls = Arc::new(AtomicUsize::new(0));
    let increments = Arc::clone(&compute_calls);

    let make_compute = || -> Result<String, ()> {
        increments.fetch_add(1, Ordering::SeqCst);
        Ok("computed-value".to_string())
    };

    // First call: cache empty → compute runs once.
    let v1 = cache
        .cached_or_compute(key.clone(), make_compute)
        .expect("first call");
    assert_eq!(v1, "computed-value");
    assert_eq!(compute_calls.load(Ordering::SeqCst), 1);

    // Subsequent calls: cache hit → compute does NOT run.
    for _ in 0..10 {
        let v = cache
            .cached_or_compute(key.clone(), make_compute)
            .expect("hit");
        assert_eq!(v, "computed-value");
    }
    assert_eq!(
        compute_calls.load(Ordering::SeqCst),
        1,
        "compute closure must run exactly once across 11 cached_or_compute calls"
    );
}

#[test]
fn cached_or_compute_propagates_compute_error() {
    let cache = cache();
    let key = QueryKey::new("col".into(), "q".into(), 10, None);

    let result: Result<String, &'static str> =
        cache.cached_or_compute(key.clone(), || Err("backend unavailable"));
    assert_eq!(result.unwrap_err(), "backend unavailable");

    // The error path must NOT poison the cache — a subsequent
    // successful compute should be cached normally.
    let result: Result<String, &'static str> =
        cache.cached_or_compute(key.clone(), || Ok("recovered".into()));
    assert_eq!(result.unwrap(), "recovered");
    assert_eq!(cache.get(&key).as_deref(), Some("recovered"));
}

#[test]
fn concurrent_readers_and_writers_are_consistent() {
    use std::sync::Barrier;
    use std::thread;

    let cache: Arc<QueryCache<String>> = Arc::new(cache());
    let shared_key = QueryKey::new("col".into(), "shared".into(), 10, None);
    cache.insert(shared_key.clone(), "initial".into());

    // 16 threads run for ~5k iterations each. Half do gets, half do
    // inserts on a per-thread key (so they don't all collide on the
    // same entry). One thread invalidates the whole "col" collection
    // periodically.
    let n_threads = 16;
    let iters_per_thread = 5_000;
    let barrier = Arc::new(Barrier::new(n_threads));

    let handles: Vec<_> = (0..n_threads)
        .map(|tid| {
            let cache = Arc::clone(&cache);
            let barrier = Arc::clone(&barrier);
            let shared_key = shared_key.clone();
            thread::spawn(move || {
                barrier.wait();
                for i in 0..iters_per_thread {
                    if tid % 4 == 0 {
                        // Writer: insert per-thread keys so we exercise
                        // the LRU eviction path under contention.
                        let key = QueryKey::new("col".into(), format!("k-{tid}-{i}"), 10, None);
                        cache.insert(key, format!("v-{tid}-{i}"));
                    } else if tid == 1 && i % 500 == 0 {
                        // One thread periodically invalidates the
                        // whole "col" collection — the other threads
                        // must not panic or deadlock under that.
                        cache.invalidate_collection("col");
                    } else {
                        // Reader: hit the shared key. May be Some or
                        // None depending on whether the invalidator
                        // got there first; both are fine.
                        let _ = cache.get(&shared_key);
                    }
                }
            })
        })
        .collect();

    for h in handles {
        h.join().expect("worker thread must not panic");
    }

    // After the storm the cache is in some valid state — the only
    // assertion that matters is "no panic, no deadlock". Sanity-check
    // that the cache is still usable.
    cache.insert(shared_key.clone(), "final".into());
    assert_eq!(cache.get(&shared_key).as_deref(), Some("final"));
}

#[test]
fn prometheus_counter_increments_on_every_cache_get() {
    use vectorizer::monitoring::metrics::METRICS;

    let cache = cache();
    let key = QueryKey::new("col".into(), "metric-test".into(), 10, None);

    let baseline_hit = METRICS
        .cache_requests_total
        .with_label_values(&["query", "hit"])
        .get();
    let baseline_miss = METRICS
        .cache_requests_total
        .with_label_values(&["query", "miss"])
        .get();

    // Three misses then five hits.
    let _ = cache.get(&key);
    let _ = cache.get(&key);
    let _ = cache.get(&key);
    cache.insert(key.clone(), "value".into());
    for _ in 0..5 {
        let _ = cache.get(&key);
    }

    let hit_delta = METRICS
        .cache_requests_total
        .with_label_values(&["query", "hit"])
        .get()
        - baseline_hit;
    let miss_delta = METRICS
        .cache_requests_total
        .with_label_values(&["query", "miss"])
        .get()
        - baseline_miss;

    assert_eq!(
        miss_delta, 3.0,
        "three pre-insert reads must register as misses on the Prometheus counter"
    );
    assert_eq!(
        hit_delta, 5.0,
        "five post-insert reads must register as hits on the Prometheus counter"
    );
}
