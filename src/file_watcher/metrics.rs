//! Metrics for file watcher operations
//!
//! This module provides metrics structures for monitoring file watcher performance.

use std::collections::HashMap;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU32, AtomicU64, Ordering};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;

/// Timing metrics for file watcher operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimingMetrics {
    pub avg_file_processing_ms: f64,
    pub avg_discovery_ms: f64,
    pub avg_sync_ms: f64,
    pub uptime_seconds: u64,
    pub last_activity: Option<String>,
    pub peak_processing_ms: u64,
}

/// File processing metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileMetrics {
    pub total_files_processed: u64,
    pub files_processed_success: u64,
    pub files_processed_error: u64,
    pub files_skipped: u64,
    pub files_in_progress: u64,
    pub files_discovered: u64,
    pub files_removed: u64,
    pub files_indexed_realtime: u64,
}

/// System resource metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemMetrics {
    pub memory_usage_bytes: u64,
    pub cpu_usage_percent: f64,
    pub thread_count: u32,
    pub active_file_handles: u32,
    pub disk_io_ops_per_sec: u64,
    pub network_io_bytes_per_sec: u64,
}

/// Network and API metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkMetrics {
    pub total_api_requests: u64,
    pub successful_api_requests: u64,
    pub failed_api_requests: u64,
    pub avg_api_response_ms: f64,
    pub peak_api_response_ms: u64,
    pub active_connections: u32,
}

/// Status and error metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatusMetrics {
    pub total_errors: u64,
    pub errors_by_type: HashMap<String, u64>,
    pub current_status: String,
    pub last_error: Option<String>,
    pub health_score: u8,
    pub restart_count: u32,
}

/// Collection-specific metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollectionMetrics {
    pub total_vectors: u64,
    pub vectors_added: u64,
    pub vectors_removed: u64,
    pub vectors_updated: u64,
    pub last_update: Option<String>,
}

/// Comprehensive file watcher metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileWatcherMetrics {
    pub timing: TimingMetrics,
    pub files: FileMetrics,
    pub system: SystemMetrics,
    pub network: NetworkMetrics,
    pub status: StatusMetrics,
    pub collections: HashMap<String, CollectionMetrics>,
}

/// Internal metrics storage using atomics for thread-safe access
#[derive(Debug)]
struct InternalMetrics {
    // File processing metrics
    total_files_processed: AtomicU64,
    files_processed_success: AtomicU64,
    files_processed_error: AtomicU64,
    files_skipped: AtomicU64,
    files_in_progress: AtomicU64,
    files_discovered: AtomicU64,
    files_removed: AtomicU64,
    files_indexed_realtime: AtomicU64,

    // Timing metrics
    total_processing_time_ms: AtomicU64,
    total_discovery_time_ms: AtomicU64,
    total_sync_time_ms: AtomicU64,
    peak_processing_ms: AtomicU64,

    // System metrics
    total_errors: AtomicU64,
    restart_count: AtomicU32,

    // Network metrics
    total_api_requests: AtomicU64,
    successful_api_requests: AtomicU64,
    failed_api_requests: AtomicU64,
    total_api_response_time_ms: AtomicU64,
    peak_api_response_ms: AtomicU64,
    active_connections: AtomicU32,

    // I/O metrics
    disk_io_ops: AtomicU64,
    network_io_bytes: AtomicU64,

    // Status
    start_time: Instant,
    is_running: AtomicBool,
}

/// Metrics collector for file watcher operations
#[derive(Debug)]
pub struct MetricsCollector {
    metrics: Arc<InternalMetrics>,
    errors_by_type: Arc<RwLock<HashMap<String, u64>>>,
    collections: Arc<RwLock<HashMap<String, CollectionMetrics>>>,
    last_activity: Arc<RwLock<Option<Instant>>>,
    last_error: Arc<RwLock<Option<String>>>,
}

impl InternalMetrics {
    fn new() -> Self {
        Self {
            total_files_processed: AtomicU64::new(0),
            files_processed_success: AtomicU64::new(0),
            files_processed_error: AtomicU64::new(0),
            files_skipped: AtomicU64::new(0),
            files_in_progress: AtomicU64::new(0),
            files_discovered: AtomicU64::new(0),
            files_removed: AtomicU64::new(0),
            files_indexed_realtime: AtomicU64::new(0),
            total_processing_time_ms: AtomicU64::new(0),
            total_discovery_time_ms: AtomicU64::new(0),
            total_sync_time_ms: AtomicU64::new(0),
            peak_processing_ms: AtomicU64::new(0),
            total_errors: AtomicU64::new(0),
            restart_count: AtomicU32::new(0),
            total_api_requests: AtomicU64::new(0),
            successful_api_requests: AtomicU64::new(0),
            failed_api_requests: AtomicU64::new(0),
            total_api_response_time_ms: AtomicU64::new(0),
            peak_api_response_ms: AtomicU64::new(0),
            active_connections: AtomicU32::new(0),
            disk_io_ops: AtomicU64::new(0),
            network_io_bytes: AtomicU64::new(0),
            start_time: Instant::now(),
            is_running: AtomicBool::new(true),
        }
    }
}

impl MetricsCollector {
    /// Create a new metrics collector
    pub fn new() -> Self {
        Self {
            metrics: Arc::new(InternalMetrics::new()),
            errors_by_type: Arc::new(RwLock::new(HashMap::new())),
            collections: Arc::new(RwLock::new(HashMap::new())),
            last_activity: Arc::new(RwLock::new(None)),
            last_error: Arc::new(RwLock::new(None)),
        }
    }

    /// Get current metrics
    pub async fn get_metrics(&self) -> FileWatcherMetrics {
        let total_processed = self.metrics.total_files_processed.load(Ordering::Relaxed);
        let total_processing_time = self
            .metrics
            .total_processing_time_ms
            .load(Ordering::Relaxed);
        let total_discovery_time = self.metrics.total_discovery_time_ms.load(Ordering::Relaxed);
        let total_sync_time = self.metrics.total_sync_time_ms.load(Ordering::Relaxed);
        let total_api_requests = self.metrics.total_api_requests.load(Ordering::Relaxed);
        let total_api_response_time = self
            .metrics
            .total_api_response_time_ms
            .load(Ordering::Relaxed);

        let uptime = self.metrics.start_time.elapsed().as_secs();
        let last_activity = self
            .last_activity
            .read()
            .await
            .map(|t| format!("{}s ago", t.elapsed().as_secs()));

        let errors_by_type = self.errors_by_type.read().await.clone();
        let collections = self.collections.read().await.clone();
        let last_error = self.last_error.read().await.clone();

        let avg_file_processing_ms = if total_processed > 0 {
            total_processing_time as f64 / total_processed as f64
        } else {
            0.0
        };

        let avg_discovery_ms = if self.metrics.files_discovered.load(Ordering::Relaxed) > 0 {
            total_discovery_time as f64
                / self.metrics.files_discovered.load(Ordering::Relaxed) as f64
        } else {
            0.0
        };

        let avg_sync_ms = if self.metrics.files_removed.load(Ordering::Relaxed) > 0 {
            total_sync_time as f64 / self.metrics.files_removed.load(Ordering::Relaxed) as f64
        } else {
            0.0
        };

        let avg_api_response_ms = if total_api_requests > 0 {
            total_api_response_time as f64 / total_api_requests as f64
        } else {
            0.0
        };

        let health_score = self.calculate_health_score().await;
        let current_status = if self.metrics.is_running.load(Ordering::Relaxed) {
            "running"
        } else {
            "stopped"
        };

        FileWatcherMetrics {
            timing: TimingMetrics {
                avg_file_processing_ms,
                avg_discovery_ms,
                avg_sync_ms,
                uptime_seconds: uptime,
                last_activity,
                peak_processing_ms: self.metrics.peak_processing_ms.load(Ordering::Relaxed),
            },
            files: FileMetrics {
                total_files_processed: self.metrics.total_files_processed.load(Ordering::Relaxed),
                files_processed_success: self
                    .metrics
                    .files_processed_success
                    .load(Ordering::Relaxed),
                files_processed_error: self.metrics.files_processed_error.load(Ordering::Relaxed),
                files_skipped: self.metrics.files_skipped.load(Ordering::Relaxed),
                files_in_progress: self.metrics.files_in_progress.load(Ordering::Relaxed),
                files_discovered: self.metrics.files_discovered.load(Ordering::Relaxed),
                files_removed: self.metrics.files_removed.load(Ordering::Relaxed),
                files_indexed_realtime: self.metrics.files_indexed_realtime.load(Ordering::Relaxed),
            },
            system: SystemMetrics {
                memory_usage_bytes: self.get_memory_usage(),
                cpu_usage_percent: self.get_cpu_usage(),
                thread_count: self.get_thread_count(),
                active_file_handles: self.get_active_file_handles(),
                disk_io_ops_per_sec: self.metrics.disk_io_ops.load(Ordering::Relaxed),
                network_io_bytes_per_sec: self.metrics.network_io_bytes.load(Ordering::Relaxed),
            },
            network: NetworkMetrics {
                total_api_requests,
                successful_api_requests: self
                    .metrics
                    .successful_api_requests
                    .load(Ordering::Relaxed),
                failed_api_requests: self.metrics.failed_api_requests.load(Ordering::Relaxed),
                avg_api_response_ms,
                peak_api_response_ms: self.metrics.peak_api_response_ms.load(Ordering::Relaxed),
                active_connections: self.metrics.active_connections.load(Ordering::Relaxed),
            },
            status: StatusMetrics {
                total_errors: self.metrics.total_errors.load(Ordering::Relaxed),
                errors_by_type,
                current_status: current_status.to_string(),
                last_error,
                health_score,
                restart_count: self.metrics.restart_count.load(Ordering::Relaxed),
            },
            collections,
        }
    }

    /// Reset metrics
    pub async fn reset(&self) {
        // Reset all atomic counters
        self.metrics
            .total_files_processed
            .store(0, Ordering::Relaxed);
        self.metrics
            .files_processed_success
            .store(0, Ordering::Relaxed);
        self.metrics
            .files_processed_error
            .store(0, Ordering::Relaxed);
        self.metrics.files_skipped.store(0, Ordering::Relaxed);
        self.metrics.files_in_progress.store(0, Ordering::Relaxed);
        self.metrics.files_discovered.store(0, Ordering::Relaxed);
        self.metrics.files_removed.store(0, Ordering::Relaxed);
        self.metrics
            .files_indexed_realtime
            .store(0, Ordering::Relaxed);
        self.metrics
            .total_processing_time_ms
            .store(0, Ordering::Relaxed);
        self.metrics
            .total_discovery_time_ms
            .store(0, Ordering::Relaxed);
        self.metrics.total_sync_time_ms.store(0, Ordering::Relaxed);
        self.metrics.peak_processing_ms.store(0, Ordering::Relaxed);
        self.metrics.total_errors.store(0, Ordering::Relaxed);
        self.metrics.total_api_requests.store(0, Ordering::Relaxed);
        self.metrics
            .successful_api_requests
            .store(0, Ordering::Relaxed);
        self.metrics.failed_api_requests.store(0, Ordering::Relaxed);
        self.metrics
            .total_api_response_time_ms
            .store(0, Ordering::Relaxed);
        self.metrics
            .peak_api_response_ms
            .store(0, Ordering::Relaxed);
        self.metrics.active_connections.store(0, Ordering::Relaxed);
        self.metrics.disk_io_ops.store(0, Ordering::Relaxed);
        self.metrics.network_io_bytes.store(0, Ordering::Relaxed);

        // Reset collections and errors
        self.errors_by_type.write().await.clear();
        self.collections.write().await.clear();
        *self.last_activity.write().await = None;
        *self.last_error.write().await = None;
    }

    /// Get summary of metrics
    pub async fn get_summary(&self) -> String {
        let metrics = self.get_metrics().await;
        format!(
            "Files processed: {}, Success: {}, Errors: {}, Uptime: {}s",
            metrics.files.total_files_processed,
            metrics.files.files_processed_success,
            metrics.files.files_processed_error,
            metrics.timing.uptime_seconds
        )
    }

    /// Record file processing completion
    pub async fn record_file_processing_complete(&self, success: bool, processing_time_ms: f64) {
        self.metrics
            .total_files_processed
            .fetch_add(1, Ordering::Relaxed);
        self.metrics
            .total_processing_time_ms
            .fetch_add(processing_time_ms as u64, Ordering::Relaxed);

        if success {
            self.metrics
                .files_processed_success
                .fetch_add(1, Ordering::Relaxed);
        } else {
            self.metrics
                .files_processed_error
                .fetch_add(1, Ordering::Relaxed);
            self.metrics.total_errors.fetch_add(1, Ordering::Relaxed);
        }

        // Update peak processing time
        let current_peak = self.metrics.peak_processing_ms.load(Ordering::Relaxed);
        if processing_time_ms as u64 > current_peak {
            self.metrics
                .peak_processing_ms
                .store(processing_time_ms as u64, Ordering::Relaxed);
        }

        // Update last activity
        *self.last_activity.write().await = Some(Instant::now());
    }

    /// Record discovery operation
    pub async fn record_discovery(&self, files_found: u64, discovery_time_ms: f64) {
        self.metrics
            .files_discovered
            .fetch_add(files_found, Ordering::Relaxed);
        self.metrics
            .total_discovery_time_ms
            .fetch_add(discovery_time_ms as u64, Ordering::Relaxed);
        *self.last_activity.write().await = Some(Instant::now());
    }

    /// Record sync operation
    pub async fn record_sync(
        &self,
        orphaned_removed: u64,
        unindexed_found: u64,
        sync_time_ms: f64,
    ) {
        self.metrics
            .files_removed
            .fetch_add(orphaned_removed, Ordering::Relaxed);
        self.metrics
            .files_indexed_realtime
            .fetch_add(unindexed_found, Ordering::Relaxed);
        self.metrics
            .total_sync_time_ms
            .fetch_add(sync_time_ms as u64, Ordering::Relaxed);
        *self.last_activity.write().await = Some(Instant::now());
    }

    /// Record error
    pub async fn record_error(&self, error_type: &str, error_message: &str) {
        self.metrics.total_errors.fetch_add(1, Ordering::Relaxed);

        // Update error count by type
        {
            let mut errors = self.errors_by_type.write().await;
            *errors.entry(error_type.to_string()).or_insert(0) += 1;
        }

        // Update last error
        *self.last_error.write().await = Some(format!("{}: {}", error_type, error_message));
    }

    /// Update collection metrics
    pub async fn update_collection_metrics(
        &self,
        collection_name: &str,
        total_vectors: u64,
        size_bytes: u64,
    ) {
        let mut collections = self.collections.write().await;
        let collection = collections
            .entry(collection_name.to_string())
            .or_insert_with(|| CollectionMetrics {
                total_vectors: 0,
                vectors_added: 0,
                vectors_removed: 0,
                vectors_updated: 0,
                last_update: None,
            });

        collection.total_vectors = total_vectors;
        collection.last_update = Some(chrono::Utc::now().to_rfc3339());
    }

    /// Record file skip
    pub fn record_file_skipped(&self) {
        self.metrics.files_skipped.fetch_add(1, Ordering::Relaxed);
    }

    /// Record file in progress
    pub fn record_file_in_progress(&self) {
        self.metrics
            .files_in_progress
            .fetch_add(1, Ordering::Relaxed);
    }

    /// Record file processing finished
    pub fn record_file_processing_finished(&self) {
        self.metrics
            .files_in_progress
            .fetch_sub(1, Ordering::Relaxed);
    }

    /// Record API request
    pub fn record_api_request(&self, success: bool, response_time_ms: f64) {
        self.metrics
            .total_api_requests
            .fetch_add(1, Ordering::Relaxed);
        self.metrics
            .total_api_response_time_ms
            .fetch_add(response_time_ms as u64, Ordering::Relaxed);

        if success {
            self.metrics
                .successful_api_requests
                .fetch_add(1, Ordering::Relaxed);
        } else {
            self.metrics
                .failed_api_requests
                .fetch_add(1, Ordering::Relaxed);
        }

        // Update peak response time
        let current_peak = self.metrics.peak_api_response_ms.load(Ordering::Relaxed);
        if response_time_ms as u64 > current_peak {
            self.metrics
                .peak_api_response_ms
                .store(response_time_ms as u64, Ordering::Relaxed);
        }
    }

    /// Record restart
    pub fn record_restart(&self) {
        self.metrics.restart_count.fetch_add(1, Ordering::Relaxed);
    }

    /// Set running status
    pub fn set_running(&self, running: bool) {
        self.metrics.is_running.store(running, Ordering::Relaxed);
    }

    /// Record connection opened
    pub fn record_connection_opened(&self) {
        self.metrics
            .active_connections
            .fetch_add(1, Ordering::Relaxed);
    }

    /// Record connection closed
    pub fn record_connection_closed(&self) {
        self.metrics
            .active_connections
            .fetch_sub(1, Ordering::Relaxed);
    }

    /// Record disk I/O operation
    pub fn record_disk_io(&self, bytes: u64) {
        self.metrics.disk_io_ops.fetch_add(1, Ordering::Relaxed);
    }

    /// Record network I/O
    pub fn record_network_io(&self, bytes: u64) {
        self.metrics
            .network_io_bytes
            .fetch_add(bytes, Ordering::Relaxed);
    }

    /// Calculate health score based on error rate and performance
    async fn calculate_health_score(&self) -> u8 {
        let total_processed = self.metrics.total_files_processed.load(Ordering::Relaxed);
        let total_errors = self.metrics.total_errors.load(Ordering::Relaxed);

        if total_processed == 0 {
            return 100; // No processing yet, assume healthy
        }

        let error_rate = total_errors as f64 / total_processed as f64;
        let health_score = ((1.0 - error_rate) * 100.0) as u8;

        health_score.min(100).max(0)
    }

    /// Get current memory usage
    fn get_memory_usage(&self) -> u64 {
        // Get process memory usage
        if let Ok(status) = std::fs::read_to_string("/proc/self/status") {
            for line in status.lines() {
                if line.starts_with("VmRSS:") {
                    if let Some(kb_str) = line.split_whitespace().nth(1) {
                        if let Ok(kb) = kb_str.parse::<u64>() {
                            return kb * 1024; // Convert KB to bytes
                        }
                    }
                }
            }
        }
        0
    }

    /// Get current CPU usage
    fn get_cpu_usage(&self) -> f64 {
        // Simple CPU usage estimation based on system load
        if let Ok(loadavg) = std::fs::read_to_string("/proc/loadavg") {
            if let Some(load_str) = loadavg.split_whitespace().next() {
                if let Ok(load) = load_str.parse::<f64>() {
                    // Convert load average to percentage (rough estimation)
                    return (load * 100.0).min(100.0);
                }
            }
        }
        0.0
    }

    /// Get current thread count
    fn get_thread_count(&self) -> u32 {
        // Get thread count from /proc/self/status
        if let Ok(status) = std::fs::read_to_string("/proc/self/status") {
            for line in status.lines() {
                if line.starts_with("Threads:") {
                    if let Some(thread_str) = line.split_whitespace().nth(1) {
                        if let Ok(threads) = thread_str.parse::<u32>() {
                            return threads;
                        }
                    }
                }
            }
        }
        // Fallback: count active threads in the process
        std::thread::available_parallelism()
            .map(|n| n.get() as u32)
            .unwrap_or(1)
    }

    /// Get active file handles
    fn get_active_file_handles(&self) -> u32 {
        // Count open file descriptors
        if let Ok(fd_dir) = std::fs::read_dir("/proc/self/fd") {
            fd_dir.count() as u32
        } else {
            0
        }
    }
}

impl Default for MetricsCollector {
    fn default() -> Self {
        Self::new()
    }
}
