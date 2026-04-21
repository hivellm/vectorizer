## 1. Proto surface

- [x] 1.1 Add `HybridSearchRequest` / `HybridSearchResponse` to the cluster proto file.
- [x] 1.2 Regenerate the Rust bindings.

## 2. Server-side handler

- [x] 2.1 Implement the RPC in the cluster gRPC server by delegating to the existing local hybrid-search entry point.
- [x] 2.2 Return a clear `Unimplemented` error for clients hitting a server that predates this RPC (so they can fall back).

## 3. Client-side integration

- [x] 3.1 Call the new RPC from `DistributedShardedCollection::hybrid_search` at `src/db/distributed_sharded_collection.rs:618`.
- [x] 3.2 Merge per-shard results via the existing fusion helper.
- [x] 3.3 Fall back to dense-only when the server returns `Unimplemented`.

## 4. Tests

- [x] 4.1 Integration test against a local cluster of two shards asserting hybrid results match the single-node baseline.
- [x] 4.2 Compatibility test: point the client at a server without the RPC; assert dense fallback fires.

## 5. Tail (mandatory)

- [x] 5.1 Update or create documentation covering the implementation: `docs/users/api/GRPC.md` now lists the new `RemoteHybridSearch` RPC, its message shapes, and the `Unimplemented`-fallback semantics. `CHANGELOG.md` Unreleased/Added entry summarizes the new wire surface and the `VectorizerError::Unimplemented` variant.
- [x] 5.2 Write tests covering the new behavior: `tests/integration/cluster_hybrid_search.rs` covers (a) the success path against an in-process `ClusterGrpcService` and (b) the compatibility path that exercises the dense-only fallback when the server lacks `ClusterServiceServer`. Existing `tests/integration/distributed_search.rs` and `tests/integration/hybrid_search.rs` continue to pass.
- [x] 5.3 Run `cargo check --tests --all-features` (clean), `cargo clippy --lib --tests -- -D warnings` (clean), `cargo fmt` (clean), and the full integration::cluster_hybrid_search/distributed_search/hybrid_search test groups (17/17 pass, 1 pre-existing ignored).
