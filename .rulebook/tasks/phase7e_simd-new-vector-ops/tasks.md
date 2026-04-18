## 1. Prerequisites

- [ ] 1.1 Confirm phases 7a, 7b, 7c, 7d are merged and every ISA backend exists

## 2. Extend SimdBackend trait

- [ ] 2.1 Add `fn normalize_in_place(&self, a: &mut [f32])` to `src/simd/backend.rs`
- [ ] 2.2 Add `fn manhattan_distance(&self, a: &[f32], b: &[f32]) -> f32`
- [ ] 2.3 Add `fn add_assign(&self, a: &mut [f32], b: &[f32])`, `fn sub_assign(&self, a: &mut [f32], b: &[f32])`, `fn scale(&self, a: &mut [f32], s: f32)`
- [ ] 2.4 Add `fn horizontal_min_index(&self, a: &[f32]) -> Option<(usize, f32)>`
- [ ] 2.5 Document each method's preconditions (equal slice lengths where applicable, zero-norm handling)

## 3. Scalar backend implementations

- [ ] 3.1 Implement all new trait methods in `src/simd/scalar.rs` with plain loops
- [ ] 3.2 Property-based tests comparing against hand-written reference implementations

## 4. x86 backend implementations

- [ ] 4.1 `Sse2Backend`: `normalize_in_place` via two-pass (norm then divide) reusing dot kernel
- [ ] 4.2 `Sse2Backend`: `manhattan_distance` using `_mm_andnot_ps` with sign-mask `_mm_set1_ps(-0.0)`
- [ ] 4.3 `Sse2Backend`: `add_assign` / `sub_assign` / `scale` using `_mm_add_ps` / `_mm_sub_ps` / `_mm_mul_ps`
- [ ] 4.4 `Sse2Backend`: `horizontal_min_index` via `_mm_min_ps` + index tracking in parallel
- [ ] 4.5 `Avx2Backend`: same primitives on 8-lane `__m256`, FMA-aware where applicable
- [ ] 4.6 `Avx512Backend`: same primitives on 16-lane `__m512`; use `_mm512_abs_ps` for Manhattan and `_mm512_reduce_min_ps` for horizontal min

## 5. aarch64 backend implementations

- [ ] 5.1 `NeonBackend`: `manhattan_distance` using `vabsq_f32`
- [ ] 5.2 `NeonBackend`: `normalize_in_place`, `add_assign`, `sub_assign`, `scale`, `horizontal_min_index` (use `vminvq_f32` for horizontal min)
- [ ] 5.3 `SveBackend`: same primitives with predicated loops
- [ ] 5.4 `Sve2Backend`: delegate f32 primitives to `SveBackend`

## 6. wasm32 backend implementations

- [ ] 6.1 `Wasm128Backend`: `manhattan_distance` using `f32x4_abs`
- [ ] 6.2 `Wasm128Backend`: `normalize_in_place`, `add_assign`, `sub_assign`, `scale`, `horizontal_min_index`

## 7. Rewire call sites

- [ ] 7.1 Rewrite `src/models/mod.rs::normalize_vector` to call `crate::simd::normalize_in_place` on a cloned buffer
- [ ] 7.2 Introduce `crate::simd::normalize_in_place` public helper and migrate internal callers to the in-place form where the buffer is owned
- [ ] 7.3 Rewrite `src/models/sparse_vector.rs::norm` to call `crate::simd::l2_norm(&self.values)`
- [ ] 7.4 Update `src/db/collection.rs:594` and `:788` to use the SIMD normalizer
- [ ] 7.5 Add `DistanceMetric::Manhattan` variant wiring into the metric dispatch so `crate::simd::manhattan_distance` is called when the user selects it

## 8. Top-k inner loop

- [ ] 8.1 Locate the candidate-scan loop in `src/db/optimized_hnsw.rs` search
- [ ] 8.2 Replace inner scalar-compare with `horizontal_min_index` on 8-wide batches when heap size ≥ 8
- [ ] 8.3 Verify search recall is unchanged with an integration test against the existing dataset fixtures

## 9. Tail (mandatory — enforced by rulebook v5.3.0)

- [ ] 9.1 Extend `docs/architecture/simd.md` with the new ops table and Manhattan metric usage example in `docs/api/distance-metrics.md`
- [ ] 9.2 Add per-backend unit tests for each new primitive in `tests/simd/ops/` covering lengths 1, 3, 7, 8, 15, 16, 31, 32, 63, 64, 127, 1024 and zero-norm edge case
- [ ] 9.3 Add integration tests for the normalize rewire (`tests/integration/normalize_simd_parity.rs`) and Manhattan metric (`tests/integration/manhattan_distance.rs`)
- [ ] 9.4 Run `cargo check --all-features`, `cargo clippy --all-features -- -D warnings`, `cargo test --all-features -- simd ops normalize manhattan` and confirm zero warnings and 100% pass
