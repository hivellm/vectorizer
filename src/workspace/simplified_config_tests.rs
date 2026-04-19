//! Extracted unit tests (phase3 test-extraction).
//!
//! Wired from `src/workspace/simplified_config.rs` via the `#[path]` attribute.

#![allow(clippy::unwrap_used, clippy::expect_used)]

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
