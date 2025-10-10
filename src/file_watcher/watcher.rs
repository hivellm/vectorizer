//! Functional file watcher implementation

use std::sync::{Arc, atomic::{AtomicBool, Ordering}};
use std::path::Path;
use tokio::sync::mpsc;
use notify::{Watcher as NotifyWatcher, RecursiveMode, Event, EventKind};
use super::{
    config::FileWatcherConfig,
    debouncer::Debouncer,
    hash_validator::HashValidator,
    FileChangeEvent, FileChangeEventWithMetadata, Result, FileWatcherError
};

/// Functional file watcher implementation
pub struct Watcher {
    config: FileWatcherConfig,
    debouncer: Arc<Debouncer>,
    hash_validator: Arc<HashValidator>,
    is_running: Arc<AtomicBool>,
    event_sender: Option<mpsc::UnboundedSender<FileChangeEvent>>,
    notify_watcher: Option<notify::RecommendedWatcher>,
}

impl Watcher {
    pub fn new(
        config: FileWatcherConfig,
        debouncer: Arc<Debouncer>,
        hash_validator: Arc<HashValidator>,
    ) -> Result<Self> {
        Ok(Self {
            config,
            debouncer,
            hash_validator,
            is_running: Arc::new(AtomicBool::new(false)),
            event_sender: None,
            notify_watcher: None,
        })
    }

    pub async fn start(&mut self) -> Result<()> {
        if self.is_running.load(Ordering::Relaxed) {
            return Err(FileWatcherError::AlreadyRunning);
        }

        // Create event channel
        let (tx, mut rx) = mpsc::unbounded_channel();
        
        // Clone tx for the watcher callback
        let tx_for_watcher = tx.clone();
        
        // Store tx to keep the channel alive
        self.event_sender = Some(tx);

        // Store the receiver for later use
        let mut rx_for_task = rx;

        // Clone config for the watcher closure
        let config_for_watcher = self.config.clone();
        
        // Create notify watcher with specific configuration
        let mut notify_watcher = notify::recommended_watcher(
            move |res: std::result::Result<Event, notify::Error>| {
                match res {
                    Ok(event) => {
                        // Filter out excluded files and directories BEFORE processing using config
                        let should_process = event.paths.iter().all(|path| {
                            // Use the configuration to check if file should be processed (silently)
                            config_for_watcher.should_process_file_silent(path)
                        });
                        
                        if !should_process {
                            // Skip logging for .log files to avoid spam
                            return;
                        }
                        
                        tracing::info!("🔍 NOTIFY: Raw event received: kind={:?}, paths={:?}", event.kind, event.paths);
                        
                        // Filter events to only process relevant ones
                        match &event.kind {
                            notify::EventKind::Create(_) => {
                                tracing::info!("🔍 NOTIFY: CREATE event detected: {:?}", event.paths);
                            }
                            notify::EventKind::Modify(_) => {
                                tracing::info!("🔍 NOTIFY: MODIFY event detected: {:?}", event.paths);
                            }
                            notify::EventKind::Remove(_) => {
                                tracing::info!("🔍 NOTIFY: REMOVE event detected: {:?}", event.paths);
                            }
                            notify::EventKind::Access(_) => {
                                tracing::info!("🔍 NOTIFY: ACCESS event detected: {:?}", event.paths);
                            }
                            _ => {
                                tracing::info!("🔍 NOTIFY: OTHER event detected: {:?}", event.paths);
                            }
                        }
                        
                        let file_event = FileChangeEvent::from_notify_event(event);
                        tracing::info!("🔍 NOTIFY: Converted to FileChangeEvent: {:?}", file_event);
                        
                        // Try to send event to channel
                        match tx_for_watcher.send(file_event.clone()) {
                            Ok(_) => {
                                tracing::info!("✅ NOTIFY: Event sent to channel successfully: {:?}", file_event);
                            }
                            Err(e) => {
                                tracing::error!("❌ NOTIFY: Failed to send event to channel: {:?}", e);
                                // If channel is closed, return to avoid infinite errors
                                return;
                            }
                        }
                    }
                    Err(e) => tracing::error!("Watch error: {:?}", e),
                }
            }
        ).map_err(|e| FileWatcherError::WatcherCreationFailed(e.to_string()))?;

        // Add paths to watch
        if let Some(paths) = &self.config.watch_paths {
            tracing::info!("🔍 WATCHER: Processing {} watch paths: {:?}", paths.len(), paths);
            for (i, path) in paths.iter().enumerate() {
                tracing::info!("🔍 WATCHER: Processing path {}/{}: {:?}", i+1, paths.len(), path);
                if path.exists() {
                    let recursive_mode = if self.config.recursive {
                        RecursiveMode::Recursive
                    } else {
                        RecursiveMode::NonRecursive
                    };
                    
                    tracing::info!("🔍 WATCHER: Adding path to notify watcher: {:?} (recursive: {})", path, self.config.recursive);
                    
                    // Skip paths that are already covered by parent paths
                    if i > 0 {
                        let is_covered = paths[0..i].iter().any(|parent_path| {
                            path.starts_with(parent_path)
                        });
                        
                        if is_covered {
                            tracing::info!("🔍 WATCHER: Path {:?} is already covered by a parent path, skipping", path);
                            continue;
                        }
                    }
                    
                    match notify_watcher.watch(path, recursive_mode) {
                        Ok(_) => {
                            tracing::info!("✅ WATCHER: Successfully watching path: {:?} (recursive: {})", path, self.config.recursive);
                        }
                        Err(e) => {
                            tracing::error!("❌ WATCHER: Failed to watch path {:?}: {:?}", path, e);
                            return Err(FileWatcherError::PathWatchFailed(path.clone(), e.to_string()));
                        }
                    }
                } else {
                    tracing::warn!("❌ WATCHER: Path does not exist, skipping: {:?}", path);
                }
            }
        } else {
            tracing::warn!("❌ WATCHER: No watch paths configured");
        }

        self.notify_watcher = Some(notify_watcher);
        self.is_running.store(true, Ordering::Relaxed);

        // Now spawn the event processing task AFTER watcher is created
        let debouncer = self.debouncer.clone();
        let hash_validator = self.hash_validator.clone();
        tracing::info!("🔍 About to spawn event processing task AFTER watcher creation...");
        let task_handle = tokio::spawn(async move {
            tracing::info!("🔍 Event processing task ENTERED and STARTED");
            tracing::info!("🔍 Event processing task waiting for events...");
            
            // Test if task is actually running
            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
            tracing::info!("🔍 Task is running - sleep completed");
            
            while let Some(event) = rx_for_task.recv().await {
                tracing::info!("🔍 File change detected: {:?}", event);
                
                // Check if the path is a file or directory
                let path = match &event {
                    FileChangeEvent::Created(path) => path,
                    FileChangeEvent::Modified(path) => path,
                    FileChangeEvent::Deleted(path) => path,
                    FileChangeEvent::Renamed(_, new_path) => new_path,
                };
                
                if path.exists() {
                    if path.is_file() {
                        tracing::info!("🔍 FILE: Processing file event: {:?}", path);
                    } else if path.is_dir() {
                        tracing::info!("🔍 DIR: Processing directory event: {:?}", path);
                    } else {
                        tracing::info!("🔍 OTHER: Processing other event: {:?}", path);
                    }
                } else {
                    tracing::info!("🔍 MISSING: Path does not exist: {:?}", path);
                }
                
                // Create event with metadata
                let event_with_metadata = FileChangeEventWithMetadata {
                    event: event.clone(),
                    timestamp: chrono::Utc::now(),
                    content_hash: None, // Will be calculated by hash_validator if needed
                    file_size: None,    // Will be calculated if needed
                };
                
                // Add event to debouncer
                debouncer.add_event_with_metadata(event_with_metadata).await;
            }
            tracing::info!("🔍 File watcher event processing task ended");
        });
        
        tracing::info!("✅ Event processing task spawned with handle: {:?}", task_handle.id());

        tracing::info!("File watcher started successfully");
        Ok(())
    }

    pub fn stop(&mut self) -> Result<()> {
        if !self.is_running.load(Ordering::Relaxed) {
            return Ok(());
        }

        // Close event sender to stop the processing task
        self.event_sender = None;
        
        // Stop the notify watcher
        if let Some(mut watcher) = self.notify_watcher.take() {
            watcher.unwatch(std::path::Path::new("."))
                .map_err(|e| FileWatcherError::WatcherStopFailed(e.to_string()))?;
        }

        self.is_running.store(false, Ordering::Relaxed);
        tracing::info!("File watcher stopped");
        Ok(())
    }

    pub fn is_running(&self) -> bool {
        self.is_running.load(Ordering::Relaxed)
    }

    pub fn get_config(&self) -> &FileWatcherConfig {
        &self.config
    }

    pub fn get_watched_paths(&self) -> Vec<String> {
        self.config.watch_paths.clone().unwrap_or_default()
            .into_iter()
            .map(|p| p.to_string_lossy().to_string())
            .collect()
    }

    pub fn get_event_count(&self) -> u64 {
        0 // Simple implementation
    }

    pub fn get_last_event_time(&self) -> Option<std::time::SystemTime> {
        None // Simple implementation
    }

    pub fn clear_events(&mut self) {
        // Simple implementation - no-op
    }

    pub fn get_recent_events(&self, _limit: usize) -> Vec<FileChangeEventWithMetadata> {
        Vec::new() // Simple implementation
    }
}
