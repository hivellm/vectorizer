## 1. Audit

- [ ] 1.1 Dump `grep -rnE '\.unwrap\(\)|\.expect\(' src/ --include='*.rs' > design-unwrap-audit.txt`; classify each hit (fix / safe / test).
- [ ] 1.2 Same for `.ok()` chains that discard Err: classify (convert to `?` / legit filter / test-only).

## 2. Fix passes (top offender files first; 1-2 files per sub-step)

- [ ] 2.1 `src/server/mcp_tools.rs` (31 hits) — MCP handler entry points MUST NOT panic.
- [ ] 2.2 `src/monitoring/metrics.rs` (30 hits) — lock acquisitions + env parse.
- [ ] 2.3 `src/file_watcher/hash_validator.rs` (30 hits).
- [ ] 2.4 `src/db/collection.rs` (29 hits).
- [ ] 2.5 `src/server/hub_tenant_handlers.rs` (28 hits) — handler entry points.
- [ ] 2.6 `src/utils/file_hash.rs` (27 hits) + `src/quantization/storage.rs` (27 hits).
- [ ] 2.7 `src/storage/advanced.rs` + `src/persistence/wal.rs` (25 each).
- [ ] 2.8 Remaining offender files below 25 hits: sweep in batches of 2-3 per commit.

## 3. Test modules

- [ ] 3.1 Add `#![allow(clippy::unwrap_used, clippy::expect_used)]` to every `#[cfg(test)] mod tests { ... }` once the lints flip to `deny` — unwrap is idiomatic in tests.

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
