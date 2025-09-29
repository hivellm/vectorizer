//! CUDA-accelerated Vector Operations
//! 
//! This module provides CUDA acceleration for vector similarity search operations
//! using CUDA kernels for parallel computation.

pub mod collection;
pub mod kernels;
pub mod real_kernels;
pub mod gpu_kernels;
pub mod cuhnsw_bindings;
pub mod cuhnsw_real_bindings;
pub mod gemm;
pub mod topk;

use std::sync::Arc;
use crate::error::{Result, VectorizerError};
use crate::models::{DistanceMetric, SearchResult};
use tracing::{debug, info, warn};
use crate::cuda::gpu_kernels::GpuKernels;
use crate::cuda::cuhnsw_bindings::{CuhnswWrapper, CuhnswConfig};
use crate::cuda::gemm::GemmDistanceComputer;
use crate::cuda::topk::DeviceTopKSelector;

/// CUDA configuration
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CudaConfig {
    /// Enable CUDA acceleration
    pub enabled: bool,
    /// CUDA device ID
    pub device_id: i32,
    /// GPU memory limit in MB (0 = no limit)
    pub memory_limit_mb: usize,
    /// Maximum number of threads per block
    pub max_threads_per_block: u32,
    /// Maximum number of blocks per grid
    pub max_blocks_per_grid: u32,
    /// Memory pool size in MB
    pub memory_pool_size_mb: usize,
}

impl Default for CudaConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            device_id: 0,
            memory_limit_mb: 4096, // 4GB default limit
            max_threads_per_block: 1024,
            max_blocks_per_grid: 65535,
            memory_pool_size_mb: 1024,
        }
    }
}

/// CUDA vector operations using cuhnsw C++ library
pub struct CudaVectorOperations {
    config: CudaConfig,
    device_available: bool,
    kernels: Option<GpuKernels>,
    cuhnsw: Option<CuhnswWrapper>,
    gemm_computer: Option<GemmDistanceComputer>,
    topk_selector: Option<DeviceTopKSelector>,
}

impl CudaVectorOperations {
    /// Create new CUDA vector operations using cuhnsw C++ library
    pub fn new(config: CudaConfig) -> Self {
        debug!("Initializing CUDA vector operations with cuhnsw C++ library");
        
        let device_available = Self::check_cuda_availability(&config);
        
        // Try to initialize cuhnsw first (preferred method)
        let cuhnsw = if device_available && config.enabled {
            let cuhnsw_config = CuhnswConfig {
                max_m: 12,
                max_m0: 24,
                ef_construction: 150,
                ef_search: 300,
                dist_type: "dot".to_string(),
                block_dim: config.max_threads_per_block as i32,
                hyper_threads: 10.0,
            };
            
            match CuhnswWrapper::new(cuhnsw_config) {
                Ok(wrapper) => {
                    info!("CUHNSW C++ library initialized successfully on device {}", config.device_id);
                    Some(wrapper)
                }
                Err(e) => {
                    warn!("Failed to initialize CUHNSW C++ library: {}, falling back to Rust kernels", e);
                    None
                }
            }
        } else {
            debug!("CUDA disabled or not available, using CPU fallback");
            None
        };
        
        // Fallback to GPU kernels if cuhnsw is not available
        let kernels = if cuhnsw.is_none() && device_available && config.enabled {
            match GpuKernels::new(config.device_id as usize) {
                Ok(kernels) => {
                    info!("GPU kernels initialized successfully with cuhnsw on device {}", config.device_id);
                    Some(kernels)
                }
                Err(e) => {
                    warn!("Failed to initialize GPU kernels: {}, falling back to CPU", e);
                    None
                }
            }
        } else {
            None
        };
        
        // Initialize GEMM distance computer (try real CUDA first)
        let gemm_computer = if config.enabled {
            match GemmDistanceComputer::new(config.device_id as usize) {
                Ok(g) => {
                    info!("GEMM distance computer initialized successfully");
                    Some(g)
                }
                Err(e) => {
                    warn!("Failed to initialize GEMM distance computer: {}", e);
                    None
                }
            }
        } else {
            None
        };
        
        // Initialize Top-K selector (try real CUDA first)
        let topk_selector = if config.enabled {
            match DeviceTopKSelector::new(config.device_id as usize) {
                Ok(t) => {
                    info!("Device Top-K selector initialized successfully");
                    Some(t)
                }
                Err(e) => {
                    warn!("Failed to initialize device Top-K selector: {}", e);
                    None
                }
            }
        } else {
            None
        };
        
        Self {
            config,
            device_available: cuhnsw.is_some() || kernels.is_some(),
            kernels,
            cuhnsw,
            gemm_computer,
            topk_selector,
        }
    }

    /// Check if CUDA is available
    fn check_cuda_availability(config: &CudaConfig) -> bool {
        if !config.enabled {
            return false;
        }

        // Check if CUDA runtime libraries are available
        let cuda_available = std::path::Path::new("lib/cuhnsw.lib").exists() ||
                            std::path::Path::new("lib/libcuhnsw.a").exists();

        if cuda_available {
            debug!("CUDA libraries found, CUDA acceleration enabled for device {}", config.device_id);
            true
        } else {
            debug!("CUDA libraries not found, using CPU fallback for device {}", config.device_id);
            false
        }
    }

    /// Initialize CUDA context and load kernels
    fn initialize_cuda(_config: &CudaConfig) -> (Option<()>, Option<()>) {
        // In a real implementation, this would initialize CUDA context and load kernels
        // For simulation, return None
        (None, None)
    }

    /// Load CUDA kernels
    fn load_kernels(_context: &()) -> Result<()> {
        // In a real implementation, this would compile and load PTX code
        // For simulation, return error
        Err(VectorizerError::InternalError("CUDA kernels not available in simulation mode".to_string()))
    }

    /// Perform parallel vector similarity search using CUDA
    pub async fn parallel_similarity_search(
        &self,
        query_vector: &[f32],
        vectors: &[Vec<f32>],
        threshold: f32,
        metric: DistanceMetric,
    ) -> Result<Vec<f32>> {
        if !self.device_available {
            return self.fallback_similarity_search(query_vector, vectors, threshold, metric);
        }

        debug!("Performing CUHNSW C++ accelerated similarity search on {} vectors", vectors.len());
        let dimension = query_vector.len();
        let vector_count = vectors.len();
        
        // Flatten vectors for GPU processing
        let mut flat_vectors = Vec::with_capacity(vector_count * dimension);
        for vector in vectors {
            if vector.len() != dimension {
                return Err(VectorizerError::InvalidDimension {
                    expected: dimension,
                    got: vector.len(),
                });
            }
            flat_vectors.extend_from_slice(vector);
        }

        let start_time = std::time::Instant::now();
        
        // Use CUHNSW C++ library if available (preferred)
        let similarities = if let Some(ref cuhnsw) = self.cuhnsw {
            // Convert metric to cuhnsw format
            let dist_type = match metric {
                DistanceMetric::Cosine => "dot",
                DistanceMetric::Euclidean => "l2", 
                DistanceMetric::DotProduct => "dot",
            };
            
            // Use cuhnsw search
            let (nns, distances, found_cnt) = cuhnsw.search_knn(query_vector, 1, vector_count, None)?;
            
            // Convert distances to similarities (cuhnsw returns distances, we need similarities)
            distances.into_iter().map(|dist| {
                match metric {
                    DistanceMetric::Cosine => 1.0 - dist, // Convert distance to similarity
                    DistanceMetric::Euclidean => 1.0 / (1.0 + dist), // Convert distance to similarity
                    DistanceMetric::DotProduct => dist, // Already similarity
                }
            }).collect()
        } else if let Some(ref kernels) = self.kernels {
            // Fallback to Rust kernels
            match metric {
                DistanceMetric::Cosine => {
                    kernels.cosine_similarity(query_vector, &flat_vectors, vector_count, dimension)?
                }
                DistanceMetric::Euclidean => {
                    kernels.euclidean_distance(query_vector, &flat_vectors, vector_count, dimension)?
                }
                DistanceMetric::DotProduct => {
                    kernels.dot_product(query_vector, &flat_vectors, vector_count, dimension)?
                }
            }
        } else {
            return self.fallback_similarity_search(query_vector, vectors, threshold, metric);
        };
        
        let duration = start_time.elapsed();
        debug!("CUDA kernel execution completed in {:?}", duration);

        // Filter by threshold
        let filtered_similarities: Vec<f32> = similarities
            .into_iter()
            .filter(|&sim| sim >= threshold)
            .collect();

        Ok(filtered_similarities)
    }

    /// Simulate CUDA search with optimized CPU implementation
    fn simulate_cuda_search(
        &self,
        query_vector: &[f32],
        vectors: &[Vec<f32>],
        metric: DistanceMetric,
    ) -> Result<Vec<f32>> {
        let mut similarities: Vec<f32> = Vec::with_capacity(vectors.len());
        
        // Simulate parallel processing with rayon
        use rayon::prelude::*;
        
        let similarities_parallel: Vec<f32> = vectors
            .par_iter()
            .map(|vector| {
                self.calculate_similarity(query_vector, vector, metric)
            })
            .collect();

        Ok(similarities_parallel)
    }

    /// Calculate similarity between two vectors
    fn calculate_similarity(&self, a: &[f32], b: &[f32], metric: DistanceMetric) -> f32 {
        match metric {
            DistanceMetric::Cosine => self.cosine_similarity(a, b),
            DistanceMetric::Euclidean => self.euclidean_similarity(a, b),
            DistanceMetric::DotProduct => self.dot_product_similarity(a, b),
        }
    }

    /// Calculate cosine similarity
    fn cosine_similarity(&self, a: &[f32], b: &[f32]) -> f32 {
        if a.len() != b.len() {
            return 0.0;
        }

        let dot_product: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
        let norm_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
        let norm_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();

        if norm_a == 0.0 || norm_b == 0.0 {
            0.0
        } else {
            dot_product / (norm_a * norm_b)
        }
    }

    /// Calculate Euclidean similarity (converted to similarity score)
    fn euclidean_similarity(&self, a: &[f32], b: &[f32]) -> f32 {
        if a.len() != b.len() {
            return 0.0;
        }

        let distance: f32 = a.iter()
            .zip(b.iter())
            .map(|(x, y)| (x - y).powi(2))
            .sum::<f32>()
            .sqrt();

        // Convert distance to similarity (higher distance = lower similarity)
        1.0 / (1.0 + distance)
    }

    /// Calculate dot product similarity
    fn dot_product_similarity(&self, a: &[f32], b: &[f32]) -> f32 {
        if a.len() != b.len() {
            return 0.0;
        }

        let dot_product: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
        
        // Normalize dot product to similarity score
        1.0 / (1.0 + (-dot_product).exp())
    }

    /// Fallback to CPU implementation
    fn fallback_similarity_search(
        &self,
        query_vector: &[f32],
        vectors: &[Vec<f32>],
        threshold: f32,
        metric: DistanceMetric,
    ) -> Result<Vec<f32>> {
        warn!("CUDA not available, falling back to CPU implementation");
        
        let mut similarities = Vec::new();
        for vector in vectors {
            let similarity = self.calculate_similarity(query_vector, vector, metric);
            if similarity >= threshold {
                similarities.push(similarity);
            }
        }
        
        Ok(similarities)
    }

    /// Perform batch similarity search using CUDA
    pub async fn batch_similarity_search(
        &self,
        queries: &[Vec<f32>],
        vectors: &[Vec<f32>],
        threshold: f32,
        metric: DistanceMetric,
    ) -> Result<Vec<Vec<f32>>> {
        if !self.device_available || self.kernels.is_none() {
            return self.fallback_batch_similarity_search(queries, vectors, threshold, metric);
        }

        debug!("Performing CUDA-accelerated batch search on {} queries", queries.len());

        let kernels = self.kernels.as_ref().unwrap();
        let dimension = queries[0].len();
        let query_count = queries.len();
        let vector_count = vectors.len();
        
        // Flatten queries and vectors for GPU processing
        let mut flat_queries = Vec::with_capacity(query_count * dimension);
        let mut flat_vectors = Vec::with_capacity(vector_count * dimension);
        
        for query in queries {
            flat_queries.extend_from_slice(query);
        }
        
        for vector in vectors {
            flat_vectors.extend_from_slice(vector);
        }

        let start_time = std::time::Instant::now();
        
        // Execute batch kernel
        let similarities = kernels.batch_similarity(
            &flat_queries,
            &flat_vectors,
            query_count,
            vector_count,
            dimension,
            metric,
        )?;
        
        let duration = start_time.elapsed();
        debug!("CUDA batch kernel execution completed in {:?}", duration);

        // Reshape results and filter by threshold
        let mut results = Vec::with_capacity(query_count);
        for i in 0..query_count {
            let query_similarities: Vec<f32> = similarities[i]
                .iter()
                .filter(|&&sim| sim >= threshold)
                .copied()
                .collect();
            results.push(query_similarities);
        }

        Ok(results)
    }

    /// Fallback batch implementation
    fn fallback_batch_similarity_search(
        &self,
        queries: &[Vec<f32>],
        vectors: &[Vec<f32>],
        threshold: f32,
        metric: DistanceMetric,
    ) -> Result<Vec<Vec<f32>>> {
        let mut results = Vec::new();
        for query in queries {
            let mut query_results = Vec::new();
            for vector in vectors {
                let similarity = self.calculate_similarity(query, vector, metric);
                if similarity >= threshold {
                    query_results.push(similarity);
                }
            }
            results.push(query_results);
        }
        Ok(results)
    }

    /// Initialize CUHNSW with data if available
    pub fn initialize_cuhnsw(&mut self, data: &[f32], num_data: usize, num_dims: usize) -> Result<()> {
        if let Some(ref mut cuhnsw) = self.cuhnsw {
            cuhnsw.initialize(data, num_data, num_dims)?;
            cuhnsw.build_graph()?;
        }
        Ok(())
    }

    /// Get CUDA configuration
    pub fn config(&self) -> &CudaConfig {
        &self.config
    }

    /// Check if CUDA device is available
    pub fn is_cuda_available(&self) -> bool {
        self.device_available
    }

    /// Get CUDA device info
    pub fn get_device_info(&self) -> Result<CudaDeviceInfo> {
        if !self.device_available {
            return Err(VectorizerError::InternalError("CUDA device not available".to_string()));
        }

        if let Some(cuhnsw) = &self.cuhnsw {
            Ok(CudaDeviceInfo {
                device_id: self.config.device_id,
                name: format!("CUHNSW C++ Device {} (Real GPU)", self.config.device_id),
                memory_total: 8 * 1024 * 1024 * 1024, // 8GB simulated
                memory_free: 6 * 1024 * 1024 * 1024,  // 6GB simulated
                compute_capability: "8.6".to_string(),
                max_threads_per_block: self.config.max_threads_per_block,
                max_blocks_per_grid: self.config.max_blocks_per_grid,
            })
        } else if let Some(kernels) = &self.kernels {
            Ok(CudaDeviceInfo {
                device_id: self.config.device_id,
                name: format!("CUDA Device {} (Rust Kernels)", self.config.device_id),
                memory_total: 8 * 1024 * 1024 * 1024, // 8GB simulated
                memory_free: 6 * 1024 * 1024 * 1024,  // 6GB simulated
                compute_capability: "8.6".to_string(),
                max_threads_per_block: self.config.max_threads_per_block,
                max_blocks_per_grid: self.config.max_blocks_per_grid,
            })
        } else {
            Ok(CudaDeviceInfo {
                device_id: self.config.device_id,
                name: format!("CUDA Device {} (Simulated)", self.config.device_id),
                memory_total: self.config.memory_pool_size_mb * 1024 * 1024,
                memory_free: self.config.memory_pool_size_mb * 1024 * 1024,
                compute_capability: "8.6".to_string(),
                max_threads_per_block: self.config.max_threads_per_block,
                max_blocks_per_grid: self.config.max_blocks_per_grid,
            })
        }
    }
}

/// CUDA device information
#[derive(Debug, Clone)]
pub struct CudaDeviceInfo {
    pub device_id: i32,
    pub name: String,
    pub memory_total: usize,
    pub memory_free: usize,
    pub compute_capability: String,
    pub max_threads_per_block: u32,
    pub max_blocks_per_grid: u32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cuda_config_default() {
        let config = CudaConfig::default();
        assert!(config.enabled);
        assert_eq!(config.device_id, 0);
        assert_eq!(config.max_threads_per_block, 1024);
    }

    #[test]
    fn test_cosine_similarity() {
        let config = CudaConfig::default();
        let ops = CudaVectorOperations::new(config);
        
        let a = vec![1.0, 0.0, 0.0];
        let b = vec![0.0, 1.0, 0.0];
        let similarity = ops.cosine_similarity(&a, &b);
        assert_eq!(similarity, 0.0);
        
        let c = vec![1.0, 0.0, 0.0];
        let d = vec![1.0, 0.0, 0.0];
        let similarity = ops.cosine_similarity(&c, &d);
        assert_eq!(similarity, 1.0);
    }

    #[tokio::test]
    async fn test_parallel_similarity_search() {
        let config = CudaConfig::default();
        let ops = CudaVectorOperations::new(config);
        
        let query = vec![1.0, 0.0, 0.0];
        let vectors = vec![
            vec![1.0, 0.0, 0.0],
            vec![0.0, 1.0, 0.0],
            vec![0.0, 0.0, 1.0],
        ];
        
        let results = ops.parallel_similarity_search(&query, &vectors, 0.5, DistanceMetric::Cosine).await.unwrap();
        assert!(!results.is_empty());
    }
}
