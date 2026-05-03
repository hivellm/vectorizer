//! Per-API-key per-day usage ring buffer.
//!
//! Keeps a fixed-size ring of daily counters per key id. The dashboard
//! sparkline + the `GET /auth/keys/{id}/usage` endpoint read from this
//! buffer; `AuthManager::validate_api_key` writes to it on every
//! successful credential acceptance.
//!
//! Day boundaries are computed from `chrono::Utc::now().date_naive()`,
//! so a "day" always means UTC midnight-to-midnight regardless of the
//! caller's locale. The default ring holds 30 days; older buckets are
//! evicted as new days roll in.

use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

use chrono::{Duration as ChronoDuration, NaiveDate, Utc};
use dashmap::DashMap;
use parking_lot::RwLock;
use serde::Serialize;

/// Default ring buffer size — the dashboard never asks for more than
/// 30 days; longer windows would add memory pressure without a
/// matching read pattern.
pub const DEFAULT_WINDOW_DAYS: usize = 30;

/// One day's bucket: the calendar date plus the running counter for
/// that day. Atomics let concurrent `record` calls bump the same bucket
/// without taking the outer ring lock.
#[derive(Debug)]
struct DayBucket {
    date: NaiveDate,
    count: AtomicU64,
}

/// Per-key ring of daily buckets.
#[derive(Debug)]
struct KeyRing {
    /// Ring buffer of day buckets, oldest at index 0 after roll-over.
    /// Wrapped in `RwLock` so we can rotate the ring (push a new day,
    /// drop the oldest) without blocking concurrent reads of buckets
    /// for the current day, which only need a read lock.
    buckets: RwLock<Vec<Arc<DayBucket>>>,
    window: usize,
}

impl KeyRing {
    fn new(window: usize) -> Self {
        Self {
            buckets: RwLock::new(Vec::with_capacity(window)),
            window,
        }
    }

    /// Increment the counter for `today`. If the ring's most recent
    /// bucket is for an older date, push a new bucket and evict the
    /// oldest if the window is full.
    fn record(&self, today: NaiveDate) {
        // Fast path: the latest bucket already matches today — bump the
        // atomic without touching the lock structure.
        {
            let read = self.buckets.read();
            if let Some(last) = read.last() {
                if last.date == today {
                    last.count.fetch_add(1, Ordering::Relaxed);
                    return;
                }
            }
        }
        // Slow path: rotate the ring under a write lock.
        let mut write = self.buckets.write();
        if let Some(last) = write.last() {
            if last.date == today {
                last.count.fetch_add(1, Ordering::Relaxed);
                return;
            }
        }
        if write.len() >= self.window {
            write.remove(0);
        }
        write.push(Arc::new(DayBucket {
            date: today,
            count: AtomicU64::new(1),
        }));
    }

    /// Snapshot the last `n` daily buckets, ascending by date. Missing
    /// days inside the window appear as `count = 0` so the consumer can
    /// render a continuous sparkline without gap-fill logic.
    fn snapshot(&self, today: NaiveDate, n: usize) -> Vec<UsageBucket> {
        let n = n.min(self.window);
        if n == 0 {
            return Vec::new();
        }
        let read = self.buckets.read();
        let mut by_date: std::collections::HashMap<NaiveDate, u64> =
            std::collections::HashMap::with_capacity(read.len());
        for bucket in read.iter() {
            by_date.insert(bucket.date, bucket.count.load(Ordering::Relaxed));
        }
        let mut out = Vec::with_capacity(n);
        for offset in (0..n).rev() {
            let date = today - ChronoDuration::days(offset as i64);
            let count = by_date.get(&date).copied().unwrap_or(0);
            out.push(UsageBucket {
                date: date.to_string(),
                count,
            });
        }
        out
    }
}

/// One day's bucket exposed to API consumers.
#[derive(Debug, Clone, Serialize)]
pub struct UsageBucket {
    /// ISO-8601 date (UTC), e.g. "2026-05-03".
    pub date: String,
    /// Successful validations recorded for that day.
    pub count: u64,
}

/// Registry of per-key rings.
#[derive(Debug)]
pub struct ApiKeyUsageRecorder {
    rings: Arc<DashMap<String, Arc<KeyRing>>>,
    window: usize,
}

impl ApiKeyUsageRecorder {
    /// Create a new recorder with the default 30-day window.
    pub fn new() -> Self {
        Self::with_window(DEFAULT_WINDOW_DAYS)
    }

    /// Create with a custom window (in days). Used in tests to compress
    /// the rotation surface.
    pub fn with_window(window: usize) -> Self {
        Self {
            rings: Arc::new(DashMap::new()),
            window: window.max(1),
        }
    }

    /// Record one successful validation against `key_id`.
    pub fn record(&self, key_id: &str) {
        self.record_at(key_id, Utc::now().date_naive());
    }

    /// Test-friendly variant that lets the caller fix "today".
    pub fn record_at(&self, key_id: &str, day: NaiveDate) {
        let ring = self
            .rings
            .entry(key_id.to_string())
            .or_insert_with(|| Arc::new(KeyRing::new(self.window)))
            .clone();
        ring.record(day);
    }

    /// Snapshot the last `days` buckets for `key_id`. Missing days
    /// appear with `count = 0` so the SPA renders a continuous
    /// sparkline.
    pub fn snapshot(&self, key_id: &str, days: usize) -> Vec<UsageBucket> {
        self.snapshot_at(key_id, Utc::now().date_naive(), days)
    }

    /// Test-friendly variant.
    pub fn snapshot_at(&self, key_id: &str, today: NaiveDate, days: usize) -> Vec<UsageBucket> {
        match self.rings.get(key_id) {
            Some(ring) => ring.snapshot(today, days),
            None => {
                // No ring yet — synthesize an all-zero window so the
                // consumer doesn't have to special-case "no usage".
                let mut out = Vec::with_capacity(days);
                for offset in (0..days).rev() {
                    let date = today - ChronoDuration::days(offset as i64);
                    out.push(UsageBucket {
                        date: date.to_string(),
                        count: 0,
                    });
                }
                out
            }
        }
    }

    /// Total number of recorded events for `key_id` across the entire
    /// retained window. Used to back the dashboard's "Total" column
    /// without requiring the caller to sum a snapshot client-side.
    pub fn total(&self, key_id: &str) -> u64 {
        match self.rings.get(key_id) {
            Some(ring) => ring
                .buckets
                .read()
                .iter()
                .map(|b| b.count.load(Ordering::Relaxed))
                .sum(),
            None => 0,
        }
    }

    /// Drop the ring for `key_id` (called when a key is deleted).
    pub fn forget(&self, key_id: &str) {
        self.rings.remove(key_id);
    }
}

impl Default for ApiKeyUsageRecorder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;

    fn day(s: &str) -> NaiveDate {
        NaiveDate::parse_from_str(s, "%Y-%m-%d").unwrap()
    }

    #[test]
    fn snapshot_returns_continuous_window_with_zeros() {
        let r = ApiKeyUsageRecorder::with_window(7);
        r.record_at("k1", day("2026-05-01"));
        r.record_at("k1", day("2026-05-01"));
        r.record_at("k1", day("2026-05-03"));

        let snap = r.snapshot_at("k1", day("2026-05-03"), 3);
        assert_eq!(snap.len(), 3);
        assert_eq!(snap[0].date, "2026-05-01");
        assert_eq!(snap[0].count, 2);
        assert_eq!(snap[1].date, "2026-05-02");
        assert_eq!(snap[1].count, 0);
        assert_eq!(snap[2].date, "2026-05-03");
        assert_eq!(snap[2].count, 1);
    }

    #[test]
    fn ring_evicts_oldest_beyond_window() {
        let r = ApiKeyUsageRecorder::with_window(2);
        r.record_at("k", day("2026-05-01"));
        r.record_at("k", day("2026-05-02"));
        r.record_at("k", day("2026-05-03"));

        let snap = r.snapshot_at("k", day("2026-05-03"), 2);
        assert_eq!(snap.len(), 2);
        assert_eq!(snap[0].date, "2026-05-02");
        assert_eq!(snap[1].date, "2026-05-03");
    }

    #[test]
    fn unknown_key_returns_zero_window() {
        let r = ApiKeyUsageRecorder::with_window(7);
        let snap = r.snapshot_at("missing", day("2026-05-03"), 3);
        assert_eq!(snap.len(), 3);
        assert!(snap.iter().all(|b| b.count == 0));
        assert_eq!(r.total("missing"), 0);
    }

    #[test]
    fn total_sums_all_retained_buckets() {
        let r = ApiKeyUsageRecorder::with_window(7);
        for _ in 0..10 {
            r.record_at("k", day("2026-05-01"));
        }
        for _ in 0..5 {
            r.record_at("k", day("2026-05-03"));
        }
        assert_eq!(r.total("k"), 15);
    }

    #[test]
    fn fifty_records_across_two_days_yield_correct_aggregates() {
        let r = ApiKeyUsageRecorder::with_window(7);
        for _ in 0..30 {
            r.record_at("k", day("2026-05-02"));
        }
        for _ in 0..20 {
            r.record_at("k", day("2026-05-03"));
        }
        let snap = r.snapshot_at("k", day("2026-05-03"), 2);
        assert_eq!(snap.len(), 2);
        assert_eq!(snap[0].count, 30);
        assert_eq!(snap[1].count, 20);
        assert_eq!(r.total("k"), 50);
    }

    #[test]
    fn forget_removes_ring() {
        let r = ApiKeyUsageRecorder::with_window(7);
        r.record_at("k", day("2026-05-01"));
        r.forget("k");
        assert_eq!(r.total("k"), 0);
    }
}
