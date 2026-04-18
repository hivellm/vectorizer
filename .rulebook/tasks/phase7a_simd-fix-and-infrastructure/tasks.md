## 1. Bug fix — brute-force cosine using scalar path

- [ ] 1.1 Replace scalar `cosine_similarity` call at `src/quantization/hnsw_integration.rs:173` with `crate::simd::cosine_similarity`
- [ ] 1.2 Replace scalar `cosine_similarity` call at `src/quantization/hnsw_integration.rs:194` with `crate::simd::cosine_similarity`
- [ ] 1.3 Remove the now-dead local `cosine_similarity` helper at `src/quantization/hnsw_integration.rs:385-399`
- [ ] 1.4 Add regression test `tests/quantization/brute_force_uses_simd.rs` asserting numerical parity with scalar cosine on random vectors

## 2. Module skeleton

- [ ] 2.1 Create `src/simd/mod.rs`, `src/simd/backend.rs`, `src/simd/dispatch.rs`, `src/simd/scalar.rs`
- [ ] 2.2 Create `src/simd/x86/mod.rs` gated by `cfg(target_arch = "x86_64")`
- [ ] 2.3 Create `src/simd/aarch64/mod.rs` gated by `cfg(target_arch = "aarch64")`
- [ ] 2.4 Create `src/simd/wasm/mod.rs` gated by `cfg(target_arch = "wasm32")`
- [ ] 2.5 Register `pub mod simd;` in `src/lib.rs`

## 3. SimdBackend trait

- [ ] 3.1 Define `SimdBackend` trait in `src/simd/backend.rs` with methods: `dot_product`, `euclidean_distance_squared`, `cosine_similarity`, `l2_norm`
- [ ] 3.2 Add `name(&self) -> &'static str` on the trait for diagnostics
- [ ] 3.3 Require `Send + Sync + 'static` on every backend impl
- [ ] 3.4 Document invariants (slice length equality, pre-normalization assumption for cosine)

## 4. Scalar fallback backend

- [ ] 4.1 Implement `ScalarBackend` in `src/simd/scalar.rs` for every trait method
- [ ] 4.2 Add property-based tests under `tests/simd/scalar_oracle.rs` comparing against hand-written loops on random vectors
- [ ] 4.3 Ensure `ScalarBackend` compiles on every target (x86_64, aarch64, wasm32)

## 5. Runtime dispatch

- [ ] 5.1 Implement `dispatch::backend() -> &'static dyn SimdBackend` using `std::sync::OnceLock`
- [ ] 5.2 x86_64 selection order: AVX-512F > AVX2+FMA > AVX2 > SSE2 > scalar (placeholders for non-scalar until 7b fills them)
- [ ] 5.3 aarch64 selection order: SVE > NEON > scalar (placeholders until 7c fills them)
- [ ] 5.4 wasm32 selection: SIMD128 if `cfg(target_feature = "simd128")` else scalar (placeholder until 7d)
- [ ] 5.5 Expose `dispatch::selected_backend_name()` for startup log and diagnostics
- [ ] 5.6 Expose top-level `crate::simd::dot_product`, `crate::simd::euclidean_distance`, `crate::simd::cosine_similarity`, `crate::simd::l2_norm` convenience functions

## 6. Cargo features

- [ ] 6.1 Add `simd`, `simd-avx2`, `simd-avx512`, `simd-neon`, `simd-sve`, `simd-wasm` features to `Cargo.toml`
- [ ] 6.2 Update `default = [...]` to include `simd`, `simd-avx2`, `simd-neon`, `simd-wasm`
- [ ] 6.3 Gate each ISA-specific backend behind its feature flag

## 7. Migrate existing wrappers

- [ ] 7.1 Rewrite `src/models/vector_utils_simd.rs::dot_product_simd` to call `crate::simd::dot_product`
- [ ] 7.2 Rewrite `src/models/vector_utils_simd.rs::euclidean_distance_simd` to call `crate::simd::euclidean_distance`
- [ ] 7.3 Rewrite `src/models/vector_utils_simd.rs::cosine_similarity_simd` to call `crate::simd::cosine_similarity`
- [ ] 7.4 Keep the existing AVX2 intrinsics in `vector_utils_simd.rs` as a temporary in-module `Avx2Backend` impl so behaviour is preserved until 7b extracts them

## 8. Observability

- [ ] 8.1 Log selected backend name at server startup via `tracing::info!`
- [ ] 8.2 Expose `selected_backend_name()` on a new `/metrics` gauge label `simd_backend`

## 9. Tail (mandatory — enforced by rulebook v5.3.0)

- [ ] 9.1 Document the `src/simd/` architecture in `docs/architecture/simd.md` with the backend-selection flowchart
- [ ] 9.2 Write unit tests `src/simd/dispatch.rs` (selection order), `src/simd/scalar.rs` (numerical oracle) and the regression test from 1.4
- [ ] 9.3 Run `cargo check --all-features`, `cargo clippy --all-features -- -D warnings` and `cargo test --all-features -- simd` and confirm zero warnings and 100% pass
