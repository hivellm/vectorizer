//! Test file for file watcher operations

use std::fs;
use std::sync::Arc;

use tempfile::TempDir;
use tokio::sync::RwLock;

use super::operations::VectorOperations;
use super::{FileChangeEvent, FileChangeEventWithMetadata};
use crate::VectorStore;
use crate::embedding::EmbeddingManager;



#[tokio::test]
async fn test_should_process_file() {
    // Create vector store and embedding manager
    let vector_store = Arc::new(VectorStore::new());
    let embedding_manager = Arc::new(RwLock::new(EmbeddingManager::new()));

    // Create vector operations with test config
    let mut config = crate::file_watcher::FileWatcherConfig::default();
    config.include_patterns = vec![
        "*.md".to_string(),
        "*.rs".to_string(),
        "*.py".to_string(),
        "*.js".to_string(),
        "*.ts".to_string(),
        "*.json".to_string(),
        "*.yaml".to_string(),
    ];
    config.exclude_patterns = vec!["*.exe".to_string(), "*.bin".to_string()];

    let hash_validator = Arc::new(crate::file_watcher::hash_validator::HashValidator::new());
    let operations = VectorOperations::new(vector_store, embedding_manager, config);

    // Test various file extensions
    assert!(operations.should_process_file(std::path::Path::new("test.md")));
    assert!(operations.should_process_file(std::path::Path::new("test.rs")));
    assert!(operations.should_process_file(std::path::Path::new("test.py")));
    assert!(operations.should_process_file(std::path::Path::new("test.js")));
    assert!(operations.should_process_file(std::path::Path::new("test.ts")));
    assert!(operations.should_process_file(std::path::Path::new("test.json")));
    assert!(operations.should_process_file(std::path::Path::new("test.yaml")));

    // Test files that should NOT be processed
    assert!(!operations.should_process_file(std::path::Path::new("test.exe")));
    assert!(!operations.should_process_file(std::path::Path::new("test.bin")));
    assert!(!operations.should_process_file(std::path::Path::new("test"))); // no extension

    println!("✅ File filtering test passed!");
}
