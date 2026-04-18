## 1. Audit

- [x] 1.1 Grep `src/` for `.unwrap()` and `.expect(`; produce `design.md` with a per-file count and classification (replace-with-?, keep-with-justification, rewrite) — current audit: ~1,430 occurrences, top offenders being `server/mcp_tools.rs` (31), `monitoring/metrics.rs` (30), `file_watcher/hash_validator.rs` (30), `db/collection.rs` (29).
- [x] 1.2 Grep `src/` for `.ok()` chained after `Result`-returning calls; flag silent-error sites — six hits in `vector_store.rs`, only one (L1650 `get_collection_for_owner` alias resolution) was actually swallowing a meaningful error; rest are legitimate (filter-map, non-fatal dir check, test cleanup).

## 2. Fix by file (sequential, 1-2 files per sub-step)

- [x] 2.1 Rewrite `src/server/rest_handlers.rs` (or its successor under `handlers/`): every parse/lookup returns `Result<_, VectorizerError>` via `?` — current `rest_handlers.rs` has **zero** `.unwrap()`/`.expect(` occurrences, so no rewrite needed. Silent-error patterns on handler entry points are already covered by the axum Json extractor returning 400 on malformed bodies (verified by new `tests/infrastructure/handler_robustness.rs`).
- [x] 2.2 Rewrite `src/db/vector_store.rs` alias-resolution: `.ok()` → explicit error mapping. Fixed `get_collection_for_owner` (L1650) to match against the Result and log the error reason at `debug!` level before returning None, so operational issues (lock poison, corrupt alias table) stay visible.
- [ ] 2.3 Rewrite `src/server/mod.rs` bootstrap: replace `.unwrap()` with `?` in `init_*` functions — the bootstrap sweep is substantial (16 hits) and pairs naturally with the crate-wide sweep; tracked under `phase4_enforce-no-unwrap-policy`.
- [ ] 2.4 Sweep remaining offender files reported in 1.1 — tracked under `phase4_enforce-no-unwrap-policy` with explicit per-file priority list.

## 3. Enforcement

- [ ] 3.1 Add `clippy.toml` entries: `unwrap_used = "deny"`, `expect_used = "deny"`, `panic = "deny"` — must land after the crate-wide sweep lands, otherwise `cargo clippy -- -D warnings` (already used in CI) fails with ~1,000 errors immediately. Tracked under `phase4_enforce-no-unwrap-policy` step §4.
- [ ] 3.2 Run `cargo clippy --all-targets -- -D warnings` and confirm zero hits in `src/` non-test paths — gated on 3.1, same follow-up task.

## 4. Tail (mandatory — enforced by rulebook v5.3.0)

- [x] 4.1 Update `/.rulebook/specs/RUST.md` with the unwrap policy specifics — new "The unwrap/expect policy (tightened in phase3)" section with forbidden/allowed forms, `// SAFE:` convention, preferred alternatives table, silent-.ok() anti-pattern, and enforcement-status note.
- [x] 4.2 Add integration tests proving malformed paths/bodies return 4xx (not 500, not crash) — `tests/infrastructure/handler_robustness.rs` covers malformed JSON, missing required field, wrong content-type, empty body via the axum Json extractor. 4/4 pass.
- [x] 4.3 Run `cargo test --all-features` and confirm all tests pass — 1120/1120 lib, 780/780 integration, both CI gate scripts clean.

## Mandatory tail (required by rulebook v5.3.0)

- [x] Update or create documentation covering the implementation
- [x] Write tests covering the new behavior
- [x] Run tests and confirm they pass
