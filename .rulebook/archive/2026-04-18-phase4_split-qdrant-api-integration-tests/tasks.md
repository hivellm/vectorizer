## 1. Layout

- [x] 1.1 Create `tests/integration/qdrant_api/` with one file per capability — delivered 8 files: `quantization.rs`, `cluster.rs`, `points.rs`, `distance.rs`, `status.rs`, `snapshots.rs`, `sharding.rs`, `search.rs`. Each groups the former model + API test modules for that feature together.
- [x] 1.2 Move the shared setup/teardown helpers into `mod.rs` — the file had no live setup/teardown helpers, only three common `pub struct` types (`QdrantResponse<T>`, `QdrantPoint`, `ScoredPoint`). Those sit in `mod.rs` with a `pub mod` declaration for each capability.

## 2. Verification

- [x] 2.1 `cargo test --test all_tests --all-features` passes — 780/780 total, 91 scoped to `qdrant_api` (unchanged from the pre-split count).
- [x] 2.2 No sub-file exceeds 400 lines — `search.rs` is 400 lines (exactly at the threshold; contains the largest test group). Everything else is 58-309 lines.

## 3. Tail (mandatory)

- [x] 3.1 Document the new layout in `tests/integration/qdrant_api/mod.rs` — explicit header explains the capability-grouping rationale.
- [x] 3.2 Tests are the deliverable; no new tests required — layout-only refactor.
- [x] 3.3 Full integration suite green.

## Mandatory tail (required by rulebook v5.3.0)

- [x] Update or create documentation covering the implementation
- [x] Write tests covering the new behavior
- [x] Run tests and confirm they pass
