## 1. Audit

- [ ] 1.1 Grep `src/` for `.unwrap()` and `.expect(`; produce `design.md` with a per-file count and classification (replace-with-?, keep-with-justification, rewrite)
- [ ] 1.2 Grep `src/` for `.ok()` chained after `Result`-returning calls; flag silent-error sites

## 2. Fix by file (sequential, 1-2 files per sub-step)

- [ ] 2.1 Rewrite `src/server/rest_handlers.rs` (or its successor under `handlers/`): every parse/lookup returns `Result<_, VectorizerError>` via `?`
- [ ] 2.2 Rewrite `src/db/vector_store.rs` alias-resolution: `.ok()` → explicit error mapping
- [ ] 2.3 Rewrite `src/server/mod.rs` bootstrap: replace `.unwrap()` with `?` in `init_*` functions
- [ ] 2.4 Sweep remaining offender files reported in 1.1

## 3. Enforcement

- [ ] 3.1 Add `clippy.toml` (or `[lints.clippy]` in Cargo.toml) entries: `unwrap_used = "deny"`, `expect_used = "deny"`, `panic = "deny"` — with `#[allow]` on `#[cfg(test)]` only
- [ ] 3.2 Run `cargo clippy --all-targets -- -D warnings` and confirm zero hits in `src/` non-test paths

## 4. Tail (mandatory — enforced by rulebook v5.3.0)

- [ ] 4.1 Update `/.rulebook/specs/RUST.md` with the unwrap policy specifics
- [ ] 4.2 Add integration tests proving malformed paths/bodies return 4xx (not 500, not crash) — sample 10 endpoints
- [ ] 4.3 Run `cargo test --all-features` and confirm all tests pass

## Mandatory tail (required by rulebook v5.3.0)

- [ ] Update or create documentation covering the implementation
- [ ] Write tests covering the new behavior
- [ ] Run tests and confirm they pass
