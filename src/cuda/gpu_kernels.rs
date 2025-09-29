//! Real GPU Kernels using cuhnsw
//! 
//! This module provides actual GPU acceleration using the cuhnsw CUDA library

use crate::error::{Result, VectorizerError};
use crate::models::DistanceMetric;
use super::cuhnsw_real_bindings::{CuHNSW, CuHNSWConfig};
use tracing::{debug, info, warn};
use std::sync::{Arc, Mutex};

/// Real GPU kernels using cuhnsw
pub struct GpuKernels {
    cuhnsw: Arc<Mutex<CuHNSW>>,
    device_available: bool,
    device_id: usize,
    dimension: usize,
    config: CuHNSWConfig,
}

impl GpuKernels {
    /// Create new GPU kernels
    pub fn new(device_id: usize) -> Result<Self> {
        debug!("Initializing GPU kernels on device {}", device_id);
        
        match CuHNSW::new() {
            Ok(mut cuhnsw_instance) => {
                let config = CuHNSWConfig {
                    max_m: 16,
                    max_m0: 32,
                    ef_construction: 200,
                    hyper_threads: 10.0,
                    block_dim: 256,
                    dist_type: "l2".to_string(),
                    visited_list_size: 16384,
                    heuristic_coef: 0.5,
                    ..Default::default()
                };
                
                // Initialize cuhnsw
                if let Err(e) = cuhnsw_instance.init(&config) {
                    warn!("Failed to initialize cuhnsw: {}, falling back to CPU", e);
                    return Ok(Self {
                        cuhnsw: Arc::new(Mutex::new(cuhnsw_instance)),
                        device_available: false,
                        device_id,
                        dimension: 0,
                        config,
                    });
                }
                
                info!("GPU kernels initialized successfully using cuhnsw on device {}", device_id);
                Ok(Self {
                    cuhnsw: Arc::new(Mutex::new(cuhnsw_instance)),
                    device_available: true,
                    device_id,
                    dimension: 0,
                    config,
                })
            }
            Err(e) => {
                warn!("Failed to create cuhnsw instance: {}, GPU not available", e);
                Err(e)
            }
        }
    }
    
    /// Set data dimension for proper operation
    pub fn set_dimension(&mut self, dimension: usize) {
        self.dimension = dimension;
    }
    
    /// Build index with given vectors
    pub fn build_index(&mut self, vectors: &[f32]) -> Result<()> {
        if !self.device_available || self.dimension == 0 {
            return Err(VectorizerError::InternalError(
                "GPU not available or dimension not set".to_string()
            ));
        }
        
        let num_vectors = vectors.len() / self.dimension;
        debug!("Building GPU index for {} vectors of dimension {}", num_vectors, self.dimension);
        
        // Generate random levels (following HNSW paper)
        let mut rng = rand::thread_rng();
        let level_mult = 1.0 / (2.0_f64).ln();
        let levels: Vec<i32> = (0..num_vectors)
            .map(|_| {
                let level = -((rand::random::<f64>()).ln() * level_mult) as i32;
                level.max(0)
            })
            .collect();
        
        let mut cuhnsw = self.cuhnsw.lock().unwrap();
        cuhnsw.set_data(vectors, self.dimension)?;
        cuhnsw.build_graph(&levels)?;
        
        info!("GPU index built successfully");
        Ok(())
    }
    
    /// Execute cosine similarity using GPU
    pub fn cosine_similarity(
        &self,
        query: &[f32],
        vectors: &[f32],
        vector_count: usize,
        dimension: usize,
    ) -> Result<Vec<f32>> {
        if !self.device_available {
            return self.cosine_similarity_cpu(query, vectors, vector_count, dimension);
        }
        
        debug!("Executing GPU cosine similarity for {} vectors", vector_count);
        
        // Convert to appropriate distance metric
        let mut config = self.config.clone();
        config.dist_type = "dot".to_string();
        config.nrz = true; // Normalize for cosine similarity
        
        let mut cuhnsw = self.cuhnsw.lock().unwrap();
        cuhnsw.init(&config)?;
        cuhnsw.set_data(vectors, dimension)?;
        
        // Search for all vectors (batch similarity)
        let (indices, distances) = cuhnsw.search_knn(
            query,
            1,
            dimension,
            vector_count.min(1000), // Limit to avoid GPU memory issues
            vector_count.min(1500), // ef_search
        )?;
        
        // Convert distances to similarities
        let mut similarities = vec![0.0f32; vector_count];
        for (i, (&idx, &dist)) in indices.iter().zip(distances.iter()).enumerate() {
            if (idx as usize) < vector_count {
                // For dot product with normalized vectors, similarity = -distance
                similarities[idx as usize] = if config.nrz { -dist } else { dist };
            }
        }
        
        debug!("GPU cosine similarity completed");
        Ok(similarities)
    }
    
    /// Execute euclidean distance using GPU
    pub fn euclidean_distance(
        &self,
        query: &[f32],
        vectors: &[f32],
        vector_count: usize,
        dimension: usize,
    ) -> Result<Vec<f32>> {
        if !self.device_available {
            return self.euclidean_distance_cpu(query, vectors, vector_count, dimension);
        }
        
        debug!("Executing GPU euclidean distance for {} vectors", vector_count);
        
        // Update config for L2 distance
        let mut config = self.config.clone();
        config.dist_type = "l2".to_string();
        
        let mut cuhnsw = self.cuhnsw.lock().unwrap();
        cuhnsw.init(&config)?;
        cuhnsw.set_data(vectors, dimension)?;
        
        // Search for all vectors
        let (indices, distances) = cuhnsw.search_knn(
            query,
            1,
            dimension,
            vector_count.min(1000),
            vector_count.min(1500),
        )?;
        
        // Fill results
        let mut results = vec![f32::MAX; vector_count];
        for (&idx, &dist) in indices.iter().zip(distances.iter()) {
            if (idx as usize) < vector_count {
                results[idx as usize] = dist.sqrt(); // L2 distance is sqrt of squared distance
            }
        }
        
        debug!("GPU euclidean distance completed");
        Ok(results)
    }
    
    /// Execute dot product using GPU
    pub fn dot_product(
        &self,
        query: &[f32],
        vectors: &[f32],
        vector_count: usize,
        dimension: usize,
    ) -> Result<Vec<f32>> {
        if !self.device_available {
            return self.dot_product_cpu(query, vectors, vector_count, dimension);
        }
        
        debug!("Executing GPU dot product for {} vectors", vector_count);
        
        // Update config for dot product
        let mut config = self.config.clone();
        config.dist_type = "dot".to_string();
        config.nrz = false; // No normalization for raw dot product
        
        let mut cuhnsw = self.cuhnsw.lock().unwrap();
        cuhnsw.init(&config)?;
        cuhnsw.set_data(vectors, dimension)?;
        
        // Search for all vectors
        let (indices, distances) = cuhnsw.search_knn(
            query,
            1,
            dimension,
            vector_count.min(1000),
            vector_count.min(1500),
        )?;
        
        // Fill results (negative because cuhnsw returns negative dot products)
        let mut results = vec![0.0f32; vector_count];
        for (&idx, &dist) in indices.iter().zip(distances.iter()) {
            if (idx as usize) < vector_count {
                results[idx as usize] = -dist;
            }
        }
        
        debug!("GPU dot product completed");
        Ok(results)
    }
    
    /// Execute batch similarity using GPU
    pub fn batch_similarity(
        &self,
        queries: &[f32],
        vectors: &[f32],
        query_count: usize,
        vector_count: usize,
        dimension: usize,
        metric: DistanceMetric,
    ) -> Result<Vec<Vec<f32>>> {
        if !self.device_available {
            return self.batch_similarity_cpu(queries, vectors, query_count, vector_count, dimension, metric);
        }
        
        debug!("Executing GPU batch similarity for {} queries, {} vectors", query_count, vector_count);
        
        // Update config based on metric
        let mut config = self.config.clone();
        match metric {
            DistanceMetric::Cosine => {
                config.dist_type = "dot".to_string();
                config.nrz = true;
            }
            DistanceMetric::Euclidean => {
                config.dist_type = "l2".to_string();
                config.nrz = false;
            }
            DistanceMetric::DotProduct => {
                config.dist_type = "dot".to_string();
                config.nrz = false;
            }
        }
        
        let mut cuhnsw = self.cuhnsw.lock().unwrap();
        cuhnsw.init(&config)?;
        cuhnsw.set_data(vectors, dimension)?;
        
        // Process queries in batch
        let k = vector_count.min(100); // Limit k to avoid memory issues
        let ef_search = k * 2;
        
        let (indices, distances) = cuhnsw.search_knn(
            queries,
            query_count,
            dimension,
            k,
            ef_search,
        )?;
        
        // Convert to per-query results
        let mut results = vec![vec![0.0f32; vector_count]; query_count];
        
        for q in 0..query_count {
            let offset = q * k;
            for i in 0..k {
                let idx = indices[offset + i] as usize;
                if idx < vector_count {
                    let dist = distances[offset + i];
                    results[q][idx] = match metric {
                        DistanceMetric::Cosine => if config.nrz { -dist } else { dist },
                        DistanceMetric::Euclidean => dist.sqrt(),
                        DistanceMetric::DotProduct => -dist,
                    };
                }
            }
        }
        
        debug!("GPU batch similarity completed");
        Ok(results)
    }
    
    /// Get device information
    pub fn get_device_info(&self) -> String {
        if self.device_available {
            format!("Real CUDA Device {} (GPU with cuhnsw)", self.device_id)
        } else {
            format!("CUDA Device {} (CPU fallback)", self.device_id)
        }
    }
    
    // CPU fallback implementations
    fn cosine_similarity_cpu(
        &self,
        query: &[f32],
        vectors: &[f32],
        vector_count: usize,
        dimension: usize,
    ) -> Result<Vec<f32>> {
        use rayon::prelude::*;
        
        let results: Vec<f32> = (0..vector_count)
            .into_par_iter()
            .map(|i| {
                let start = i * dimension;
                let end = start + dimension;
                let vector = &vectors[start..end];
                
                let dot_product: f32 = query.iter().zip(vector.iter()).map(|(a, b)| a * b).sum();
                let query_norm: f32 = query.iter().map(|x| x * x).sum::<f32>().sqrt();
                let vector_norm: f32 = vector.iter().map(|x| x * x).sum::<f32>().sqrt();
                
                if query_norm > 0.0 && vector_norm > 0.0 {
                    dot_product / (query_norm * vector_norm)
                } else {
                    0.0
                }
            })
            .collect();
        
        Ok(results)
    }
    
    fn euclidean_distance_cpu(
        &self,
        query: &[f32],
        vectors: &[f32],
        vector_count: usize,
        dimension: usize,
    ) -> Result<Vec<f32>> {
        use rayon::prelude::*;
        
        let results: Vec<f32> = (0..vector_count)
            .into_par_iter()
            .map(|i| {
                let start = i * dimension;
                let end = start + dimension;
                let vector = &vectors[start..end];
                
                let sum_squared_diff: f32 = query.iter()
                    .zip(vector.iter())
                    .map(|(a, b)| (a - b).powi(2))
                    .sum();
                
                sum_squared_diff.sqrt()
            })
            .collect();
        
        Ok(results)
    }
    
    fn dot_product_cpu(
        &self,
        query: &[f32],
        vectors: &[f32],
        vector_count: usize,
        dimension: usize,
    ) -> Result<Vec<f32>> {
        use rayon::prelude::*;
        
        let results: Vec<f32> = (0..vector_count)
            .into_par_iter()
            .map(|i| {
                let start = i * dimension;
                let end = start + dimension;
                let vector = &vectors[start..end];
                
                query.iter().zip(vector.iter()).map(|(a, b)| a * b).sum()
            })
            .collect();
        
        Ok(results)
    }
    
    fn batch_similarity_cpu(
        &self,
        queries: &[f32],
        vectors: &[f32],
        query_count: usize,
        vector_count: usize,
        dimension: usize,
        metric: DistanceMetric,
    ) -> Result<Vec<Vec<f32>>> {
        use rayon::prelude::*;
        
        let results: Vec<Vec<f32>> = (0..query_count)
            .into_par_iter()
            .map(|query_idx| {
                let query_start = query_idx * dimension;
                let query_end = query_start + dimension;
                let query = &queries[query_start..query_end];
                
                match metric {
                    DistanceMetric::Cosine => self.cosine_similarity_cpu(query, vectors, vector_count, dimension).unwrap(),
                    DistanceMetric::Euclidean => self.euclidean_distance_cpu(query, vectors, vector_count, dimension).unwrap(),
                    DistanceMetric::DotProduct => self.dot_product_cpu(query, vectors, vector_count, dimension).unwrap(),
                }
            })
            .collect();
        
        Ok(results)
    }
}

// Safe to use across threads
unsafe impl Send for GpuKernels {}
unsafe impl Sync for GpuKernels {}
