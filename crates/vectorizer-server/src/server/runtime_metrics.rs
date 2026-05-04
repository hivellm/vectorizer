//! Runtime metrics sampler (phase25). Collects process-level CPU /
//! memory / connections / WAL / per-route latency every 1s and serves
//! them through `GET /metrics/runtime`.

use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::{Duration, Instant};

use parking_lot::RwLock;
use serde::Serialize;
use sysinfo::{Pid, ProcessRefreshKind, ProcessesToUpdate, System};
use tokio::sync::broadcast;
use tokio::task::JoinHandle;
use vectorizer::replication::{MasterNode, WalSnapshot};

/// Frame published on the dashboard broadcast bus (phase29). The
/// WebSocket multiplexer reads from this bus and forwards frames whose
/// topic is in the connection's subscription set.
#[derive(Debug, Clone, Serialize)]
#[serde(tag = "topic", content = "data", rename_all = "snake_case")]
pub enum DashboardEvent {
    /// 1 Hz runtime snapshot — same shape as `GET /metrics/runtime`.
    Runtime(RuntimeSnapshot),
    /// 5 s server status snapshot — same shape as `GET /status`.
    Status(StatusSnapshot),
}

/// `GET /status` payload, shared by the REST handler and the WS
/// publisher so both surfaces emit the same JSON.
#[derive(Debug, Clone, Serialize, Default)]
pub struct StatusSnapshot {
    /// Whether the server process is currently serving requests.
    pub online: bool,
    /// Server version string (`CARGO_PKG_VERSION`).
    pub version: String,
    /// Seconds since the server process started.
    pub uptime_seconds: u64,
    /// Number of collections currently registered in the store.
    pub collections_count: usize,
}

// ---------------------------------------------------------------------------
// Public snapshot types
// ---------------------------------------------------------------------------

/// Snapshot of process-level and request-level runtime metrics.
#[derive(Debug, Clone, Serialize, Default)]
pub struct RuntimeSnapshot {
    /// CPU usage of this process, 0–100 %.
    pub cpu_percent: f64,
    /// Resident-set size of this process in bytes.
    pub memory_rss_bytes: u64,
    /// Total physical memory of the host in bytes.
    pub memory_total_bytes: u64,
    /// RSS as a fraction of total memory, 0–100 %.
    pub memory_percent: f64,
    /// Number of active HTTP connections right now.
    pub active_connections: usize,
    /// Seconds since the server process started.
    pub uptime_seconds: u64,
    /// Rolling 60-second queries-per-second (all routes combined).
    pub qps_window_60s: f64,
    /// Fraction of requests in the last 60 s that returned 5xx, 0–1.
    pub error_rate_5xx_60s: f64,
    /// Per-route latency breakdown.
    pub throughput_by_route: Vec<RouteStats>,
    /// WAL state (zero-init when replication is disabled).
    pub wal: WalSnapshot,
}

/// Per-route latency and throughput statistics.
#[derive(Debug, Clone, Serialize, Default)]
pub struct RouteStats {
    /// Route path (raw URI path for phase25; templated in a follow-up).
    pub route: String,
    /// Queries per second for this route over the last 60 s.
    pub qps: f64,
    /// 50th-percentile latency in milliseconds.
    pub p50_ms: f64,
    /// 99th-percentile latency in milliseconds.
    pub p99_ms: f64,
}

// ---------------------------------------------------------------------------
// LatencyAggregator
// ---------------------------------------------------------------------------

#[derive(Debug)]
struct Sample {
    at: Instant,
    duration_ms: u32,
    status: u16,
}

/// Rolling 60-second per-route latency aggregator.
pub struct LatencyAggregator {
    inner: RwLock<HashMap<String, VecDeque<Sample>>>,
}

impl LatencyAggregator {
    /// Create a new, empty aggregator.
    pub fn new() -> Self {
        Self {
            inner: RwLock::new(HashMap::new()),
        }
    }

    /// Record a completed request.
    pub fn record(&self, route: &str, duration_ms: u32, status: u16) {
        let now = Instant::now();
        let mut guard = self.inner.write();
        let queue = guard.entry(route.to_string()).or_default();
        queue.push_back(Sample {
            at: now,
            duration_ms,
            status,
        });
        // Prune entries older than 60 s.
        let cutoff = now - Duration::from_secs(60);
        while queue.front().map(|s| s.at < cutoff).unwrap_or(false) {
            queue.pop_front();
        }
    }

    /// Build per-route stats from the rolling 60-second window.
    pub fn snapshot(&self) -> Vec<RouteStats> {
        let guard = self.inner.read();
        let mut out = Vec::with_capacity(guard.len());
        let now = Instant::now();
        let cutoff = now - Duration::from_secs(60);

        for (route, queue) in guard.iter() {
            let mut durations: Vec<u32> = queue
                .iter()
                .filter(|s| s.at >= cutoff)
                .map(|s| s.duration_ms)
                .collect();

            if durations.is_empty() {
                continue;
            }

            durations.sort_unstable();
            let count = durations.len();
            let qps = count as f64 / 60.0;

            let p50_idx = (count as f64 * 0.50) as usize;
            let p99_idx = ((count as f64 * 0.99) as usize).min(count - 1);
            let p50_ms = durations[p50_idx.min(count - 1)] as f64;
            let p99_ms = durations[p99_idx] as f64;

            out.push(RouteStats {
                route: route.clone(),
                qps,
                p50_ms,
                p99_ms,
            });
        }

        // Highest QPS first for the dashboard.
        out.sort_by(|a, b| {
            b.qps
                .partial_cmp(&a.qps)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        out
    }

    /// Fraction of requests (0–1) with HTTP 5xx status in the last 60 s.
    pub fn error_rate(&self) -> f64 {
        let guard = self.inner.read();
        let now = Instant::now();
        let cutoff = now - Duration::from_secs(60);

        let mut total = 0usize;
        let mut errors = 0usize;
        for queue in guard.values() {
            for s in queue.iter().filter(|s| s.at >= cutoff) {
                total += 1;
                if s.status >= 500 {
                    errors += 1;
                }
            }
        }

        if total == 0 {
            0.0
        } else {
            errors as f64 / total as f64
        }
    }

    /// Total request count across all routes in the last 60 s.
    fn total_recent_count(&self) -> usize {
        let guard = self.inner.read();
        let now = Instant::now();
        let cutoff = now - Duration::from_secs(60);
        guard
            .values()
            .flat_map(|q| q.iter())
            .filter(|s| s.at >= cutoff)
            .count()
    }
}

// ---------------------------------------------------------------------------
// RuntimeSampler
// ---------------------------------------------------------------------------

/// Background sampler that refreshes process-level metrics every second and
/// keeps the latest [`RuntimeSnapshot`] available via `snapshot()`.
pub struct RuntimeSampler {
    snapshot: Arc<RwLock<RuntimeSnapshot>>,
    aggregator: Arc<LatencyAggregator>,
    connections: Arc<AtomicUsize>,
    started_at: Instant,
    handle: Option<JoinHandle<()>>,
    master_node: Option<Arc<MasterNode>>,
    /// Optional broadcast sink (phase29). When set, every tick also
    /// emits a `DashboardEvent::Runtime` so WebSocket subscribers can
    /// receive the snapshot without polling REST.
    broadcast: Option<broadcast::Sender<DashboardEvent>>,
}

impl RuntimeSampler {
    /// Create a new, not-yet-started sampler.
    pub fn new() -> Self {
        Self {
            snapshot: Arc::new(RwLock::new(RuntimeSnapshot::default())),
            aggregator: Arc::new(LatencyAggregator::new()),
            connections: Arc::new(AtomicUsize::new(0)),
            started_at: Instant::now(),
            handle: None,
            master_node: None,
            broadcast: None,
        }
    }

    /// Plumb a `MasterNode` into the sampler so each tick can attach a
    /// fresh WAL snapshot. Must be called before [`start`]. When
    /// replication is disabled this stays `None` and the WAL section of
    /// the snapshot is zero-initialised.
    pub fn set_master_node(&mut self, master: Arc<MasterNode>) {
        self.master_node = Some(master);
    }

    /// Plumb a dashboard broadcast bus into the sampler so each tick
    /// publishes a `DashboardEvent::Runtime` for the WebSocket
    /// multiplexer (phase29). Must be called before [`start`].
    pub fn set_broadcast(&mut self, tx: broadcast::Sender<DashboardEvent>) {
        self.broadcast = Some(tx);
    }

    /// Subscribe to the dashboard broadcast bus. Returns `None` when no
    /// bus is wired (the WS handler returns this to the upgrade with a
    /// closed receiver, so subscribers immediately get `Closed`).
    pub fn dashboard_rx(&self) -> broadcast::Receiver<DashboardEvent> {
        match &self.broadcast {
            Some(tx) => tx.subscribe(),
            // No bus wired — return a receiver from a freshly-created,
            // immediately-dropped sender. `recv` returns `Closed` on
            // first poll which the WS loop translates to a clean
            // disconnect.
            None => {
                let (tx, rx) = broadcast::channel(1);
                drop(tx);
                rx
            }
        }
    }

    /// Return a clone of the latest snapshot.
    pub fn snapshot(&self) -> RuntimeSnapshot {
        self.snapshot.read().clone()
    }

    /// Return the shared `LatencyAggregator` so the middleware can record
    /// completed requests.
    pub fn aggregator(&self) -> Arc<LatencyAggregator> {
        self.aggregator.clone()
    }

    /// Return the shared connection counter so the middleware can
    /// increment/decrement it.
    pub fn connections(&self) -> Arc<AtomicUsize> {
        self.connections.clone()
    }

    /// Spawn the 1-second tick task. Must be called inside a Tokio runtime.
    pub fn start(&mut self) {
        let snapshot_arc = self.snapshot.clone();
        let aggregator_arc = self.aggregator.clone();
        let connections_arc = self.connections.clone();
        let started_at = self.started_at;
        let master_node = self.master_node.clone();
        let broadcast_tx = self.broadcast.clone();

        let handle = tokio::spawn(async move {
            let pid = Pid::from(std::process::id() as usize);
            let mut system = System::new();

            loop {
                tokio::time::sleep(Duration::from_secs(1)).await;

                // Refresh only CPU and memory for this process.
                system.refresh_processes_specifics(
                    ProcessesToUpdate::Some(&[pid]),
                    false,
                    ProcessRefreshKind::nothing().with_cpu().with_memory(),
                );
                // Also refresh total memory so we can compute the percent.
                system.refresh_memory();

                let (cpu_percent, memory_rss_bytes) = if let Some(proc) = system.process(pid) {
                    (proc.cpu_usage() as f64, proc.memory())
                } else {
                    (0.0, 0)
                };

                let memory_total_bytes = system.total_memory();
                let memory_percent = if memory_total_bytes > 0 {
                    memory_rss_bytes as f64 / memory_total_bytes as f64 * 100.0
                } else {
                    0.0
                };

                let active_connections = connections_arc.load(Ordering::Relaxed);
                let uptime_seconds = started_at.elapsed().as_secs();

                let route_stats = aggregator_arc.snapshot();
                let total_recent = aggregator_arc.total_recent_count();
                let qps_window_60s = total_recent as f64 / 60.0;
                let error_rate_5xx_60s = aggregator_arc.error_rate();
                let wal = master_node
                    .as_ref()
                    .map(|m| m.wal_snapshot())
                    .unwrap_or_default();

                let new_snap = RuntimeSnapshot {
                    cpu_percent,
                    memory_rss_bytes,
                    memory_total_bytes,
                    memory_percent,
                    active_connections,
                    uptime_seconds,
                    qps_window_60s,
                    error_rate_5xx_60s,
                    throughput_by_route: route_stats,
                    wal,
                };
                {
                    let mut snap = snapshot_arc.write();
                    *snap = new_snap.clone();
                }
                // Phase29: forward to the WS multiplexer if a broadcast
                // bus is wired. `send` only fails when there are no
                // subscribers, which is the normal idle state — drop
                // the error.
                if let Some(tx) = &broadcast_tx {
                    let _ = tx.send(DashboardEvent::Runtime(new_snap));
                }
            }
        });

        self.handle = Some(handle);
    }

    /// Abort the tick task if running.
    pub fn stop(&mut self) {
        if let Some(h) = self.handle.take() {
            h.abort();
        }
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use std::sync::Arc;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::time::Duration;

    use super::LatencyAggregator;

    /// Helper: record `n` samples for `route` with `duration_ms` and status 200.
    fn record_n(agg: &LatencyAggregator, route: &str, duration_ms: u32, n: usize) {
        for _ in 0..n {
            agg.record(route, duration_ms, 200);
        }
    }

    #[test]
    fn latency_aggregator_p50_p99_known_distribution() {
        let agg = LatencyAggregator::new();

        // Record 100 samples: 99 at 10ms, 1 at 100ms.
        // p50 should be ~10, p99 should be ~100.
        record_n(&agg, "/test", 10, 99);
        agg.record("/test", 100, 200);

        let stats = agg.snapshot();
        assert_eq!(stats.len(), 1);
        let s = &stats[0];

        // p50 must be within ±5% of 10 ms
        let p50_expected = 10.0_f64;
        assert!(
            (s.p50_ms - p50_expected).abs() <= p50_expected * 0.05 + 1.0,
            "p50={} expected ~{}",
            s.p50_ms,
            p50_expected
        );

        // p99 must be within ±5% of 100 ms
        let p99_expected = 100.0_f64;
        assert!(
            (s.p99_ms - p99_expected).abs() <= p99_expected * 0.05 + 1.0,
            "p99={} expected ~{}",
            s.p99_ms,
            p99_expected
        );
    }

    #[test]
    fn latency_aggregator_error_rate_5xx() {
        let agg = LatencyAggregator::new();

        // 9 successes + 1 server error = 10% error rate
        for _ in 0..9 {
            agg.record("/api", 5, 200);
        }
        agg.record("/api", 5, 500);

        let rate = agg.error_rate();
        assert!(
            (rate - 0.1).abs() < 0.01,
            "error_rate={} expected ~0.1",
            rate
        );
    }

    #[test]
    fn latency_aggregator_error_rate_zero_when_no_5xx() {
        let agg = LatencyAggregator::new();
        record_n(&agg, "/ok", 1, 20);
        assert_eq!(agg.error_rate(), 0.0);
    }

    #[test]
    fn connection_counter_increment_decrement() {
        let counter = Arc::new(AtomicUsize::new(0));

        // Simulate concurrent increments
        let handles: Vec<_> = (0..8)
            .map(|_| {
                let c = counter.clone();
                std::thread::spawn(move || {
                    c.fetch_add(1, Ordering::Relaxed);
                    std::thread::sleep(Duration::from_millis(1));
                    c.fetch_sub(1, Ordering::Relaxed);
                })
            })
            .collect();

        for h in handles {
            h.join().expect("thread panicked");
        }

        assert_eq!(counter.load(Ordering::Relaxed), 0);
    }
}
