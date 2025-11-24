//! Data models for the Vectorizer SDK

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// Re-export hybrid search models
pub mod hybrid_search;
pub use hybrid_search::*;

// Re-export graph models
pub mod graph;
pub use graph::*;

/// Vector similarity metrics
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SimilarityMetric {
    /// Cosine similarity
    Cosine,
    /// Euclidean distance
    Euclidean,
    /// Dot product
    DotProduct,
}

impl Default for SimilarityMetric {
    fn default() -> Self {
        Self::Cosine
    }
}

/// Vector representation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Vector {
    /// Unique identifier for the vector
    pub id: String,
    /// Vector data as an array of numbers
    pub data: Vec<f32>,
    /// Optional metadata associated with the vector
    pub metadata: Option<HashMap<String, serde_json::Value>>,
}

/// Collection representation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Collection {
    /// Collection name
    pub name: String,
    /// Vector dimension
    pub dimension: usize,
    /// Similarity metric used for search
    pub similarity_metric: SimilarityMetric,
    /// Optional description
    pub description: Option<String>,
    /// Creation timestamp
    pub created_at: Option<DateTime<Utc>>,
    /// Last update timestamp
    pub updated_at: Option<DateTime<Utc>>,
}

/// Collection information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollectionInfo {
    /// Collection name
    pub name: String,
    /// Vector dimension
    pub dimension: usize,
    /// Similarity metric used for search
    pub metric: String,
    /// Number of vectors in the collection
    pub vector_count: usize,
    /// Number of documents in the collection
    pub document_count: usize,
    /// Creation timestamp
    pub created_at: String,
    /// Last update timestamp
    pub updated_at: String,
    /// Indexing status
    pub indexing_status: IndexingStatus,
}

/// Indexing status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexingStatus {
    /// Status
    pub status: String,
    /// Progress percentage
    pub progress: f32,
    /// Total documents
    pub total_documents: usize,
    /// Processed documents
    pub processed_documents: usize,
    /// Vector count
    pub vector_count: usize,
    /// Estimated time remaining
    pub estimated_time_remaining: Option<String>,
    /// Last updated timestamp
    pub last_updated: String,
}

/// Search result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    /// Vector ID
    pub id: String,
    /// Similarity score
    pub score: f32,
    /// Vector content (if available)
    pub content: Option<String>,
    /// Optional metadata
    pub metadata: Option<HashMap<String, serde_json::Value>>,
}

/// Search response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResponse {
    /// Search results
    pub results: Vec<SearchResult>,
    /// Query time in milliseconds
    pub query_time_ms: f64,
}

/// Embedding request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbeddingRequest {
    /// Text to embed
    pub text: String,
    /// Optional model to use for embedding
    pub model: Option<String>,
    /// Optional parameters for embedding generation
    pub parameters: Option<EmbeddingParameters>,
}

/// Embedding parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbeddingParameters {
    /// Maximum sequence length
    pub max_length: Option<usize>,
    /// Whether to normalize the embedding
    pub normalize: Option<bool>,
    /// Optional prefix for the text
    pub prefix: Option<String>,
}

/// Embedding response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbeddingResponse {
    /// Generated embedding vector
    pub embedding: Vec<f32>,
    /// Model used for embedding
    pub model: String,
    /// Text that was embedded
    pub text: String,
    /// Embedding dimension
    pub dimension: usize,
    /// Provider used
    pub provider: String,
}

/// Health status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthStatus {
    /// Service status
    pub status: String,
    /// Service version
    pub version: String,
    /// Timestamp
    pub timestamp: String,
    /// Uptime in seconds
    pub uptime: Option<u64>,
    /// Number of collections
    pub collections: Option<usize>,
    /// Total number of vectors
    pub total_vectors: Option<usize>,
}

/// Collections list response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollectionsResponse {
    /// List of collections
    pub collections: Vec<CollectionInfo>,
}

/// Create collection response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateCollectionResponse {
    /// Success message
    pub message: String,
    /// Collection name
    pub collection: String,
}

/// Database statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseStats {
    /// Total number of collections
    pub total_collections: usize,
    /// Total number of vectors
    pub total_vectors: usize,
    /// Total memory estimate in bytes
    pub total_memory_estimate_bytes: usize,
    /// Collections information
    pub collections: Vec<CollectionStats>,
}

/// Collection statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollectionStats {
    /// Collection name
    pub name: String,
    /// Number of vectors
    pub vector_count: usize,
    /// Vector dimension
    pub dimension: usize,
    /// Memory estimate in bytes
    pub memory_estimate_bytes: usize,
}

/// Batch text request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchTextRequest {
    /// Text ID
    pub id: String,
    /// Text content
    pub text: String,
    /// Optional metadata
    pub metadata: Option<HashMap<String, String>>,
}

/// Batch configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchConfig {
    /// Maximum batch size
    pub max_batch_size: Option<usize>,
    /// Number of parallel workers
    pub parallel_workers: Option<usize>,
    /// Whether operations should be atomic
    pub atomic: Option<bool>,
}

/// Batch insert request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchInsertRequest {
    /// Texts to insert
    pub texts: Vec<BatchTextRequest>,
    /// Batch configuration
    pub config: Option<BatchConfig>,
}

/// Batch response
#[derive(Debug, Clone, Serialize, Deserialize)]
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
    /// Error messages
    pub errors: Vec<String>,
}

/// Batch search query
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchSearchQuery {
    /// Query text
    pub query: String,
    /// Maximum number of results
    pub limit: Option<usize>,
    /// Minimum score threshold
    pub score_threshold: Option<f32>,
}

/// Batch search request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchSearchRequest {
    /// Search queries
    pub queries: Vec<BatchSearchQuery>,
    /// Batch configuration
    pub config: Option<BatchConfig>,
}

/// Batch search response
#[derive(Debug, Clone, Serialize, Deserialize)]
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
    /// Error messages
    pub errors: Vec<String>,
}

/// Batch vector update
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchVectorUpdate {
    /// Vector ID
    pub id: String,
    /// New vector data (optional)
    pub data: Option<Vec<f32>>,
    /// New metadata (optional)
    pub metadata: Option<HashMap<String, serde_json::Value>>,
}

/// Batch update request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchUpdateRequest {
    /// Vector updates
    pub updates: Vec<BatchVectorUpdate>,
    /// Batch configuration
    pub config: Option<BatchConfig>,
}

/// Batch delete request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchDeleteRequest {
    /// Vector IDs to delete
    pub vector_ids: Vec<String>,
    /// Batch configuration
    pub config: Option<BatchConfig>,
}

/// Summarization methods
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SummarizationMethod {
    /// Extractive summarization
    Extractive,
    /// Keyword summarization
    Keyword,
    /// Sentence summarization
    Sentence,
    /// Abstractive summarization
    Abstractive,
}

impl Default for SummarizationMethod {
    fn default() -> Self {
        Self::Extractive
    }
}

/// Summarize text request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SummarizeTextRequest {
    /// Text to summarize
    pub text: String,
    /// Summarization method
    pub method: Option<SummarizationMethod>,
    /// Maximum summary length
    pub max_length: Option<usize>,
    /// Compression ratio
    pub compression_ratio: Option<f32>,
    /// Language code
    pub language: Option<String>,
}

/// Summarize text response
#[derive(Debug, Clone, Serialize, Deserialize)]
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
    pub original_length: usize,
    /// Summary length
    pub summary_length: usize,
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

/// Summarize context request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SummarizeContextRequest {
    /// Context to summarize
    pub context: String,
    /// Summarization method
    pub method: Option<SummarizationMethod>,
    /// Maximum summary length
    pub max_length: Option<usize>,
    /// Compression ratio
    pub compression_ratio: Option<f32>,
    /// Language code
    pub language: Option<String>,
}

/// Summarize context response
#[derive(Debug, Clone, Serialize, Deserialize)]
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
    pub original_length: usize,
    /// Summary length
    pub summary_length: usize,
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

/// Get summary response
#[derive(Debug, Clone, Serialize, Deserialize)]
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
    pub original_length: usize,
    /// Summary length
    pub summary_length: usize,
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

/// Summary info
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SummaryInfo {
    /// Summary ID
    pub summary_id: String,
    /// Method used
    pub method: String,
    /// Language
    pub language: String,
    /// Original text length
    pub original_length: usize,
    /// Summary length
    pub summary_length: usize,
    /// Compression ratio
    pub compression_ratio: f32,
    /// Creation timestamp
    pub created_at: String,
    /// Metadata
    pub metadata: HashMap<String, String>,
}

/// List summaries response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListSummariesResponse {
    /// List of summaries
    pub summaries: Vec<SummaryInfo>,
    /// Total count
    pub total_count: usize,
    /// Status
    pub status: String,
}

/// Indexing progress
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexingProgress {
    /// Whether indexing is in progress
    pub is_indexing: bool,
    /// Overall status
    pub overall_status: String,
    /// Collections being indexed
    pub collections: Vec<CollectionProgress>,
}

/// Collection progress
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollectionProgress {
    /// Collection name
    pub collection_name: String,
    /// Status
    pub status: String,
    /// Progress percentage
    pub progress: f32,
    /// Vector count
    pub vector_count: usize,
    /// Error message if any
    pub error_message: Option<String>,
    /// Last updated timestamp
    pub last_updated: String,
}

// ===== INTELLIGENT SEARCH MODELS =====

/// Intelligent search request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntelligentSearchRequest {
    /// Search query
    pub query: String,
    /// Collections to search (optional - searches all if not specified)
    pub collections: Option<Vec<String>>,
    /// Maximum number of results
    pub max_results: Option<usize>,
    /// Enable domain expansion
    pub domain_expansion: Option<bool>,
    /// Enable technical focus
    pub technical_focus: Option<bool>,
    /// Enable MMR diversification
    pub mmr_enabled: Option<bool>,
    /// MMR balance parameter (0.0-1.0)
    pub mmr_lambda: Option<f32>,
}

/// Semantic search request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SemanticSearchRequest {
    /// Search query
    pub query: String,
    /// Collection to search
    pub collection: String,
    /// Maximum number of results
    pub max_results: Option<usize>,
    /// Enable semantic reranking
    pub semantic_reranking: Option<bool>,
    /// Enable cross-encoder reranking
    pub cross_encoder_reranking: Option<bool>,
    /// Minimum similarity threshold
    pub similarity_threshold: Option<f32>,
}

/// Contextual search request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextualSearchRequest {
    /// Search query
    pub query: String,
    /// Collection to search
    pub collection: String,
    /// Metadata-based context filters
    pub context_filters: Option<HashMap<String, serde_json::Value>>,
    /// Maximum number of results
    pub max_results: Option<usize>,
    /// Enable context-aware reranking
    pub context_reranking: Option<bool>,
    /// Weight of context factors (0.0-1.0)
    pub context_weight: Option<f32>,
}

/// Multi-collection search request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MultiCollectionSearchRequest {
    /// Search query
    pub query: String,
    /// Collections to search
    pub collections: Vec<String>,
    /// Maximum results per collection
    pub max_per_collection: Option<usize>,
    /// Maximum total results
    pub max_total_results: Option<usize>,
    /// Enable cross-collection reranking
    pub cross_collection_reranking: Option<bool>,
}

/// Intelligent search result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntelligentSearchResult {
    /// Result ID
    pub id: String,
    /// Similarity score
    pub score: f32,
    /// Result content
    pub content: String,
    /// Metadata
    pub metadata: Option<HashMap<String, serde_json::Value>>,
    /// Collection name
    pub collection: Option<String>,
    /// Query used for this result
    pub query_used: Option<String>,
}

/// Intelligent search response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntelligentSearchResponse {
    /// Search results
    pub results: Vec<IntelligentSearchResult>,
    /// Total number of results found
    pub total_results: usize,
    /// Search duration in milliseconds
    pub duration_ms: u64,
    /// Queries generated
    pub queries_generated: Option<Vec<String>>,
    /// Collections searched
    pub collections_searched: Option<Vec<String>>,
    /// Search metadata
    pub metadata: Option<HashMap<String, serde_json::Value>>,
}

/// Semantic search response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SemanticSearchResponse {
    /// Search results
    pub results: Vec<IntelligentSearchResult>,
    /// Total number of results found
    pub total_results: usize,
    /// Search duration in milliseconds
    pub duration_ms: u64,
    /// Collection searched
    pub collection: String,
    /// Search metadata
    pub metadata: Option<HashMap<String, serde_json::Value>>,
}

/// Contextual search response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextualSearchResponse {
    /// Search results
    pub results: Vec<IntelligentSearchResult>,
    /// Total number of results found
    pub total_results: usize,
    /// Search duration in milliseconds
    pub duration_ms: u64,
    /// Collection searched
    pub collection: String,
    /// Context filters applied
    pub context_filters: Option<HashMap<String, serde_json::Value>>,
    /// Search metadata
    pub metadata: Option<HashMap<String, serde_json::Value>>,
}

/// Multi-collection search response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MultiCollectionSearchResponse {
    /// Search results
    pub results: Vec<IntelligentSearchResult>,
    /// Total number of results found
    pub total_results: usize,
    /// Search duration in milliseconds
    pub duration_ms: u64,
    /// Collections searched
    pub collections_searched: Vec<String>,
    /// Results per collection
    pub results_per_collection: Option<HashMap<String, usize>>,
    /// Search metadata
    pub metadata: Option<HashMap<String, serde_json::Value>>,
}

// ==================== REPLICATION MODELS ====================

/// Status of a replica node
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub enum ReplicaStatus {
    /// Replica is connected and healthy
    Connected,
    /// Replica is syncing data
    Syncing,
    /// Replica is lagging behind master
    Lagging,
    /// Replica is disconnected
    Disconnected,
}

/// Information about a replica node
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReplicaInfo {
    /// Unique identifier for the replica
    pub replica_id: String,
    /// Hostname or IP address of the replica
    pub host: String,
    /// Port number of the replica
    pub port: u16,
    /// Current status of the replica
    pub status: String,
    /// Timestamp of last heartbeat
    pub last_heartbeat: DateTime<Utc>,
    /// Number of operations successfully synced
    pub operations_synced: u64,

    // Legacy fields (backwards compatible)
    /// Legacy: Current offset on replica (deprecated, use operations_synced)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub offset: Option<u64>,
    /// Legacy: Lag in operations (deprecated, use status)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lag: Option<u64>,
}

/// Statistics for replication status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReplicationStats {
    // New fields (v1.2.0+)
    /// Role of the node: Master or Replica
    #[serde(skip_serializing_if = "Option::is_none")]
    pub role: Option<String>,
    /// Total bytes sent to replicas (Master only)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bytes_sent: Option<u64>,
    /// Total bytes received from master (Replica only)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bytes_received: Option<u64>,
    /// Timestamp of last synchronization
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_sync: Option<DateTime<Utc>>,
    /// Number of operations pending replication
    #[serde(skip_serializing_if = "Option::is_none")]
    pub operations_pending: Option<usize>,
    /// Size of snapshot data in bytes
    #[serde(skip_serializing_if = "Option::is_none")]
    pub snapshot_size: Option<usize>,
    /// Number of connected replicas (Master only)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub connected_replicas: Option<usize>,

    // Legacy fields (backwards compatible - always present)
    /// Current offset on master node
    pub master_offset: u64,
    /// Current offset on replica node
    pub replica_offset: u64,
    /// Number of operations behind
    pub lag_operations: u64,
    /// Total operations replicated
    pub total_replicated: u64,
}

/// Response for replication status endpoint
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReplicationStatusResponse {
    /// Overall status message
    pub status: String,
    /// Detailed replication statistics
    pub stats: ReplicationStats,
    /// Optional message with additional information
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
}

/// Response for listing replicas
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReplicaListResponse {
    /// List of replica nodes
    pub replicas: Vec<ReplicaInfo>,
    /// Total count of replicas
    pub count: usize,
    /// Status message
    pub message: String,
}
