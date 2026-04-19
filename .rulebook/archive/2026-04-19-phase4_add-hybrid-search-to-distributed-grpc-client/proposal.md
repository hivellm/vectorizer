# Proposal: phase4_add-hybrid-search-to-distributed-grpc-client

## Why

`src/db/distributed_sharded_collection.rs:618` falls back to dense-only search on the gRPC client path because the distributed gRPC service does not yet expose a hybrid-search RPC. Local collections support hybrid (dense + sparse BM25 fusion); distributed ones don't — so query results differ depending on whether a collection happens to be sharded or local, which is a correctness surprise users have to learn about from docs.

## What Changes

1. Add a `HybridSearch` RPC to the `.proto` served by the cluster's gRPC service, mirroring the fields already on the local hybrid entry point (query text, weights, fusion method, k).
2. Implement the RPC on the server side by delegating to the existing local hybrid search.
3. Update the client used by `DistributedShardedCollection` to call the new RPC and merge per-shard results via the existing fusion utility.
4. Keep the dense-only fallback for gRPC servers that haven't upgraded yet (feature negotiation).

## Impact

- Affected specs: gRPC proto definitions.
- Affected code: `src/grpc/` (proto + server), `src/db/distributed_sharded_collection.rs`.
- Breaking change: NO — new RPC, old RPCs keep working.
- User benefit: hybrid search works identically on local and distributed collections.
