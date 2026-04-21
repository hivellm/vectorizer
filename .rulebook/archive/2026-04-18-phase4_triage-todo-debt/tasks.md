## 1. Extraction

- [x] 1.1 Marker extraction script — the live enforcement script at [`scripts/check-no-tier1-markers.sh`](../../../scripts/check-no-tier1-markers.sh) (landed under `phase0_fix-all-todos`) greps `src/` for the four forbidden markers and emits the exact same signal this task asked for. Current output on `release/v3.0.0`: `Tier-1 marker gate: clean.`
- [x] 1.2 Analysis CSV — obviated. The original 1,535-marker population was cleaned up inline during the v3.0.0 sprint, leaving 19 survivors. `phase0_fix-all-todos` triaged those 19 with a per-row table in [design.md](../../archive/2026-04-18-phase0_fix-all-todos/) (archived).

## 2. Triage

- [x] 2.1 Category assignment — done across phase0 and this session. The 19 surviving markers were classified as: 11 TRACK_TO_TASK (converted to `// TASK(phase4_<slug>):`), 1 DELETE (archaeological), 3 REWRITE (documentation headings), 2 FALSE_POSITIVE (detection-feature literals).
- [x] 2.2 Bug-row rulebook tasks — created during phase0: `phase4_add-collection-name-to-raft-operations`, `phase4_add-document-id-to-vector`, `phase4_add-edge-id-collection-mapping-cache`, `phase4_implement-batch-processor-active-operations`, `phase4_query-metal-device-info-from-hive-gpu`, `phase4_add-hybrid-search-to-distributed-grpc-client`, `phase4_refactor-tests-examples-after-mcp-api-change`.
- [x] 2.3 Feature-row rulebook tasks — same seven tasks above cover the feature-gap markers (each `TASK(phase4_<slug>)` points at its owning rulebook task).

## 3. Cleanup

- [x] 3.1 Noise / obsolete markers deleted — `phase0_fix-all-todos` §4 deleted the archaeological one-line placeholder in `src/quantization/mod.rs` and rewrote three marker-bearing README headings + example snippets in `src/file_operations/README.md` to neutral phrasing.
- [x] 3.2 Batched cleanup — the 19-marker set was small enough to close in one commit rather than 2 PRs of ≤10; see archive commit `1b662c07`.
- [x] 3.3 Conforming form — every surviving in-code marker now uses `// TASK(phaseN_<slug>):` pointing at a tracked rulebook task. The literal-string occurrences inside `src/file_operations/operations.rs` carry the `// grep-ignore(tier1-markers)` sentinel documented in the CI gate.

## 4. Enforcement

- [x] 4.1 CI grep gate — [`scripts/check-no-tier1-markers.sh`](../../../scripts/check-no-tier1-markers.sh) enforces the rule; wired as a step in [`.github/workflows/rust-lint.yml`](../../../.github/workflows/rust-lint.yml) alongside the parking_lot / models-layering gates.
- [x] 4.2 Zero violations locally — verified in every commit this session (output: `Tier-1 marker gate: clean.`).

## 5. Tail (mandatory — enforced by rulebook v5.3.0)

- [x] 5.1 Specs updated — `AGENTS.md` Tier-1 #1 and `/.rulebook/specs/TIER1_PROHIBITIONS.md` both document the `TASK(phaseN_<slug>)` allow-list form and the `grep-ignore(tier1-markers)` sentinel. Landed under `phase0_fix-all-todos`.
- [x] 5.2 Regression test — [`tests/infrastructure/tier1_marker_gate.rs`](../../../tests/infrastructure/tier1_marker_gate.rs) walks the source tree with the same regex the shell gate uses; passes on every platform (no bash dependency).
- [x] 5.3 Full test suite — 1120/1120 lib, 780/780 integration, both CI gate scripts clean as of commit `5498818d`.

## Mandatory tail (required by rulebook v5.3.0)

- [x] Update or create documentation covering the implementation
- [x] Write tests covering the new behavior
- [x] Run tests and confirm they pass
