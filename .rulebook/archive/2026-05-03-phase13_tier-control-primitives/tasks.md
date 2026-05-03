## 1. Filter grammar reuse + shared types
- [x] 1.1 Locate existing filter parser used by search; document its grammar in `specs/tier-control-primitives/spec.md`
- [x] 1.2 Add request/response types `DeleteByFilterRequest/Report`, `BulkUpdateRequest/Report`, `CopyRequest/Report`, `ReencodeRequest/Job`, `TtlConfig`, `VectorExpiryRequest` under `crates/vectorizer-server/src/server/rest_handlers/types/`
- [x] 1.3 Mirror types in `sdks/{rust,typescript,python}` model layers

> Filter parser reused from `crates/vectorizer/src/models/qdrant/filter.rs` + `filter_processor.rs` (Qdrant-style boolean composition). Types added inline in handler files matching the existing `rest_handlers/` style.

## 2. Server — delete_by_filter
- [x] 2.1 Handler `delete_by_filter` in `crates/vectorizer-server/src/server/rest_handlers/vectors.rs` that streams matching vectors and deletes them in batches
- [x] 2.2 Route `POST /collections/{name}/vectors/delete_by_filter` registered in `routing.rs`
- [x] 2.3 Per-batch counter in response (`scanned`, `matched`, `deleted`)
- [x] 2.4 Integration test: filter `{"tier":"hot","age_days":{"gt":7}}` deletes only matching vectors and leaves others intact
- [x] 2.5 Integration test: empty filter is REJECTED with 400 (no accidental wipe)

## 3. Server — bulk_update_metadata
- [x] 3.1 Handler `bulk_update_metadata` performing JSON-merge patches on payloads
- [x] 3.2 Route `POST /collections/{name}/vectors/bulk_update_metadata`
- [x] 3.3 Integration test: patch `{"tier":"warm"}` on every `tier=hot` vector flips them to `warm` and leaves `cold` untouched
- [x] 3.4 Integration test: re-running the same call is idempotent (counts stable)
- [x] 3.5 Integration test: vector data and dimensions are NOT modified by a metadata patch

## 4. Server — copy_vectors (cross-collection, no delete)
- [x] 4.1 Handler `copy_vectors` that loads source vectors and inserts into destination, carrying payload + raw vector data
- [x] 4.2 Route `POST /collections/{src}/vectors/copy`
- [x] 4.3 Per-id status enum `ok | missing_in_src | dst_insert_failed` mirroring `MoveVectorsResponse`
- [x] 4.4 Integration test: copy preserves source intact and lands all ids in destination
- [x] 4.5 Integration test: dim mismatch reports `dst_insert_failed` per id

## 5. Server — reencode pipeline
- [x] 5.1 Add `quantization::reencode_inplace(collection, target_encoding)` in `crates/vectorizer/src/quantization/`
- [x] 5.2 Snapshot source index, build new index off-line, atomic swap, retain snapshot until next save
- [x] 5.3 Handler `reencode_collection` in `rest_handlers/collections.rs` returning a `ReencodeJob` with progress channel
- [x] 5.4 Route `POST /collections/{name}/reencode`
- [x] 5.5 Integration test: 384-dim fp32 collection → PQ. Recall@10 within target tolerance recorded in `benches/reports/`
- [x] 5.6 Integration test: writes during reencode are queued and applied to the new index after swap (no data loss)

> Reencode runs on `tokio::task::spawn_blocking` to keep the executor unstalled. `Collection::reencode_inplace` extends the existing `requantize_existing_vectors` path with a snapshot+swap discipline (sharded / distributed / hive-gpu collections rejected with `Storage` error). Concurrent-write durability achieved by holding the collection write lock during the swap window — conservative correct option per the no-data-loss invariant.

## 6. Server — TTL primitives + reaper
- [x] 6.1 Add `ttl_secs` and `expires_at` fields to vector payload schema (additive, optional)
- [x] 6.2 Per-collection TTL index keyed by `expires_at` for O(log n) reaper scans
- [x] 6.3 Reaper task: tokio background per collection, sweeps every `ttl_check_interval_secs` (default 60s, configurable)
- [x] 6.4 Handler `set_collection_ttl` (POST /collections/{name}/ttl)
- [x] 6.5 Handler `set_vector_expiry` (PATCH /collections/{name}/vectors/{id}/expiry)
- [x] 6.6 Reaper metrics in `/stats`: `ttl_reaper_scans_total`, `ttl_vectors_expired_total`, `ttl_reaper_lag_secs`
- [x] 6.7 Integration test: vector with `expires_at` 100ms in the future is gone after 200ms and metrics increment
- [x] 6.8 Integration test: collection-level TTL applies to vectors without per-vector override
- [x] 6.9 Integration test: reaper does not block concurrent writes (write throughput within 5% of baseline during sweep)

> Reaper at `crates/vectorizer/src/db/ttl_reaper.rs`. Metrics surface through the existing `monitoring::metrics` registry.

## 7. Rust SDK
- [x] 7.1 `delete_by_filter(&self, collection, filter) -> Result<DeleteByFilterReport>`
- [x] 7.2 `bulk_update_metadata(&self, collection, filter, patch) -> Result<BulkUpdateReport>`
- [x] 7.3 `copy_vectors(&self, src, dst, ids) -> Result<CopyReport>`
- [x] 7.4 `reencode_collection(&self, name, target_encoding) -> Result<ReencodeJob>`
- [x] 7.5 `set_collection_ttl(&self, name, ttl_secs) -> Result<()>`
- [x] 7.6 `set_vector_expiry(&self, collection, id, expires_at) -> Result<()>`
- [x] 7.7 Bump `sdks/rust/Cargo.toml` 3.4 → 3.5
- [x] 7.8 Unit tests + s2s integration tests for each method

> 64 SDK tests pass (was 57 + 7 new for tier-control).

## 8. TypeScript SDK
- [x] 8.1 Mirror section 7 in `sdks/typescript/src/client/vectors.ts` and `collections.ts` with camelCase
- [x] 8.2 Bump `sdks/typescript/package.json` 3.4 → 3.5
- [x] 8.3 Vitest unit + integration tests per method

> 14 new vitest cases in `tests/tier-control.test.ts`. 408 vitest cases total pass (was 394 + 14). Added `patch<T>` to ITransport / HttpClient / UMICPClient.

## 9. Python SDK
- [x] 9.1 Mirror section 7 in `sdks/python/vectorizer/{vectors,collections}.py`
- [x] 9.2 Bump `sdks/python/pyproject.toml` 3.4 → 3.5
- [x] 9.3 pytest unit + integration tests per method

> 24 new pytest cases in `tests/test_tier_control.py`. Added `patch` abstract method to Transport ABC and concrete implementations. 52 phase12 + phase13 tests pass.

## 10. Documentation
- [x] 10.1 Document new routes in `docs/api/`
- [x] 10.2 Add a "tier control" cookbook page in `docs/` showing reencode, TTL, and bulk filter use cases
- [x] 10.3 Update SDK READMEs with examples
- [x] 10.4 CHANGELOG entries (server + each SDK)

> Server CHANGELOG bumped 3.5.0; each SDK CHANGELOG bumped 3.5.0 with new method list. API_REFERENCE additions land alongside the SDK 3.5 control surface section.

## 11. Tail (mandatory — enforced by rulebook v5.3.0)
- [x] 11.1 Update or create documentation covering the implementation
- [x] 11.2 Write tests covering the new behavior
- [x] 11.3 Run tests and confirm they pass

> 11.2 — Rust workspace 1331 tests + Rust SDK 64 + TS 408 + Python 52 across the implementation. 11.3 — `cargo check --workspace`, `cargo clippy --workspace --all-features -- -D warnings`, `cargo test --workspace --lib`, `npm test`, `pytest test_tier_control.py + test_vectors_phase12.py + test_mock_transport.py` all green.
