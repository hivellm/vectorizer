//! GEMM-based Distance Computation for CUDA
//! 
//! This module implements optimized distance computation using GEMM operations
//! for better GPU utilization and performance.

use crate::error::{Result, VectorizerError};
use crate::models::DistanceMetric;
use tracing::{debug, info, warn};
use rayon::prelude::*;

/// GEMM-based distance computation using optimized matrix operations
pub struct GemmDistanceComputer {
    device_available: bool,
    device_id: usize,
}

impl GemmDistanceComputer {
    /// Create new GEMM distance computer
    pub fn new(device_id: usize) -> Result<Self> {
        debug!("Initializing GEMM distance computer on device {}", device_id);
        
        // For now, always use CPU simulation until CUDA is properly set up
        let device_available = false;
        
        info!("GEMM distance computer initialized successfully on device {} (CPU simulation with optimized BLAS-like operations)", device_id);
        
        Ok(Self {
            device_available,
            device_id,
        })
    }

    /// Compute cosine similarity using GEMM-like operations
    pub fn cosine_similarity_gemm(
        &self,
        queries: &[Vec<f32>],
        vectors: &[Vec<f32>],
    ) -> Result<Vec<Vec<f32>>> {
        debug!("Computing cosine similarity using GEMM for {} queries, {} vectors", queries.len(), vectors.len());
        
        if queries.is_empty() || vectors.is_empty() {
            return Ok(vec![]);
        }

        let dimension = queries[0].len();
        
        // Flatten matrices for GEMM-like operations
        let query_matrix: Vec<f32> = queries.iter().flat_map(|q| q.iter()).copied().collect();
        let vector_matrix: Vec<f32> = vectors.iter().flat_map(|v| v.iter()).copied().collect();
        
        // Compute dot products using optimized matrix multiplication
        let dot_products = self.compute_dot_products_optimized(&query_matrix, &vector_matrix, queries.len(), vectors.len(), dimension)?;
        
        // Compute norms for queries and vectors
        let query_norms = self.compute_norms_optimized(&query_matrix, queries.len(), dimension)?;
        let vector_norms = self.compute_norms_optimized(&vector_matrix, vectors.len(), dimension)?;
        
        // Compute cosine similarities
        let mut results = Vec::with_capacity(queries.len());
        for (i, query_norm) in query_norms.iter().enumerate() {
            let mut query_similarities = Vec::with_capacity(vectors.len());
            for (j, vector_norm) in vector_norms.iter().enumerate() {
                let dot_product = dot_products[i * vectors.len() + j];
                let similarity = if *query_norm > 0.0 && *vector_norm > 0.0 {
                    dot_product / (query_norm * vector_norm)
                } else {
                    0.0
                };
                query_similarities.push(similarity);
            }
            results.push(query_similarities);
        }
        
        debug!("GEMM cosine similarity completed for {} queries, {} vectors", queries.len(), vectors.len());
        Ok(results)
    }

    /// Compute Euclidean distance using GEMM-like operations
    pub fn euclidean_distance_gemm(
        &self,
        queries: &[Vec<f32>],
        vectors: &[Vec<f32>],
    ) -> Result<Vec<Vec<f32>>> {
        debug!("Computing Euclidean distance using GEMM for {} queries, {} vectors", queries.len(), vectors.len());
        
        if queries.is_empty() || vectors.is_empty() {
            return Ok(vec![]);
        }

        let dimension = queries[0].len();
        
        // Flatten matrices for GEMM-like operations
        let query_matrix: Vec<f32> = queries.iter().flat_map(|q| q.iter()).copied().collect();
        let vector_matrix: Vec<f32> = vectors.iter().flat_map(|v| v.iter()).copied().collect();
        
        // Compute squared norms for queries and vectors
        let query_norms_squared = self.compute_norms_squared_optimized(&query_matrix, queries.len(), dimension)?;
        let vector_norms_squared = self.compute_norms_squared_optimized(&vector_matrix, vectors.len(), dimension)?;
        
        // Compute dot products
        let dot_products = self.compute_dot_products_optimized(&query_matrix, &vector_matrix, queries.len(), vectors.len(), dimension)?;
        
        // Compute Euclidean distances using ||a-b||² = ||a||² + ||b||² - 2a·b
        let mut results = Vec::with_capacity(queries.len());
        for (i, query_norm_sq) in query_norms_squared.iter().enumerate() {
            let mut query_distances = Vec::with_capacity(vectors.len());
            for (j, vector_norm_sq) in vector_norms_squared.iter().enumerate() {
                let dot_product = dot_products[i * vectors.len() + j];
                let distance_squared = query_norm_sq + vector_norm_sq - 2.0 * dot_product;
                let distance = if distance_squared >= 0.0 {
                    distance_squared.sqrt()
                } else {
                    0.0 // Handle numerical precision issues
                };
                query_distances.push(distance);
            }
            results.push(query_distances);
        }
        
        debug!("GEMM Euclidean distance completed for {} queries, {} vectors", queries.len(), vectors.len());
        Ok(results)
    }

    /// Compute dot products using optimized operations
    fn compute_dot_products_optimized(
        &self,
        query_matrix: &[f32],
        vector_matrix: &[f32],
        query_count: usize,
        vector_count: usize,
        dimension: usize,
    ) -> Result<Vec<f32>> {
        let mut dot_products = vec![0.0f32; query_count * vector_count];
        
        // Use parallel processing for matrix multiplication
        dot_products.par_chunks_mut(vector_count)
            .enumerate()
            .for_each(|(query_idx, chunk)| {
                let query_start = query_idx * dimension;
                let query_end = query_start + dimension;
                let query = &query_matrix[query_start..query_end];
                
                for (vector_idx, dot_product) in chunk.iter_mut().enumerate() {
                    let vector_start = vector_idx * dimension;
                    let vector_end = vector_start + dimension;
                    let vector = &vector_matrix[vector_start..vector_end];
                    
                    // Compute dot product with SIMD-like optimization
                    *dot_product = query.iter()
                        .zip(vector.iter())
                        .map(|(a, b)| a * b)
                        .sum();
                }
            });
        
        Ok(dot_products)
    }

    /// Compute norms using optimized operations
    fn compute_norms_optimized(
        &self,
        matrix: &[f32],
        vector_count: usize,
        dimension: usize,
    ) -> Result<Vec<f32>> {
        let norms_squared = self.compute_norms_squared_optimized(matrix, vector_count, dimension)?;
        Ok(norms_squared.iter().map(|&norm_sq| norm_sq.sqrt()).collect())
    }

    /// Compute squared norms using optimized operations
    fn compute_norms_squared_optimized(
        &self,
        matrix: &[f32],
        vector_count: usize,
        dimension: usize,
    ) -> Result<Vec<f32>> {
        let mut norms_squared = vec![0.0f32; vector_count];
        
        // Use parallel processing for norm computation
        norms_squared.par_iter_mut()
            .enumerate()
            .for_each(|(vector_idx, norm_sq)| {
                let start = vector_idx * dimension;
                let end = start + dimension;
                let vector = &matrix[start..end];
                
                *norm_sq = vector.iter()
                    .map(|&x| x * x)
                    .sum();
            });
        
        Ok(norms_squared)
    }

    /// Get device information
    pub fn get_device_info(&self) -> String {
        if self.device_available {
            format!("GEMM Device {} (GPU BLAS)", self.device_id)
        } else {
            format!("GEMM Device {} (CPU BLAS-like)", self.device_id)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gemm_distance_computer_creation() {
        let computer = GemmDistanceComputer::new(0).unwrap();
        assert_eq!(computer.device_id, 0);
    }

    #[test]
    fn test_cosine_similarity_gemm() {
        let computer = GemmDistanceComputer::new(0).unwrap();
        let queries = vec![vec![1.0, 0.0, 0.0], vec![0.0, 1.0, 0.0]];
        let vectors = vec![vec![1.0, 0.0, 0.0], vec![0.0, 1.0, 0.0], vec![0.0, 0.0, 1.0]];
        
        let results = computer.cosine_similarity_gemm(&queries, &vectors).unwrap();
        assert_eq!(results.len(), 2);
        assert_eq!(results[0].len(), 3);
        assert_eq!(results[1].len(), 3);
        
        // First query should be most similar to first vector
        assert!((results[0][0] - 1.0).abs() < 1e-6);
        assert!((results[0][1] - 0.0).abs() < 1e-6);
        
        // Second query should be most similar to second vector
        assert!((results[1][1] - 1.0).abs() < 1e-6);
        assert!((results[1][0] - 0.0).abs() < 1e-6);
    }

    #[test]
    fn test_euclidean_distance_gemm() {
        let computer = GemmDistanceComputer::new(0).unwrap();
        let queries = vec![vec![0.0, 0.0, 0.0], vec![1.0, 1.0, 1.0]];
        let vectors = vec![vec![1.0, 0.0, 0.0], vec![0.0, 1.0, 0.0]];
        
        let results = computer.euclidean_distance_gemm(&queries, &vectors).unwrap();
        assert_eq!(results.len(), 2);
        assert_eq!(results[0].len(), 2);
        assert_eq!(results[1].len(), 2);
        
        // First query (origin) should have distance 1.0 to both vectors
        assert!((results[0][0] - 1.0).abs() < 1e-6);
        assert!((results[0][1] - 1.0).abs() < 1e-6);
    }
}
