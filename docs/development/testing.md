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
| `tests/core/wal_vector_store.rs::test_vector_store_wal_integration` | `phase4_triage-wal-recovery-bugs` | WAL integration fails |
| `tests/core/wal_crash_recovery.rs` (4 tests) | `phase4_triage-wal-recovery-bugs` | WAL recovery not replaying correctly |
| `tests/core/wal_comprehensive.rs` (4 tests) | `phase4_triage-wal-recovery-bugs` | Multi-op / update-sequence / multi-collection |
| `tests/core/storage.rs` (2 tests) | `phase4_triage-mmap-storage-bugs` | MMap insert+retrieve / update+delete failing |
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
