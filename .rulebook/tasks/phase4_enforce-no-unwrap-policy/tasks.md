## 1. Audit

- [x] 1.1 Dumped `grep -rnE '\.unwrap\(\)|\.expect\(' src/ --include='*.rs' > scripts/design-unwrap-audit.txt` (1456 hits) and built `scripts/classify_unwraps.py` to split by call-site scope. Latest classification: **1033 test-only** in 106 files (need only `#[allow]`), **233 production-code** in 67 files (need fixes / SAFE comments).
- [ ] 1.2 Same for `.ok()` chains that discard Err: classify (convert to `?` / legit filter / test-only).

## 2. Fix passes (top offender files first; 1-2 files per sub-step)

- [x] 2.1 `src/server/mcp/tools.rs` (was 31 hits) — replaced every `json!({...}).as_object().unwrap().clone().into()` with a new local `schema(value)` helper that pattern-matches `Value::Object(map)` and `unreachable!()`s on the impossible arm. Zero `.unwrap()` calls remain in this file.
- [x] 2.2 `src/monitoring/metrics.rs` (30 hits) — every unwrap is inside `Metrics::new()` constructing prometheus collectors from static `&'static str` names, which can only fail on malformed names (compile-time-checked). Added a doc block + function-level `#[allow(clippy::unwrap_used)]` documenting the static-invariant rationale. No semantic change.
- [x] 2.3 `src/file_watcher/hash_validator.rs` (30 hits) — every unwrap is inside the `#[cfg(test)] mod tests` block. Added `#[allow(clippy::unwrap_used, clippy::expect_used)]` to that block; production code is unwrap-free.
- [x] 2.4 `src/db/collection.rs` was split into `src/db/collection/{mod,data,graph,index,persistence,quantization}.rs`; only 2 unwraps remain in `mod.rs::Collection::new`. Documented both with `// SAFE:` rationale + `#[allow(clippy::expect_used)]` (HNSW from validated config = static-invariant; mmap is genuine I/O — flagged for follow-up Result conversion).
- [x] 2.5 `src/server/hub_handlers/tenant.rs` (was 28 hits; 1 prod, rest test) — annotated the lone prod hit (`as_array_mut().unwrap()` on a literal `json!` two lines above) with a SAFE comment + `#[allow]`; the test mod got the bulk allow.
- [ ] 2.6 `src/utils/file_hash.rs` (27 hits) + `src/quantization/storage.rs` (27 hits).
- [ ] 2.7 `src/storage/advanced.rs` + `src/persistence/wal.rs` (25 each).
- [ ] 2.8 Remaining offender files below 25 hits: sweep in batches of 2-3 per commit.

## 3. Test modules

- [x] 3.1 Built `scripts/add_test_unwrap_allow.py` — idempotent codemod that scans every `#[cfg(test)] mod tests { ... }` block and prepends `#[allow(clippy::unwrap_used, clippy::expect_used)]` if the body contains a `.unwrap()` / `.expect(...)` call. Initial run patched **110 files**; production-only unwraps now isolated to ~67 files (`scripts/classify_unwraps.py` re-run).

## 4. Enforcement flip

- [ ] 4.1 In `Cargo.toml` `[lints.clippy]` block: add `unwrap_used = "deny"`, `expect_used = "deny"`.
- [ ] 4.2 Run `cargo clippy --workspace --all-targets -- -D warnings` and confirm zero hits.

## 5. Integration tests

- [ ] 5.1 `tests/integration/handler_robustness.rs`: post malformed JSON to 10 endpoints; assert 400.
- [ ] 5.2 Same file: send missing path params, missing required headers, oversized bodies; assert 4xx.

## 6. Tail (mandatory)

- [ ] 6.1 Update `.rulebook/specs/RUST.md` — remove the "follow-up is pending" paragraph once the sweep is done.
- [ ] 6.2 Tests above cover the regression surface.
- [ ] 6.3 Run `cargo test --all-features` and confirm full pass.
