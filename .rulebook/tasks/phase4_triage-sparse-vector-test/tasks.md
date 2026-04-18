## 1. Reproduction

- [ ] 1.1 Run `cargo test --test all_tests integration::sparse_vector::test_sparse_vector_search -- --ignored --nocapture` and confirm the race.

## 2. Fix

- [ ] 2.1 Refactor the test to use a dedicated `VectorStore` / tempdir.
- [ ] 2.2 Remove the `#[ignore]`.

## 3. Tail (mandatory — enforced by rulebook v5.3.0)

- [ ] 3.1 Update `docs/development/testing.md` to remove the entry for this test.
- [ ] 3.2 Test becomes the regression guard for sparse-vector search.
- [ ] 3.3 Run `cargo test --test all_tests integration::sparse_vector` without `--ignored`.

## Mandatory tail (required by rulebook v5.3.0)

- [ ] Update or create documentation covering the implementation
- [ ] Write tests covering the new behavior
- [ ] Run tests and confirm they pass
