//! Simplified workspace configuration structures
//!
//! Defines minimal configuration structures with intelligent defaults

use std::path::PathBuf;

use serde::{Deserialize, Serialize};

/// Simplified workspace configuration root structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimplifiedWorkspaceConfig {
    /// Workspace metadata (optional - will use built-in defaults if not specified)
    pub workspace: Option<WorkspaceMetadata>,

    /// Global defaults applied to all collections (optional - will use built-in defaults if not specified)
    pub defaults: Option<DefaultConfiguration>,

    /// List of projects in the workspace
    pub projects: Vec<SimplifiedProjectConfig>,
}

/// Workspace metadata (same as original)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceMetadata {
    /// Workspace name
    pub name: String,

    /// Workspace version
    pub version: String,

    /// Workspace description
    pub description: String,
}

/// Default configuration applied to all collections
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DefaultConfiguration {
    /// Default embedding configuration
    pub embedding: EmbeddingConfig,

    /// Default collection settings
    pub dimension: u32,

    /// Default metric
    pub metric: String,

    /// Default indexing settings
    pub indexing: IndexingConfig,

    /// Default processing settings
    pub processing: ProcessingConfig,
}

/// Embedding configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbeddingConfig {
    /// Embedding model
    pub model: String,

    /// Vector dimension
    pub dimension: u32,

    /// Model parameters
    pub parameters: serde_yaml::Value,
}

/// Indexing configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexingConfig {
    /// Index type
    pub index_type: String,

    /// Index parameters
    pub parameters: serde_yaml::Value,
}

/// Processing configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessingConfig {
    /// Chunk size
    pub chunk_size: u32,

    /// Chunk overlap
    pub chunk_overlap: u32,

    /// Maximum file size in MB
    pub max_file_size_mb: u32,

    /// Supported file extensions
    pub supported_extensions: Vec<String>,
}

/// Simplified project configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimplifiedProjectConfig {
    /// Project name
    pub name: String,

    /// Project path
    pub path: String,

    /// Project description
    pub description: String,

    /// Collections for this project
    pub collections: Vec<SimplifiedCollectionConfig>,
}

/// Simplified collection configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimplifiedCollectionConfig {
    /// Collection name
    pub name: String,

    /// Collection description
    pub description: String,

    /// File patterns to include
    pub include_patterns: Vec<String>,

    /// File patterns to exclude (optional - defaults to empty)
    #[serde(default)]
    pub exclude_patterns: Vec<String>,

    // Optional overrides (inherit from defaults if not specified)
    /// Override embedding configuration
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub embedding: Option<EmbeddingConfig>,

    /// Override dimension
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub dimension: Option<u32>,

    /// Override metric
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub metric: Option<String>,

    /// Override indexing configuration
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub indexing: Option<IndexingConfig>,

    /// Override processing configuration
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub processing: Option<ProcessingConfig>,
}

impl Default for DefaultConfiguration {
    fn default() -> Self {
        Self {
            embedding: EmbeddingConfig {
                model: "bm25".to_string(),
                dimension: 512,
                parameters: serde_yaml::Value::Mapping({
                    let mut map = serde_yaml::Mapping::new();
                    map.insert(
                        serde_yaml::Value::String("k1".to_string()),
                        serde_yaml::Value::Number(serde_yaml::Number::from(1.5)),
                    );
                    map.insert(
                        serde_yaml::Value::String("b".to_string()),
                        serde_yaml::Value::Number(serde_yaml::Number::from(0.75)),
                    );
                    map
                }),
            },
            dimension: 512,
            metric: "cosine".to_string(),
            indexing: IndexingConfig {
                index_type: "hnsw".to_string(),
                parameters: serde_yaml::Value::Mapping({
                    let mut map = serde_yaml::Mapping::new();
                    map.insert(
                        serde_yaml::Value::String("m".to_string()),
                        serde_yaml::Value::Number(serde_yaml::Number::from(16)),
                    );
                    map.insert(
                        serde_yaml::Value::String("ef_construction".to_string()),
                        serde_yaml::Value::Number(serde_yaml::Number::from(200)),
                    );
                    map.insert(
                        serde_yaml::Value::String("ef_search".to_string()),
                        serde_yaml::Value::Number(serde_yaml::Number::from(64)),
                    );
                    map
                }),
            },
            processing: ProcessingConfig {
                chunk_size: 2048,
                chunk_overlap: 256,
                max_file_size_mb: 10,
                supported_extensions: vec![
                    ".md".to_string(),
                    ".txt".to_string(),
                    ".rs".to_string(),
                    ".py".to_string(),
                    ".js".to_string(),
                    ".ts".to_string(),
                    ".json".to_string(),
                    ".yaml".to_string(),
                    ".yml".to_string(),
                    ".toml".to_string(),
                    ".cpp".to_string(),
                    ".h".to_string(),
                    ".cc".to_string(),
                    ".html".to_string(),
                    ".css".to_string(),
                    ".sh".to_string(),
                    ".bat".to_string(),
                ],
            },
        }
    }
}

impl SimplifiedCollectionConfig {
    /// Get the effective embedding configuration (collection override or default)
    pub fn get_embedding_config<'a>(
        &'a self,
        defaults: &'a DefaultConfiguration,
    ) -> &'a EmbeddingConfig {
        self.embedding.as_ref().unwrap_or(&defaults.embedding)
    }

    /// Get the effective dimension (collection override or default)
    pub fn get_dimension(&self, defaults: &DefaultConfiguration) -> u32 {
        self.dimension.unwrap_or(defaults.dimension)
    }

    /// Get the effective metric (collection override or default)
    pub fn get_metric<'a>(&'a self, defaults: &'a DefaultConfiguration) -> &'a str {
        self.metric.as_deref().unwrap_or(&defaults.metric)
    }

    /// Get the effective indexing configuration (collection override or default)
    pub fn get_indexing_config<'a>(
        &'a self,
        defaults: &'a DefaultConfiguration,
    ) -> &'a IndexingConfig {
        self.indexing.as_ref().unwrap_or(&defaults.indexing)
    }

    /// Get the effective processing configuration (collection override or default)
    pub fn get_processing_config<'a>(
        &'a self,
        defaults: &'a DefaultConfiguration,
    ) -> &'a ProcessingConfig {
        self.processing.as_ref().unwrap_or(&defaults.processing)
    }

    /// Convert to full collection configuration for compatibility
    pub fn to_full_collection_config(
        &self,
        defaults: &DefaultConfiguration,
    ) -> crate::workspace::config::CollectionConfig {
        use crate::models::{DistanceMetric, HnswConfig};
        use crate::workspace::config::*;

        let embedding_config = self.get_embedding_config(defaults);
        let dimension = self.get_dimension(defaults);
        let metric = self.get_metric(defaults);
        let indexing_config = self.get_indexing_config(defaults);
        let processing_config = self.get_processing_config(defaults);

        // Parse indexing parameters
        let hnsw_params = if let serde_yaml::Value::Mapping(params) = &indexing_config.parameters {
            HnswConfig {
                m: params.get("m").and_then(|v| v.as_u64()).unwrap_or(16) as usize,
                ef_construction: params
                    .get("ef_construction")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(200) as usize,
                ef_search: params
                    .get("ef_search")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(64) as usize,
                seed: None,
            }
        } else {
            HnswConfig::default()
        };

        // Parse embedding parameters
        let embedding_params = if let serde_yaml::Value::Mapping(params) =
            &embedding_config.parameters
        {
            let mut param_map = std::collections::HashMap::new();
            for (key, value) in params {
                if let Some(key_str) = key.as_str() {
                    // Convert serde_yaml::Value to serde_json::Value
                    let json_value = match value {
                        serde_yaml::Value::Null => serde_json::Value::Null,
                        serde_yaml::Value::Bool(b) => serde_json::Value::Bool(*b),
                        serde_yaml::Value::Number(n) => {
                            if let Some(i) = n.as_i64() {
                                serde_json::Value::Number(serde_json::Number::from(i))
                            } else if let Some(f) = n.as_f64() {
                                if let Some(n) = serde_json::Number::from_f64(f) {
                                    serde_json::Value::Number(n)
                                } else {
                                    serde_json::Value::Null
                                }
                            } else {
                                serde_json::Value::Null
                            }
                        }
                        serde_yaml::Value::String(s) => serde_json::Value::String(s.clone()),
                        serde_yaml::Value::Sequence(seq) => {
                            let json_seq: Result<Vec<serde_json::Value>, String> = seq
                                .iter()
                                .map(|v| match v {
                                    serde_yaml::Value::Null => Ok(serde_json::Value::Null),
                                    serde_yaml::Value::Bool(b) => Ok(serde_json::Value::Bool(*b)),
                                    serde_yaml::Value::Number(n) => {
                                        if let Some(i) = n.as_i64() {
                                            Ok(serde_json::Value::Number(serde_json::Number::from(
                                                i,
                                            )))
                                        } else if let Some(f) = n.as_f64() {
                                            if let Some(n) = serde_json::Number::from_f64(f) {
                                                Ok(serde_json::Value::Number(n))
                                            } else {
                                                Ok(serde_json::Value::Null)
                                            }
                                        } else {
                                            Ok(serde_json::Value::Null)
                                        }
                                    }
                                    serde_yaml::Value::String(s) => {
                                        Ok(serde_json::Value::String(s.clone()))
                                    }
                                    _ => Ok(serde_json::Value::String(format!("{:?}", v))),
                                })
                                .collect();
                            serde_json::Value::Array(json_seq.unwrap_or_default())
                        }
                        serde_yaml::Value::Mapping(map) => {
                            let json_map: Result<
                                std::collections::HashMap<String, serde_json::Value>,
                                String,
                            > = map
                                .iter()
                                .map(|(k, v)| {
                                    let key = if let Some(key_str) = k.as_str() {
                                        key_str.to_string()
                                    } else {
                                        format!("{:?}", k)
                                    };
                                    let value = match v {
                                        serde_yaml::Value::Null => serde_json::Value::Null,
                                        serde_yaml::Value::Bool(b) => serde_json::Value::Bool(*b),
                                        serde_yaml::Value::Number(n) => {
                                            if let Some(i) = n.as_i64() {
                                                serde_json::Value::Number(serde_json::Number::from(
                                                    i,
                                                ))
                                            } else if let Some(f) = n.as_f64() {
                                                if let Some(n) = serde_json::Number::from_f64(f) {
                                                    serde_json::Value::Number(n)
                                                } else {
                                                    serde_json::Value::Null
                                                }
                                            } else {
                                                serde_json::Value::Null
                                            }
                                        }
                                        serde_yaml::Value::String(s) => {
                                            serde_json::Value::String(s.clone())
                                        }
                                        _ => serde_json::Value::String(format!("{:?}", v)),
                                    };
                                    Ok((key, value))
                                })
                                .collect();
                            serde_json::Value::Object(serde_json::Map::from_iter(
                                json_map.unwrap_or_default(),
                            ))
                        }
                        _ => serde_json::Value::String(format!("{:?}", value)),
                    };
                    param_map.insert(key_str.to_string(), json_value);
                }
            }
            param_map
        } else {
            std::collections::HashMap::new()
        };

        // Convert metric string to enum
        let distance_metric = match metric {
            "cosine" => crate::workspace::config::DistanceMetric::Cosine,
            "euclidean" => crate::workspace::config::DistanceMetric::Euclidean,
            "dot_product" => crate::workspace::config::DistanceMetric::DotProduct,
            _ => crate::workspace::config::DistanceMetric::Cosine,
        };

        CollectionConfig {
            name: self.name.clone(),
            description: self.description.clone(),
            dimension: dimension as usize,
            metric: distance_metric,
            embedding: EmbeddingConfig {
                model: match embedding_config.model.as_str() {
                    "tfidf" => EmbeddingModel::TfIdf,
                    "bm25" => EmbeddingModel::Bm25,
                    "svd" => EmbeddingModel::Svd,
                    "bert" => EmbeddingModel::Bert,
                    "minilm" => EmbeddingModel::MiniLm,
                    "bagofwords" => EmbeddingModel::BagOfWords,
                    "charngram" => EmbeddingModel::CharNGram,
                    "real_model" => EmbeddingModel::RealModel,
                    "onnx_model" => EmbeddingModel::OnnxModel,
                    _ => EmbeddingModel::Bm25,
                },
                dimension: embedding_config.dimension as usize,
                parameters: embedding_params,
            },
            indexing: IndexingConfig {
                index_type: indexing_config.index_type.clone(),
                parameters: {
                    let mut param_map = std::collections::HashMap::new();
                    if let serde_yaml::Value::Mapping(params) = &indexing_config.parameters {
                        for (key, value) in params {
                            if let Some(key_str) = key.as_str() {
                                // Convert serde_yaml::Value to serde_json::Value (simplified)
                                let json_value = match value {
                                    serde_yaml::Value::Number(n) => {
                                        if let Some(i) = n.as_i64() {
                                            serde_json::Value::Number(serde_json::Number::from(i))
                                        } else if let Some(f) = n.as_f64() {
                                            if let Some(n) = serde_json::Number::from_f64(f) {
                                                serde_json::Value::Number(n)
                                            } else {
                                                serde_json::Value::Null
                                            }
                                        } else {
                                            serde_json::Value::Null
                                        }
                                    }
                                    serde_yaml::Value::String(s) => {
                                        serde_json::Value::String(s.clone())
                                    }
                                    _ => serde_json::Value::String(format!("{:?}", value)),
                                };
                                param_map.insert(key_str.to_string(), json_value);
                            }
                        }
                    }
                    param_map
                },
            },
            processing: CollectionProcessing {
                chunk_size: processing_config.chunk_size as usize,
                chunk_overlap: processing_config.chunk_overlap as usize,
                include_patterns: self.include_patterns.clone(),
                exclude_patterns: self.exclude_patterns.clone(),
            },
        }
    }
}

impl SimplifiedProjectConfig {
    /// Convert to full project configuration for compatibility
    pub fn to_full_project_config(
        &self,
        defaults: &DefaultConfiguration,
    ) -> crate::workspace::config::ProjectConfig {
        use crate::workspace::config::*;

        let collections: Vec<CollectionConfig> = self
            .collections
            .iter()
            .map(|c| c.to_full_collection_config(defaults))
            .collect();

        ProjectConfig {
            name: self.name.clone(),
            path: PathBuf::from(&self.path),
            description: self.description.clone(),
            enabled: true,   // Simplified configs are always enabled
            embedding: None, // Use defaults
            collections,
        }
    }
}

impl SimplifiedWorkspaceConfig {
    /// Get the effective defaults (user-specified or built-in)
    pub fn get_effective_defaults(&self) -> DefaultConfiguration {
        self.defaults
            .clone()
            .unwrap_or_else(|| DefaultConfiguration::default())
    }

    /// Get effective workspace metadata, using built-in defaults if none specified
    pub fn get_effective_workspace(&self) -> WorkspaceMetadata {
        self.workspace.clone().unwrap_or_else(|| WorkspaceMetadata {
            name: "Default Workspace".to_string(),
            version: "1.0.0".to_string(),
            description: "Auto-generated workspace configuration".to_string(),
        })
    }

    /// Convert to full workspace configuration for compatibility
    pub fn to_full_workspace_config(&self) -> crate::workspace::config::WorkspaceConfig {
        use crate::models::HnswConfig;
        use crate::workspace::config::*;

        let effective_defaults = self.get_effective_defaults();
        let effective_workspace = self.get_effective_workspace();
        let projects: Vec<ProjectConfig> = self
            .projects
            .iter()
            .map(|p| p.to_full_project_config(&effective_defaults))
            .collect();

        WorkspaceConfig {
            workspace: WorkspaceMetadata {
                name: effective_workspace.name.clone(),
                version: effective_workspace.version.clone(),
                description: effective_workspace.description.clone(),
                created_at: chrono::Utc::now().to_rfc3339(),
                last_updated: chrono::Utc::now().to_rfc3339(),
            },
            global: GlobalSettings {
                default_embedding: EmbeddingConfig {
                    model: match effective_defaults.embedding.model.as_str() {
                        "tfidf" => EmbeddingModel::TfIdf,
                        "bm25" => EmbeddingModel::Bm25,
                        "svd" => EmbeddingModel::Svd,
                        "bert" => EmbeddingModel::Bert,
                        "minilm" => EmbeddingModel::MiniLm,
                        "bagofwords" => EmbeddingModel::BagOfWords,
                        "charngram" => EmbeddingModel::CharNGram,
                        "real_model" => EmbeddingModel::RealModel,
                        "onnx_model" => EmbeddingModel::OnnxModel,
                        _ => EmbeddingModel::Bm25,
                    },
                    dimension: effective_defaults.embedding.dimension as usize,
                    parameters: {
                        let mut param_map = std::collections::HashMap::new();
                        if let serde_yaml::Value::Mapping(params) =
                            &effective_defaults.embedding.parameters
                        {
                            for (key, value) in params {
                                if let Some(key_str) = key.as_str() {
                                    // Convert serde_yaml::Value to serde_json::Value (simplified for defaults)
                                    let json_value = match value {
                                        serde_yaml::Value::Number(n) => {
                                            if let Some(i) = n.as_i64() {
                                                serde_json::Value::Number(serde_json::Number::from(
                                                    i,
                                                ))
                                            } else if let Some(f) = n.as_f64() {
                                                if let Some(n) = serde_json::Number::from_f64(f) {
                                                    serde_json::Value::Number(n)
                                                } else {
                                                    serde_json::Value::Null
                                                }
                                            } else {
                                                serde_json::Value::Null
                                            }
                                        }
                                        serde_yaml::Value::String(s) => {
                                            serde_json::Value::String(s.clone())
                                        }
                                        _ => serde_json::Value::String(format!("{:?}", value)),
                                    };
                                    param_map.insert(key_str.to_string(), json_value);
                                }
                            }
                        }
                        param_map
                    },
                },
                default_collection: CollectionDefaults {
                    metric: match effective_defaults.metric.as_str() {
                        "cosine" => crate::workspace::config::DistanceMetric::Cosine,
                        "euclidean" => crate::workspace::config::DistanceMetric::Euclidean,
                        "dot_product" => crate::workspace::config::DistanceMetric::DotProduct,
                        _ => crate::workspace::config::DistanceMetric::Cosine,
                    },
                    quantization: None,
                    compression: CompressionConfig {
                        enabled: true,
                        threshold_bytes: 1024,
                        algorithm: "lz4".to_string(),
                    },
                },
                default_indexing: IndexingDefaults {
                    index_type: effective_defaults.indexing.index_type.clone(),
                    parameters: {
                        let mut param_map = std::collections::HashMap::new();
                        if let serde_yaml::Value::Mapping(params) =
                            &effective_defaults.indexing.parameters
                        {
                            for (key, value) in params {
                                if let Some(key_str) = key.as_str() {
                                    // Convert serde_yaml::Value to serde_json::Value (simplified for defaults)
                                    let json_value = match value {
                                        serde_yaml::Value::Number(n) => {
                                            if let Some(i) = n.as_i64() {
                                                serde_json::Value::Number(serde_json::Number::from(
                                                    i,
                                                ))
                                            } else if let Some(f) = n.as_f64() {
                                                if let Some(n) = serde_json::Number::from_f64(f) {
                                                    serde_json::Value::Number(n)
                                                } else {
                                                    serde_json::Value::Null
                                                }
                                            } else {
                                                serde_json::Value::Null
                                            }
                                        }
                                        serde_yaml::Value::String(s) => {
                                            serde_json::Value::String(s.clone())
                                        }
                                        _ => serde_json::Value::String(format!("{:?}", value)),
                                    };
                                    param_map.insert(key_str.to_string(), json_value);
                                }
                            }
                        }
                        param_map
                    },
                },
                processing: ProcessingDefaults {
                    chunk_size: effective_defaults.processing.chunk_size as usize,
                    chunk_overlap: effective_defaults.processing.chunk_overlap as usize,
                    max_file_size_mb: effective_defaults.processing.max_file_size_mb as usize,
                    supported_extensions: effective_defaults
                        .processing
                        .supported_extensions
                        .clone(),
                },
            },
            projects,
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
                    max_memory_usage_gb: 1.0,
                    gc_threshold_mb: 512,
                },
                error_handling: ErrorHandlingSettings {
                    max_retries: 3,
                    retry_delay_seconds: 1,
                    continue_on_error: true,
                    log_errors: true,
                },
            },
            monitoring: MonitoringSettings {
                health_check: HealthCheckSettings {
                    enabled: false,
                    interval_seconds: 10,
                    check_projects: true,
                    check_collections: true,
                },
                metrics: MetricsSettings {
                    enabled: false,
                    collection_interval_seconds: 5,
                    project_metrics: vec![
                        "indexing_time".to_string(),
                        "document_count".to_string(),
                    ],
                    collection_metrics: vec!["vector_count".to_string(), "search_time".to_string()],
                },
                logging: LoggingSettings {
                    level: "info".to_string(),
                    log_file: "vectorizer.log".to_string(),
                    max_log_size_mb: 10,
                    max_log_files: 5,
                },
            },
            validation: ValidationSettings {
                paths: PathValidationSettings {
                    validate_existence: true,
                    validate_permissions: false,
                    create_missing_dirs: true,
                },
                config: ConfigValidationSettings {
                    validate_embedding_models: true,
                    validate_dimensions: true,
                    validate_collections: true,
                },
                data: DataValidationSettings {
                    validate_file_types: true,
                    validate_file_sizes: true,
                    validate_encoding: false,
                },
            },
            file_watcher: None,
        }
    }
}

/// Parse simplified workspace configuration from YAML file
pub fn parse_simplified_workspace_config<P: AsRef<std::path::Path>>(
    path: P,
) -> Result<SimplifiedWorkspaceConfig, Box<dyn std::error::Error>> {
    use std::fs;

    let content = fs::read_to_string(path)?;
    let config: SimplifiedWorkspaceConfig = serde_yaml::from_str(&content)?;
    Ok(config)
}

/// Parse simplified workspace configuration from string
pub fn parse_simplified_workspace_config_from_str(
    content: &str,
) -> Result<SimplifiedWorkspaceConfig, Box<dyn std::error::Error>> {
    let config: SimplifiedWorkspaceConfig = serde_yaml::from_str(content)?;
    Ok(config)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_configuration() {
        let defaults = DefaultConfiguration::default();

        assert_eq!(defaults.embedding.model, "bm25");
        assert_eq!(defaults.dimension, 512);
        assert_eq!(defaults.metric, "cosine");
        assert_eq!(defaults.processing.chunk_size, 2048);
        assert_eq!(defaults.processing.chunk_overlap, 256);
    }

    #[test]
    fn test_embedding_config_creation() {
        let config = EmbeddingConfig {
            model: "bert".to_string(),
            dimension: 768,
            parameters: serde_yaml::from_str("{}").unwrap(),
        };

        assert_eq!(config.model, "bert");
        assert_eq!(config.dimension, 768);
    }

    #[test]
    fn test_processing_config_creation() {
        let config = ProcessingConfig {
            chunk_size: 4096,
            chunk_overlap: 512,
            max_file_size_mb: 10,
            supported_extensions: vec!["rs".to_string(), "md".to_string()],
        };

        assert_eq!(config.chunk_size, 4096);
        assert_eq!(config.max_file_size_mb, 10);
        assert_eq!(config.supported_extensions.len(), 2);
    }

    #[test]
    fn test_workspace_metadata_creation() {
        let metadata = WorkspaceMetadata {
            name: "test_workspace".to_string(),
            version: "1.0.0".to_string(),
            description: "Test workspace".to_string(),
        };

        assert_eq!(metadata.name, "test_workspace");
        assert_eq!(metadata.version, "1.0.0");
    }

    #[test]
    fn test_simplified_collection_config() {
        let collection = SimplifiedCollectionConfig {
            name: "test_collection".to_string(),
            description: "Test collection".to_string(),
            include_patterns: vec!["**/*.rs".to_string()],
            exclude_patterns: vec!["**/target/**".to_string()],
            embedding: None,
            dimension: None,
            metric: None,
            indexing: None,
            processing: None,
        };

        assert_eq!(collection.name, "test_collection");
        assert!(collection.embedding.is_none());
        assert_eq!(collection.include_patterns.len(), 1);
    }

    #[test]
    fn test_simplified_project_config() {
        let project = SimplifiedProjectConfig {
            name: "test_project".to_string(),
            path: "/path/to/project".to_string(),
            description: "Test project".to_string(),
            collections: vec![],
        };

        assert_eq!(project.name, "test_project");
        assert_eq!(project.collections.len(), 0);
    }

    #[test]
    fn test_simplified_workspace_config() {
        let workspace = SimplifiedWorkspaceConfig {
            workspace: None,
            defaults: None,
            projects: vec![],
        };

        assert!(workspace.workspace.is_none());
        assert!(workspace.defaults.is_none());
        assert_eq!(workspace.projects.len(), 0);
    }

    #[test]
    fn test_parse_simple_yaml() {
        let yaml = r#"
workspace:
  name: "test"
  version: "1.0.0"
  description: "Test workspace"
projects: []
"#;

        let result = parse_simplified_workspace_config_from_str(yaml);
        assert!(result.is_ok());

        let config = result.unwrap();
        assert!(config.workspace.is_some());
        assert_eq!(config.workspace.unwrap().name, "test");
    }

    #[test]
    fn test_parse_with_defaults() {
        let yaml = r#"
defaults:
  embedding:
    model: "bert"
    dimension: 768
    parameters: {}
  dimension: 768
  metric: "cosine"
  indexing:
    index_type: "hnsw"
    parameters: {}
  processing:
    chunk_size: 2048
    chunk_overlap: 256
    max_file_size_mb: 10
    supported_extensions: ["rs", "md"]
projects: []
"#;

        let result = parse_simplified_workspace_config_from_str(yaml);
        assert!(result.is_ok());

        let config = result.unwrap();
        assert!(config.defaults.is_some());

        let defaults = config.defaults.unwrap();
        assert_eq!(defaults.embedding.model, "bert");
        assert_eq!(defaults.dimension, 768);
    }

    #[test]
    fn test_parse_with_projects() {
        let yaml = r#"
projects:
  - name: "project1"
    path: "/path/to/project1"
    description: "First project"
    collections:
      - name: "docs"
        description: "Documentation"
        include_patterns: ["**/*.md"]
        exclude_patterns: ["**/node_modules/**"]
"#;

        let result = parse_simplified_workspace_config_from_str(yaml);
        assert!(result.is_ok());

        let config = result.unwrap();
        assert_eq!(config.projects.len(), 1);
        assert_eq!(config.projects[0].name, "project1");
        assert_eq!(config.projects[0].collections.len(), 1);
    }

    #[test]
    fn test_collection_with_overrides() {
        let collection = SimplifiedCollectionConfig {
            name: "custom".to_string(),
            description: "Custom collection".to_string(),
            include_patterns: vec!["**/*.rs".to_string()],
            exclude_patterns: vec![],
            embedding: Some(EmbeddingConfig {
                model: "custom_model".to_string(),
                dimension: 1024,
                parameters: serde_yaml::from_str("{}").unwrap(),
            }),
            dimension: Some(1024),
            metric: Some("euclidean".to_string()),
            indexing: None,
            processing: None,
        };

        assert!(collection.embedding.is_some());
        assert_eq!(collection.embedding.unwrap().dimension, 1024);
        assert_eq!(collection.dimension, Some(1024));
        assert_eq!(collection.metric, Some("euclidean".to_string()));
    }

    #[test]
    fn test_indexing_config() {
        let config = IndexingConfig {
            index_type: "hnsw".to_string(),
            parameters: serde_yaml::from_str("m: 16\nef_construction: 200").unwrap(),
        };

        assert_eq!(config.index_type, "hnsw");
    }

    #[test]
    fn test_parse_invalid_yaml() {
        let invalid_yaml = "invalid: yaml: content:";
        let result = parse_simplified_workspace_config_from_str(invalid_yaml);

        assert!(result.is_err());
    }

    #[test]
    fn test_parse_minimal_config() {
        let yaml = r#"
projects: []
"#;

        let result = parse_simplified_workspace_config_from_str(yaml);
        assert!(result.is_ok());

        let config = result.unwrap();
        assert_eq!(config.projects.len(), 0);
        assert!(config.workspace.is_none());
        assert!(config.defaults.is_none());
    }
}
