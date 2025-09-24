//! API request/response types for the Vectorizer REST API

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Request to create a new collection
#[derive(Debug, Deserialize)]
pub struct CreateCollectionRequest {
    /// Collection name
    pub name: String,
    /// Vector dimension
    pub dimension: usize,
    /// Distance metric to use
    pub metric: DistanceMetric,
    /// HNSW configuration (optional)
    pub hnsw_config: Option<HnswConfig>,
}

/// Response for collection creation
#[derive(Debug, Serialize)]
pub struct CreateCollectionResponse {
    /// Success message
    pub message: String,
    /// Collection name
    pub collection: String,
}

/// Collection information
#[derive(Debug, Serialize)]
pub struct CollectionInfo {
    /// Collection name
    pub name: String,
    /// Vector dimension
    pub dimension: usize,
    /// Distance metric
    pub metric: DistanceMetric,
    /// Number of vectors
    pub vector_count: usize,
    /// Creation timestamp
    pub created_at: String,
    /// Last update timestamp
    pub updated_at: String,
}

/// List collections response
#[derive(Debug, Serialize)]
pub struct ListCollectionsResponse {
    /// Collections list
    pub collections: Vec<CollectionInfo>,
}

/// Vector information
#[derive(Debug, Serialize)]
pub struct VectorResponse {
    /// Vector ID
    pub id: String,
    /// Vector payload (optional)
    pub payload: Option<serde_json::Value>,
}

/// List vectors response
#[derive(Debug, Serialize)]
pub struct ListVectorsResponse {
    /// Vectors list
    pub vectors: Vec<VectorResponse>,
    /// Total number of vectors
    pub total: usize,
    /// Limit used
    pub limit: usize,
    /// Offset used
    pub offset: usize,
}

/// Request to insert vectors
#[derive(Debug, Deserialize)]
pub struct InsertVectorsRequest {
    /// Vectors to insert
    pub vectors: Vec<VectorData>,
}

/// Vector data for API
#[derive(Debug, Serialize, Deserialize)]
pub struct VectorData {
    /// Vector ID
    pub id: String,
    /// Vector values
    #[serde(alias = "data")]
    pub vector: Vec<f32>,
    /// Optional payload
    pub payload: Option<serde_json::Value>,
}

/// Response for vector insertion
#[derive(Debug, Serialize)]
pub struct InsertVectorsResponse {
    /// Success message
    pub message: String,
    /// Number of vectors inserted
    pub inserted: usize,
    /// Number of vectors inserted (alternative key for compatibility)
    pub inserted_count: usize,
}

/// Search request
#[derive(Debug, Deserialize)]
pub struct SearchRequest {
    /// Query vector
    pub vector: Vec<f32>,
    /// Number of results to return
    pub limit: Option<usize>,
    /// Minimum score threshold
    pub score_threshold: Option<f32>,
    /// Filter by file path (optional)
    pub file_filter: Option<String>,
}

/// Search request with text (will be embedded automatically)
#[derive(Debug, Deserialize)]
pub struct SearchTextRequest {
    /// Query text
    pub query: String,
    /// Number of results to return
    pub limit: Option<usize>,
    /// Minimum score threshold
    pub score_threshold: Option<f32>,
    /// Filter by file path (optional)
    pub file_filter: Option<String>,
}

/// Unified search request supporting either vector or text
#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum SearchUnifiedRequest {
    Vector(SearchRequest),
    Text(SearchTextRequest),
}

/// Search result
#[derive(Debug, Serialize)]
pub struct SearchResult {
    /// Vector ID
    pub id: String,
    /// Similarity score
    pub score: f32,
    /// Vector data
    pub vector: Vec<f32>,
    /// Payload if available
    pub payload: Option<serde_json::Value>,
}

/// Search response
#[derive(Debug, Serialize)]
pub struct SearchResponse {
    /// Search results
    pub results: Vec<SearchResult>,
    /// Query execution time in milliseconds
    pub query_time_ms: f64,
}

/// Search by file request
#[derive(Debug, Deserialize)]
pub struct SearchByFileRequest {
    /// File path to search for
    pub file_path: String,
    /// Number of results to return
    pub limit: Option<usize>,
    /// Minimum score threshold
    pub score_threshold: Option<f32>,
}

/// List files request
#[derive(Debug, Deserialize)]
pub struct ListFilesRequest {
    /// Number of results to return
    pub limit: Option<usize>,
    /// Offset for pagination
    pub offset: Option<usize>,
    /// Filter by file extension (optional)
    pub extension_filter: Option<String>,
}

/// File information
#[derive(Debug, Serialize)]
pub struct FileInfo {
    /// File path
    pub file_path: String,
    /// Number of chunks in this file
    pub chunk_count: usize,
    /// File extension
    pub extension: Option<String>,
}

/// List files response
#[derive(Debug, Serialize)]
pub struct ListFilesResponse {
    /// Files list
    pub files: Vec<FileInfo>,
    /// Total number of files
    pub total: usize,
    /// Limit used
    pub limit: usize,
    /// Offset used
    pub offset: usize,
}

/// Distance metric options
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "lowercase")]
pub enum DistanceMetric {
    Cosine,
    Euclidean,
    DotProduct,
}

impl From<DistanceMetric> for crate::models::DistanceMetric {
    fn from(metric: DistanceMetric) -> Self {
        match metric {
            DistanceMetric::Cosine => crate::models::DistanceMetric::Cosine,
            DistanceMetric::Euclidean => crate::models::DistanceMetric::Euclidean,
            DistanceMetric::DotProduct => crate::models::DistanceMetric::DotProduct,
        }
    }
}

impl From<crate::models::DistanceMetric> for DistanceMetric {
    fn from(metric: crate::models::DistanceMetric) -> Self {
        match metric {
            crate::models::DistanceMetric::Cosine => DistanceMetric::Cosine,
            crate::models::DistanceMetric::Euclidean => DistanceMetric::Euclidean,
            crate::models::DistanceMetric::DotProduct => DistanceMetric::DotProduct,
        }
    }
}

/// HNSW configuration for API
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct HnswConfig {
    /// Number of bi-directional links created for every new element during construction
    pub m: Option<usize>,
    /// Size of the dynamic candidate list
    pub ef_construction: Option<usize>,
    /// Size of the dynamic candidate list for search
    pub ef_search: Option<usize>,
    /// Random seed for reproducible results
    pub seed: Option<u64>,
}

impl From<HnswConfig> for crate::models::HnswConfig {
    fn from(config: HnswConfig) -> Self {
        crate::models::HnswConfig {
            m: config.m.unwrap_or(16),
            ef_construction: config.ef_construction.unwrap_or(200),
            ef_search: config.ef_search.unwrap_or(64),
            seed: config.seed,
        }
    }
}

/// Generic API error response
#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    /// Error message
    pub error: String,
    /// Error code
    pub code: String,
    /// Additional details
    pub details: Option<HashMap<String, serde_json::Value>>,
}

/// Health check response
#[derive(Debug, Serialize)]
pub struct HealthResponse {
    /// Service status
    pub status: String,
    /// Service version
    pub version: String,
    /// Current timestamp
    pub timestamp: String,
    /// Uptime in seconds
    pub uptime: u64,
    /// Number of collections
    pub collections: usize,
    /// Total vectors across all collections
    pub total_vectors: usize,
}

/// Request to set embedding provider
#[derive(Debug, Deserialize)]
pub struct SetEmbeddingProviderRequest {
    pub provider_name: String,
}

/// Response for setting embedding provider
#[derive(Debug, Serialize)]
pub struct SetEmbeddingProviderResponse {
    pub success: bool,
    pub message: String,
    pub provider_name: String,
}

/// Response for listing embedding providers
#[derive(Debug, Serialize)]
pub struct ListEmbeddingProvidersResponse {
    pub providers: Vec<String>,
    pub default_provider: Option<String>,
}
