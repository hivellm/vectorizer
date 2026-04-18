## 1. Inventory

- [ ] 1.1 Re-run `grep -rEc "\b(T-O-D-O|F-I-X-M-E|H-A-C-K|X-X-X)\b" src/` (with the hyphens removed in the actual invocation) to confirm the 19-marker count hasn't changed since task creation; dump the fresh list into `design.md`.
- [ ] 1.2 Classify each row as one of: FIX_INLINE, TRACK_TO_TASK, DELETE, FALSE_POSITIVE.

## 2. Resolution — fix-inline sites

- [ ] 2.1 `src/db/raft.rs:131,204,208` — extract the actual collection name from the `ClusterCommand::Insert { collection_name, .. }` payload instead of hardcoding `"default"`. Three identical sites.
- [ ] 2.2 `src/db/quantized_collection.rs:163,172` — add `document_id: Option<String>` to `crate::models::Vector` (or confirm it already exists under a different name) and wire the two sites to use it.
- [ ] 2.3 `src/api/graph.rs:668` — implement the edge_id→collection lookup cache if feasible in a single-file change; otherwise convert to TRACK_TO_TASK.
- [ ] 2.4 `src/batch/operations.rs:234` — implement the missing `BatchProcessor` method OR convert to TRACK_TO_TASK with a new rulebook task named after the method.

## 3. Resolution — track-to-task sites

- [ ] 3.1 `src/db/gpu_detection.rs:133` — create `phase4_query-metal-device-info-from-hive-gpu`; convert the marker to `// TASK(phase4_query-metal-device-info-from-hive-gpu): ...`.
- [ ] 3.2 `src/db/distributed_sharded_collection.rs:618` — create `phase4_add-hybrid-search-to-distributed-grpc-client`; convert the marker.
- [ ] 3.3 `src/discovery/tests.rs:8` — create `phase4_refactor-discovery-integration-tests`; convert the marker.
- [ ] 3.4 `src/intelligent_search/examples.rs:311,325,344` — create `phase4_refactor-intelligent-search-examples`; convert all three markers to the same TASK form.
- [ ] 3.5 `src/quantization/mod.rs:15` — convert to `// TASK(phase4_add-quantization-submodules): ...` OR delete if the submodules are not planned.

## 4. Resolution — delete / false-positive sites

- [ ] 4.1 `src/file_operations/README.md:198` (heading uses a marker word) — rewrite the heading as `### Roadmap (Priority 2 & 3)` so it stops matching the grep.
- [ ] 4.2 `src/file_operations/README.md:234,294` — these are code examples showing the patterns to replace. Surround the snippet with ``<!-- grep-ignore -->`` comment markers, or rewrite the example to not use the forbidden words.
- [ ] 4.3 `src/file_operations/operations.rs:421-422` — FALSE POSITIVE: the string literals are the actual markers the feature detects in user files. Add a `// grep-ignore(tier1-markers): detection feature, literal strings required` comment + configure the CI gate to ignore lines matching that inline-marker.

## 5. CI enforcement

- [ ] 5.1 Add a grep gate to `.github/workflows/rust-lint.yml` with the exact forbidden pattern and the allowed forms. Pseudocode:
  ```bash
  grep -rnE '\b(<forbidden-four>)\b' src/ \
    | grep -vE 'TASK\(phase[0-9]+_[a-z0-9-]+\)' \
    | grep -vE 'grep-ignore\(tier1-markers\)' \
    && { echo "forbidden marker found"; exit 1; } || true
  ```
- [ ] 5.2 Run the gate locally on the current tree; confirm zero hits.

## 6. Tail (mandatory — enforced by rulebook v5.3.0)

- [ ] 6.1 Update `AGENTS.md` Tier-1 section + `/.rulebook/specs/TIER1_PROHIBITIONS.md` documenting the `TASK(phase\d+_...)` allow-list form.
- [ ] 6.2 The CI gate IS the regression test. Add a one-line integration test that shells out to the gate script to confirm the guard runs successfully.
- [ ] 6.3 Run `cargo test --all-features` and confirm no behavior regressions.

## Mandatory tail (required by rulebook v5.3.0)

- [ ] Update or create documentation covering the implementation
- [ ] Write tests covering the new behavior
- [ ] Run tests and confirm they pass
