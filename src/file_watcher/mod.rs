//! File Watcher System for real-time file monitoring and incremental reindexing
//!
//! This module provides a cross-platform file monitoring system that tracks changes
//! in indexed files and updates the vector database in real-time through vector operations.

pub mod config;
pub mod debouncer;
pub mod hash_validator;
pub mod watcher;
pub mod file_index;
pub mod enhanced_watcher;
pub mod operations;

#[cfg(test)]
pub mod tests;

#[cfg(test)]
pub mod test_operations;

#[cfg(test)]
pub mod test_integration;

pub use config::FileWatcherConfig;
pub use watcher::Watcher as FileWatcher;
pub use operations::VectorOperations;
pub use file_index::{FileIndex, FileIndexArc, CollectionVectorMapping, FileIndexStats};
pub use enhanced_watcher::{EnhancedFileWatcher, FileSystemEvent, WorkspaceConfig, ProjectConfig, CollectionConfig};

// Re-export FileWatcherSystem for external use

use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;
use notify::EventKind;
use crate::VectorStore;
use crate::embedding::EmbeddingManager;

/// Workspace watch configuration
#[derive(Debug, Clone)]
struct WorkspaceWatchConfig {
    watch_paths: Vec<PathBuf>,
}

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

impl FileChangeEvent {
    /// Convert from notify::Event to FileChangeEvent
    pub fn from_notify_event(event: notify::Event) -> Self {
        match event.kind {
            EventKind::Create(_) => {
                if let Some(path) = event.paths.first() {
                    FileChangeEvent::Created(path.clone())
                } else {
                    // Fallback - shouldn't happen but handle gracefully
                    FileChangeEvent::Created(PathBuf::new())
                }
            }
            EventKind::Modify(_) => {
                if let Some(path) = event.paths.first() {
                    FileChangeEvent::Modified(path.clone())
                } else {
                    FileChangeEvent::Modified(PathBuf::new())
                }
            }
            EventKind::Remove(_) => {
                if let Some(path) = event.paths.first() {
                    FileChangeEvent::Deleted(path.clone())
                } else {
                    FileChangeEvent::Deleted(PathBuf::new())
                }
            }
            EventKind::Other => {
                // Handle rename events and other types
                if event.paths.len() >= 2 {
                    FileChangeEvent::Renamed(event.paths[0].clone(), event.paths[1].clone())
                } else if let Some(path) = event.paths.first() {
                    // Treat as modify if we can't determine the type
                    FileChangeEvent::Modified(path.clone())
                } else {
                    FileChangeEvent::Modified(PathBuf::new())
                }
            }
            _ => {
                // Handle any other event types as modify
                if let Some(path) = event.paths.first() {
                    FileChangeEvent::Modified(path.clone())
                } else {
                    FileChangeEvent::Modified(PathBuf::new())
                }
            }
        }
    }
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
    vector_operations: Arc<operations::VectorOperations>,
    debouncer: Arc<debouncer::Debouncer>,
    hash_validator: Arc<hash_validator::HashValidator>,
}

impl FileWatcherSystem {
    /// Create a new File Watcher System
    pub fn new(
        config: FileWatcherConfig,
        vector_store: Arc<VectorStore>,
        embedding_manager: Arc<RwLock<EmbeddingManager>>,
    ) -> Self {
        let debouncer = Arc::new(debouncer::Debouncer::new(config.debounce_delay_ms));
        let hash_validator = Arc::new(hash_validator::HashValidator::new());
        
        // Create vector operations - we'll pass the Arc<RwLock<EmbeddingManager>> directly
        let vector_operations = Arc::new(operations::VectorOperations::new(
            vector_store.clone(),
            embedding_manager.clone(),
        ));

        Self {
            config,
            vector_store,
            embedding_manager,
            vector_operations,
            debouncer,
            hash_validator,
        }
    }

    /// Start the file watcher system
    pub async fn start(&self) -> Result<()> {
        tracing::info!("🔍 FW_STEP_1: Starting File Watcher System with config: {:?}", self.config);
        
        tracing::info!("🔍 FW_STEP_2: Setting up event processing callback...");
        // Set up event processing callback
        let vector_operations = self.vector_operations.clone();
        self.debouncer.set_event_callback(move |event| {
            let vector_operations = vector_operations.clone();
            tokio::spawn(async move {
                if let Err(e) = vector_operations.process_file_change(&event).await {
                    tracing::error!("Failed to process file change event: {:?}", e);
                }
            });
        }).await;
        tracing::info!("✅ FW_STEP_2: Event processing callback set up");
        
        tracing::info!("🔍 FW_STEP_3: Discovering indexed files...");
        // Discover indexed files and set up watch paths
        let indexed_files = self.discover_indexed_files().await?;
        tracing::info!("✅ FW_STEP_3: Found {} indexed files", indexed_files.len());
        
        tracing::info!("🔍 FW_STEP_4: Extracting unique directories from indexed files...");
        let mut watch_paths: Vec<PathBuf> = Vec::new();
        
        // Extract unique directories from indexed files
        for file_path in indexed_files {
            if let Some(parent) = file_path.parent() {
                let parent_path = parent.to_path_buf();
                if !watch_paths.contains(&parent_path) {
                    watch_paths.push(parent_path);
                }
            }
        }
        
        // If no indexed files found, use workspace configuration
        if watch_paths.is_empty() {
            tracing::info!("No indexed files found, loading workspace configuration...");
            
            // Load workspace configuration
            if let Ok(workspace_config) = self.load_workspace_config().await {
                tracing::info!("Loaded workspace config with {} watch paths", workspace_config.watch_paths.len());
                watch_paths.extend(workspace_config.watch_paths);
            } else {
                tracing::warn!("Failed to load workspace config, using fallback paths");
                // Fallback to current directory
                watch_paths.push(std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from(".")));
            }
        }
        
        tracing::info!("✅ FW_STEP_4: Setting up file watcher for {} directories: {:?}", watch_paths.len(), watch_paths);
        
        tracing::info!("🔍 FW_STEP_5: Initializing watcher with discovered paths...");
        // Initialize the watcher with discovered paths
        let mut dynamic_config = self.config.clone();
        dynamic_config.watch_paths = Some(watch_paths);
        
        let mut watcher = watcher::Watcher::new(
            dynamic_config,
            self.debouncer.clone(),
            self.hash_validator.clone(),
        )?;
        tracing::info!("✅ FW_STEP_5: Watcher instance created");

        tracing::info!("🔍 FW_STEP_6: Starting watcher...");
        // Start watching
        watcher.start().await?;
        tracing::info!("✅ FW_STEP_6: Watcher started");
        
        tracing::info!("✅ FW_STEP_7: File Watcher System started successfully");

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
        tracing::info!("🔍 DISCOVER_STEP_1: Starting discovery of indexed files...");
        let mut indexed_files = std::collections::HashSet::new();
        
        tracing::info!("🔍 DISCOVER_STEP_2: Getting all collections from vector store...");
        // Get all collections from vector store
        let collections = self.vector_store.list_collections();
        tracing::info!("✅ DISCOVER_STEP_2: Found {} collections to scan for indexed files", collections.len());
        
        tracing::info!("🔍 DISCOVER_STEP_3: Scanning each collection for indexed files...");
        for (i, collection_name) in collections.iter().enumerate() {
            tracing::info!("🔍 DISCOVER_STEP_3.{}/{}: Scanning collection '{}'...", i+1, collections.len(), collection_name);
            if let Ok(collection) = self.vector_store.get_collection(collection_name) {
                tracing::debug!("Scanning collection '{}' for indexed files", collection_name);
                
                // Get all vectors in the collection
                let vectors = collection.get_all_vectors();
                tracing::info!("✅ DISCOVER_STEP_3.{}/{}: Collection '{}' has {} vectors", i+1, collections.len(), collection_name, vectors.len());
                
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
        tracing::info!("✅ DISCOVER_STEP_4: Discovery completed - found {} unique indexed files", file_count);
        if file_count == 0 {
            tracing::warn!("⚠️ DISCOVER_STEP_4: No indexed files found. File watcher will start with empty watch list.");
        } else {
            tracing::info!("✅ DISCOVER_STEP_4: Discovered {} unique indexed files to monitor", file_count);
        }
        
        Ok(indexed_files.into_iter().collect())
    }

    /// Load workspace configuration from vectorize-workspace.yml
    async fn load_workspace_config(&self) -> Result<WorkspaceWatchConfig> {
        let workspace_file = std::env::current_dir()
            .unwrap_or_else(|_| std::path::PathBuf::from("."))
            .join("vectorize-workspace.yml");
        
        if !workspace_file.exists() {
            return Err(FileWatcherError::ConfigError(format!(
                "Workspace file not found: {:?}", workspace_file
            )));
        }
        
        let content = tokio::fs::read_to_string(&workspace_file).await
            .map_err(|e| FileWatcherError::ConfigError(format!("Failed to read workspace file: {}", e)))?;
        
        let workspace: serde_yaml::Value = serde_yaml::from_str(&content)
            .map_err(|e| FileWatcherError::ConfigError(format!("Failed to parse workspace file: {}", e)))?;
        
        // Extract watch paths from global_settings
        let mut watch_paths = Vec::new();
        if let Some(global_settings) = workspace.get("global_settings") {
            if let Some(file_watcher) = global_settings.get("file_watcher") {
                if let Some(paths) = file_watcher.get("watch_paths") {
                    if let Some(paths_array) = paths.as_sequence() {
                        for path in paths_array {
                            if let Some(path_str) = path.as_str() {
                                watch_paths.push(std::path::PathBuf::from(path_str));
                            }
                        }
                    }
                }
            }
        }
        
        // Extract project paths
        if let Some(projects) = workspace.get("projects") {
            if let Some(projects_array) = projects.as_sequence() {
                for project in projects_array {
                    if let Some(path) = project.get("path") {
                        if let Some(path_str) = path.as_str() {
                            let project_path = std::env::current_dir()
                                .unwrap_or_else(|_| std::path::PathBuf::from("."))
                                .join(path_str);
                            if project_path.exists() {
                                watch_paths.push(project_path);
                            }
                        }
                    }
                }
            }
        }
        
        tracing::info!("Extracted {} watch paths from workspace config", watch_paths.len());
        Ok(WorkspaceWatchConfig { watch_paths })
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
    
    #[error("Embedding error: {0}")]
    Embedding(String),
    
    #[error("Configuration error: {0}")]
    Configuration(String),
    
    #[error("Config error: {0}")]
    ConfigError(String),
    
    #[error("Hash validation error: {0}")]
    HashValidation(String),
    
    #[error("Debouncing error: {0}")]
    Debouncing(String),
    
    #[error("Watcher is already running")]
    AlreadyRunning,
    
    #[error("Failed to create watcher: {0}")]
    WatcherCreationFailed(String),
    
    #[error("Failed to watch path {0}: {1}")]
    PathWatchFailed(PathBuf, String),
    
    #[error("Failed to stop watcher: {0}")]
    WatcherStopFailed(String),
}

pub type Result<T> = std::result::Result<T, FileWatcherError>;

