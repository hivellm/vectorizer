## 1. Reproduction

- [x] 1.1 Run `cargo test --test storage -- --ignored --nocapture` and capture the failure output — reproduction failed. On the current `release/v3.0.0` tree both `test_mmap_insert_and_retrieve` and `test_mmap_update_and_delete` pass green, both under `--ignored --nocapture` in isolation and alongside their siblings (22/22 core::storage tests pass). The "fails locally" claim in the original ignore reason was evidently a symptom of a bug that got fixed indirectly by the v3.0.0 sprint's storage locking / mmap remap work.
- [x] 1.2 Read the assertion that fires — no assertion fires.

## 2. Fix

- [x] 2.1 Root-cause and patch `src/storage/mmap.rs` — no patch needed; both tests green against the current implementation.
- [x] 2.2 Remove `#[ignore]` from both tests once they pass — done.

## 3. Tail (mandatory — enforced by rulebook v5.3.0)

- [x] 3.1 Update `docs/architecture/storage.md` — not created; the mmap file format description belongs alongside a real bug fix (there wasn't one). The inline doc comments at `src/storage/mmap.rs` remain the source of truth. Removed the Category-C row from `docs/development/testing.md` so the doc no longer claims the tests are broken.
- [x] 3.2 The two un-ignored tests ARE the regression tests.
- [x] 3.3 Run `cargo test --test storage` and confirm both pass without `--ignored` — 22/22 pass in the core::storage suite; 783/783 in the full integration suite (was 781 before this + the sparse-vector un-ignore).

## Mandatory tail (required by rulebook v5.3.0)

- [x] Update or create documentation covering the implementation
- [x] Write tests covering the new behavior
- [x] Run tests and confirm they pass
