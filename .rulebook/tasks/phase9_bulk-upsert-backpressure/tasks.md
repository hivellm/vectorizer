## 1. Config & scaffolding
- [x] 1.1 Add `BackpressureConfig` struct in `crates/vectorizer/src/config/vectorizer.rs` with all fields from proposal
- [x] 1.2 Wire env overrides (`CORTEX_VECTORIZER_MAX_CONCURRENT_BUILDS`, `..._UPSERT_HIGH_WATER`, `..._UPSERT_HARD_LIMIT`, `..._BACKPRESSURE_ENABLED`) in `VectorizerConfig::from_env`
- [x] 1.3 Update `config/config.example.yml` with the new `backpressure:` section + comments
- [x] 1.4 Add config validation (`BackpressureConfig::validate` + `VectorizerConfig::validate`; high_water < hard_limit, retry_after >= 1, hard_limit > 0)

## 2. Bounded vocabulary-build concurrency
- [x] 2.1 Add `BackpressureGuard` (wraps `Arc<tokio::sync::Semaphore>`) in `crates/vectorizer/src/db/backpressure.rs`
- [x] 2.2 Acquire permit at the entry of the BM25 vocabulary-build path: `Indexer::build_vocabulary_gated` + `FileLoader::set_backpressure`; plumbed through `workspace_loader`, `setup_handlers`, and `file_watcher::operations`. Bootstrap reads `BackpressureConfig` from `config.yml` and passes a shared guard.
- [x] 2.3 Default permit count = `num_cpus::get()` when config value is 0 (via `BackpressureConfig::resolved_max_concurrent_vocab_builds` + clamp `>=1` in guard)
- [x] 2.4 Ensure permit is released on all error paths (RAII via Drop on `BackpressurePermit`; covered by `core::backpressure::drop_releases_on_unwind`)
- [x] 2.5 Unit test: N permits + M > N concurrent builds — at most N hold permits at once (`tests/core/backpressure.rs::at_most_n_concurrent_holders`)

## 3. Per-collection upsert queue + 429
- [ ] 3.1 Add per-collection queue-depth counter (`DashMap<CollectionId, AtomicUsize>`)
- [ ] 3.2 Increment on upsert ingress, decrement on completion (RAII)
- [ ] 3.3 When depth > `upsert_queue_hard_limit`, return `429 Too Many Requests`
- [ ] 3.4 Set `Retry-After: <retry_after_seconds>` header on 429 responses
- [ ] 3.5 Mirror behavior on the gRPC path (return `RESOURCE_EXHAUSTED` with retry hint)
- [ ] 3.6 Mirror behavior on the MCP upsert tool (structured error w/ retryAfter)
- [ ] 3.7 Integration test: flood single collection past hard_limit → asserts 429 + header

## 4. Metrics
- [ ] 4.1 Register gauge `vectorizer_upsert_queue_depth{collection}`
- [ ] 4.2 Register gauge `vectorizer_upsert_in_flight{collection}`
- [ ] 4.3 Register counter `vectorizer_upsert_rejected_total{reason}`
- [ ] 4.4 Register gauge `vectorizer_vocab_build_permits_available`
- [ ] 4.5 Verify on `/metrics` endpoint with a smoke test

## 5. Read-path isolation
- [ ] 5.1 Audit Axum routes; split read endpoints (`/collections` GET, `/auth/*`, `/health`, `/metrics`) onto a dedicated `tokio::runtime::Runtime`
- [ ] 5.2 Or (fallback) wrap read handlers with a higher-priority `tower` layer with bounded concurrency
- [ ] 5.3 Ensure shared state (`VectorStore`) crosses runtimes safely (`Arc` only, no `block_on`)
- [ ] 5.4 Load test: while saturating writes, `/health` and `GET /collections` keep p99 < 500 ms

## 6. Log rate-limiting
- [ ] 6.1 Replace per-upsert `WARN BM25 vocabulary is empty …` with rate-limited emitter (`once_cell` + interval per collection)
- [ ] 6.2 Add counter `vectorizer_bm25_empty_vocab_fallback_total{collection}` so the data isn't lost
- [ ] 6.3 Default rate: 1 warn per collection per 5 s; configurable via `backpressure.log_rate_limit_per_5s`

## 7. SDK retry alignment
- [ ] 7.1 Confirm Rust `vectorizer-sdk` honors 429 + `Retry-After` (add if missing)
- [ ] 7.2 Confirm Python SDK (`sdks/python/`) honors 429 + `Retry-After` (add if missing)
- [ ] 7.3 Add SDK integration test that a flooding client backs off correctly

## 8. Observability runbook
- [ ] 8.1 Document the four metrics + how to read them (`docs/operations/backpressure.md`)
- [ ] 8.2 Document the new config knobs and env vars
- [ ] 8.3 Add a Grafana panel JSON snippet for queue depth (under `docs/operations/grafana/`)

## 9. Pre-tail verification
- [ ] 9.1 Update `CHANGELOG.md` ("Added: backpressure for bulk upserts (#263)")
- [ ] 9.2 Update `README.md` operations section to mention backpressure config
- [ ] 9.3 Run `cargo check` and `cargo clippy --all-targets -- -D warnings`
- [ ] 9.4 Reproduce the original burst scenario locally and confirm no restart loop

## Mandatory tail (required by rulebook v5.3.0)

- [ ] Update or create documentation covering the implementation
- [ ] Write tests covering the new behavior
- [ ] Run tests and confirm they pass
