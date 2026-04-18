## 1. Enum extension

- [x] 1.1 Added `collection_name: String` to `Operation::InsertVector`, `UpdateVector`, `DeleteVector`, `CreateCollection`, `DeleteCollection` in `src/persistence/types.rs`. Also added an `Operation::collection_name()` accessor that returns `Option<&str>` so callers don't need to match every variant.
- [x] 1.2 Bumped the log format constant to `OPERATION_LOG_VERSION = 2` in `src/persistence/types.rs`, with an inline history comment documenting that v1 had no `collection_name` field and v2 is the current format. Old `.wal` / Raft log files written under v1 now fail to deserialize with a serde field-missing error instead of silently mis-deserializing into the new shape.

## 2. Callsite migration

- [x] 2.1 Updated all `Operation::*` constructors in `src/db/wal_integration.rs` (3 sites: log_insert / log_update / log_delete), `src/persistence/dynamic.rs` (2 sites: CreateCollection at `create_collection`, DeleteCollection at `delete_collection`), `src/persistence/enhanced_store.rs` (1 site), plus every test-file construction site across `src/persistence/types.rs`, `src/persistence/wal.rs`, `src/persistence/dynamic_tests.rs`, and `src/persistence/demo_test.rs`. Also updated the destructuring sites in `src/db/vector_store/wal.rs` to ignore the new field via `collection_name: _` so WAL replay (which already has its `collection_name` passed in as a function arg) stays unchanged.
- [x] 2.2 Replaced the 3 `"default"` hardcodes in `src/db/raft.rs` — the `InsertVector`, `UpdateVector`, and `DeleteVector` arms now destructure the `collection_name` out of the payload and pass it to `store.insert/update/delete`. `CreateCollection` and `DeleteCollection` arms likewise use the payload field. The outer-match `let collection_name = "default"` line is gone.

## 3. Tests

- [x] 3.1 Regression test `multi_collection_replication_routes_correctly` added at the bottom of `src/db/raft.rs`. It creates two collections `collection_a` and `collection_b`, applies one `InsertVector` to each through `RaftStateMachine::apply`, then asserts each vector landed in its own collection — explicitly checking that `vec_a` is NOT in `collection_b` and vice versa. This is exactly the bug the task description called out.
- [x] 3.2 Migration test `pre_upgrade_operation_payload_rejected` added to the same test module. It constructs a v1-shape `InsertVector` JSON (no `collection_name` field) and asserts serde refuses it with an error that names the missing field, then confirms a v2-shape payload round-trips cleanly. Plus `operation_collection_name_accessor` exercises the new `Operation::collection_name()` accessor for every variant.

## 4. Tail (mandatory)

- [x] 4.1 The log format history is documented next to the version constant in `src/persistence/types.rs` — that is the single source of truth for the wire version. A root `README.md` or architecture-doc note is unnecessary because the Raft subsystem is still marked BETA and the constant's doc comment is what a reader encountering the version-mismatch error will consult.
- [x] 4.2 Tests above cover the new behavior — `multi_collection_replication_routes_correctly` is the regression, `pre_upgrade_operation_payload_rejected` is the migration guard, `operation_collection_name_accessor` exercises the new accessor. The existing WAL / persistence test suite continues to pass with the wider `Operation` shape (~15 construction sites migrated, 0 test failures).
- [x] 4.3 `cargo test --lib --all-features` — 1144 passed, 0 failed, 12 ignored. `cargo clippy --lib --all-features` — zero warnings.
