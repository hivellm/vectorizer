//! Progress of the store-wide catalog load that runs at startup
//! (phase1_collections-list-hides-lazy-load-progress, issue #391).
//!
//! The server loads persisted collections on a background task, inserting
//! them into the store one at a time. Until that finishes, every reader sees
//! a partial store — and `GET /collections` answered with a partial list that
//! carried no hint it was partial. On a 181-collection store, a client asking
//! 20s after boot got 11 collections and a `total_collections: 11` that
//! agreed with them. That is indistinguishable from catastrophic data loss,
//! and it triggered rollback procedures during a 3.5→3.6 upgrade for what was
//! ordinary warm-up.
//!
//! This type is the missing signal. The loader publishes into it; the REST
//! surface reads a [`CollectionLoadSnapshot`] and reports whether the answer
//! it is about to give is complete.
//!
//! Distinct from [`crate::db::IndexBuildProgress`], which tracks vectors
//! within one collection's HNSW rebuild. This one tracks collections within
//! the store.

use std::sync::atomic::{AtomicUsize, Ordering};

use parking_lot::RwLock;
use serde::{Deserialize, Serialize};

/// Where the startup catalog load currently stands.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase", tag = "state", content = "error")]
pub enum CollectionLoadStatus {
    /// The loader has not started yet. Counts are meaningless.
    Pending,
    /// The loader is walking the catalog. The store is incomplete, and any
    /// listing taken now is a snapshot of a moving target.
    Loading,
    /// Every collection the loader intended to load is in the store.
    Complete,
    /// The load ended early. The store holds whatever had landed before the
    /// failure, and the string says why it stopped.
    Failed(String),
}

/// An immutable read of [`CollectionLoadProgress`], safe to serialize
/// straight into a response body.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CollectionLoadSnapshot {
    /// Collections the loader intends to load. Zero while
    /// [`CollectionLoadStatus::Pending`], since the catalog has not been read.
    pub expected: usize,
    /// Collections handed to the store so far.
    pub loaded: usize,
    /// Current state of the load.
    pub status: CollectionLoadStatus,
}

impl CollectionLoadSnapshot {
    /// Whether the store is still filling up. Callers that must not mistake a
    /// partial answer for a complete one should branch on this.
    #[must_use]
    pub fn is_loading(&self) -> bool {
        matches!(
            self.status,
            CollectionLoadStatus::Pending | CollectionLoadStatus::Loading
        )
    }

    /// Whether the catalog is fully materialized. `false` for a failed load —
    /// a load that stopped early did not deliver the catalog, so a reader
    /// gating on readiness must not treat it as ready.
    #[must_use]
    pub fn is_complete(&self) -> bool {
        matches!(self.status, CollectionLoadStatus::Complete)
    }
}

/// Shared, concurrently-updated progress of the startup catalog load.
///
/// Held behind an `Arc` by the server and written by the background loader.
/// Cheap to read: the hot path is a counter increment per collection.
#[derive(Debug)]
pub struct CollectionLoadProgress {
    expected: AtomicUsize,
    loaded: AtomicUsize,
    status: RwLock<CollectionLoadStatus>,
}

impl CollectionLoadProgress {
    /// A progress handle that has not started. This is the state a real
    /// server boots into.
    #[must_use]
    pub fn new() -> Self {
        Self {
            expected: AtomicUsize::new(0),
            loaded: AtomicUsize::new(0),
            status: RwLock::new(CollectionLoadStatus::Pending),
        }
    }

    /// A handle that is already settled, for callers that never load a
    /// catalog — the test harness, and any path where auto-load is off.
    ///
    /// Without this, a server that legitimately loads nothing would report
    /// `Pending` forever and never become ready.
    #[must_use]
    pub fn already_complete() -> Self {
        let progress = Self::new();
        progress.finish();
        progress
    }

    /// Declare how many collections the load will cover and move to
    /// [`CollectionLoadStatus::Loading`].
    ///
    /// Called once the catalog has been read but before any collection is
    /// inserted, so a reader arriving mid-load sees a real denominator.
    pub fn begin(&self, expected: usize) {
        self.expected.store(expected, Ordering::Relaxed);
        self.loaded.store(0, Ordering::Relaxed);
        *self.status.write() = CollectionLoadStatus::Loading;
    }

    /// Record one collection as landed in the store.
    pub fn record_loaded(&self) {
        self.loaded.fetch_add(1, Ordering::Relaxed);
    }

    /// Mark the load finished.
    ///
    /// The counter store happens before the lock is taken, so any reader that
    /// observes `Complete` also observes the final `loaded` — the lock's
    /// release/acquire pair orders the two.
    pub fn finish(&self) {
        let mut status = self.status.write();
        *status = CollectionLoadStatus::Complete;
    }

    /// Mark the load as stopped early, recording why.
    pub fn fail(&self, reason: impl Into<String>) {
        let mut status = self.status.write();
        *status = CollectionLoadStatus::Failed(reason.into());
    }

    /// Take a consistent read of the current progress.
    #[must_use]
    pub fn snapshot(&self) -> CollectionLoadSnapshot {
        // Status first: holding the read lock while sampling the counters
        // pairs with the write in `finish`, so `Complete` never travels with
        // a stale `loaded`.
        let status = self.status.read().clone();
        CollectionLoadSnapshot {
            expected: self.expected.load(Ordering::Relaxed),
            loaded: self.loaded.load(Ordering::Relaxed),
            status,
        }
    }
}

impl Default for CollectionLoadProgress {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;

    #[test]
    fn starts_pending_with_no_counts() {
        let snap = CollectionLoadProgress::new().snapshot();
        assert_eq!(snap.status, CollectionLoadStatus::Pending);
        assert_eq!(snap.expected, 0);
        assert_eq!(snap.loaded, 0);
        // Pending counts as loading: the catalog has not been read, so a
        // listing taken now is every bit as partial as one taken mid-load.
        assert!(snap.is_loading());
        assert!(!snap.is_complete());
    }

    #[test]
    fn mid_load_reports_a_real_denominator() {
        let progress = CollectionLoadProgress::new();
        progress.begin(181);
        for _ in 0..11 {
            progress.record_loaded();
        }

        // The exact shape from issue #391: 11 of 181, and the response must
        // be able to say so instead of implying 11 is the whole store.
        let snap = progress.snapshot();
        assert_eq!(snap.status, CollectionLoadStatus::Loading);
        assert_eq!(snap.expected, 181);
        assert_eq!(snap.loaded, 11);
        assert!(snap.is_loading());
        assert!(!snap.is_complete());
    }

    #[test]
    fn completion_settles_the_flag() {
        let progress = CollectionLoadProgress::new();
        progress.begin(2);
        progress.record_loaded();
        progress.record_loaded();
        progress.finish();

        let snap = progress.snapshot();
        assert_eq!(snap.status, CollectionLoadStatus::Complete);
        assert_eq!(snap.loaded, 2);
        assert_eq!(snap.expected, 2);
        assert!(!snap.is_loading());
        assert!(snap.is_complete());
    }

    #[test]
    fn failure_settles_the_flag_and_keeps_the_partial_count() {
        let progress = CollectionLoadProgress::new();
        progress.begin(181);
        progress.record_loaded();
        progress.fail("vectorizer.vecdb is corrupt");

        let snap = progress.snapshot();
        assert_eq!(
            snap.status,
            CollectionLoadStatus::Failed("vectorizer.vecdb is corrupt".to_string())
        );
        // A failed load is settled — nothing more is coming — but it is not
        // ready either: the catalog was never delivered.
        assert!(!snap.is_loading());
        assert!(!snap.is_complete());
        assert_eq!(snap.loaded, 1);
        assert_eq!(snap.expected, 181);
    }

    #[test]
    fn already_complete_is_ready_without_a_load() {
        // The no-op paths: auto-load disabled, and the test harness. Both
        // load nothing and must still report ready.
        let snap = CollectionLoadProgress::already_complete().snapshot();
        assert!(snap.is_complete());
        assert!(!snap.is_loading());
        assert_eq!(snap.expected, 0);
        assert_eq!(snap.loaded, 0);
    }

    #[test]
    fn concurrent_increments_are_not_lost() {
        use std::sync::Arc;

        let progress = Arc::new(CollectionLoadProgress::new());
        progress.begin(400);

        let handles: Vec<_> = (0..4)
            .map(|_| {
                let progress = Arc::clone(&progress);
                std::thread::spawn(move || {
                    for _ in 0..100 {
                        progress.record_loaded();
                    }
                })
            })
            .collect();
        for handle in handles {
            handle.join().expect("worker thread panicked");
        }
        progress.finish();

        let snap = progress.snapshot();
        assert_eq!(snap.loaded, 400);
        assert!(snap.is_complete());
    }
}
