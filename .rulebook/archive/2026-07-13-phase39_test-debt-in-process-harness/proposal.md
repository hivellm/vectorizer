# Proposal: phase39_test-debt-in-process-harness

Source: docs/analysis/2026-07-11-improvement-analysis/ (§3)

## Why

The 2026-07-11 improvement analysis found the effective CI test
coverage far below what the tree suggests:

1. **152 `#[ignore]`d tests across 43 files** (the testing doc claims
   ~40). The bulk (~60) are REST/hub suites gated on a live server at
   127.0.0.1:15002 — they never run in CI.
2. **~30 REST handlers have zero non-ignored coverage** (batch
   update/delete, explain_search, copy_vectors, snapshots, discovery
   pipeline, slow queries, admin, …). Their only exercise is the
   ignored live suites → effectively 0 CI coverage. Error branches
   (dim mismatch, missing collection, quota) are asserted only in
   ignored suites.
3. **Five replication test files are orphaned** — `comprehensive.rs`,
   `integration_basic.rs`, `api.rs`, `qdrant_migration.rs`,
   `qdrant_api.rs` are not declared in `tests/replication/mod.rs` and
   never compile.
4. The gRPC "update fails in CI" ignore reason is identical across
   5+ files — almost certainly a real update bug mislabeled as
   flakiness.
5. All five SDK CI workflows run unit tests only; every
   integration/s2s test is env-skipped. No SDK-to-server integration
   runs anywhere.

An in-process harness is feasible: `tower::ServiceExt::oneshot` is
already used in three test files; what is missing is a shared harness
that builds the real `VectorizerServer` router with real app state.

## What Changes

- New shared in-process test harness (axum Router from real app state
  + `tower::ServiceExt::oneshot`) under `tests/common/`.
- Migrate the `*_real.rs` / `*_live.rs` REST and hub suites onto the
  harness so they run in CI without a live binary.
- Router-level tests for the ~30 uncovered handlers, including error
  branches.
- Wire the five orphaned replication test files into
  `tests/replication/mod.rs` (fixing compile bit-rot) or delete those
  that duplicate wired coverage — with justification per file.
- Triage the gRPC update failure as a bug: reproduce, root-cause, fix
  or file a dedicated task with findings.
- Add reason strings to all bare `#[ignore]` attributes.
- Regenerate `docs/development/testing.md` from actual counts; add a
  CI gate that fails when the ignore count grows.
- Add one gated SDK integration job that boots the server in docker
  and runs each SDK's integration suite.

## Impact

- Affected specs: `specs/test-harness/spec.md` (new, in this task)
- Affected code: `crates/vectorizer/tests/`,
  `crates/vectorizer-server/tests/`, `tests/replication/mod.rs`,
  `.github/workflows/sdk-*.yml`, `docs/development/testing.md`
- Breaking change: NO (test-only)
- User benefit: regressions caught in CI instead of production;
  restart/search regression class (see phase37) becomes testable
