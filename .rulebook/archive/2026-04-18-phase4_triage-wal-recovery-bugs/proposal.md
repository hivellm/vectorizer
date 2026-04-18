# Proposal: phase4_triage-wal-recovery-bugs

## Why

The `phase4_reenable-ignored-tests` audit surfaced 9 `#[ignore]`d tests in `tests/core/wal_*.rs` that are marked "WAL recovery not working correctly" or "recovery operation hangs". These cover the core Write-Ahead-Log correctness surface — a bug here means the server silently drops operations on crash-restart. The tests were muted rather than fixed at some point and the bug went silent.

## What Changes

Root-cause and repair the WAL recovery path:

1. Reproduce locally using `cargo test --test wal_crash_recovery -- --ignored` and `cargo test --test wal_comprehensive -- --ignored`.
2. Identify whether the hang is inside `enable_wal` replay, `WalIntegration::recover_all_collections`, or the tokio timer integration.
3. Fix the underlying bug.
4. Remove the `#[ignore]` from every test now passing.
5. Leave `#[ignore]` only on tests that remain slow even when correct (>30 seconds) — those move to a nightly runner.

## Impact

- Affected specs: WAL / persistence spec
- Affected code: `src/persistence/wal.rs`, `src/persistence/wal_integration.rs` (if present), `src/db/vector_store.rs` recovery path
- Breaking change: NO (fixing latent bug)
- User benefit: correct crash recovery; operators no longer silently lose operations.
