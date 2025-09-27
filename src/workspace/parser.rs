//! Workspace configuration parser
//!
//! Provides functionality to parse and load workspace configuration files

use crate::error::{Result, VectorizerError};
use crate::workspace::config::WorkspaceConfig;
use serde_yaml;
use std::fs;
use std::path::Path;
use tracing::{debug, info};

/// Parse workspace configuration from YAML file
pub fn parse_workspace_config<P: AsRef<Path>>(path: P) -> Result<WorkspaceConfig> {
    let path = path.as_ref();

    debug!("Parsing workspace configuration from: {}", path.display());

    // Check if file exists
    if !path.exists() {
        return Err(VectorizerError::IoError(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            format!("Workspace configuration file not found: {}", path.display()),
        )));
    }

    // Read file content
    let content = fs::read_to_string(path)?;

    debug!("Read {} bytes from workspace config file", content.len());

    // Parse YAML
    let config: WorkspaceConfig = serde_yaml::from_str(&content)?;

    debug!("Successfully parsed workspace configuration");
    debug!(
        "Workspace: {}, Projects: {}",
        config.workspace.name,
        config.projects.len()
    );

    Ok(config)
}

/// Parse workspace configuration from string content
pub fn parse_workspace_config_from_str(content: &str) -> Result<WorkspaceConfig> {
    debug!("Parsing workspace configuration from string content");

    let config: WorkspaceConfig = serde_yaml::from_str(content)?;

    info!("Successfully parsed workspace configuration from string");
    debug!(
        "Workspace: {}, Projects: {}",
        config.workspace.name,
        config.projects.len()
    );

    Ok(config)
}

/// Save workspace configuration to YAML file
pub fn save_workspace_config<P: AsRef<Path>>(config: &WorkspaceConfig, path: P) -> Result<()> {
    let path = path.as_ref();

    info!("Saving workspace configuration to: {}", path.display());

    // Create parent directories if they don't exist
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    // Serialize to YAML
    let yaml_content = serde_yaml::to_string(config)?;

    // Write to file
    fs::write(path, yaml_content)?;

    info!("Successfully saved workspace configuration");
    Ok(())
}

/// Find workspace configuration file in directory hierarchy
pub fn find_workspace_config<P: AsRef<Path>>(start_path: P) -> Result<Option<std::path::PathBuf>> {
    let start_path = start_path.as_ref();

    debug!(
        "Searching for workspace config starting from: {}",
        start_path.display()
    );

    let mut current_path = start_path.to_path_buf();

    loop {
        let config_path = current_path.join("vectorize-workspace.yml");

        if config_path.exists() {
            info!("Found workspace config: {}", config_path.display());
            return Ok(Some(config_path));
        }

        // Move up one directory
        if let Some(parent) = current_path.parent() {
            current_path = parent.to_path_buf();
        } else {
            // Reached root directory
            break;
        }
    }

    debug!("No workspace config found in directory hierarchy");
    Ok(None)
}

/// Create default workspace configuration
pub fn create_default_workspace_config() -> WorkspaceConfig {
    info!("Creating default workspace configuration");
    WorkspaceConfig::default()
}

/// Validate workspace configuration file exists and is readable
pub fn validate_workspace_config_file<P: AsRef<Path>>(path: P) -> Result<()> {
    let path = path.as_ref();

    debug!("Validating workspace config file: {}", path.display());

    // Check if file exists
    if !path.exists() {
        return Err(VectorizerError::IoError(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            format!("Workspace configuration file not found: {}", path.display()),
        )));
    }

    // Check if file is readable
    if !path.is_file() {
        return Err(VectorizerError::IoError(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            format!("Path is not a file: {}", path.display()),
        )));
    }

    // Try to read file
    fs::read_to_string(path)?;

    info!("Workspace config file validation successful");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_parse_workspace_config_from_str() {
        let yaml_content = r#"
workspace:
  name: "Test Workspace"
  version: "1.0.0"
  description: "Test workspace"
  created_at: "2024-01-01T00:00:00Z"
  last_updated: "2024-01-01T00:00:00Z"

global:
  default_embedding:
    model: "native_bow"
    dimension: 384
    parameters: {}
  default_collection:
    metric: "cosine"
    compression:
      enabled: true
      threshold_bytes: 1024
      algorithm: "lz4"
  default_indexing:
    index_type: "hnsw"
    parameters: {}
  processing:
    chunk_size: 2048
    chunk_overlap: 256
    max_file_size_mb: 10
    supported_extensions: [".md", ".txt"]

projects: []

processing:
  parallel_processing: true
  max_concurrent_projects: 4
  max_concurrent_collections: 8
  file_processing:
    batch_size: 100
    max_file_size_mb: 10
    skip_hidden_files: true
    skip_binary_files: true
  memory:
    max_memory_usage_gb: 8.0
    gc_threshold_mb: 1024
  error_handling:
    max_retries: 3
    retry_delay_seconds: 5
    continue_on_error: true
    log_errors: true

monitoring:
  health_check:
    enabled: true
    interval_seconds: 60
    check_projects: true
    check_collections: true
  metrics:
    enabled: true
    collection_interval_seconds: 300
    project_metrics: ["file_count", "total_size_mb"]
    collection_metrics: ["vector_count", "index_size_mb"]
  logging:
    level: "info"
    log_file: "./logs/workspace.log"
    max_log_size_mb: 100
    max_log_files: 5

validation:
  paths:
    validate_existence: true
    validate_permissions: true
    create_missing_dirs: false
  config:
    validate_embedding_models: true
    validate_dimensions: true
    validate_collections: true
  data:
    validate_file_types: true
    validate_file_sizes: true
    validate_encoding: true
"#;

        let result = parse_workspace_config_from_str(yaml_content);
        assert!(result.is_ok());

        let config = result.unwrap();
        assert_eq!(config.workspace.name, "Test Workspace");
        assert_eq!(config.projects.len(), 0);
    }

    #[test]
    fn test_save_and_load_workspace_config() {
        let temp_dir = tempdir().unwrap();
        let config_path = temp_dir.path().join("test-workspace.yml");

        let config = create_default_workspace_config();

        // Save config
        let save_result = save_workspace_config(&config, &config_path);
        assert!(save_result.is_ok());

        // Load config
        let load_result = parse_workspace_config(&config_path);
        assert!(load_result.is_ok());

        let loaded_config = load_result.unwrap();
        assert_eq!(loaded_config.workspace.name, config.workspace.name);
    }

    #[test]
    fn test_find_workspace_config() {
        let temp_dir = tempdir().unwrap();
        let config_path = temp_dir.path().join("vectorize-workspace.yml");

        // Create a test config file
        let config = create_default_workspace_config();
        save_workspace_config(&config, &config_path).unwrap();

        // Find the config
        let result = find_workspace_config(temp_dir.path());
        assert!(result.is_ok());

        let found_path = result.unwrap();
        assert!(found_path.is_some());
        assert_eq!(found_path.unwrap(), config_path);
    }

    #[test]
    fn test_validate_workspace_config_file() {
        let temp_dir = tempdir().unwrap();
        let config_path = temp_dir.path().join("test-config.yml");

        // Test with non-existent file
        let result = validate_workspace_config_file(&config_path);
        assert!(result.is_err());

        // Create a test config file
        let config = create_default_workspace_config();
        save_workspace_config(&config, &config_path).unwrap();

        // Validate existing file
        let result = validate_workspace_config_file(&config_path);
        assert!(result.is_ok());
    }
}
