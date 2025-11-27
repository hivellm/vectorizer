//! Qdrant collection models
//!
//! This module provides data structures for Qdrant collection management,
//! including collection info, configuration, and status.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

/// Qdrant collection information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QdrantCollectionInfo {
    /// Collection name
    pub name: String,
    /// Collection status
    pub status: QdrantCollectionStatus,
    /// Collection configuration
    pub config: QdrantCollectionConfig,
    /// Collection statistics
    pub stats: QdrantCollectionStats,
    /// Collection optimizer status
    pub optimizer_status: QdrantOptimizerStatus,
    /// Collection payload schema
    pub payload_schema: HashMap<String, QdrantPayloadSchema>,
}

/// Collection status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum QdrantCollectionStatus {
    /// Collection is being created
    #[serde(rename = "creating")]
    Creating,
    /// Collection is ready for use
    #[serde(rename = "green")]
    Green,
    /// Collection is being optimized
    #[serde(rename = "yellow")]
    Yellow,
    /// Collection has issues
    #[serde(rename = "red")]
    Red,
}

/// Collection configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QdrantCollectionConfig {
    /// Vector parameters
    pub vectors: QdrantVectorsConfig,
    /// Shard number
    pub shard_number: u32,
    /// Replication factor
    pub replication_factor: u32,
    /// Write consistency factor
    pub write_consistency_factor: u32,
    /// On disk payload
    pub on_disk_payload: bool,
    /// Distance metric
    pub distance: QdrantDistance,
    /// HNSW configuration
    pub hnsw_config: QdrantHnswConfig,
    /// Optimizer configuration
    pub optimizer_config: QdrantOptimizerConfig,
    /// Wal configuration
    pub wal_config: QdrantWalConfig,
    /// Quantization configuration
    pub quantization_config: Option<QdrantQuantizationConfig>,
}

/// Vector configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QdrantVectorsConfig {
    /// Vector size
    pub size: usize,
    /// Distance metric
    pub distance: QdrantDistance,
}

/// Distance metric
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum QdrantDistance {
    /// Cosine similarity
    #[serde(rename = "Cosine")]
    Cosine,
    /// Euclidean distance
    #[serde(rename = "Euclid")]
    Euclid,
    /// Dot product
    #[serde(rename = "Dot")]
    Dot,
}

/// HNSW configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QdrantHnswConfig {
    /// M parameter
    pub m: u32,
    /// Ef construct
    pub ef_construct: u32,
    /// Full scan threshold
    pub full_scan_threshold: u32,
    /// Max indexing threads
    pub max_indexing_threads: Option<u32>,
    /// On disk
    pub on_disk: Option<bool>,
}

/// Optimizer configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QdrantOptimizerConfig {
    /// Deleted threshold
    pub deleted_threshold: f64,
    /// Vacuum min vector number
    pub vacuum_min_vector_number: u32,
    /// Default segment number
    pub default_segment_number: u32,
    /// Max segment size
    pub max_segment_size: Option<u32>,
    /// Memmap threshold
    pub memmap_threshold: Option<u32>,
    /// Indexing threshold
    pub indexing_threshold: Option<u32>,
    /// Flush interval seconds
    pub flush_interval_sec: u32,
    /// Max optimization threads
    pub max_optimization_threads: Option<u32>,
}

/// WAL configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QdrantWalConfig {
    /// WAL capacity MB
    pub wal_capacity_mb: u32,
    /// WAL segments ahead
    pub wal_segments_ahead: u32,
}

/// Quantization configuration (supports Scalar, Product, and Binary quantization)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum QdrantQuantizationConfig {
    /// Scalar quantization configuration
    Scalar(QdrantScalarQuantizationConfig),
    /// Product quantization configuration
    Product(QdrantProductQuantizationConfig),
    /// Binary quantization configuration
    Binary(QdrantBinaryQuantizationConfig),
}

/// Wrapper for scalar quantization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QdrantScalarQuantizationConfig {
    /// Scalar quantization parameters
    pub scalar: QdrantScalarQuantization,
}

/// Wrapper for product quantization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QdrantProductQuantizationConfig {
    /// Product quantization parameters
    pub product: QdrantProductQuantization,
}

/// Wrapper for binary quantization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QdrantBinaryQuantizationConfig {
    /// Binary quantization parameters
    pub binary: QdrantBinaryQuantization,
}

/// Quantization type for scalar quantization
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum QdrantScalarQuantizationType {
    /// 8-bit integer quantization
    Int8,
}

/// Scalar quantization parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QdrantScalarQuantization {
    /// Quantization type (int8)
    pub r#type: QdrantScalarQuantizationType,
    /// Quantile for quantization (0.0-1.0, typically 0.99)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub quantile: Option<f32>,
    /// Always keep quantized vectors in RAM
    #[serde(skip_serializing_if = "Option::is_none")]
    pub always_ram: Option<bool>,
}

/// Product quantization parameters (PQ)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QdrantProductQuantization {
    /// Compression ratio (e.g., x4, x8, x16, x32, x64)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub compression: Option<QdrantPQCompression>,
    /// Always keep quantized vectors in RAM
    #[serde(skip_serializing_if = "Option::is_none")]
    pub always_ram: Option<bool>,
}

/// PQ compression ratio
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum QdrantPQCompression {
    /// 4x compression
    #[serde(rename = "x4")]
    X4,
    /// 8x compression
    #[serde(rename = "x8")]
    X8,
    /// 16x compression
    #[serde(rename = "x16")]
    X16,
    /// 32x compression
    #[serde(rename = "x32")]
    X32,
    /// 64x compression
    #[serde(rename = "x64")]
    X64,
}

/// Binary quantization parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QdrantBinaryQuantization {
    /// Always keep quantized vectors in RAM
    #[serde(skip_serializing_if = "Option::is_none")]
    pub always_ram: Option<bool>,
}

/// Collection statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QdrantCollectionStats {
    /// Number of points
    pub points_count: u64,
    /// Number of indexed vectors
    pub indexed_vectors_count: u64,
    /// Number of segments
    pub segments_count: u32,
    /// Segment statistics
    pub segments: Vec<QdrantSegmentInfo>,
}

/// Segment information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QdrantSegmentInfo {
    /// Segment number
    pub segment_num: u32,
    /// Number of points
    pub num_vectors: u64,
    /// Number of indexed vectors
    pub num_indexed_vectors: u64,
    /// Number of points with payload
    pub num_payloads: u64,
    /// Number of deleted points
    pub num_deleted_vectors: u64,
    /// RAM usage bytes
    pub ram_usage_bytes: u64,
    /// Disk usage bytes
    pub disk_usage_bytes: u64,
}

/// Optimizer status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QdrantOptimizerStatus {
    /// Whether optimization is running
    pub ok: bool,
    /// Error message if optimization failed
    pub error: Option<String>,
}

/// Payload schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QdrantPayloadSchema {
    /// Data type
    pub data_type: QdrantPayloadDataType,
    /// Whether field is indexed
    pub indexed: bool,
}

/// Payload data type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum QdrantPayloadDataType {
    /// Keyword type
    #[serde(rename = "keyword")]
    Keyword,
    /// Integer type
    #[serde(rename = "integer")]
    Integer,
    /// Float type
    #[serde(rename = "float")]
    Float,
    /// Boolean type
    #[serde(rename = "bool")]
    Bool,
    /// Geo point type
    #[serde(rename = "geo")]
    Geo,
    /// Text type
    #[serde(rename = "text")]
    Text,
}

/// Collection creation request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QdrantCreateCollectionRequest {
    /// Collection configuration
    pub config: QdrantCollectionConfig,
}

/// Collection update request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QdrantUpdateCollectionRequest {
    /// Collection configuration updates
    pub config: QdrantCollectionConfig,
}

/// Collection list response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QdrantCollectionListResponse {
    /// List of collections
    pub collections: Vec<QdrantCollectionInfo>,
}

/// Collection info response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QdrantCollectionInfoResponse {
    /// Collection information
    pub result: QdrantCollectionInfo,
}

/// Collection statistics response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QdrantCollectionStatsResponse {
    /// Collection statistics
    pub result: QdrantCollectionStats,
}

/// Collection response (alias for compatibility)
pub type QdrantCollectionResponse = QdrantCollectionInfoResponse;
