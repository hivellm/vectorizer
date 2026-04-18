# Proposal: phase4_fix-replication-snapshot-sync

## Why

`tests/replication/integration_basic.rs` has 12 `#[ignore]`d tests, each with a variation of "Replication full sync issue - replica not receiving snapshot". This is the core of the master→replica replication contract — a replica that can't bootstrap from a master's snapshot can't rejoin the cluster after a restart, breaking the HA promise.

The tests are muted because the bug is real: running replication, the replica never observes the initial snapshot the master is meant to push.

## What Changes

1. Reproduce one of the simpler tests locally (e.g. `test_replica_full_sync_on_connect`).
2. Trace the master → replica handshake: who sends the snapshot, at which offset, how the replica acknowledges.
3. Identify where the snapshot is dropped (likely: master never sends it on first-connect, OR replica ignores it, OR the stream closes before completion).
4. Fix and re-enable the tests.

## Impact

- Affected specs: replication spec, HA spec
- Affected code: `src/replication/master.rs`, `src/replication/replica.rs`, possibly the Raft integration (`src/cluster/raft_node.rs`)
- Breaking change: NO (fixing latent bug)
- User benefit: functional replica bootstrap; HA mode actually HAs.
