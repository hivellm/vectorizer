# Proposal: Native Engine Optimizations

## Why
To compete with industry-standard vector databases like Qdrant and Faiss, the native engine requires significant performance and scalability improvements. Currently, the engine lacks SIMD acceleration for distance calculations, relies entirely on in-memory storage (limiting dataset size), and has unintegrated Product Quantization support.

## What Changes

### Phase 1: Core Performance Optimizations (Immediate)
1.  **SIMD Acceleration**: Rewrite `vector_utils` to use SIMD intrinsics (AVX2/NEON) for dot product, cosine similarity, and Euclidean distance.
2.  **Memory-Mapped Storage**: Implement a disk-backed storage mechanism using `mmap` to allow collections to exceed available RAM.
3.  **Product Quantization (PQ) Integration**: Integrate the existing PQ implementation into the main `Collection` pipeline for efficient storage and search.

### Phase 2: High-Priority Features (Next)
4.  **Write-Ahead Log (WAL)**: Implement crash recovery and data durability (competitors: Qdrant ✓, Milvus ✓)
5.  **Advanced Payload Filtering**: Range queries, geo-filtering, nested field support (Qdrant excels at this)
6.  **gRPC API**: Add gRPC support for 2-5x faster communication vs REST
7.  **Async Indexing**: Non-blocking background HNSW rebuilding

### Phase 3: Scalability Features (Future)
8.  **Distributed Clustering/Sharding**: Horizontal scaling for billions of vectors
9.  **Raft Consensus**: Multi-master replication with leader election
10. **Multi-Tenancy**: Tenant isolation and quotas

## Impact
- Affected specs: `db`, `quantization`, `storage`, `api`
- Affected code: Multiple modules across the codebase
- Breaking change: NO (Internal optimizations, backward-compatible config additions)
- User benefits:
    - **Phase 1**: 10x-100x faster distance calculations, datasets larger than RAM, 4x-64x compression
    - **Phase 2**: Crash recovery, advanced filtering, 2-5x faster API, non-blocking inserts
    - **Phase 3**: Horizontal scaling to billions of vectors, high availability, enterprise-grade multi-tenancy

## Priority Justification
These improvements are based on feature gaps compared to industry leaders (Qdrant, Milvus, pgvector). Phase 1 addresses performance bottlenecks, Phase 2 adds critical enterprise features, and Phase 3 enables massive scale.
