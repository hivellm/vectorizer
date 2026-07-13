# Test Harness Spec — In-Process Coverage

## ADDED Requirements

### Requirement: In-process REST test harness

The test suite MUST provide a shared harness that constructs the real
`VectorizerServer` router with real application state and serves
requests in-process (no TCP listener, no live binary).

#### Scenario: Harness serves a real route

Given the in-process harness with a created collection
When a `POST /insert` request is dispatched via `oneshot`
Then the response MUST be produced by the production router and
return the same status and body shape as the live server

### Requirement: REST suites run in CI

Formerly live-server REST test suites MUST execute in default
`cargo test` runs without a running server.

#### Scenario: CI run without server

Given no vectorizer process is listening on any port
When `cargo test` runs the migrated REST suites
Then the suites MUST execute (not be filtered as ignored) and pass

### Requirement: Handler error-branch coverage

Every public REST handler MUST have at least one non-ignored test,
and error responses (dimension mismatch, missing collection, invalid
input) MUST be asserted on status code and `error_type`.

#### Scenario: Missing collection error asserted

Given the harness with no collection named "ghost"
When a search against "ghost" is dispatched
Then the test MUST assert the not-found status and error body shape

### Requirement: No orphaned test files

Every `.rs` file under `tests/` MUST be reachable from a `mod.rs`
declaration or a `[[test]]` target — files that never compile are
forbidden.

#### Scenario: Replication tests compile

Given the five formerly orphaned replication test files
When `cargo test --no-run` executes
Then each retained file MUST compile as part of the test build

### Requirement: Ignore attributes carry reasons

Every `#[ignore]` in the repository MUST use the
`#[ignore = "reason"]` form.

#### Scenario: Bare ignore rejected

Given a test annotated with bare `#[ignore]`
When the ignore-audit CI check runs
Then the check MUST fail and name the offending test

### Requirement: Ignore-count regression gate

CI MUST fail when the total `#[ignore]` count exceeds the recorded
baseline.

#### Scenario: New ignored test without baseline update

Given a PR adding one `#[ignore]`d test without updating the baseline
When CI runs
Then the ignore-count gate MUST fail

### Requirement: SDK integration in CI

At least one CI job MUST run each SDK's integration test suite
against a running server instance.

#### Scenario: SDK round-trip in CI

Given the gated SDK integration job with the server booted in docker
When the TypeScript SDK integration suite runs
Then create-collection, insert, and search calls MUST succeed against
the live server
