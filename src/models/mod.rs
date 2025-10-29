//! Data models for Vectorizer

use std::fmt;

use serde::{Deserialize, Serialize};

/// A vector with its associated data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Vector {
    /// Unique identifier for the vector
    pub id: String,
    /// The vector data (always f32 for compatibility)
    pub data: Vec<f32>,
    /// Optional payload associated with the vector
    pub payload: Option<Payload>,
}

/// Internal storage format for quantized vectors (memory optimized)
#[derive(Debug, Clone)]
pub struct QuantizedVector {
    /// Unique identifier for the vector
    pub id: String,
    /// Quantized vector data (1 byte per dimension instead of 4)
    pub quantized_data: Vec<u8>,
    /// Quantization parameters for reconstruction
    pub min_val: f32,
    pub max_val: f32,
    /// Optional payload associated with the vector
    pub payload: Option<Payload>,
}

impl QuantizedVector {
    /// Create from full precision vector
    pub fn from_vector(vector: Vector) -> Self {
        let (quantized_data, min_val, max_val) = quantize_to_u8(&vector.data);
        Self {
            id: vector.id,
            quantized_data,
            min_val,
            max_val,
            payload: vector.payload,
        }
    }

    /// Convert back to full precision vector (for search/API responses)
    pub fn to_vector(&self) -> Vector {
        let data = dequantize_from_u8(&self.quantized_data, self.min_val, self.max_val);
        Vector {
            id: self.id.clone(),
            data,
            payload: self.payload.clone(),
        }
    }

    /// Get memory usage in bytes (1 byte per dimension + overhead)
    pub fn memory_size(&self) -> usize {
        self.quantized_data.len() + std::mem::size_of::<f32>() * 2 + self.id.len()
    }
}

/// Quantize f32 vector to u8 (0-255 range)
fn quantize_to_u8(data: &[f32]) -> (Vec<u8>, f32, f32) {
    let min_val = data.iter().copied().fold(f32::INFINITY, f32::min);
    let max_val = data.iter().copied().fold(f32::NEG_INFINITY, f32::max);
    let range = max_val - min_val;

    let quantized = if range > 0.0 {
        data.iter()
            .map(|&v| (((v - min_val) / range) * 255.0) as u8)
            .collect()
    } else {
        vec![0u8; data.len()]
    };

    (quantized, min_val, max_val)
}

/// Dequantize u8 vector back to f32
fn dequantize_from_u8(quantized: &[u8], min_val: f32, max_val: f32) -> Vec<f32> {
    let range = max_val - min_val;
    quantized
        .iter()
        .map(|&v| (v as f32 / 255.0) * range + min_val)
        .collect()
}

/// Arbitrary JSON payload associated with a vector
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct Payload {
    /// The payload data as a JSON value
    #[serde(flatten)]
    pub data: serde_json::Value,
}

impl Payload {
    /// Normalize text content in payload using proper normalization pipeline
    /// This applies conservative normalization (CRLF->LF) to preserve structure
    pub fn normalize(&mut self) {
        Self::normalize_value(&mut self.data);
    }

    /// Recursively normalize text values in JSON
    /// Normalizes line endings and collapses excessive whitespace
    fn normalize_value(value: &mut serde_json::Value) {
        match value {
            serde_json::Value::String(s) => {
                // Step 1: Normalize all line endings to LF
                *s = s.replace("\r\n", "\n").replace('\r', "\n");

                // Step 2: Collapse multiple consecutive newlines (more than 2) into 2
                while s.contains("\n\n\n") {
                    *s = s.replace("\n\n\n", "\n\n");
                }

                // Step 3: Trim leading/trailing whitespace from each line
                *s = s
                    .lines()
                    .map(|line| line.trim_end())
                    .collect::<Vec<_>>()
                    .join("\n");

                // Step 4: Remove leading/trailing empty lines
                *s = s.trim().to_string();
            }
            serde_json::Value::Object(map) => {
                for v in map.values_mut() {
                    Self::normalize_value(v);
                }
            }
            serde_json::Value::Array(arr) => {
                for v in arr.iter_mut() {
                    Self::normalize_value(v);
                }
            }
            _ => {}
        }
    }

    /// Create a normalized copy of this payload
    pub fn normalized(&self) -> Self {
        let mut copy = self.clone();
        copy.normalize();
        copy
    }
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
    /// Quantization configuration (enabled by default for memory optimization)
    pub quantization: QuantizationConfig,
    /// Compression configuration
    pub compression: CompressionConfig,
    /// Text normalization configuration (optional, disabled by default)
    #[serde(default)]
    pub normalization: Option<crate::normalization::NormalizationConfig>,
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

impl Default for CollectionConfig {
    fn default() -> Self {
        Self {
            dimension: 512,
            metric: DistanceMetric::Cosine,
            hnsw_config: HnswConfig::default(),
            quantization: QuantizationConfig::SQ { bits: 8 }, // Enable Scalar Quantization by default
            compression: CompressionConfig::default(),
            normalization: Some(crate::normalization::NormalizationConfig::moderate()), // Enable moderate normalization by default
        }
    }
}

/// Vector quantization configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum QuantizationConfig {
    /// No quantization (store full precision vectors)
    None,
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

impl Default for QuantizationConfig {
    fn default() -> Self {
        Self::SQ { bits: 8 } // Default to 8-bit scalar quantization
    }
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

impl CollectionMetadata {
    /// Check if text normalization is enabled
    pub fn is_normalization_enabled(&self) -> bool {
        self.config
            .normalization
            .as_ref()
            .map(|n| n.enabled)
            .unwrap_or(false)
    }

    /// Get normalization level if enabled
    pub fn normalization_level(&self) -> Option<String> {
        self.config
            .normalization
            .as_ref()
            .filter(|n| n.enabled)
            .map(|n| format!("{:?}", n.policy.level))
    }

    /// Get normalization configuration details
    pub fn normalization_config(&self) -> Option<&crate::normalization::NormalizationConfig> {
        self.config.normalization.as_ref()
    }
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

    /// Create a new payload from a serializable type
    pub fn from_serializable<T: serde::Serialize>(value: &T) -> Result<Self, serde_json::Error> {
        let data = serde_json::to_value(value)?;
        Ok(Self::new(data))
    }

    /// Deserialize payload to a specific type
    pub fn deserialize<T: serde::de::DeserializeOwned>(&self) -> Result<T, serde_json::Error> {
        serde_json::from_value(self.data.clone())
    }
}

/// Collection metadata module for tracking indexed files
pub mod collection_metadata;

/// Qdrant API compatibility module
pub mod qdrant;
