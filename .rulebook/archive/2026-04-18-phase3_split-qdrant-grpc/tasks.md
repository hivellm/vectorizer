## 1. Layout

- [x] 1.1 Create `src/grpc/qdrant_grpc/` and move the existing file to `mod.rs`.
- [x] 1.2 Extract `impl CollectionsService ...` into `collections.rs`.
- [x] 1.3 Extract `impl PointsService ...` into `points.rs`.
- [x] 1.4 Extract `impl SnapshotsService ...` into `snapshots.rs`.

## 2. Verification

- [x] 2.1 `cargo check --all-features` clean.
- [x] 2.2 `tests/grpc/` suite passes unchanged.

## 3. Tail (mandatory)

- [x] 3.1 Update the module-level doc comment.
- [x] 3.2 No new tests required — layout-only.
- [x] 3.3 `cargo test --all-features` pass.

## Mandatory tail (required by rulebook v5.3.0)

- [x] Update or create documentation covering the implementation
- [x] Write tests covering the new behavior
- [x] Run tests and confirm they pass
