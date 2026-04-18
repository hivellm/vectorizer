## 1. Implementation

- [ ] 1.1 Migrate `src/batch/mod.rs` and `src/batch/processor.rs` (7 sites) — `std::sync::Mutex` → `parking_lot::Mutex`, drop `.unwrap()` on lock calls
- [ ] 1.2 Migrate `src/db/vector_store.rs` — `pending_saves`, `save_task_handle`, `wal`
- [ ] 1.3 Migrate `src/cluster/raft_node.rs` — `snapshot_idx`
- [ ] 1.4 Sweep remaining sites: `grep -rn 'std::sync::Mutex\|std::sync::RwLock' src/` → migrate each
- [ ] 1.5 Add a CI grep gate (in `.github/workflows/rust-lint.yml`) that fails if any new `std::sync::Mutex`/`std::sync::RwLock` appears in `src/`

## 2. Tail (mandatory — enforced by rulebook v5.3.0)

- [ ] 2.1 Update `AGENTS.md` / `/.rulebook/specs/RUST.md` if the exclusions list needs clarification (Arc/atomic/Once/OnceLock allowed)
- [ ] 2.2 Write a microbenchmark in `benches/locks.rs` comparing contended throughput before/after (document in design.md)
- [ ] 2.3 Run `cargo test --all-features` and confirm all tests pass (no deadlocks, no regressions)

## Mandatory tail (required by rulebook v5.3.0)

- [ ] Update or create documentation covering the implementation
- [ ] Write tests covering the new behavior
- [ ] Run tests and confirm they pass
