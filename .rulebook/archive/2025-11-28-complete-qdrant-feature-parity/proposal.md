# Complete Qdrant Feature Parity - Proposal

## Why

Vectorizer currently provides good Qdrant REST API compatibility for basic operations (collections, points CRUD, search, recommend), but several advanced Qdrant features are missing. After analyzing the Qdrant codebase, we identified missing endpoints, gRPC protocol support, and advanced search features. Completing feature parity will enable seamless migration from Qdrant and ensure Vectorizer can serve as a complete drop-in replacement.

## What Changes

This task completes Qdrant feature parity by implementing:

1. **Qdrant gRPC Protocol Support**: Complete Qdrant gRPC API compatibility (Collections, Points, Snapshots, Query services)
2. **Snapshots via Qdrant API**: Expose `/collections/{name}/snapshots` endpoints (GET, POST, DELETE)
3. **Sharding via Qdrant API**: Expose `/collections/{name}/shards` endpoints (PUT, POST for create/delete shard keys)
4. **Cluster Management via Qdrant API**: Expose `/cluster` endpoint for cluster status
5. **Search Groups**: Implement `/collections/{name}/points/search/groups` endpoint
6. **Search Matrix**: Implement `/collections/{name}/points/search/matrix/pairs` and `/matrix/offsets` endpoints
7. **Query API**: Implement `/collections/{name}/points/query`, `/query/batch`, and `/query/groups` endpoints
8. **Named Vectors (`using` parameter)**: Support `using` parameter in search/query requests to select specific named vectors
9. **Prefetch Operations**: Support prefetch in query/search requests for optimization
10. **PQ/Binary Quantization via Qdrant API**: Expose Product Quantization and Binary Quantization configuration through Qdrant API (implementations exist but not exposed)

**Note**: Many features already exist in Vectorizer (sharding, snapshots, PQ quantization) but need to be exposed via Qdrant API endpoints.

## Impact

- **Affected specs**: 
  - `specs/QDRANT_MIGRATION.md` - Update with new features
  - `specs/api-rest/spec.md` - Add Qdrant endpoints
  - `specs/grpc/spec.md` - Add Qdrant gRPC protocol specs
  - `specs/cluster/spec.md` - Add cluster management specs
- **Affected code**: 
  - `src/grpc/` - Add Qdrant gRPC protocol implementation (separate from Vectorizer gRPC)
  - `src/server/qdrant_handlers.rs` - Add snapshots, shards, cluster endpoints
  - `src/server/qdrant_search_handlers.rs` - Add search/groups, search/matrix endpoints
  - `src/server/qdrant_query_handlers.rs` - NEW: Add query API endpoints
  - `src/server/qdrant_vector_handlers.rs` - Add named vectors and prefetch support
  - `src/db/collection.rs` - Add named vectors support
  - `src/db/vector_store.rs` - Add prefetch operations
  - `src/cluster/` - Expose cluster management via Qdrant API
- **Breaking change**: NO (additive only)
- **User benefit**: 
  - Complete Qdrant compatibility for seamless migration
  - Access to existing Vectorizer features via Qdrant API
  - Production-ready feature parity with Qdrant
