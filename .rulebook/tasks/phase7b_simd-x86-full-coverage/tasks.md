## 1. Prerequisites

- [ ] 1.1 Confirm `phase7a_simd-fix-and-infrastructure` is merged and `src/simd/` scaffolding exists
- [ ] 1.2 Confirm `SimdBackend` trait is stable and the runtime dispatch `OnceLock` is in place

## 2. SSE2 backend (baseline)

- [ ] 2.1 Create `src/simd/x86/sse2.rs` with `Sse2Backend` struct and `name() = "sse2"`
- [ ] 2.2 Implement `dot_product` using `_mm_loadu_ps`, `_mm_mul_ps`, `_mm_add_ps` on 4-lane `__m128`
- [ ] 2.3 Implement `euclidean_distance_squared` using `_mm_sub_ps`, `_mm_mul_ps`, `_mm_add_ps`
- [ ] 2.4 Implement `l2_norm` reusing the dot-product kernel on `(a, a)` plus `sqrt`
- [ ] 2.5 Implement `cosine_similarity` assuming pre-normalized inputs (delegates to `dot_product`)
- [ ] 2.6 Implement `horizontal_sum_sse2` helper reducing `__m128` to scalar
- [ ] 2.7 Add tail-loop handling for len % 4 remainder

## 3. AVX2 backend (with optional FMA)

- [ ] 3.1 Create `src/simd/x86/avx2.rs` with `Avx2Backend { with_fma: bool }`
- [ ] 3.2 Port existing `dot_product_avx2` from `src/models/vector_utils_simd.rs`, branching on `with_fma` at compile-time via two private monomorphized fns
- [ ] 3.3 Use `_mm256_fmadd_ps` in the FMA path for both `dot_product` and `euclidean_distance_squared`
- [ ] 3.4 Port `euclidean_distance_avx2` and `horizontal_sum_avx2`
- [ ] 3.5 Implement `l2_norm` and `cosine_similarity` reusing the kernels
- [ ] 3.6 Keep `#[target_feature(enable = "avx2")]` / `"avx2,fma"` on every inner kernel
- [ ] 3.7 Mark kernels `#[inline]` and verify via `cargo asm` that FMA instructions are emitted when flag set

## 4. AVX-512 backend

- [ ] 4.1 Create `src/simd/x86/avx512.rs` with `Avx512Backend` gated by `cfg(feature = "simd-avx512")`
- [ ] 4.2 Implement `dot_product` using `__m512`, `_mm512_loadu_ps`, `_mm512_fmadd_ps`
- [ ] 4.3 Implement `euclidean_distance_squared` using `_mm512_sub_ps` + `_mm512_fmadd_ps`
- [ ] 4.4 Implement `l2_norm` + `cosine_similarity`
- [ ] 4.5 Implement `horizontal_sum_avx512` via `_mm512_reduce_add_ps`
- [ ] 4.6 Use `_mm512_mask_*` intrinsics for the final partial block instead of a scalar tail loop
- [ ] 4.7 Add `#[target_feature(enable = "avx512f,avx512dq,avx512bw")]` on each kernel

## 5. AVX-512 VNNI backend

- [ ] 5.1 Create `src/simd/x86/avx512_vnni.rs` with `Avx512VnniBackend` gated by `cfg(feature = "simd-avx512")`
- [ ] 5.2 Delegate all f32 methods to `Avx512Backend` via composition
- [ ] 5.3 Add `int8_dot_product(&self, a: &[i8], b: &[i8]) -> i32` as a new trait method on `SimdBackend` with default scalar fallback
- [ ] 5.4 Implement `int8_dot_product` with `_mm512_dpbusd_epi32` in this backend
- [ ] 5.5 Document that the int8 primitive is consumed by the phase7f quantized-distance code path

## 6. Dispatch integration

- [ ] 6.1 Update `src/simd/dispatch.rs` x86_64 selection order: AVX-512 + VNNI → AVX-512 → AVX2+FMA → AVX2 → SSE2 → scalar
- [ ] 6.2 Read `VECTORIZER_SIMD_BACKEND` env var at `OnceLock` init; if set to `"scalar"|"sse2"|"avx2"|"avx512"`, force that backend
- [ ] 6.3 Cache individual feature-detection results (`is_x86_feature_detected!`) in static `OnceLock<bool>` so repeated queries are branch-free
- [ ] 6.4 Log the selected backend at `INFO` level on first call

## 7. Trim compatibility shim

- [ ] 7.1 Delete the AVX2 intrinsics now migrated to `src/simd/x86/avx2.rs` from `src/models/vector_utils_simd.rs`
- [ ] 7.2 Leave `vector_utils_simd.rs` as pure forwarding functions (≤10 lines of body total)
- [ ] 7.3 Verify no other module still imports `vector_utils_simd` internals

## 8. Tail (mandatory — enforced by rulebook v5.3.0)

- [ ] 8.1 Extend `docs/architecture/simd.md` with the x86 ladder, env-override knob, and the AVX-512 downclock caveat
- [ ] 8.2 Add backend-specific tests under `tests/simd/x86/`: per-backend correctness vs. scalar on random vectors with lengths 1, 3, 7, 8, 15, 16, 31, 32, 63, 64, 127, 1024, 4096
- [ ] 8.3 Gate AVX-512 tests with `is_x86_feature_detected!("avx512f")` at runtime so they are excluded on CI hosts without the feature
- [ ] 8.4 Run `cargo check --all-features`, `cargo clippy --all-features -- -D warnings`, `cargo test --all-features -- simd::x86` and confirm zero warnings and 100% pass
