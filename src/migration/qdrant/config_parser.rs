//! Qdrant configuration file parser
//!
//! Parses Qdrant configuration files (YAML/JSON) and converts them to Vectorizer configuration.

use std::fs;
use std::path::Path;

use serde::{Deserialize, Serialize};
use tracing::{debug, info, warn};

use crate::error::{Result, VectorizerError};
use crate::models::{CollectionConfig, DistanceMetric, HnswConfig, QuantizationConfig};

/// Qdrant configuration file parser
pub struct QdrantConfigParser;

impl QdrantConfigParser {
    /// Parse Qdrant configuration from file
    ///
    /// Supports both YAML and JSON formats
    pub fn parse_file<P: AsRef<Path>>(path: P) -> Result<QdrantConfigFile> {
        let path = path.as_ref();
        let content = fs::read_to_string(path).map_err(|e| VectorizerError::Io(e))?;

        info!("📄 Parsing Qdrant config file: {}", path.display());

        let config: QdrantConfigFile = if path.extension().and_then(|s| s.to_str()) == Some("yaml")
            || path.extension().and_then(|s| s.to_str()) == Some("yml")
        {
            serde_yaml::from_str(&content).map_err(|e| {
                VectorizerError::Deserialization(format!("Failed to parse YAML config: {}", e))
            })?
        } else {
            serde_json::from_str(&content).map_err(|e| {
                VectorizerError::Deserialization(format!("Failed to parse JSON config: {}", e))
            })?
        };

        debug!("✅ Successfully parsed Qdrant config");
        Ok(config)
    }

    /// Parse Qdrant configuration from string
    pub fn parse_str(content: &str, format: ConfigFormat) -> Result<QdrantConfigFile> {
        let config: QdrantConfigFile = match format {
            ConfigFormat::Yaml => serde_yaml::from_str(content).map_err(|e| {
                VectorizerError::Deserialization(format!("Failed to parse YAML: {}", e))
            })?,
            ConfigFormat::Json => serde_json::from_str(content).map_err(|e| {
                VectorizerError::Deserialization(format!("Failed to parse JSON: {}", e))
            })?,
        };

        Ok(config)
    }

    /// Validate Qdrant configuration
    pub fn validate(config: &QdrantConfigFile) -> Result<ValidationResult> {
        let mut errors = Vec::new();
        let mut warnings = Vec::new();

        // Validate collections
        if let Some(collections) = &config.collections {
            for (name, collection_config) in collections {
                // Validate dimension
                if let Some(vectors_config) = &collection_config.vectors {
                    match vectors_config {
                        QdrantVectorsConfig::Vector { size, distance: _ } => {
                            if *size == 0 {
                                errors.push(format!(
                                    "Collection '{}': vector size must be > 0",
                                    name
                                ));
                            }
                            if *size > 65535 {
                                warnings.push(format!(
                                    "Collection '{}': very large vector dimension ({})",
                                    name, size
                                ));
                            }
                        }
                        QdrantVectorsConfig::NamedVectors { .. } => {
                            warnings.push(format!(
                                "Collection '{}': named vectors not supported, will use first vector",
                                name
                            ));
                        }
                    }
                }

                // Validate HNSW config
                if let Some(hnsw_config) = &collection_config.hnsw_config {
                    if hnsw_config.m < 4 || hnsw_config.m > 64 {
                        warnings.push(format!(
                            "Collection '{}': HNSW m parameter ({}) outside recommended range (4-64)",
                            name, hnsw_config.m
                        ));
                    }
                    if hnsw_config.ef_construct < 4 {
                        errors.push(format!(
                            "Collection '{}': ef_construct ({}) must be >= 4",
                            name, hnsw_config.ef_construct
                        ));
                    }
                }
            }
        }

        let is_valid = errors.is_empty();

        Ok(ValidationResult {
            is_valid,
            errors,
            warnings,
        })
    }

    /// Convert Qdrant config to Vectorizer collection configs
    pub fn convert_to_vectorizer(
        config: &QdrantConfigFile,
    ) -> Result<Vec<(String, CollectionConfig)>> {
        let mut vectorizer_configs = Vec::new();

        if let Some(collections) = &config.collections {
            for (name, qdrant_config) in collections {
                debug!("🔄 Converting collection '{}'", name);

                let vectorizer_config = Self::convert_collection_config(qdrant_config)?;
                vectorizer_configs.push((name.clone(), vectorizer_config));
            }
        }

        info!(
            "✅ Converted {} collections to Vectorizer format",
            vectorizer_configs.len()
        );
        Ok(vectorizer_configs)
    }

    /// Convert a single Qdrant collection config to Vectorizer format
    fn convert_collection_config(
        qdrant_config: &QdrantCollectionConfigFile,
    ) -> Result<CollectionConfig> {
        // Extract dimension
        let dimension = match &qdrant_config.vectors {
            Some(QdrantVectorsConfig::Vector { size, distance: _ }) => *size as usize,
            Some(QdrantVectorsConfig::NamedVectors { named }) => {
                // Use first named vector's size
                if let Some((_, config)) = named.iter().next() {
                    config.size as usize
                } else {
                    return Err(VectorizerError::Other(
                        "No vectors defined in collection config".to_string(),
                    ));
                }
            }
            None => {
                return Err(VectorizerError::Other(
                    "Missing vectors configuration".to_string(),
                ));
            }
        };

        // Extract distance metric
        let metric = match &qdrant_config.vectors {
            Some(QdrantVectorsConfig::Vector { size: _, distance }) => {
                Self::convert_distance_metric(distance)
            }
            Some(QdrantVectorsConfig::NamedVectors { named }) => {
                // Use first named vector's distance
                if let Some((_, config)) = named.iter().next() {
                    Self::convert_distance_metric(&config.distance)
                } else {
                    DistanceMetric::Cosine // Default
                }
            }
            None => DistanceMetric::Cosine, // Default
        };

        // Extract HNSW config
        let hnsw_config = if let Some(qdrant_hnsw) = &qdrant_config.hnsw_config {
            HnswConfig {
                m: qdrant_hnsw.m as usize,
                ef_construction: qdrant_hnsw.ef_construct as usize,
                ef_search: qdrant_hnsw.ef.unwrap_or(qdrant_hnsw.ef_construct) as usize,
                seed: None,
            }
        } else {
            HnswConfig::default()
        };

        // Extract quantization config
        let quantization = if let Some(qdrant_quant) = &qdrant_config.quantization_config {
            match qdrant_quant.quantization {
                QdrantQuantizationType::Int8 => QuantizationConfig::SQ { bits: 8 },
                _ => {
                    warn!("Only SQ8 quantization supported, using default");
                    QuantizationConfig::SQ { bits: 8 }
                }
            }
        } else {
            QuantizationConfig::SQ { bits: 8 } // Default
        };

        Ok(CollectionConfig {
            dimension,
            metric,
            hnsw_config,
            quantization,
            compression: crate::models::CompressionConfig::default(),
            normalization: None,
            storage_type: Some(crate::models::StorageType::Memory),
            sharding: None,
            graph: None,
            encryption: None,
        })
    }

    /// Convert Qdrant distance metric to Vectorizer metric
    fn convert_distance_metric(qdrant_distance: &str) -> DistanceMetric {
        match qdrant_distance.to_lowercase().as_str() {
            "cosine" => DistanceMetric::Cosine,
            "euclidean" => DistanceMetric::Euclidean,
            "dot" => DistanceMetric::DotProduct,
            _ => {
                warn!(
                    "Unknown distance metric '{}', defaulting to Cosine",
                    qdrant_distance
                );
                DistanceMetric::Cosine
            }
        }
    }
}

/// Configuration file format
#[derive(Debug, Clone, Copy)]
pub enum ConfigFormat {
    Yaml,
    Json,
}

/// Qdrant configuration file structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QdrantConfigFile {
    /// Collections configuration
    pub collections: Option<std::collections::HashMap<String, QdrantCollectionConfigFile>>,
    /// Storage configuration
    pub storage: Option<QdrantStorageConfigFile>,
    /// Service configuration
    pub service: Option<QdrantServiceConfigFile>,
}

/// Qdrant collection configuration from file
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QdrantCollectionConfigFile {
    /// Vectors configuration
    pub vectors: Option<QdrantVectorsConfig>,
    /// HNSW configuration
    #[serde(rename = "hnsw_config")]
    pub hnsw_config: Option<QdrantHnswConfigFile>,
    /// Optimizer configuration
    #[serde(rename = "optimizer_config")]
    pub optimizer_config: Option<QdrantOptimizerConfigFile>,
    /// Quantization configuration
    #[serde(rename = "quantization_config")]
    pub quantization_config: Option<QdrantQuantizationConfigFile>,
}

/// Qdrant vectors configuration from file
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum QdrantVectorsConfig {
    /// Single vector configuration
    Vector {
        /// Vector size (dimension)
        size: u32,
        /// Distance metric
        distance: String,
    },
    /// Named vectors configuration (not fully supported)
    NamedVectors {
        /// Named vector configurations
        #[serde(flatten)]
        named: std::collections::HashMap<String, QdrantNamedVectorConfig>,
    },
}

/// Qdrant named vector configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QdrantNamedVectorConfig {
    /// Vector size
    pub size: u32,
    /// Distance metric
    pub distance: String,
}

/// Qdrant HNSW configuration from file
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QdrantHnswConfigFile {
    /// M parameter
    pub m: u32,
    /// EF construction parameter
    #[serde(rename = "ef_construct")]
    pub ef_construct: u32,
    /// EF search parameter (optional)
    pub ef: Option<u32>,
    /// Full scan threshold
    #[serde(rename = "full_scan_threshold")]
    pub full_scan_threshold: Option<u32>,
}

/// Qdrant optimizer configuration from file
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QdrantOptimizerConfigFile {
    /// Deleted threshold
    #[serde(rename = "deleted_threshold")]
    pub deleted_threshold: Option<f64>,
    /// Vacuum min vector number
    #[serde(rename = "vacuum_min_vector_number")]
    pub vacuum_min_vector_number: Option<u32>,
    /// Default segment number
    #[serde(rename = "default_segment_number")]
    pub default_segment_number: Option<u32>,
}

/// Qdrant quantization configuration from file
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QdrantQuantizationConfigFile {
    /// Quantization type
    pub quantization: QdrantQuantizationType,
}

/// Qdrant quantization type
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum QdrantQuantizationType {
    /// Scalar quantization (Int8)
    Int8,
    /// Product quantization (not supported)
    Product,
    /// Binary quantization (not supported)
    Binary,
}

/// Qdrant storage configuration from file
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QdrantStorageConfigFile {
    /// Storage path
    pub storage_path: Option<String>,
}

/// Qdrant service configuration from file
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QdrantServiceConfigFile {
    /// Host
    pub host: Option<String>,
    /// Port
    pub port: Option<u16>,
}

/// Configuration validation result
#[derive(Debug, Clone)]
pub struct ValidationResult {
    /// Whether configuration is valid
    pub is_valid: bool,
    /// Validation errors
    pub errors: Vec<String>,
    /// Validation warnings
    pub warnings: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_yaml_config() {
        let yaml = r#"
collections:
  my_collection:
    vectors:
      size: 384
      distance: Cosine
    hnsw_config:
      m: 16
      ef_construct: 100
"#;

        let config = QdrantConfigParser::parse_str(yaml, ConfigFormat::Yaml).unwrap();
        assert!(config.collections.is_some());
        let collections = config.collections.unwrap();
        assert!(collections.contains_key("my_collection"));
    }

    #[test]
    fn test_parse_json_config() {
        let json = r#"
{
  "collections": {
    "my_collection": {
      "vectors": {
        "size": 384,
        "distance": "Cosine"
      }
    }
  }
}
"#;

        let config = QdrantConfigParser::parse_str(json, ConfigFormat::Json).unwrap();
        assert!(config.collections.is_some());
    }

    #[test]
    fn test_convert_to_vectorizer() {
        let yaml = r#"
collections:
  test_collection:
    vectors:
      size: 128
      distance: Euclidean
    hnsw_config:
      m: 16
      ef_construct: 100
"#;

        let config = QdrantConfigParser::parse_str(yaml, ConfigFormat::Yaml).unwrap();
        let vectorizer_configs = QdrantConfigParser::convert_to_vectorizer(&config).unwrap();

        assert_eq!(vectorizer_configs.len(), 1);
        let (name, config) = &vectorizer_configs[0];
        assert_eq!(name, "test_collection");
        assert_eq!(config.dimension, 128);
        assert_eq!(config.metric, DistanceMetric::Euclidean);
    }

    #[test]
    fn test_validate_config() {
        let yaml = r#"
collections:
  valid_collection:
    vectors:
      size: 384
      distance: Cosine
  invalid_collection:
    vectors:
      size: 0
      distance: Cosine
"#;

        let config = QdrantConfigParser::parse_str(yaml, ConfigFormat::Yaml).unwrap();
        let validation = QdrantConfigParser::validate(&config).unwrap();

        assert!(!validation.is_valid);
        assert!(!validation.errors.is_empty());
    }
}
