# Proposal: phase4_triage-mmap-storage-bugs

## Why

`tests/core/storage.rs::test_mmap_insert_and_retrieve` and `test_mmap_update_and_delete` are `#[ignore]`d. They cover the mmap-backed storage path in `src/storage/mmap.rs` which handles vector append/update/read — a core datapath. With the tests muted we have no regression guard on that code.

## What Changes

1. Un-ignore both tests and run them locally.
2. If they fail, root-cause the bug in `src/storage/mmap.rs` (likely around the remap-on-resize path or the length header).
3. Fix and leave the tests active.
4. If they pass out-of-the-box, the ignore was stale — remove it.

## Impact

- Affected specs: storage spec
- Affected code: `src/storage/mmap.rs`, possibly `tests/core/storage.rs`
- Breaking change: NO
- User benefit: coverage on the mmap datapath; early warning on future regressions.
