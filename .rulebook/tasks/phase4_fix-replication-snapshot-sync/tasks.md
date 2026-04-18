## 1. Reproduction

- [x] 1.1 Ran `cargo test --test all_tests replication::integration::integration_basic -- --ignored --test-threads=1`; 11 of 12 passed on first run, 1 (`test_replica_delete_operations`) still fails.
- [x] 1.2 Snapshot receive path verified - `test_replica_full_sync_on_connect`, `test_replica_partial_sync_on_reconnect`, `test_master_multiple_replicas_and_stats`, `test_large_payload_replication` all exercise snapshot delivery and pass. The "snapshot sync" ignore reasons on these tests were stale from an earlier bug already resolved.

## 2. Fix

- [x] 2.1 Traced master-side path (`replicate` -> `replication_tx` -> `start_replication_task` -> per-replica sender -> `send_command_half`). Split read/write halves + ACK reader task work end-to-end.
- [x] 2.2 Traced replica-side (`connect_and_sync` -> `receive_command` -> `apply_operation` -> `send_ack`). Snapshot path applies all collections and updates `state.offset` correctly.
- [x] 2.3 The snapshot-sync handshake itself is working; diagnosis documented in the new `docs/architecture/replication.md`. Remaining replica-delete issue split out to `phase4_fix-replica-delete-operations`.
- [x] 2.4 Un-ignored 11 integration_basic tests; kept `test_replica_delete_operations` ignored with a pointer to the follow-up task.

## 3. Tail (mandatory)

- [x] 3.1 Added `docs/architecture/replication.md` covering the handshake, sync strategy selection, live replication path, ACK + write-concern flow, snapshot wire format, 5 invariants, and known limitations (including the replica-delete limitation handed off to `phase4_fix-replica-delete-operations`).
- [x] 3.2 The 11 newly-active tests in `integration_basic.rs` are the regression guard.
- [x] 3.3 `cargo test --test all_tests replication::integration::integration_basic -- --test-threads=1` -> 12 passed, 1 ignored (follow-up), 0 failed.
