## 1. Reproduction + diagnosis

- [x] 1.1 Ran the previously-ignored multi-collection recovery test; observed assertion `left: 1, right: 2` — one collection's replay was silently dropped.
- [x] 1.2 Ran the three `wal_vector_store` tests; `_empty` and `_with_data` pass, `test_vector_store_wal_integration` failed on an unrelated Cosine-metric assertion (normalization hid the update).
- [x] 1.3 Traced to `WriteAheadLog::recover()` — it validated per-collection sequences as dense `0..N`, but `append()` uses a single global `AtomicU64`, so per-collection sequences are strictly monotonic but sparse.

## 2. Fix

- [x] 2.1 Patched `src/persistence/wal.rs::recover()` to validate strict monotonicity instead of dense indexing — on-disk format unchanged.
- [x] 2.2 Un-ignored all 4 tests; switched `test_vector_store_wal_integration` from Cosine to Euclidean metric so `data[0] == 3.0` is a meaningful roundtrip check.

## 3. Tail (mandatory)

- [x] 3.1 Added `docs/architecture/wal.md` covering the global-sequence invariant, the recovery order guarantees, and the lock discipline.
- [x] 3.2 Added unit-level regression guard `persistence::wal::tests::test_wal_recover_multi_collection_sparse_sequences` covering sparse sequences directly at the WAL layer.
- [x] 3.3 `cargo test --test all_tests core::wal --test-threads=1` → 15/15 pass.
