//! Debouncing mechanism for file change events

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use tokio::time::sleep;
use crate::file_watcher::{FileChangeEvent, FileChangeEventWithMetadata};

/// Debouncer for file change events
pub struct Debouncer {
    /// Debounce delay in milliseconds
    delay_ms: u64,
    /// Pending events waiting for debounce
    pending_events: Arc<RwLock<HashMap<PathBuf, PendingEvent>>>,
    /// Event callback
    event_callback: Arc<RwLock<Option<Box<dyn Fn(FileChangeEventWithMetadata) + Send + Sync>>>>,
}

/// Pending event with metadata
#[derive(Debug, Clone)]
struct PendingEvent {
    event: FileChangeEvent,
    timestamp: chrono::DateTime<chrono::Utc>,
    content_hash: Option<String>,
    file_size: Option<u64>,
    last_modified: Instant,
}

impl Debouncer {
    /// Create a new debouncer
    pub fn new(delay_ms: u64) -> Self {
        Self {
            delay_ms,
            pending_events: Arc::new(RwLock::new(HashMap::new())),
            event_callback: Arc::new(RwLock::new(None)),
        }
    }

    /// Set the event callback
    pub async fn set_event_callback<F>(&self, callback: F)
    where
        F: Fn(FileChangeEventWithMetadata) + Send + Sync + 'static,
    {
        let mut cb = self.event_callback.write().await;
        *cb = Some(Box::new(callback));
    }

    /// Add a file change event for debouncing
    pub async fn add_event(&self, event: FileChangeEvent) {
        let path = match &event {
            FileChangeEvent::Created(path) => path.clone(),
            FileChangeEvent::Modified(path) => path.clone(),
            FileChangeEvent::Deleted(path) => path.clone(),
            FileChangeEvent::Renamed(_, new_path) => new_path.clone(),
        };

        let pending_event = PendingEvent {
            event: event.clone(),
            timestamp: chrono::Utc::now(),
            content_hash: None,
            file_size: None,
            last_modified: Instant::now(),
        };

        // Store the pending event
        {
            let mut events = self.pending_events.write().await;
            events.insert(path.clone(), pending_event);
        }

        // Start debounce timer for this event
        self.start_debounce_timer(path).await;
    }

    /// Start debounce timer for a specific path
    async fn start_debounce_timer(&self, path: PathBuf) {
        let delay = Duration::from_millis(self.delay_ms);
        let pending_events = Arc::clone(&self.pending_events);
        let event_callback = Arc::clone(&self.event_callback);

        tokio::spawn(async move {
            sleep(delay).await;

            // Check if event is still pending
            let event = {
                let mut events = pending_events.write().await;
                events.remove(&path)
            };

            if let Some(pending_event) = event {
                // Get file metadata if available
                let (content_hash, file_size) = if let Ok(metadata) = std::fs::metadata(&path) {
                    let file_size = Some(metadata.len());
                    let content_hash = if metadata.is_file() {
                        // Calculate content hash for files
                        match std::fs::read(&path) {
                            Ok(content) => {
                                use sha2::Digest;
                                Some(sha2::Sha256::digest(&content).iter().map(|b| format!("{:02x}", b)).collect::<String>())
                            },
                            Err(_) => None,
                        }
                    } else {
                        None
                    };
                    (content_hash, file_size)
                } else {
                    (None, None)
                };

                let event_with_metadata = FileChangeEventWithMetadata {
                    event: pending_event.event.clone(),
                    timestamp: pending_event.timestamp,
                    content_hash,
                    file_size,
                };

                // Call the event callback
                if let Some(callback) = event_callback.read().await.as_ref() {
                    tracing::info!("🔍 DEBOUNCER: Calling callback for event: {:?}", event_with_metadata.event);
                    callback(event_with_metadata);
                    tracing::info!("✅ DEBOUNCER: Callback completed for event: {:?}", pending_event.event);
                } else {
                    tracing::warn!("⚠️ DEBOUNCER: No callback set for event: {:?}", pending_event.event);
                }
            }
        });
    }

    /// Get pending events count
    pub async fn pending_events_count(&self) -> usize {
        let events = self.pending_events.read().await;
        events.len()
    }

    /// Clear all pending events
    pub async fn clear_pending_events(&self) {
        let mut events = self.pending_events.write().await;
        events.clear();
    }

    /// Get debounce delay
    pub fn delay_ms(&self) -> u64 {
        self.delay_ms
    }

    /// Update debounce delay
    pub fn set_delay_ms(&mut self, delay_ms: u64) {
        self.delay_ms = delay_ms;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use tokio::sync::Mutex;

    #[tokio::test]
    async fn test_debouncer_creation() {
        let debouncer = Debouncer::new(100);
        assert_eq!(debouncer.delay_ms(), 100);
        assert_eq!(debouncer.pending_events_count().await, 0);
    }

    #[tokio::test]
    async fn test_debouncer_event_handling() {
        let debouncer = Debouncer::new(50);
        let events_received = Arc::new(Mutex::new(Vec::new()));

        let events_clone = Arc::clone(&events_received);
        debouncer.set_event_callback(move |event| {
            let events_clone = Arc::clone(&events_clone);
            tokio::spawn(async move {
                let mut events = events_clone.lock().await;
                events.push(event);
            });
        }).await;

        // Add an event
        let test_path = PathBuf::from("test.txt");
        debouncer.add_event(FileChangeEvent::Modified(test_path.clone())).await;

        // Wait for debounce
        sleep(Duration::from_millis(100)).await;

        // Check if event was received
        let events = events_received.lock().await;
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].event, FileChangeEvent::Modified(test_path));
    }

    #[tokio::test]
    async fn test_debouncer_multiple_events() {
        let debouncer = Debouncer::new(50);
        let events_received = Arc::new(Mutex::new(Vec::new()));

        let events_clone = Arc::clone(&events_received);
        debouncer.set_event_callback(move |event| {
            let events_clone = Arc::clone(&events_clone);
            tokio::spawn(async move {
                let mut events = events_clone.lock().await;
                events.push(event);
            });
        }).await;

        // Add multiple events for the same file
        let test_path = PathBuf::from("test.txt");
        debouncer.add_event(FileChangeEvent::Modified(test_path.clone())).await;
        debouncer.add_event(FileChangeEvent::Modified(test_path.clone())).await;
        debouncer.add_event(FileChangeEvent::Modified(test_path.clone())).await;

        // Wait for debounce
        sleep(Duration::from_millis(100)).await;

        // Should only receive one event (last one)
        let events = events_received.lock().await;
        assert_eq!(events.len(), 1);
    }

    #[tokio::test]
    async fn test_debouncer_clear_pending() {
        let debouncer = Debouncer::new(1000); // Long delay
        let test_path = PathBuf::from("test.txt");
        
        debouncer.add_event(FileChangeEvent::Modified(test_path)).await;
        assert_eq!(debouncer.pending_events_count().await, 1);

        debouncer.clear_pending_events().await;
        assert_eq!(debouncer.pending_events_count().await, 0);
    }
}
