## 1. Filter grammar reuse + shared types
- [ ] 1.1 Locate existing filter parser used by search; document its grammar in `specs/tier-control-primitives/spec.md`
- [ ] 1.2 Add request/response types `DeleteByFilterRequest/Report`, `BulkUpdateRequest/Report`, `CopyRequest/Report`, `ReencodeRequest/Job`, `TtlConfig`, `VectorExpiryRequest` under `crates/vectorizer-server/src/server/rest_handlers/types/`
- [ ] 1.3 Mirror types in `sdks/{rust,typescript,python}` model layers

## 2. Server — delete_by_filter
- [ ] 2.1 Handler `delete_by_filter` in `crates/vectorizer-server/src/server/rest_handlers/vectors.rs` that streams matching vectors and deletes them in batches
- [ ] 2.2 Route `POST /collections/{name}/vectors/delete_by_filter` registered in `routing.rs`
- [ ] 2.3 Per-batch counter in response (`scanned`, `matched`, `deleted`)
- [ ] 2.4 Integration test: filter `{"tier":"hot","age_days":{"gt":7}}` deletes only matching vectors and leaves others intact
- [ ] 2.5 Integration test: empty filter is REJECTED with 400 (no accidental wipe)

## 3. Server — bulk_update_metadata
- [ ] 3.1 Handler `bulk_update_metadata` performing JSON-merge patches on payloads
- [ ] 3.2 Route `POST /collections/{name}/vectors/bulk_update_metadata`
- [ ] 3.3 Integration test: patch `{"tier":"warm"}` on every `tier=hot` vector flips them to `warm` and leaves `cold` untouched
- [ ] 3.4 Integration test: re-running the same call is idempotent (counts stable)
- [ ] 3.5 Integration test: vector data and dimensions are NOT modified by a metadata patch

## 4. Server — copy_vectors (cross-collection, no delete)
- [ ] 4.1 Handler `copy_vectors` that loads source vectors and inserts into destination, carrying payload + raw vector data
- [ ] 4.2 Route `POST /collections/{src}/vectors/copy`
- [ ] 4.3 Per-id status enum `ok | missing_in_src | dst_insert_failed` mirroring `MoveVectorsResponse`
- [ ] 4.4 Integration test: copy preserves source intact and lands all ids in destination
- [ ] 4.5 Integration test: dim mismatch reports `dst_insert_failed` per id

## 5. Server — reencode pipeline
- [ ] 5.1 Add `quantization::reencode_inplace(collection, target_encoding)` in `crates/vectorizer/src/quantization/`
- [ ] 5.2 Snapshot source index, build new index off-line, atomic swap, retain snapshot until next save
- [ ] 5.3 Handler `reencode_collection` in `rest_handlers/collections.rs` returning a `ReencodeJob` with progress channel
- [ ] 5.4 Route `POST /collections/{name}/reencode`
- [ ] 5.5 Integration test: 384-dim fp32 collection → PQ. Recall@10 within target tolerance recorded in `benches/reports/`
- [ ] 5.6 Integration test: writes during reencode are queued and applied to the new index after swap (no data loss)

## 6. Server — TTL primitives + reaper
- [ ] 6.1 Add `ttl_secs` and `expires_at` fields to vector payload schema (additive, optional)
- [ ] 6.2 Per-collection TTL index keyed by `expires_at` for O(log n) reaper scans
- [ ] 6.3 Reaper task: tokio background per collection, sweeps every `ttl_check_interval_secs` (default 60s, configurable)
- [ ] 6.4 Handler `set_collection_ttl` (POST /collections/{name}/ttl)
- [ ] 6.5 Handler `set_vector_expiry` (PATCH /collections/{name}/vectors/{id}/expiry)
- [ ] 6.6 Reaper metrics in `/stats`: `ttl_reaper_scans_total`, `ttl_vectors_expired_total`, `ttl_reaper_lag_secs`
- [ ] 6.7 Integration test: vector with `expires_at` 100ms in the future is gone after 200ms and metrics increment
- [ ] 6.8 Integration test: collection-level TTL applies to vectors without per-vector override
- [ ] 6.9 Integration test: reaper does not block concurrent writes (write throughput within 5% of baseline during sweep)

## 7. Rust SDK
- [ ] 7.1 `delete_by_filter(&self, collection, filter) -> Result<DeleteByFilterReport>`
- [ ] 7.2 `bulk_update_metadata(&self, collection, filter, patch) -> Result<BulkUpdateReport>`
- [ ] 7.3 `copy_vectors(&self, src, dst, ids) -> Result<CopyReport>`
- [ ] 7.4 `reencode_collection(&self, name, target_encoding) -> Result<ReencodeJob>`
- [ ] 7.5 `set_collection_ttl(&self, name, ttl_secs) -> Result<()>`
- [ ] 7.6 `set_vector_expiry(&self, collection, id, expires_at) -> Result<()>`
- [ ] 7.7 Bump `sdks/rust/Cargo.toml` 3.4 → 3.5
- [ ] 7.8 Unit tests + s2s integration tests for each method

## 8. TypeScript SDK
- [ ] 8.1 Mirror section 7 in `sdks/typescript/src/client/vectors.ts` and `collections.ts` with camelCase
- [ ] 8.2 Bump `sdks/typescript/package.json` 3.4 → 3.5
- [ ] 8.3 Vitest unit + integration tests per method

## 9. Python SDK
- [ ] 9.1 Mirror section 7 in `sdks/python/vectorizer/{vectors,collections}.py`
- [ ] 9.2 Bump `sdks/python/pyproject.toml` 3.4 → 3.5
- [ ] 9.3 pytest unit + integration tests per method

## 10. Documentation
- [ ] 10.1 Document new routes in `docs/api/`
- [ ] 10.2 Add a "tier control" cookbook page in `docs/` showing reencode, TTL, and bulk filter use cases
- [ ] 10.3 Update SDK READMEs with examples
- [ ] 10.4 CHANGELOG entries (server + each SDK)

## 11. Tail (mandatory — enforced by rulebook v5.3.0)
- [ ] 11.1 Update or create documentation covering the implementation
- [ ] 11.2 Write tests covering the new behavior
- [ ] 11.3 Run tests and confirm they pass
