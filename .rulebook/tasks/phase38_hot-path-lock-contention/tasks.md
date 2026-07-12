## 1. Insert-path lock scope

- [ ] 1.1 Restructure `insert_batch` (`data.rs:61-226`) so the HNSW `index.write()` wraps only `index.add`; payload/sparse/graph/quantization work runs outside the lock
- [ ] 1.2 Remove redundant per-vector clones (`data.rs:75,87,141,153`) — move `data` into the index, single ownership transfer of `vector`
- [ ] 1.3 Add a mixed-load benchmark (concurrent insert_batch + search) proving search p99 no longer degrades with batch size

## 2. Quantization wiring

- [ ] 2.1 Add PQ and Binary arms to `hnsw_integration.rs:74-83`
- [ ] 2.2 Replace the silent 8-bit fallback (`hnsw_integration.rs:122-125`) with an explicit error for unsupported types
- [ ] 2.3 Round-trip tests: PQ- and Binary-quantized collections insert + search with acceptable recall

## 3. SIMD kernels

- [ ] 3.1 Implement AVX2 and NEON `quantize_f32_to_u8`/`dequantize_u8_to_f32` backends; correct the false doc comment at `backend.rs:156-194`
- [ ] 3.2 Implement AVX2 `int8_dot_product` via `_mm256_maddubs_epi16`
- [ ] 3.3 Restore a minimal SIMD correctness matrix in CI: scalar-oracle runs with forced `VECTORIZER_SIMD_BACKEND` values

## 4. Hot-path allocations

- [ ] 4.1 `try_admit` (`upsert_queue.rs:134-138`): `get()` fast path before `entry(collection.to_string())`
- [ ] 4.2 Switch `QueryKey::from_vector` (`query_cache.rs:67-85`) from SHA-256 to xxh3

## 5. Benchmark coverage

- [ ] 5.1 Re-register the commented-out benches in `Cargo.toml:221-265` (`core_operations`, `search_bench`, `update_bench`, `query_cache_bench`, `cache`, `embeddings`, `quantization`, `storage`) and fix any bit-rot
- [ ] 5.2 Add insert-pipeline and BM25 vocab-build benches

## 6. Tail (mandatory — enforced by rulebook v5.3.0)

- [ ] 6.1 Update or create documentation covering the implementation
- [ ] 6.2 Write tests covering the new behavior
- [ ] 6.3 Run tests and confirm they pass
