# Proposal: phase5_wire-cache-into-read-paths

## Why

`src/cache/` exists with two layers (`query_cache.rs`, `advanced_cache.rs`) but is **opt-in** — consumers must explicitly fetch from cache before hitting the store. The architecture audit found:

- Read paths in `src/file_operations/` bypass cache entirely.
- Some handlers in `src/server/rest_handlers.rs` go directly to `VectorStore` without consulting cache.
- `advanced_cache.rs` duplicates logic from `query_cache.rs` instead of composing.

Result: the cache "exists" but doesn't actually protect the store from hot-query pressure, and its reported hit-rate metrics are misleading.

## What Changes

1. Pick one cache implementation as canonical (`query_cache.rs` is simpler — prefer it unless `advanced_cache.rs` has a feature we need).
2. Delete the duplicate.
3. Introduce a `CachedVectorStore` wrapper that wraps `VectorStore` and transparently caches read operations. All read paths go through it.
4. Retrofit handlers and file_operations to use the wrapper.
5. Add explicit invalidation on writes (insert/delete/collection drop must evict related cache entries).
6. Add cache metrics (hit/miss/size) to the existing metrics endpoint.

## Impact

- Affected specs: cache spec, API spec
- Affected code: `src/cache/*`, `src/db/vector_store.rs` (or successor), `src/server/handlers/*`, `src/file_operations/`
- Breaking change: NO external
- User benefit: actual cache effect (measurable hit rate); removes duplicate code; correct invalidation.
