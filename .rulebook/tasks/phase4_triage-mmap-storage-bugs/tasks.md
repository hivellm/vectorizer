## 1. Reproduction

- [ ] 1.1 Run `cargo test --test storage -- --ignored --nocapture` and capture the failure output for both tests.
- [ ] 1.2 Read the assertion that fires to identify whether the bug is in `append`, `update`, `get`, or the remap-on-resize path.

## 2. Fix

- [ ] 2.1 Root-cause and patch `src/storage/mmap.rs`.
- [ ] 2.2 Remove `#[ignore]` from both tests once they pass.

## 3. Tail (mandatory — enforced by rulebook v5.3.0)

- [ ] 3.1 Update `docs/architecture/storage.md` (or create) describing the mmap file format and the append invariants.
- [ ] 3.2 The two un-ignored tests ARE the regression tests.
- [ ] 3.3 Run `cargo test --test storage` and confirm both pass without `--ignored`.

## Mandatory tail (required by rulebook v5.3.0)

- [ ] Update or create documentation covering the implementation
- [ ] Write tests covering the new behavior
- [ ] Run tests and confirm they pass
