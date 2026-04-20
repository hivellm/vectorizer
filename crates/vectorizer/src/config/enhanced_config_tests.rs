//! Extracted unit tests (phase3 test-extraction).
//!
//! Wired from `src/config/enhanced_config.rs` via the `#[path]` attribute.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use super::*;



#[tokio::test]
async fn test_config_loading() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("config.yaml");

    let config_content = r#"
workspace:
  name: "Test Workspace"
  version: "1.0.0"
  description: "Test workspace"
  created_at: "2024-01-01T00:00:00Z"
  last_updated: "2024-01-01T00:00:00Z"
global:
  default_embedding:
model: "bm25"
dimension: 384
parameters: {}
  default_collection:
metric: "cosine"
quantization:
  type: "sq"
  bits: 8
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
supported_extensions: [".md", ".txt", ".rs"]
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
log_file: "./.logs/workspace.log"
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

    fs::write(&config_path, config_content).await.unwrap();

    let mut config_manager = EnhancedConfigManager::new(config_path);
    config_manager.load_config().await.unwrap();

    let config = config_manager.get_config();
    assert_eq!(config.workspace.name, "Test Workspace");
    assert_eq!(config.workspace.version, "1.0.0");
}

#[tokio::test]
async fn test_env_variable_substitution() {
    // SAFETY: test runs in a `#[tokio::test]` task with `flavor = "current_thread"`
    // by default; no concurrent threads are reading `env` here, and cargo test
    // isolates per-test process state. `std::env::set_var` was marked unsafe in
    // Rust 1.80 purely because the stdlib can't guard against reads from other
    // threads, which is not a concern in this single-threaded test.
    unsafe {
        std::env::set_var("WORKSPACE_NAME", "Env Test Workspace");
        std::env::set_var("EMBEDDING_DIMENSION", "512");
    }

    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("config.yaml");

    let config_content = r#"
workspace:
  name: "${WORKSPACE_NAME}"
  version: "1.0.0"
  description: "Test workspace"
  created_at: "2024-01-01T00:00:00Z"
  last_updated: "2024-01-01T00:00:00Z"
global:
  default_embedding:
model: "bm25"
dimension: ${EMBEDDING_DIMENSION}
parameters: {}
  default_collection:
metric: "cosine"
quantization:
  type: "sq"
  bits: 8
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
supported_extensions: [".md", ".txt", ".rs"]
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
log_file: "./.logs/workspace.log"
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

    fs::write(&config_path, config_content).await.unwrap();

    let mut config_manager = EnhancedConfigManager::new(config_path);
    config_manager.load_config().await.unwrap();

    let config = config_manager.get_config();
    assert_eq!(config.workspace.name, "Env Test Workspace");
    assert_eq!(config.global.default_embedding.dimension, 512);
}

#[test]
fn test_config_export_env_vars() {
    let config_manager = EnhancedConfigManager::new(PathBuf::from("test.yaml"));
    let env_vars = config_manager.get_config_as_env_vars();

    assert!(env_vars.contains_key("WORKSPACE_NAME"));
    assert!(env_vars.contains_key("WORKSPACE_VERSION"));
    assert!(env_vars.contains_key("DEFAULT_EMBEDDING_MODEL"));
    assert!(env_vars.contains_key("PARALLEL_PROCESSING"));
}
