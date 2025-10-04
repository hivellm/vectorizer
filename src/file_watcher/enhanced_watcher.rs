//! Enhanced File Watcher with full CRUD support

use std::path::PathBuf;
use std::sync::Arc;
use std::collections::HashMap;
use tokio::sync::RwLock;
use notify::{Watcher as NotifyWatcher, RecursiveMode, Event, EventKind};
use crate::file_watcher::{
    FileChangeEvent, FileChangeEventWithMetadata, Result, FileWatcherError,
    FileWatcherConfig, FileIndex, FileIndexArc, CollectionVectorMapping
};
use super::debouncer::Debouncer;
use super::hash_validator::HashValidator;
use super::grpc_operations::GrpcVectorOperations;

/// Enhanced file system event types
#[derive(Debug, Clone, PartialEq)]
pub enum FileSystemEvent {
    Created { path: PathBuf },
    Modified { path: PathBuf },
    Deleted { path: PathBuf },
    Renamed { from: PathBuf, to: PathBuf },
}

/// Enhanced File Watcher with full CRUD support
pub struct EnhancedFileWatcher {
    config: FileWatcherConfig,
    debouncer: Arc<Debouncer>,
    hash_validator: Arc<HashValidator>,
    grpc_operations: Arc<GrpcVectorOperations>,
    file_index: FileIndexArc,
    watcher: Option<notify::RecommendedWatcher>,
    running: Arc<RwLock<bool>>,
    workspace_config: Arc<RwLock<Option<WorkspaceConfig>>>,
}

/// Workspace configuration for pattern matching
#[derive(Debug, Clone)]
pub struct WorkspaceConfig {
    pub projects: Vec<ProjectConfig>,
}

#[derive(Debug, Clone)]
pub struct ProjectConfig {
    pub name: String,
    pub path: PathBuf,
    pub collections: Vec<CollectionConfig>,
}

#[derive(Debug, Clone)]
pub struct CollectionConfig {
    pub name: String,
    pub include_patterns: Vec<String>,
    pub exclude_patterns: Vec<String>,
}

impl EnhancedFileWatcher {
    /// Create a new enhanced file watcher
    pub fn new(
        config: FileWatcherConfig,
        debouncer: Arc<Debouncer>,
        hash_validator: Arc<HashValidator>,
        grpc_operations: Arc<GrpcVectorOperations>,
        file_index: FileIndexArc,
    ) -> Result<Self> {
        Ok(Self {
            config,
            debouncer,
            hash_validator,
            grpc_operations,
            file_index,
            watcher: None,
            running: Arc::new(RwLock::new(false)),
            workspace_config: Arc::new(RwLock::new(None)),
        })
    }

    /// Set workspace configuration for pattern matching
    pub async fn set_workspace_config(&self, config: WorkspaceConfig) {
        let mut workspace_config = self.workspace_config.write().await;
        *workspace_config = Some(config);
    }

    /// Start the enhanced file watcher
    pub async fn start(&mut self) -> Result<()> {
        // Validate configuration
        self.config.validate()
            .map_err(|e| FileWatcherError::Configuration(e))?;

        // Set up event callback
        let grpc_operations = Arc::clone(&self.grpc_operations);
        let file_index = Arc::clone(&self.file_index);
        let workspace_config = Arc::clone(&self.workspace_config);
        let hash_validator = Arc::clone(&self.hash_validator);

        self.debouncer.set_event_callback(move |event| {
            let grpc_operations = Arc::clone(&grpc_operations);
            let file_index = Arc::clone(&file_index);
            let workspace_config = Arc::clone(&workspace_config);
            let hash_validator = Arc::clone(&hash_validator);
            
            tokio::spawn(async move {
                if let Err(e) = Self::handle_file_change_enhanced(
                    event,
                    &grpc_operations,
                    &file_index,
                    &workspace_config,
                    &hash_validator,
                ).await {
                    tracing::error!("Failed to handle enhanced file change: {}", e);
                }
            });
        }).await;

        // Create notify watcher
        let (tx, mut rx) = std::sync::mpsc::channel();
        let mut watcher = notify::recommended_watcher(tx)
            .map_err(|e| FileWatcherError::Notify(e))?;

        // Start watching paths
        let empty_paths = vec![];
        let watch_paths = self.config.watch_paths.as_ref().unwrap_or(&empty_paths);
        for path in watch_paths {
            let recursive_mode = if self.config.recursive {
                RecursiveMode::Recursive
            } else {
                RecursiveMode::NonRecursive
            };

            watcher.watch(path, recursive_mode)
                .map_err(|e| FileWatcherError::Notify(e))?;

            tracing::info!("Enhanced watcher watching path: {:?} (recursive: {})", path, self.config.recursive);
        }

        self.watcher = Some(watcher);

        // Mark as running
        {
            let mut running = self.running.write().await;
            *running = true;
        }

        // Start event processing loop
        let debouncer = Arc::clone(&self.debouncer);
        let config = self.config.clone();
        tokio::spawn(async move {
            while let Ok(res) = rx.recv() {
                match res {
                    Ok(event) => {
                        if let Err(e) = Self::process_notify_event_enhanced(event, &debouncer, &config).await {
                            tracing::error!("Failed to process enhanced notify event: {}", e);
                        }
                    }
                    Err(e) => {
                        tracing::error!("Enhanced notify error: {}", e);
                    }
                }
            }
        });

        tracing::info!("Enhanced file watcher started successfully");
        Ok(())
    }

    /// Stop the enhanced file watcher
    pub async fn stop(&mut self) -> Result<()> {
        // Mark as not running
        {
            let mut running = self.running.write().await;
            *running = false;
        }

        // Clear pending events
        self.debouncer.clear_pending_events().await;

        // Clear hash cache
        self.hash_validator.clear_hashes().await;

        tracing::info!("Enhanced file watcher stopped");
        Ok(())
    }

    /// Check if watcher is running
    pub async fn is_running(&self) -> bool {
        let running = self.running.read().await;
        *running
    }

    /// Process notify event with enhanced logic
    async fn process_notify_event_enhanced(
        event: Event,
        debouncer: &Arc<Debouncer>,
        config: &FileWatcherConfig,
    ) -> Result<()> {
        for path in event.paths {
            // Check if file should be processed based on patterns first
            let should_process = config.should_process_file(&path);

            // Convert notify event to our enhanced event type
            let file_event = match event.kind {
                EventKind::Create(_) => {
                    if path.is_dir() {
                        // Handle directory creation - scan for new files
                        Self::handle_directory_created(&path, debouncer).await?;
                        continue;
                    } else {
                        FileChangeEvent::Created(path.clone())
                    }
                }
                EventKind::Modify(_) => {
                    if path.is_dir() {
                        continue; // Skip directory modifications
                    } else {
                        FileChangeEvent::Modified(path.clone())
                    }
                }
                EventKind::Remove(_) => {
                    if path.is_dir() {
                        // Handle directory deletion
                        Self::handle_directory_deleted(&path, debouncer).await?;
                        continue;
                    } else {
                        FileChangeEvent::Deleted(path.clone())
                    }
                }
                EventKind::Access(_) => {
                    continue; // Skip access events
                }
                EventKind::Other => {
                    continue; // Skip other events
                }
                _ => {
                    continue; // Skip unknown events
                }
            };

            if !should_process {
                continue;
            }

            // Add event to debouncer
            debouncer.add_event(file_event).await;
        }

        Ok(())
    }

    /// Handle directory creation by scanning for new files
    fn handle_directory_created<'a>(
        dir_path: &'a PathBuf,
        debouncer: &'a Arc<Debouncer>,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<()>> + Send + 'a>> {
        Box::pin(async move {
        tracing::info!("Directory created: {:?}, scanning for new files", dir_path);

        // Recursively scan directory for files
        if let Ok(mut entries) = tokio::fs::read_dir(dir_path).await {
            while let Ok(Some(entry)) = entries.next_entry().await {
                let path = entry.path();
                
                if path.is_dir() {
                    // Recursively handle subdirectories
                    Self::handle_directory_created(&path, debouncer).await?;
                } else if path.is_file() {
                    // Add file creation event
                    debouncer.add_event(FileChangeEvent::Created(path)).await;
                }
            }
        }

        Ok(())
        })
    }

    /// Handle directory deletion
    async fn handle_directory_deleted(
        dir_path: &PathBuf,
        debouncer: &Arc<Debouncer>,
    ) -> Result<()> {
        tracing::info!("Directory deleted: {:?}", dir_path);
        // Directory deletion will be handled by the file index cleanup
        Ok(())
    }

    /// Handle file change event with enhanced logic
    async fn handle_file_change_enhanced(
        event: FileChangeEventWithMetadata,
        grpc_operations: &Arc<GrpcVectorOperations>,
        file_index: &FileIndexArc,
        workspace_config: &Arc<RwLock<Option<WorkspaceConfig>>>,
        hash_validator: &Arc<HashValidator>,
    ) -> Result<()> {
        let path = match &event.event {
            FileChangeEvent::Created(path) => path,
            FileChangeEvent::Modified(path) => path,
            FileChangeEvent::Deleted(path) => path,
            FileChangeEvent::Renamed(_, new_path) => new_path,
        };

        // Find matching collections for this file
        let matching_collections = Self::find_matching_collections(
            path,
            workspace_config,
        ).await;

        if matching_collections.is_empty() {
            tracing::debug!("No matching collections for file: {:?}", path);
            return Ok(());
        }

        match &event.event {
            FileChangeEvent::Created(path) => {
                Self::handle_file_created(
                    path.clone(),
                    matching_collections,
                    grpc_operations,
                    file_index,
                    hash_validator,
                ).await
            }
            FileChangeEvent::Modified(path) => {
                // Check if content has actually changed
                if hash_validator.is_enabled() {
                    if let Ok(has_changed) = hash_validator.has_content_changed(path).await {
                        if !has_changed {
                            tracing::debug!("File content unchanged, skipping: {:?}", path);
                            return Ok(());
                        }
                    }
                }

                Self::handle_file_modified(
                    path.clone(),
                    matching_collections,
                    grpc_operations,
                    file_index,
                    hash_validator,
                ).await
            }
            FileChangeEvent::Deleted(path) => {
                Self::handle_file_deleted(
                    path.clone(),
                    grpc_operations,
                    file_index,
                ).await
            }
            FileChangeEvent::Renamed(old_path, new_path) => {
                Self::handle_file_renamed(
                    old_path.clone(),
                    new_path.clone(),
                    matching_collections,
                    grpc_operations,
                    file_index,
                    hash_validator,
                ).await
            }
        }
    }

    /// Find collections that match the file pattern
    async fn find_matching_collections(
        file_path: &PathBuf,
        workspace_config: &Arc<RwLock<Option<WorkspaceConfig>>>,
    ) -> Vec<String> {
        let workspace_config = workspace_config.read().await;
        let Some(config) = workspace_config.as_ref() else {
            return Vec::new();
        };

        let mut matching_collections = Vec::new();

        for project in &config.projects {
            for collection in &project.collections {
                if Self::file_matches_patterns(file_path, &collection.include_patterns, &collection.exclude_patterns) {
                    matching_collections.push(collection.name.clone());
                }
            }
        }

        matching_collections
    }

    /// Check if file matches include/exclude patterns
    pub fn file_matches_patterns(
        file_path: &PathBuf,
        include_patterns: &[String],
        exclude_patterns: &[String],
    ) -> bool {
        // Check exclude patterns first
        for pattern in exclude_patterns {
            if Self::matches_pattern(file_path, pattern) {
                return false;
            }
            // Also check if any part of the path starts with a dot (hidden files/dirs)
            if pattern == "**/.*" {
                if let Some(path_str) = file_path.to_str() {
                    if path_str.contains("/.") || path_str.starts_with('.') {
                        return false;
                    }
                }
            }
        }

        // If no include patterns, match everything
        if include_patterns.is_empty() {
            return true;
        }

        // Check include patterns
        for pattern in include_patterns {
            if Self::matches_pattern(file_path, pattern) {
                return true;
            }
        }

        false
    }

    /// Check if file matches a glob pattern
    pub fn matches_pattern(file_path: &PathBuf, pattern: &str) -> bool {
        // Simple pattern matching - can be enhanced with proper glob library
        if let Some(file_str) = file_path.to_str() {
            if pattern.contains("**") {
                // Handle recursive patterns like **/*.rs
                if pattern.starts_with("**/") {
                    let suffix = &pattern[3..]; // Remove "**/"
                    if suffix == "*" {
                        return true; // **/* matches everything
                    } else if suffix.starts_with("*.") {
                        let ext = &suffix[1..]; // Remove "*" to get ".ext"
                        return file_str.ends_with(ext);
                    }
                }
                return file_str.contains(&pattern.replace("**", ""));
            } else if pattern.contains("*") {
                // Handle simple wildcards like *.rs
                if pattern.starts_with("*.") {
                    return file_str.ends_with(&pattern[1..]); // Remove "*" to get ".ext"
                } else {
                    let pattern = pattern.replace("*", "");
                    return file_str.contains(&pattern);
                }
            } else {
                // Exact match
                return file_str == pattern || file_str.ends_with(pattern);
            }
        } else {
            false
        }
    }

    /// Handle file creation
    async fn handle_file_created(
        path: PathBuf,
        matching_collections: Vec<String>,
        grpc_operations: &Arc<GrpcVectorOperations>,
        file_index: &FileIndexArc,
        hash_validator: &Arc<HashValidator>,
    ) -> Result<()> {
        tracing::info!("Handling file creation: {:?} -> collections: {:?}", path, matching_collections);

        // Read file content
        let content = match tokio::fs::read_to_string(&path).await {
            Ok(content) => content,
            Err(e) => {
                tracing::error!("Failed to read created file {:?}: {}", path, e);
                return Ok(());
            }
        };

        // Index into each matching collection
        for collection_name in matching_collections {
            let event = FileChangeEventWithMetadata {
                event: FileChangeEvent::Created(path.clone()),
                timestamp: chrono::Utc::now(),
                content_hash: Some(hash_validator.calculate_content_hash(&content).await),
                file_size: Some(content.len() as u64),
            };

            if let Err(e) = grpc_operations.process_file_change(event, &collection_name).await {
                tracing::error!("Failed to index created file {:?} into collection {}: {}", path, collection_name, e);
            } else {
                tracing::info!("Successfully indexed created file {:?} into collection {}", path, collection_name);
            }
        }

        Ok(())
    }

    /// Handle file modification
    async fn handle_file_modified(
        path: PathBuf,
        matching_collections: Vec<String>,
        grpc_operations: &Arc<GrpcVectorOperations>,
        file_index: &FileIndexArc,
        hash_validator: &Arc<HashValidator>,
    ) -> Result<()> {
        tracing::info!("Handling file modification: {:?} -> collections: {:?}", path, matching_collections);

        // Read file content
        let content = match tokio::fs::read_to_string(&path).await {
            Ok(content) => content,
            Err(e) => {
                tracing::error!("Failed to read modified file {:?}: {}", path, e);
                return Ok(());
            }
        };

        // Update in each matching collection
        for collection_name in matching_collections {
            let event = FileChangeEventWithMetadata {
                event: FileChangeEvent::Modified(path.clone()),
                timestamp: chrono::Utc::now(),
                content_hash: Some(hash_validator.calculate_content_hash(&content).await),
                file_size: Some(content.len() as u64),
            };

            if let Err(e) = grpc_operations.process_file_change(event, &collection_name).await {
                tracing::error!("Failed to update modified file {:?} in collection {}: {}", path, collection_name, e);
            } else {
                tracing::info!("Successfully updated modified file {:?} in collection {}", path, collection_name);
            }
        }

        Ok(())
    }

    /// Handle file deletion
    async fn handle_file_deleted(
        path: PathBuf,
        grpc_operations: &Arc<GrpcVectorOperations>,
        file_index: &FileIndexArc,
    ) -> Result<()> {
        tracing::info!("Handling file deletion: {:?}", path);

        // Remove from file index and get affected collections
        let removed_collections = {
            let mut index = file_index.write().await;
            index.remove_file(&path)
        };

        // Remove vectors from each affected collection
        for (collection_name, vector_ids) in removed_collections {
            tracing::info!("Removing {} vectors from collection {} for deleted file {:?}", vector_ids.len(), collection_name, path);

            // Remove vectors via GRPC operations
            for vector_id in vector_ids {
                if let Err(e) = grpc_operations.remove_vector(&vector_id, &collection_name).await {
                    tracing::error!("Failed to remove vector {} from collection {}: {}", vector_id, collection_name, e);
                }
            }
        }

        Ok(())
    }

    /// Handle file rename
    async fn handle_file_renamed(
        old_path: PathBuf,
        new_path: PathBuf,
        matching_collections: Vec<String>,
        grpc_operations: &Arc<GrpcVectorOperations>,
        file_index: &FileIndexArc,
        hash_validator: &Arc<HashValidator>,
    ) -> Result<()> {
        tracing::info!("Handling file rename: {:?} -> {:?}", old_path, new_path);

        // Handle as deletion of old file and creation of new file
        Self::handle_file_deleted(old_path, grpc_operations, file_index).await?;
        Self::handle_file_created(new_path, matching_collections, grpc_operations, file_index, hash_validator).await?;

        Ok(())
    }

    /// Get file index statistics
    pub async fn get_file_index_stats(&self) -> crate::file_watcher::FileIndexStats {
        let index = self.file_index.read().await;
        index.get_stats()
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

impl Drop for EnhancedFileWatcher {
    fn drop(&mut self) {
        // Ensure watcher is stopped
        if let Some(_watcher) = &self.watcher {
            // Note: notify watcher doesn't have a stop method
            // It will be stopped when dropped
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_pattern_matching() {
        let path = PathBuf::from("src/main.rs");
        
        // Test include patterns
        assert!(EnhancedFileWatcher::matches_pattern(&path, "**/*.rs"));
        assert!(EnhancedFileWatcher::matches_pattern(&path, "src/*.rs"));
        assert!(EnhancedFileWatcher::matches_pattern(&path, "main.rs"));
        
        // Test exclude patterns
        assert!(!EnhancedFileWatcher::matches_pattern(&path, "**/*.py"));
        assert!(!EnhancedFileWatcher::matches_pattern(&path, "test/*.rs"));
    }

    #[tokio::test]
    async fn test_file_matches_patterns() {
        let path = PathBuf::from("src/main.rs");
        let include_patterns = vec!["**/*.rs".to_string()];
        let exclude_patterns = vec!["**/test.rs".to_string()];

        assert!(EnhancedFileWatcher::file_matches_patterns(&path, &include_patterns, &exclude_patterns));

        let test_path = PathBuf::from("src/test.rs");
        assert!(!EnhancedFileWatcher::file_matches_patterns(&test_path, &include_patterns, &exclude_patterns));
    }
}
