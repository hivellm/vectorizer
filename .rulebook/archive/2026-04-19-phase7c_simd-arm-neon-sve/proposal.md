# Proposal: phase7c_simd-arm-neon-sve

## Why

ARM64 is a first-class deployment target: Apple Silicon (every M-series Mac), AWS Graviton (2/3/4), Ampere Altra, Oracle Ampere A1, Azure Cobalt, and virtually every mobile device. All of them expose NEON unconditionally, and Graviton3+/Neoverse V1+ expose SVE/SVE2. Vectorizer currently gives these machines zero SIMD speedup because `src/models/vector_utils_simd.rs` only has x86_64 intrinsics. On an M2 Pro, a scalar dot product runs at ~1 GFLOPS vs ~8 GFLOPS achievable with NEON — an 8× gap paid by every search and every embedding normalization.

SVE/SVE2 adds a vector-length-agnostic programming model: the same binary runs on 128-bit, 256-bit, 512-bit implementations without recompilation. Graviton3 is 256-bit, Fujitsu A64FX is 512-bit. Writing SVE kernels once buys future hardware for free.

## What Changes

New files under `src/simd/aarch64/`:

- `src/simd/aarch64/neon.rs` — `NeonBackend`. Uses 128-bit `float32x4_t` (4 f32 lanes). Intrinsics: `vld1q_f32`, `vmulq_f32`, `vaddq_f32`, `vfmaq_f32` (fused multiply-add, always available on aarch64), `vsubq_f32`, `vaddvq_f32` (horizontal sum, single instruction on ARMv8).
- `src/simd/aarch64/sve.rs` — `SveBackend`. Gated by `cfg(feature = "simd-sve")` and `cfg(target_feature = "sve")`. Uses `svfloat32_t`, `svld1_f32`, `svmla_f32_x`, `svaddv_f32` from the ACLE (`std::arch::aarch64::*`). Loop driver uses `svwhilelt_b32` for the predicate-based tail, eliminating the scalar remainder loop.
- `src/simd/aarch64/sve2.rs` — `Sve2Backend`. Extends `SveBackend` with SVE2-only ops (saturating arithmetic, `svmlslb`) that phase7f will use for quantized int8 dot product.

Dispatch update in `src/simd/dispatch.rs`:

- Selection order on aarch64: SVE2 → SVE → NEON → scalar.
- NEON detection is compile-time: on aarch64 NEON is in the psABI, so `NeonBackend` is always available when `target_arch = "aarch64"`.
- SVE detection: `std::arch::is_aarch64_feature_detected!("sve")`. Requires Rust 1.74+ (already used by the project per `Cargo.toml`).
- SVE2 detection: `std::arch::is_aarch64_feature_detected!("sve2")`.
- `VECTORIZER_SIMD_BACKEND` env override (introduced in 7b) gains `"neon"|"sve"|"sve2"` values.

NEON implementation details:

- `vfmaq_f32` is the fused multiply-add primitive; unlike x86 there is no non-FMA NEON variant worth shipping, so `dot_product` and `euclidean_distance_squared` use it unconditionally.
- `vaddvq_f32` reduces a 4-lane register to a scalar in one instruction (no horizontal shuffle chain needed).
- Tail loop handles `len % 4` with scalar ops.

SVE implementation details:

- Vector length is decided by the CPU; we discover it with `svcntw()` at kernel entry.
- Loop structure: `while (i < len) { pred = svwhilelt_b32(i, len); a = svld1_f32(pred, &A[i]); ... ; i += svcntw(); }` — no separate tail loop needed, SVE handles partial vectors via predication.
- `svaddv_f32(svptrue_b32(), acc)` produces the final scalar reduction.

Apple Silicon caveat:

- M1/M2/M3/M4 do NOT implement SVE; they are NEON-only. The dispatch correctly falls back to NEON on Apple Silicon and the docs call this out.
- Documented in `docs/architecture/simd.md` alongside the AVX-512 downclock note from 7b.

## Impact

- Affected specs: `.rulebook/tasks/phase7c_simd-arm-neon-sve/specs/simd-aarch64/spec.md` (new).
- Affected code: new `src/simd/aarch64/{neon,sve,sve2}.rs`; edits to `src/simd/aarch64/mod.rs`, `src/simd/dispatch.rs`, `Cargo.toml` (feature flags), `docs/architecture/simd.md`.
- Breaking change: NO.
- User benefit: 6-8× speedup on every aarch64 deployment (Apple Silicon dev machines, Graviton cloud hosts, ARM mobile) for all existing f32 ops, plus VLA-correct kernels that scale to wider SVE implementations without recompilation.
