# Proposal: phase7a_simd-fix-and-infrastructure

## Why

Vectorizer today only has AVX2 SIMD in `src/models/vector_utils_simd.rs` for dot product, euclidean and cosine. Every other hot path (normalize, quantize/dequantize, sparse norm, brute-force search, PQ k-means, batch distance) is scalar. ARM64 (Apple Silicon, AWS Graviton), AVX-512 CPUs and WASM builds get zero SIMD benefit, and there is a latent bug in `src/quantization/hnsw_integration.rs` where brute-force cosine calls a locally-defined scalar `cosine_similarity` instead of the SIMD wrapper, losing the 3-8× speedup that already exists.

Before adding any new ISA or new SIMD operation we need a single, coherent dispatch layer: one trait that every backend implements, runtime detection cached once per process, compile-time feature gating per ISA, and a scalar fallback that is always correct. Without this foundation, phases 7b–7g will either duplicate detection logic across files or end up with per-ISA ad-hoc modules that silently diverge.

This task also captures the trivial brute-force cosine fix so the bug does not persist while the infrastructure is being built.

## What Changes

New module `src/simd/` with the following layout:

- `src/simd/mod.rs` — public API; re-exports `SimdBackend` trait, top-level functions (`dot_product`, `euclidean_distance`, `cosine_similarity`, `l2_norm`, etc.) that dispatch to the selected backend at runtime.
- `src/simd/backend.rs` — `SimdBackend` trait defining every SIMD-accelerated primitive (f32 ops only in this task; quantization / sparse ops land in later phases).
- `src/simd/dispatch.rs` — runtime CPU feature detection, `OnceLock<&'static dyn SimdBackend>` cache, platform-specific selection order (x86_64: AVX-512 > AVX2+FMA > AVX2 > SSE2 > scalar).
- `src/simd/scalar.rs` — `ScalarBackend` implementing every primitive with a plain loop; always available, used on unsupported targets and as correctness oracle for tests.
- `src/simd/x86/mod.rs` — stub placeholders for `Avx2Backend` / `Avx512Backend` (filled in 7b); module gated by `cfg(target_arch = "x86_64")`.
- `src/simd/aarch64/mod.rs` — stub for `NeonBackend` / `SveBackend` (filled in 7c); gated by `cfg(target_arch = "aarch64")`.
- `src/simd/wasm/mod.rs` — stub for `Wasm128Backend` (filled in 7d); gated by `cfg(target_arch = "wasm32")`.

New Cargo features in `Cargo.toml`:

- `simd` — master flag, default on.
- `simd-avx2`, `simd-avx512`, `simd-neon`, `simd-sve`, `simd-wasm` — per-ISA opt-outs for constrained builds. Default set: `["simd-avx2", "simd-neon", "simd-wasm"]` (the widely-available baselines).

Existing code migration:

- `src/models/vector_utils_simd.rs` — keep the file but have `dot_product_simd` / `euclidean_distance_simd` / `cosine_similarity_simd` delegate to `crate::simd::dispatch::backend()` instead of inlining AVX2. The AVX2 intrinsics move into `src/simd/x86/avx2.rs` in task 7b — this task leaves them in place and only introduces the trait + dispatch scaffolding so `vector_utils_simd` becomes a thin compatibility shim.
- `src/models/mod.rs` (`dot_product`, `euclidean_distance`, `cosine_similarity`, `normalize_vector`) — continue to work; no API break.

Bug fix (trivial, 1-line):

- `src/quantization/hnsw_integration.rs:173` and `:194` — replace the call to the file-local scalar `cosine_similarity(query, vector)` with `crate::simd::cosine_similarity(query, vector)`. Remove the now-dead local `cosine_similarity` at lines 385-399.

Observability:

- `src/simd/dispatch.rs` exposes `selected_backend_name()` returning `"avx512" | "avx2" | "sse2" | "neon" | "sve" | "wasm128" | "scalar"`; logged once at server startup so users can see which SIMD path the binary is using.

## Impact

- Affected specs: `.rulebook/tasks/phase7a_simd-fix-and-infrastructure/specs/simd/spec.md` (new).
- Affected code: new `src/simd/` module tree; edits to `src/models/vector_utils_simd.rs`, `src/models/mod.rs`, `src/quantization/hnsw_integration.rs`, `Cargo.toml`, `src/lib.rs`.
- Breaking change: NO. Existing public functions keep their signatures and behaviour; only their implementations are re-routed.
- User benefit: immediate 3-8× speedup on the brute-force quantized search code path (bug fix) and a single extension point for every subsequent SIMD task in phase 7.
