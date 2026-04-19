# Proposal: phase4_test-discovery-module

## Why

`src/discovery/` (file discovery and indexing pipeline) has **no dedicated test directory** or integration test file. The module is security-sensitive (see `phase2_sanitize-discovery-paths`) and performance-critical (it runs on every workspace change). Untested today means:

- Path-traversal protections (once added) have no enforcement.
- Filesystem walker has no regression test for symlink loops, large trees, permission errors.
- YAML manifest parsing has no fuzz or property tests.
- No test for concurrent re-indexing behavior.

## What Changes

Create `tests/discovery/` with:

1. `basics.rs` — happy-path discovery of a small fixture tree; asserts file count, content-hash stability.
2. `edge_cases.rs` — symlink loops, unreadable files, empty directories, hidden files, BOM / non-UTF8 names.
3. `path_traversal.rs` — attempts `../`, absolute, symlink-escape; expects rejection (pair with `phase2_sanitize-discovery-paths`).
4. `manifest_parse.rs` — valid/invalid/malformed `workspace.yml`; property-based tests using `proptest`.
5. `concurrent.rs` — concurrent discovery + file-watcher events; asserts no double-indexing, no lost events.
6. `fixtures/` — small tree of test files including exotic names.

## Impact

- Affected specs: discovery module spec
- Affected code: new `tests/discovery/*`, possibly small refactors in `src/discovery/` to expose test seams
- Breaking change: NO
- User benefit: confidence in correctness + security of the ingest pipeline.
