## 1. Proto surface

- [ ] 1.1 Add `HybridSearchRequest` / `HybridSearchResponse` to the cluster proto file.
- [ ] 1.2 Regenerate the Rust bindings.

## 2. Server-side handler

- [ ] 2.1 Implement the RPC in the cluster gRPC server by delegating to the existing local hybrid-search entry point.
- [ ] 2.2 Return a clear `Unimplemented` error for clients hitting a server that predates this RPC (so they can fall back).

## 3. Client-side integration

- [ ] 3.1 Call the new RPC from `DistributedShardedCollection::hybrid_search` at `src/db/distributed_sharded_collection.rs:618`.
- [ ] 3.2 Merge per-shard results via the existing fusion helper.
- [ ] 3.3 Fall back to dense-only when the server returns `Unimplemented`.

## 4. Tests

- [ ] 4.1 Integration test against a local cluster of two shards asserting hybrid results match the single-node baseline.
- [ ] 4.2 Compatibility test: point the client at a server without the RPC; assert dense fallback fires.

## 5. Tail (mandatory)

- [ ] 5.1 Update the clustering doc with the new RPC.
- [ ] 5.2 Tests above cover the new behavior.
- [ ] 5.3 Run `cargo test --all-features` and confirm pass.
