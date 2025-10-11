//! Test file for file watcher operations

use std::sync::Arc;
use tokio::sync::RwLock;
use tempfile::TempDir;
use std::fs;
use crate::VectorStore;
use crate::embedding::EmbeddingManager;
use super::operations::VectorOperations;
use super::{FileChangeEvent, FileChangeEventWithMetadata};

#[tokio::test]
async fn test_file_processing_basic() {
    // Create temporary directory
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("test.txt");
    
    // Create test file
    fs::write(&test_file, "Hello, World! This is a test file.").unwrap();
    
    // Create vector store and embedding manager
    let vector_store = Arc::new(VectorStore::new());
    let embedding_manager = Arc::new(RwLock::new(EmbeddingManager::new()));
    
    // Create vector operations
    let hash_validator = Arc::new(crate::file_watcher::HashValidator::new());
    let operations = VectorOperations::new(vector_store.clone(), embedding_manager, crate::file_watcher::FileWatcherConfig::default(), hash_validator);
    
    // Test file filtering logic
    assert!(operations.should_process_file(&test_file), "Text file should be processed");
    
    // Test collection name determination
    let collection_name = operations.determine_collection_name(&test_file);
    assert!(!collection_name.is_empty(), "Collection name should not be empty");
    
    println!("✅ File processing basic test passed!");
}

#[tokio::test]
async fn test_file_removal_basic() {
    // Create temporary directory
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("test.txt");
    
    // Create vector store and embedding manager
    let vector_store = Arc::new(VectorStore::new());
    let embedding_manager = Arc::new(RwLock::new(EmbeddingManager::new()));
    
    // Create vector operations
    let hash_validator = Arc::new(crate::file_watcher::HashValidator::new());
    let operations = VectorOperations::new(vector_store.clone(), embedding_manager, crate::file_watcher::FileWatcherConfig::default(), hash_validator);
    
    // Test file filtering for deletion
    assert!(operations.should_process_file(&test_file), "Text file should be processed");
    
    // Test collection name determination
    let collection_name = operations.determine_collection_name(&test_file);
    assert!(!collection_name.is_empty(), "Collection name should not be empty");
    
    println!("✅ File removal basic test passed!");
}

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
    config.exclude_patterns = vec![
        "*.exe".to_string(),
        "*.bin".to_string(),
    ];
    
    let hash_validator = Arc::new(crate::file_watcher::HashValidator::new());
    let operations = VectorOperations::new(vector_store, embedding_manager, config, hash_validator);
    
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
