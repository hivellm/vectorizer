## 1. Reproduction

- [x] 1.1 Run `cargo test --test all_tests integration::sparse_vector::test_sparse_vector_search -- --ignored --nocapture` and confirm the race — could not reproduce. Against the current `release/v3.0.0` tree the test passes both in isolation (`--ignored --nocapture` → 1 passed) and alongside its siblings (`integration::sparse_vector -- --ignored` → 6/6 passed). The collision that triggered the original ignore was evidently fixed indirectly by the v3.0.0 locking / parking_lot migrations.

## 2. Fix

- [x] 2.1 Refactor the test to use a dedicated `VectorStore` / tempdir — the test was already using a fresh `VectorStore::new()` plus a timestamp-suffixed collection name, so no refactor was necessary once the underlying collision was gone. Added an inline comment documenting why the ignore was removed so the next reviewer understands the reasoning.
- [x] 2.2 Remove the `#[ignore]` — done.

## 3. Tail (mandatory — enforced by rulebook v5.3.0)

- [x] 3.1 Update `docs/development/testing.md` to remove the entry for this test — done. The Category-C table row for `tests/integration/sparse_vector.rs::test_sparse_vector_search` is gone.
- [x] 3.2 Test becomes the regression guard for sparse-vector search.
- [x] 3.3 Run `cargo test --test all_tests integration::sparse_vector` without `--ignored` — 14 passed, 5 ignored (the 5 remaining `#[ignore]`s in that file are tracked separately under "Sparse vector X has issues" and are out of scope for this task).

## Mandatory tail (required by rulebook v5.3.0)

- [x] Update or create documentation covering the implementation
- [x] Write tests covering the new behavior
- [x] Run tests and confirm they pass
