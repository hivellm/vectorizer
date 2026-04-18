# Proposal: phase4_add-collection-name-to-raft-operations

## Why

`src/db/raft.rs` currently hardcodes `"default"` as the collection name at three sites (L131, L204, L208) because the `Operation` enum in `src/persistence/types.rs` carries no `collection_name` field. This means Raft replication silently routes every data-modifying operation to a single "default" collection — correct only for single-collection deployments.

A Raft cluster with multiple collections would miss-apply writes: an `Operation::InsertVector` meant for collection `A` would land in `"default"`. This task threads `collection_name` through the `Operation` enum so the state machine routes each log entry to the right collection.

## What Changes

1. Add `collection_name: String` to every data-modifying variant of `Operation` (`InsertVector`, `UpdateVector`, `DeleteVector`, `CreateCollection`, `DeleteCollection`) in `src/persistence/types.rs`.
2. Update all ~25 construction sites across `src/db/`, `src/persistence/`, tests to pass the real collection name.
3. Replace the three `"default"` hardcodes in `src/db/raft.rs` with the field from the payload.
4. Add a regression test that replicates two operations on two collections and asserts the state machine routed each correctly.

## Impact

- Affected specs: none (Raft is a BETA module with no frozen spec).
- Affected code: `src/persistence/types.rs`, `src/db/raft.rs`, `src/db/wal_integration.rs`, `src/db/vector_store.rs`, `src/persistence/wal.rs`, `src/persistence/dynamic.rs`, `src/persistence/demo_test.rs`, `src/persistence/enhanced_store.rs`.
- Breaking change: YES — `Operation` serialization changes. Raft log files written by older versions will fail to deserialize. Add a migration step or a version field.
- User benefit: multi-collection Raft clusters become correct instead of silently mis-routing writes.
