# Proposal: phase4_fix-replica-delete-operations

## Why

Split off from `phase4_fix-replication-snapshot-sync`. The snapshot-sync
work re-enabled 11 of 12 previously-ignored replica integration tests;
one test (`test_replica_delete_operations`) still fails with a
different root cause that deserves a focused task.

Empirical trace (collected with temporary `eprintln!` in master + replica):

- Master's `replicate()` appends all 5 DeleteVector operations to the
  replication log (offsets 1..5) and dispatches to the per-replica
  channel, then `send_command_half` writes all 5 commands to the TCP
  stream successfully.
- Replica reads exactly **one** `Operation` command (offset 1), enters
  `apply_operation` -> `vector_store.delete("delete_test", "vec_0")`,
  applies successfully, then never returns to read the remaining four
  commands within the 10s test window.

So the apply path is finishing eventually (the "delete OK" debug line
prints post-panic) but is extremely slow, blocking the replica's read
loop long enough that subsequent deletes are never processed. Suspected
culprits: HNSW `index.write()` contention on the replica's collection
that was seeded by the snapshot, or a sync lock held across the single
stream read/write alternation.

## What Changes

1. Instrument `Collection::delete` timings on the replica path to
   confirm the hot spot (HNSW remove vs. DashMap remove vs. other).
2. Fix the blocking call - either:
   - move `vector_store.delete()` onto `spawn_blocking` so the tokio
     runtime can keep reading the stream, OR
   - split the replica stream into read/write halves (mirroring the
     master) so stream reads don't serialize behind ACK writes, OR
   - fix the underlying slow-path in the HNSW index if it's an actual
     O(n) / lock-contention bug.
3. Un-ignore `test_replica_delete_operations` and add a timing
   regression guard.

## Impact
- Affected specs: replication spec (delete path).
- Affected code: `src/replication/replica.rs` (apply loop), possibly `src/db/collection/data.rs` (delete) or `src/hnsw/*`.
- Breaking change: NO
- User benefit: deletes propagate to replicas in bounded time; HA deployments don't leak deleted data on the replica side.
