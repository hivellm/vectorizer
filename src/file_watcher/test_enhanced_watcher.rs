//! Integration tests for Enhanced File Watcher

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use std::sync::Arc;
    use std::time::Duration;
    use tokio::sync::RwLock;
    use tempfile::tempdir;
    use crate::file_watcher::{
        EnhancedFileWatcher, FileWatcherConfig, FileIndex, FileIndexArc,
        WorkspaceConfig, ProjectConfig, CollectionConfig
    };
    use crate::file_watcher::{
        debouncer::Debouncer, hash_validator::HashValidator, GrpcVectorOperations
    };

    #[tokio::test]
    async fn test_enhanced_file_watcher_creation() {
        // Create temporary directory for testing
        let temp_dir = tempdir().unwrap();
        let test_path = temp_dir.path().to_path_buf();
        
        // Create test files
        std::fs::write(test_path.join("test1.txt"), "Hello, World!").unwrap();
        std::fs::write(test_path.join("test2.md"), "# Test Markdown").unwrap();
        
        // Create configuration
        let config = FileWatcherConfig {
            watch_paths: Some(vec![test_path.clone()]),
            include_patterns: vec!["**/*.txt".to_string(), "**/*.md".to_string()],
            exclude_patterns: vec!["**/.*".to_string(), "**/*.tmp".to_string()],
            debounce_delay_ms: 500,
            max_file_size: 1024 * 1024, // 1MB
            enable_hash_validation: true,
            grpc_endpoint: None,
            collection_name: "test-collection".to_string(),
            recursive: true,
            max_concurrent_tasks: 4,
            enable_realtime_indexing: true,
            batch_size: 100,
            grpc_timeout_ms: 5000,
            enable_monitoring: true,
            log_level: "info".to_string(),
        };
        
        // Create file index
        let file_index: FileIndexArc = Arc::new(RwLock::new(FileIndex::new()));
        
        // Create debouncer
        let debouncer = Arc::new(Debouncer::new(config.debounce_delay_ms));
        
        // Create hash validator
        let hash_validator = Arc::new(HashValidator::new());
        
        // Create mock GRPC operations (without actual GRPC client)
        let grpc_operations = Arc::new(GrpcVectorOperations::new(
            Arc::new(crate::VectorStore::new_auto()),
            Arc::new(RwLock::new(crate::embedding::EmbeddingManager::new())),
            None, // No GRPC client for testing
        ));
        
        // Create enhanced watcher
        let enhanced_watcher = EnhancedFileWatcher::new(
            config.clone(),
            debouncer,
            hash_validator,
            grpc_operations,
            file_index.clone(),
        );
        
        assert!(enhanced_watcher.is_ok());
        println!("✅ Enhanced File Watcher created successfully");
    }

    #[tokio::test]
    async fn test_file_index_operations() {
        let file_index = FileIndex::new();
        
        // Test adding mappings
        let file_path = PathBuf::from("test.rs");
        let collection_name = "test-collection".to_string();
        let vector_ids = vec!["vec1".to_string(), "vec2".to_string()];
        let hash = "abc123".to_string();
        
        // Create a mutable reference for testing
        let mut file_index = file_index;
        file_index.add_mapping(
            file_path.clone(),
            collection_name.clone(),
            vector_ids.clone(),
            hash,
        );
        
        // Test retrieval
        assert!(file_index.contains_file(&file_path));
        
        let collections = file_index.get_collections_for_file(&file_path);
        assert_eq!(collections.len(), 1);
        assert_eq!(collections[0], collection_name);
        
        let retrieved_ids = file_index.get_vector_ids(&file_path, &collection_name).unwrap();
        assert_eq!(retrieved_ids, vector_ids);
        
        // Test statistics
        let stats = file_index.get_stats();
        assert_eq!(stats.total_files, 1);
        assert_eq!(stats.total_collections, 1);
        assert_eq!(stats.total_mappings, 1);
        
        println!("✅ File Index operations work correctly");
    }

    #[tokio::test]
    async fn test_pattern_matching() {
        use crate::file_watcher::enhanced_watcher::EnhancedFileWatcher;
        
        let test_cases = vec![
            (PathBuf::from("src/main.rs"), "**/*.rs", true),
            (PathBuf::from("src/main.py"), "**/*.rs", false),
            (PathBuf::from("docs/README.md"), "**/*.md", true),
            (PathBuf::from("test/file.txt"), "**/*.txt", true),
            (PathBuf::from(".hidden/file"), "**/*.txt", false),
        ];
        
        for (path, pattern, expected) in test_cases {
            let result = EnhancedFileWatcher::matches_pattern(&path, pattern);
            assert_eq!(result, expected, "Pattern matching failed for {:?} with pattern {}", path, pattern);
        }
        
        println!("✅ Pattern matching works correctly");
    }

    #[tokio::test]
    async fn test_file_patterns_matching() {
        use crate::file_watcher::enhanced_watcher::EnhancedFileWatcher;
        
        let file_path = PathBuf::from("src/main.rs");
        let include_patterns = vec!["**/*.rs".to_string(), "**/*.py".to_string()];
        let exclude_patterns = vec!["**/.*".to_string(), "**/*.tmp".to_string()];
        
        let result = EnhancedFileWatcher::file_matches_patterns(&file_path, &include_patterns, &exclude_patterns);
        assert!(result);
        
        let hidden_file = PathBuf::from(".hidden/secret.rs");
        let result_hidden = EnhancedFileWatcher::file_matches_patterns(&hidden_file, &include_patterns, &exclude_patterns);
        assert!(!result_hidden);
        
        println!("✅ File patterns matching works correctly");
    }

    #[tokio::test]
    async fn test_workspace_config() {
        let workspace_config = WorkspaceConfig {
            projects: vec![ProjectConfig {
                name: "test-project".to_string(),
                path: PathBuf::from("test"),
                collections: vec![CollectionConfig {
                    name: "test-collection".to_string(),
                    include_patterns: vec!["**/*.rs".to_string()],
                    exclude_patterns: vec!["**/.*".to_string()],
                }],
            }],
        };
        
        assert_eq!(workspace_config.projects.len(), 1);
        assert_eq!(workspace_config.projects[0].collections.len(), 1);
        assert_eq!(workspace_config.projects[0].collections[0].name, "test-collection");
        
        println!("✅ Workspace configuration works correctly");
    }

    #[tokio::test]
    async fn test_file_index_json_serialization() {
        let mut file_index = FileIndex::new();
        
        // Add some test data
        file_index.add_mapping(
            PathBuf::from("test1.rs"),
            "collection1".to_string(),
            vec!["vec1".to_string()],
            "hash1".to_string(),
        );
        
        file_index.add_mapping(
            PathBuf::from("test2.py"),
            "collection2".to_string(),
            vec!["vec2".to_string()],
            "hash2".to_string(),
        );
        
        // Test JSON serialization
        let json = file_index.to_json().unwrap();
        assert!(!json.is_empty());
        
        // Test JSON deserialization
        let deserialized = FileIndex::from_json(&json).unwrap();
        let stats = deserialized.get_stats();
        assert_eq!(stats.total_files, 2);
        assert_eq!(stats.total_collections, 2);
        
        println!("✅ File Index JSON serialization works correctly");
    }
}
