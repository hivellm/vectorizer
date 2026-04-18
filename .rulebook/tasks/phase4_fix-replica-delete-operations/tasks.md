## 1. Diagnosis

- [ ] 1.1 Time every stage of `Collection::delete` on the replica (payload index, sparse index, storage, vector_order, HNSW `index.write()`) and identify the >1s hot spot.
- [ ] 1.2 Confirm whether the sync `delete` call itself returns quickly and the slowness is elsewhere (e.g. ACK write blocking on the single TCP stream), or whether `delete` is actually slow.

## 2. Fix

- [ ] 2.1 Apply the smallest viable fix based on 1.1: either wrap the sync delete in `tokio::task::spawn_blocking`, split the replica stream into read/write halves so stream reads do not serialize behind ACK writes, or fix the underlying HNSW / collection slow path.
- [ ] 2.2 Un-ignore `tests/replication/integration_basic.rs::test_replica_delete_operations`.

## 3. Tail (mandatory)

- [ ] 3.1 Update `docs/architecture/replication.md` covering the replica apply-loop invariants (never hold read side blocked by the write side).
- [ ] 3.2 Re-enabled `test_replica_delete_operations` is the regression guard.
- [ ] 3.3 `cargo test --test all_tests replication::integration::integration_basic -- --test-threads=1` green, 0 ignored.
