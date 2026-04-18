## 1. Enum extension

- [ ] 1.1 Add `collection_name: String` to `Operation::InsertVector`, `UpdateVector`, `DeleteVector`, `CreateCollection`, `DeleteCollection` in `src/persistence/types.rs`.
- [ ] 1.2 Bump the log format version constant so old Raft logs are rejected with a clear error instead of silently mis-deserializing.

## 2. Callsite migration

- [ ] 2.1 Update all constructors in `src/db/wal_integration.rs`, `src/db/vector_store.rs`, `src/persistence/wal.rs`, `src/persistence/dynamic.rs`, `src/persistence/demo_test.rs`, `src/persistence/enhanced_store.rs` to pass the real collection name.
- [ ] 2.2 Replace the three `"default"` hardcodes in `src/db/raft.rs` (formerly L131, L204, L208) with the payload field.

## 3. Tests

- [ ] 3.1 Regression test: apply `InsertVector` to collection A and `InsertVector` to collection B through the state machine; assert each landed in the correct collection.
- [ ] 3.2 Migration test: attempt to load a pre-upgrade log file and assert a clear version-mismatch error.

## 4. Tail (mandatory)

- [ ] 4.1 Update `README.md` / architecture doc with the Raft log format version bump.
- [ ] 4.2 Tests above cover the new behavior.
- [ ] 4.3 Run `cargo test --all-features` and confirm pass.
