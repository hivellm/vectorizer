//! Qdrant collection models
//!
//! This module provides data structures for Qdrant collection management,
//! including collection info, configuration, and status.

use std::collections::HashMap;

use serde::{Deserialize, Deserializer, Serialize};

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

/// Collection configuration.
///
/// Matches Qdrant's upstream REST schema: only `vectors` is required;
/// every other field defaults server-side so real Qdrant clients
/// (qdrant-client-python, qdrant-client-js, ...) can send the minimal
/// `{vectors: {size, distance}}` shape the upstream docs prescribe.
/// See `phase8_qdrant-compat-minimal-request-shape` (probe 3.6).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QdrantCollectionConfig {
    /// Vector parameters (the only required field).
    pub vectors: QdrantVectorsConfig,
    /// Shard number. Defaults to 1 when absent.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub shard_number: Option<u32>,
    /// Replication factor. Defaults to 1 when absent.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub replication_factor: Option<u32>,
    /// Write consistency factor. Defaults to 1 when absent.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub write_consistency_factor: Option<u32>,
    /// On disk payload. Defaults to false when absent.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub on_disk_payload: Option<bool>,
    /// HNSW configuration. Defaults server-side when absent.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub hnsw_config: Option<QdrantHnswConfig>,
    /// Optimizer configuration. Defaults server-side when absent.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub optimizer_config: Option<QdrantOptimizerConfig>,
    /// WAL configuration. Defaults server-side when absent.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub wal_config: Option<QdrantWalConfig>,
    /// Quantization configuration. Optional — server defaults to SQ(8).
    #[serde(default, skip_serializing_if = "Option::is_none")]
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

/// HNSW configuration. Defaults match Qdrant's upstream REST spec
/// (`m=16`, `ef_construct=100`, `full_scan_threshold=10000`,
/// `max_indexing_threads=0` meaning "auto", `on_disk=false`) so a
/// Qdrant-native `create_collection({vectors: {...}})` call resolves
/// to a sane index without the client having to set everything.
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

impl Default for QdrantHnswConfig {
    fn default() -> Self {
        Self {
            m: 16,
            ef_construct: 100,
            full_scan_threshold: 10_000,
            max_indexing_threads: None,
            on_disk: None,
        }
    }
}

/// Optimizer configuration. Defaults match Qdrant's upstream REST
/// spec so a Qdrant-native request without `optimizer_config`
/// resolves to the same values the upstream server would apply.
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

impl Default for QdrantOptimizerConfig {
    fn default() -> Self {
        Self {
            deleted_threshold: 0.2,
            vacuum_min_vector_number: 1_000,
            default_segment_number: 0, // 0 = auto
            max_segment_size: None,
            memmap_threshold: None,
            indexing_threshold: Some(20_000),
            flush_interval_sec: 5,
            max_optimization_threads: None,
        }
    }
}

/// WAL configuration. Defaults match Qdrant's upstream REST spec
/// (`wal_capacity_mb=32`, `wal_segments_ahead=0`).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QdrantWalConfig {
    /// WAL capacity MB
    pub wal_capacity_mb: u32,
    /// WAL segments ahead
    pub wal_segments_ahead: u32,
}

impl Default for QdrantWalConfig {
    fn default() -> Self {
        Self {
            wal_capacity_mb: 32,
            wal_segments_ahead: 0,
        }
    }
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

/// Collection creation request.
///
/// Accepts both the historic wrapped shape (`{"config": {"vectors":
/// ...}}`) and the minimal Qdrant-native shape (`{"vectors": ...}`).
/// qdrant-client-python / qdrant-client-js / the Qdrant REST docs
/// all send the flat form; earlier Vectorizer releases required the
/// wrapped form, so real Qdrant clients failed at collection
/// creation with a 422 about the missing `config` field. See
/// `phase8_qdrant-compat-minimal-request-shape` (probe 3.6). The
/// handler always reads the parsed value through `.config`.
#[derive(Debug, Clone, Serialize)]
pub struct QdrantCreateCollectionRequest {
    /// Collection configuration. Populated from either the wrapped
    /// or flat request shape.
    pub config: QdrantCollectionConfig,
}

impl<'de> Deserialize<'de> for QdrantCreateCollectionRequest {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = serde_json::Value::deserialize(deserializer)?;
        let config = if let Some(nested) = value.get("config") {
            // Wrapped shape: {"config": {"vectors": ...}}
            serde_json::from_value::<QdrantCollectionConfig>(nested.clone())
                .map_err(serde::de::Error::custom)?
        } else {
            // Flat shape: {"vectors": ..., ...} — the whole body IS the config.
            serde_json::from_value::<QdrantCollectionConfig>(value)
                .map_err(serde::de::Error::custom)?
        };
        Ok(QdrantCreateCollectionRequest { config })
    }
}

/// Collection update request. Same dual-shape parsing as
/// `QdrantCreateCollectionRequest` so a client may send either
/// `{"config": {...}}` or the top-level fields directly.
#[derive(Debug, Clone, Serialize)]
pub struct QdrantUpdateCollectionRequest {
    /// Collection configuration updates.
    pub config: QdrantCollectionConfig,
}

impl<'de> Deserialize<'de> for QdrantUpdateCollectionRequest {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = serde_json::Value::deserialize(deserializer)?;
        let config = if let Some(nested) = value.get("config") {
            serde_json::from_value::<QdrantCollectionConfig>(nested.clone())
                .map_err(serde::de::Error::custom)?
        } else {
            serde_json::from_value::<QdrantCollectionConfig>(value)
                .map_err(serde::de::Error::custom)?
        };
        Ok(QdrantUpdateCollectionRequest { config })
    }
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

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;

    /// The minimal Qdrant-native create-collection payload must parse
    /// without requiring the historic `config:` wrapper. Regression
    /// guard for probe 3.6 / phase8_qdrant-compat-minimal-request-shape.
    #[test]
    fn flat_create_collection_request_parses_minimal_body() {
        let body = r#"{"vectors": {"size": 4, "distance": "Cosine"}}"#;
        let parsed: QdrantCreateCollectionRequest = serde_json::from_str(body)
            .expect("flat {vectors: ...} shape must deserialize without a `config` wrapper");
        assert_eq!(parsed.config.vectors.size, 4);
        assert!(matches!(
            parsed.config.vectors.distance,
            QdrantDistance::Cosine
        ));
        assert!(parsed.config.shard_number.is_none());
        assert!(parsed.config.hnsw_config.is_none());
        assert!(parsed.config.optimizer_config.is_none());
        assert!(parsed.config.wal_config.is_none());
    }

    /// The wrapped shape used by earlier Vectorizer releases must
    /// keep parsing so existing operator scripts and SDK tests do not
    /// regress when the flat shape is accepted.
    #[test]
    fn wrapped_create_collection_request_still_parses() {
        let body = r#"{"config": {"vectors": {"size": 8, "distance": "Euclid"}}}"#;
        let parsed: QdrantCreateCollectionRequest = serde_json::from_str(body)
            .expect("wrapped {config: {vectors: ...}} shape must keep deserializing");
        assert_eq!(parsed.config.vectors.size, 8);
        assert!(matches!(
            parsed.config.vectors.distance,
            QdrantDistance::Euclid
        ));
    }

    /// A partially-populated flat request should fill the absent
    /// blocks with `None` so the handler can resolve upstream defaults.
    #[test]
    fn flat_create_collection_request_tolerates_partial_blocks() {
        let body = r#"{
            "vectors": {"size": 16, "distance": "Dot"},
            "shard_number": 3,
            "hnsw_config": {"m": 32, "ef_construct": 200, "full_scan_threshold": 5000}
        }"#;
        let parsed: QdrantCreateCollectionRequest =
            serde_json::from_str(body).expect("partially-populated flat request must deserialize");
        assert_eq!(parsed.config.shard_number, Some(3));
        let hnsw = parsed
            .config
            .hnsw_config
            .as_ref()
            .expect("hnsw_config was present in the body");
        assert_eq!(hnsw.m, 32);
        assert_eq!(hnsw.ef_construct, 200);
        assert!(parsed.config.replication_factor.is_none());
        assert!(parsed.config.wal_config.is_none());
    }

    /// `QdrantUpdateCollectionRequest` shares the dual-shape parser;
    /// both forms must deserialize.
    #[test]
    fn update_collection_request_accepts_both_shapes() {
        let flat = r#"{"vectors": {"size": 4, "distance": "Cosine"}}"#;
        let wrapped = r#"{"config": {"vectors": {"size": 4, "distance": "Cosine"}}}"#;
        let _: QdrantUpdateCollectionRequest = serde_json::from_str(flat).unwrap();
        let _: QdrantUpdateCollectionRequest = serde_json::from_str(wrapped).unwrap();
    }

    /// The three sub-config Default impls must match Qdrant's upstream
    /// REST spec so a minimal request resolves to the same server-side
    /// config the upstream server would apply.
    #[test]
    fn subconfig_defaults_match_qdrant_upstream_spec() {
        let hnsw = QdrantHnswConfig::default();
        assert_eq!(hnsw.m, 16);
        assert_eq!(hnsw.ef_construct, 100);
        assert_eq!(hnsw.full_scan_threshold, 10_000);

        let opt = QdrantOptimizerConfig::default();
        assert_eq!(opt.flush_interval_sec, 5);
        assert_eq!(opt.indexing_threshold, Some(20_000));

        let wal = QdrantWalConfig::default();
        assert_eq!(wal.wal_capacity_mb, 32);
        assert_eq!(wal.wal_segments_ahead, 0);
    }
}
