//! Qdrant search models
//!
//! This module provides data structures for Qdrant search operations,
//! including search requests, responses, and scoring functions.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

/// Search request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QdrantSearchRequest {
    /// Query vector
    pub vector: Vec<f32>,
    /// Filter
    pub filter: Option<QdrantFilter>,
    /// Limit
    pub limit: Option<u32>,
    /// Offset
    pub offset: Option<u32>,
    /// With payload
    pub with_payload: Option<bool>,
    /// With vector
    pub with_vector: Option<bool>,
    /// Score threshold
    pub score_threshold: Option<f32>,
    /// Using
    pub using: Option<String>,
    /// Lookup from
    pub lookup_from: Option<QdrantLookupLocation>,
}

/// Search response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QdrantSearchResponse {
    /// Search results
    pub result: Vec<QdrantScoredPoint>,
}

/// Scored point
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QdrantScoredPoint {
    /// Point ID
    pub id: QdrantPointId,
    /// Point vector
    pub vector: Option<QdrantVector>,
    /// Point payload
    pub payload: Option<HashMap<String, QdrantValue>>,
    /// Score
    pub score: f32,
}

/// Filter condition (re-exported from point module)
pub use super::point::QdrantFilter;
/// Point ID (re-exported from point module)
pub use super::point::QdrantPointId;
/// Payload value (re-exported from point module)
pub use super::point::QdrantValue;
/// Vector data (re-exported from point module)
pub use super::point::QdrantVector;

/// Lookup location
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QdrantLookupLocation {
    /// Collection name
    pub collection: String,
    /// Vector name
    pub vector: Option<String>,
}

/// Recommend request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QdrantRecommendRequest {
    /// Positive examples
    pub positive: Vec<QdrantPointId>,
    /// Negative examples
    pub negative: Option<Vec<QdrantPointId>>,
    /// Filter
    pub filter: Option<QdrantFilter>,
    /// Limit
    pub limit: Option<u32>,
    /// Offset
    pub offset: Option<u32>,
    /// With payload
    pub with_payload: Option<bool>,
    /// With vector
    pub with_vector: Option<bool>,
    /// Score threshold
    pub score_threshold: Option<f32>,
    /// Using
    pub using: Option<String>,
    /// Lookup from
    pub lookup_from: Option<QdrantLookupLocation>,
    /// Strategy
    pub strategy: Option<QdrantRecommendStrategy>,
}

/// Recommend strategy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum QdrantRecommendStrategy {
    /// Average vector
    #[serde(rename = "average_vector")]
    AverageVector,
    /// Best score
    #[serde(rename = "best_score")]
    BestScore,
}

/// Recommend response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QdrantRecommendResponse {
    /// Recommend results
    pub result: Vec<QdrantScoredPoint>,
}

/// Scroll request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QdrantScrollRequest {
    /// Filter
    pub filter: Option<QdrantFilter>,
    /// Limit
    pub limit: Option<u32>,
    /// Offset
    pub offset: Option<QdrantPointId>,
    /// With payload
    pub with_payload: Option<bool>,
    /// With vector
    pub with_vector: Option<bool>,
    /// Order by
    pub order_by: Option<QdrantOrderBy>,
}

/// Order by
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QdrantOrderBy {
    /// Key
    pub key: String,
    /// Direction
    pub direction: Option<QdrantDirection>,
}

/// Direction
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum QdrantDirection {
    /// Ascending
    #[serde(rename = "asc")]
    Asc,
    /// Descending
    #[serde(rename = "desc")]
    Desc,
}

/// Scroll response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QdrantScrollResponse {
    /// Scroll results
    pub result: QdrantScrollResult,
}

/// Scroll result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QdrantScrollResult {
    /// Points
    pub points: Vec<QdrantPointStruct>,
    /// Next page offset
    pub next_page_offset: Option<QdrantPointId>,
}

/// Point struct (re-exported from point module)
pub use super::point::QdrantPointStruct;

/// Count request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QdrantCountRequest {
    /// Filter
    pub filter: Option<QdrantFilter>,
    /// Exact count
    pub exact: Option<bool>,
}

/// Count response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QdrantCountResponse {
    /// Count result
    pub result: QdrantCountResult,
}

/// Count result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QdrantCountResult {
    /// Number of points
    pub count: u64,
}

/// Batch search request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QdrantBatchSearchRequest {
    /// Search requests
    pub searches: Vec<QdrantSearchRequest>,
}

/// Batch search response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QdrantBatchSearchResponse {
    /// Search results
    pub result: Vec<QdrantSearchResponse>,
}

/// Batch recommend request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QdrantBatchRecommendRequest {
    /// Recommend requests
    pub searches: Vec<QdrantRecommendRequest>,
}

/// Batch recommend response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QdrantBatchRecommendResponse {
    /// Recommend results
    pub result: Vec<QdrantRecommendResponse>,
}
