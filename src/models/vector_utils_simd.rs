//! SIMD-accelerated vector utilities for high-performance distance calculations
//!
//! This module provides SIMD-optimized implementations using platform intrinsics.
//! Uses runtime CPU feature detection to enable SIMD when available.

#[cfg(target_arch = "x86_64")]
use std::arch::x86_64::*;

const SIMD_LANES: usize = 8;

/// Check if AVX2 is available at runtime
#[cfg(target_arch = "x86_64")]
#[inline]
fn is_avx2_available() -> bool {
    is_x86_feature_detected!("avx2")
}

#[cfg(not(target_arch = "x86_64"))]
#[inline]
fn is_avx2_available() -> bool {
    false
}

/// SIMD-accelerated dot product (AVX2 on x86_64, scalar fallback otherwise)
#[inline]
pub fn dot_product_simd(a: &[f32], b: &[f32]) -> f32 {
    debug_assert_eq!(a.len(), b.len(), "Vectors must have same length");

    #[cfg(target_arch = "x86_64")]
    {
        if is_avx2_available() {
            unsafe { dot_product_avx2(a, b) }
        } else {
            dot_product_scalar(a, b)
        }
    }

    #[cfg(not(target_arch = "x86_64"))]
    {
        dot_product_scalar(a, b)
    }
}

#[inline]
fn dot_product_scalar(a: &[f32], b: &[f32]) -> f32 {
    a.iter().zip(b.iter()).map(|(x, y)| x * y).sum()
}

#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "avx2")]
#[inline]
unsafe fn dot_product_avx2(a: &[f32], b: &[f32]) -> f32 {
    let len = a.len();
    let simd_len = len - (len % SIMD_LANES);

    let mut sum = _mm256_setzero_ps();

    // Process 8 floats at a time
    let mut i = 0;
    while i < simd_len {
        let va = _mm256_loadu_ps(a.as_ptr().add(i));
        let vb = _mm256_loadu_ps(b.as_ptr().add(i));
        let prod = _mm256_mul_ps(va, vb);
        sum = _mm256_add_ps(sum, prod);
        i += SIMD_LANES;
    }

    // Horizontal sum
    let mut result = horizontal_sum_avx2(sum);

    // Handle tail
    for idx in simd_len..len {
        result += a[idx] * b[idx];
    }

    result
}

/// SIMD-accelerated Euclidean distance
#[inline]
pub fn euclidean_distance_simd(a: &[f32], b: &[f32]) -> f32 {
    debug_assert_eq!(a.len(), b.len(), "Vectors must have same length");

    #[cfg(target_arch = "x86_64")]
    {
        if is_avx2_available() {
            unsafe { euclidean_distance_avx2(a, b) }
        } else {
            euclidean_distance_scalar(a, b)
        }
    }

    #[cfg(not(target_arch = "x86_64"))]
    {
        euclidean_distance_scalar(a, b)
    }
}

#[inline]
fn euclidean_distance_scalar(a: &[f32], b: &[f32]) -> f32 {
    a.iter()
        .zip(b.iter())
        .map(|(x, y)| (x - y) * (x - y))
        .sum::<f32>()
        .sqrt()
}

#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "avx2")]
#[inline]
unsafe fn euclidean_distance_avx2(a: &[f32], b: &[f32]) -> f32 {
    let len = a.len();
    let simd_len = len - (len % SIMD_LANES);

    let mut sum_sq = _mm256_setzero_ps();

    let mut i = 0;
    while i < simd_len {
        let va = _mm256_loadu_ps(a.as_ptr().add(i));
        let vb = _mm256_loadu_ps(b.as_ptr().add(i));
        let diff = _mm256_sub_ps(va, vb);
        let sq = _mm256_mul_ps(diff, diff);
        sum_sq = _mm256_add_ps(sum_sq, sq);
        i += SIMD_LANES;
    }

    let mut result = horizontal_sum_avx2(sum_sq);

    // Handle tail
    for idx in simd_len..len {
        let diff = a[idx] - b[idx];
        result += diff * diff;
    }

    result.sqrt()
}

/// SIMD-accelerated cosine similarity (assumes normalized vectors)
#[inline]
pub fn cosine_similarity_simd(a: &[f32], b: &[f32]) -> f32 {
    dot_product_simd(a, b).clamp(-1.0, 1.0)
}

#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "avx2")]
#[inline]
unsafe fn horizontal_sum_avx2(v: __m256) -> f32 {
    // Horizontal add within 256-bit vector
    let hi = _mm256_extractf128_ps(v, 1);
    let lo = _mm256_castps256_ps128(v);
    let sum128 = _mm_add_ps(hi, lo);

    // Horizontal add within 128-bit vector
    let shuf = _mm_movehdup_ps(sum128);
    let sums = _mm_add_ps(sum128, shuf);
    let shuf = _mm_movehl_ps(shuf, sums);
    let sums = _mm_add_ss(sums, shuf);

    _mm_cvtss_f32(sums)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_avx2_detection() {
        #[cfg(target_arch = "x86_64")]
        {
            tracing::info!("AVX2 available: {}", is_avx2_available());
        }
    }

    #[test]
    fn test_dot_product_simd() {
        let a = vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0];
        let b = vec![8.0, 7.0, 6.0, 5.0, 4.0, 3.0, 2.0, 1.0];

        let result = dot_product_simd(&a, &b);
        let expected: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();

        assert!((result - expected).abs() < 1e-5);
    }

    #[test]
    fn test_euclidean_distance_simd() {
        let a = vec![0.0, 0.0, 0.0, 0.0];
        let b = vec![3.0, 4.0, 0.0, 0.0];

        let result = euclidean_distance_simd(&a, &b);
        assert!((result - 5.0).abs() < 1e-5);
    }

    #[test]
    fn test_cosine_similarity_simd() {
        let a = vec![1.0, 0.0];
        let b = vec![1.0, 0.0];

        let result = cosine_similarity_simd(&a, &b);
        assert!((result - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_non_aligned_vectors() {
        let a = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let b = vec![5.0, 4.0, 3.0, 2.0, 1.0];

        let result = dot_product_simd(&a, &b);
        let expected: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();

        assert!((result - expected).abs() < 1e-5);
    }

    #[test]
    fn test_large_vectors() {
        let a: Vec<f32> = (0..1000).map(|i| i as f32 * 0.1).collect();
        let b: Vec<f32> = (0..1000).map(|i| i as f32 * 0.2).collect();

        let result_simd = dot_product_simd(&a, &b);
        let result_scalar: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();

        // Use relative error tolerance for large vectors to account for floating point accumulation
        let relative_error = if result_scalar.abs() > 1e-6 {
            (result_simd - result_scalar).abs() / result_scalar.abs()
        } else {
            (result_simd - result_scalar).abs()
        };
        assert!(
            relative_error < 1e-4,
            "Relative error: {} (simd: {}, scalar: {})",
            relative_error,
            result_simd,
            result_scalar
        );
    }
}
