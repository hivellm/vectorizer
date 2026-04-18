# Proposal: phase0_fix-all-todos

## Why

`AGENTS.md` Tier-1 rule #1 explicitly forbids deferral markers (`T O D O`, `F I X M E`, `H A C K`, `X X X`) in source code: "Implement completely or explain concretely why you can't." The earlier audit found 1,535 such markers across 167 files. Most were already cleaned up indirectly during the v3.0.0 sprint (as the files carrying them were refactored), but **19 markers still survive** across 11 files as of 2026-04-18 and need a direct resolution pass.

Phase 0 because: per project policy these are a Tier-1 violation. They block a clean `-D warnings` story, confuse reviewers who can't tell which markers are live vs. archaeological, and hide real bugs inside a wall of "will be done later" noise. Every Phase 1+ task assumes a clean baseline — this task establishes it.

## Current inventory (grep on current tree)

```
src/api/graph.rs:668                           — optimization note
src/batch/operations.rs:234                    — unimplemented method
src/db/distributed_sharded_collection.rs:618   — gRPC hybrid search gap
src/db/gpu_detection.rs:133                    — upstream hive-gpu API wait
src/db/quantized_collection.rs:163,172         — Vector::document_id field gap (2)
src/db/raft.rs:131,204,208                     — extract collection name (3)
src/discovery/tests.rs:8                       — test needs refactor after API change
src/file_operations/README.md:198,234,294      — doc + example code (3)
src/file_operations/operations.rs:421-422      — FALSE POSITIVE: code that searches FOR the marker words
src/intelligent_search/examples.rs:311,325,344 — tests need refactor (3)
src/quantization/mod.rs:15                     — future-modules placeholder
```

## What Changes

For each of the 19 markers:

1. **Fix inline** if the underlying item is small and has enough context to implement now. Examples: the `raft.rs` "extract from operation" sites (just wire the real name), the `quantized_collection.rs` document-id gap (add the field and use it).

2. **Convert to `// TASK(phaseN_<task-id>):`** with a real rulebook follow-up task if the fix is substantial. Current format going forward: `// TASK(phase4_test-refactor-after-mcp-api-change): ...` — each marker points at a concrete, tracked deliverable.

3. **Delete** if the item is archaeological (the feature never shipped, the doc section is outdated, the test note references a scenario that no longer exists).

4. **Mark as FALSE POSITIVE** where the marker word appears as a literal string in code that searches for these tokens (e.g. `file_operations/operations.rs` has a feature that detects TODO-like markers in user files). Add a leading `#[allow(clippy::...)]` or a lint-escape comment so the CI grep gate doesn't flag them.

5. **CI grep gate** in `rust-lint.yml` that fails the build on any unqualified marker. Allowed forms only:
   - `// TASK(phase\d+_[a-z0-9-]+):` — tracked in rulebook
   - string literals in designated files (operations.rs, test fixtures) — escape via `#[allow]` or comment marker

## Impact

- Affected specs: `/.rulebook/specs/TIER1_PROHIBITIONS.md`
- Affected code: 11 files with surviving markers + `.github/workflows/rust-lint.yml` (new gate)
- Breaking change: NO
- User benefit: removes 19 unresolved "placeholders" that would otherwise ship in v3.0.0; establishes the `TASK(...)` convention for future backlog markers so every reader instantly knows where the fix is tracked.
