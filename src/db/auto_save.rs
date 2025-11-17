//! Auto-save manager for periodic compaction of vector store

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant};

use tokio::sync::RwLock;
use tokio::time::sleep;
use tracing::{error, info, warn};

use crate::db::VectorStore;
use crate::error::Result;
use crate::storage::{SnapshotManager, StorageCompactor};

/// Auto-save interval: 5 minutes
const SAVE_INTERVAL_SECS: u64 = 300;

/// Snapshot interval: 1 hour
const SNAPSHOT_INTERVAL_SECS: u64 = 3600;

/// Auto-save manager for periodic compaction
pub struct AutoSaveManager {
    /// Reference to the vector store
    store: Arc<VectorStore>,

    /// Storage compactor
    compactor: StorageCompactor,

    /// Snapshot manager
    snapshot_manager: SnapshotManager,

    /// Last save timestamp
    last_save: Arc<RwLock<Instant>>,

    /// Last snapshot timestamp
    last_snapshot: Arc<RwLock<Instant>>,

    /// Save interval (5 minutes)
    save_interval: Duration,

    /// Snapshot interval (1 hour)
    snapshot_interval: Duration,

    /// Flag indicating if changes were detected since last save
    changes_detected: Arc<AtomicBool>,

    /// Shutdown signal
    shutdown: Arc<AtomicBool>,
}

impl AutoSaveManager {
    /// Create a new auto-save manager with 5min save interval and 1h snapshot interval
    pub fn new(store: Arc<VectorStore>, _interval_hours: u64) -> Self {
        let data_dir = VectorStore::get_data_dir();
        let compactor = StorageCompactor::new(&data_dir, 6, 1000);

        // Create snapshot manager (keep last 48 hours / 2 days of snapshots)
        let snapshots_dir = data_dir.join(crate::storage::SNAPSHOT_DIR);
        let snapshot_manager = SnapshotManager::new(&data_dir, snapshots_dir, 48, 2); // 24 snapshots/day * 2 days = 48 snapshots max

        Self {
            store,
            compactor,
            snapshot_manager,
            last_save: Arc::new(RwLock::new(Instant::now())),
            last_snapshot: Arc::new(RwLock::new(Instant::now())),
            save_interval: Duration::from_secs(SAVE_INTERVAL_SECS),
            snapshot_interval: Duration::from_secs(SNAPSHOT_INTERVAL_SECS),
            changes_detected: Arc::new(AtomicBool::new(false)),
            shutdown: Arc::new(AtomicBool::new(false)),
        }
    }

    /// Start the auto-save background task
    pub fn start(&self) -> tokio::task::JoinHandle<()> {
        let store = self.store.clone();
        let last_save = self.last_save.clone();
        let last_snapshot = self.last_snapshot.clone();
        let save_interval = self.save_interval;
        let snapshot_interval = self.snapshot_interval;
        let changes_detected = self.changes_detected.clone();
        let shutdown = self.shutdown.clone();
        let data_dir = VectorStore::get_data_dir();
        let snapshots_dir = data_dir.join(crate::storage::SNAPSHOT_DIR);

        info!("🔄 AutoSave: Starting periodic tasks");
        info!("   Save interval: {} minutes", SAVE_INTERVAL_SECS / 60);
        info!(
            "   Snapshot interval: {} hour",
            SNAPSHOT_INTERVAL_SECS / 3600
        );

        tokio::spawn(async move {
            loop {
                // Check shutdown signal
                if shutdown.load(Ordering::Relaxed) {
                    info!("🛑 AutoSave: Shutdown signal received");
                    break;
                }

                // Sleep for 1 minute, then check what needs to be done
                sleep(Duration::from_secs(60)).await;

                // Check if it's time to save
                let time_since_save = {
                    let last = last_save.read().await;
                    last.elapsed()
                };

                if time_since_save >= save_interval && changes_detected.load(Ordering::Relaxed) {
                    info!(
                        "💾 AutoSave: {} minutes elapsed, starting compaction from memory...",
                        time_since_save.as_secs() / 60
                    );

                    // Perform compaction from memory (no raw files)
                    let mut compactor = StorageCompactor::new(&data_dir, 6, 1000);
                    match compactor.compact_from_memory(&store) {
                        Ok(index) => {
                            info!("✅ AutoSave: Successfully updated vectorizer.vecdb");
                            info!("   Collections: {}", index.collection_count());
                            info!("   Total vectors: {}", index.total_vectors());

                            // Update last save timestamp
                            let mut last = last_save.write().await;
                            *last = Instant::now();

                            // Reset changes flag
                            changes_detected.store(false, Ordering::Relaxed);
                        }
                        Err(e) => {
                            error!("❌ AutoSave: Compaction failed: {}", e);
                            error!("   vectorizer.vecdb remains unchanged");
                        }
                    }
                }

                // Check if it's time to snapshot
                let time_since_snapshot = {
                    let last = last_snapshot.read().await;
                    last.elapsed()
                };

                if time_since_snapshot >= snapshot_interval {
                    info!(
                        "📸 Snapshot: {} hour elapsed, creating snapshot...",
                        time_since_snapshot.as_secs() / 3600
                    );

                    // Ensure data directory exists before creating snapshot manager
                    if let Err(e) = std::fs::create_dir_all(&data_dir) {
                        error!(
                            "❌ Snapshot: Failed to create data directory {:?}: {}",
                            data_dir, e
                        );
                        continue;
                    }

                    let snapshot_mgr =
                        SnapshotManager::new(&data_dir, snapshots_dir.clone(), 48, 2); // 48 hours retention
                    match snapshot_mgr.create_snapshot() {
                        Ok(snapshot) => {
                            info!(
                                "✅ Snapshot: Created {} ({} MB)",
                                snapshot.id,
                                snapshot.size_bytes / 1_048_576
                            );

                            // Update last snapshot timestamp
                            let mut last = last_snapshot.write().await;
                            *last = Instant::now();
                        }
                        Err(e) => {
                            error!("❌ Snapshot: Failed to create snapshot: {}", e);
                        }
                    }
                }
            }

            info!("✅ AutoSave: Background task stopped");
        })
    }

    /// Mark that changes have been detected
    pub fn mark_changed(&self) {
        self.changes_detected.store(true, Ordering::Relaxed);
    }

    /// Force an immediate save from memory
    pub async fn force_save(&self) -> Result<()> {
        info!("💾 AutoSave: Forcing immediate compaction from memory...");

        let data_dir = VectorStore::get_data_dir();
        let mut compactor = StorageCompactor::new(&data_dir, 6, 1000);

        match compactor.compact_from_memory(&self.store) {
            Ok(index) => {
                info!("✅ AutoSave: Force save completed");
                info!("   Collections: {}", index.collection_count());

                // Update last save timestamp
                let mut last = self.last_save.write().await;
                *last = Instant::now();

                // Reset changes flag
                self.changes_detected.store(false, Ordering::Relaxed);

                Ok(())
            }
            Err(e) => {
                error!("❌ AutoSave: Force save failed: {}", e);
                Err(e)
            }
        }
    }

    /// Signal shutdown to the background task
    pub fn shutdown(&self) {
        info!("🛑 AutoSave: Signaling shutdown...");
        self.shutdown.store(true, Ordering::Relaxed);
    }

    /// Get time since last save
    pub async fn time_since_last_save(&self) -> Duration {
        let last = self.last_save.read().await;
        last.elapsed()
    }

    /// Check if changes are pending
    pub fn has_pending_changes(&self) -> bool {
        self.changes_detected.load(Ordering::Relaxed)
    }
}

impl Drop for AutoSaveManager {
    fn drop(&mut self) {
        self.shutdown();
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use tokio::time::sleep;

    use super::*;

    #[tokio::test]
    async fn test_auto_save_manager_creation() {
        let store = Arc::new(VectorStore::new());
        let manager = AutoSaveManager::new(store, 1);

        assert_eq!(manager.save_interval, Duration::from_secs(300)); // 5 minutes
        assert_eq!(manager.snapshot_interval, Duration::from_secs(3600)); // 1 hour
        assert!(!manager.has_pending_changes());
    }

    #[tokio::test]
    async fn test_mark_changed() {
        let store = Arc::new(VectorStore::new());
        let manager = AutoSaveManager::new(store, 1);

        assert!(!manager.has_pending_changes());

        manager.mark_changed();
        assert!(manager.has_pending_changes());
    }

    #[tokio::test]
    async fn test_time_since_last_save() {
        let store = Arc::new(VectorStore::new());
        let manager = AutoSaveManager::new(store, 1);

        sleep(Duration::from_millis(100)).await;

        let elapsed = manager.time_since_last_save().await;
        assert!(elapsed >= Duration::from_millis(100));
    }
}
