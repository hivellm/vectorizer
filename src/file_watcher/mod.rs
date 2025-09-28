//! File Watcher System for real-time file monitoring and incremental reindexing
//!
//! This module provides a cross-platform file monitoring system that tracks changes
//! in indexed files and updates the vector database in real-time through GRPC operations.

pub mod config;
pub mod debouncer;
pub mod hash_validator;
pub mod watcher;
pub mod grpc_operations;

pub use config::FileWatcherConfig;
pub use watcher::Watcher as FileWatcher;
pub use grpc_operations::GrpcVectorOperations;

// Re-export FileWatcherSystem for external use

use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;
use crate::VectorStore;
use crate::embedding::EmbeddingManager;
use crate::grpc::vectorizer::vectorizer_service_client::VectorizerServiceClient;

/// File change event types
#[derive(Debug, Clone, PartialEq)]
pub enum FileChangeEvent {
    /// File was created
    Created(PathBuf),
    /// File was modified
    Modified(PathBuf),
    /// File was deleted
    Deleted(PathBuf),
    /// File was renamed
    Renamed(PathBuf, PathBuf),
}

/// File change event with metadata
#[derive(Debug, Clone)]
pub struct FileChangeEventWithMetadata {
    pub event: FileChangeEvent,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub content_hash: Option<String>,
    pub file_size: Option<u64>,
}

/// File Watcher System for real-time monitoring
pub struct FileWatcherSystem {
    config: FileWatcherConfig,
    vector_store: Arc<VectorStore>,
    embedding_manager: Arc<RwLock<EmbeddingManager>>,
    grpc_client: Option<Arc<VectorizerServiceClient<tonic::transport::Channel>>>,
    debouncer: Arc<debouncer::Debouncer>,
    hash_validator: Arc<hash_validator::HashValidator>,
    grpc_operations: Arc<grpc_operations::GrpcVectorOperations>,
}

impl FileWatcherSystem {
    /// Create a new File Watcher System
    pub fn new(
        config: FileWatcherConfig,
        vector_store: Arc<VectorStore>,
        embedding_manager: Arc<RwLock<EmbeddingManager>>,
        grpc_client: Option<Arc<VectorizerServiceClient<tonic::transport::Channel>>>,
    ) -> Self {
        let debouncer = Arc::new(debouncer::Debouncer::new(config.debounce_delay_ms));
        let hash_validator = Arc::new(hash_validator::HashValidator::new());
        let grpc_operations = Arc::new(grpc_operations::GrpcVectorOperations::new(
            vector_store.clone(),
            embedding_manager.clone(),
            grpc_client.clone(),
        ));

        Self {
            config,
            vector_store,
            embedding_manager,
            grpc_client,
            debouncer,
            hash_validator,
            grpc_operations,
        }
    }

    /// Start the file watcher system
    pub async fn start(&self) -> Result<()> {
        tracing::info!("Starting File Watcher System with config: {:?}", self.config);
        
        // Start with empty paths - will be populated incrementally
        let mut dynamic_config = self.config.clone();
        dynamic_config.watch_paths = Some(vec![]);
        
        // Initialize the watcher with empty paths initially
        let mut watcher = watcher::Watcher::new(
            dynamic_config,
            self.debouncer.clone(),
            self.hash_validator.clone(),
            self.grpc_operations.clone(),
        )?;

        // Start watching
        watcher.start().await?;

        Ok(())
    }

    /// Update file watcher with new indexed files (called after each collection is indexed)
    pub async fn update_with_collection(&self, collection_name: &str) -> Result<()> {
        tracing::info!("Updating file watcher with collection: {}", collection_name);
        
        // Discover files from this specific collection
        if let Ok(collection) = self.vector_store.get_collection(collection_name) {
            let vectors = collection.get_all_vectors();
            let mut new_files = Vec::new();
            
            for vector in vectors {
                if let Some(payload) = &vector.payload {
                    if let Some(metadata) = payload.data.get("metadata") {
                        if let Some(file_path) = metadata.get("file_path")
                            .or_else(|| metadata.get("source"))
                            .or_else(|| metadata.get("path")) {
                            
                            if let Some(path_str) = file_path.as_str() {
                                let path = std::path::PathBuf::from(path_str);
                                if path.exists() && !new_files.contains(&path) {
                                    new_files.push(path.clone());
                                    tracing::debug!("Added file to watcher: {:?}", path);
                                }
                            }
                        }
                    }
                }
            }
            
            tracing::info!("Added {} files from collection '{}' to file watcher", new_files.len(), collection_name);
        }
        
        Ok(())
    }

    /// Discover files that are already indexed in collections
    async fn discover_indexed_files(&self) -> Result<Vec<std::path::PathBuf>> {
        let mut indexed_files = std::collections::HashSet::new();
        
        // Get all collections from vector store
        let collections = self.vector_store.list_collections();
        tracing::info!("Found {} collections to scan for indexed files", collections.len());
        
        for collection_name in collections {
            if let Ok(collection) = self.vector_store.get_collection(&collection_name) {
                tracing::debug!("Scanning collection '{}' for indexed files", collection_name);
                
                // Get all vectors in the collection
                let vectors = collection.get_all_vectors();
                tracing::debug!("Collection '{}' has {} vectors", collection_name, vectors.len());
                
                // Extract file paths from vector payload
                for vector in vectors {
                    if let Some(payload) = &vector.payload {
                        // Look for file path in payload metadata
                        if let Some(metadata) = payload.data.get("metadata") {
                            if let Some(file_path) = metadata.get("file_path")
                                .or_else(|| metadata.get("source"))
                                .or_else(|| metadata.get("path")) {
                                
                                if let Some(path_str) = file_path.as_str() {
                                    let path = std::path::PathBuf::from(path_str);
                                    if path.exists() {
                                        indexed_files.insert(path.clone());
                                        tracing::debug!("Added indexed file: {:?}", path);
                                    } else {
                                        tracing::warn!("Indexed file not found: {:?}", path);
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        
        let file_count = indexed_files.len();
        if file_count == 0 {
            tracing::warn!("No indexed files found. File watcher will start with empty watch list.");
        } else {
            tracing::info!("Discovered {} unique indexed files to monitor", file_count);
        }
        
        Ok(indexed_files.into_iter().collect())
    }

    /// Stop the file watcher system
    pub async fn stop(&self) -> Result<()> {
        tracing::info!("Stopping File Watcher System");
        // Implementation will be added in the watcher module
        Ok(())
    }

    /// Get current configuration
    pub fn config(&self) -> &FileWatcherConfig {
        &self.config
    }

    /// Update configuration
    pub fn update_config(&mut self, config: FileWatcherConfig) {
        self.config = config;
    }
}

/// Error types for File Watcher System
#[derive(Debug, thiserror::Error)]
pub enum FileWatcherError {
    #[error("File system error: {0}")]
    FileSystem(#[from] std::io::Error),
    
    #[error("Notify error: {0}")]
    Notify(#[from] notify::Error),
    
    #[error("GRPC error: {0}")]
    Grpc(#[from] tonic::Status),
    
    #[error("Embedding error: {0}")]
    Embedding(String),
    
    #[error("Configuration error: {0}")]
    Configuration(String),
    
    #[error("Hash validation error: {0}")]
    HashValidation(String),
    
    #[error("Debouncing error: {0}")]
    Debouncing(String),
}

pub type Result<T> = std::result::Result<T, FileWatcherError>;

#[cfg(test)]
mod tests;
