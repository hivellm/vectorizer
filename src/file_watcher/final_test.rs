//! Final comprehensive test for Enhanced File Watcher

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
    async fn test_enhanced_file_watcher_complete_workflow() {
        println!("🧪 Starting Complete Enhanced File Watcher Workflow Test");
        
        // Create temporary directory for testing
        let temp_dir = tempdir().unwrap();
        let test_path = temp_dir.path().to_path_buf();
        println!("📁 Test directory: {:?}", test_path);
        
        // Create test files
        let test_file = test_path.join("test.rs");
        let initial_content = "fn main() {\n    println!(\"Hello, Enhanced File Watcher!\");\n}";
        std::fs::write(&test_file, initial_content).unwrap();
        println!("📝 Created test file: {:?}", test_file);
        
        // Create vector store and components
        let vector_store = Arc::new(VectorStore::new_auto());
        let embedding_manager = Arc::new(RwLock::new(EmbeddingManager::new()));
        let file_index: FileIndexArc = Arc::new(RwLock::new(FileIndex::new()));
        
        // Create configuration
        let config = FileWatcherConfig {
            watch_paths: Some(vec![test_path.clone()]),
            include_patterns: vec!["**/*.rs".to_string(), "**/*.md".to_string()],
            exclude_patterns: vec!["**/.*".to_string(), "**/*.tmp".to_string()],
            debounce_delay_ms: 100,
            max_file_size: 1024 * 1024,
            enable_hash_validation: true,
            grpc_endpoint: None,
            collection_name: "test-collection".to_string(),
            recursive: true,
            max_concurrent_tasks: 4,
            enable_realtime_indexing: true,
            batch_size: 100,
            grpc_timeout_ms: 5000,
            enable_monitoring: true,
            log_level: "debug".to_string(),
        };
        
        // Create components
        let debouncer = Arc::new(Debouncer::new(config.debounce_delay_ms));
        let hash_validator = Arc::new(HashValidator::new());
        let grpc_operations = Arc::new(GrpcVectorOperations::new(
            vector_store.clone(),
            embedding_manager.clone(),
            None,
        ));
        
        // Create enhanced watcher
        let mut enhanced_watcher = EnhancedFileWatcher::new(
            config.clone(),
            debouncer,
            hash_validator.clone(),
            grpc_operations,
            file_index.clone(),
        ).unwrap();
        
        // Test 1: Create collection
        println!("🗄️ Creating test collection...");
        let collection_config = crate::models::CollectionConfig {
            dimension: 512,
            metric: crate::models::DistanceMetric::Cosine,
            hnsw_config: crate::models::HnswConfig::default(),
            quantization: QuantizationConfig::None,
            compression: Default::default(),
        };
        
        match vector_store.create_collection("test-collection", collection_config) {
            Ok(_) => println!("✅ Created test collection"),
            Err(_) => println!("ℹ️ Collection already exists"),
        }
        
        // Test 2: Test pattern matching directly
        println!("🔍 Testing pattern matching...");
        let test_patterns = vec![
            ("test.rs", "**/*.rs", true, "Rust file should match *.rs pattern"),
            ("README.md", "**/*.md", true, "Markdown file should match *.md pattern"),
            (".hidden", "**/*.rs", false, "Hidden file should not match"),
            ("temp.tmp", "**/*.rs", false, "Temporary file should not match"),
        ];
        
        for (filename, pattern, expected, description) in test_patterns {
            let file_path = test_path.join(filename);
            let result = EnhancedFileWatcher::matches_pattern(&file_path, pattern);
            println!("🔍 {:?} matches {} -> {} (expected: {}) - {}", 
                file_path, pattern, result, expected, description);
            assert_eq!(result, expected, "Pattern matching failed: {}", description);
        }
        
        // Test 3: Test file patterns matching
        println!("🔍 Testing file patterns matching...");
        let include_patterns = vec!["**/*.rs".to_string(), "**/*.md".to_string()];
        let exclude_patterns = vec!["**/.*".to_string(), "**/*.tmp".to_string()];
        
        let pattern_tests = vec![
            (test_path.join("test.rs"), true, "Rust file should match include patterns"),
            (test_path.join("README.md"), true, "Markdown file should match include patterns"),
            (test_path.join(".hidden"), false, "Hidden file should be excluded"),
            (test_path.join("temp.tmp"), false, "Temporary file should be excluded"),
        ];
        
        for (file_path, expected, description) in pattern_tests {
            let result = EnhancedFileWatcher::file_matches_patterns(
                &file_path,
                &include_patterns,
                &exclude_patterns,
            );
            println!("🔍 {:?} matches patterns -> {} (expected: {}) - {}", 
                file_path, result, expected, description);
            assert_eq!(result, expected, "File patterns matching failed: {}", description);
        }
        
        // Test 4: Test hash validation
        println!("🔐 Testing hash validation...");
        let hash1 = hash_validator.calculate_content_hash(&initial_content).await;
        let hash2 = hash_validator.calculate_content_hash("fn main() { println!(\"Different content\"); }").await;
        let hash3 = hash_validator.calculate_content_hash(&initial_content).await;
        
        assert_ne!(hash1, hash2, "Different content should produce different hashes");
        assert_eq!(hash1, hash3, "Same content should produce same hash");
        println!("✅ Hash validation working correctly");
        
        // Test 5: Test file indexing
        println!("📝 Testing file indexing...");
        let content = std::fs::read_to_string(&test_file).unwrap();
        let content_hash = hash_validator.calculate_content_hash(&content).await;
        
        // Create and insert vector
        let vector_id = format!("test_file_{}", test_file.to_string_lossy().replace("/", "_").replace("\\", "_"));
        let vector = Vector {
            id: vector_id.clone(),
            data: vec![0.1; 512],
            payload: Some(Payload {
                data: serde_json::json!({
                    "file_path": test_file.to_string_lossy(),
                    "content_hash": content_hash,
                    "file_size": content.len(),
                    "last_modified": chrono::Utc::now().to_rfc3339(),
                }),
            }),
        };
        
        vector_store.insert("test-collection", vec![vector]).unwrap();
        println!("✅ File indexed in collection");
        
        // Test 6: Test file index operations
        println!("📊 Testing file index operations...");
        {
            let mut index = file_index.write().await;
            index.add_mapping(
                test_file.clone(),
                "test-collection".to_string(),
                vec![vector_id.clone()],
                content_hash.clone(),
            );
        }
        
        let stats = enhanced_watcher.get_file_index_stats().await;
        println!("📊 File index stats: {:?}", stats);
        assert_eq!(stats.total_files, 1, "Should have 1 file in index");
        assert_eq!(stats.total_collections, 1, "Should have 1 collection in index");
        
        // Test 7: Test file modification
        println!("✏️ Testing file modification...");
        let modified_content = "fn main() {\n    println!(\"Hello, Enhanced File Watcher - MODIFIED!\");\n    println!(\"File has been changed!\");\n}";
        std::fs::write(&test_file, modified_content).unwrap();
        
        let new_hash = hash_validator.calculate_content_hash(&modified_content).await;
        assert_ne!(content_hash, new_hash, "File modification should change hash");
        println!("✅ File modification detected via hash change");
        
        // Test 8: Test file index JSON serialization
        println!("💾 Testing file index persistence...");
        let index_json = {
            let index = file_index.read().await;
            index.to_json().unwrap()
        };
        
        let restored_index = FileIndex::from_json(&index_json).unwrap();
        let restored_stats = restored_index.get_stats();
        assert_eq!(restored_stats.total_files, stats.total_files);
        println!("✅ File index serialization/deserialization working");
        
        // Test 9: Test file deletion simulation
        println!("🗑️ Testing file deletion simulation...");
        {
            let mut index = file_index.write().await;
            let removed_mappings = index.remove_file(&test_file);
            assert_eq!(removed_mappings.len(), 1, "Should remove 1 mapping");
        }
        
        let final_stats = enhanced_watcher.get_file_index_stats().await;
        assert_eq!(final_stats.total_files, 0, "Should have 0 files after deletion");
        println!("✅ File deletion from index successful");
        
        // Test 10: Verify collection persistence
        let collection_metadata = vector_store.get_collection_metadata("test-collection").unwrap();
        println!("📊 Collection vector count: {}", collection_metadata.vector_count);
        assert_eq!(collection_metadata.vector_count, 1, "Collection should still have the vector");
        
        // Clean up
        std::fs::remove_file(&test_file).unwrap();
        
        println!("🎉 Complete Enhanced File Watcher Workflow Test PASSED!");
        println!("✅ All functionality verified:");
        println!("  - ✅ Pattern matching (individual and file patterns)");
        println!("  - ✅ Hash validation and change detection");
        println!("  - ✅ File indexing and vector storage");
        println!("  - ✅ File index operations and statistics");
        println!("  - ✅ File modification detection");
        println!("  - ✅ File index persistence (JSON)");
        println!("  - ✅ File deletion simulation");
        println!("  - ✅ Collection persistence");
    }

    #[tokio::test]
    async fn test_file_watcher_performance_benchmarks() {
        println!("⚡ Testing Enhanced File Watcher Performance");
        
        let temp_dir = tempdir().unwrap();
        let test_path = temp_dir.path().to_path_buf();
        
        // Create multiple test files
        let file_count = 10;
        let mut test_files = Vec::new();
        
        for i in 0..file_count {
            let file_path = test_path.join(format!("test_{}.rs", i));
            let content = format!("fn test_{}() {{\n    println!(\"Test function {}\");\n}}", i, i);
            std::fs::write(&file_path, content).unwrap();
            test_files.push(file_path);
        }
        
        println!("📝 Created {} test files", file_count);
        
        // Test pattern matching performance
        let start = std::time::Instant::now();
        for file_path in &test_files {
            let result = EnhancedFileWatcher::matches_pattern(file_path, "**/*.rs");
            assert!(result, "All test files should match *.rs pattern");
        }
        let pattern_time = start.elapsed();
        println!("⚡ Pattern matching for {} files: {:?}", file_count, pattern_time);
        
        // Test hash calculation performance
        let start = std::time::Instant::now();
        for file_path in &test_files {
            let content = std::fs::read_to_string(file_path).unwrap();
            let hash_validator = HashValidator::new();
            let _hash = hash_validator.calculate_content_hash(&content).await;
        }
        let hash_time = start.elapsed();
        println!("⚡ Hash calculation for {} files: {:?}", file_count, hash_time);
        
        // Test file index operations performance
        let start = std::time::Instant::now();
        let mut file_index = FileIndex::new();
        for (i, file_path) in test_files.iter().enumerate() {
            file_index.add_mapping(
                file_path.clone(),
                "test-collection".to_string(),
                vec![format!("vector_{}", i)],
                format!("hash_{}", i),
            );
        }
        let index_time = start.elapsed();
        println!("⚡ File index operations for {} files: {:?}", file_count, index_time);
        
        // Performance assertions (these are generous thresholds)
        assert!(pattern_time < Duration::from_millis(100), "Pattern matching should be fast");
        assert!(hash_time < Duration::from_millis(500), "Hash calculation should be reasonably fast");
        assert!(index_time < Duration::from_millis(100), "File index operations should be fast");
        
        println!("✅ All performance benchmarks passed!");
    }
}
