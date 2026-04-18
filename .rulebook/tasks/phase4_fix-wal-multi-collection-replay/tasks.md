## 1. Reproduction + diagnosis

- [ ] 1.1 Run `cargo test --test all_tests core::wal::wal_crash_recovery::test_wal_recover_all_collections -- --ignored --nocapture --test-threads=1` with `RUST_LOG=trace`; capture where the hang occurs.
- [ ] 1.2 Same for the three `wal_vector_store` tests.
- [ ] 1.3 Identify the call stack — typically one of: `WAL::replay`, `WalIntegration::recover_all_collections`, per-collection apply loop, or an unbounded channel.

## 2. Fix

- [ ] 2.1 Patch `src/persistence/wal.rs` + `src/db/wal_integration.rs` with minimal scope. Keep the on-disk WAL format compatible.
- [ ] 2.2 Un-ignore each of the 4 tests as it starts passing.

## 3. Tail (mandatory)

- [ ] 3.1 Add `docs/architecture/wal.md` covering the multi-collection replay invariants + the recovery order guarantees.
- [ ] 3.2 The 4 un-ignored tests ARE the regression tests.
- [ ] 3.3 Run `cargo test --test all_tests core::wal --test-threads=1` and confirm all previously-ignored tests now pass.
