//! Qdrant configuration models

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

/// Qdrant server configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QdrantServerConfig {
    /// Server host
    pub host: String,
    /// Server port
    pub port: u16,
    /// gRPC port
    pub grpc_port: Option<u16>,
    /// Enable CORS
    pub enable_cors: Option<bool>,
    /// Max request size
    pub max_request_size_mb: Option<u64>,
    /// Max batch size
    pub max_batch_size: Option<u64>,
    /// Storage configuration
    pub storage: Option<QdrantStorageConfig>,
    /// Service configuration
    pub service: Option<QdrantServiceConfig>,
    /// Cluster configuration
    pub cluster: Option<QdrantClusterConfig>,
}

/// Qdrant storage configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QdrantStorageConfig {
    /// Storage path
    pub storage_path: String,
    /// Temporary storage path
    pub temp_path: Option<String>,
    /// Snapshot path
    pub snapshots_path: Option<String>,
    /// Performance configuration
    pub performance: Option<QdrantPerformanceConfig>,
    /// WAL configuration
    pub wal: Option<QdrantWalConfig>,
    /// Optimizer configuration
    pub optimizers: Option<QdrantOptimizerConfig>,
    /// Quantization configuration
    pub quantization: Option<QdrantQuantizationConfig>,
}

/// Qdrant performance configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QdrantPerformanceConfig {
    /// Max segment size
    pub max_segment_size: Option<u64>,
    /// Memmap threshold
    pub memmap_threshold: Option<u64>,
    /// Indexing threshold
    pub indexing_threshold: Option<u64>,
    /// Payload indexing threshold
    pub payload_indexing_threshold: Option<u64>,
    /// Flush interval seconds
    pub flush_interval_sec: Option<u64>,
    /// Max optimization threads
    pub max_optimization_threads: Option<u32>,
}

/// Qdrant service configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QdrantServiceConfig {
    /// Enable telemetry
    pub enable_telemetry: Option<bool>,
    /// Telemetry host
    pub telemetry_host: Option<String>,
    /// Telemetry port
    pub telemetry_port: Option<u16>,
    /// Telemetry gRPC port
    pub telemetry_grpc_port: Option<u16>,
    /// Max request size
    pub max_request_size_mb: Option<u64>,
    /// Max batch size
    pub max_batch_size: Option<u64>,
    /// Max workers
    pub max_workers: Option<u32>,
}

/// Qdrant cluster configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QdrantClusterConfig {
    /// Enable cluster mode
    pub enabled: bool,
    /// Cluster configuration
    pub config: QdrantClusterConfigDetails,
}

/// Qdrant cluster configuration details
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QdrantClusterConfigDetails {
    /// P2P configuration
    pub p2p: QdrantP2PConfig,
    /// Consensus configuration
    pub consensus: QdrantConsensusConfig,
}

/// Qdrant P2P configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QdrantP2PConfig {
    /// Port for P2P communication
    pub port: u16,
    /// Connection pool size
    pub connection_pool_size: Option<u32>,
    /// Max message size
    pub max_message_size: Option<u64>,
    /// Max concurrent streams
    pub max_concurrent_streams: Option<u32>,
}

/// Qdrant consensus configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QdrantConsensusConfig {
    /// Tick period milliseconds
    pub tick_period_ms: Option<u64>,
    /// Bootstrap timeout seconds
    pub bootstrap_timeout_sec: Option<u64>,
    /// Max message batch size
    pub max_message_batch_size: Option<u32>,
}

/// Qdrant collection configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QdrantCollectionConfig {
    /// Vector parameters
    pub params: QdrantCollectionParams,
    /// HNSW configuration
    pub hnsw_config: QdrantHnswConfig,
    /// Optimizer configuration
    pub optimizer_config: QdrantOptimizerConfig,
    /// WAL configuration
    pub wal_config: QdrantWalConfig,
    /// Quantization configuration
    pub quantization_config: Option<QdrantQuantizationConfig>,
}

/// Qdrant collection parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QdrantCollectionParams {
    /// Vector size
    pub vectors: QdrantVectorsConfig,
    /// Shard number
    pub shard_number: Option<u32>,
    /// Replication factor
    pub replication_factor: Option<u32>,
    /// Write consistency factor
    pub write_consistency_factor: Option<u32>,
    /// On disk payload
    pub on_disk_payload: Option<bool>,
}

/// Qdrant vectors configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum QdrantVectorsConfig {
    /// Single vector configuration
    Single(QdrantVectorParams),
    /// Multiple named vector configurations
    Multiple(HashMap<String, QdrantVectorParams>),
}

/// Qdrant vector parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QdrantVectorParams {
    /// Vector size
    pub size: u64,
    /// Distance metric
    pub distance: QdrantDistance,
}

/// Qdrant distance metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum QdrantDistance {
    /// Cosine similarity
    Cosine,
    /// Dot product
    Dot,
    /// Euclidean distance
    Euclid,
}

/// Qdrant HNSW configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QdrantHnswConfig {
    /// M parameter
    pub m: Option<u32>,
    /// EF construct parameter
    pub ef_construct: Option<u32>,
    /// Full scan threshold
    pub full_scan_threshold: Option<u32>,
    /// Max indexing threads
    pub max_indexing_threads: Option<u32>,
    /// On disk flag
    pub on_disk: Option<bool>,
    /// Payload M parameter
    pub payload_m: Option<u32>,
}

/// Qdrant optimizer configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QdrantOptimizerConfig {
    /// Deleted threshold
    pub deleted_threshold: f64,
    /// Vacuum min vector number
    pub vacuum_min_vector_number: u64,
    /// Default segment number
    pub default_segment_number: u64,
    /// Max segment size
    pub max_segment_size: Option<u64>,
    /// Memmap threshold
    pub memmap_threshold: Option<u64>,
    /// Indexing threshold
    pub indexing_threshold: Option<u64>,
    /// Flush interval seconds
    pub flush_interval_sec: u64,
    /// Max optimization threads
    pub max_optimization_threads: Option<u32>,
}

/// Qdrant WAL configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QdrantWalConfig {
    /// WAL capacity MB
    pub wal_capacity_mb: Option<u64>,
    /// WAL segments ahead
    pub wal_segments_ahead: Option<u32>,
}

/// Qdrant quantization configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QdrantQuantizationConfig {
    /// Quantization type
    pub scalar: QdrantScalarQuantization,
}

/// Qdrant scalar quantization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QdrantScalarQuantization {
    /// Quantization type
    pub r#type: QdrantQuantizationType,
    /// Quantile
    pub quantile: Option<f32>,
    /// Always RAM
    pub always_ram: Option<bool>,
}

/// Qdrant quantization types
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum QdrantQuantizationType {
    /// Int8 quantization
    Int8,
}

/// Qdrant client configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QdrantClientConfig {
    /// Server URL
    pub url: String,
    /// API key
    pub api_key: Option<String>,
    /// Timeout seconds
    pub timeout: Option<u64>,
    /// Connection timeout seconds
    pub connection_timeout: Option<u64>,
    /// Retry attempts
    pub retry_attempts: Option<u32>,
    /// Retry delay milliseconds
    pub retry_delay_ms: Option<u64>,
    /// Enable TLS
    pub enable_tls: Option<bool>,
    /// TLS certificate path
    pub tls_cert_path: Option<String>,
    /// TLS key path
    pub tls_key_path: Option<String>,
    /// TLS CA path
    pub tls_ca_path: Option<String>,
}

/// Qdrant configuration builder
#[derive(Debug, Clone)]
pub struct QdrantConfigBuilder {
    config: QdrantServerConfig,
}

impl QdrantConfigBuilder {
    /// Create a new config builder
    pub fn new(host: &str, port: u16) -> Self {
        Self {
            config: QdrantServerConfig {
                host: host.to_string(),
                port,
                grpc_port: None,
                enable_cors: None,
                max_request_size_mb: None,
                max_batch_size: None,
                storage: None,
                service: None,
                cluster: None,
            },
        }
    }

    /// Set gRPC port
    pub fn grpc_port(mut self, port: u16) -> Self {
        self.config.grpc_port = Some(port);
        self
    }

    /// Enable CORS
    pub fn enable_cors(mut self, enable: bool) -> Self {
        self.config.enable_cors = Some(enable);
        self
    }

    /// Set max request size
    pub fn max_request_size_mb(mut self, size: u64) -> Self {
        self.config.max_request_size_mb = Some(size);
        self
    }

    /// Set max batch size
    pub fn max_batch_size(mut self, size: u64) -> Self {
        self.config.max_batch_size = Some(size);
        self
    }

    /// Set storage configuration
    pub fn storage(mut self, storage: QdrantStorageConfig) -> Self {
        self.config.storage = Some(storage);
        self
    }

    /// Set service configuration
    pub fn service(mut self, service: QdrantServiceConfig) -> Self {
        self.config.service = Some(service);
        self
    }

    /// Set cluster configuration
    pub fn cluster(mut self, cluster: QdrantClusterConfig) -> Self {
        self.config.cluster = Some(cluster);
        self
    }

    /// Build the configuration
    pub fn build(self) -> QdrantServerConfig {
        self.config
    }
}

impl Default for QdrantConfigBuilder {
    fn default() -> Self {
        Self::new("127.0.0.1", 6333)
    }
}

/// Helper functions for creating common configurations
impl QdrantServerConfig {
    /// Create a default configuration
    pub fn default() -> Self {
        QdrantConfigBuilder::default().build()
    }

    /// Create a development configuration
    pub fn development() -> Self {
        QdrantConfigBuilder::new("127.0.0.1", 6333)
            .enable_cors(true)
            .max_request_size_mb(64)
            .max_batch_size(1000)
            .build()
    }

    /// Create a production configuration
    pub fn production() -> Self {
        QdrantConfigBuilder::new("0.0.0.0", 6333)
            .grpc_port(6334)
            .enable_cors(false)
            .max_request_size_mb(256)
            .max_batch_size(10000)
            .build()
    }
}

/// Helper functions for creating storage configurations
impl QdrantStorageConfig {
    /// Create a default storage configuration
    pub fn default() -> Self {
        Self {
            storage_path: "./storage".to_string(),
            temp_path: None,
            snapshots_path: None,
            performance: None,
            wal: None,
            optimizers: None,
            quantization: None,
        }
    }

    /// Create a high-performance storage configuration
    pub fn high_performance() -> Self {
        Self {
            storage_path: "./storage".to_string(),
            temp_path: Some("./temp".to_string()),
            snapshots_path: Some("./snapshots".to_string()),
            performance: Some(QdrantPerformanceConfig {
                max_segment_size: Some(100_000),
                memmap_threshold: Some(50_000),
                indexing_threshold: Some(20_000),
                payload_indexing_threshold: Some(10_000),
                flush_interval_sec: Some(5),
                max_optimization_threads: Some(4),
            }),
            wal: Some(QdrantWalConfig {
                wal_capacity_mb: Some(1024),
                wal_segments_ahead: Some(2),
            }),
            optimizers: Some(QdrantOptimizerConfig {
                deleted_threshold: 0.2,
                vacuum_min_vector_number: 1000,
                default_segment_number: 2,
                max_segment_size: Some(100_000),
                memmap_threshold: Some(50_000),
                indexing_threshold: Some(20_000),
                flush_interval_sec: 5,
                max_optimization_threads: Some(4),
            }),
            quantization: None,
        }
    }
}

/// Helper functions for creating service configurations
impl QdrantServiceConfig {
    /// Create a default service configuration
    pub fn default() -> Self {
        Self {
            enable_telemetry: Some(false),
            telemetry_host: None,
            telemetry_port: None,
            telemetry_grpc_port: None,
            max_request_size_mb: Some(64),
            max_batch_size: Some(1000),
            max_workers: Some(4),
        }
    }

    /// Create a production service configuration
    pub fn production() -> Self {
        Self {
            enable_telemetry: Some(true),
            telemetry_host: Some("0.0.0.0".to_string()),
            telemetry_port: Some(6335),
            telemetry_grpc_port: Some(6336),
            max_request_size_mb: Some(256),
            max_batch_size: Some(10000),
            max_workers: Some(16),
        }
    }
}
