//! # GPU Adapter Layer
//!
//! This module provides an adapter layer between hive-vectorizer and hive-gpu.
//! It translates between vectorizer types and hive-gpu types.

use crate::error::{Result, VectorizerError};
use crate::models::{Vector, Payload};
use std::collections::HashMap;

// Re-export hive-gpu types for convenience
pub use hive_gpu::{
    GpuVector as HiveGpuVector,
    GpuDistanceMetric as HiveGpuDistanceMetric,
    GpuSearchResult as HiveGpuSearchResult,
    HnswConfig as HiveGpuHnswConfig,
    HiveGpuError,
    GpuBackend,
    GpuVectorStorage,
    GpuContext,
};

/// Adapter for converting between vectorizer and hive-gpu types
pub struct GpuAdapter;

impl GpuAdapter {
    /// Convert vectorizer Vector to hive-gpu GpuVector
    pub fn vector_to_gpu_vector(vector: &Vector) -> HiveGpuVector {
        HiveGpuVector {
            id: vector.id.clone(),
            data: vector.data.clone(),
            metadata: vector.payload.as_ref().map(|p| {
                // Convert Payload to HashMap<String, String>
                match &p.data {
                    serde_json::Value::Object(map) => {
                        map.iter()
                            .filter_map(|(k, v)| {
                                if let Some(s) = v.as_str() {
                                    Some((k.clone(), s.to_string()))
                                } else {
                                    None
                                }
                            })
                            .collect()
                    }
                    _ => std::collections::HashMap::new(),
                }
            }).unwrap_or_default(),
        }
    }

    /// Convert hive-gpu GpuVector to vectorizer Vector
    pub fn gpu_vector_to_vector(gpu_vector: &HiveGpuVector) -> Vector {
        Vector {
            id: gpu_vector.id.clone(),
            data: gpu_vector.data.clone(),
            payload: if gpu_vector.metadata.is_empty() {
                None
            } else {
                // Convert HashMap<String, String> to Payload
                let json_value = serde_json::Value::Object(
                    gpu_vector.metadata.iter()
                        .map(|(k, v)| (k.clone(), serde_json::Value::String(v.clone())))
                        .collect()
                );
                Some(Payload::new(json_value))
            },
        }
    }

    /// Convert vectorizer distance metric to hive-gpu metric
    pub fn distance_metric_to_gpu_metric(metric: crate::models::DistanceMetric) -> HiveGpuDistanceMetric {
        match metric {
            crate::models::DistanceMetric::Cosine => HiveGpuDistanceMetric::Cosine,
            crate::models::DistanceMetric::Euclidean => HiveGpuDistanceMetric::Euclidean,
            crate::models::DistanceMetric::DotProduct => HiveGpuDistanceMetric::DotProduct,
        }
    }

    /// Convert hive-gpu distance metric to vectorizer metric
    pub fn gpu_metric_to_distance_metric(gpu_metric: HiveGpuDistanceMetric) -> crate::models::DistanceMetric {
        match gpu_metric {
            HiveGpuDistanceMetric::Cosine => crate::models::DistanceMetric::Cosine,
            HiveGpuDistanceMetric::Euclidean => crate::models::DistanceMetric::Euclidean,
            HiveGpuDistanceMetric::DotProduct => crate::models::DistanceMetric::DotProduct,
        }
    }

    /// Convert vectorizer HNSW config to hive-gpu config
    pub fn hnsw_config_to_gpu_config(config: &crate::models::HnswConfig) -> HiveGpuHnswConfig {
        HiveGpuHnswConfig {
            max_connections: config.m,
            ef_construction: config.ef_construction,
            ef_search: config.ef_search,
            max_level: 8, // Default value
            level_multiplier: 0.5, // Default value
            seed: config.seed,
        }
    }

    /// Convert hive-gpu HNSW config to vectorizer config
    pub fn gpu_config_to_hnsw_config(gpu_config: &HiveGpuHnswConfig) -> crate::models::HnswConfig {
        crate::models::HnswConfig {
            m: gpu_config.max_connections,
            ef_construction: gpu_config.ef_construction,
            ef_search: gpu_config.ef_search,
            seed: gpu_config.seed,
        }
    }

    /// Convert hive-gpu error to vectorizer error
    pub fn gpu_error_to_vectorizer_error(error: HiveGpuError) -> VectorizerError {
        match error {
            HiveGpuError::NoDeviceAvailable => VectorizerError::Other("No GPU device available".to_string()),
            HiveGpuError::DimensionMismatch { expected, actual } => {
                VectorizerError::DimensionMismatch { expected, actual }
            },
            HiveGpuError::VectorNotFound(id) => VectorizerError::Other(format!("Vector not found: {}", id)),
            HiveGpuError::VramLimitExceeded { requested, limit } => {
                VectorizerError::Other(format!("VRAM limit exceeded: requested {}, limit {}", requested, limit))
            },
            HiveGpuError::ShaderCompilationFailed(msg) => {
                VectorizerError::Other(format!("Shader compilation failed: {}", msg))
            },
            HiveGpuError::InvalidDimension { expected, got } => {
                VectorizerError::DimensionMismatch { expected, actual: got }
            },
            HiveGpuError::GpuOperationFailed(msg) => VectorizerError::Other(format!("GPU operation failed: {}", msg)),
            HiveGpuError::BufferAllocationFailed(msg) => VectorizerError::Other(format!("Buffer allocation failed: {}", msg)),
            HiveGpuError::DeviceInitializationFailed(msg) => VectorizerError::Other(format!("Device initialization failed: {}", msg)),
            HiveGpuError::MemoryAllocationFailed(msg) => VectorizerError::Other(format!("Memory allocation failed: {}", msg)),
            HiveGpuError::JsonError(e) => VectorizerError::Other(format!("JSON error: {}", e)),
            HiveGpuError::SearchFailed(msg) => VectorizerError::Other(format!("Search failed: {}", msg)),
            HiveGpuError::InvalidConfiguration(msg) => VectorizerError::Other(format!("Invalid configuration: {}", msg)),
            HiveGpuError::InternalError(msg) => VectorizerError::Other(format!("Internal error: {}", msg)),
            HiveGpuError::IoError(e) => VectorizerError::Other(format!("IO error: {}", e)),
            HiveGpuError::Other(msg) => VectorizerError::Other(msg),
        }
    }

    /// Convert vectorizer error to hive-gpu error
    pub fn vectorizer_error_to_gpu_error(error: VectorizerError) -> HiveGpuError {
        match error {
            VectorizerError::DimensionMismatch { expected, actual } => {
                HiveGpuError::DimensionMismatch { expected, actual }
            },
            VectorizerError::Other(msg) => HiveGpuError::Other(msg),
            _ => HiveGpuError::Other(format!("Unknown error: {:?}", error)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{Vector, Payload, DistanceMetric, HnswConfig};

    #[test]
    fn test_vector_conversion() {
        let vector = Vector::new("test_id".to_string(), vec![1.0, 2.0, 3.0]);
        let gpu_vector = GpuAdapter::vector_to_gpu_vector(&vector);
        
        assert_eq!(gpu_vector.id, "test_id");
        assert_eq!(gpu_vector.data, vec![1.0, 2.0, 3.0]);
        assert!(gpu_vector.metadata.is_empty());
        
        let converted_back = GpuAdapter::gpu_vector_to_vector(&gpu_vector);
        assert_eq!(converted_back.id, vector.id);
        assert_eq!(converted_back.data, vector.data);
        assert_eq!(converted_back.payload, vector.payload);
    }

    #[test]
    fn test_vector_with_payload() {
        let payload = Payload::new(serde_json::json!({
            "title": "Test Document",
            "content": "This is a test"
        }));
        let vector = Vector::with_payload("test_id".to_string(), vec![1.0, 2.0, 3.0], payload);
        let gpu_vector = GpuAdapter::vector_to_gpu_vector(&vector);
        
        assert_eq!(gpu_vector.metadata.get("title"), Some(&"Test Document".to_string()));
        assert_eq!(gpu_vector.metadata.get("content"), Some(&"This is a test".to_string()));
    }

    #[test]
    fn test_distance_metric_conversion() {
        let cpu_metric = DistanceMetric::Cosine;
        let gpu_metric = GpuAdapter::distance_metric_to_gpu_metric(cpu_metric);
        let converted_back = GpuAdapter::gpu_metric_to_distance_metric(gpu_metric);
        
        assert_eq!(converted_back, DistanceMetric::Cosine);
    }

    #[test]
    fn test_hnsw_config_conversion() {
        let cpu_config = HnswConfig {
            m: 16,
            ef_construction: 200,
            ef_search: 50,
            seed: Some(42),
        };
        
        let gpu_config = GpuAdapter::hnsw_config_to_gpu_config(&cpu_config);
        assert_eq!(gpu_config.max_connections, 16);
        assert_eq!(gpu_config.ef_construction, 200);
        assert_eq!(gpu_config.ef_search, 50);
        assert_eq!(gpu_config.seed, Some(42));
        assert_eq!(gpu_config.max_level, 8);
        assert_eq!(gpu_config.level_multiplier, 0.5);
        
        let converted_back = GpuAdapter::gpu_config_to_hnsw_config(&gpu_config);
        assert_eq!(converted_back.m, cpu_config.m);
        assert_eq!(converted_back.ef_construction, cpu_config.ef_construction);
        assert_eq!(converted_back.ef_search, cpu_config.ef_search);
        assert_eq!(converted_back.seed, cpu_config.seed);
    }
}