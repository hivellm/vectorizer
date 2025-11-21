## 1. SIMD Optimization Phase
- [x] 1.1 Add `simba` or `packed_simd` dependency (or use `std::arch`)
- [x] 1.2 Implement SIMD-accelerated `dot_product`
- [x] 1.3 Implement SIMD-accelerated `euclidean_distance`
- [x] 1.4 Implement SIMD-accelerated `cosine_similarity`
- [x] 1.5 Benchmark SIMD vs Scalar implementations

## 2. Storage Optimization Phase (MMap)
- [x] 2.1 Add `memmap2` dependency
- [x] 2.2 Create `MmapVectorStorage` struct in `src/storage`
- [x] 2.3 Implement `VectorCollection` trait for MMap storage
- [x] 2.4 Update `CollectionConfig` to allow selecting `storage_type: "mmap"`

## 3. Quantization Integration Phase (PQ)
- [x] 3.1 Update `CollectionConfig` to support PQ parameters
- [x] 3.2 Integrate `ProductQuantization` training into `Collection::insert_batch`
- [x] 3.3 Implement PQ-based search (asymmetric distance) in `Collection::search`
- [x] 3.4 Verify recall/precision trade-offs

## 4. Verification Phase
- [x] 4.1 Run benchmarks for all distance metrics
- [x] 4.2 Verify MMap storage persistence and recovery
- [x] 4.3 Verify PQ compression and search accuracy

---

## Phase 2: High-Priority Features

## 5. WAL Implementation Phase
- [x] 5.1 Design WAL file format and rotation policy
- [x] 5.2 Implement `WalWriter` for operation logging
- [x] 5.3 Implement `WalReader` for crash recovery
- [x] 5.4 Integrate WAL into `Collection::insert/update/delete`
- [x] 5.5 Test crash recovery scenarios

## 6. Advanced Filtering Phase
- [x] 6.1 Extend `PayloadIndex` for range queries (✅ Already implemented with `get_ids_in_range` and `get_ids_in_float_range`)
- [x] 6.2 Implement geo-filtering (distance from point) (✅ Already implemented with `get_ids_in_geo_bounding_box` and `get_ids_in_geo_radius`)
- [x] 6.3 Add nested field indexing support (✅ Implemented - supports dot notation like "user.age")
- [x] 6.4 Benchmark filter performance (✅ Created `benchmark/filter/filter_benchmark.rs`)

## 7. gRPC API Phase
- [x] 7.1 Define Protobuf schemas for vector operations ✅
- [x] 7.2 Implement gRPC server using `tonic` ✅
- [x] 7.3 Add streaming support for bulk operations ✅ (implemented in InsertVectors)
- [x] 7.4 Benchmark gRPC vs REST performance ✅

**Status**: Implemented. gRPC API server is fully functional with:
- Complete Protobuf schemas for all vector operations
- Full VectorizerService implementation using tonic 0.12
- Streaming support for bulk insert operations
- Integrated into main server (runs on port+1)
- Type conversions between Protobuf and internal types
- Support for collections, vectors, search, and hybrid search operations

## 8. Async Indexing Phase
- [x] 8.1 Refactor `OptimizedHnswIndex` for background building (✅ Created `AsyncIndexManager` with background build support)
- [x] 8.2 Implement double-buffering for index swaps (✅ Implemented primary/secondary index swap mechanism)
- [x] 8.3 Add index build progress tracking (✅ Progress tracking with percentage, ETA, and status updates)
- [x] 8.4 Verify search quality during async rebuild (✅ Implemented `verify_search_quality` method with overlap ratio and score difference metrics, added comprehensive tests)

---

## Phase 3: Scalability Features (Future)

## 9. Distributed Sharding Phase
- [ ] 9.1 Design consistent hash sharding strategy
- [ ] 9.2 Implement shard routing logic
- [ ] 9.3 Add shard rebalancing mechanism
- [ ] 9.4 Test multi-shard queries

## 10. Raft Consensus Phase
- [ ] 10.1 Integrate Raft library (e.g., `tikv/raft-rs`)
- [ ] 10.2 Implement state machine for vector operations
- [ ] 10.3 Add leader election and failover
- [ ] 10.4 Test partition tolerance

## 11. Multi-Tenancy Phase
- [ ] 11.1 Add tenant ID to collection metadata
- [ ] 11.2 Implement resource quotas (memory, QPS)
- [ ] 11.3 Add tenant-level access control
- [ ] 11.4 Test tenant isolation
