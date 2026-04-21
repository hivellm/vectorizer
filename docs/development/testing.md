# Testing strategy

This document catalogs the project's test suites, what runs when, and which
tests are currently `#[ignore]`d with the reason for each. It exists because
`phase4_reenable-ignored-tests` found ~40 ignored tests across the repo and
we needed a central record of why.

## What runs in CI today

| Suite | Command | Runs on |
|---|---|---|
| Library unit tests (default features) | `cargo test --lib` | every PR |
| Library unit tests (all features) | `cargo test --all-features --lib` | every PR |
| Doc tests | `cargo test --doc` | every PR |
| Integration tests | `cargo test --test all_tests` | every PR |
| gRPC tests | `cargo test --test grpc_integration` | every PR |
| Clippy (`-D warnings`) | `cargo clippy --all-targets -- -D warnings` | every PR |
| Rustfmt (nightly) | `cargo +nightly fmt --all -- --check` | every PR |
| Go SDK | `go vet ./... && go test -v -short ./...` | every PR |

Ignored tests are NOT run by any of the above jobs.

## Ignored-test taxonomy

Every `#[ignore]` in this repo falls into one of four categories. New ignores
added after 2026-04-18 MUST use the `#[ignore = "..."]` form with a reason
that names the category and (if applicable) the rulebook follow-up task.

### Category A — environment-dependent

The test requires external infrastructure that the default CI matrix doesn't
provide (running server, specific filesystem, elevated privileges). These
stay ignored forever; the reason string names the requirement.

| Test | Reason |
|---|---|
| `tests/grpc_s2s.rs::test_real_server_batch_search` | requires a real gRPC server listening on the s2s port |
| `tests/replication/failover.rs` (5 tests) | requires TCP-level replication harness |
| `tests/hub/failover_tests.rs` (4 tests) | requires running Vectorizer + HiveHub servers |
| `tests/core/persistence.rs` (2 tests) | requires specific filesystem setup (mtime, fsync semantics) |

### Category B — slow

The test takes long enough that running it per-PR would hurt the CI budget.
Candidate for a future nightly workflow gated by an env var.

| Test | Reason |
|---|---|
| `tests/core/wal_vector_store.rs::test_wal_recover_all_collections_empty` | >60 seconds; recovery operation hangs |
| `tests/core/wal_vector_store.rs::test_wal_recover_all_collections_with_data` | >60 seconds; recovery operation hangs |
| `tests/workflow/api_workflow.rs::test_full_crud_workflow` | timeout on CRUD workflow |

### Category C — known bug

The code-under-test has a real bug, tracked in a rulebook task. The ignore
remains until the fix lands. The reason string names the tracking task.

| Test | Tracking task | Summary |
|---|---|---|
| `tests/core/wal_vector_store.rs` (3 tests) | `phase4_triage-wal-recovery-bugs` | `test_vector_store_wal_integration` fails; `test_wal_recover_all_collections_{empty,with_data}` hang >60s |
| `tests/core/wal_crash_recovery.rs::test_wal_recover_all_collections` | `phase4_triage-wal-recovery-bugs` | hangs on multi-collection replay; insert/update/delete recover fine |
| `tests/replication/integration_basic.rs` (12 tests) | `phase4_fix-replication-snapshot-sync` | replica not receiving snapshot on initial sync |

### Category D — CI flaky

The test passes locally but fails in CI due to concurrency or environment
differences. Marked for investigation but not a priority over real bugs.

| Test | Reason |
|---|---|
| `tests/grpc_integration.rs::test_update_vector` | update operation fails in CI environment |
| `tests/grpc_comprehensive.rs` (2 tests) | update operation fails in CI environment |
| `tests/grpc/vectors.rs::test_update_vector` | update operation fails in CI environment |
| `tests/test_new_features.rs::test_collection_with_wal_disabled` | Vector ID conflict in CI environment |

## How to run ignored tests

- Single test: `cargo test --test <suite> <test_name> -- --ignored`
- All ignored in a suite: `cargo test --test <suite> -- --ignored`
- All ignored anywhere: `cargo test -- --ignored`

The last form is how you'd run the full test population locally when
investigating category C / D issues.

## Adding a new ignore

```rust
#[ignore = "<category>: <short reason>"]
```

Valid `<category>` values:

- `env` — Category A, needs external infrastructure
- `slow` — Category B, too slow for per-PR
- `bug:<task-id>` — Category C, tracked by a rulebook task
- `flaky` — Category D, investigation pending

Bare `#[ignore]` (no reason string) is considered a regression and will be
flagged by a future CI grep gate.

## Related rulebook tasks

- `phase4_reenable-ignored-tests` — the audit that produced this document
- `phase4_triage-wal-recovery-bugs` — Category C tests in `wal_*.rs`
- `phase4_triage-mmap-storage-bugs` — Category C tests in `storage.rs`
- `phase4_fix-replication-snapshot-sync` — 12 Category C tests in `replication/integration_basic.rs`
- `phase4_triage-sparse-vector-test` — single Category C test

## Discovery pipeline tests (`tests/discovery/`)

`src/discovery/` is the **search-result orchestration pipeline**
(filter → score → expand → broad → focus → compress → readme → plan
→ render). Despite the directory name it does NOT walk the
filesystem, parse manifests, or touch any path-traversal surface —
those concerns live in `src/file_loader/` and `src/file_watcher/`,
each tested in their own slot.

The `tests/discovery/` suite exercises the orchestration stages
end-to-end against an in-memory `VectorStore` plus a BM25-backed
`EmbeddingManager` (the same fixture pattern
`MCPToolHandler::new_with_store` uses). Five files, 22 tests:

| File | Coverage |
|---|---|
| `basics.rs` | `Discovery::discover` happy path; empty store; default exclude-list (`*-test`, `*-backup`) drops blacklisted collections. |
| `filter_score.rs` | `filter_collections` with no inputs / explicit include / exclude / no-match / empty input. `score_collections` ordering, empty-terms handling, empty-collections handling. |
| `expand.rs` | `expand_queries_baseline` with each toggle individually, all-on with `max_expansions` truncation, `max_expansions = 0`, stopword stripping in main-term extraction, empty query. |
| `compress.rs` | `compress_evidence` with empty / single chunk; per-doc cap; global `max_bullets` cap; descending-score ordering. |
| `concurrent.rs` | 16 concurrent `discover` calls against the same `Discovery` produce identical `collections_searched`; mixed-query concurrent calls don't interfere. |

All tests live under `tests/discovery/` (wired into
`tests/all_tests.rs::mod discovery`); run with
`cargo test --test all_tests discovery`.

The original task spec (`phase4_test-discovery-module`) listed
`edge_cases.rs` (symlink loops, unreadable files), `path_traversal.rs`,
and `manifest_parse.rs` (YAML proptest). Those did not survive the
recon pass — the module has zero filesystem code (`grep -rn
"fn read_dir|symlink|workspace\.yml|canonicalize" src/discovery/`
returns no matches) and no manifest parser, so the corresponding
tests would have been testing the wrong code. The proposal items
were re-scoped at archive time to match what `src/discovery/`
actually does.
