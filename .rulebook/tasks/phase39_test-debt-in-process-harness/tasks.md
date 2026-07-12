## 1. In-process harness

- [ ] 1.1 Build a shared test harness under `tests/common/`: real `VectorizerServer` router + app state, driven via `tower::ServiceExt::oneshot`
- [ ] 1.2 Migrate `tests/api/rest/*_real.rs` suites onto the harness and remove their `#[ignore]` attributes
- [ ] 1.3 Migrate `hub_integration_live.rs` and GraphQL hub/encryption suites onto the harness where no external hub is required

## 2. Handler coverage

- [ ] 2.1 Router-level happy-path tests for the ~30 uncovered handlers listed in analysis §3.2
- [ ] 2.2 Error-branch tests: dimension mismatch, missing collection, empty IDs, quota exceeded, invalid filter

## 3. Orphaned + mislabeled tests

- [ ] 3.1 Wire `comprehensive.rs`, `integration_basic.rs`, `api.rs`, `qdrant_migration.rs`, `qdrant_api.rs` into `tests/replication/mod.rs`, fixing compile bit-rot; delete only files that duplicate wired coverage, with per-file justification
- [ ] 3.2 Root-cause the gRPC `test_update_vector` failure shared across 5+ files; fix the bug or document findings in a dedicated follow-up task created before this task archives
- [ ] 3.3 Add reason strings to every bare `#[ignore]` (`failover.rs`, `comprehensive.rs`, `gpu/hive_gpu.rs`, `product.rs`)

## 4. CI + SDK integration

- [ ] 4.1 Add a CI check failing the build when the repository `#[ignore]` count exceeds the recorded baseline
- [ ] 4.2 Add one gated SDK integration job: boot the server via docker, run each SDK's integration suite against it
- [ ] 4.3 Regenerate `docs/development/testing.md` from actual counts (152 ignores, orphaned files, live-server suites)

## 5. Tail (mandatory — enforced by rulebook v5.3.0)

- [ ] 5.1 Update or create documentation covering the implementation
- [ ] 5.2 Write tests covering the new behavior
- [ ] 5.3 Run tests and confirm they pass
