## 1. Insert-path lock scope

- [x] 1.1 `insert_batch` takes `index.read()` + dedicated writer mutex (`Collection::insert_lock`) — better than the planned narrow-write: `OptimizedHnswIndex` is internally synchronized, so the outer write lock was pure reader-blocking; writers stay serialized for is-new/order/count atomicity, searches contend only on the index's internal locks
- [x] 1.2 Per-vector copies cut to one `data` clone: normalize in place; graph discovery moved before the storage insert so `vector` MOVES into storage (quantized and full-precision paths)
- [x] 1.3 `tests/insert_search_concurrency.rs`: searches complete while a 30k-vector batch runs (6k in debug); asserts interleaving, reports slowest search on failure

## 2. Quantization wiring

- [x] 2.1 `QuantizationMethod` implemented for `ProductQuantization` (batch quantize/dequantize + codebook serialize/deserialize); Scalar/Product/Binary arms in construction AND first-batch fit; cache stride derived from payload length (correct for every method)
- [x] 2.2 `QuantizationType::None` + the old silent 8-bit fit fallback are explicit `InvalidParameters` errors (test pins it). Bonus fix: search paths fed raw vectors to `crate::simd::cosine_similarity` (clamped dot, ASSUMES unit inputs) → every pair scored 1.0 on non-normalized data; now true cosine dot/(|q||v|)
- [x] 2.3 Round-trip tests: PQ (search + storage save/load of codebooks) and Binary (search); self-query returns itself first

## 3. SIMD kernels

- [x] 3.1 AVX2 (`x86/avx2.rs:134,427+`) and NEON (`aarch64/neon.rs`, +337 lines) `quantize_f32_to_u8`/`dequantize_u8_to_f32` overrides with scalar-tail handling + SAFETY comments; false doc comment at the trait defaults corrected; oracle tests over lengths [0..1000] incl. zero-scale edge — bit-exact vs scalar (2+2+1 tests pass with `--features simd,simd-avx2`; full suite 114/114)
- [x] 3.2 AVX2 `int8_dot_product` via `_mm256_maddubs_epi16` + madd accumulation with sign handling; oracle + i16-intermediate-overflow-extremes tests
- [x] 3.3 `.github/workflows/simd-matrix.yml`: x86_64 job forcing scalar/sse2/avx2 + native aarch64 (ubuntu-24.04-arm) forcing scalar/neon, scoped to `-p vectorizer-core --lib simd::` with correct feature flags (avoids the DashboardAssets trap that killed the old matrix, cf298f7d). NEON compile-verification happens on the CI arm runner — local cross-check blocked by missing aarch64-linux-gnu-gcc for C deps, not by the Rust code

## 4. Hot-path allocations

- [x] 4.1 `try_admit`: `get()` fast path; `entry(to_string())` only on first admission per collection
- [x] 4.2 `QueryKey::from_vector`: xxh3-128 (cache keys need speed, not cryptographic strength; collision = wrong cache hit only)

## 5. Benchmark coverage

- [x] 5.1 Re-registered 7 of 8: core_operations (as-is), search_bench + update_bench (CollectionConfig bit-rot fixed), query_cache_bench (as-is), cache_benchmark (`tracing::info!()` zero-arg fix), benchmark_embeddings (as-is), storage_benchmark (bincode 2.0 → vectorizer::codec). `quantization_benchmark` stays commented with the reason IN Cargo.toml: needs its dataset loader rewritten (await in non-async fn, never populates all_documents, hardcodes ../gov sibling checkouts) — beyond bit-rot repair
- [x] 5.2 New `benches/insert_pipeline.rs` (insert_batch, dim 256, batches 100/1000) + `benches/bm25_vocab.rs` (build_vocabulary 1k docs + fitted embed); `cargo bench --no-run -p vectorizer` compiles all 17 targets; clippy --benches 0 warnings

## 6. Tail (mandatory — enforced by rulebook v5.3.0)

- [x] 6.1 Update or create documentation covering the implementation — CHANGELOG [3.5.0] Performance section; struct-level doc on `Collection::insert_lock`; corrected SimdBackend trait docs; simd-matrix.yml header explains scoping
- [x] 6.2 Write tests covering the new behavior — insert/search concurrency test, PQ + Binary round-trips, None explicit-error test, AVX2 oracle tests (quantize/dequantize/int8 + overflow edge), dispatch avx2-selection test
- [x] 6.3 Run tests and confirm they pass — vectorizer-core 114/114 (`--features simd,simd-avx2`), vectorizer lib 1026, quantization 41, concurrency test debug+release, bench --no-run clean, clippy 0
