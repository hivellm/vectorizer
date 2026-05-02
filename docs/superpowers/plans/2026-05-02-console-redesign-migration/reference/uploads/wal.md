# Write-Ahead Log (WAL) Architecture

The WAL (Write-Ahead Log) is Vectorizer's durability mechanism. Every
mutating operation (insert, update, delete) is appended to the WAL
before being applied to the in-memory collection, so that a crash can
be recovered by replaying the log at startup.

This document captures the invariants that the multi-collection WAL
relies on. Two regressions in recent history were caused by violating
one of these invariants, so the rules are enumerated here explicitly.

## Files

- `src/persistence/wal.rs` — `WriteAheadLog` (file I/O, sequencing,
  checkpointing, recovery).
- `src/db/wal_integration.rs` — `WalIntegration` (thin async wrapper
  called from `VectorStore`).
- `src/db/vector_store/wal.rs` — `VectorStore` methods that log writes
  and run recovery at startup.

## On-disk format

The WAL is a single append-only JSON-Lines file per data directory
(`vectorizer.wal`). Each line is a serialized [`WALEntry`]:

```json
{
  "sequence": 42,
  "timestamp": "2026-04-18T10:30:00Z",
  "operation": { "InsertVector": { "collection_name": "col1", ... } },
  "collection_id": "col1",
  "transaction_id": null
}
```

Because the file contains entries for every collection intermixed, the
per-collection recovery path filters by `collection_id` while reading.

## Sequence numbers

`WriteAheadLog` holds a single `AtomicU64` sequence counter that is
incremented once per appended entry. **Sequences are global to the WAL
file, not per-collection.** This has two consequences:

1. The sequence of entries for any single collection is **strictly
   monotonically increasing** but **not dense**. For example, with two
   collections interleaved:

   ```
   seq=0  col1  insert v1
   seq=1  col2  insert v2
   seq=2  col1  insert v3
   ```

   `recover("col1")` returns entries with sequences `[0, 2]`. Sequence
   `1` is missing because it belongs to `col2`.

2. `recover()` must therefore validate strict monotonicity, not dense
   `0..N` indexing. Validating dense indexing (`entry.sequence == i`)
   is a bug — it will reject legitimate multi-collection WALs with
   `WALError::InvalidSequence` and leave collections unrecovered.

This is why [`recover()`](../../src/persistence/wal.rs) checks
`entry.sequence > previous` instead of `entry.sequence == i`. A
regression test lives at
`persistence::wal::tests::test_wal_recover_multi_collection_sparse_sequences`.

## Locking

The inner WAL file handle uses `tokio::sync::Mutex` (not parking_lot)
because all file I/O crosses `.await` boundaries. The `VectorStore`
side uses `parking_lot::Mutex` around `Option<WalIntegration>`, but
releases the guard before entering `async` code paths — never hold a
`parking_lot` guard across `.await`.

## Recovery

Startup recovery runs in `VectorStore::recover_all_from_wal`:

1. List all collections known to the store.
2. For each collection name, call `recover_and_replay_wal`, which
   reads the entries via `WriteAheadLog::recover(collection_name)`
   and reapplies insert / update / delete operations against the
   in-memory state.
3. A failed replay for one collection **must not** abort the whole
   recovery — the error is logged as a warning and the loop continues
   with the next collection. This keeps one corrupt collection from
   taking the entire node down.

Collections must exist (be created with `create_collection`) before
`recover_and_replay_wal` can apply inserts/updates/deletes to them.
Callers of `recover_all_from_wal` are expected to re-materialize
collections from their persisted metadata first and then call
`recover_all_from_wal` to re-apply the tail of unflushed operations.

## Checkpoint

`checkpoint()` truncates the WAL file once a durable snapshot has been
persisted. The current sequence counter is **not** reset — monotonicity
across a checkpoint is preserved so replaying a partially-checkpointed
log still orders correctly against newly-appended entries.

## Invariants (summary)

The recovery path breaks if any of these is violated:

- **I1.** Sequences are assigned from a single global atomic,
  incrementing by one per appended entry.
- **I2.** Per-collection filtering produces a strictly monotonic
  (not necessarily dense) subsequence.
- **I3.** `recover()` validation treats sequences as strictly
  monotonic, never dense.
- **I4.** Replay failures for a single collection are logged and
  skipped, not propagated, so multi-collection deployments recover
  as much as possible after a crash.
- **I5.** No `parking_lot::Mutex` guard is held across `.await`.
