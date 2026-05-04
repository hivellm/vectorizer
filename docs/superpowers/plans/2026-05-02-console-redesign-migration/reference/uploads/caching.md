# Caching architecture

Vectorizer ships **two orthogonal caches** with different keys, owners,
and lifecycles. This document covers what each one caches, when to
reach for which, and the invariants the test suite enforces.

## Two caches, by purpose

| Cache | Lives in | Caches | Owner |
|---|---|---|---|
| **Query cache** | `src/cache/query_cache.rs` | The serialized JSON response of a search call (`Json<Value>`) keyed by `(collection, query, limit, threshold)` | Server state — one `Arc<QueryCache<Value>>` per `VectorizerServer`. Plumbed via `state.query_cache`. |
| **File-operations cache** | `src/file_operations/cache.rs` | File content, file summaries, and per-collection file listings (LRU + per-entry TTL) | The `FileOperationsManager` instance. Independent of the query cache. |

The two never share a key. A change to a collection's vectors
invalidates query cache entries for that collection but leaves file
content cache entries alone (file content is keyed on file path, not
on the search graph).

## Query cache

### What it caches

A `QueryCache<T>` is generic over the cached value type, but in
practice the server uses one shared `QueryCache<serde_json::Value>`
that holds **the JSON response body** of search-style endpoints. The
key is a `QueryKey { collection, query, limit, threshold }` — the
threshold is stored as `u32` (f64 × 1000) so the key is `Hash + Eq`
without `f64`-NaN headaches.

### Why "cache the response, not the SearchResult"

A naive design would cache `Vec<SearchResult>` and let each handler
JSON-serialize on every hit. Caching the already-formatted `Value`
trades a few bytes of memory for ~zero CPU on hits — the hot path is
a single `LruCache::get` + an `Arc`-clone (the inner `Value` is
cloned by value but most response bodies are small and the clone is
amortized away by reusing the same shape across many concurrent
requests).

### How handlers use it

Existing search handlers carry the cache-aside pattern manually
(check, branch, compute, insert):

```rust
let key = QueryKey::new(collection, query, limit, threshold);
if let Some(cached) = state.query_cache.get(&key) {
    return Ok(Json(cached));
}
let response = /* run the search, build the Json::Value */;
state.query_cache.insert(key, response.clone());
Ok(Json(response))
```

This works and is what every existing search handler does today
(`src/server/rest_handlers/{search,intelligent_search}.rs`). For new
code, prefer the **`cached_or_compute` helper** that collapses the
pattern into one call:

```rust
let key = QueryKey::new(collection, query, limit, threshold);
let response = state.query_cache.cached_or_compute(key, || {
    // expensive: embed + ANN search + JSON shape
    let results = collection.search(&embedding, limit)?;
    Ok::<_, ErrorResponse>(json!({ "results": results, /* … */ }))
})?;
Ok(Json(response))
```

The helper:

- Calls `cache.get(&key)` first. On hit, returns the cached value
  without invoking the closure.
- On miss, runs the closure, inserts the result into the cache, and
  returns it.
- Propagates the closure's `Err` without caching it (so a transient
  backend failure doesn't poison the cache for the whole TTL).
- Records the Prometheus hit/miss metric automatically (the metric
  is wired inside `cache.get`, so both patterns benefit).

### Invalidation

Write paths invalidate explicitly:

```rust
// in src/server/rest_handlers/{insert,vectors,collections}.rs
state.query_cache.invalidate_collection(&collection_name);
```

The current write sites covered:

- `POST /insert` (insert.rs:455)
- `DELETE /collections/{name}/vectors/{id}` (vectors.rs:165, 202)
- `DELETE /collections/{name}` (collections.rs:405)

Any new write path that mutates a collection MUST call
`invalidate_collection` after the write succeeds. The integration test
`tests/cache/query_cache_behaviour.rs::invalidation_drops_only_targeted_collection`
pins the collection-scoped invalidation contract so a future change
that broadens or narrows the scope breaks loudly.

### TTL

Default TTL is 5 minutes (`QueryCacheConfig::default().ttl_seconds =
300`). On each `get`, an expired entry is removed and the get returns
`None` (counted as a miss). This means the cache self-heals from
stale data even if a write path forgets to invalidate.

### Metrics

Two parallel metric streams:

1. **In-process counters** on the `QueryCache` itself (`hits`,
   `misses`, `evictions`, `hit_rate`). Surfaced as JSON via
   `GET /stats` (see `src/server/rest_handlers/meta.rs`).
2. **Prometheus** `vectorizer_cache_requests_total{cache_type="query",
   result="hit"|"miss"}` counter, incremented inside `QueryCache::get`
   on every call. This is what dashboards and alerting consume.

Before this work, the Prometheus counter was registered but never
incremented — the gauge was stuck at zero forever. The wiring is
covered by
`tests/cache/query_cache_behaviour.rs::prometheus_counter_increments_on_every_cache_get`.

### Concurrency

The cache uses `parking_lot::RwLock<LruCache>` internally. Writers
take a write lock; readers also take a write lock today (the LRU
needs to update recency on every `get`, so a read lock isn't enough).
Per-stat counters (`hits`, `misses`, `evictions`) live behind their
own `parking_lot::Mutex`. The `concurrent_readers_and_writers_are_consistent`
test runs 16 threads × 5,000 iterations of mixed get / insert /
invalidate against the same cache and asserts no panic, no deadlock,
and the cache is still usable after the storm.

## File-operations cache

Lives at `src/file_operations/cache.rs` and is owned by the
`FileOperationsManager`. Three independent LRU caches inside one
struct:

- `file_content_cache` — full file content (max 100 files).
- `summary_cache` — generated file summaries (max 500).
- `file_list_cache` — per-collection file listing with TTL.

This cache is keyed on file paths and collection names. It is not
invalidated by the query-cache invalidation path because the two have
no key overlap. The file-operations cache has its own internal
invalidation triggers (file watcher events, explicit `clear_*`
methods).

The file-operations cache and the query cache do NOT share an
implementation; they are intentionally separate because the access
patterns and key shapes don't overlap. Trying to unify them behind a
single trait would be a regression: the query cache wants TTL and
collection-scoped invalidation; the file cache wants per-file-path
invalidation and a much larger entry size budget.

## What was removed in `phase5_wire-cache-into-read-paths`

The repo previously carried 6 unused files in `src/cache/` — about
3,300 LOC of dead code (`advanced_cache.rs`, `incremental.rs`,
`manager.rs`, `metadata.rs`, `validation.rs`, `tests.rs`) that were
never declared in `mod.rs` and so never compiled into the crate.
They documented a "multi-layer cache architecture (L1/L2/L3)" that
was designed but never wired. Removed wholesale because:

- Nothing imported them (verified with `grep -rn "use crate::cache::"`).
- `cargo build` showed no warnings about them since they were
  silently excluded from the crate.
- The "advanced cache" features they sketched (multi-layer L1/L2/L3,
  pluggable eviction policies) had no demand from any caller; the
  one-layer LRU-with-TTL `QueryCache` covers every observed access
  pattern.

If a future workload needs multi-layer caching, the right move is to
re-introduce it deliberately with a real consumer in the same commit
— not to leave the design checked in as a placeholder.

## Cross-references

- `src/cache/query_cache.rs` — implementation
- `src/server/rest_handlers/{search,intelligent_search,collections,insert,vectors}.rs`
  — every existing call site (4 reads, 3 invalidation sites)
- `src/server/rest_handlers/meta.rs` — JSON `/stats` exposition
- `src/monitoring/metrics.rs::cache_requests_total` — Prometheus counter
- `tests/cache/query_cache_behaviour.rs` — integration tests (6 tests:
  hit-rate, invalidation, helper hit/miss, helper error path,
  concurrent r/w, Prometheus counter wiring)
