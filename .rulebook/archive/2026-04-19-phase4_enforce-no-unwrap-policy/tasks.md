## 1. Audit

- [x] 1.1 Dumped `grep -rnE '\.unwrap\(\)|\.expect\(' src/ --include='*.rs' > scripts/design-unwrap-audit.txt` (1456 hits) and built `scripts/classify_unwraps.py` to split by call-site scope. Latest classification: **1033 test-only** in 106 files (need only `#[allow]`), **233 production-code** in 67 files (need fixes / SAFE comments).
- [x] 1.2 Dumped `grep -rnE '\.ok\(\)' src/ --include='*.rs' > scripts/design-ok-audit.txt` (131 hits). Inspection shows the vast majority are legitimate `.parse().ok()` / `serde_json::from_str(...).ok()` / `flush().ok()` patterns that explicitly discard recoverable errors as designed. None convert a user-facing error path into a silent failure; no rewrites required.

## 2. Fix passes (top offender files first; 1-2 files per sub-step)

- [x] 2.1 `src/server/mcp/tools.rs` (was 31 hits) — replaced every `json!({...}).as_object().unwrap().clone().into()` with a new local `schema(value)` helper that pattern-matches `Value::Object(map)` and `unreachable!()`s on the impossible arm. Zero `.unwrap()` calls remain in this file.
- [x] 2.2 `src/monitoring/metrics.rs` (30 hits) — every unwrap is inside `Metrics::new()` constructing prometheus collectors from static `&'static str` names, which can only fail on malformed names (compile-time-checked). Added a doc block + function-level `#[allow(clippy::unwrap_used)]` documenting the static-invariant rationale. No semantic change.
- [x] 2.3 `src/file_watcher/hash_validator.rs` (30 hits) — every unwrap is inside the `#[cfg(test)] mod tests` block. Added `#[allow(clippy::unwrap_used, clippy::expect_used)]` to that block; production code is unwrap-free.
- [x] 2.4 `src/db/collection.rs` was split into `src/db/collection/{mod,data,graph,index,persistence,quantization}.rs`; only 2 unwraps remain in `mod.rs::Collection::new`. Documented both with `// SAFE:` rationale + `#[allow(clippy::expect_used)]` (HNSW from validated config = static-invariant; mmap is genuine I/O — flagged for follow-up Result conversion).
- [x] 2.5 `src/server/hub_handlers/tenant.rs` (was 28 hits; 1 prod, rest test) — annotated the lone prod hit (`as_array_mut().unwrap()` on a literal `json!` two lines above) with a SAFE comment + `#[allow]`; the test mod got the bulk allow.
- [x] 2.6 `src/utils/file_hash.rs` (27) and `src/quantization/storage.rs` (27): both files' unwrap counts came entirely from `#[cfg(test)]` test modules and were cleared by the bulk codemod from item 3.1.
- [x] 2.7 `src/storage/advanced.rs` (25) and `src/persistence/wal.rs` (32): same pattern — every unwrap was test-scoped and is now covered by the file-level `#![allow]` from `scripts/add_file_unwrap_allow.py`.
- [x] 2.8 Sweep of files below 25 hits: cleared in three commits — see `git log --oneline release/v3.0.0 ^main -- src/`. Notable per-file fixes (production code):
  * NonZeroU32/NonZeroUsize literals → module-level `const … = match Some(n) => n, None => unreachable!()` (rate_limit, advanced_cache, query_cache, file_operations/cache).
  * `Regex::new(r"…").unwrap()` on `&'static str` patterns → function-level `#[allow]` with rationale (normalization/detector, server/core/helpers).
  * `partial_cmp(...).unwrap()` on f32/f64 → `unwrap_or(Ordering::Equal)` everywhere it appeared (intelligent_search/mcp_tools, ml/advanced_ml, benchmark/utils, benchmark/metrics, benchmark/reporter, file_operations/operations, hybrid_search, intelligent_search/simple_search_engine, search/advanced_search/{engine,ranker}).
  * `SystemTime::now().duration_since(UNIX_EPOCH).unwrap()` → `.map(...).unwrap_or(0)` (replication/{master,replica,sync,replication_log}, hub/{request_signing,mcp_gateway}, processing/advanced_pipeline, search/advanced_search/engine, security/enhanced_security, ml/advanced_ml).
  * `serde_json::to_value(...).unwrap()` → `.map_err(...)?` (intelligent_search/mcp_server_integration, umicp/transport).
  * `.try_into::<[u8;4]>().unwrap()` on `chunks_exact(4)` → explicit `[chunk[0], chunk[1], chunk[2], chunk[3]]` (embedding/cache).
  * `.last().unwrap()` / `.front().unwrap()` after `is_empty()` early-return → `let Some(_) = ... else { return; }` (workspace/manager, normalization/cache/metrics, replication/replication_log).
  * Genuine option extraction → `let Some(_) = … else { … }` (db/vector_store/collections, server/qdrant/{vector,query}_handlers, migration/hub_migration).
  * Cryptographic / rng / serialization-of-known-types unwraps → function-level `#[allow]` + `// SAFE:` (auth/jwt_secret, auth/persistence, hub/request_signing::compute_signature, normalization/hasher).
  * Bench / CLI / examples binaries → file-level `#![allow]` (benches/multi_tenant_overhead, benchmark/{filter,grpc,comparison}/*, bin/vectorizer-cli, intelligent_search/examples).

## 3. Test modules

- [x] 3.1 Built `scripts/add_test_unwrap_allow.py` (matches any `#[cfg(test)] mod <name> { … }` block whose body contains `.unwrap[_err]()` or `.expect[_err](...)`) plus `scripts/add_file_unwrap_allow.py` (file-level `#![allow]` for `*_tests.rs` and every file under `tests/`). Together they patched ~120 source files and ~90 integration-test files. `src/persistence/tests.rs` is `include!()`-included and instead got the allow on the wrapping `mod persistence_tests`.

## 4. Enforcement flip

- [x] 4.1 `Cargo.toml` `[lints.clippy]` now sets `unwrap_used = "deny"` and `expect_used = "deny"` with a comment block referencing this task. The lint applies workspace-wide; opt-outs require an explicit `#[allow(...)]` with a `// SAFE:` rationale.
- [x] 4.2 `cargo clippy --all-targets --all-features` returns clean (zero warnings, zero errors). Verified by injecting a deliberate `x.unwrap()` into `src/gpu_adapter.rs` and confirming clippy reports `error: used 'unwrap()' on an 'Option' value`, then reverting.

## 5. Integration tests

- [x] 5.1 `tests/integration/handler_robustness.rs` covers the canonical malformed-input shapes that previously could surface a stray production `.unwrap()`: malformed `CollectionConfig` JSON (six variants), out-of-range timestamps, NaN scores in ranking sorts, NaN floats in `Number::from_f64` metadata serialisation, malformed sparse-vector indices, and dimension-mismatch fixtures. 8/8 tests passing.
- [x] 5.2 The same file covers the `pre_epoch_timestamp_falls_back_to_zero` and `empty_score_slice_does_not_panic_in_sort` edge cases — together with the lint enforcement (item 4.2), this is the regression guard for any future `.unwrap()` reintroduction.

## 6. Tail (mandatory)

- [x] 6.1 Update or create documentation covering the implementation: `.rulebook/specs/RUST.md` § "Enforcement" rewritten to reflect the now-active clippy lint, the `// SAFE:` + `#[allow]` opt-out pattern, and the regression-test surface. `CHANGELOG.md` Unreleased/Added entry summarises the workspace-wide flip and the fix catalog. The audit script + classifier (`scripts/classify_unwraps.py`) and the codemod scripts (`scripts/add_test_unwrap_allow.py`, `scripts/add_file_unwrap_allow.py`) are checked in for future re-runs.
- [x] 6.2 Write tests covering the new behavior: new `tests/integration/handler_robustness.rs` (8 tests) pins the contract for malformed JSON, NaN scores, NaN floats in metadata, out-of-range timestamps, pre-epoch system clocks, malformed sparse-vector indices, dimension mismatches, and empty score-slice sorts. Existing `tests/integration/{cluster_hybrid_search,distributed_search,hybrid_search}` (25 tests, 1 pre-existing ignore) continue to pass. The clippy lint itself (`unwrap_used = "deny"` / `expect_used = "deny"`) is the compile-time regression guard.
- [x] 6.3 Run tests and confirm they pass: `cargo test --lib --all-features` → 1158/1158 pass (12 pre-existing ignores). `cargo clippy --all-targets --all-features` → clean. `cargo fmt` → clean. `cargo test --test all_tests -- integration::handler_robustness::` → 8/8 pass.
