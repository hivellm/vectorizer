# Implementation Tasks - Query Caching

## 1. Cache Implementation
- [x] 1.1 Verify `lru` dependency (v0.16.1 already in Cargo.toml)
- [x] 1.2 Create `src/cache/query_cache.rs`
- [x] 1.3 Define QueryCache struct (with LRU + TTL support)
- [x] 1.4 Define QueryKey struct (collection, query, limit, threshold)
- [x] 1.5 Implement get/insert methods (with expiration check)
- [x] 1.6 Implement invalidation logic (per-collection and global clear)
- [x] 1.7 Add comprehensive tests (9 tests, all passing)

## 2. Configuration
- [x] 2.1 Configure cache size (max_size: 1000)
- [x] 2.2 Configure TTL (ttl_seconds: 300 = 5 min)
- [x] 2.3 Add warmup logic (warmup_enabled flag)
- [x] 2.4 Add to config.yml (in performance.query_cache section)

## 3. Integration
- [ ] 3.1 Integrate with search endpoints
- [ ] 3.2 Integrate with intelligent search
- [ ] 3.3 Add invalidation on updates
- [ ] 3.4 Test cache behavior

## 4. Metrics
- [ ] 4.1 Add cache hit/miss metrics
- [ ] 4.2 Add eviction metrics
- [ ] 4.3 Add size gauge
- [ ] 4.4 Add to /health endpoint

## 5. Testing & Docs
- [ ] 5.1 Add unit tests
- [ ] 5.2 Add integration tests
- [ ] 5.3 Add benchmarks
- [ ] 5.4 Document caching strategy
- [ ] 5.5 Update CHANGELOG.md

