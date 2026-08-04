//! TTL reaper.
//!
//! One `TtlReaper` runs on a tokio background task for the whole store.
//! It wakes up every `check_interval_secs` (default 60 s), enumerates the
//! collections, scans their vectors for an `__expires_at` payload field, and
//! batch-deletes the expired ones via the normal `VectorStore::delete` path
//! (which writes to the WAL and marks the collection for auto-save).
//!
//! The sweep enumerates collections on every tick rather than being bound to a
//! fixed set at spawn time. Collections are created at runtime over REST, RPC,
//! MCP and disk load, so a reaper-per-collection spawned at boot would silently
//! skip every collection created afterwards — and there is no single choke
//! point to hook a spawn into.
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
    /// Shutdown flag. Set to `true` to stop the loop.
    pub shutdown: Arc<AtomicBool>,
}

impl TtlReaper {
    /// Spawn the TTL reaper with metrics disabled (a [`NoopMetricsSink`]).
    /// Use [`TtlReaper::spawn_with_metrics`] to wire up real instrumentation —
    /// the three `ttl_*` Prometheus families report nothing through this one.
    ///
    /// Returns the reaper handle. The background task runs until the
    /// `shutdown` flag is set to `true`, which includes dropping this handle.
    pub fn spawn(store: Arc<VectorStore>, check_interval_secs: u64) -> Self {
        Self::spawn_with_metrics(store, check_interval_secs, Arc::new(NoopMetricsSink))
    }

    /// Spawn the TTL reaper, recording sweep lag, scan completions and
    /// expired-vector counts through `metrics`.
    ///
    /// Returns the reaper handle. The background task runs until the
    /// `shutdown` flag is set to `true`. **Keep the handle alive**: `Drop`
    /// signals shutdown, so letting it fall out of scope stops the reaper.
    pub fn spawn_with_metrics(
        store: Arc<VectorStore>,
        check_interval_secs: u64,
        metrics: Arc<dyn MetricsSink>,
    ) -> Self {
        let shutdown = Arc::new(AtomicBool::new(false));
        let shutdown_clone = shutdown.clone();

        tokio::spawn(async move {
            let interval = Duration::from_secs(check_interval_secs);
            info!("TTL reaper started (interval {}s)", check_interval_secs);

            loop {
                let scheduled_at = Instant::now();

                sleep(interval).await;

                if shutdown_clone.load(Ordering::Relaxed) {
                    info!("TTL reaper shutting down");
                    break;
                }

                // Record lag: how far past the scheduled wake-up are we?
                let lag = scheduled_at.elapsed().saturating_sub(interval);

                // Enumerated per tick, so collections created since the last
                // sweep are covered without any spawn hook on the create paths.
                for collection in store.list_collections() {
                    metrics.ttl_reaper_lag_seconds(&collection, lag.as_secs_f64());
                    Self::sweep(&store, &collection, metrics.as_ref());
                    metrics.ttl_reaper_scan_completed(&collection);
                }
            }
        });

        Self { shutdown }
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

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use vectorizer_core::metrics_sink::MetricsSink;

    use super::*;
    use crate::models::{CollectionConfig, DistanceMetric, Payload, Vector};

    /// Records what the reaper reported, so a test can assert the metrics the
    /// three `ttl_*` Prometheus families are fed from.
    #[derive(Debug, Default)]
    struct RecordingSink {
        scans: std::sync::Mutex<Vec<String>>,
        expired: std::sync::Mutex<Vec<(String, f64)>>,
    }

    impl MetricsSink for RecordingSink {
        fn ttl_reaper_scan_completed(&self, collection: &str) {
            self.scans.lock().unwrap().push(collection.to_string());
        }

        fn ttl_vectors_expired(&self, collection: &str, count: f64) {
            self.expired
                .lock()
                .unwrap()
                .push((collection.to_string(), count));
        }
    }

    fn store_with_collection(name: &str) -> Arc<VectorStore> {
        let store = Arc::new(VectorStore::new());
        store
            .create_collection(
                name,
                CollectionConfig {
                    dimension: 4,
                    metric: DistanceMetric::Cosine,
                    ..Default::default()
                },
            )
            .unwrap();
        store
    }

    /// Insert a vector whose expiry is `offset_ms` from now — negative for
    /// already expired, positive for still live.
    fn insert_with_expiry(store: &VectorStore, collection: &str, id: &str, offset_ms: i64) {
        let mut payload = Payload::new(serde_json::json!({ "id": id }));
        payload.set_expires_at(chrono::Utc::now().timestamp_millis() + offset_ms);
        store
            .insert(
                collection,
                vec![Vector {
                    id: id.to_string(),
                    data: vec![0.1, 0.2, 0.3, 0.4],
                    sparse: None,
                    payload: Some(payload),
                    document_id: None,
                }],
            )
            .unwrap();
    }

    fn insert_without_expiry(store: &VectorStore, collection: &str, id: &str) {
        store
            .insert(
                collection,
                vec![Vector {
                    id: id.to_string(),
                    data: vec![0.4, 0.3, 0.2, 0.1],
                    sparse: None,
                    payload: Some(Payload::new(serde_json::json!({ "id": id }))),
                    document_id: None,
                }],
            )
            .unwrap();
    }

    /// Is the vector still *stored*, expired or not?
    ///
    /// Deliberately not `store.get_vector`: that hides an expired vector from
    /// readers, so asserting on it would pass whether or not the sweep
    /// actually deleted anything. `get_all_vectors` is the raw accessor the
    /// reaper itself uses, so it distinguishes "filtered" from "removed".
    fn is_stored(store: &VectorStore, collection: &str, id: &str) -> bool {
        store
            .get_collection(collection)
            .map(|coll| coll.get_all_vectors().iter().any(|v| v.id == id))
            .unwrap_or(false)
    }

    #[test]
    fn sweep_deletes_expired_and_spares_the_rest() {
        let store = store_with_collection("reap");
        insert_with_expiry(&store, "reap", "gone", -60_000);
        insert_with_expiry(&store, "reap", "later", 600_000);
        insert_without_expiry(&store, "reap", "eternal");

        let sink = RecordingSink::default();
        TtlReaper::sweep(&store, "reap", &sink);

        assert!(
            !is_stored(&store, "reap", "gone"),
            "an expired vector must be removed from the store, not merely hidden"
        );
        assert!(
            is_stored(&store, "reap", "later"),
            "a future expiry must survive the sweep"
        );
        assert!(
            is_stored(&store, "reap", "eternal"),
            "a vector with no expiry must survive the sweep"
        );
        assert_eq!(
            sink.expired.lock().unwrap().as_slice(),
            &[("reap".to_string(), 1.0)],
            "the sweep must report how many it removed"
        );
    }

    #[test]
    fn sweep_reports_nothing_when_no_vector_expired() {
        let store = store_with_collection("quiet");
        insert_with_expiry(&store, "quiet", "later", 600_000);

        let sink = RecordingSink::default();
        TtlReaper::sweep(&store, "quiet", &sink);

        assert!(is_stored(&store, "quiet", "later"));
        assert!(
            sink.expired.lock().unwrap().is_empty(),
            "no deletions means no expired-count sample"
        );
    }

    #[test]
    fn sweep_on_a_missing_collection_does_not_panic() {
        let store = Arc::new(VectorStore::new());
        let sink = RecordingSink::default();
        // A collection can be dropped between enumeration and sweep.
        TtlReaper::sweep(&store, "never_existed", &sink);
        assert!(sink.expired.lock().unwrap().is_empty());
    }

    /// The reason the reaper sweeps the whole store instead of one collection:
    /// collections created after it started must still be reaped.
    #[tokio::test]
    async fn reaper_sweeps_a_collection_created_after_it_started() {
        let store = Arc::new(VectorStore::new());
        let sink = Arc::new(RecordingSink::default());
        let reaper = TtlReaper::spawn_with_metrics(store.clone(), 1, sink.clone());

        // Created only now — after the reaper was already running.
        store
            .create_collection(
                "late",
                CollectionConfig {
                    dimension: 4,
                    metric: DistanceMetric::Cosine,
                    ..Default::default()
                },
            )
            .unwrap();
        insert_with_expiry(&store, "late", "gone", -60_000);

        let deadline = tokio::time::Instant::now() + Duration::from_secs(10);
        while tokio::time::Instant::now() < deadline {
            if !is_stored(&store, "late", "gone") {
                break;
            }
            tokio::time::sleep(Duration::from_millis(200)).await;
        }
        assert!(
            !is_stored(&store, "late", "gone"),
            "the reaper never swept a collection created after it started"
        );
        assert!(
            sink.scans.lock().unwrap().iter().any(|c| c == "late"),
            "the late collection was never scanned"
        );
        reaper.stop();
    }

    #[tokio::test]
    async fn stop_halts_the_sweep_loop() {
        let store = store_with_collection("halt");
        let sink = Arc::new(RecordingSink::default());
        let reaper = TtlReaper::spawn_with_metrics(store.clone(), 1, sink.clone());
        reaper.stop();

        // The loop checks the flag after its first sleep, so give it a couple
        // of intervals to exit, then confirm it never scanned anything.
        tokio::time::sleep(Duration::from_secs(3)).await;
        insert_with_expiry(&store, "halt", "gone", -60_000);
        tokio::time::sleep(Duration::from_secs(2)).await;

        assert!(
            is_stored(&store, "halt", "gone"),
            "a stopped reaper must not delete anything"
        );
        assert!(
            sink.scans.lock().unwrap().is_empty(),
            "a reaper stopped before its first tick must not scan"
        );
    }

    // ── Read-path filtering ──────────────────────────────────────────────
    //
    // The reaper bounds how long an expired vector occupies memory; these
    // pin that it stops being *served* immediately, with no sweep involved.

    #[test]
    fn an_expired_vector_reads_as_absent_without_a_sweep() {
        let store = store_with_collection("filter_get");
        insert_with_expiry(&store, "filter_get", "gone", -1);
        insert_with_expiry(&store, "filter_get", "later", 600_000);

        assert!(
            store.get_vector("filter_get", "gone").is_err(),
            "an expired vector must not be readable, sweep or no sweep"
        );
        assert!(
            is_stored(&store, "filter_get", "gone"),
            "this test must exercise the read filter, not a deletion"
        );
        assert!(
            store.get_vector("filter_get", "later").is_ok(),
            "a live vector stays readable"
        );
    }

    #[test]
    fn an_expired_vector_is_not_a_search_hit() {
        let store = store_with_collection("filter_search");
        insert_with_expiry(&store, "filter_search", "gone", -1);
        insert_without_expiry(&store, "filter_search", "eternal");

        let hits = store
            .search("filter_search", &[0.4, 0.3, 0.2, 0.1], 10)
            .unwrap();
        let ids: Vec<&str> = hits.iter().map(|h| h.id.as_str()).collect();

        assert!(
            !ids.contains(&"gone"),
            "an expired vector was returned as a search hit: {ids:?}"
        );
        assert!(
            ids.contains(&"eternal"),
            "the live vector must still be found: {ids:?}"
        );
        assert!(
            is_stored(&store, "filter_search", "gone"),
            "this test must exercise the read filter, not a deletion"
        );
    }

    #[test]
    fn an_expiry_in_the_future_is_not_filtered() {
        let store = store_with_collection("filter_future");
        insert_with_expiry(&store, "filter_future", "later", 600_000);

        assert!(store.get_vector("filter_future", "later").is_ok());
        let hits = store
            .search("filter_future", &[0.1, 0.2, 0.3, 0.4], 10)
            .unwrap();
        assert!(
            hits.iter().any(|h| h.id == "later"),
            "a future expiry must not hide the vector"
        );
    }
}
