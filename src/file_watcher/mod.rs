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
        
        // Initialize the watcher
        let mut watcher = watcher::Watcher::new(
            self.config.clone(),
            self.debouncer.clone(),
            self.hash_validator.clone(),
            self.grpc_operations.clone(),
        )?;

        // Start watching
        watcher.start().await?;

        Ok(())
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
