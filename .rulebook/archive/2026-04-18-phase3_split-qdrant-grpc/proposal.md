# Proposal: phase3_split-qdrant-grpc

## Why

`src/grpc/qdrant_grpc.rs` is **2,109 lines** implementing three separate Qdrant gRPC service traits (collections, points, snapshots) in one file. The traits have no shared state beyond the top-level service struct; they just happen to live together because `tonic::codegen` dumped them that way in the original implementation.

See [docs/refactoring/oversized-files-audit.md](../../../docs/refactoring/oversized-files-audit.md) for context.

## What Changes

1. Create `src/grpc/qdrant_grpc/` with one file per service trait:
   - `collections.rs` — `impl CollectionsService for QdrantGrpcService`.
   - `points.rs` — `impl PointsService for QdrantGrpcService`.
   - `snapshots.rs` — `impl SnapshotsService for QdrantGrpcService`.
2. Keep `src/grpc/qdrant_grpc.rs` as a `mod.rs` that defines `QdrantGrpcService` + re-exports.
3. No behavior change — pure layout.

## Impact

- Affected specs: none.
- Affected code: `src/grpc/qdrant_grpc.rs`, new `src/grpc/qdrant_grpc/*.rs`.
- Breaking change: NO.
- User benefit: three reviewable service impls instead of one scrollable wall; adding a new method to one service no longer forces a merge-conflict domino across the others.
