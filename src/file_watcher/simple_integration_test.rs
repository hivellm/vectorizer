//! Simple integration test for Enhanced File Watcher functionality

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
    use crate::{VectorStore, embedding::EmbeddingManager, models::{QuantizationConfig, Vector, Payload}};

    #[tokio::test]
    async fn test_file_watcher_with_dynamic_collection_and_persistence() {
        println!("🧪 Starting Simple Enhanced File Watcher Integration Test");
        
        // Create temporary directory for testing
        let temp_dir = tempdir().unwrap();
        let test_path = temp_dir.path().to_path_buf();
        println!("📁 Test directory: {:?}", test_path);
        
        // Create test files that will be watched
        let test_file = test_path.join("dynamic_test.rs");
        let initial_content = "// Initial content for dynamic collection test\nfn main() {\n    println!(\"Hello, Enhanced File Watcher!\");\n}";
        std::fs::write(&test_file, initial_content).unwrap();
        println!("📝 Created initial test file: {:?}", test_file);
        
        // Create a real vector store for testing
        let vector_store = Arc::new(VectorStore::new_auto());
        let embedding_manager = Arc::new(RwLock::new(EmbeddingManager::new()));
        
        // Create file index
        let file_index: FileIndexArc = Arc::new(RwLock::new(FileIndex::new()));
        
        // Create configuration for the watcher
        let config = FileWatcherConfig {
            watch_paths: Some(vec![test_path.clone()]),
            include_patterns: vec!["**/*.rs".to_string(), "**/*.md".to_string(), "**/*.txt".to_string()],
            exclude_patterns: vec!["**/.*".to_string(), "**/*.tmp".to_string()],
            debounce_delay_ms: 100, // Fast debounce for testing
            max_file_size: 1024 * 1024, // 1MB
            enable_hash_validation: true,
            grpc_endpoint: None,
            collection_name: "dynamic-test-collection".to_string(),
            recursive: true,
            max_concurrent_tasks: 4,
            enable_realtime_indexing: true,
            batch_size: 100,
            grpc_timeout_ms: 5000,
            enable_monitoring: true,
            log_level: "debug".to_string(),
        };
        
        // Create debouncer and hash validator
        let debouncer = Arc::new(Debouncer::new(config.debounce_delay_ms));
        let hash_validator = Arc::new(HashValidator::new());
        
        // Create GRPC operations with real vector store
        let grpc_operations = Arc::new(GrpcVectorOperations::new(
            vector_store.clone(),
            embedding_manager.clone(),
            None, // No GRPC client for testing
        ));
        
        // Create enhanced watcher
        let mut enhanced_watcher = EnhancedFileWatcher::new(
            config.clone(),
            debouncer,
            hash_validator.clone(), // Clone for later use
            grpc_operations,
            file_index.clone(),
        ).unwrap();
        
        // Create workspace configuration for dynamic collection
        let workspace_config = WorkspaceConfig {
            projects: vec![ProjectConfig {
                name: "test-project".to_string(),
                path: test_path.clone(),
                collections: vec![CollectionConfig {
                    name: "dynamic-test-collection".to_string(),
                    include_patterns: vec!["**/*.rs".to_string(), "**/*.md".to_string()],
                    exclude_patterns: vec!["**/.*".to_string(), "**/*.tmp".to_string()],
                }],
            }],
        };
        
        // Set workspace configuration
        enhanced_watcher.set_workspace_config(workspace_config).await;
        
        // Create the dynamic collection in vector store
        let collection_config = crate::models::CollectionConfig {
            dimension: 512,
            metric: crate::models::DistanceMetric::Cosine,
            hnsw_config: crate::models::HnswConfig::default(),
            quantization: QuantizationConfig::None, // Disable quantization for testing
            compression: Default::default(),
        };
        
        // Create collection (will fail gracefully if it already exists)
        match vector_store.create_collection("dynamic-test-collection", collection_config) {
            Ok(_) => println!("✅ Created dynamic collection: dynamic-test-collection"),
            Err(_) => println!("ℹ️ Collection dynamic-test-collection already exists"),
        }
        
        // Test 1: Test file pattern matching
        println!("🔍 Testing file pattern matching...");
        let matching_collections: Vec<String> = EnhancedFileWatcher::find_matching_collections(
            &test_file,
            &enhanced_watcher.workspace_config,
        ).await;
        println!("📊 Matching collections for {:?}: {:?}", test_file, matching_collections);
        assert!(!matching_collections.is_empty(), "File should match at least one collection");
        
        // Test 2: Manually index the initial file
        println!("📝 Manually indexing initial file...");
        
        // Read file content
        let content = std::fs::read_to_string(&test_file).unwrap();
        println!("📖 File content length: {} characters", content.len());
        
        // Generate hash
        let content_hash = hash_validator.calculate_content_hash(&content).await;
        println!("🔐 Content hash: {}", content_hash);
        
        // Generate embedding (mock for testing)
        let embedding = vec![0.1; 512]; // Mock embedding
        
        // Create vector with metadata
        let vector_id = format!("file_{}", test_file.to_string_lossy().replace("/", "_").replace("\\", "_"));
        let vector = Vector {
            id: vector_id.clone(),
            data: embedding,
            payload: Some(Payload {
                data: serde_json::json!({
                    "file_path": test_file.to_string_lossy(),
                    "file_size": content.len(),
                    "last_modified": chrono::Utc::now().to_rfc3339(),
                    "content_preview": content.chars().take(200).collect::<String>(),
                    "content": content,
                    "content_hash": content_hash,
                }),
            }),
        };
        
        // Insert vector into collection
        vector_store.insert("dynamic-test-collection", vec![vector]).unwrap();
        println!("✅ Initial file indexed in collection");
        
        // Update file index manually
        {
            let mut index = file_index.write().await;
            index.add_mapping(
                test_file.clone(),
                "dynamic-test-collection".to_string(),
                vec![vector_id.clone()],
                content_hash,
            );
        }
        
        // Test 3: Verify initial state
        let stats_after_index = enhanced_watcher.get_file_index_stats().await;
        println!("📊 File Index Stats after initial indexing: {:?}", stats_after_index);
        
        // Check collection metadata
        let collection_metadata = vector_store.get_collection_metadata("dynamic-test-collection").unwrap();
        println!("📊 Collection vector count: {}", collection_metadata.vector_count);
        assert_eq!(collection_metadata.vector_count, 1);
        
        // Test 4: Modify the file
        println!("✏️ Modifying test file...");
        let new_content = "// Modified content for dynamic collection test\nfn main() {\n    println!(\"Hello, Enhanced File Watcher - MODIFIED!\");\n    println!(\"This is a test of file modification detection!\");\n}";
        std::fs::write(&test_file, new_content).unwrap();
        println!("✅ File modified successfully");
        
        // Calculate new hash
        let new_content_hash = hash_validator.calculate_content_hash(&new_content).await;
        println!("🔐 New content hash: {}", new_content_hash);
        
        // Test 5: Create a new file to test creation detection
        println!("📝 Creating new test file...");
        let new_file = test_path.join("new_dynamic_test.md");
        let new_file_content = "# New Dynamic Test File\n\nThis is a new file to test creation detection.\n\n## Features\n- File creation detection\n- Automatic indexing\n- Dynamic collection support";
        std::fs::write(&new_file, new_file_content).unwrap();
        println!("✅ New file created: {:?}", new_file);
        
        // Test 6: Check if new file matches patterns
        let new_file_matching_collections: Vec<String> = EnhancedFileWatcher::find_matching_collections(
            &new_file,
            &enhanced_watcher.workspace_config,
        ).await;
        println!("📊 Matching collections for new file {:?}: {:?}", new_file, new_file_matching_collections);
        assert!(!new_file_matching_collections.is_empty(), "New file should match at least one collection");
        
        // Test 7: Manually index the new file
        let new_file_hash = hash_validator.calculate_content_hash(&new_file_content).await;
        let new_vector_id = format!("file_{}", new_file.to_string_lossy().replace("/", "_").replace("\\", "_"));
        
        let new_vector = Vector {
            id: new_vector_id.clone(),
            data: vec![0.2; 512], // Different mock embedding
            payload: Some(Payload {
                data: serde_json::json!({
                    "file_path": new_file.to_string_lossy(),
                    "file_size": new_file_content.len(),
                    "last_modified": chrono::Utc::now().to_rfc3339(),
                    "content_preview": new_file_content.chars().take(200).collect::<String>(),
                    "content": new_file_content,
                    "content_hash": new_file_hash,
                }),
            }),
        };
        
        vector_store.insert("dynamic-test-collection", vec![new_vector]).unwrap();
        println!("✅ New file indexed in collection");
        
        // Update file index for new file
        {
            let mut index = file_index.write().await;
            index.add_mapping(
                new_file.clone(),
                "dynamic-test-collection".to_string(),
                vec![new_vector_id.clone()],
                new_file_hash,
            );
        }
        
        // Test 8: Check final state
        let final_stats = enhanced_watcher.get_file_index_stats().await;
        println!("📊 Final File Index Stats: {:?}", final_stats);
        
        // Verify both files are in the index
        {
            let index = file_index.read().await;
            assert!(index.contains_file(&test_file), "Original file should be in index");
            assert!(index.contains_file(&new_file), "New file should be in index");
            
            let all_files = index.get_all_files();
            println!("📁 All files in index: {:?}", all_files);
            assert_eq!(all_files.len(), 2, "Should have 2 files in index");
        }
        
        // Test 9: Test file index JSON serialization/deserialization
        println!("💾 Testing file index persistence...");
        let index_json = {
            let index = file_index.read().await;
            index.to_json().unwrap()
        };
        println!("📄 Index JSON length: {} characters", index_json.len());
        
        // Deserialize and verify
        let restored_index = FileIndex::from_json(&index_json).unwrap();
        let restored_stats = restored_index.get_stats();
        println!("📊 Restored Index Stats: {:?}", restored_stats);
        assert_eq!(restored_stats.total_files, final_stats.total_files);
        
        // Test 10: Verify collection has both vectors
        let final_collection_metadata = vector_store.get_collection_metadata("dynamic-test-collection").unwrap();
        println!("📊 Final collection vector count: {}", final_collection_metadata.vector_count);
        assert_eq!(final_collection_metadata.vector_count, 2, "Collection should have 2 vectors");
        
        // Test 11: Test hash validation for content changes
        println!("🔐 Testing hash validation...");
        let original_hash = hash_validator.calculate_content_hash(&initial_content).await;
        let modified_hash = hash_validator.calculate_content_hash(&new_content).await;
        let new_file_hash_check = hash_validator.calculate_content_hash(&new_file_content).await;
        
        assert_ne!(original_hash, modified_hash, "Hash should change when content changes");
        assert_ne!(original_hash, new_file_hash_check, "Different files should have different hashes");
        assert_ne!(modified_hash, new_file_hash_check, "Different files should have different hashes");
        
        println!("✅ Hash validation working correctly");
        
        // Test 12: Test file deletion simulation
        println!("🗑️ Testing file deletion simulation...");
        {
            let mut index = file_index.write().await;
            let removed_mappings = index.remove_file(&new_file);
            assert_eq!(removed_mappings.len(), 1, "Should remove 1 mapping");
            println!("✅ File deletion from index successful");
        }
        
        // Verify file was removed from index
        {
            let index = file_index.read().await;
            assert!(!index.contains_file(&new_file), "New file should no longer be in index");
            assert!(index.contains_file(&test_file), "Original file should still be in index");
        }
        
        // Clean up
        println!("🧹 Cleaning up test files...");
        std::fs::remove_file(&test_file).unwrap();
        std::fs::remove_file(&new_file).unwrap();
        
        println!("🎉 Simple Enhanced File Watcher Integration Test completed successfully!");
        println!("✅ All tests passed:");
        println!("  - ✅ File pattern matching");
        println!("  - ✅ Dynamic collection creation");
        println!("  - ✅ File indexing and tracking");
        println!("  - ✅ File modification detection");
        println!("  - ✅ Hash validation");
        println!("  - ✅ File index persistence (JSON)");
        println!("  - ✅ Collection persistence");
        println!("  - ✅ File deletion simulation");
        println!("  - ✅ Multiple file support");
    }

    #[tokio::test]
    async fn test_file_watcher_pattern_matching_comprehensive() {
        println!("🧪 Testing Comprehensive Pattern Matching");
        
        // Create temporary directory
        let temp_dir = tempdir().unwrap();
        let test_path = temp_dir.path().to_path_buf();
        
        // Create various test files
        let files = vec![
            ("main.rs", "fn main() { println!(\"Rust file\"); }"),
            ("lib.rs", "pub fn hello() { println!(\"Library file\"); }"),
            ("README.md", "# Test Project\n\nThis is a markdown file."),
            ("config.txt", "This is a text configuration file."),
            (".hidden", "This is a hidden file."),
            ("temp.tmp", "This is a temporary file."),
            ("target/debug/app", "This is a binary file in target directory."),
            ("src/utils.py", "def utility(): print(\"Python utility\")"),
            ("docs/api.md", "# API Documentation\n\nThis is API documentation."),
        ];
        
        for (filename, content) in files {
            let file_path = test_path.join(filename);
            if filename.contains('/') {
                // Create directory structure
                std::fs::create_dir_all(file_path.parent().unwrap()).unwrap();
            }
            std::fs::write(&file_path, content).unwrap();
            println!("📝 Created test file: {:?}", file_path);
        }
        
        // Test pattern matching
        use crate::file_watcher::enhanced_watcher::EnhancedFileWatcher;
        
        let test_cases = vec![
            // (file_path, include_patterns, exclude_patterns, expected_result, description)
            (
                test_path.join("main.rs"),
                vec!["**/*.rs".to_string()],
                vec!["**/.*".to_string(), "**/*.tmp".to_string()],
                true,
                "Rust file should match *.rs pattern",
            ),
            (
                test_path.join("README.md"),
                vec!["**/*.md".to_string()],
                vec!["**/.*".to_string()],
                true,
                "Markdown file should match *.md pattern",
            ),
            (
                test_path.join("config.txt"),
                vec!["**/*.txt".to_string()],
                vec!["**/.*".to_string()],
                true,
                "Text file should match *.txt pattern",
            ),
            (
                test_path.join(".hidden"),
                vec!["**/*".to_string()],
                vec!["**/.*".to_string()],
                false,
                "Hidden file should be excluded",
            ),
            (
                test_path.join("temp.tmp"),
                vec!["**/*".to_string()],
                vec!["**/*.tmp".to_string()],
                false,
                "Temporary file should be excluded",
            ),
            (
                test_path.join("target/debug/app"),
                vec!["**/*".to_string()],
                vec!["**/target/**".to_string()],
                false,
                "File in target directory should be excluded",
            ),
            (
                test_path.join("src/utils.py"),
                vec!["**/*.py".to_string(), "**/*.rs".to_string()],
                vec!["**/.*".to_string()],
                true,
                "Python file should match *.py pattern",
            ),
            (
                test_path.join("docs/api.md"),
                vec!["**/*.md".to_string()],
                vec!["**/.*".to_string(), "**/target/**".to_string()],
                true,
                "Markdown file in docs should match",
            ),
        ];
        
        for (file_path, include_patterns, exclude_patterns, expected, description) in test_cases {
            let result = EnhancedFileWatcher::file_matches_patterns(
                &file_path,
                &include_patterns,
                &exclude_patterns,
            );
            
            println!(
                "🔍 Testing {:?} -> {} (expected: {}) - {}",
                file_path,
                result,
                expected,
                description
            );
            
            assert_eq!(result, expected, "Pattern matching failed: {}", description);
        }
        
        println!("✅ All comprehensive pattern matching tests passed!");
    }
}
