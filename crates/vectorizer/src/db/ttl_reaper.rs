//! Per-collection TTL reaper.
//!
//! Each active collection may have a `TtlReaper` running on a tokio
//! background task. The reaper wakes up every `check_interval_secs`
//! (default 60 s), scans all vectors in the collection for an
//! `__expires_at` payload field, and batch-deletes the expired ones via
//! the normal `VectorStore::delete` path (which writes to the WAL and
//! marks the collection for auto-save).
//!
//! The reaper does NOT hold a write lock across the whole sweep — it
//! collects the IDs of expired vectors into a `Vec`, then issues
//! individual deletes using the store's interior-mutable
//! `VectorStore::delete`. Concurrent writes therefore see at most the
//! normal per-delete lock contention of the storage backend.
//!
//! Shutdown is signalled via an `Arc<AtomicBool>`, matching the pattern
//! used by `AutoSaveManager` in `src/db/auto_save.rs`.
//!
//! Metrics are recorded through the [`MetricsSink`] trait (injected via
//! [`TtlReaper::spawn_with_metrics`]) instead of a direct dependency on
//! `crate::monitoring` — see phase41 §1 (2026-07-11 improvement
//! analysis, §1.1).

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant};

use tokio::time::sleep;
use tracing::{debug, info, warn};
use vectorizer_core::metrics_sink::{MetricsSink, NoopMetricsSink};

use crate::db::VectorStore;

/// Default reaper sweep interval in seconds.
pub const DEFAULT_REAPER_INTERVAL_SECS: u64 = 60;

/// A handle to a running TTL reaper task.
///
/// Dropping this handle does NOT stop the task — call [`TtlReaper::stop`]
/// or signal the shared `shutdown` flag first.
pub struct TtlReaper {
    /// Collection being swept.
    pub collection: String,
    /// Shutdown flag. Set to `true` to stop the loop.
    pub shutdown: Arc<AtomicBool>,
}

impl TtlReaper {
    /// Spawn a TTL reaper task for `collection`, with metrics disabled
    /// (a [`NoopMetricsSink`]). Use [`TtlReaper::spawn_with_metrics`]
    /// to wire up real instrumentation.
    ///
    /// Returns the reaper handle. The background task runs until the
    /// `shutdown` flag is set to `true`.
    pub fn spawn(store: Arc<VectorStore>, collection: String, check_interval_secs: u64) -> Self {
        Self::spawn_with_metrics(
            store,
            collection,
            check_interval_secs,
            Arc::new(NoopMetricsSink),
        )
    }

    /// Spawn a TTL reaper task for `collection`, recording sweep lag,
    /// scan completions, and expired-vector counts through `metrics`.
    ///
    /// Returns the reaper handle. The background task runs until the
    /// `shutdown` flag is set to `true`.
    pub fn spawn_with_metrics(
        store: Arc<VectorStore>,
        collection: String,
        check_interval_secs: u64,
        metrics: Arc<dyn MetricsSink>,
    ) -> Self {
        let shutdown = Arc::new(AtomicBool::new(false));
        let shutdown_clone = shutdown.clone();
        let collection_clone = collection.clone();

        tokio::spawn(async move {
            let interval = Duration::from_secs(check_interval_secs);
            info!(
                "TTL reaper started for collection '{}' (interval {}s)",
                collection_clone, check_interval_secs
            );

            loop {
                let scheduled_at = Instant::now();

                sleep(interval).await;

                if shutdown_clone.load(Ordering::Relaxed) {
                    info!(
                        "TTL reaper shutting down for collection '{}'",
                        collection_clone
                    );
                    break;
                }

                // Record lag: how far past the scheduled wake-up are we?
                let lag = scheduled_at.elapsed().saturating_sub(interval);
                metrics.ttl_reaper_lag_seconds(&collection_clone, lag.as_secs_f64());

                Self::sweep(&store, &collection_clone, metrics.as_ref());

                metrics.ttl_reaper_scan_completed(&collection_clone);
            }
        });

        Self {
            collection,
            shutdown,
        }
    }

    /// Signal the reaper task to stop on the next wake-up.
    pub fn stop(&self) {
        self.shutdown.store(true, Ordering::Relaxed);
    }

    /// Perform a single sweep: collect expired vector IDs, batch-delete.
    ///
    /// Uses a shared (read) reference to the collection so concurrent
    /// writes are not blocked during ID collection. Individual deletes
    /// use the standard `VectorStore::delete` path.
    fn sweep(store: &VectorStore, collection: &str, metrics: &dyn MetricsSink) {
        let now_ms = chrono::Utc::now().timestamp_millis();

        // Collect expired IDs via a read-only pass.
        let expired_ids: Vec<String> = match store.get_collection(collection) {
            Ok(coll_ref) => {
                let all = coll_ref.get_all_vectors();
                all.into_iter()
                    .filter(|v| v.payload.as_ref().map_or(false, |p| p.is_expired(now_ms)))
                    .map(|v| v.id)
                    .collect()
            }
            Err(e) => {
                warn!(
                    "TTL reaper: cannot access collection '{}': {}",
                    collection, e
                );
                return;
            }
        };

        if expired_ids.is_empty() {
            debug!("TTL reaper: no expired vectors in '{}'", collection);
            return;
        }

        let count = expired_ids.len();
        let mut deleted: usize = 0;
        for id in &expired_ids {
            match store.delete(collection, id) {
                Ok(()) => deleted += 1,
                Err(e) => {
                    // The vector may already have been deleted by a concurrent call.
                    debug!(
                        "TTL reaper: could not delete '{}' from '{}': {}",
                        id, collection, e
                    );
                }
            }
        }

        if deleted > 0 {
            metrics.ttl_vectors_expired(collection, deleted as f64);
            info!(
                "TTL reaper: expired {}/{} vectors from '{}'",
                deleted, count, collection
            );
        }
    }
}

impl Drop for TtlReaper {
    fn drop(&mut self) {
        self.stop();
    }
}
