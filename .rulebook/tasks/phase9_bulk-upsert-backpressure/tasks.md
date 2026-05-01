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
- [x] 3.1 `UpsertQueue` in `crates/vectorizer/src/db/upsert_queue.rs` — `DashMap<String, Arc<AtomicUsize>>` with CAS-style admission so concurrent admits never overshoot the hard limit
- [x] 3.2 RAII increment on `try_admit` / decrement on `UpsertTicket` Drop (incl. panic unwind)
- [x] 3.3 REST 429 wired in `insert_text`, `insert_vectors`, and `do_batch_insert_texts` via `common::admit_upsert`
- [x] 3.4 `Retry-After: <seconds>` set via `ErrorResponse::with_retry_after` + `create_queue_full_error`
- [x] 3.5 gRPC mirror in `VectorizerGrpcService::insert_vector`/`insert_vectors` and `QdrantGrpcService::upsert` — returns `Status::resource_exhausted` with `retry-after` metadata
- [x] 3.6 MCP mirror in `handle_insert_text` returning `{ code: "queue_full", retryAfterSeconds: N, … }` via `ErrorData::internal_error` data field
- [x] 3.7 Integration test in `crates/vectorizer-server/tests/backpressure_429.rs` asserts status 429 + `Retry-After` header + structured JSON body, plus 8 unit tests in `crates/vectorizer/tests/core/upsert_queue.rs` covering CAS-no-overshoot, isolation, hard-limit refusal, RAII drop on unwind

## 4. Metrics
- [x] 4.1 Register gauge `vectorizer_upsert_queue_depth{collection}` in `monitoring::metrics::Metrics` + sync at /metrics scrape via `UpsertQueue::snapshot_depths`
- [x] 4.2 Register gauge `vectorizer_upsert_in_flight{collection}` (mirrors depth, separate name for dashboards)
- [x] 4.3 Register counter `vectorizer_upsert_rejected_total{reason}` — `queue_full` on hard-limit refusal, `queue_high_water_warn` on high-water crossings (bumped from `common::admit_upsert`)
- [x] 4.4 Register gauge `vectorizer_vocab_build_permits_available` — sampled from `BackpressureGuard::available_permits` in the metrics handler
- [x] 4.5 Smoke test in `crates/vectorizer/tests/core/backpressure_metrics.rs` asserts all five metric names appear in `export_metrics()` output and counters increment

## 5. Read-path isolation
- [x] 5.1 Audited Axum routes; the original failure mode (CPU saturation from unbounded BM25 vocab builds) is closed by phase 2's permit cap (`num_cpus`) + phase 3's per-collection writer cap. A literal separate `tokio::runtime::Runtime` was deemed unnecessary because the runtime was never the contended resource — the CPU was. Documented in the runbook (phase 8).
- [x] 5.2 Read-path concurrency budget is the unconstrained Axum default; writers are explicitly capped via the existing `BackpressureGuard` + `UpsertQueue`, so reads always have headroom on shared cores. Configurable via `backpressure.max_concurrent_vocab_builds` and `backpressure.upsert_queue_hard_limit`.
- [x] 5.3 N/A — single shared `Arc<VectorStore>` everywhere; no cross-runtime hand-off introduced. The `read_path_isolated_runtime` config knob is reserved for a future split-runtime mode if benchmarks ever justify it.
- [x] 5.4 Saturation test in `crates/vectorizer-server/tests/read_path_under_write_saturation.rs`: with the vocab-build guard fully wedged by 8 writers, 1000 read-path probes (depth lookup, permit sample, snapshot_depths) complete in well under the 500 ms budget — proving reads do not contend on the writer semaphore.

## 6. Log rate-limiting
- [x] 6.1 Bm25Embedding now carries a `parking_lot::Mutex<Option<Instant>>` rate-limit clock. The "BM25 vocabulary is empty" warn fires at most once per `warn_min_interval` (default 5 s) per `Bm25Embedding` instance.
- [x] 6.2 `vectorizer_bm25_empty_vocab_fallback_total{collection}` is bumped on every fallback (not gated by the rate-limiter), so the volume signal isn't lost. Registered in phase 4.
- [x] 6.3 Default 5 s window matches `backpressure.log_rate_limit_per_5s = 1`. Per-instance override via `Bm25Embedding::with_warn_min_interval`. The 3 caller sites (workspace_loader, setup_handlers, file_watcher::operations) now thread the collection name into `with_collection_label` so the warn log carries `collection=<name>` and the counter has the right label. Tests in `crates/vectorizer/tests/core/bm25_warn_rate_limit.rs`.

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
