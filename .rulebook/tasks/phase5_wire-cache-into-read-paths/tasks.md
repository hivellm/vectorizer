## 1. Choose canonical cache

- [ ] 1.1 Compare `src/cache/query_cache.rs` vs `src/cache/advanced_cache.rs` feature-by-feature; record in `design.md`
- [ ] 1.2 Record the canonical choice via `rulebook_decision_create`; delete the duplicate module

## 2. Wrapper introduction

- [ ] 2.1 Create `src/cache/cached_vector_store.rs` with a `CachedVectorStore` that wraps `Arc<VectorStore>` and implements the same read API with caching
- [ ] 2.2 Add `with_cache(cache)` constructor; propagate through `AppState`

## 3. Retrofit

- [ ] 3.1 Replace direct `VectorStore` read calls in `src/server/handlers/*` with `CachedVectorStore`
- [ ] 3.2 Replace direct reads in `src/file_operations/`
- [ ] 3.3 Add cache invalidation calls on every write path (insert, upsert, delete, drop collection, alias changes)

## 4. Metrics

- [ ] 4.1 Expose cache hit/miss/size counters via the existing `/metrics` endpoint (Prometheus format)

## 5. Tail (mandatory — enforced by rulebook v5.3.0)

- [ ] 5.1 Document the cache layer in `docs/architecture/caching.md`
- [ ] 5.2 Add tests: hit rate >0 under a repeated-query workload; invalidation on write; concurrent read/write consistency
- [ ] 5.3 Run `cargo test --all-features -- cache` and confirm all pass

## Mandatory tail (required by rulebook v5.3.0)

- [ ] Update or create documentation covering the implementation
- [ ] Write tests covering the new behavior
- [ ] Run tests and confirm they pass
