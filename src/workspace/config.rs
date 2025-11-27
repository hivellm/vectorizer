//! Workspace configuration structures
//!
//! Defines the data structures for workspace configuration

use std::collections::HashMap;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

/// Workspace configuration root structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceConfig {
    /// Workspace metadata
    pub workspace: WorkspaceMetadata,

    /// Global settings applied to all projects
    pub global: GlobalSettings,

    /// List of projects in the workspace
    pub projects: Vec<ProjectConfig>,

    /// Workspace processing settings
    pub processing: ProcessingSettings,

    /// Workspace monitoring settings
    pub monitoring: MonitoringSettings,

    /// Workspace validation settings
    pub validation: ValidationSettings,

    /// File watcher configuration
    pub file_watcher: Option<crate::config::FileWatcherYamlConfig>,
}

/// Workspace metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceMetadata {
    /// Workspace name
    pub name: String,

    /// Workspace version
    pub version: String,

    /// Workspace description
    pub description: String,

    /// Creation timestamp
    pub created_at: String,

    /// Last update timestamp
    pub last_updated: String,
}

/// Global settings for all projects
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GlobalSettings {
    /// Default embedding configuration
    pub default_embedding: EmbeddingConfig,

    /// Default collection settings
    pub default_collection: CollectionDefaults,

    /// Default indexing settings
    pub default_indexing: IndexingDefaults,

    /// Processing settings
    pub processing: ProcessingDefaults,
}

/// Project configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectConfig {
    /// Project name (unique identifier)
    pub name: String,

    /// Project path (relative to workspace root)
    pub path: PathBuf,

    /// Project description
    pub description: String,

    /// Whether this project is enabled
    pub enabled: bool,

    /// Project-specific embedding configuration (overrides global)
    pub embedding: Option<EmbeddingConfig>,

    /// Collections for this project
    pub collections: Vec<CollectionConfig>,
}

/// Collection configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollectionConfig {
    /// Collection name
    pub name: String,

    /// Collection description
    pub description: String,

    /// Vector dimension
    pub dimension: usize,

    /// Distance metric
    pub metric: DistanceMetric,

    /// Collection-specific embedding configuration
    pub embedding: EmbeddingConfig,

    /// Indexing configuration
    pub indexing: IndexingConfig,

    /// Processing configuration
    pub processing: CollectionProcessing,
}

/// Embedding configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbeddingConfig {
    /// Embedding model type
    pub model: EmbeddingModel,

    /// Vector dimension
    pub dimension: usize,

    /// Model-specific parameters
    pub parameters: HashMap<String, serde_json::Value>,
}

/// Embedding model types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum EmbeddingModel {
    /// TF-IDF embedding
    #[serde(rename = "tfidf")]
    TfIdf,

    /// BM25 sparse retrieval
    #[serde(rename = "bm25")]
    Bm25,

    /// SVD embedding
    #[serde(rename = "svd")]
    Svd,

    /// BERT embedding
    #[serde(rename = "bert")]
    Bert,

    /// MiniLM embedding
    #[serde(rename = "minilm")]
    MiniLm,

    /// Bag of Words embedding
    #[serde(rename = "bagofwords")]
    BagOfWords,

    /// Character N-gram embedding
    #[serde(rename = "charngram")]
    CharNGram,

    /// Real transformer model (Candle)
    #[serde(rename = "real_model")]
    RealModel,

    /// ONNX model
    #[serde(rename = "onnx_model")]
    OnnxModel,
}

/// Distance metrics
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum DistanceMetric {
    /// Cosine similarity
    #[serde(rename = "cosine")]
    Cosine,

    /// Euclidean distance
    #[serde(rename = "euclidean")]
    Euclidean,

    /// Dot product
    #[serde(rename = "dot_product")]
    DotProduct,
}

/// Collection defaults
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollectionDefaults {
    /// Default distance metric
    pub metric: DistanceMetric,

    /// Quantization settings
    pub quantization: Option<QuantizationDefaults>,

    /// Compression settings
    pub compression: CompressionConfig,
}

/// Quantization defaults
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuantizationDefaults {
    /// Quantization type
    #[serde(rename = "type")]
    pub quantization_type: String,

    /// Number of bits for scalar quantization
    pub bits: usize,
}

/// Compression configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompressionConfig {
    /// Whether compression is enabled
    pub enabled: bool,

    /// Compression threshold in bytes
    pub threshold_bytes: usize,

    /// Compression algorithm
    pub algorithm: String,
}

/// Indexing defaults
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexingDefaults {
    /// Index type
    pub index_type: String,

    /// Index parameters
    pub parameters: HashMap<String, serde_json::Value>,
}

/// Indexing configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexingConfig {
    /// Index type
    pub index_type: String,

    /// Index parameters
    pub parameters: HashMap<String, serde_json::Value>,
}

/// Processing defaults
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessingDefaults {
    /// Default chunk size
    pub chunk_size: usize,

    /// Default chunk overlap
    pub chunk_overlap: usize,

    /// Maximum file size in MB
    pub max_file_size_mb: usize,

    /// Supported file extensions
    pub supported_extensions: Vec<String>,
}

/// Collection processing configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollectionProcessing {
    /// Chunk size for this collection
    pub chunk_size: usize,

    /// Chunk overlap for this collection
    pub chunk_overlap: usize,

    /// File patterns to include
    pub include_patterns: Vec<String>,

    /// File patterns to exclude
    pub exclude_patterns: Vec<String>,
}

/// Processing settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessingSettings {
    /// Whether to use parallel processing
    pub parallel_processing: bool,

    /// Maximum concurrent projects
    pub max_concurrent_projects: usize,

    /// Maximum concurrent collections
    pub max_concurrent_collections: usize,

    /// File processing settings
    pub file_processing: FileProcessingSettings,

    /// Memory management settings
    pub memory: MemorySettings,

    /// Error handling settings
    pub error_handling: ErrorHandlingSettings,
}

/// File processing settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileProcessingSettings {
    /// Batch size for processing
    pub batch_size: usize,

    /// Maximum file size in MB
    pub max_file_size_mb: usize,

    /// Skip hidden files
    pub skip_hidden_files: bool,

    /// Skip binary files
    pub skip_binary_files: bool,
}

/// Memory settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemorySettings {
    /// Maximum memory usage in GB
    pub max_memory_usage_gb: f64,

    /// Garbage collection threshold in MB
    pub gc_threshold_mb: usize,
}

/// Error handling settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorHandlingSettings {
    /// Maximum retries
    pub max_retries: usize,

    /// Retry delay in seconds
    pub retry_delay_seconds: usize,

    /// Continue processing on error
    pub continue_on_error: bool,

    /// Log errors
    pub log_errors: bool,
}

/// Monitoring settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitoringSettings {
    /// Health check settings
    pub health_check: HealthCheckSettings,

    /// Metrics collection settings
    pub metrics: MetricsSettings,

    /// Logging settings
    pub logging: LoggingSettings,
}

/// Health check settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthCheckSettings {
    /// Whether health checks are enabled
    pub enabled: bool,

    /// Health check interval in seconds
    pub interval_seconds: usize,

    /// Check projects health
    pub check_projects: bool,

    /// Check collections health
    pub check_collections: bool,
}

/// Metrics settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricsSettings {
    /// Whether metrics collection is enabled
    pub enabled: bool,

    /// Metrics collection interval in seconds
    pub collection_interval_seconds: usize,

    /// Project metrics to collect
    pub project_metrics: Vec<String>,

    /// Collection metrics to collect
    pub collection_metrics: Vec<String>,
}

/// Logging settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingSettings {
    /// Log level
    pub level: String,

    /// Log file path
    pub log_file: String,

    /// Maximum log file size in MB
    pub max_log_size_mb: usize,

    /// Maximum number of log files
    pub max_log_files: usize,
}

/// Validation settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationSettings {
    /// Path validation settings
    pub paths: PathValidationSettings,

    /// Configuration validation settings
    pub config: ConfigValidationSettings,

    /// Data validation settings
    pub data: DataValidationSettings,
}

/// Path validation settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PathValidationSettings {
    /// Validate path existence
    pub validate_existence: bool,

    /// Validate path permissions
    pub validate_permissions: bool,

    /// Create missing directories
    pub create_missing_dirs: bool,
}

/// Configuration validation settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigValidationSettings {
    /// Validate embedding models
    pub validate_embedding_models: bool,

    /// Validate dimensions
    pub validate_dimensions: bool,

    /// Validate collections
    pub validate_collections: bool,
}

/// Data validation settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataValidationSettings {
    /// Validate file types
    pub validate_file_types: bool,

    /// Validate file sizes
    pub validate_file_sizes: bool,

    /// Validate file encoding
    pub validate_encoding: bool,
}

impl Default for WorkspaceConfig {
    fn default() -> Self {
        Self {
            workspace: WorkspaceMetadata {
                name: "Default Workspace".to_string(),
                version: "1.0.0".to_string(),
                description: "Default workspace configuration".to_string(),
                created_at: chrono::Utc::now().to_rfc3339(),
                last_updated: chrono::Utc::now().to_rfc3339(),
            },
            global: GlobalSettings {
                default_embedding: EmbeddingConfig {
                    model: EmbeddingModel::Bm25,
                    dimension: 384,
                    parameters: HashMap::new(),
                },
                default_collection: CollectionDefaults {
                    metric: DistanceMetric::Cosine,
                    quantization: Some(QuantizationDefaults {
                        quantization_type: "sq".to_string(),
                        bits: 8,
                    }),
                    compression: CompressionConfig {
                        enabled: true,
                        threshold_bytes: 1024,
                        algorithm: "lz4".to_string(),
                    },
                },
                default_indexing: IndexingDefaults {
                    index_type: "hnsw".to_string(),
                    parameters: HashMap::new(),
                },
                processing: ProcessingDefaults {
                    chunk_size: 2048,   // Chunks maiores para melhor contexto
                    chunk_overlap: 256, // Overlap maior para melhor continuidade
                    max_file_size_mb: 10,
                    supported_extensions: vec![
                        ".md".to_string(),
                        ".txt".to_string(),
                        ".rs".to_string(),
                        ".py".to_string(),
                        ".js".to_string(),
                        ".ts".to_string(),
                        ".json".to_string(),
                    ],
                },
            },
            projects: Vec::new(),
            processing: ProcessingSettings {
                parallel_processing: true,
                max_concurrent_projects: 4,
                max_concurrent_collections: 8,
                file_processing: FileProcessingSettings {
                    batch_size: 100,
                    max_file_size_mb: 10,
                    skip_hidden_files: true,
                    skip_binary_files: true,
                },
                memory: MemorySettings {
                    max_memory_usage_gb: 8.0,
                    gc_threshold_mb: 1024,
                },
                error_handling: ErrorHandlingSettings {
                    max_retries: 3,
                    retry_delay_seconds: 5,
                    continue_on_error: true,
                    log_errors: true,
                },
            },
            monitoring: MonitoringSettings {
                health_check: HealthCheckSettings {
                    enabled: true,
                    interval_seconds: 60,
                    check_projects: true,
                    check_collections: true,
                },
                metrics: MetricsSettings {
                    enabled: true,
                    collection_interval_seconds: 300,
                    project_metrics: vec![
                        "file_count".to_string(),
                        "total_size_mb".to_string(),
                        "processing_time_seconds".to_string(),
                        "error_count".to_string(),
                    ],
                    collection_metrics: vec![
                        "vector_count".to_string(),
                        "index_size_mb".to_string(),
                        "query_latency_ms".to_string(),
                        "memory_usage_mb".to_string(),
                    ],
                },
                logging: LoggingSettings {
                    level: "info".to_string(),
                    log_file: "./.logs/workspace.log".to_string(),
                    max_log_size_mb: 100,
                    max_log_files: 5,
                },
            },
            validation: ValidationSettings {
                paths: PathValidationSettings {
                    validate_existence: true,
                    validate_permissions: true,
                    create_missing_dirs: false,
                },
                config: ConfigValidationSettings {
                    validate_embedding_models: true,
                    validate_dimensions: true,
                    validate_collections: true,
                },
                data: DataValidationSettings {
                    validate_file_types: true,
                    validate_file_sizes: true,
                    validate_encoding: true,
                },
            },
            file_watcher: None,
        }
    }
}
