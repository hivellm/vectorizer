## 1. Preparation

- [ ] 1.1 After `phase3_split-rest-handlers-monolith` completes, inventory the remaining contents of `src/server/mod.rs` and draft the new file map in `design.md`

## 2. Sequential migration

- [ ] 2.1 Extract `AppState` + builders to `src/server/state.rs`; `cargo check`
- [ ] 2.2 Extract route buckets to `src/server/routes/{public,authenticated,admin,mcp}.rs`; `cargo check`
- [ ] 2.3 Extract startup sequence to `src/server/bootstrap.rs` (init functions per subsystem, each fallible); `cargo check`
- [ ] 2.4 Extract graceful shutdown logic to `src/server/shutdown.rs`; `cargo check`
- [ ] 2.5 Trim `src/server/mod.rs` to <300 LOC orchestrator

## 3. Verification

- [ ] 3.1 Replace `.unwrap()` calls in bootstrap with `?` returning boxed errors; no panics in the happy path
- [ ] 3.2 `cargo clippy --all-targets -- -D warnings` — zero warnings

## 4. Tail (mandatory — enforced by rulebook v5.3.0)

- [ ] 4.1 Update `docs/architecture/server-layer.md` with the new layout
- [ ] 4.2 Add boot-sequence unit tests for `bootstrap.rs` (config validation, subsystem init ordering)
- [ ] 4.3 Run `cargo test --all-features` and confirm all tests pass

## Mandatory tail (required by rulebook v5.3.0)

- [ ] Update or create documentation covering the implementation
- [ ] Write tests covering the new behavior
- [ ] Run tests and confirm they pass
