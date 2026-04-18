## 1. Preparation

- [ ] 1.1 Create `src/server/handlers/` directory scaffold with empty submodules
- [ ] 1.2 Create `src/server/handlers/common.rs` with shared helpers (error-to-response, pagination, auth extractor)

## 2. Sequential migration (1-2 files per sub-step, verify compilation between)

- [ ] 2.1 Move collection handlers to `handlers/collections.rs`; update router imports; `cargo check`
- [ ] 2.2 Move vector handlers to `handlers/vectors.rs`; `cargo check`
- [ ] 2.3 Move search handlers to `handlers/search.rs`; `cargo check`
- [ ] 2.4 Move alias handlers to `handlers/aliases.rs`; `cargo check`
- [ ] 2.5 Move snapshot/backup handlers to `handlers/snapshots.rs`; `cargo check`
- [ ] 2.6 Move admin/setup/config handlers to `handlers/admin.rs`; `cargo check`
- [ ] 2.7 Move intelligent-search handlers to `handlers/intelligent_search.rs`; `cargo check`
- [ ] 2.8 Move graph handlers to `handlers/graph.rs`; `cargo check`
- [ ] 2.9 Move meta/health handlers to `handlers/meta.rs`; `cargo check`
- [ ] 2.10 Delete the (now empty) `rest_handlers.rs`

## 3. Verification

- [ ] 3.1 `cargo clippy --all-targets -- -D warnings` — zero warnings
- [ ] 3.2 Each new handler file ≤500 LOC (or justify in `design.md`)

## 4. Tail (mandatory — enforced by rulebook v5.3.0)

- [ ] 4.1 Update `docs/api/` index and CONTRIBUTING.md's "where do handlers live" pointer
- [ ] 4.2 Existing integration tests must continue to pass without modification (behavior unchanged); add a file-size regression test
- [ ] 4.3 Run `cargo test --all-features` and confirm all tests pass

## Mandatory tail (required by rulebook v5.3.0)

- [ ] Update or create documentation covering the implementation
- [ ] Write tests covering the new behavior
- [ ] Run tests and confirm they pass
