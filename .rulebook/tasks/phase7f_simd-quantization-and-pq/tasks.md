## 1. Prerequisites

- [ ] 1.1 Confirm phases 7a, 7b, 7c, 7d, 7e are merged
- [ ] 1.2 Confirm `int8_dot_product` is present on `SimdBackend` with VNNI and SVE2 implementations from phases 7b and 7c

## 2. Extend SimdBackend trait

- [ ] 2.1 Add `fn quantize_f32_to_u8(&self, src: &[f32], dst: &mut [u8], scale: f32, offset: f32, levels: u32)` to `src/simd/backend.rs`
- [ ] 2.2 Add `fn dequantize_u8_to_f32(&self, src: &[u8], dst: &mut [f32], scale: f32, offset: f32)`
- [ ] 2.3 Add `fn quantize_f32_to_u4_packed(&self, src: &[f32], dst: &mut [u8], scale: f32, offset: f32)` (two 4-bit values per byte)
- [ ] 2.4 Add `fn dequantize_u4_packed_to_f32(&self, src: &[u8], dst: &mut [f32], scale: f32, offset: f32)`
- [ ] 2.5 Add `fn batch_euclidean_distance_sq(&self, query: &[f32], centroids: &[f32], dim: usize, out: &mut [f32])`
- [ ] 2.6 Add `fn pq_asymmetric_distance_lut(&self, codes: &[u8], lut: &[f32], n_subquantizers: usize) -> f32`
- [ ] 2.7 Add `fn sparse_dot_product_matched(&self, a_values: &[f32], b_values: &[f32], match_pairs: &[(u32, u32)]) -> f32`
- [ ] 2.8 Document preconditions and invariants for each primitive

## 3. Scalar backend reference impls

- [ ] 3.1 Implement all new trait methods in `src/simd/scalar.rs` with plain loops
- [ ] 3.2 Property-based tests under `tests/simd/quantization/scalar_oracle.rs`

## 4. x86 implementations

- [ ] 4.1 `Sse2Backend`: `quantize_f32_to_u8` using `_mm_mul_ps`, `_mm_sub_ps`, `_mm_max_ps`/`_mm_min_ps` for clamp, `_mm_cvtps_epi32` + `_mm_packus_epi16`
- [ ] 4.2 `Sse2Backend`: `dequantize_u8_to_f32` using `_mm_cvtepu8_epi32` + `_mm_cvtepi32_ps` + FMA-free mul/add
- [ ] 4.3 `Sse2Backend`: 4-bit pack/unpack via shifts and `_mm_and_si128`
- [ ] 4.4 `Sse2Backend`: remaining trait methods (`batch_euclidean_distance_sq`, `pq_asymmetric_distance_lut`, `sparse_dot_product_matched`)
- [ ] 4.5 `Avx2Backend`: 8-wide variants of every primitive; use `_mm256_i32gather_ps` for the PQ LUT path
- [ ] 4.6 `Avx512Backend`: 16-wide variants; use `_mm512_permutexvar_ps` for the PQ LUT (faster than gather on Skylake-X+)
- [ ] 4.7 `Avx512VnniBackend`: override `pq_asymmetric_distance_lut` to use `_mm512_dpbusd_epi32` when the LUT is quantized to INT8

## 5. aarch64 implementations

- [ ] 5.1 `NeonBackend`: `quantize_f32_to_u8` using `vcvtnq_s32_f32` (round-to-nearest) + `vqmovun_s16` (saturating narrow)
- [ ] 5.2 `NeonBackend`: `dequantize_u8_to_f32` using `vmovl_u8` + `vcvtq_f32_u32` + FMA
- [ ] 5.3 `NeonBackend`: 4-bit pack/unpack via `vshrq_n_s16` and `vandq_s8`
- [ ] 5.4 `NeonBackend`: PQ LUT gather emulated via `vqtbl4q_u8` for small LUTs; indexed loads for larger LUTs
- [ ] 5.5 `SveBackend`: predicated VLA variants of every primitive
- [ ] 5.6 `Sve2Backend`: override int8 paths with `svdot_s32` where applicable

## 6. wasm32 implementations

- [ ] 6.1 `Wasm128Backend`: `quantize_f32_to_u8` using `f32x4_nearest` + saturating narrow
- [ ] 6.2 `Wasm128Backend`: `dequantize_u8_to_f32` using `u8x16_extract_lane` + `f32x4_convert_i32x4`
- [ ] 6.3 `Wasm128Backend`: remaining primitives (gather is emulated)

## 7. Rewire quantization callers

- [ ] 7.1 Replace the inner loop at `src/quantization/scalar.rs:106` with `crate::simd::backend().quantize_f32_to_u8(...)`
- [ ] 7.2 Replace the inner loop at `src/quantization/scalar.rs:119` with `dequantize_u8_to_f32`
- [ ] 7.3 Replace 4-bit pack/unpack loops with the packed trait methods
- [ ] 7.4 Add a 2-bit pack/unpack path in the same style (extend trait with `quantize_f32_to_u2_packed` variants if the shape differs materially)

## 8. Rewire PQ callers

- [ ] 8.1 Replace the k-means distance loop at `src/quantization/product.rs:129-145` with `batch_euclidean_distance_sq`
- [ ] 8.2 Replace the PQ asymmetric-distance scan with `pq_asymmetric_distance_lut`
- [ ] 8.3 In `src/quantization/hnsw_integration.rs`, when `crate::simd::selected_backend_name()` is `"avx512_vnni"` or `"sve2"`, switch the quantized-search distance to the `int8_dot_product` path
- [ ] 8.4 Verify recall@10 on the existing benchmark dataset is unchanged within 0.1% vs the pre-task baseline

## 9. Rewire sparse-vector dot product

- [ ] 9.1 Refactor `src/models/sparse_vector.rs:85-103` to collect matched index pairs first, then call `sparse_dot_product_matched`
- [ ] 9.2 Keep a length threshold: for very small matched-pair sets (< 8) stay on the scalar two-pointer merge

## 10. Tail (mandatory — enforced by rulebook v5.3.0)

- [ ] 10.1 Extend `docs/architecture/simd.md` and `docs/architecture/quantization.md` with the new primitives and selection rules
- [ ] 10.2 Add per-backend tests under `tests/simd/quantization/` covering quantize/dequantize numerical parity (within 1 ULP), PQ training parity, PQ ADC parity, sparse-dot parity
- [ ] 10.3 Add an integration test asserting recall@10 on a fixture dataset is unchanged within 0.1% for quantized search before and after rewiring
- [ ] 10.4 Run `cargo check --all-features`, `cargo clippy --all-features -- -D warnings`, `cargo test --all-features -- simd::quantization quantization::scalar quantization::product` and confirm zero warnings and 100% pass
