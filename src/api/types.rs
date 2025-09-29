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

/// Indexing status for collections
#[derive(Debug, Serialize, Clone)]
pub struct IndexingStatus {
    /// Current status
    pub status: String,
    /// Progress percentage (0-100)
    pub progress: f32,
    /// Total documents to process
    pub total_documents: usize,
    /// Documents processed so far
    pub processed_documents: usize,
    /// Number of vectors in the collection
    pub vector_count: usize,
    /// Estimated time remaining (in seconds, if available)
    pub estimated_time_remaining: Option<u64>,
    /// Last update timestamp
    pub last_updated: String,
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
    /// Embedding provider used by this collection
    pub embedding_provider: String,
    /// Number of vectors
    pub vector_count: usize,
    /// Number of documents indexed
    pub document_count: usize,
    /// Creation timestamp
    pub created_at: String,
    /// Last update timestamp
    pub updated_at: String,
    /// Indexing status
    pub indexing_status: IndexingStatus,
}

/// List collections response
#[derive(Debug, Serialize)]
pub struct ListCollectionsResponse {
    /// Collections list
    pub collections: Vec<CollectionInfo>,
}

/// Indexing progress response
#[derive(Debug, Serialize)]
pub struct IndexingProgressResponse {
    /// Overall indexing status
    pub overall_status: String,
    /// Collections being indexed
    pub collections: Vec<IndexingStatus>,
    /// Start time of indexing process
    pub started_at: String,
    /// Estimated completion time
    pub estimated_completion: Option<String>,
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
    /// Optional message explaining response behavior
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
}

/// Request to insert texts (embeddings generated automatically)
#[derive(Debug, Deserialize)]
pub struct InsertTextsRequest {
    /// Texts to insert
    pub texts: Vec<TextData>,
}

/// Text data for API
#[derive(Debug, Serialize, Deserialize)]
pub struct TextData {
    /// Text ID
    pub id: String,
    /// Text content
    pub text: String,
    /// Optional metadata
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<serde_json::Value>,
}

/// Vector data for API
#[derive(Debug, Serialize, Deserialize)]
pub struct VectorData {
    /// Vector ID
    pub id: String,
    /// Vector values (optional - will be auto-generated if not provided)
    #[serde(alias = "data")]
    pub vector: Option<Vec<f32>>,
    /// Content text (required for context and embedding generation)
    pub content: String,
    /// Additional metadata (optional)
    pub metadata: Option<serde_json::Value>,
}

/// Response for text insertion
#[derive(Debug, Serialize)]
pub struct InsertTextsResponse {
    /// Success message
    pub message: String,
    /// Number of texts inserted
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

/// Query parameters for listing vectors
#[derive(Debug, Deserialize)]
pub struct ListVectorsQuery {
    /// Maximum number of vectors to return
    pub limit: Option<usize>,
    /// Number of vectors to skip
    pub offset: Option<usize>,
    /// Minimum score threshold for filtering vectors (0.0 to 1.0)
    pub min_score: Option<f32>,
}

/// Batch configuration for API requests
#[derive(Debug, Deserialize, Clone)]
pub struct BatchConfigRequest {
    /// Maximum batch size
    pub max_batch_size: Option<usize>,
    /// Maximum memory usage in MB
    pub max_memory_usage_mb: Option<usize>,
    /// Number of parallel workers
    pub parallel_workers: Option<usize>,
    /// Chunk size for processing
    pub chunk_size: Option<usize>,
    /// Whether to enable progress reporting
    pub progress_reporting: Option<bool>,
}

impl Default for BatchConfigRequest {
    fn default() -> Self {
        Self {
            max_batch_size: Some(1000),
            max_memory_usage_mb: Some(512),
            parallel_workers: Some(4),
            chunk_size: Some(100),
            progress_reporting: Some(true),
        }
    }
}

/// Text data for batch operations
#[derive(Debug, Deserialize)]
pub struct BatchTextRequest {
    /// Text ID
    pub id: String,
    /// Text content
    pub content: String,
    /// Optional metadata
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<serde_json::Value>,
    /// Optional pre-generated vector data (if not provided, will be generated)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<Vec<f32>>,
}

/// Vector data for batch operations
#[derive(Debug, Deserialize)]
pub struct BatchVectorRequest {
    /// Vector ID
    pub id: String,
    /// Vector data (optional - will be auto-generated if not provided)
    pub data: Option<Vec<f32>>,
    /// Content text (required for context and embedding generation)
    pub content: String,
    /// Additional metadata (optional)
    pub metadata: Option<serde_json::Value>,
}

/// Vector update for batch operations
#[derive(Debug, Deserialize)]
pub struct BatchVectorUpdateRequest {
    /// Vector ID
    pub id: String,
    /// New vector data (optional)
    pub data: Option<Vec<f32>>,
    /// New metadata (optional)
    pub metadata: Option<HashMap<String, String>>,
}

/// Search query for batch operations
#[derive(Debug, Deserialize)]
pub struct BatchSearchQueryRequest {
    /// Query vector (optional)
    pub query_vector: Option<Vec<f32>>,
    /// Query text (optional)
    pub query_text: Option<String>,
    /// Maximum number of results
    pub limit: usize,
    /// Score threshold (optional)
    pub threshold: Option<f32>,
    /// Metadata filters (optional)
    pub filters: Option<HashMap<String, String>>,
}

/// Batch insert request
#[derive(Debug, Deserialize)]
pub struct BatchInsertRequest {
    /// Texts to insert
    pub texts: Vec<BatchTextRequest>,
    /// Batch configuration (optional)
    pub config: Option<BatchConfigRequest>,
    /// Whether operations should be atomic (optional, defaults to true)
    pub atomic: Option<bool>,
}

/// Batch update request
#[derive(Debug, Deserialize)]
pub struct BatchUpdateRequest {
    /// Updates to apply
    pub updates: Vec<BatchVectorUpdateRequest>,
    /// Batch configuration (optional)
    pub config: Option<BatchConfigRequest>,
    /// Whether operations should be atomic (optional, defaults to true)
    pub atomic: Option<bool>,
}

/// Batch delete request
#[derive(Debug, Deserialize)]
pub struct BatchDeleteRequest {
    /// Vector IDs to delete
    pub vector_ids: Vec<String>,
    /// Batch configuration (optional)
    pub config: Option<BatchConfigRequest>,
    /// Whether operations should be atomic (optional, defaults to true)
    pub atomic: Option<bool>,
}

/// Batch search request
#[derive(Debug, Deserialize)]
pub struct BatchSearchRequest {
    /// Search queries to execute
    pub queries: Vec<BatchSearchQueryRequest>,
    /// Batch configuration (optional)
    pub config: Option<BatchConfigRequest>,
    /// Whether operations should be atomic (optional, defaults to true)
    pub atomic: Option<bool>,
}

/// Batch operation response
#[derive(Debug, Serialize)]
pub struct BatchResponse {
    /// Whether the operation was successful
    pub success: bool,
    /// Collection name
    pub collection: String,
    /// Operation type
    pub operation: String,
    /// Total number of operations
    pub total_operations: usize,
    /// Number of successful operations
    pub successful_operations: usize,
    /// Number of failed operations
    pub failed_operations: usize,
    /// Duration in milliseconds
    pub duration_ms: u64,
    /// Error messages (if any)
    pub errors: Vec<String>,
}

/// Batch search response
#[derive(Debug, Serialize)]
pub struct BatchSearchResponse {
    /// Whether the operation was successful
    pub success: bool,
    /// Collection name
    pub collection: String,
    /// Total number of queries
    pub total_queries: usize,
    /// Number of successful queries
    pub successful_queries: usize,
    /// Number of failed queries
    pub failed_queries: usize,
    /// Duration in milliseconds
    pub duration_ms: u64,
    /// Search results
    pub results: Vec<Vec<SearchResult>>,
    /// Error messages (if any)
    pub errors: Vec<String>,
}
/// Statistics response
#[derive(Debug, Serialize)]
pub struct StatsResponse {
    /// Total number of collections
    pub total_collections: usize,
    /// Total number of vectors across all collections
    pub total_vectors: usize,
    /// Total number of documents
    pub total_documents: usize,
    /// Server uptime in seconds
    pub uptime_seconds: u64,
    /// Memory usage in MB
    pub memory_usage_mb: f64,
    /// CPU usage percentage
    pub cpu_usage_percent: f64,
    /// Timestamp of the stats
    pub timestamp: String,
}

/// Collection statistics
#[derive(Debug, Serialize)]
pub struct CollectionStats {
    /// Collection name
    pub name: String,
    /// Number of vectors
    pub vector_count: usize,
    /// Number of documents
    pub document_count: usize,
    /// Collection dimension
    pub dimension: usize,
    /// Similarity metric
    pub metric: String,
    /// Last updated timestamp
    pub last_updated: String,
}

// =============================================================================
// SUMMARIZATION TYPES
// =============================================================================

/// Request to summarize text
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SummarizeTextRequest {
    /// Text to summarize
    pub text: String,
    /// Summarization method (extractive, keyword, sentence, abstractive)
    pub method: String,
    /// Maximum summary length (optional)
    pub max_length: Option<i32>,
    /// Compression ratio (optional)
    pub compression_ratio: Option<f32>,
    /// Language code (optional)
    pub language: Option<String>,
    /// Additional metadata (optional)
    pub metadata: Option<HashMap<String, String>>,
}

/// Response for text summarization
#[derive(Debug, Serialize, Deserialize)]
pub struct SummarizeTextResponse {
    /// Summary ID
    pub summary_id: String,
    /// Original text
    pub original_text: String,
    /// Generated summary
    pub summary: String,
    /// Method used
    pub method: String,
    /// Original text length
    pub original_length: i32,
    /// Summary length
    pub summary_length: i32,
    /// Compression ratio
    pub compression_ratio: f32,
    /// Language
    pub language: String,
    /// Status
    pub status: String,
    /// Message
    pub message: String,
    /// Metadata
    pub metadata: HashMap<String, String>,
}

/// Request to summarize context
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SummarizeContextRequest {
    /// Context to summarize
    pub context: String,
    /// Summarization method (extractive, keyword, sentence, abstractive)
    pub method: String,
    /// Maximum summary length (optional)
    pub max_length: Option<i32>,
    /// Compression ratio (optional)
    pub compression_ratio: Option<f32>,
    /// Language code (optional)
    pub language: Option<String>,
    /// Additional metadata (optional)
    pub metadata: Option<HashMap<String, String>>,
}

/// Response for context summarization
#[derive(Debug, Serialize, Deserialize)]
pub struct SummarizeContextResponse {
    /// Summary ID
    pub summary_id: String,
    /// Original context
    pub original_context: String,
    /// Generated summary
    pub summary: String,
    /// Method used
    pub method: String,
    /// Original context length
    pub original_length: i32,
    /// Summary length
    pub summary_length: i32,
    /// Compression ratio
    pub compression_ratio: f32,
    /// Language
    pub language: String,
    /// Status
    pub status: String,
    /// Message
    pub message: String,
    /// Metadata
    pub metadata: HashMap<String, String>,
}

/// Response for getting a summary
#[derive(Debug, Serialize, Deserialize)]
pub struct GetSummaryResponse {
    /// Summary ID
    pub summary_id: String,
    /// Original text
    pub original_text: String,
    /// Generated summary
    pub summary: String,
    /// Method used
    pub method: String,
    /// Original text length
    pub original_length: i32,
    /// Summary length
    pub summary_length: i32,
    /// Compression ratio
    pub compression_ratio: f32,
    /// Language
    pub language: String,
    /// Creation timestamp
    pub created_at: String,
    /// Metadata
    pub metadata: HashMap<String, String>,
    /// Status
    pub status: String,
}

/// Query parameters for listing summaries
#[derive(Debug, Deserialize)]
pub struct ListSummariesQuery {
    /// Filter by method (optional)
    pub method: Option<String>,
    /// Filter by language (optional)
    pub language: Option<String>,
    /// Maximum number of summaries to return (optional)
    pub limit: Option<i32>,
    /// Offset for pagination (optional)
    pub offset: Option<i32>,
}

/// Summary information for listing
#[derive(Debug, Serialize, Deserialize)]
pub struct SummaryInfo {
    /// Summary ID
    pub summary_id: String,
    /// Method used
    pub method: String,
    /// Language
    pub language: String,
    /// Original text length
    pub original_length: i32,
    /// Summary length
    pub summary_length: i32,
    /// Compression ratio
    pub compression_ratio: f32,
    /// Creation timestamp
    pub created_at: String,
    /// Metadata
    pub metadata: HashMap<String, String>,
}

/// Response for listing summaries
#[derive(Debug, Serialize, Deserialize)]
pub struct ListSummariesResponse {
    /// List of summaries
    pub summaries: Vec<SummaryInfo>,
    /// Total count
    pub total_count: i32,
    /// Status
    pub status: String,
}
