//! Data models for Vectorizer

use serde::{Deserialize, Serialize};
use std::fmt;

/// A vector with its associated data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Vector {
    /// Unique identifier for the vector
    pub id: String,
    /// The vector data
    pub data: Vec<f32>,
    /// Optional payload associated with the vector
    pub payload: Option<Payload>,
}

/// Arbitrary JSON payload associated with a vector
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Payload {
    /// The payload data as a JSON value
    #[serde(flatten)]
    pub data: serde_json::Value,
}

/// Configuration for a collection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollectionConfig {
    /// Vector dimension
    pub dimension: usize,
    /// Distance metric for similarity calculations
    pub metric: DistanceMetric,
    /// HNSW index configuration
    pub hnsw_config: HnswConfig,
    /// Optional quantization configuration
    pub quantization: Option<QuantizationConfig>,
    /// Compression configuration
    pub compression: CompressionConfig,
}

/// Distance metrics for vector similarity
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum DistanceMetric {
    /// Cosine similarity
    Cosine,
    /// Euclidean distance
    Euclidean,
    /// Dot product
    DotProduct,
}

impl fmt::Display for DistanceMetric {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DistanceMetric::Cosine => write!(f, "cosine"),
            DistanceMetric::Euclidean => write!(f, "euclidean"),
            DistanceMetric::DotProduct => write!(f, "dot_product"),
        }
    }
}

/// HNSW index configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HnswConfig {
    /// Number of bidirectional links created for each node (except initial layer)
    pub m: usize,
    /// Size of the dynamic list for the nearest neighbors (used during construction)
    pub ef_construction: usize,
    /// Size of the dynamic list for the nearest neighbors (used during search)
    pub ef_search: usize,
    /// Random seed for level assignment
    pub seed: Option<u64>,
}

impl Default for HnswConfig {
    fn default() -> Self {
        Self {
            m: 16,
            ef_construction: 200,
            ef_search: 100,
            seed: None,
        }
    }
}

/// Vector quantization configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum QuantizationConfig {
    /// Product Quantization
    PQ {
        n_centroids: usize,
        n_subquantizers: usize,
    },
    /// Scalar Quantization
    SQ { bits: usize },
    /// Binary Quantization
    Binary,
}

/// Compression configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompressionConfig {
    /// Enable compression for payloads
    pub enabled: bool,
    /// Minimum payload size in bytes to trigger compression
    pub threshold_bytes: usize,
    /// Compression algorithm
    pub algorithm: CompressionAlgorithm,
}

impl Default for CompressionConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            threshold_bytes: 1024,
            algorithm: CompressionAlgorithm::Lz4,
        }
    }
}

/// Compression algorithms
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum CompressionAlgorithm {
    /// No compression
    None,
    /// LZ4 compression
    Lz4,
}

/// Search result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    /// Vector ID
    pub id: String,
    /// Similarity score
    pub score: f32,
    /// Vector data (optional)
    pub vector: Option<Vec<f32>>,
    /// Associated payload
    pub payload: Option<Payload>,
}

/// Collection metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollectionMetadata {
    /// Collection name
    pub name: String,
    /// Creation timestamp
    pub created_at: chrono::DateTime<chrono::Utc>,
    /// Last update timestamp
    pub updated_at: chrono::DateTime<chrono::Utc>,
    /// Number of vectors
    pub vector_count: usize,
    /// Number of documents indexed
    pub document_count: usize,
    /// Collection configuration
    pub config: CollectionConfig,
}

/// Vector normalization and similarity utilities
pub mod vector_utils {
    use super::DistanceMetric;

    /// Normalize a vector to unit length (for cosine similarity)
    pub fn normalize_vector(vector: &[f32]) -> Vec<f32> {
        let norm: f32 = vector.iter().map(|x| x * x).sum::<f32>().sqrt();
        if norm == 0.0 {
            vector.to_vec()
        } else {
            vector.iter().map(|x| x / norm).collect()
        }
    }

    /// Calculate dot product of two vectors
    pub fn dot_product(a: &[f32], b: &[f32]) -> f32 {
        a.iter().zip(b.iter()).map(|(x, y)| x * y).sum()
    }

    /// Calculate Euclidean distance between two vectors
    pub fn euclidean_distance(a: &[f32], b: &[f32]) -> f32 {
        a.iter()
            .zip(b.iter())
            .map(|(x, y)| (x - y) * (x - y))
            .sum::<f32>()
            .sqrt()
    }

    /// Calculate cosine similarity between two vectors (assumes normalized vectors)
    pub fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
        dot_product(a, b).clamp(-1.0, 1.0) // Clamp to [-1, 1]
    }

    /// Convert distance metric result to similarity score
    pub fn distance_to_similarity(distance: f32, metric: DistanceMetric) -> f32 {
        match metric {
            DistanceMetric::Euclidean => {
                // Convert Euclidean distance to similarity (higher values = more similar)
                // Using exponential decay: similarity = exp(-distance)
                (-distance).exp()
            }
            DistanceMetric::Cosine => {
                // Cosine similarity is already in [-1, 1] range
                // Convert to [0, 1] range for consistency
                (distance + 1.0) / 2.0
            }
            DistanceMetric::DotProduct => {
                // Dot product can be any value, normalize to [0, 1]
                // Using sigmoid function: similarity = 1 / (1 + exp(-dot_product))
                1.0 / (1.0 + (-distance).exp())
            }
        }
    }
}

impl Vector {
    /// Create a new vector
    pub fn new(id: String, data: Vec<f32>) -> Self {
        Self {
            id,
            data,
            payload: None,
        }
    }

    /// Create a new vector with payload
    pub fn with_payload(id: String, data: Vec<f32>, payload: Payload) -> Self {
        Self {
            id,
            data,
            payload: Some(payload),
        }
    }

    /// Get the dimension of the vector
    pub fn dimension(&self) -> usize {
        self.data.len()
    }
}

impl Payload {
    /// Create a new payload from a JSON value
    pub fn new(data: serde_json::Value) -> Self {
        Self { data }
    }

    /// Create a payload from a serializable type
    pub fn from_value<T: Serialize>(value: T) -> Result<Self, serde_json::Error> {
        Ok(Self {
            data: serde_json::to_value(value)?,
        })
    }

    /// Extract a value from the payload
    pub fn get<T: for<'de> Deserialize<'de>>(&self) -> Result<T, serde_json::Error> {
        serde_json::from_value(self.data.clone())
    }
}

/// Collection metadata module for tracking indexed files
pub mod collection_metadata;
