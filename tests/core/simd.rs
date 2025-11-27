//! Tests for SIMD-accelerated vector operations

use vectorizer::models::{vector_utils, vector_utils_simd};

#[test]
fn test_simd_dot_product() {
    // Test with aligned vectors
    let a = vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0];
    let b = vec![8.0, 7.0, 6.0, 5.0, 4.0, 3.0, 2.0, 1.0];

    let result = vector_utils_simd::dot_product_simd(&a, &b);
    let expected: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();

    assert!(
        (result - expected).abs() < 1e-5,
        "Dot product mismatch: {result} vs {expected}"
    );
}

#[test]
fn test_simd_dot_product_large() {
    // Test with large vectors (384 dimensions - common embedding size)
    let a: Vec<f32> = (0..384).map(|i| (i % 10) as f32).collect();
    let b: Vec<f32> = (0..384).map(|i| ((i + 5) % 10) as f32).collect();

    let result = vector_utils_simd::dot_product_simd(&a, &b);
    let expected: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();

    assert!(
        (result - expected).abs() < 1e-4,
        "Large dot product mismatch"
    );
}

#[test]
fn test_simd_euclidean_distance() {
    let a = vec![0.0, 0.0, 0.0, 0.0];
    let b = vec![3.0, 4.0, 0.0, 0.0];

    let result = vector_utils_simd::euclidean_distance_simd(&a, &b);
    let expected = 5.0; // sqrt(3^2 + 4^2) = 5

    assert!(
        (result - expected).abs() < 1e-5,
        "Euclidean distance mismatch"
    );
}

#[test]
fn test_simd_euclidean_distance_large() {
    // Test with 384-dimensional vectors
    let a: Vec<f32> = vec![1.0; 384];
    let b: Vec<f32> = vec![2.0; 384];

    let result = vector_utils_simd::euclidean_distance_simd(&a, &b);
    let expected = (384.0_f32).sqrt(); // sqrt(sum of (1-2)^2) = sqrt(384)

    assert!(
        (result - expected).abs() < 1e-4,
        "Large Euclidean distance mismatch"
    );
}

#[test]
fn test_simd_cosine_similarity() {
    // Test with normalized vectors
    let a = vec![1.0, 0.0];
    let b = vec![1.0, 0.0];

    let result = vector_utils_simd::cosine_similarity_simd(&a, &b);
    assert!(
        (result - 1.0).abs() < 1e-5,
        "Cosine similarity should be 1.0 for identical vectors"
    );
}

#[test]
fn test_simd_cosine_similarity_orthogonal() {
    // Test with orthogonal vectors
    let a = vec![1.0, 0.0];
    let b = vec![0.0, 1.0];

    let result = vector_utils_simd::cosine_similarity_simd(&a, &b);
    assert!(
        (result - 0.0).abs() < 1e-5,
        "Cosine similarity should be 0.0 for orthogonal vectors"
    );
}

#[test]
fn test_simd_odd_length_vectors() {
    // Test with vectors that don't align to SIMD lanes
    let a = vec![1.0, 2.0, 3.0, 4.0, 5.0];
    let b = vec![5.0, 4.0, 3.0, 2.0, 1.0];

    let result = vector_utils_simd::dot_product_simd(&a, &b);
    let expected: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();

    assert!(
        (result - expected).abs() < 1e-5,
        "Odd-length dot product mismatch"
    );
}

#[test]
fn test_vector_utils_wrapper() {
    // Test that the vector_utils module correctly uses SIMD
    let a = vec![1.0, 2.0, 3.0, 4.0];
    let b = vec![4.0, 3.0, 2.0, 1.0];

    let dot = vector_utils::dot_product(&a, &b);
    let expected: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();

    assert!(
        (dot - expected).abs() < 1e-5,
        "Wrapper dot product mismatch"
    );

    let euclidean = vector_utils::euclidean_distance(&a, &b);
    let expected_dist: f32 = a
        .iter()
        .zip(b.iter())
        .map(|(x, y)| (x - y) * (x - y))
        .sum::<f32>()
        .sqrt();

    assert!(
        (euclidean - expected_dist).abs() < 1e-5,
        "Wrapper Euclidean distance mismatch"
    );
}

#[test]
fn test_simd_performance_consistency() {
    // Test that SIMD produces consistent results across multiple calls
    let a: Vec<f32> = (0..128).map(|i| (i % 20) as f32).collect();
    let b: Vec<f32> = (0..128).map(|i| ((i + 10) % 20) as f32).collect();

    let result1 = vector_utils_simd::dot_product_simd(&a, &b);
    let result2 = vector_utils_simd::dot_product_simd(&a, &b);
    let result3 = vector_utils_simd::dot_product_simd(&a, &b);

    assert!(
        (result1 - result2).abs() < 1e-6,
        "Results should be consistent"
    );
    assert!(
        (result2 - result3).abs() < 1e-6,
        "Results should be consistent"
    );
}

#[test]
fn test_simd_zero_vectors() {
    // Test edge case: zero vectors
    let a = vec![0.0; 128];
    let b = vec![0.0; 128];

    let dot = vector_utils_simd::dot_product_simd(&a, &b);
    assert!(
        (dot - 0.0).abs() < 1e-6,
        "Dot product of zero vectors should be 0"
    );

    let euclidean = vector_utils_simd::euclidean_distance_simd(&a, &b);
    assert!(
        (euclidean - 0.0).abs() < 1e-6,
        "Euclidean distance of identical vectors should be 0"
    );
}

#[test]
fn test_simd_negative_values() {
    // Test with negative values
    let a = vec![-1.0, -2.0, -3.0, -4.0];
    let b = vec![1.0, 2.0, 3.0, 4.0];

    let result = vector_utils_simd::dot_product_simd(&a, &b);
    let expected: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();

    assert!(
        (result - expected).abs() < 1e-5,
        "Negative values dot product mismatch"
    );
}
