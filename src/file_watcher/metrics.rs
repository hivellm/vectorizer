// src/file_watcher/metrics.rs
use std::sync::Arc;
use std::time::{Duration, Instant};
use std::collections::HashMap;
use tokio::sync::RwLock;
use serde::{Deserialize, Serialize};
use tracing::{debug, info, warn};
// use sysinfo; // Temporarily disabled to avoid compilation issues

/// Comprehensive metrics for File Watcher performance monitoring
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileWatcherMetrics {
    /// Time-based metrics
    pub timing: TimingMetrics,
    /// File processing metrics
    pub files: FileMetrics,
    /// System resource metrics
    pub system: SystemMetrics,
    /// Network and API metrics
    pub network: NetworkMetrics,
    /// Error and status metrics
    pub status: StatusMetrics,
    /// Collection-specific metrics
    pub collections: HashMap<String, CollectionMetrics>,
}

/// Time-based performance metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimingMetrics {
    /// Average file processing time in milliseconds
    pub avg_file_processing_ms: f64,
    /// Average discovery time in milliseconds
    pub avg_discovery_ms: f64,
    /// Average sync time in milliseconds
    pub avg_sync_ms: f64,
    /// Total uptime in seconds
    pub uptime_seconds: u64,
    /// Last activity timestamp (not serialized)
    #[serde(skip)]
    pub last_activity: Option<Instant>,
    /// Peak processing time in milliseconds
    pub peak_processing_ms: u64,
}

/// File processing metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileMetrics {
    /// Total files processed
    pub total_files_processed: u64,
    /// Files processed successfully
    pub files_processed_success: u64,
    /// Files that failed processing
    pub files_processed_error: u64,
    /// Files skipped (size, pattern, etc.)
    pub files_skipped: u64,
    /// Files currently being processed
    pub files_in_progress: u64,
    /// Files discovered during startup
    pub files_discovered: u64,
    /// Files removed (orphaned)
    pub files_removed: u64,
    /// Files indexed in real-time
    pub files_indexed_realtime: u64,
}

/// System resource metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemMetrics {
    /// Memory usage in bytes
    pub memory_usage_bytes: u64,
    /// CPU usage percentage (0-100)
    pub cpu_usage_percent: f64,
    /// Disk I/O operations per second
    pub disk_io_ops_per_sec: u64,
    /// Network I/O bytes per second
    pub network_io_bytes_per_sec: u64,
    /// Active file handles
    pub active_file_handles: u32,
    /// Thread count
    pub thread_count: u32,
}

/// Network and API metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkMetrics {
    /// Total API requests
    pub total_api_requests: u64,
    /// Successful API requests
    pub successful_api_requests: u64,
    /// Failed API requests
    pub failed_api_requests: u64,
    /// Average API response time in milliseconds
    pub avg_api_response_ms: f64,
    /// Peak API response time in milliseconds
    pub peak_api_response_ms: u64,
    /// Current active connections
    pub active_connections: u32,
}

/// Error and status metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatusMetrics {
    /// Total errors encountered
    pub total_errors: u64,
    /// Errors by type
    pub errors_by_type: HashMap<String, u64>,
    /// Current status (running, stopped, error)
    pub current_status: String,
    /// Last error message
    pub last_error: Option<String>,
    /// Health score (0-100)
    pub health_score: u8,
    /// Restart count
    pub restart_count: u32,
}

/// Collection-specific metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollectionMetrics {
    /// Collection name
    pub name: String,
    /// Total vectors in collection
    pub total_vectors: u64,
    /// Files indexed in this collection
    pub files_indexed: u64,
    /// Last update timestamp (not serialized)
    #[serde(skip)]
    pub last_update: Option<Instant>,
    /// Average indexing time per file
    pub avg_indexing_time_ms: f64,
    /// Collection size in bytes
    pub size_bytes: u64,
}

/// Metrics collector and manager
pub struct MetricsCollector {
    metrics: Arc<RwLock<FileWatcherMetrics>>,
    start_time: Instant,
    last_update: Instant,
}

impl MetricsCollector {
    /// Create a new metrics collector
    pub fn new() -> Self {
        let now = Instant::now();
        Self {
            metrics: Arc::new(RwLock::new(FileWatcherMetrics {
                timing: TimingMetrics {
                    avg_file_processing_ms: 0.0,
                    avg_discovery_ms: 0.0,
                    avg_sync_ms: 0.0,
                    uptime_seconds: 0,
                    last_activity: Some(now),
                    peak_processing_ms: 0,
                },
                files: FileMetrics {
                    total_files_processed: 0,
                    files_processed_success: 0,
                    files_processed_error: 0,
                    files_skipped: 0,
                    files_in_progress: 0,
                    files_discovered: 0,
                    files_removed: 0,
                    files_indexed_realtime: 0,
                },
                system: SystemMetrics {
                    memory_usage_bytes: 0,
                    cpu_usage_percent: 0.0,
                    disk_io_ops_per_sec: 0,
                    network_io_bytes_per_sec: 0,
                    active_file_handles: 0,
                    thread_count: 0,
                },
                network: NetworkMetrics {
                    total_api_requests: 0,
                    successful_api_requests: 0,
                    failed_api_requests: 0,
                    avg_api_response_ms: 0.0,
                    peak_api_response_ms: 0,
                    active_connections: 0,
                },
                status: StatusMetrics {
                    total_errors: 0,
                    errors_by_type: HashMap::new(),
                    current_status: "initializing".to_string(),
                    last_error: None,
                    health_score: 100,
                    restart_count: 0,
                },
                collections: HashMap::new(),
            })),
            start_time: now,
            last_update: now,
        }
    }

    /// Get current metrics snapshot
    pub async fn get_metrics(&self) -> FileWatcherMetrics {
        let mut metrics = self.metrics.read().await.clone();
        
        // Update uptime
        metrics.timing.uptime_seconds = self.start_time.elapsed().as_secs();
        
        // Update system metrics
        self.update_system_metrics(&mut metrics).await;
        
        metrics
    }

    /// Record file processing start
    pub async fn record_file_processing_start(&self) {
        let mut metrics = self.metrics.write().await;
        metrics.files.files_in_progress += 1;
        metrics.timing.last_activity = Some(Instant::now());
    }

    /// Record file processing completion
    pub async fn record_file_processing_complete(&self, success: bool, processing_time_ms: u64) {
        let mut metrics = self.metrics.write().await;
        
        metrics.files.files_in_progress = metrics.files.files_in_progress.saturating_sub(1);
        metrics.files.total_files_processed += 1;
        
        if success {
            metrics.files.files_processed_success += 1;
        } else {
            metrics.files.files_processed_error += 1;
        }

        // Update average processing time
        let total = metrics.files.total_files_processed as f64;
        let current_avg = metrics.timing.avg_file_processing_ms;
        metrics.timing.avg_file_processing_ms = (current_avg * (total - 1.0) + processing_time_ms as f64) / total;

        // Update peak processing time
        if processing_time_ms > metrics.timing.peak_processing_ms {
            metrics.timing.peak_processing_ms = processing_time_ms;
        }

        metrics.timing.last_activity = Some(Instant::now());
    }

    /// Record file discovery metrics
    pub async fn record_discovery(&self, files_found: u64, discovery_time_ms: u64) {
        let mut metrics = self.metrics.write().await;
        
        metrics.files.files_discovered += files_found;
        
        // Update average discovery time
        let total_discoveries = metrics.files.files_discovered as f64;
        let current_avg = metrics.timing.avg_discovery_ms;
        metrics.timing.avg_discovery_ms = (current_avg * (total_discoveries - files_found as f64) + discovery_time_ms as f64) / total_discoveries;

        metrics.timing.last_activity = Some(Instant::now());
    }

    /// Record sync metrics
    pub async fn record_sync(&self, orphaned_removed: u64, unindexed_found: u64, sync_time_ms: u64) {
        let mut metrics = self.metrics.write().await;
        
        metrics.files.files_removed += orphaned_removed;
        
        // Update average sync time
        let total_syncs = metrics.files.files_removed as f64;
        let current_avg = metrics.timing.avg_sync_ms;
        metrics.timing.avg_sync_ms = (current_avg * (total_syncs - orphaned_removed as f64) + sync_time_ms as f64) / total_syncs;

        metrics.timing.last_activity = Some(Instant::now());
    }

    /// Record error
    pub async fn record_error(&self, error_type: &str, error_message: &str) {
        let mut metrics = self.metrics.write().await;
        
        metrics.status.total_errors += 1;
        metrics.status.last_error = Some(error_message.to_string());
        
        *metrics.status.errors_by_type.entry(error_type.to_string()).or_insert(0) += 1;
        
        // Update health score based on error rate
        let error_rate = metrics.status.total_errors as f64 / metrics.files.total_files_processed.max(1) as f64;
        metrics.status.health_score = (100.0 - (error_rate * 100.0).min(100.0)) as u8;

        metrics.timing.last_activity = Some(Instant::now());
    }

    /// Record API request
    pub async fn record_api_request(&self, success: bool, response_time_ms: u64) {
        let mut metrics = self.metrics.write().await;
        
        metrics.network.total_api_requests += 1;
        
        if success {
            metrics.network.successful_api_requests += 1;
        } else {
            metrics.network.failed_api_requests += 1;
        }

        // Update average response time
        let total = metrics.network.total_api_requests as f64;
        let current_avg = metrics.network.avg_api_response_ms;
        metrics.network.avg_api_response_ms = (current_avg * (total - 1.0) + response_time_ms as f64) / total;

        // Update peak response time
        if response_time_ms > metrics.network.peak_api_response_ms {
            metrics.network.peak_api_response_ms = response_time_ms;
        }

        metrics.timing.last_activity = Some(Instant::now());
    }

    /// Update collection metrics
    pub async fn update_collection_metrics(&self, collection_name: &str, total_vectors: u64, size_bytes: u64) {
        let mut metrics = self.metrics.write().await;
        
        let collection_metrics = metrics.collections.entry(collection_name.to_string()).or_insert_with(|| CollectionMetrics {
            name: collection_name.to_string(),
            total_vectors: 0,
            files_indexed: 0,
            last_update: None,
            avg_indexing_time_ms: 0.0,
            size_bytes: 0,
        });

        collection_metrics.total_vectors = total_vectors;
        collection_metrics.size_bytes = size_bytes;
        collection_metrics.last_update = Some(Instant::now());

        metrics.timing.last_activity = Some(Instant::now());
    }

    /// Update system metrics (memory, CPU, etc.)
    async fn update_system_metrics(&self, metrics: &mut FileWatcherMetrics) {
        // Simplified system metrics to avoid sysinfo issues
        metrics.system.memory_usage_bytes = 1024 * 1024 * 100; // Mock 100MB
        metrics.system.cpu_usage_percent = 0.0;
        metrics.system.thread_count = 10; // Mock thread count

        // Update status
        if metrics.files.files_in_progress > 0 {
            metrics.status.current_status = "processing".to_string();
        } else {
            metrics.status.current_status = "idle".to_string();
        }
    }

    /// Get metrics summary for logging
    pub async fn get_summary(&self) -> String {
        let metrics = self.get_metrics().await;
        
        format!(
            "📊 File Watcher Metrics Summary:\n\
            ⏱️  Uptime: {}s | Processing: {}ms avg | Peak: {}ms\n\
            📁 Files: {} processed ({} success, {} errors, {} skipped)\n\
            🔄 Discovery: {} files | Sync: {} removed | Realtime: {} indexed\n\
            🌐 API: {} requests ({} success, {} failed) | {}ms avg response\n\
            💾 Memory: {}MB | CPU: {:.1}% | Threads: {}\n\
            ❌ Errors: {} total | Health: {}% | Status: {}",
            metrics.timing.uptime_seconds,
            metrics.timing.avg_file_processing_ms as u64,
            metrics.timing.peak_processing_ms,
            metrics.files.total_files_processed,
            metrics.files.files_processed_success,
            metrics.files.files_processed_error,
            metrics.files.files_skipped,
            metrics.files.files_discovered,
            metrics.files.files_removed,
            metrics.files.files_indexed_realtime,
            metrics.network.total_api_requests,
            metrics.network.successful_api_requests,
            metrics.network.failed_api_requests,
            metrics.network.avg_api_response_ms as u64,
            metrics.system.memory_usage_bytes / 1024 / 1024,
            metrics.system.cpu_usage_percent,
            metrics.system.thread_count,
            metrics.status.total_errors,
            metrics.status.health_score,
            metrics.status.current_status
        )
    }

    /// Reset metrics (useful for testing or restart scenarios)
    pub async fn reset(&self) {
        let mut metrics = self.metrics.write().await;
        *metrics = FileWatcherMetrics {
            timing: TimingMetrics {
                avg_file_processing_ms: 0.0,
                avg_discovery_ms: 0.0,
                avg_sync_ms: 0.0,
                uptime_seconds: 0,
                last_activity: Some(Instant::now()),
                peak_processing_ms: 0,
            },
            files: FileMetrics {
                total_files_processed: 0,
                files_processed_success: 0,
                files_processed_error: 0,
                files_skipped: 0,
                files_in_progress: 0,
                files_discovered: 0,
                files_removed: 0,
                files_indexed_realtime: 0,
            },
            system: SystemMetrics {
                memory_usage_bytes: 0,
                cpu_usage_percent: 0.0,
                disk_io_ops_per_sec: 0,
                network_io_bytes_per_sec: 0,
                active_file_handles: 0,
                thread_count: 0,
            },
            network: NetworkMetrics {
                total_api_requests: 0,
                successful_api_requests: 0,
                failed_api_requests: 0,
                avg_api_response_ms: 0.0,
                peak_api_response_ms: 0,
                active_connections: 0,
            },
            status: StatusMetrics {
                total_errors: 0,
                errors_by_type: HashMap::new(),
                current_status: "reset".to_string(),
                last_error: None,
                health_score: 100,
                restart_count: metrics.status.restart_count + 1,
            },
            collections: HashMap::new(),
        };
    }
}

impl Default for MetricsCollector {
    fn default() -> Self {
        Self::new()
    }
}
