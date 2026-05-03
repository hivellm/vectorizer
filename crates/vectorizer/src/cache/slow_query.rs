//! Slow-query ring buffer — captures searches whose latency exceeds a
//! configured threshold.
//!
//! The `SlowQueryRing` is designed to add **zero overhead** to fast
//! queries: the only hot-path cost is a single `Duration >= threshold`
//! comparison. The ring buffer itself is written only when the threshold
//! is crossed.

use std::collections::VecDeque;
use std::sync::Arc;
use std::time::{Duration, Instant};

use chrono::{DateTime, Utc};
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use tracing::debug;

/// Runtime configuration for the slow-query log.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SlowQueryConfig {
    /// Latency threshold in milliseconds. Queries at or above this
    /// value are recorded. Default: 100 ms.
    pub threshold_ms: u64,
    /// Maximum number of entries retained in memory. When capacity is
    /// reached, the oldest entry is evicted. Default: 1 000.
    pub capacity: usize,
}

impl Default for SlowQueryConfig {
    fn default() -> Self {
        Self {
            threshold_ms: 100,
            capacity: 1_000,
        }
    }
}

/// One recorded slow query.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SlowQueryEntry {
    /// Wall-clock time the query was received.
    pub timestamp: DateTime<Utc>,
    /// Collection that was searched.
    pub collection: String,
    /// Number of results requested (`k`).
    pub k: usize,
    /// Actual latency in milliseconds.
    pub duration_ms: u64,
}

/// Capacity-bounded ring buffer for slow-query entries.
///
/// All public methods acquire an internal `RwLock`; the write path is
/// only exercised when a query exceeds the threshold, so fast queries
/// never contend on the lock.
#[derive(Clone, Debug)]
pub struct SlowQueryRing {
    inner: Arc<RwLock<SlowQueryRingInner>>,
}

#[derive(Debug)]
struct SlowQueryRingInner {
    config: SlowQueryConfig,
    entries: VecDeque<SlowQueryEntry>,
}

impl SlowQueryRing {
    /// Create a new ring buffer with the given configuration.
    pub fn new(config: SlowQueryConfig) -> Self {
        let capacity = config.capacity;
        Self {
            inner: Arc::new(RwLock::new(SlowQueryRingInner {
                config,
                entries: VecDeque::with_capacity(capacity),
            })),
        }
    }

    /// Create a ring buffer with default configuration.
    pub fn new_default() -> Self {
        Self::new(SlowQueryConfig::default())
    }

    /// Record a completed search.
    ///
    /// This is the **only** hot-path entry point. When `elapsed` is
    /// below the configured threshold the method returns immediately
    /// without acquiring the write lock.
    pub fn record(&self, collection: &str, k: usize, elapsed: Duration) {
        let elapsed_ms = elapsed.as_millis() as u64;

        // Fast path: skip lock acquisition entirely for fast queries.
        {
            let guard = self.inner.read();
            if elapsed_ms < guard.config.threshold_ms {
                return;
            }
        }

        debug!("slow query on '{}': {}ms (k={})", collection, elapsed_ms, k);

        let entry = SlowQueryEntry {
            timestamp: Utc::now(),
            collection: collection.to_string(),
            k,
            duration_ms: elapsed_ms,
        };

        let mut guard = self.inner.write();
        let capacity = guard.config.capacity;
        if guard.entries.len() >= capacity {
            guard.entries.pop_front();
        }
        guard.entries.push_back(entry);
    }

    /// Convenience wrapper: start a timer at the call site and record
    /// on drop / explicit `stop`.
    pub fn start_timer(&self) -> SlowQueryTimer<'_> {
        SlowQueryTimer {
            ring: self,
            start: Instant::now(),
            collection: String::new(),
            k: 0,
        }
    }

    /// Return a snapshot of all currently retained entries (oldest
    /// first).
    pub fn entries(&self) -> Vec<SlowQueryEntry> {
        self.inner.read().entries.iter().cloned().collect()
    }

    /// Return the current configuration.
    pub fn config(&self) -> SlowQueryConfig {
        self.inner.read().config.clone()
    }

    /// Replace the configuration.  Existing entries are retained; only
    /// `threshold_ms` and `capacity` change going forward. If the new
    /// capacity is smaller than the current entry count, the oldest
    /// entries are evicted until the buffer fits.
    pub fn set_config(&self, config: SlowQueryConfig) {
        let mut guard = self.inner.write();
        let new_cap = config.capacity;
        guard.config = config;
        while guard.entries.len() > new_cap {
            guard.entries.pop_front();
        }
    }

    /// Clear all retained entries.
    pub fn clear(&self) {
        self.inner.write().entries.clear();
    }
}

/// RAII timer that records the elapsed duration when `stop` is called.
///
/// The `collection` and `k` fields must be set before calling `stop`.
pub struct SlowQueryTimer<'a> {
    ring: &'a SlowQueryRing,
    start: Instant,
    /// Collection name — set this before calling `stop`.
    pub collection: String,
    /// Result count (`k`) — set this before calling `stop`.
    pub k: usize,
}

impl<'a> SlowQueryTimer<'a> {
    /// Record the elapsed time. Consumes the timer.
    pub fn stop(self) {
        self.ring
            .record(&self.collection, self.k, self.start.elapsed());
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use std::time::Duration;

    use super::*;

    #[test]
    fn below_threshold_not_recorded() {
        let ring = SlowQueryRing::new(SlowQueryConfig {
            threshold_ms: 100,
            capacity: 10,
        });
        ring.record("col", 10, Duration::from_millis(50));
        assert!(ring.entries().is_empty());
    }

    #[test]
    fn at_threshold_is_recorded() {
        let ring = SlowQueryRing::new(SlowQueryConfig {
            threshold_ms: 100,
            capacity: 10,
        });
        ring.record("col", 10, Duration::from_millis(100));
        assert_eq!(ring.entries().len(), 1);
        assert_eq!(ring.entries()[0].collection, "col");
        assert_eq!(ring.entries()[0].k, 10);
    }

    #[test]
    fn ring_buffer_eviction_at_capacity() {
        let capacity = 5;
        let ring = SlowQueryRing::new(SlowQueryConfig {
            threshold_ms: 0,
            capacity,
        });
        for i in 0..10u64 {
            ring.record(&format!("col{}", i), 1, Duration::from_millis(i + 1));
        }
        let entries = ring.entries();
        assert_eq!(entries.len(), capacity);
        // Oldest 5 entries (col0..col4) are evicted; newest 5 remain.
        assert_eq!(entries[0].collection, "col5");
        assert_eq!(entries[4].collection, "col9");
    }

    #[test]
    fn set_config_shrinks_buffer() {
        let ring = SlowQueryRing::new(SlowQueryConfig {
            threshold_ms: 0,
            capacity: 10,
        });
        for i in 0..10u64 {
            ring.record("col", 1, Duration::from_millis(i + 1));
        }
        assert_eq!(ring.entries().len(), 10);
        ring.set_config(SlowQueryConfig {
            threshold_ms: 0,
            capacity: 3,
        });
        assert_eq!(ring.entries().len(), 3);
    }
}
