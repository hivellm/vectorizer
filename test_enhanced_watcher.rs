//! Test script for Enhanced File Watcher functionality

use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use vectorizer::file_watcher::{
    EnhancedFileWatcher, FileWatcherConfig, FileIndex, FileIndexArc,
    WorkspaceConfig, ProjectConfig, CollectionConfig
};
use vectorizer::file_watcher::{Debouncer, HashValidator, GrpcVectorOperations};
use vectorizer::{VectorStore, embedding::EmbeddingManager};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::fmt::init();
    
    println!("🧪 Testing Enhanced File Watcher...");
    
    // Create test directory
    let test_dir = PathBuf::from("test-file-watcher");
    if !test_dir.exists() {
        std::fs::create_dir_all(&test_dir)?;
    }
    
    // Create mock components
    let config = FileWatcherConfig {
        collection_name: "test-collection".to_string(),
        watch_paths: Some(vec![test_dir.clone()]),
        recursive: true,
        debounce_delay: Duration::from_millis(500),
        hash_validation: true,
        file_patterns: Some(vec!["**/*.txt".to_string(), "**/*.md".to_string()]),
        exclude_patterns: Some(vec!["**/.*".to_string(), "**/*.tmp".to_string()]),
    };
    
    // Create file index
    let file_index: FileIndexArc = Arc::new(RwLock::new(FileIndex::new()));
    
    // Create mock vector store and embedding manager
    let vector_store = Arc::new(VectorStore::new_auto()?);
    let embedding_manager = Arc::new(RwLock::new(EmbeddingManager::new()?));
    
    // Create debouncer
    let debouncer = Arc::new(Debouncer::new(config.debounce_delay));
    
    // Create hash validator
    let hash_validator = Arc::new(HashValidator::new(config.hash_validation));
    
    // Create GRPC operations (without actual GRPC client for testing)
    let grpc_operations = Arc::new(GrpcVectorOperations::new(
        vector_store,
        embedding_manager,
        None, // No GRPC client for testing
    ));
    
    // Create enhanced watcher
    let mut enhanced_watcher = EnhancedFileWatcher::new(
        config,
        debouncer,
        hash_validator,
        grpc_operations,
        file_index.clone(),
    )?;
    
    // Create workspace configuration for pattern matching
    let workspace_config = WorkspaceConfig {
        projects: vec![ProjectConfig {
            name: "test-project".to_string(),
            path: test_dir.clone(),
            collections: vec![CollectionConfig {
                name: "test-collection".to_string(),
                include_patterns: vec!["**/*.txt".to_string(), "**/*.md".to_string()],
                exclude_patterns: vec!["**/.*".to_string(), "**/*.tmp".to_string()],
            }],
        }],
    };
    
    // Set workspace configuration
    enhanced_watcher.set_workspace_config(workspace_config).await;
    
    println!("✅ Enhanced File Watcher components created successfully");
    
    // Test 1: Start the watcher
    println!("🔍 Starting Enhanced File Watcher...");
    enhanced_watcher.start().await?;
    println!("✅ Enhanced File Watcher started successfully");
    
    // Test 2: Check file index statistics
    let stats = enhanced_watcher.get_file_index_stats().await;
    println!("📊 File Index Stats: {:?}", stats);
    
    // Test 3: Create a new file and see if it's detected
    println!("📝 Creating new test file...");
    let new_file = test_dir.join("new_test.txt");
    std::fs::write(&new_file, "This is a new test file for the Enhanced File Watcher!")?;
    
    // Wait a bit for the watcher to process
    tokio::time::sleep(Duration::from_millis(1000)).await;
    
    // Check file index again
    let stats_after = enhanced_watcher.get_file_index_stats().await;
    println!("📊 File Index Stats after file creation: {:?}", stats_after);
    
    // Test 4: Modify an existing file
    println!("✏️ Modifying existing file...");
    std::fs::write(&new_file, "This file has been modified!")?;
    
    // Wait for processing
    tokio::time::sleep(Duration::from_millis(1000)).await;
    
    // Test 5: Delete a file
    println!("🗑️ Deleting test file...");
    std::fs::remove_file(&new_file)?;
    
    // Wait for processing
    tokio::time::sleep(Duration::from_millis(1000)).await;
    
    // Check final stats
    let final_stats = enhanced_watcher.get_file_index_stats().await;
    println!("📊 Final File Index Stats: {:?}", final_stats);
    
    // Test 6: Stop the watcher
    println!("🛑 Stopping Enhanced File Watcher...");
    enhanced_watcher.stop().await?;
    println!("✅ Enhanced File Watcher stopped successfully");
    
    println!("🎉 Enhanced File Watcher test completed successfully!");
    
    Ok(())
}
