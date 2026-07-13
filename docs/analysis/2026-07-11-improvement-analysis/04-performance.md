# §4 — Performance

> Scope: `crates/vectorizer/src/{db,embedding,cache,batch}`,
> `crates/vectorizer-core/src/simd/`, `benches/`, plus the insert /
> search / BM25-vocab hot paths.

## 4.1 Findings

| # | Sev | Location | Description | Est. impact | Suggested fix |
|---|-----|----------|-------------|-------------|---------------|
| 1 | **HIGH** | `db/collection/data.rs:61-226` | `insert_batch` holds `self.index.write()` (HNSW) for the **entire** batch loop — payload indexing, sparse indexing, quantization, graph discovery all run under the write lock. Blocks all concurrent readers (`search` takes `index.read()` at `:449`) and serializes writers. | Insert throughput and search p99 collapse under mixed load | Hold the index write only around `index.add`; do payload/sparse/graph work outside the lock |
| 2 | **HIGH** | `db/collection/data.rs:75,87,141,153` | Per-vector clone churn: `vector.id.clone()` + `vector.data.clone()` each iteration, then `vector.clone()` (`:141`) and `data.clone()` again (`:153`) — 3-4 full copies of the f32 array (768-1024 dims) per inserted vector. | Large alloc/memcpy overhead on bulk upsert | Move `data` into the index; store `vector` by value once; `Arc<str>` for id |
| 3 | **HIGH** | `embedding/providers/bm25.rs` + `db/vector_store/autosave.rs:398-426,508-534` | **BM25 vocab not persisted by auto-save**: `save_collection_tokenizer` writes a stub (`"vocab_size":0`, empty vocab). The real `save_vocabulary_json` (`bm25.rs:143`) is only called from the file-watcher `indexer.rs:238`. `load_vocabulary_json`/`restore_vocabulary_data` have **zero callers in `src/`**. On reload the BM25 provider stays empty → query embed hits the hash fallback (`bm25.rs:385-425`), a different space from stored vectors → **no hits until re-index**. Reproduced manually during v3.4.0 Docker validation. | Confirmed correctness/relevance regression on every restart | Wire `save_vocabulary_json` into auto-save per collection; call `load_vocabulary_json` in `load_all_persisted_collections` |
| 4 | **MED** | `vectorizer-core/src/simd/backend.rs:156-194` | `quantize_f32_to_u8` / `dequantize_u8_to_f32` are **scalar-only** — no backend overrides them (overrides exist only for `manhattan` on avx2/neon and `int8_dot_product` on avx512vnni/sve2). Doc comment falsely claims "SIMD backends benefit." | Quantized insert/search leaves 4-8x on the table | AVX2/AVX-512/NEON (de)quantize kernels |
| 5 | **MED** | `simd/x86/*`, `simd/aarch64/*` | Coverage gaps: `int8_dot_product` SIMD only on AVX-512 VNNI + SVE2 (AVX2/SSE2/NEON/AVX-512F → scalar); `manhattan_distance` only AVX2+NEON; `normalize_in_place`, `add_assign`, `sub_assign`, `scale`, `horizontal_min_index` scalar everywhere. Core 4 (dot/euclidean/cosine/l2) fully covered. | INT8 distance slow on common AVX2 hardware | AVX2 int8 via `_mm256_maddubs_epi16`; manhattan on SSE2/AVX-512 |
| 6 | **MED** | CI (simd-matrix removed in `cf298f7d`) | Only `tests/simd/scalar_oracle.rs` on the default host backend remains. Non-default backends (AVX-512, NEON, SVE, WASM128) are **no longer correctness-verified in CI**; the perf regression bench guard is gone. A divergent intrinsic ships undetected. | Silent numeric drift / perf regression risk | Minimal restored matrix: forced `VECTORIZER_SIMD_BACKEND` oracle runs + cross-compiled aarch64 |
| 7 | **MED** | `db/upsert_queue.rs:134-138` | `try_admit` does `collection.to_string()` on **every** admission (even when the DashMap entry exists) to satisfy `entry()`. Heap alloc per request on the hot admission path. | Alloc per upsert request | `get()` fast-path first; `entry(to_string())` only on miss |
| 8 | LOW | `cache/query_cache.rs:67-85` | `QueryKey::from_vector` runs SHA-256 over every f32 of the query vector per lookup; `invalidate_collection` (`:228`) clones all matching keys. | Per-search hashing cost | xxh3 for cache keys |
| 9 | LOW | `server/core/bootstrap.rs` | Config re-read + full `serde_yaml` parse ~17× (`:237,308,319,431,466,845,861,927,989,1016,1040,1076,1268,1309,1341,1362,1438`). `build_default_provider` runs 3× sequentially (`:263,286,789`) — for `fastembed:` this loads the ONNX model **three times** serially. | Slower cold start (esp. fastembed) | Parse config once; build provider once, share via `Arc`; `tokio::join!` the three managers |
| 10 | LOW | `benches/` + `Cargo.toml:221-265` | Active benches: 6 SIMD micro + `filter_benchmark` + `multi_tenant_overhead`. `core_operations`, `search_bench`, `update_bench`, `query_cache_bench`, `cache`, `embeddings`, `quantization`, `storage` bench files exist but are **commented out**. No bench covers insert pipeline, end-to-end search, BM25 vocab build, or upsert admission. | No regression detection on real hot paths | Register the commented benches; add BM25 + admission benches |

## 4.2 Notes

- **Item 1 is the highest-leverage fix.** `get_collection_mut`
  (`collections.rs:731`) also double-looks-up (calls `get_collection`
  then `get_mut`) and returns a DashMap shard write guard, so writers
  to the same shard serialize even before reaching the index lock.
- **Backpressure layer** (`db/backpressure.rs`, `db/upsert_queue.rs`)
  is otherwise clean: lock-free atomics, CAS admission
  (`upsert_queue.rs:155-169`), no O(n) admission scan.
  `snapshot_depths` (`:99`) is O(n) but scrape-time only.
- **Two BM25 implementations coexist**: `embedding/bm25.rs`
  (`BM25Provider`, `Arc<RwLock<…>>` per field) vs
  `embedding/providers/bm25.rs` (`Bm25Embedding`, used in
  production). `BM25Provider::calculate_bm25_score` (`bm25.rs:148`)
  re-acquires 4 RwLock reads **per term per document** — avoid that
  path on hot search, or delete the dead impl.
- Search result construction (`data.rs:481-487`) clones `id` + full
  `vector.data` per hit; for large `k` return `Arc`s.

(→ phase38, with the BM25 persistence item shared with phase37)
