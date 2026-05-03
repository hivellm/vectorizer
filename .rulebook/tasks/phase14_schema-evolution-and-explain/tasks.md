## 1. Server — rename_collection
- [ ] 1.1 Add `rename_collection(old, new)` in `crates/vectorizer/src/db/store.rs` that atomically swaps the index entry and persists a tombstone alias entry
- [ ] 1.2 Wire alias resolution into the lookup path so requests for `old` transparently hit `new` for one minor version
- [ ] 1.3 Handler `rename_collection` in `rest_handlers/collections.rs`
- [ ] 1.4 Route `POST /collections/{name}/rename`
- [ ] 1.5 Integration test: rename `c1` → `c2`; reads/writes by `c1` succeed and target the new collection
- [ ] 1.6 Integration test: alias is dropped after the configured grace window

## 2. Server — reindex_collection (new HNSW params)
- [ ] 2.1 Add `reindex_with_params(collection, HnswParams)` in `crates/vectorizer/src/index/hnsw/`
- [ ] 2.2 Build new index off-line, atomic swap, retain old until next save
- [ ] 2.3 Handler `reindex_collection` returning a `ReindexJob` with progress channel
- [ ] 2.4 Route `POST /collections/{name}/reindex`
- [ ] 2.5 Integration test: reindex from `M=16` to `M=32` on a 10k-vector collection; record recall@10 + latency in `benches/reports/`
- [ ] 2.6 Integration test: writes during reindex are queued and applied to the new index post-swap

## 3. Server — native snapshot endpoints
- [ ] 3.1 Add `snapshot_native(collection, options)` in `crates/vectorizer/src/persistence/` emitting a single `.vecdb` tarball
- [ ] 3.2 Handlers `create_native_snapshot`, `list_native_snapshots`, `restore_native_snapshot`
- [ ] 3.3 Routes `POST /collections/{name}/snapshot`, `GET /collections/{name}/snapshots`, `POST /collections/{name}/snapshots/{id}/restore`
- [ ] 3.4 Integration test: snapshot a collection, drop it, restore from snapshot, verify all vectors and metadata round-trip

## 4. Server — explain_search
- [ ] 4.1 Instrument HNSW search in `crates/vectorizer/src/index/hnsw/search.rs` to record `visited_nodes`, `layer_path`, `payload_filter_evals`, `quantization_score_ms` (gated by an `explain: bool` flag, zero overhead when off)
- [ ] 4.2 Handler `explain_search` running the same search path with `explain=true`
- [ ] 4.3 Route `POST /collections/{name}/explain`
- [ ] 4.4 Integration test: explain returns a trace whose `visited_nodes` ≤ `ef_search × layers`
- [ ] 4.5 Integration test: explain hits match exactly the hits a regular search returns for the same query

## 5. Server — slow-query log
- [ ] 5.1 Add `SlowQueryRing` (capacity-bounded ring buffer) in `crates/vectorizer/src/cache/slow_query.rs`
- [ ] 5.2 Capture every search whose duration ≥ configured threshold
- [ ] 5.3 Handler `list_slow_queries(params)` and `set_slow_query_config(config)`
- [ ] 5.4 Routes `GET /slow_queries` and `POST /slow_queries/config`
- [ ] 5.5 Integration test: queries below the threshold are NOT recorded; queries above ARE recorded
- [ ] 5.6 Integration test: ring buffer eviction works at capacity

## 6. Rust SDK
- [ ] 6.1 `rename_collection(&self, old, new) -> Result<()>`
- [ ] 6.2 `reindex_collection(&self, name, params) -> Result<ReindexJob>`
- [ ] 6.3 `snapshot_collection_native(&self, name, request) -> Result<SnapshotInfo>`
- [ ] 6.4 `list_collection_snapshots_native(&self, name) -> Result<Vec<SnapshotInfo>>`
- [ ] 6.5 `restore_collection_snapshot_native(&self, name, id) -> Result<()>`
- [ ] 6.6 `explain_search(&self, collection, request) -> Result<ExplainResponse>`
- [ ] 6.7 `list_slow_queries(&self, params) -> Result<Vec<SlowQueryEntry>>`
- [ ] 6.8 `set_slow_query_config(&self, config) -> Result<SlowQueryConfig>`
- [ ] 6.9 Bump `sdks/rust/Cargo.toml` 3.5 → 3.6
- [ ] 6.10 Unit + s2s integration tests per method

## 7. TypeScript SDK
- [ ] 7.1 Mirror section 6 in `sdks/typescript/src/client/collections.ts` and `search.ts`
- [ ] 7.2 Bump `sdks/typescript/package.json` 3.5 → 3.6
- [ ] 7.3 Vitest unit + integration tests

## 8. Python SDK
- [ ] 8.1 Mirror section 6 in `sdks/python/vectorizer/{collections,search}.py`
- [ ] 8.2 Bump `sdks/python/pyproject.toml` 3.5 → 3.6
- [ ] 8.3 pytest unit + integration tests

## 9. Documentation
- [ ] 9.1 Document new routes in `docs/api/`
- [ ] 9.2 Add a "Day-2 ops" page covering rename, reindex, native snapshots, explain, and slow-query log
- [ ] 9.3 Update SDK READMEs
- [ ] 9.4 CHANGELOG entries (server + each SDK)

## 10. Tail (mandatory — enforced by rulebook v5.3.0)
- [ ] 10.1 Update or create documentation covering the implementation
- [ ] 10.2 Write tests covering the new behavior
- [ ] 10.3 Run tests and confirm they pass
