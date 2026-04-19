# Proposal: phase7f_simd-quantization-and-pq

## Why

Quantization is the biggest remaining scalar hotspot after the f32 primitives are covered by phases 7a-7e. Every vector that enters a quantized collection is encoded scalar, every query is decoded or compared scalar, and product-quantization k-means training walks distances one centroid at a time:

- `quantize_8bit` at `src/quantization/scalar.rs:106` — per-element subtract/divide/clamp/round/cast loop. 5-8× achievable because the operation is trivially data-parallel.
- `dequantize_8bit` at `src/quantization/scalar.rs:119` — per-element cast/multiply/add. 8× achievable.
- `quantize_4bit` / `quantize_2bit` in the same file — bit packing with shifts; 2-3× achievable with byte-level SIMD permute.
- Product quantization k-means in `src/quantization/product.rs:112` — nested loop computing Euclidean to every centroid for every subvector. 3-5× achievable with batch distance.
- PQ asymmetric distance (the "ADC" lookup-table inner loop) — the whole point of PQ is that queries become a sum over precomputed 256-entry LUTs indexed by the quantized codes. This is a `gather + horizontal-sum` pattern that maps well to AVX2 `_mm256_i32gather_ps` and AVX-512 `_mm512_permutexvar_ps`; 4-6× achievable.
- Sparse-vector dot product in `src/models/sparse_vector.rs:85` — two-pointer merge is harder to vectorize, but the accumulation of matched pairs is SIMD-friendly once matches are grouped.

AVX-512 VNNI (landed in 7b) and SVE2 (landed in 7c) both expose a single-instruction INT8 dot product that is 4× faster than the f32 AVX2 path — using them requires wiring the int8 primitive from the trait into the PQ asymmetric distance code here.

## What Changes

Extend the `SimdBackend` trait (`src/simd/backend.rs`) with quantization-specific primitives:

- `fn quantize_f32_to_u8(&self, src: &[f32], dst: &mut [u8], scale: f32, offset: f32, levels: u32)` — fused subtract/scale/clamp/round/cast.
- `fn dequantize_u8_to_f32(&self, src: &[u8], dst: &mut [f32], scale: f32, offset: f32)` — fused cast/multiply/add.
- `fn quantize_f32_to_u4_packed(&self, src: &[f32], dst: &mut [u8], scale: f32, offset: f32)` — packs two 4-bit values per byte.
- `fn dequantize_u4_packed_to_f32(&self, src: &[u8], dst: &mut [f32], scale: f32, offset: f32)`.
- `fn batch_euclidean_distance_sq(&self, query: &[f32], centroids: &[f32], dim: usize, n_centroids: usize, out: &mut [f32])` — compute distance from one query to N centroids in a single call; backends fuse the outer loop to amortize query loads.
- `fn pq_asymmetric_distance_lut(&self, codes: &[u8], lut: &[f32], n_subquantizers: usize) -> f32` — gather-based sum over precomputed LUTs.
- `fn sparse_dot_product_matched(&self, a_values: &[f32], b_values: &[f32], match_pairs: &[(u32, u32)]) -> f32` — called after the merge step to compute the sum over matched index pairs.
- The `int8_dot_product` method from phases 7b (VNNI) / 7c (SVE2) is now consumed here.

Implementation per backend:

- Every backend (scalar, sse2, avx2, avx512, avx512_vnni, neon, sve, sve2, wasm128) implements each primitive. Quantization kernels use broadcast scale/offset and SIMD round-to-nearest (`_mm256_round_ps`, `vrndnq_f32`, `f32x4_nearest`). Bit-packing uses shifts + `_mm256_packus_epi16` (x86) or `vqmovun_s16` (NEON). Gather-based LUT: `_mm256_i32gather_ps` (AVX2), `_mm512_permutexvar_ps` (AVX-512), emulated via indexed loads on NEON/WASM.

Call-site rewiring:

- `src/quantization/scalar.rs::quantize_8bit` and `::dequantize_8bit` — replace the inner loops with calls to the new trait methods.
- `src/quantization/scalar.rs::quantize_4bit` / `_2bit` — use the packed trait methods.
- `src/quantization/product.rs::train_subquantizers` (the k-means loop) — replace per-centroid distance calls with `batch_euclidean_distance_sq`.
- `src/quantization/product.rs` ADC scan — replace scalar LUT sum with `pq_asymmetric_distance_lut`.
- `src/quantization/hnsw_integration.rs` — quantized-search code path: when the backend is Avx512Vnni or Sve2, switch the distance metric to `int8_dot_product` directly instead of the f32 path.
- `src/models/sparse_vector.rs::dot_product` — keep the two-pointer merge but feed the matched-pair batch into `sparse_dot_product_matched`.

Precision guardrails:

- All quantize/dequantize methods must match the scalar reference within one ULP on the same input; property-based tests enforce this.
- The int8 dot product code path uses symmetric quantization (zero-point = 128) consistent with Vectorizer's existing quantization spec; asymmetric support is left as a non-goal to keep the task scoped.

## Impact

- Affected specs: `.rulebook/tasks/phase7f_simd-quantization-and-pq/specs/simd-quantization/spec.md` (new).
- Affected code: `src/simd/backend.rs` (trait extension), every backend implementation file under `src/simd/`, `src/quantization/scalar.rs`, `src/quantization/product.rs`, `src/quantization/hnsw_integration.rs`, `src/models/sparse_vector.rs`.
- Breaking change: NO. Public quantization APIs preserved; behaviour preserved within numerical tolerance.
- User benefit: 5-8× on quantize/dequantize (every indexed vector), 3-5× on PQ training, 4-6× on PQ asymmetric-distance scan (the dominant cost of quantized search), 4× on int8 distance when AVX-512 VNNI or SVE2 are available.
