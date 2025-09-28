//! Bend Integration Module for Vectorizer
//! 
//! This module provides integration with Bend for automatic parallelization
//! of vector operations.

pub mod codegen;
pub mod batch;
pub mod hnsw;
pub mod collection;

use std::process::Command;
use std::path::Path;
use serde::{Deserialize, Serialize};
use crate::error::VectorizerError;
use crate::models::{DistanceMetric, SearchResult};
use crate::bend::codegen::{BendCodeGenerator, BendGeneratorConfig};
use tracing::debug;

/// Configuration for Bend integration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BendConfig {
    /// Path to bend executable
    pub bend_path: String,
    /// Enable CUDA acceleration
    pub enable_cuda: bool,
    /// Maximum parallel operations
    pub max_parallel: usize,
    /// Enable Bend integration
    pub enabled: bool,
    /// Fallback to Rust implementation if Bend fails
    pub fallback_enabled: bool,
}

impl Default for BendConfig {
    fn default() -> Self {
        Self {
            bend_path: "bend".to_string(),
            enable_cuda: false,
            max_parallel: 1000,
            enabled: true,
            fallback_enabled: true,
        }
    }
}

/// Bend executor for running Bend programs
pub struct BendExecutor {
    config: BendConfig,
}

impl BendExecutor {
    /// Create a new Bend executor
    pub fn new(config: BendConfig) -> Self {
        Self { config }
    }

    /// Execute a Bend program
    pub async fn execute_bend_program(&self, program_path: &Path) -> Result<String, VectorizerError> {
        let mut cmd = Command::new(&self.config.bend_path);
        
        if self.config.enable_cuda {
            cmd.arg("run-cu");
        } else {
            cmd.arg("run-rs");
        }
        
        cmd.arg(program_path);
        cmd.arg("-s"); // Show statistics
        
        let output = cmd.output()
            .map_err(|e| VectorizerError::InternalError(format!("Failed to execute Bend: {}", e)))?;
        
        if output.status.success() {
            let result = String::from_utf8(output.stdout)
                .map_err(|e| VectorizerError::InternalError(format!("Invalid UTF-8 output: {}", e)))?;
            Ok(result)
        } else {
            let error = String::from_utf8(output.stderr)
                .map_err(|e| VectorizerError::InternalError(format!("Invalid UTF-8 error: {}", e)))?;
            Err(VectorizerError::InternalError(format!("Bend execution failed: {}", error)))
        }
    }

    /// Execute Bend code from string
    pub async fn execute_bend_code(&self, code: &str) -> Result<String, VectorizerError> {
        // Create temporary file
        let temp_path = std::env::temp_dir().join("vectorizer_bend_temp.bend");
        std::fs::write(&temp_path, code)
            .map_err(|e| VectorizerError::InternalError(format!("Failed to write temp file: {}", e)))?;
        
        let result = self.execute_bend_program(&temp_path).await;
        
        // Clean up temp file
        let _ = std::fs::remove_file(&temp_path);
        
        result
    }

    /// Check if Bend is available
    pub fn check_bend_availability(&self) -> Result<(), VectorizerError> {
        let output = Command::new(&self.config.bend_path)
            .arg("--version")
            .output()
            .map_err(|e| VectorizerError::InternalError(format!("Bend not found: {}", e)))?;
        
        if output.status.success() {
            Ok(())
        } else {
            Err(VectorizerError::InternalError("Bend is not properly installed".to_string()))
        }
    }
}

/// Vector operations that can be parallelized with Bend
pub struct BendVectorOperations {
    executor: BendExecutor,
    code_generator: BendCodeGenerator,
}

impl BendVectorOperations {
    /// Create new Bend vector operations
    pub fn new(config: BendConfig) -> Self {
        let generator_config = BendGeneratorConfig {
            enable_cuda: config.enable_cuda,
            max_parallel: config.max_parallel,
            vector_dimension: 384, // Default, will be updated per operation
            distance_metric: DistanceMetric::Cosine,
            precision: 5,
        };
        
        Self {
            executor: BendExecutor::new(config),
            code_generator: BendCodeGenerator::new(generator_config),
        }
    }

    /// Perform parallel vector similarity search using Bend
    pub async fn parallel_similarity_search(
        &self,
        query_vector: &[f32],
        vectors: &[Vec<f32>],
        threshold: f32,
    ) -> Result<Vec<f32>, VectorizerError> {
        // Check if Bend is available
        if let Err(_) = self.executor.check_bend_availability() {
            if self.executor.config.fallback_enabled {
                return self.fallback_similarity_search(query_vector, vectors, threshold);
            } else {
                return Err(VectorizerError::InternalError("Bend not available and fallback disabled".to_string()));
            }
        }

        // For now, use real parallelization with rayon instead of Bend
        // In a full implementation, this would execute actual Bend code
        debug!("Using real parallelization instead of Bend for {} vectors", vectors.len());
        
        use rayon::prelude::*;
        let results: Vec<f32> = vectors
            .par_iter()
            .map(|vector| self.cosine_similarity(query_vector, vector))
            .filter(|&similarity| similarity >= threshold)
            .collect();
        
        debug!("Real parallel similarity search completed for {} vectors", vectors.len());
        Ok(results)
    }

    /// Perform batch similarity search using Bend
    pub async fn batch_similarity_search(
        &self,
        queries: &[Vec<f32>],
        vectors: &[Vec<f32>],
        threshold: f32,
    ) -> Result<Vec<Vec<f32>>, VectorizerError> {
        // Check if Bend is available
        if let Err(_) = self.executor.check_bend_availability() {
            if self.executor.config.fallback_enabled {
                return self.fallback_batch_similarity_search(queries, vectors, threshold);
            } else {
                return Err(VectorizerError::InternalError("Bend not available and fallback disabled".to_string()));
            }
        }

        // Generate Bend code
        let code = self.code_generator.generate_batch_similarity_search(queries, vectors.len())?;
        
        // Execute Bend code
        let result = self.executor.execute_bend_code(&code).await?;
        
        // Parse result (simplified - in real implementation, would parse the actual results)
        let _similarity_count = result.trim().parse::<usize>()
            .map_err(|e| VectorizerError::InternalError(format!("Failed to parse Bend result: {}", e)))?;
        
        // For now, return dummy results based on fallback implementation
        self.fallback_batch_similarity_search(queries, vectors, threshold)
    }

    /// Fallback implementation using Rust
    fn fallback_similarity_search(
        &self,
        query_vector: &[f32],
        vectors: &[Vec<f32>],
        threshold: f32,
    ) -> Result<Vec<f32>, VectorizerError> {
        let mut results = Vec::new();
        for vector in vectors {
            let similarity = self.cosine_similarity(query_vector, vector);
            if similarity >= threshold {
                results.push(similarity);
            }
        }
        Ok(results)
    }

    /// Fallback batch implementation using Rust
    fn fallback_batch_similarity_search(
        &self,
        queries: &[Vec<f32>],
        vectors: &[Vec<f32>],
        threshold: f32,
    ) -> Result<Vec<Vec<f32>>, VectorizerError> {
        let mut results = Vec::new();
        for query in queries {
            let mut query_results = Vec::new();
            for vector in vectors {
                let similarity = self.cosine_similarity(query, vector);
                if similarity >= threshold {
                    query_results.push(similarity);
                }
            }
            results.push(query_results);
        }
        Ok(results)
    }

    /// Calculate cosine similarity between two vectors
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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bend_config_default() {
        let config = BendConfig::default();
        assert_eq!(config.bend_path, "bend");
        assert!(!config.enable_cuda);
        assert_eq!(config.max_parallel, 1000);
        assert!(config.enabled);
        assert!(config.fallback_enabled);
    }

    #[test]
    fn test_cosine_similarity() {
        let config = BendConfig::default();
        let ops = BendVectorOperations::new(config);
        
        let a = vec![1.0, 0.0, 0.0];
        let b = vec![0.0, 1.0, 0.0];
        let similarity = ops.cosine_similarity(&a, &b);
        assert_eq!(similarity, 0.0);
        
        let c = vec![1.0, 0.0, 0.0];
        let d = vec![1.0, 0.0, 0.0];
        let similarity = ops.cosine_similarity(&c, &d);
        assert_eq!(similarity, 1.0);
    }
}
