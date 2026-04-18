## 1. Audit

- [ ] 1.1 Grep all `#[ignore]` annotations in `tests/` and `src/`; classify each in `design.md`

## 2. Fix by test

- [ ] 2.1 Replication tests: spin up secondary process via testcontainers or a harness; re-enable
- [ ] 2.2 gRPC s2s tests: enable feature `s2s-tests`; ensure CI runs this feature on a dedicated matrix row
- [ ] 2.3 WAL tests: fix race conditions or provide deterministic harness
- [ ] 2.4 GPU tests: add CI matrix row with `hive-gpu` feature on a GPU-capable runner (or mark nightly-only, not `ignore`d)
- [ ] 2.5 Cluster performance tests: move to a nightly CI workflow with perf budget assertions
- [ ] 2.6 Sparse vector / graph / storage integration tests: fix or delete with decision record

## 3. CI integration

- [ ] 3.1 Add new CI matrix rows for feature-gated tests
- [ ] 3.2 Add nightly workflow `.github/workflows/nightly.yml` that runs slow tests
- [ ] 3.3 Remove `.bak` test files (`tests/integration/sharding_validation.rs.bak`) — see `phase5_delete-bak-files`

## 4. Tail (mandatory — enforced by rulebook v5.3.0)

- [ ] 4.1 Document the CI matrix + nightly strategy in `docs/development/testing.md`
- [ ] 4.2 Verify every previously-ignored test now runs in some CI job
- [ ] 4.3 Run the updated CI workflows on a branch and confirm green

## Mandatory tail (required by rulebook v5.3.0)

- [ ] Update or create documentation covering the implementation
- [ ] Write tests covering the new behavior
- [ ] Run tests and confirm they pass
