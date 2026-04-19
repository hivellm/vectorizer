//! Runtime backend dispatch.
//!
//! Picks the best [`SimdBackend`] for the running CPU once at first
//! use, caches it in a `OnceLock`, and hands `&'static dyn SimdBackend`
//! references to every call site.
//!
//! ## Selection ladder (x86_64, post phase7b)
//!
//! Highest priority wins. Every branch is gated by both the runtime
//! feature detector AND the matching `simd-*` Cargo feature, so
//! disabling an opt-in feature shrinks the binary AND shifts the
//! ladder one step down.
//!
//! 1. AVX-512 + VNNI (`simd-avx512`)
//! 2. AVX-512F (`simd-avx512`)
//! 3. AVX2 + FMA (`simd-avx2`) — the FMA flag is auto-detected and
//!    folded into [`crate::simd::x86::avx2::Avx2Backend::auto_detect`]
//! 4. AVX2 (`simd-avx2`) — same backend struct with `with_fma=false`
//!    when the CPU lacks FMA
//! 5. SSE2 — psABI baseline; always available on x86_64
//! 6. scalar — `simd` feature off, or non-x86 / unsupported target
//!
//! ## Env override
//!
//! Read once from `VECTORIZER_SIMD_BACKEND`. Accepted values:
//! `"scalar" | "sse2" | "avx2" | "avx512" | "avx512vnni"`. Setting
//! the env var FORCES the backend regardless of CPU capability. If
//! the requested backend's runtime check fails the dispatcher logs a
//! warning and falls back to scalar — never silently ignoring the
//! override and continuing on a faster path.
//!
//! Why an env override at all? Two operator stories:
//!
//! - **AVX-512 downclock**: on Skylake-X server CPUs, AVX-512 trips
//!   a ~10% sustained frequency drop. For a workload that's not
//!   batch-throughput-bound, forcing AVX2 can be a net win. Knob:
//!   `VECTORIZER_SIMD_BACKEND=avx2`.
//! - **Numerical drift debugging**: forcing scalar with
//!   `VECTORIZER_SIMD_BACKEND=scalar` collapses every code path to
//!   the oracle so divergences in test output have one suspect.

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

/// Read the `VECTORIZER_SIMD_BACKEND` env override and resolve it to
/// a backend instance, or `None` to fall through to auto-detection.
/// An unknown value is treated as no override (with a warning) so a
/// typo doesn't silently downgrade everyone to scalar.
fn env_override() -> Option<&'static dyn SimdBackend> {
    let raw = std::env::var("VECTORIZER_SIMD_BACKEND").ok()?;
    let normalised = raw.to_ascii_lowercase();
    match normalised.as_str() {
        "scalar" => Some(&ScalarBackend),

        #[cfg(target_arch = "x86_64")]
        "sse2" => Some(&super::x86::sse2::Sse2Backend),

        #[cfg(all(target_arch = "x86_64", feature = "simd-avx2"))]
        "avx2" => {
            // Cache one Avx2Backend instance so the with_fma flag is
            // sampled exactly once even when the env override is
            // exercised by tests (which can call back into dispatch
            // multiple times).
            static CACHED: OnceLock<super::x86::avx2::Avx2Backend> = OnceLock::new();
            Some(CACHED.get_or_init(super::x86::avx2::Avx2Backend::auto_detect))
        }

        #[cfg(all(target_arch = "x86_64", feature = "simd-avx512"))]
        "avx512" => {
            if std::is_x86_feature_detected!("avx512f") {
                Some(&super::x86::avx512::Avx512Backend)
            } else {
                tracing::warn!(
                    "VECTORIZER_SIMD_BACKEND=avx512 requested but CPU lacks AVX-512F; \
                     falling back to scalar"
                );
                Some(&ScalarBackend)
            }
        }

        #[cfg(all(target_arch = "x86_64", feature = "simd-avx512"))]
        "avx512vnni" => {
            if std::is_x86_feature_detected!("avx512vnni") {
                Some(&super::x86::avx512_vnni::Avx512VnniBackend)
            } else {
                tracing::warn!(
                    "VECTORIZER_SIMD_BACKEND=avx512vnni requested but CPU lacks AVX-512 VNNI; \
                     falling back to scalar"
                );
                Some(&ScalarBackend)
            }
        }

        other => {
            tracing::warn!(
                "VECTORIZER_SIMD_BACKEND={other:?} is not a recognised backend name; \
                 ignoring override and using auto-detection"
            );
            None
        }
    }
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

    // Env override beats auto-detection. Operators get the final say.
    #[cfg(feature = "simd")]
    if let Some(forced) = env_override() {
        return forced;
    }

    #[cfg(all(feature = "simd", target_arch = "x86_64"))]
    {
        // Highest-priority slots first. AVX-512 VNNI implies
        // AVX-512F so the VNNI check naturally subsumes both.
        #[cfg(feature = "simd-avx512")]
        {
            if std::is_x86_feature_detected!("avx512vnni")
                && std::is_x86_feature_detected!("avx512bw")
            {
                return &super::x86::avx512_vnni::Avx512VnniBackend;
            }
            if std::is_x86_feature_detected!("avx512f") {
                return &super::x86::avx512::Avx512Backend;
            }
        }

        #[cfg(feature = "simd-avx2")]
        {
            if std::is_x86_feature_detected!("avx2") {
                // `Avx2Backend` carries a `with_fma` flag picked at
                // construction; cache one instance for the dispatch
                // table so the flag is set exactly once.
                static AVX2: OnceLock<super::x86::avx2::Avx2Backend> = OnceLock::new();
                return AVX2.get_or_init(super::x86::avx2::Avx2Backend::auto_detect);
            }
        }

        // SSE2 is always available on x86_64 — no feature gate
        // beyond the arch check.
        return &super::x86::sse2::Sse2Backend;
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
        let a_addr = a as *const dyn SimdBackend as *const () as usize;
        let b_addr = b as *const dyn SimdBackend as *const () as usize;
        assert_eq!(a_addr, b_addr);
    }

    #[test]
    fn name_is_one_of_the_supported_set() {
        let n = selected_backend_name();
        let supported = [
            "avx512vnni",
            "avx512",
            "avx2+fma",
            "avx2",
            "sse2",
            "neon",
            "sve",
            "wasm128",
            "scalar",
        ];
        assert!(
            supported.contains(&n),
            "selected_backend_name() = {n:?}, expected one of {supported:?}"
        );
    }

    #[test]
    #[cfg(all(target_arch = "x86_64", feature = "simd-avx2"))]
    fn x86_picks_avx2_or_better_when_available() {
        // On a CI box with AVX2 (every modern x86_64 server qualifies),
        // we must land on AVX2 (or AVX-512 / VNNI if the CPU also
        // has those). Never plain scalar.
        if std::is_x86_feature_detected!("avx2") {
            let n = selected_backend_name();
            assert!(
                ["avx2", "avx2+fma", "avx512", "avx512vnni"].contains(&n),
                "expected an AVX-class backend on AVX2-capable CPU, got {n:?}"
            );
        }
    }
}
