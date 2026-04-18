## 1. Reproduction

- [ ] 1.1 Run `cargo test --test wal_crash_recovery -- --ignored --nocapture` and capture the hang / failure output for each of the four tests.
- [ ] 1.2 Run `cargo test --test wal_comprehensive -- --ignored --nocapture` and capture the four hangs / failures.
- [ ] 1.3 Identify which call stack hangs or asserts — typically one of: `WAL::replay`, `WalIntegration::recover_all_collections`, or the per-collection apply loop.

## 2. Fix

- [ ] 2.1 Root-cause the hang: is it a lock held across await, an unbounded channel, or a file-seek that never returns EOF?
- [ ] 2.2 Patch `src/persistence/wal.rs` (and `wal_integration.rs` if relevant) with minimal scope — keep the file format on-disk compatible.
- [ ] 2.3 Remove `#[ignore]` from every test now passing; leave it only on tests that remain genuinely slow (move those to nightly).

## 3. Tail (mandatory — enforced by rulebook v5.3.0)

- [ ] 3.1 Update `docs/architecture/wal.md` (or create) describing the recovery invariants.
- [ ] 3.2 The un-ignored tests ARE the regression tests; they must pass under `cargo test --test wal_crash_recovery` and `cargo test --test wal_comprehensive` without `--ignored`.
- [ ] 3.3 Run `cargo test --all-features` and confirm the newly-active tests pass.

## Mandatory tail (required by rulebook v5.3.0)

- [ ] Update or create documentation covering the implementation
- [ ] Write tests covering the new behavior
- [ ] Run tests and confirm they pass
