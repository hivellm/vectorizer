//! Integration test for file watcher with server

use std::fs;
use std::sync::Arc;

use tempfile::TempDir;
use tokio::sync::RwLock;

use super::{FileWatcherConfig, FileWatcherSystem};
use crate::VectorStore;
use crate::embedding::EmbeddingManager;

#[tokio::test]
async fn test_file_watcher_system_creation() {
    // Create vector store and embedding manager
    let vector_store = Arc::new(VectorStore::new());
    let mut embedding_manager = EmbeddingManager::new();

    // Set up a basic embedding provider
    let bm25 = crate::embedding::Bm25Embedding::new(512);
    embedding_manager.register_provider("bm25".to_string(), Box::new(bm25));
    embedding_manager.set_default_provider("bm25").unwrap();

    let embedding_manager = Arc::new(RwLock::new(embedding_manager));

    // Create file watcher config
    let config = FileWatcherConfig::default();

    // Create file watcher system
    let watcher_system = FileWatcherSystem::new(config, vector_store, embedding_manager);

    // Test that the system was created successfully
    assert_eq!(watcher_system.config().debounce_delay_ms, 1000);
    assert_eq!(watcher_system.config().collection_name, "watched_files");
    assert!(watcher_system.config().enable_realtime_indexing);

    println!("✅ File watcher system creation test passed!");
}

#[tokio::test]
async fn test_file_watcher_config_validation() {
    let config = FileWatcherConfig::default();

    // Test default values
    assert_eq!(config.debounce_delay_ms, 1000);
    assert_eq!(config.max_file_size, 10 * 1024 * 1024); // 10MB
    assert!(config.enable_hash_validation);
    assert_eq!(config.collection_name, "watched_files");
    assert!(config.recursive);
    assert_eq!(config.max_concurrent_tasks, 4);
    assert!(config.enable_realtime_indexing);
    assert_eq!(config.batch_size, 100);
    assert!(config.enable_monitoring);
    assert_eq!(config.log_level, "info");

    // Test include patterns - now loaded from workspace config
    // assert!(config.include_patterns.contains(&"*.md".to_string()));
    // assert!(config.include_patterns.contains(&"*.rs".to_string()));
    // assert!(config.include_patterns.contains(&"*.py".to_string()));

    // Test exclude patterns - now loaded from workspace config
    // assert!(config.exclude_patterns.contains(&"**/target/**".to_string()));
    // assert!(config.exclude_patterns.contains(&"**/node_modules/**".to_string()));
    // assert!(config.exclude_patterns.contains(&"**/.git/**".to_string()));

    println!("✅ File watcher config validation test passed!");
}

#[tokio::test]
async fn test_file_watcher_with_temp_directory() {
    // Create temporary directory
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("test.md");

    // Create test file
    fs::write(&test_file, "# Test File\n\nThis is a test markdown file.").unwrap();

    // Create vector store and embedding manager
    let vector_store = Arc::new(VectorStore::new());
    let mut embedding_manager = EmbeddingManager::new();

    // Set up a basic embedding provider
    let bm25 = crate::embedding::Bm25Embedding::new(512);
    embedding_manager.register_provider("bm25".to_string(), Box::new(bm25));
    embedding_manager.set_default_provider("bm25").unwrap();

    let embedding_manager = Arc::new(RwLock::new(embedding_manager));

    // Create file watcher config with temp directory
    let mut config = FileWatcherConfig::default();
    config.watch_paths = Some(vec![temp_dir.path().to_path_buf()]);

    // Create file watcher system
    let watcher_system = FileWatcherSystem::new(config, vector_store, embedding_manager);

    // Test that the system was created with the correct watch path
    assert!(watcher_system.config().watch_paths.is_some());
    let watch_paths = watcher_system.config().watch_paths.as_ref().unwrap();
    assert_eq!(watch_paths.len(), 1);
    assert_eq!(watch_paths[0], temp_dir.path());

    println!("✅ File watcher with temp directory test passed!");
}
