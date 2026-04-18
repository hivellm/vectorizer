# Proposal: phase4_fix-wal-multi-collection-replay

## Why

`phase4_triage-wal-recovery-bugs` un-ignored 7 of the 11 previously-
ignored WAL tests (they already passed). 4 tests remain legitimately
broken and they all exercise the same code path — **multi-collection
WAL replay**:

- `tests/core/wal_crash_recovery.rs::test_wal_recover_all_collections` — hangs.
- `tests/core/wal_vector_store.rs::test_vector_store_wal_integration` — fails.
- `tests/core/wal_vector_store.rs::test_wal_recover_all_collections_empty` — hangs (>60s).
- `tests/core/wal_vector_store.rs::test_wal_recover_all_collections_with_data` — hangs (>60s).

The single-collection insert / update / delete recovery flows work
fine (the 3 `wal_crash_recovery::test_wal_crash_recovery_*` tests
are green); so is multi-operation replay inside one collection
(`wal_comprehensive` suite green). The bug is isolated to the
"recover multiple collections" path.

Likely culprit is either:

- A lock held across an await inside the per-collection apply loop,
  causing the second collection to wait on the first forever.
- An unbounded channel or file-seek that never returns EOF when
  switching collections.
- An ordering assumption in `WalIntegration::recover_all_collections`
  that expects single-collection state.

## What Changes

1. Run each of the 4 failing tests under `--nocapture` + `--test-threads=1`
   with tracing enabled (`RUST_LOG=trace`) and capture the exact hang
   point.
2. Identify the call stack at the hang: `WAL::replay`,
   `WalIntegration::recover_all_collections`, or the per-collection
   apply loop.
3. Patch `src/persistence/wal.rs` and/or `src/db/wal_integration.rs`
   with minimal scope; keep the on-disk WAL format compatible so
   existing production data still replays.
4. Un-ignore the 4 tests once they pass.
5. Add `docs/architecture/wal.md` (or extend it) documenting the
   multi-collection replay invariants so this regression class
   doesn't return.

## Impact

- Affected specs: WAL / persistence spec.
- Affected code: `src/persistence/wal.rs`, `src/db/wal_integration.rs`,
  and the 4 test files once they un-ignore.
- Breaking change: NO — the fix must preserve on-disk compatibility.
- User benefit: multi-collection deployments recover cleanly after a
  crash instead of hanging on replay.
