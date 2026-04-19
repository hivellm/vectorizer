//! Runtime backend dispatch.
//!
//! Picks the best [`SimdBackend`] for the running CPU once at first
//! use, caches it in a `OnceLock`, and hands `&'static dyn SimdBackend`
//! references to every call site. The selection logic is a per-arch
//! priority list — phase7a wires only the AVX2 path on x86_64 (via
//! the existing `models::vector_utils_simd` AVX2 implementation
//! re-exposed as a backend); phases 7b–7d will fill in AVX-512,
//! NEON/SVE, and WASM128.
//!
//! Why `OnceLock` and not a `lazy_static`/`std::sync::Mutex`:
//!
//! - The selected backend is immutable for the life of the process —
//!   `OnceLock` is the cheapest fit. After initialisation it's a
//!   single relaxed atomic load on every dispatch.
//! - We never need to swap the backend at runtime. If a future
//!   workload wants test-only override, do it through the
//!   compile-time `simd` feature flag, not a runtime mutator.

use std::sync::OnceLock;

use super::backend::SimdBackend;
use super::scalar::ScalarBackend;

/// Returns the best backend for the running CPU, picking it on the
/// first call and reusing the same `&'static dyn SimdBackend` from
/// then on. Safe to call from multiple threads.
pub fn backend() -> &'static dyn SimdBackend {
    static CACHED: OnceLock<&'static dyn SimdBackend> = OnceLock::new();
    *CACHED.get_or_init(select_backend)
}

/// Diagnostic helper. Returns the `name()` of the backend the
/// dispatcher chose. Used by the `/metrics` `simd_backend` gauge
/// label and the startup log line. Calling this also primes the
/// `OnceLock` if it hasn't been hit yet.
pub fn selected_backend_name() -> &'static str {
    backend().name()
}

/// Selection logic. Per-arch priority lists below; everything outside
/// the supported arches falls back to scalar. Adding a new ISA means
/// a new branch here AND a new feature flag in `Cargo.toml`.
fn select_backend() -> &'static dyn SimdBackend {
    // Master `simd` feature — when off, every dispatch goes scalar.
    // Useful for debugging numerical drift between backends.
    #[cfg(not(feature = "simd"))]
    {
        return &ScalarBackend;
    }

    #[cfg(all(feature = "simd", target_arch = "x86_64"))]
    {
        // AVX-512F slot reserved for phase7b. Detection helper exists
        // (`is_x86_feature_detected!("avx512f")`) but no backend yet,
        // so the branch is a no-op for now.
        // SSE2 is a baseline guarantee on x86_64; we still gate it
        // behind a future backend in case someone runs an exotic
        // emulator.

        #[cfg(feature = "simd-avx2")]
        {
            if std::is_x86_feature_detected!("avx2") {
                return &super::x86::avx2::Avx2Backend;
            }
        }
    }

    #[cfg(all(feature = "simd", target_arch = "aarch64"))]
    {
        // SVE > NEON. Both backends land in phase7c; the priority
        // list lives here so future additions touch one file.
    }

    #[cfg(all(feature = "simd", target_arch = "wasm32"))]
    {
        // WASM SIMD128 — a `cfg(target_feature = "simd128")` gate
        // controls availability at compile time (browsers without
        // SIMD support fall back to scalar at runtime by simply not
        // shipping the SIMD wasm module). The backend lives at
        // `src/simd/wasm/mod.rs`; phase7d implements it.
    }

    &ScalarBackend
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;

    #[test]
    fn backend_is_stable_across_calls() {
        // `OnceLock` semantics: every call returns the same pointer.
        let a = backend();
        let b = backend();
        // Comparing trait-object addresses by casting to pointer-to-unit.
        let a_addr = a as *const dyn SimdBackend as *const () as usize;
        let b_addr = b as *const dyn SimdBackend as *const () as usize;
        assert_eq!(a_addr, b_addr);
    }

    #[test]
    fn name_is_one_of_the_supported_set() {
        let n = selected_backend_name();
        let supported = ["avx512", "avx2", "sse2", "neon", "sve", "wasm128", "scalar"];
        assert!(
            supported.contains(&n),
            "selected_backend_name() = {n:?}, expected one of {supported:?}"
        );
    }

    #[test]
    #[cfg(all(target_arch = "x86_64", feature = "simd-avx2"))]
    fn x86_picks_avx2_when_available() {
        // On a CI box with AVX2 (every modern x86_64 server qualifies),
        // we must land on the AVX2 backend, not scalar.
        if std::is_x86_feature_detected!("avx2") {
            assert_eq!(selected_backend_name(), "avx2");
        }
    }
}
