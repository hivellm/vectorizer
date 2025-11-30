//! Hybrid search models for combining dense and sparse vectors

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

/// Sparse vector representation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SparseVector {
    /// Non-zero indices
    pub indices: Vec<usize>,
    /// Values at corresponding indices
    pub values: Vec<f32>,
}

impl SparseVector {
    /// Create a new sparse vector
    pub fn new(indices: Vec<usize>, values: Vec<f32>) -> Result<Self, String> {
        if indices.len() != values.len() {
            return Err("Indices and values must have the same length".to_string());
        }
        if indices.is_empty() {
            return Err("Sparse vector cannot be empty".to_string());
        }
        for &idx in &indices {
            if idx == usize::MAX {
                return Err("Indices must be valid".to_string());
            }
        }
        for &val in &values {
            if val.is_nan() || val.is_infinite() {
                return Err("Values must be finite numbers".to_string());
            }
        }
        Ok(Self { indices, values })
    }
}

/// Hybrid search request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HybridSearchRequest {
    /// Collection name
    pub collection: String,
    /// Text query for dense vector search
    pub query: String,
    /// Optional sparse vector query
    pub query_sparse: Option<SparseVector>,
    /// Alpha parameter for blending (0.0-1.0)
    #[serde(default = "default_alpha")]
    pub alpha: f32,
    /// Scoring algorithm
    #[serde(default = "default_algorithm")]
    pub algorithm: HybridScoringAlgorithm,
    /// Number of dense results to retrieve
    #[serde(default = "default_dense_k")]
    pub dense_k: usize,
    /// Number of sparse results to retrieve
    #[serde(default = "default_sparse_k")]
    pub sparse_k: usize,
    /// Final number of results to return
    #[serde(default = "default_final_k")]
    pub final_k: usize,
}

fn default_alpha() -> f32 {
    0.7
}

fn default_algorithm() -> HybridScoringAlgorithm {
    HybridScoringAlgorithm::ReciprocalRankFusion
}

fn default_dense_k() -> usize {
    20
}

fn default_sparse_k() -> usize {
    20
}

fn default_final_k() -> usize {
    10
}

/// Hybrid scoring algorithm
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum HybridScoringAlgorithm {
    /// Reciprocal Rank Fusion
    #[serde(rename = "rrf")]
    ReciprocalRankFusion,
    /// Weighted Combination
    #[serde(rename = "weighted")]
    WeightedCombination,
    /// Alpha Blending
    #[serde(rename = "alpha")]
    AlphaBlending,
}

/// Hybrid search result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HybridSearchResult {
    /// Result ID
    pub id: String,
    /// Similarity score
    pub score: f32,
    /// Optional vector data
    pub vector: Option<Vec<f32>>,
    /// Optional payload data
    pub payload: Option<HashMap<String, serde_json::Value>>,
}

/// Hybrid search response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HybridSearchResponse {
    /// Search results
    pub results: Vec<HybridSearchResult>,
    /// Query text
    pub query: String,
    /// Optional sparse query
    pub query_sparse: Option<SparseVectorResponse>,
    /// Alpha parameter used
    pub alpha: f32,
    /// Algorithm used
    pub algorithm: String,
    /// Duration in milliseconds
    pub duration_ms: Option<u64>,
}

/// Sparse vector response format
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SparseVectorResponse {
    /// Indices
    pub indices: Vec<usize>,
    /// Values
    pub values: Vec<f32>,
}
