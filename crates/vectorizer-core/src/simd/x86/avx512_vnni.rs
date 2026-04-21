//! AVX-512 VNNI backend — adds the single-instruction INT8 dot
//! product (`_mm512_dpbusd_epi32`) on top of the AVX-512F f32 path.
//!
//! VNNI (Vector Neural Network Instructions) ships on Cascade Lake,
//! Ice Lake, Tiger Lake, and every newer Intel + AMD platform. The
//! key primitive is `vpdpbusd`, which computes the four-way dot
//! product of u8×i8 byte pairs into i32 accumulators in one cycle —
//! roughly **4×** the f32 AVX-512 throughput on the same lane width
//! and ~16× over a scalar `i32` accumulation.
//!
//! For the f32 primitives this backend forwards to
//! [`super::avx512::Avx512Backend`] verbatim — there's no VNNI gain
//! on float math, only on byte/word integer math. Callers that only
//! exercise f32 stay on the AVX-512F path; the only reason to pick
//! this backend over plain AVX-512 is when the workload includes the
//! quantized INT8 distance kernel from phase 7f.

use std::arch::x86_64::*;

use crate::simd::backend::SimdBackend;
use crate::simd::x86::avx512::Avx512Backend;

/// Marker type for the AVX-512 VNNI backend.
pub struct Avx512VnniBackend;

impl SimdBackend for Avx512VnniBackend {
    fn dot_product(&self, a: &[f32], b: &[f32]) -> f32 {
        Avx512Backend.dot_product(a, b)
    }

    fn euclidean_distance_squared(&self, a: &[f32], b: &[f32]) -> f32 {
        Avx512Backend.euclidean_distance_squared(a, b)
    }

    fn cosine_similarity(&self, a: &[f32], b: &[f32]) -> f32 {
        Avx512Backend.cosine_similarity(a, b)
    }

    fn l2_norm(&self, a: &[f32]) -> f32 {
        Avx512Backend.l2_norm(a)
    }

    fn int8_dot_product(&self, a: &[i8], b: &[i8]) -> i32 {
        debug_assert_eq!(a.len(), b.len(), "Vectors must have same length");
        if std::is_x86_feature_detected!("avx512vnni") && std::is_x86_feature_detected!("avx512bw")
        {
            // SAFETY: VNNI + AVX-512BW verified by the runtime
            // detector immediately above; equal-length precondition
            // is debug-asserted on entry.
            unsafe { int8_dot_product_vnni(a, b) }
        } else {
            // Inherit the trait's default scalar fallback.
            crate::simd::scalar::ScalarBackend.int8_dot_product(a, b)
        }
    }

    fn name(&self) -> &'static str {
        "avx512vnni"
    }
}

/// # Safety
///
/// Caller must ensure both AVX-512VNNI and AVX-512BW are available
/// on the running CPU. `vpdpbusd` operates on a u8 × i8 pair (one
/// operand is unsigned, one signed); we feed the same bit pattern
/// in for both — the trick relies on `i8 as u8` being a bit-identity
/// reinterpretation, which Rust guarantees. Slices must have equal
/// length; the masked-load tail respects the actual remaining count.
#[target_feature(enable = "avx512f,avx512bw,avx512vnni")]
#[inline]
unsafe fn int8_dot_product_vnni(a: &[i8], b: &[i8]) -> i32 {
    const LANES: usize = 64; // 64 i8 lanes per __m512i

    let len = a.len();
    let simd_len = len - (len % LANES);

    // SAFETY: VNNI gated by `#[target_feature]`. `vpdpbusd` reads
    // 64 byte lanes per call and accumulates four-way dots into 16
    // i32 lanes. Loop bound `i + LANES <= simd_len <= len` keeps
    // every load inside the slice; the masked tail load uses a mask
    // derived from the actual remaining count.
    unsafe {
        let mut acc = _mm512_setzero_epi32();

        let mut i = 0;
        while i < simd_len {
            // Load 64 bytes from each operand. `vpdpbusd` wants a
            // u8 first operand and an i8 second; we load both as
            // raw bytes and let the bitwise reinterpret stand.
            let va = _mm512_loadu_si512(a.as_ptr().add(i) as *const _);
            let vb = _mm512_loadu_si512(b.as_ptr().add(i) as *const _);
            acc = _mm512_dpbusd_epi32(acc, va, vb);
            i += LANES;
        }

        // Tail: load the remaining bytes with a mask that enables
        // exactly `len - simd_len` lanes. `_mm512_maskz_loadu_epi8`
        // takes a 64-bit mask (one bit per byte lane).
        let tail = (len - simd_len) as u64;
        if tail > 0 {
            let mask = if tail >= 64 {
                u64::MAX
            } else {
                (1u64 << tail) - 1
            };
            let va = _mm512_maskz_loadu_epi8(mask, a.as_ptr().add(simd_len) as *const _);
            let vb = _mm512_maskz_loadu_epi8(mask, b.as_ptr().add(simd_len) as *const _);
            acc = _mm512_dpbusd_epi32(acc, va, vb);
        }

        _mm512_reduce_add_epi32(acc)
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;

    fn skip_unless_vnni() -> bool {
        if std::is_x86_feature_detected!("avx512vnni") {
            return false;
        }
        eprintln!("avx512vnni not available on this CPU; skipping VNNI backend test");
        true
    }

    #[test]
    fn int8_dot_product_aligned() {
        if skip_unless_vnni() {
            return;
        }
        // 64 elements = 1 SIMD chunk.
        let a: Vec<i8> = (0..64).map(|i| i as i8).collect();
        let b: Vec<i8> = (0..64).map(|i| (64 - i) as i8).collect();
        let got = Avx512VnniBackend.int8_dot_product(&a, &b);
        let want: i32 = a
            .iter()
            .zip(b.iter())
            .map(|(x, y)| (*x as i32) * (*y as i32))
            .sum();
        assert_eq!(got, want);
    }

    #[test]
    fn int8_dot_product_with_tail() {
        if skip_unless_vnni() {
            return;
        }
        // 70 elements = 1 SIMD chunk + 6 tail bytes.
        let a: Vec<i8> = (0..70).map(|i| (i % 100) as i8).collect();
        let b: Vec<i8> = (0..70).map(|i| ((50 - i) % 100) as i8).collect();
        let got = Avx512VnniBackend.int8_dot_product(&a, &b);
        let want: i32 = a
            .iter()
            .zip(b.iter())
            .map(|(x, y)| (*x as i32) * (*y as i32))
            .sum();
        assert_eq!(got, want);
    }

    #[test]
    fn int8_zero_input_returns_zero() {
        if skip_unless_vnni() {
            return;
        }
        let a = vec![0i8; 128];
        let b = vec![42i8; 128];
        assert_eq!(Avx512VnniBackend.int8_dot_product(&a, &b), 0);
    }

    #[test]
    fn name_is_avx512vnni() {
        assert_eq!(Avx512VnniBackend.name(), "avx512vnni");
    }
}
