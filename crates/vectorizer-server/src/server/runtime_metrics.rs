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
    /// 30 s collections snapshot (phase30) — also pushed immediately on
    /// create / delete / rename so the dashboard table reflects
    /// mutations without waiting for the next tick.
    Collections(CollectionsSnapshot),
    /// One log entry per new line tailed from the active log file
    /// (phase30). Admin-only — the WS upgrade route already runs
    /// behind the admin gate so subscribers are filtered before they
    /// reach this bus. Renamed to `logs` on the wire so the topic
    /// frame matches `Topic::Logs`.
    #[serde(rename = "logs")]
    Log(LogEntry),
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

/// Slim per-collection summary carried on the WS `collections` topic
/// (phase30). Just enough for the dashboard table without re-shipping
/// the full `GET /collections/{name}` metadata payload.
#[derive(Debug, Clone, Serialize, Default)]
pub struct CollectionSummary {
    /// Collection name (unique within the store).
    pub name: String,
    /// Number of vectors currently in the collection.
    pub vector_count: usize,
    /// Embedding dimension.
    pub dimension: usize,
}

/// Build a `CollectionsSnapshot` from the live store. Shared by the
/// 30 s tick task in `bootstrap.rs` and the collection-mutation hooks
/// in `rest_handlers/collections.rs` so both surfaces emit the same
/// shape. Names are sorted alphabetically to match the REST list
/// ordering. Collections that fail `get_collection` (e.g. a rename
/// race) are skipped silently.
pub fn build_collections_snapshot(store: &vectorizer::VectorStore) -> CollectionsSnapshot {
    let mut names = store.list_collections();
    names.sort();
    let mut collections = Vec::with_capacity(names.len());
    for name in names {
        if let Ok(coll) = store.get_collection(&name) {
            collections.push(CollectionSummary {
                name: name.clone(),
                vector_count: coll.vector_count(),
                dimension: coll.config().dimension,
            });
        }
    }
    CollectionsSnapshot { collections }
}

/// Snapshot frame for `Topic::Collections`.
#[derive(Debug, Clone, Serialize, Default)]
pub struct CollectionsSnapshot {
    /// All collections visible to the (admin) subscriber, sorted by
    /// name to match the REST response shape.
    pub collections: Vec<CollectionSummary>,
}

/// Single log line tailed from the active log file (phase30). Mirrors
/// the row shape the `GET /logs` REST handler emits so the dashboard
/// can use the same renderer for both paths.
#[derive(Debug, Clone, Serialize, Default)]
pub struct LogEntry {
    /// RFC-3339 timestamp (UTC) at the moment the line was tailed.
    pub timestamp: String,
    /// Best-effort level extracted from the line: `ERROR`, `WARN`,
    /// `INFO`, `DEBUG`. Falls back to `INFO` when the line has no
    /// recognisable marker.
    pub level: String,
    /// The raw log line.
    pub message: String,
    /// Origin tag — always `"vectorizer"` for now (phase30).
    pub source: String,
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
// LogTailer
// ---------------------------------------------------------------------------

/// Tails the active log file (resolved via [`crate::logging::get_logs_dir`])
/// and yields one [`LogEntry`] per new line. The tailer polls — typically
/// every 500 ms — instead of using filesystem watchers so it works
/// uniformly on every OS the server runs on (Linux, macOS, Windows,
/// Docker overlayfs).
///
/// State across polls:
///
/// - `current_path`: the most-recently-tailed file. The tailer
///   re-resolves the active file on every poll, so when the daily log
///   rotation creates `vectorizer-<NEW_DATE>.log` it follows the new
///   file without restart.
/// - `offset`: byte offset within `current_path` from which the next
///   poll resumes reading.
/// - `partial`: a tail-end byte sequence that didn't yet end in `\n`.
///   Held over so we never emit a truncated line.
///
/// On rotation (current path changes), `offset` and `partial` reset to
/// 0 / empty so the new file is read from the start.
#[derive(Debug, Default)]
pub struct LogTailer {
    current_path: Option<std::path::PathBuf>,
    offset: u64,
    partial: String,
}

impl LogTailer {
    /// Drain any new lines from the active log file and return them as
    /// `LogEntry` values. Returns an empty `Vec` when the logs
    /// directory is empty or unreadable.
    pub fn poll(&mut self) -> Vec<LogEntry> {
        let active = match resolve_active_log_file() {
            Some(p) => p,
            None => return Vec::new(),
        };
        self.poll_path(&active)
    }

    /// Drain any new lines from the given file path. Test seam — the
    /// production [`poll`](Self::poll) call resolves the active file
    /// via [`crate::logging::get_logs_dir`] then forwards here.
    pub fn poll_path(&mut self, active: &std::path::Path) -> Vec<LogEntry> {
        // Rotation: drop accumulated state and read the new file from
        // the top.
        if self.current_path.as_deref() != Some(active) {
            self.current_path = Some(active.to_path_buf());
            self.offset = 0;
            self.partial.clear();
        }

        let metadata = match std::fs::metadata(active) {
            Ok(m) => m,
            Err(_) => return Vec::new(),
        };
        let file_len = metadata.len();

        // Truncation (or unexpected shrink): re-read from the top.
        if file_len < self.offset {
            self.offset = 0;
            self.partial.clear();
        }
        if file_len == self.offset {
            return Vec::new();
        }

        let new_bytes = match read_range(active, self.offset, file_len) {
            Ok(b) => b,
            Err(_) => return Vec::new(),
        };
        self.offset = file_len;

        let text = String::from_utf8_lossy(&new_bytes);
        let mut buf = std::mem::take(&mut self.partial);
        buf.push_str(&text);

        let mut out = Vec::new();
        let mut start = 0usize;
        let bytes = buf.as_bytes();
        for (i, &b) in bytes.iter().enumerate() {
            if b == b'\n' {
                let line = &buf[start..i];
                let trimmed = line.trim_end_matches('\r');
                if !trimmed.trim().is_empty() {
                    out.push(parse_log_line(trimmed));
                }
                start = i + 1;
            }
        }
        // Anything past the last `\n` is partial — keep it for the
        // next poll so split lines never produce truncated frames.
        self.partial = buf[start..].to_string();
        out
    }
}

/// Resolve the most-recently-modified `.log` file in the canonical logs
/// directory. Returns `None` when the directory is missing or contains
/// no `.log` files.
fn resolve_active_log_file() -> Option<std::path::PathBuf> {
    let dir = crate::logging::get_logs_dir();
    let entries = std::fs::read_dir(&dir).ok()?;
    let mut newest: Option<(std::time::SystemTime, std::path::PathBuf)> = None;
    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().and_then(|s| s.to_str()) != Some("log") {
            continue;
        }
        let modified = entry
            .metadata()
            .ok()
            .and_then(|m| m.modified().ok())
            .unwrap_or(std::time::UNIX_EPOCH);
        if newest.as_ref().is_none_or(|(t, _)| modified > *t) {
            newest = Some((modified, path));
        }
    }
    newest.map(|(_, p)| p)
}

fn read_range(path: &std::path::Path, from: u64, to: u64) -> std::io::Result<Vec<u8>> {
    use std::io::{Read, Seek, SeekFrom};
    let mut file = std::fs::File::open(path)?;
    if from > 0 {
        file.seek(SeekFrom::Start(from))?;
    }
    let len = to.saturating_sub(from) as usize;
    let mut buf = vec![0u8; len];
    file.read_exact(&mut buf)?;
    Ok(buf)
}

fn parse_log_line(line: &str) -> LogEntry {
    let upper = line.to_ascii_uppercase();
    let level = if upper.contains("ERROR") {
        "ERROR"
    } else if upper.contains("WARN") {
        "WARN"
    } else if upper.contains("INFO") {
        "INFO"
    } else if upper.contains("DEBUG") {
        "DEBUG"
    } else {
        "INFO"
    };
    LogEntry {
        timestamp: chrono::Utc::now().to_rfc3339(),
        level: level.to_string(),
        message: line.to_string(),
        source: "vectorizer".to_string(),
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
    fn parse_log_line_picks_levels() {
        use super::parse_log_line;
        assert_eq!(parse_log_line("2026-05-04 ERROR boom").level, "ERROR");
        assert_eq!(parse_log_line("2026-05-04 WARN hot").level, "WARN");
        assert_eq!(parse_log_line("2026-05-04 INFO ready").level, "INFO");
        assert_eq!(parse_log_line("2026-05-04 DEBUG trace").level, "DEBUG");
        // Unknown markers fall back to INFO.
        assert_eq!(parse_log_line("plain line, no level").level, "INFO");
        // Lower-case markers count too.
        assert_eq!(parse_log_line("2026-05-04 error nope").level, "ERROR");
    }

    #[test]
    fn parse_log_line_preserves_message_and_source() {
        use super::parse_log_line;
        let line = "2026-05-04T10:00:00Z INFO server: ready on :15002";
        let entry = parse_log_line(line);
        assert_eq!(entry.message, line);
        assert_eq!(entry.source, "vectorizer");
        // `timestamp` is generated at parse time, so just check shape.
        assert!(!entry.timestamp.is_empty());
    }

    #[test]
    fn log_tailer_emits_one_entry_per_complete_line() {
        use std::io::Write;

        use super::LogTailer;

        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("vectorizer-test.log");
        std::fs::write(&path, b"2026-05-04 INFO first\n2026-05-04 ERROR second\n").unwrap();

        let mut tailer = LogTailer::default();
        let entries = tailer.poll_path(&path);
        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].level, "INFO");
        assert_eq!(entries[0].message, "2026-05-04 INFO first");
        assert_eq!(entries[1].level, "ERROR");
        assert_eq!(entries[1].message, "2026-05-04 ERROR second");

        // A second poll with no new bytes returns nothing.
        let entries = tailer.poll_path(&path);
        assert!(entries.is_empty());

        // Append more lines — only the appended ones come back.
        let mut f = std::fs::OpenOptions::new()
            .append(true)
            .open(&path)
            .unwrap();
        writeln!(f, "2026-05-04 WARN third").unwrap();
        let entries = tailer.poll_path(&path);
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].level, "WARN");
        assert_eq!(entries[0].message, "2026-05-04 WARN third");
    }

    #[test]
    fn log_tailer_buffers_partial_line_until_newline_arrives() {
        use std::io::Write;

        use super::LogTailer;

        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("vectorizer-test.log");
        std::fs::write(&path, b"2026-05-04 INFO begin").unwrap(); // no \n

        let mut tailer = LogTailer::default();
        let entries = tailer.poll_path(&path);
        // Partial line — held back until the trailing newline arrives.
        assert!(entries.is_empty());

        let mut f = std::fs::OpenOptions::new()
            .append(true)
            .open(&path)
            .unwrap();
        f.write_all(b" finished\n").unwrap();
        let entries = tailer.poll_path(&path);
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].message, "2026-05-04 INFO begin finished");
    }

    #[test]
    fn log_tailer_handles_truncation_by_rereading_from_top() {
        use super::LogTailer;

        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("vectorizer-test.log");
        // Long original content so the post-truncate file is strictly
        // shorter than the recorded offset.
        std::fs::write(
            &path,
            b"2026-05-04 INFO original-long-line-that-takes-some-bytes\n",
        )
        .unwrap();
        let mut tailer = LogTailer::default();
        let _ = tailer.poll_path(&path); // primes offset

        // Rewrite the file with shorter contents — simulates rotation
        // by truncate.
        std::fs::write(&path, b"2026-05-04 INFO fresh\n").unwrap();
        let entries = tailer.poll_path(&path);
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].message, "2026-05-04 INFO fresh");
    }

    #[test]
    fn log_tailer_resets_on_path_rotation() {
        use super::LogTailer;

        let dir = tempfile::tempdir().unwrap();
        let p1 = dir.path().join("vectorizer-2026-05-03.log");
        let p2 = dir.path().join("vectorizer-2026-05-04.log");
        std::fs::write(&p1, b"2026-05-03 INFO yesterday\n").unwrap();
        std::fs::write(&p2, b"2026-05-04 INFO today\n").unwrap();

        let mut tailer = LogTailer::default();
        let entries = tailer.poll_path(&p1);
        assert_eq!(entries.len(), 1);
        // Switching path rebases the offset so the entirety of p2 is read.
        let entries = tailer.poll_path(&p2);
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].message, "2026-05-04 INFO today");
    }

    #[tokio::test]
    async fn collections_snapshot_round_trips_through_broadcast_bus() {
        use tokio::sync::broadcast;
        use vectorizer::VectorStore;
        use vectorizer::models::CollectionConfig;

        use super::{DashboardEvent, build_collections_snapshot};

        // Build a tiny store with one collection so the snapshot is
        // non-empty.
        let store = VectorStore::new();
        let cfg = CollectionConfig {
            dimension: 4,
            ..CollectionConfig::default()
        };
        store.create_collection("docs", cfg).unwrap();

        let (tx, mut rx) = broadcast::channel::<DashboardEvent>(8);
        let snap = build_collections_snapshot(&store);
        tx.send(DashboardEvent::Collections(snap)).unwrap();

        // The receiver should see exactly the event we sent, with the
        // expected collection summary.
        let frame = rx.recv().await.unwrap();
        match frame {
            DashboardEvent::Collections(s) => {
                assert_eq!(s.collections.len(), 1);
                assert_eq!(s.collections[0].name, "docs");
                assert_eq!(s.collections[0].dimension, 4);
            }
            other => panic!("expected Collections, got {other:?}"),
        }
    }

    #[test]
    fn build_collections_snapshot_sorts_alphabetically() {
        use vectorizer::VectorStore;
        use vectorizer::models::CollectionConfig;

        use super::build_collections_snapshot;

        let store = VectorStore::new();
        let cfg = || CollectionConfig {
            dimension: 8,
            ..CollectionConfig::default()
        };
        store.create_collection("zeta", cfg()).unwrap();
        store.create_collection("alpha", cfg()).unwrap();
        store.create_collection("mango", cfg()).unwrap();

        let snap = build_collections_snapshot(&store);
        let names: Vec<_> = snap.collections.iter().map(|c| c.name.clone()).collect();
        assert_eq!(names, vec!["alpha", "mango", "zeta"]);
        // Each summary carries the configured dimension.
        for c in &snap.collections {
            assert_eq!(c.dimension, 8);
            assert_eq!(c.vector_count, 0);
        }
    }

    #[tokio::test]
    async fn log_event_round_trips_through_broadcast_bus() {
        use tokio::sync::broadcast;

        use super::{DashboardEvent, LogEntry};

        let (tx, mut rx) = broadcast::channel::<DashboardEvent>(8);
        let entry = LogEntry {
            timestamp: "2026-05-04T10:00:00Z".to_string(),
            level: "ERROR".to_string(),
            message: "boom".to_string(),
            source: "vectorizer".to_string(),
        };
        tx.send(DashboardEvent::Log(entry)).unwrap();

        match rx.recv().await.unwrap() {
            DashboardEvent::Log(e) => {
                assert_eq!(e.level, "ERROR");
                assert_eq!(e.message, "boom");
            }
            other => panic!("expected Log, got {other:?}"),
        }
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
