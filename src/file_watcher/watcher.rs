//! File watcher implementation using notify crate

use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use notify::{Watcher as NotifyWatcher, RecursiveMode, Event, EventKind, EventHandler};
use crate::file_watcher::{
    FileChangeEvent, FileChangeEventWithMetadata, Result, FileWatcherError
};
use super::config::FileWatcherConfig;
use super::debouncer::Debouncer;
use super::hash_validator::HashValidator;
use super::grpc_operations::GrpcVectorOperations;

/// File watcher implementation
pub struct Watcher {
    config: FileWatcherConfig,
    debouncer: Arc<Debouncer>,
    hash_validator: Arc<HashValidator>,
    grpc_operations: Arc<GrpcVectorOperations>,
    watcher: Option<notify::RecommendedWatcher>,
    running: Arc<RwLock<bool>>,
}

impl Watcher {
    /// Create a new file watcher
    pub fn new(
        config: FileWatcherConfig,
        debouncer: Arc<Debouncer>,
        hash_validator: Arc<HashValidator>,
        grpc_operations: Arc<GrpcVectorOperations>,
    ) -> Result<Self> {
        Ok(Self {
            config,
            debouncer,
            hash_validator,
            grpc_operations,
            watcher: None,
            running: Arc::new(RwLock::new(false)),
        })
    }

    /// Start the file watcher
    pub async fn start(&mut self) -> Result<()> {
        // Validate configuration
        self.config.validate()
            .map_err(|e| FileWatcherError::Configuration(e))?;

        // Set up event callback
        let grpc_operations = Arc::clone(&self.grpc_operations);
        let collection_name = self.config.collection_name.clone();
        let hash_validator = Arc::clone(&self.hash_validator);

        self.debouncer.set_event_callback(move |event| {
            let grpc_operations = Arc::clone(&grpc_operations);
            let collection_name = collection_name.clone();
            let hash_validator = Arc::clone(&hash_validator);
            
            tokio::spawn(async move {
                if let Err(e) = Self::handle_file_change(
                    event,
                    &collection_name,
                    &grpc_operations,
                    &hash_validator,
                ).await {
                    tracing::error!("Failed to handle file change: {}", e);
                }
            });
        }).await;

        // Create notify watcher
        let (tx, mut rx) = std::sync::mpsc::channel();
        let mut watcher = notify::recommended_watcher(tx)
            .map_err(|e| FileWatcherError::Notify(e))?;

        // Start watching paths (if any are configured)
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

            tracing::info!("Watching path: {:?} (recursive: {})", path, self.config.recursive);
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
                        if let Err(e) = Self::process_notify_event(event, &debouncer, &config).await {
                            tracing::error!("Failed to process notify event: {}", e);
                        }
                    }
                    Err(e) => {
                        tracing::error!("Notify error: {}", e);
                    }
                }
            }
        });

        tracing::info!("File watcher started successfully");
        Ok(())
    }

    /// Stop the file watcher
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

        tracing::info!("File watcher stopped");
        Ok(())
    }

    /// Check if watcher is running
    pub async fn is_running(&self) -> bool {
        let running = self.running.read().await;
        *running
    }

    /// Process notify event
    async fn process_notify_event(
        event: Event,
        debouncer: &Arc<Debouncer>,
        config: &FileWatcherConfig,
    ) -> Result<()> {
        for path in event.paths {
            if !path.exists() {
                continue;
            }

            // Skip directories
            if path.is_dir() {
                continue;
            }

            // Skip files that don't exist or are temporary
            if path.file_name()
                .and_then(|name| name.to_str())
                .map(|name| name.starts_with('.') || name.contains(".tmp") || name.contains("~"))
                .unwrap_or(false) {
                continue;
            }

            // Check if file should be processed based on patterns
            let should_process = config.should_process_file(&path);

            if !should_process {
                continue;
            }

            // Convert notify event to our event type
            let file_event = match event.kind {
                EventKind::Create(_) => FileChangeEvent::Created(path),
                EventKind::Modify(_) => FileChangeEvent::Modified(path),
                EventKind::Remove(_) => FileChangeEvent::Deleted(path),
                EventKind::Access(_) => {
                    // Skip access events
                    continue;
                }
                EventKind::Other => {
                    // Skip other events
                    continue;
                }
                _ => {
                    // Skip unknown events
                    continue;
                }
            };

            // Add event to debouncer
            debouncer.add_event(file_event).await;
        }

        Ok(())
    }

    /// Handle file change event
    async fn handle_file_change(
        event: FileChangeEventWithMetadata,
        collection_name: &str,
        grpc_operations: &Arc<GrpcVectorOperations>,
        hash_validator: &Arc<HashValidator>,
    ) -> Result<()> {
        let path = match &event.event {
            FileChangeEvent::Created(path) => path,
            FileChangeEvent::Modified(path) => path,
            FileChangeEvent::Deleted(path) => path,
            FileChangeEvent::Renamed(_, new_path) => new_path,
        };

        // Check if content has actually changed (if hash validation is enabled)
        if hash_validator.is_enabled() {
            match &event.event {
                FileChangeEvent::Created(_) | FileChangeEvent::Modified(_) => {
                    if let Ok(has_changed) = hash_validator.has_content_changed(path).await {
                        if !has_changed {
                            tracing::debug!("File content unchanged, skipping: {:?}", path);
                            return Ok(());
                        }
                    }
                }
                _ => {} // For deletions and renames, we don't check content
            }
        }

        // Process the file change
        grpc_operations.process_file_change(event, collection_name).await?;

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

    /// Get pending events count
    pub async fn pending_events_count(&self) -> usize {
        self.debouncer.pending_events_count().await
    }

    /// Get cached hashes count
    pub async fn cached_hashes_count(&self) -> usize {
        self.hash_validator.cached_hashes_count().await
    }
}

impl Drop for Watcher {
    fn drop(&mut self) {
        // Ensure watcher is stopped
        if let Some(watcher) = &self.watcher {
            // Note: notify watcher doesn't have a stop method
            // It will be stopped when dropped
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    use std::fs;
    use std::time::Duration;
    use crate::VectorStore;
    use crate::embedding::EmbeddingManager;

    #[tokio::test]
    async fn test_watcher_creation() {
        let config = FileWatcherConfig::default();
        let debouncer = Arc::new(Debouncer::new(100));
        let hash_validator = Arc::new(HashValidator::new());
        let grpc_operations = Arc::new(GrpcVectorOperations::new(
            Arc::new(VectorStore::new()),
            Arc::new(RwLock::new(EmbeddingManager::new())),
            None,
        ));

        let watcher = Watcher::new(config, debouncer, hash_validator, grpc_operations);
        assert!(watcher.is_ok());
    }

    #[tokio::test]
    async fn test_watcher_start_stop() {
        let temp_dir = tempdir().unwrap();
        let mut config = FileWatcherConfig::default();
        config.watch_paths = Some(vec![temp_dir.path().to_path_buf()]);
        config.debounce_delay_ms = 50;

        let debouncer = Arc::new(Debouncer::new(config.debounce_delay_ms));
        let hash_validator = Arc::new(HashValidator::new());
        let grpc_operations = Arc::new(GrpcVectorOperations::new(
            Arc::new(VectorStore::new()),
            Arc::new(RwLock::new(EmbeddingManager::new())),
            None,
        ));

        let mut watcher = Watcher::new(config, debouncer, hash_validator, grpc_operations).unwrap();

        // Start watcher
        watcher.start().await.unwrap();
        assert!(watcher.is_running().await);

        // Stop watcher
        watcher.stop().await.unwrap();
        assert!(!watcher.is_running().await);
    }
}
