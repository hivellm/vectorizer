## 1. Layout

- [ ] 1.1 Create `src/grpc/qdrant_grpc/` and move the existing file to `mod.rs`.
- [ ] 1.2 Extract `impl CollectionsService ...` into `collections.rs`.
- [ ] 1.3 Extract `impl PointsService ...` into `points.rs`.
- [ ] 1.4 Extract `impl SnapshotsService ...` into `snapshots.rs`.

## 2. Verification

- [ ] 2.1 `cargo check --all-features` clean.
- [ ] 2.2 `tests/grpc/` suite passes unchanged.

## 3. Tail (mandatory)

- [ ] 3.1 Update the module-level doc comment.
- [ ] 3.2 No new tests required — layout-only.
- [ ] 3.3 `cargo test --all-features` pass.
