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
        self.event_sender = Some(tx.clone());

        // Create notify watcher
        let mut notify_watcher = notify::recommended_watcher(
            move |res: std::result::Result<Event, notify::Error>| {
                match res {
                    Ok(event) => {
                        let _ = tx.send(FileChangeEvent::from_notify_event(event));
                    }
                    Err(e) => tracing::error!("Watch error: {:?}", e),
                }
            }
        ).map_err(|e| FileWatcherError::WatcherCreationFailed(e.to_string()))?;

        // Add paths to watch
        if let Some(paths) = &self.config.watch_paths {
            for path in paths {
                if path.exists() {
                    let recursive_mode = if self.config.recursive {
                        RecursiveMode::Recursive
                    } else {
                        RecursiveMode::NonRecursive
                    };
                    
                    notify_watcher.watch(path, recursive_mode)
                        .map_err(|e| FileWatcherError::PathWatchFailed(path.clone(), e.to_string()))?;
                    
                    tracing::info!("Watching path: {:?} (recursive: {})", path, self.config.recursive);
                } else {
                    tracing::warn!("Path does not exist, skipping: {:?}", path);
                }
            }
        }

        self.notify_watcher = Some(notify_watcher);
        self.is_running.store(true, Ordering::Relaxed);

        // Spawn event processing task
        let debouncer = self.debouncer.clone();
        tokio::spawn(async move {
            while let Some(event) = rx.recv().await {
                tracing::info!("🔍 File change detected: {:?}", event);
                debouncer.add_event(event).await;
            }
        });

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
