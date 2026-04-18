## 1. Layout

- [ ] 1.1 Create `tests/integration/qdrant_api/` with one file per capability (points, collections, search, snapshots, filters, aliases).
- [ ] 1.2 Move the shared setup/teardown helpers into `mod.rs`.

## 2. Verification

- [ ] 2.1 `cargo test --test all_tests --all-features` passes with the same number of tests.
- [ ] 2.2 No sub-file exceeds 400 lines.

## 3. Tail (mandatory)

- [ ] 3.1 Document the new layout in `tests/integration/qdrant_api/mod.rs`.
- [ ] 3.2 Tests are the deliverable; no new tests required.
- [ ] 3.3 Full integration suite green.
