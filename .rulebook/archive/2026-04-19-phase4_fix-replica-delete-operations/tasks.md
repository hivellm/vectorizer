## 1. Diagnosis

- [x] 1.1 Time every stage of `Collection::delete` on the replica (payload index, sparse index, storage, vector_order, HNSW `index.write()`) and identify the >1s hot spot.
- [x] 1.2 Confirm whether the sync `delete` call itself returns quickly and the slowness is elsewhere (e.g. ACK write blocking on the single TCP stream), or whether `delete` is actually slow.

## 2. Fix

- [x] 2.1 Apply the smallest viable fix based on 1.1: either wrap the sync delete in `tokio::task::spawn_blocking`, split the replica stream into read/write halves so stream reads do not serialize behind ACK writes, or fix the underlying HNSW / collection slow path.
- [x] 2.2 Un-ignore `tests/replication/integration_basic.rs::test_replica_delete_operations`.

## 3. Tail (mandatory — enforced by rulebook v5.3.0)

- [x] 3.1 Update or create documentation covering the implementation: `docs/architecture/replication.md` gains invariant **I6** documenting the shared-DashMap-shard rule for vector-level mutations; the obsolete "Replica's apply loop holds the read side and the write side..." Known limitation is removed since the underlying contention is now fixed. Knowledge entry `code/sync-mutations-holding-exclusive-dashmap-shard-lock-starve-concurrent-readers-and-apply-loops` and learning `2026-04-19-replica-delete-bug-was-dashmap-shard-contention-not-slow-hnsw-or-stream-serialization` capture the diagnostic + anti-pattern for the next person who hits this.
- [x] 3.2 Write tests covering the new behavior: re-enabled `tests/replication/integration_basic.rs::test_replica_delete_operations` (previously `#[ignore]`) is the end-to-end regression guard. Test was hardened to bound both `let collection = replica_store.get_collection(...)` `Ref` lifetimes inside `{ }` blocks so future test edits don't accidentally re-introduce the same shard contention.
- [x] 3.3 Run tests and confirm they pass: `cargo test --test all_tests replication::integration::integration_basic -- --test-threads=1` → 13/13 pass, 0 ignored. `cargo test --test all_tests replication:: -- --test-threads=1` → 44 pass, 10 pre-existing ignores from unrelated suites. `cargo clippy --workspace --all-targets --all-features -- -D warnings` → 0 warnings. `cargo +nightly fmt -- src/db/vector_store/vectors.rs src/replication/replica.rs src/db/collection/data.rs tests/replication/integration_basic.rs` → clean.
