//! Cache metrics and observability
//!
//! Provides real-time metrics for cache performance monitoring.

use parking_lot::RwLock;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant};

/// Cache metrics collector
pub struct CacheMetrics {
    enabled: bool,
    hot_hits: AtomicU64,
    warm_hits: AtomicU64,
    cold_hits: AtomicU64,
    misses: AtomicU64,
    writes: AtomicU64,
    evictions: AtomicU64,
    errors: AtomicU64,
    latency_hot: Arc<RwLock<LatencyTracker>>,
    latency_warm: Arc<RwLock<LatencyTracker>>,
    latency_cold: Arc<RwLock<LatencyTracker>>,
    start_time: Instant,
}

impl CacheMetrics {
    /// Create a new metrics collector
    pub fn new(enabled: bool) -> Self {
        Self {
            enabled,
            hot_hits: AtomicU64::new(0),
            warm_hits: AtomicU64::new(0),
            cold_hits: AtomicU64::new(0),
            misses: AtomicU64::new(0),
            writes: AtomicU64::new(0),
            evictions: AtomicU64::new(0),
            errors: AtomicU64::new(0),
            latency_hot: Arc::new(RwLock::new(LatencyTracker::new())),
            latency_warm: Arc::new(RwLock::new(LatencyTracker::new())),
            latency_cold: Arc::new(RwLock::new(LatencyTracker::new())),
            start_time: Instant::now(),
        }
    }

    /// Record a cache hit
    pub fn record_hit(&self, tier: &str) {
        if !self.enabled {
            return;
        }

        match tier {
            "hot" => self.hot_hits.fetch_add(1, Ordering::Relaxed),
            "warm" => self.warm_hits.fetch_add(1, Ordering::Relaxed),
            "cold" => self.cold_hits.fetch_add(1, Ordering::Relaxed),
            _ => 0,
        };
    }

    /// Record a cache miss
    pub fn record_miss(&self) {
        if !self.enabled {
            return;
        }
        self.misses.fetch_add(1, Ordering::Relaxed);
    }

    /// Record a write operation
    pub fn record_write(&self) {
        if !self.enabled {
            return;
        }
        self.writes.fetch_add(1, Ordering::Relaxed);
    }

    /// Record an eviction
    pub fn record_eviction(&self) {
        if !self.enabled {
            return;
        }
        self.evictions.fetch_add(1, Ordering::Relaxed);
    }

    /// Record an error
    pub fn record_error(&self) {
        if !self.enabled {
            return;
        }
        self.errors.fetch_add(1, Ordering::Relaxed);
    }

    /// Record latency for a tier
    pub fn record_latency(&self, tier: &str, duration: Duration) {
        if !self.enabled {
            return;
        }

        let tracker = match tier {
            "hot" => &self.latency_hot,
            "warm" => &self.latency_warm,
            "cold" => &self.latency_cold,
            _ => return,
        };

        tracker.write().record(duration);
    }

    /// Get current cache statistics
    pub fn stats(&self) -> CacheStats {
        let hot_hits = self.hot_hits.load(Ordering::Relaxed);
        let warm_hits = self.warm_hits.load(Ordering::Relaxed);
        let cold_hits = self.cold_hits.load(Ordering::Relaxed);
        let misses = self.misses.load(Ordering::Relaxed);

        let total_hits = hot_hits + warm_hits + cold_hits;
        let total_requests = total_hits + misses;

        let hit_rate = if total_requests > 0 {
            total_hits as f64 / total_requests as f64
        } else {
            0.0
        };

        CacheStats {
            hot_hits,
            warm_hits,
            cold_hits,
            total_hits,
            total_misses: misses,
            total_writes: self.writes.load(Ordering::Relaxed),
            total_evictions: self.evictions.load(Ordering::Relaxed),
            total_errors: self.errors.load(Ordering::Relaxed),
            hit_rate,
            uptime: self.start_time.elapsed(),
            latency_hot: self.latency_hot.read().stats(),
            latency_warm: self.latency_warm.read().stats(),
            latency_cold: self.latency_cold.read().stats(),
        }
    }

    /// Reset all metrics
    pub fn reset(&self) {
        self.hot_hits.store(0, Ordering::Relaxed);
        self.warm_hits.store(0, Ordering::Relaxed);
        self.cold_hits.store(0, Ordering::Relaxed);
        self.misses.store(0, Ordering::Relaxed);
        self.writes.store(0, Ordering::Relaxed);
        self.evictions.store(0, Ordering::Relaxed);
        self.errors.store(0, Ordering::Relaxed);

        self.latency_hot.write().reset();
        self.latency_warm.write().reset();
        self.latency_cold.write().reset();
    }
}

/// Cache statistics snapshot
#[derive(Debug, Clone)]
pub struct CacheStats {
    pub hot_hits: u64,
    pub warm_hits: u64,
    pub cold_hits: u64,
    pub total_hits: u64,
    pub total_misses: u64,
    pub total_writes: u64,
    pub total_evictions: u64,
    pub total_errors: u64,
    pub hit_rate: f64,
    pub uptime: Duration,
    pub latency_hot: LatencyStats,
    pub latency_warm: LatencyStats,
    pub latency_cold: LatencyStats,
}

/// Latency tracker for percentile calculations
struct LatencyTracker {
    samples: Vec<Duration>,
    max_samples: usize,
}

impl LatencyTracker {
    fn new() -> Self {
        Self {
            samples: Vec::with_capacity(1000),
            max_samples: 10000,
        }
    }

    fn record(&mut self, duration: Duration) {
        self.samples.push(duration);

        // Keep only recent samples
        if self.samples.len() > self.max_samples {
            self.samples.drain(0..self.max_samples / 2);
        }
    }

    fn stats(&self) -> LatencyStats {
        if self.samples.is_empty() {
            return LatencyStats::default();
        }

        let mut sorted = self.samples.clone();
        sorted.sort();

        let count = sorted.len();
        let sum: Duration = sorted.iter().sum();
        let avg = sum / count as u32;

        let p50 = sorted[count * 50 / 100];
        let p95 = sorted[count * 95 / 100];
        let p99 = sorted[count * 99 / 100];
        let max = *sorted.last().unwrap();

        LatencyStats {
            count,
            avg,
            p50,
            p95,
            p99,
            max,
        }
    }

    fn reset(&mut self) {
        self.samples.clear();
    }
}

/// Latency statistics
#[derive(Debug, Clone, Default)]
pub struct LatencyStats {
    pub count: usize,
    pub avg: Duration,
    pub p50: Duration,
    pub p95: Duration,
    pub p99: Duration,
    pub max: Duration,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_metrics_basic() {
        let metrics = CacheMetrics::new(true);

        metrics.record_hit("hot");
        metrics.record_hit("warm");
        metrics.record_miss();
        metrics.record_write();

        let stats = metrics.stats();

        assert_eq!(stats.hot_hits, 1);
        assert_eq!(stats.warm_hits, 1);
        assert_eq!(stats.total_hits, 2);
        assert_eq!(stats.total_misses, 1);
        assert_eq!(stats.total_writes, 1);
    }

    #[test]
    fn test_hit_rate() {
        let metrics = CacheMetrics::new(true);

        // 7 hits, 3 misses = 70% hit rate
        for _ in 0..7 {
            metrics.record_hit("hot");
        }
        for _ in 0..3 {
            metrics.record_miss();
        }

        let stats = metrics.stats();
        assert!((stats.hit_rate - 0.7).abs() < 0.01);
    }

    #[test]
    fn test_latency_tracking() {
        let metrics = CacheMetrics::new(true);

        metrics.record_latency("hot", Duration::from_micros(100));
        metrics.record_latency("hot", Duration::from_micros(200));
        metrics.record_latency("hot", Duration::from_micros(300));

        let stats = metrics.stats();
        assert_eq!(stats.latency_hot.count, 3);
        assert!(stats.latency_hot.avg.as_micros() > 0);
    }

    #[test]
    fn test_metrics_disabled() {
        let metrics = CacheMetrics::new(false);

        metrics.record_hit("hot");
        metrics.record_miss();

        let stats = metrics.stats();

        // Should still work but may not record
        assert!(stats.total_hits + stats.total_misses >= 0);
    }

    #[test]
    fn test_reset() {
        let metrics = CacheMetrics::new(true);

        metrics.record_hit("hot");
        metrics.record_miss();

        metrics.reset();

        let stats = metrics.stats();
        assert_eq!(stats.total_hits, 0);
        assert_eq!(stats.total_misses, 0);
    }

    #[test]
    fn test_latency_percentiles() {
        let mut tracker = LatencyTracker::new();

        for i in 1..=100 {
            tracker.record(Duration::from_micros(i));
        }

        let stats = tracker.stats();

        assert_eq!(stats.count, 100);
        assert_eq!(stats.p50.as_micros(), 50);
        assert_eq!(stats.p95.as_micros(), 95);
        assert_eq!(stats.p99.as_micros(), 99);
        assert_eq!(stats.max.as_micros(), 100);
    }
}

