//! Workspace configuration validator
//!
//! Provides validation functionality for workspace configurations

use std::collections::HashSet;
use std::path::Path;

use tracing::{debug, error, info, warn};

use crate::workspace::config::*;

/// Validation result
#[derive(Debug, Clone)]
pub struct ValidationResult {
    /// Whether validation passed
    pub is_valid: bool,

    /// List of validation errors
    pub errors: Vec<String>,

    /// List of validation warnings
    pub warnings: Vec<String>,
}

impl ValidationResult {
    /// Create a new validation result
    pub fn new() -> Self {
        Self {
            is_valid: true,
            errors: Vec::new(),
            warnings: Vec::new(),
        }
    }

    /// Add an error
    pub fn add_error(&mut self, error: String) {
        self.is_valid = false;
        self.errors.push(error);
    }

    /// Add a warning
    pub fn add_warning(&mut self, warning: String) {
        self.warnings.push(warning);
    }

    /// Check if validation passed
    pub fn is_valid(&self) -> bool {
        self.is_valid
    }
}

/// Validate workspace configuration
pub fn validate_workspace_config(
    config: &WorkspaceConfig,
    workspace_root: &Path,
) -> ValidationResult {
    let mut result = ValidationResult::new();

    debug!("Validating workspace configuration");

    // Validate workspace metadata
    validate_workspace_metadata(config, &mut result);

    // Validate global settings
    validate_global_settings(config, &mut result);

    // Validate projects
    validate_projects(config, workspace_root, &mut result);

    // Validate processing settings
    validate_processing_settings(config, &mut result);

    // Validate monitoring settings
    validate_monitoring_settings(config, &mut result);

    // Validate validation settings
    validate_validation_settings(config, &mut result);

    if result.is_valid {
        debug!("Workspace configuration validation passed");
    } else {
        debug!(
            "Workspace configuration validation failed with {} errors",
            result.errors.len()
        );
    }

    if !result.warnings.is_empty() {
        warn!(
            "Workspace configuration validation completed with {} warnings",
            result.warnings.len()
        );
    }

    result
}

/// Validate workspace metadata
fn validate_workspace_metadata(config: &WorkspaceConfig, result: &mut ValidationResult) {
    debug!("Validating workspace metadata");

    // Check workspace name
    if config.workspace.name.is_empty() {
        result.add_error("Workspace name cannot be empty".to_string());
    }

    // Check workspace version
    if config.workspace.version.is_empty() {
        result.add_error("Workspace version cannot be empty".to_string());
    }

    // Check timestamps
    if config.workspace.created_at.is_empty() {
        result.add_warning("Workspace created_at timestamp is empty".to_string());
    }

    if config.workspace.last_updated.is_empty() {
        result.add_warning("Workspace last_updated timestamp is empty".to_string());
    }
}

/// Validate global settings
fn validate_global_settings(config: &WorkspaceConfig, result: &mut ValidationResult) {
    debug!("Validating global settings");

    // Validate default embedding
    validate_embedding_config(
        &config.global.default_embedding,
        "global.default_embedding",
        result,
    );

    // Validate default collection settings
    validate_collection_defaults(&config.global.default_collection, result);

    // Validate default indexing settings
    validate_indexing_defaults(&config.global.default_indexing, result);

    // Validate processing defaults
    validate_processing_defaults(&config.global.processing, result);
}

/// Validate projects
fn validate_projects(
    config: &WorkspaceConfig,
    workspace_root: &Path,
    result: &mut ValidationResult,
) {
    debug!("Validating {} projects", config.projects.len());

    let mut project_names = HashSet::new();

    for (index, project) in config.projects.iter().enumerate() {
        let project_prefix = format!("projects[{}]", index);

        // Check for duplicate project names
        if !project_names.insert(&project.name) {
            result.add_error(format!(
                "{}: Duplicate project name '{}'",
                project_prefix, project.name
            ));
        }

        // Validate project configuration
        validate_project_config(project, workspace_root, &project_prefix, result);
    }

    // Check if any projects are enabled
    let enabled_projects: Vec<_> = config.projects.iter().filter(|p| p.enabled).collect();
    if enabled_projects.is_empty() {
        result.add_warning("No projects are enabled in the workspace".to_string());
    }
}

/// Validate individual project configuration
fn validate_project_config(
    project: &ProjectConfig,
    workspace_root: &Path,
    prefix: &str,
    result: &mut ValidationResult,
) {
    debug!("Validating project: {}", project.name);

    // Validate project name
    if project.name.is_empty() {
        result.add_error(format!("{}: Project name cannot be empty", prefix));
    }

    // Validate project path
    let project_path = workspace_root.join(&project.path);
    if !project_path.exists() {
        result.add_error(format!(
            "{}: Project path does not exist: {}",
            prefix,
            project_path.display()
        ));
    } else if !project_path.is_dir() {
        result.add_error(format!(
            "{}: Project path is not a directory: {}",
            prefix,
            project_path.display()
        ));
    }

    // Validate project embedding configuration
    if let Some(embedding) = &project.embedding {
        validate_embedding_config(embedding, &format!("{}.embedding", prefix), result);
    }

    // Validate collections
    validate_collections(&project.collections, prefix, result);
}

/// Validate collections
fn validate_collections(
    collections: &[CollectionConfig],
    prefix: &str,
    result: &mut ValidationResult,
) {
    debug!("Validating {} collections", collections.len());

    let mut collection_names = HashSet::new();

    for (index, collection) in collections.iter().enumerate() {
        let collection_prefix = format!("{}.collections[{}]", prefix, index);

        // Check for duplicate collection names
        if !collection_names.insert(&collection.name) {
            result.add_error(format!(
                "{}: Duplicate collection name '{}'",
                collection_prefix, collection.name
            ));
        }

        // Validate collection configuration
        validate_collection_config(collection, &collection_prefix, result);
    }
}

/// Validate individual collection configuration
fn validate_collection_config(
    collection: &CollectionConfig,
    prefix: &str,
    result: &mut ValidationResult,
) {
    debug!("Validating collection: {}", collection.name);

    // Validate collection name
    if collection.name.is_empty() {
        result.add_error(format!("{}: Collection name cannot be empty", prefix));
    }

    // Validate dimension
    if collection.dimension == 0 {
        result.add_error(format!(
            "{}: Collection dimension must be greater than 0",
            prefix
        ));
    } else if collection.dimension > 4096 {
        result.add_warning(format!(
            "{}: Collection dimension {} is very large, consider using a smaller dimension",
            prefix, collection.dimension
        ));
    }

    // Validate embedding configuration
    validate_embedding_config(
        &collection.embedding,
        &format!("{}.embedding", prefix),
        result,
    );

    // Validate indexing configuration
    validate_indexing_config(
        &collection.indexing,
        &format!("{}.indexing", prefix),
        result,
    );

    // Validate processing configuration
    validate_collection_processing(&collection.processing, prefix, result);
}

/// Validate embedding configuration
fn validate_embedding_config(
    embedding: &EmbeddingConfig,
    prefix: &str,
    result: &mut ValidationResult,
) {
    debug!("Validating embedding configuration: {}", prefix);

    // Validate dimension
    if embedding.dimension == 0 {
        result.add_error(format!(
            "{}: Embedding dimension must be greater than 0",
            prefix
        ));
    }

    // Validate model-specific parameters
    match embedding.model {
        EmbeddingModel::TfIdf => {
            validate_tfidf_parameters(&embedding.parameters, prefix, result);
        }
        EmbeddingModel::Bm25 => {
            validate_bm25_parameters(&embedding.parameters, prefix, result);
        }
        EmbeddingModel::Svd => {
            validate_svd_parameters(&embedding.parameters, prefix, result);
        }
        EmbeddingModel::Bert => {
            validate_bert_parameters(&embedding.parameters, prefix, result);
        }
        EmbeddingModel::MiniLm => {
            validate_minilm_parameters(&embedding.parameters, prefix, result);
        }
        EmbeddingModel::BagOfWords => {
            validate_bow_parameters(&embedding.parameters, prefix, result);
        }
        EmbeddingModel::CharNGram => {
            validate_charngram_parameters(&embedding.parameters, prefix, result);
        }
        EmbeddingModel::RealModel => {
            validate_real_model_parameters(&embedding.parameters, prefix, result);
        }
        EmbeddingModel::OnnxModel => {
            validate_onnx_model_parameters(&embedding.parameters, prefix, result);
        }
    }
}

/// Validate Bag of Words parameters
fn validate_bow_parameters(
    parameters: &std::collections::HashMap<String, serde_json::Value>,
    prefix: &str,
    result: &mut ValidationResult,
) {
    if let Some(vocab_size) = parameters.get("vocab_size") {
        if let Some(size) = vocab_size.as_u64() {
            if size == 0 {
                result.add_error(format!("{}: vocab_size must be greater than 0", prefix));
            }
        }
    }

    if let Some(max_seq_len) = parameters.get("max_sequence_length") {
        if let Some(len) = max_seq_len.as_u64() {
            if len == 0 {
                result.add_error(format!(
                    "{}: max_sequence_length must be greater than 0",
                    prefix
                ));
            }
        }
    }
}

/// Validate feature hashing parameters
fn validate_hash_parameters(
    parameters: &std::collections::HashMap<String, serde_json::Value>,
    prefix: &str,
    result: &mut ValidationResult,
) {
    if let Some(hash_size) = parameters.get("hash_size") {
        if let Some(size) = hash_size.as_u64() {
            if size == 0 {
                result.add_error(format!("{}: hash_size must be greater than 0", prefix));
            }
        }
    }
}

/// Validate N-gram parameters
fn validate_ngram_parameters(
    parameters: &std::collections::HashMap<String, serde_json::Value>,
    prefix: &str,
    result: &mut ValidationResult,
) {
    if let Some(ngram_range) = parameters.get("ngram_range") {
        if let Some(range) = ngram_range.as_array() {
            if range.len() != 2 {
                result.add_error(format!(
                    "{}: ngram_range must have exactly 2 elements",
                    prefix
                ));
            } else {
                if let (Some(min), Some(max)) = (range[0].as_u64(), range[1].as_u64()) {
                    if min > max {
                        result.add_error(format!(
                            "{}: ngram_range min ({}) must be <= max ({})",
                            prefix, min, max
                        ));
                    }
                }
            }
        }
    }

    if let Some(vocab_size) = parameters.get("vocab_size") {
        if let Some(size) = vocab_size.as_u64() {
            if size == 0 {
                result.add_error(format!("{}: vocab_size must be greater than 0", prefix));
            }
        }
    }
}

/// Validate TF-IDF parameters
fn validate_tfidf_parameters(
    parameters: &std::collections::HashMap<String, serde_json::Value>,
    prefix: &str,
    result: &mut ValidationResult,
) {
    if let Some(vocab_size) = parameters.get("vocab_size") {
        if let Some(size) = vocab_size.as_u64() {
            if size == 0 {
                result.add_error(format!("{}: vocab_size must be greater than 0", prefix));
            }
        }
    }

    if let Some(min_df) = parameters.get("min_df") {
        if let Some(df) = min_df.as_f64() {
            if df < 0.0 || df > 1.0 {
                result.add_error(format!("{}: min_df must be between 0.0 and 1.0", prefix));
            }
        }
    }
}

/// Validate SVD parameters
fn validate_svd_parameters(
    parameters: &std::collections::HashMap<String, serde_json::Value>,
    prefix: &str,
    result: &mut ValidationResult,
) {
    if let Some(n_components) = parameters.get("n_components") {
        if let Some(components) = n_components.as_u64() {
            if components == 0 {
                result.add_error(format!("{}: n_components must be greater than 0", prefix));
            }
        }
    }

    if let Some(iterations) = parameters.get("iterations") {
        if let Some(iter) = iterations.as_u64() {
            if iter == 0 {
                result.add_error(format!("{}: iterations must be greater than 0", prefix));
            }
        }
    }
}

/// Validate BERT parameters
fn validate_bert_parameters(
    parameters: &std::collections::HashMap<String, serde_json::Value>,
    prefix: &str,
    result: &mut ValidationResult,
) {
    if let Some(max_seq_len) = parameters.get("max_sequence_length") {
        if let Some(len) = max_seq_len.as_u64() {
            if len == 0 || len > 512 {
                result.add_error(format!(
                    "{}: max_sequence_length must be between 1 and 512",
                    prefix
                ));
            }
        }
    }

    if let Some(model_name) = parameters.get("model_name") {
        if let Some(name) = model_name.as_str() {
            if name.is_empty() {
                result.add_error(format!("{}: model_name cannot be empty", prefix));
            }
        }
    }
}

/// Validate MiniLM parameters
fn validate_minilm_parameters(
    parameters: &std::collections::HashMap<String, serde_json::Value>,
    prefix: &str,
    result: &mut ValidationResult,
) {
    if let Some(max_seq_len) = parameters.get("max_sequence_length") {
        if let Some(len) = max_seq_len.as_u64() {
            if len == 0 || len > 256 {
                result.add_error(format!(
                    "{}: max_sequence_length must be between 1 and 256",
                    prefix
                ));
            }
        }
    }

    if let Some(model_name) = parameters.get("model_name") {
        if let Some(name) = model_name.as_str() {
            if name.is_empty() {
                result.add_error(format!("{}: model_name cannot be empty", prefix));
            }
        }
    }
}

/// Validate CharNGram parameters
fn validate_charngram_parameters(
    parameters: &std::collections::HashMap<String, serde_json::Value>,
    prefix: &str,
    result: &mut ValidationResult,
) {
    if let Some(n) = parameters.get("n") {
        if let Some(n_val) = n.as_u64() {
            if n_val == 0 || n_val > 10 {
                result.add_error(format!("{}: n must be between 1 and 10", prefix));
            }
        }
    }

    if let Some(vocab_size) = parameters.get("vocab_size") {
        if let Some(size) = vocab_size.as_u64() {
            if size == 0 {
                result.add_error(format!("{}: vocab_size must be greater than 0", prefix));
            }
        }
    }
}

/// Validate BM25 parameters
fn validate_bm25_parameters(
    parameters: &std::collections::HashMap<String, serde_json::Value>,
    prefix: &str,
    result: &mut ValidationResult,
) {
    if let Some(k1) = parameters.get("k1") {
        if let Some(k1_val) = k1.as_f64() {
            if k1_val < 0.0 {
                result.add_error(format!("{}: BM25 k1 parameter must be >= 0", prefix));
            }
        }
    }

    if let Some(b) = parameters.get("b") {
        if let Some(b_val) = b.as_f64() {
            if b_val < 0.0 || b_val > 1.0 {
                result.add_error(format!(
                    "{}: BM25 b parameter must be between 0 and 1",
                    prefix
                ));
            }
        }
    }
}

/// Validate real model parameters
fn validate_real_model_parameters(
    parameters: &std::collections::HashMap<String, serde_json::Value>,
    prefix: &str,
    result: &mut ValidationResult,
) {
    if let Some(model_type) = parameters.get("model_type") {
        if let Some(model_str) = model_type.as_str() {
            if model_str.is_empty() {
                result.add_error(format!("{}: model_type cannot be empty", prefix));
            }
        }
    }

    if let Some(cache_dir) = parameters.get("cache_dir") {
        if let Some(dir) = cache_dir.as_str() {
            if dir.is_empty() {
                result.add_warning(format!("{}: cache_dir is empty, using default", prefix));
            }
        }
    }
}

/// Validate ONNX model parameters
fn validate_onnx_model_parameters(
    parameters: &std::collections::HashMap<String, serde_json::Value>,
    prefix: &str,
    result: &mut ValidationResult,
) {
    if let Some(model_type) = parameters.get("model_type") {
        if let Some(model_str) = model_type.as_str() {
            if model_str.is_empty() {
                result.add_error(format!("{}: model_type cannot be empty", prefix));
            }
        }
    }

    if let Some(batch_size) = parameters.get("batch_size") {
        if let Some(size) = batch_size.as_u64() {
            if size == 0 {
                result.add_error(format!("{}: batch_size must be greater than 0", prefix));
            }
        }
    }

    if let Some(num_threads) = parameters.get("num_threads") {
        if let Some(threads) = num_threads.as_u64() {
            if threads == 0 {
                result.add_error(format!("{}: num_threads must be greater than 0", prefix));
            }
        }
    }
}

/// Validate collection defaults
fn validate_collection_defaults(defaults: &CollectionDefaults, result: &mut ValidationResult) {
    debug!("Validating collection defaults");

    // Validate compression settings
    if defaults.compression.threshold_bytes == 0 {
        result.add_error("Global compression threshold_bytes must be greater than 0".to_string());
    }

    if defaults.compression.algorithm.is_empty() {
        result.add_error("Global compression algorithm cannot be empty".to_string());
    }
}

/// Validate indexing defaults
fn validate_indexing_defaults(defaults: &IndexingDefaults, result: &mut ValidationResult) {
    debug!("Validating indexing defaults");

    if defaults.index_type.is_empty() {
        result.add_error("Global index_type cannot be empty".to_string());
    }
}

/// Validate processing defaults
fn validate_processing_defaults(defaults: &ProcessingDefaults, result: &mut ValidationResult) {
    debug!("Validating processing defaults");

    if defaults.chunk_size == 0 {
        result.add_error("Global chunk_size must be greater than 0".to_string());
    }

    if defaults.chunk_overlap >= defaults.chunk_size {
        result.add_error("Global chunk_overlap must be less than chunk_size".to_string());
    }

    if defaults.max_file_size_mb == 0 {
        result.add_error("Global max_file_size_mb must be greater than 0".to_string());
    }

    if defaults.supported_extensions.is_empty() {
        result.add_warning("No supported file extensions configured".to_string());
    }
}

/// Validate indexing configuration
fn validate_indexing_config(
    indexing: &IndexingConfig,
    prefix: &str,
    result: &mut ValidationResult,
) {
    debug!("Validating indexing configuration: {}", prefix);

    if indexing.index_type.is_empty() {
        result.add_error(format!("{}: index_type cannot be empty", prefix));
    }

    // Validate HNSW parameters
    if indexing.index_type == "hnsw" {
        validate_hnsw_parameters(&indexing.parameters, prefix, result);
    }
}

/// Validate HNSW parameters
fn validate_hnsw_parameters(
    parameters: &std::collections::HashMap<String, serde_json::Value>,
    prefix: &str,
    result: &mut ValidationResult,
) {
    if let Some(m) = parameters.get("m") {
        if let Some(m_val) = m.as_u64() {
            if m_val == 0 {
                result.add_error(format!(
                    "{}: HNSW m parameter must be greater than 0",
                    prefix
                ));
            }
        }
    }

    if let Some(ef_construction) = parameters.get("ef_construction") {
        if let Some(ef_val) = ef_construction.as_u64() {
            if ef_val == 0 {
                result.add_error(format!(
                    "{}: HNSW ef_construction must be greater than 0",
                    prefix
                ));
            }
        }
    }

    if let Some(ef_search) = parameters.get("ef_search") {
        if let Some(ef_val) = ef_search.as_u64() {
            if ef_val == 0 {
                result.add_error(format!("{}: HNSW ef_search must be greater than 0", prefix));
            }
        }
    }
}

/// Validate collection processing configuration
fn validate_collection_processing(
    processing: &CollectionProcessing,
    prefix: &str,
    result: &mut ValidationResult,
) {
    debug!("Validating collection processing: {}", prefix);

    if processing.chunk_size == 0 {
        result.add_error(format!("{}: chunk_size must be greater than 0", prefix));
    }

    if processing.chunk_overlap >= processing.chunk_size {
        result.add_error(format!(
            "{}: chunk_overlap must be less than chunk_size",
            prefix
        ));
    }

    if processing.include_patterns.is_empty() {
        result.add_warning(format!(
            "{}: No include patterns specified, all files will be processed",
            prefix
        ));
    }
}

/// Validate processing settings
fn validate_processing_settings(config: &WorkspaceConfig, result: &mut ValidationResult) {
    debug!("Validating processing settings");

    if config.processing.max_concurrent_projects == 0 {
        result.add_error("max_concurrent_projects must be greater than 0".to_string());
    }

    if config.processing.max_concurrent_collections == 0 {
        result.add_error("max_concurrent_collections must be greater than 0".to_string());
    }

    if config.processing.file_processing.batch_size == 0 {
        result.add_error("file_processing.batch_size must be greater than 0".to_string());
    }

    if config.processing.memory.max_memory_usage_gb <= 0.0 {
        result.add_error("memory.max_memory_usage_gb must be greater than 0".to_string());
    }
}

/// Validate monitoring settings
fn validate_monitoring_settings(config: &WorkspaceConfig, result: &mut ValidationResult) {
    debug!("Validating monitoring settings");

    if config.monitoring.health_check.interval_seconds == 0 {
        result.add_error("health_check.interval_seconds must be greater than 0".to_string());
    }

    if config.monitoring.metrics.collection_interval_seconds == 0 {
        result.add_error("metrics.collection_interval_seconds must be greater than 0".to_string());
    }

    if config.monitoring.logging.level.is_empty() {
        result.add_error("logging.level cannot be empty".to_string());
    }
}

/// Validate validation settings
fn validate_validation_settings(_config: &WorkspaceConfig, _result: &mut ValidationResult) {
    debug!("Validating validation settings");

    // No specific validation needed for validation settings
    // They are just configuration flags
}

#[cfg(test)]
mod tests {
    use tempfile::tempdir;

    use super::*;

    #[test]
    fn test_validate_workspace_config() {
        let temp_dir = tempdir().unwrap();
        let config = WorkspaceConfig::default();

        let result = validate_workspace_config(&config, temp_dir.path());
        assert!(result.is_valid());
    }

    #[test]
    fn test_validate_workspace_config_with_errors() {
        let temp_dir = tempdir().unwrap();
        let mut config = WorkspaceConfig::default();

        // Add invalid project
        config.projects.push(ProjectConfig {
            name: "".to_string(),                          // Invalid: empty name
            path: std::path::PathBuf::from("nonexistent"), // Invalid: path doesn't exist
            description: "Test project".to_string(),
            enabled: true,
            embedding: None,
            collections: vec![],
        });

        let result = validate_workspace_config(&config, temp_dir.path());
        assert!(!result.is_valid());
        assert!(!result.errors.is_empty());
    }

    #[test]
    fn test_validation_result() {
        let mut result = ValidationResult::new();
        assert!(result.is_valid());

        result.add_error("Test error".to_string());
        assert!(!result.is_valid());
        assert_eq!(result.errors.len(), 1);

        result.add_warning("Test warning".to_string());
        assert_eq!(result.warnings.len(), 1);
    }
}
