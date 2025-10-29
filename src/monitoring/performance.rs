//! Performance monitoring utilities for the Vectorizer server
//!
//! This module provides comprehensive performance monitoring capabilities
//! including metrics collection, performance tracking, and optimization insights.

use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};

use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;

/// Performance metrics for different operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OperationMetrics {
    /// Operation name
    pub operation: String,
    /// Total number of operations
    pub count: u64,
    /// Total time spent
    pub total_duration: Duration,
    /// Average time per operation
    pub avg_duration: Duration,
    /// Minimum time
    pub min_duration: Duration,
    /// Maximum time
    pub max_duration: Duration,
    /// Success count
    pub success_count: u64,
    /// Error count
    pub error_count: u64,
    /// P50 percentile duration
    pub p50_duration: Duration,
    /// P95 percentile duration
    pub p95_duration: Duration,
    /// P99 percentile duration
    pub p99_duration: Duration,
    /// Recent operations (last 100)
    pub recent_durations: Vec<Duration>,
    /// Throughput (operations per second)
    pub throughput: f64,
    /// Error rate (0.0 to 1.0)
    pub error_rate: f64,
}

/// System resource metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemMetrics {
    /// Memory usage in bytes
    pub memory_usage: u64,
    /// CPU usage percentage
    pub cpu_usage: f64,
    /// Number of active threads
    pub thread_count: usize,
    /// Number of open file handles
    pub file_handles: u64,
    /// Disk I/O operations per second
    pub disk_io_ops: u64,
    /// Network I/O bytes per second
    pub network_io_bytes: u64,
    /// Available memory in bytes
    pub available_memory: u64,
    /// Total memory in bytes
    pub total_memory: u64,
    /// Memory usage percentage
    pub memory_usage_percent: f64,
    /// Load average (1 minute)
    pub load_average_1m: f64,
    /// Load average (5 minutes)
    pub load_average_5m: f64,
    /// Load average (15 minutes)
    pub load_average_15m: f64,
    /// Disk usage percentage
    pub disk_usage_percent: f64,
    /// Network connections count
    pub network_connections: u64,
    /// Process count
    pub process_count: u64,
    /// Uptime in seconds
    pub uptime_seconds: u64,
}

/// Collection-specific metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollectionMetrics {
    /// Collection name
    pub name: String,
    /// Number of vectors
    pub vector_count: u64,
    /// Memory usage in bytes
    pub memory_usage: u64,
    /// Search operations count
    pub search_count: u64,
    /// Insert operations count
    pub insert_count: u64,
    /// Update operations count
    pub update_count: u64,
    /// Delete operations count
    pub delete_count: u64,
    /// Average search time
    pub avg_search_time: Duration,
    /// Last access time (as timestamp)
    pub last_access: Option<u64>,
}

/// Comprehensive performance report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceReport {
    /// System metrics
    pub system: SystemMetrics,
    /// Operation metrics
    pub operations: HashMap<String, OperationMetrics>,
    /// Collection metrics
    pub collections: HashMap<String, CollectionMetrics>,
    /// Server uptime
    pub uptime: Duration,
    /// Report generation time (as timestamp)
    pub generated_at: u64,
}

/// Performance monitor for tracking server metrics
pub struct PerformanceMonitor {
    /// Operation metrics
    operations: Arc<RwLock<HashMap<String, OperationMetrics>>>,
    /// Collection metrics
    collections: Arc<RwLock<HashMap<String, CollectionMetrics>>>,
    /// Server start time
    start_time: Instant,
    /// System metrics cache
    system_metrics: Arc<RwLock<Option<SystemMetrics>>>,
}

impl PerformanceMonitor {
    /// Create a new performance monitor
    pub fn new() -> Self {
        Self {
            operations: Arc::new(RwLock::new(HashMap::new())),
            collections: Arc::new(RwLock::new(HashMap::new())),
            start_time: Instant::now(),
            system_metrics: Arc::new(RwLock::new(None)),
        }
    }

    /// Record an operation with timing
    pub async fn record_operation(&self, operation: &str, duration: Duration, success: bool) {
        let mut ops = self.operations.write().await;
        let metrics = ops
            .entry(operation.to_string())
            .or_insert_with(|| OperationMetrics {
                operation: operation.to_string(),
                count: 0,
                total_duration: Duration::ZERO,
                avg_duration: Duration::ZERO,
                min_duration: Duration::MAX,
                max_duration: Duration::ZERO,
                success_count: 0,
                error_count: 0,
                p50_duration: Duration::ZERO,
                p95_duration: Duration::ZERO,
                p99_duration: Duration::ZERO,
                recent_durations: Vec::with_capacity(100),
                throughput: 0.0,
                error_rate: 0.0,
            });

        metrics.count += 1;
        metrics.total_duration += duration;
        metrics.avg_duration = metrics.total_duration / metrics.count as u32;
        metrics.min_duration = metrics.min_duration.min(duration);
        metrics.max_duration = metrics.max_duration.max(duration);

        // Update recent durations for percentile calculations
        metrics.recent_durations.push(duration);
        if metrics.recent_durations.len() > 100 {
            metrics.recent_durations.remove(0);
        }

        // Calculate percentiles
        if !metrics.recent_durations.is_empty() {
            let mut sorted_durations = metrics.recent_durations.clone();
            sorted_durations.sort();
            let len = sorted_durations.len();

            metrics.p50_duration = sorted_durations[len * 50 / 100];
            metrics.p95_duration = sorted_durations[len * 95 / 100];
            metrics.p99_duration = sorted_durations[len * 99 / 100];
        }

        if success {
            metrics.success_count += 1;
        } else {
            metrics.error_count += 1;
        }

        // Calculate error rate
        metrics.error_rate = metrics.error_count as f64 / metrics.count as f64;

        // Calculate throughput (operations per second)
        let uptime_seconds = self.start_time.elapsed().as_secs_f64();
        if uptime_seconds > 0.0 {
            metrics.throughput = metrics.count as f64 / uptime_seconds;
        }
    }

    /// Record collection metrics
    pub async fn record_collection_metrics(
        &self,
        collection_name: &str,
        vector_count: u64,
        memory_usage: u64,
        operation: &str,
        duration: Duration,
    ) {
        let mut collections = self.collections.write().await;
        let metrics = collections
            .entry(collection_name.to_string())
            .or_insert_with(|| CollectionMetrics {
                name: collection_name.to_string(),
                vector_count: 0,
                memory_usage: 0,
                search_count: 0,
                insert_count: 0,
                update_count: 0,
                delete_count: 0,
                avg_search_time: Duration::ZERO,
                last_access: None,
            });

        metrics.vector_count = vector_count;
        metrics.memory_usage = memory_usage;
        metrics.last_access = Some(Instant::now().elapsed().as_secs());

        match operation {
            "search" => {
                metrics.search_count += 1;
                // Update average search time
                let total_time =
                    metrics.avg_search_time * (metrics.search_count - 1) as u32 + duration;
                metrics.avg_search_time = total_time / metrics.search_count as u32;
            }
            "insert" => metrics.insert_count += 1,
            "update" => metrics.update_count += 1,
            "delete" => metrics.delete_count += 1,
            _ => {}
        }
    }

    /// Update system metrics
    pub async fn update_system_metrics(&self, metrics: SystemMetrics) {
        let mut system_metrics = self.system_metrics.write().await;
        *system_metrics = Some(metrics);
    }

    /// Generate a comprehensive performance report
    pub async fn generate_report(&self) -> PerformanceReport {
        let operations = self.operations.read().await.clone();
        let collections = self.collections.read().await.clone();
        let system = self
            .system_metrics
            .read()
            .await
            .clone()
            .unwrap_or_else(|| SystemMetrics {
                memory_usage: 0,
                cpu_usage: 0.0,
                thread_count: 0,
                file_handles: 0,
                disk_io_ops: 0,
                network_io_bytes: 0,
                available_memory: 0,
                total_memory: 0,
                memory_usage_percent: 0.0,
                load_average_1m: 0.0,
                load_average_5m: 0.0,
                load_average_15m: 0.0,
                disk_usage_percent: 0.0,
                network_connections: 0,
                process_count: 0,
                uptime_seconds: 0,
            });

        PerformanceReport {
            system,
            operations,
            collections,
            uptime: self.start_time.elapsed(),
            generated_at: Instant::now().elapsed().as_secs(),
        }
    }

    /// Get operation metrics for a specific operation
    pub async fn get_operation_metrics(&self, operation: &str) -> Option<OperationMetrics> {
        let operations = self.operations.read().await;
        operations.get(operation).cloned()
    }

    /// Get collection metrics for a specific collection
    pub async fn get_collection_metrics(&self, collection: &str) -> Option<CollectionMetrics> {
        let collections = self.collections.read().await;
        collections.get(collection).cloned()
    }

    /// Reset all metrics
    pub async fn reset(&self) {
        let mut operations = self.operations.write().await;
        operations.clear();

        let mut collections = self.collections.write().await;
        collections.clear();

        let mut system_metrics = self.system_metrics.write().await;
        *system_metrics = None;
    }

    /// Get server uptime
    pub fn uptime(&self) -> Duration {
        self.start_time.elapsed()
    }
}

impl Default for PerformanceMonitor {
    fn default() -> Self {
        Self::new()
    }
}

/// Performance monitoring utilities
pub mod utils {
    use std::future::Future;

    use super::*;

    /// Measure the execution time of an async operation
    pub async fn measure_async<F, T>(
        operation: &str,
        monitor: &PerformanceMonitor,
        f: F,
    ) -> Result<T, String>
    where
        F: Future<Output = Result<T, String>>,
    {
        let start = Instant::now();
        let result = f.await;
        let duration = start.elapsed();

        let success = result.is_ok();
        monitor.record_operation(operation, duration, success).await;

        result
    }

    /// Measure the execution time of a sync operation
    pub fn measure_sync<F, T>(
        operation: &str,
        monitor: &PerformanceMonitor,
        f: F,
    ) -> Result<T, String>
    where
        F: FnOnce() -> Result<T, String>,
    {
        let start = Instant::now();
        let result = f();
        let duration = start.elapsed();

        let success = result.is_ok();
        // Note: This would need to be async in a real implementation
        // monitor.record_operation(operation, duration, success).await;

        result
    }
}

#[cfg(test)]
mod tests {
    use tokio::time::sleep;

    use super::*;

    #[tokio::test]
    async fn test_performance_monitor_creation() {
        let monitor = PerformanceMonitor::new();
        assert_eq!(monitor.uptime().as_secs(), 0);
    }

    #[tokio::test]
    async fn test_operation_recording() {
        let monitor = PerformanceMonitor::new();

        // Record a successful operation
        monitor
            .record_operation("test_op", Duration::from_millis(100), true)
            .await;

        let metrics = monitor.get_operation_metrics("test_op").await.unwrap();
        assert_eq!(metrics.count, 1);
        assert_eq!(metrics.success_count, 1);
        assert_eq!(metrics.error_count, 0);
        assert_eq!(metrics.total_duration, Duration::from_millis(100));
    }

    #[tokio::test]
    async fn test_collection_metrics() {
        let monitor = PerformanceMonitor::new();

        monitor
            .record_collection_metrics(
                "test_collection",
                1000,
                1024 * 1024, // 1MB
                "search",
                Duration::from_millis(50),
            )
            .await;

        let metrics = monitor
            .get_collection_metrics("test_collection")
            .await
            .unwrap();
        assert_eq!(metrics.vector_count, 1000);
        assert_eq!(metrics.memory_usage, 1024 * 1024);
        assert_eq!(metrics.search_count, 1);
    }

    #[tokio::test]
    async fn test_performance_report() {
        let monitor = PerformanceMonitor::new();

        // Record some operations
        monitor
            .record_operation("op1", Duration::from_millis(100), true)
            .await;
        monitor
            .record_operation("op2", Duration::from_millis(200), false)
            .await;

        // Record collection metrics
        monitor
            .record_collection_metrics(
                "collection1",
                500,
                512 * 1024,
                "insert",
                Duration::from_millis(75),
            )
            .await;

        let report = monitor.generate_report().await;
        assert_eq!(report.operations.len(), 2);
        assert_eq!(report.collections.len(), 1);
        // Uptime is always non-negative (u64), so no assertion needed
    }

    #[tokio::test]
    async fn test_measure_async() {
        let monitor = PerformanceMonitor::new();

        let result = utils::measure_async("test_async", &monitor, async {
            sleep(Duration::from_millis(10)).await;
            Ok("success")
        })
        .await;

        assert_eq!(result, Ok("success"));

        let metrics = monitor.get_operation_metrics("test_async").await.unwrap();
        assert_eq!(metrics.count, 1);
        assert_eq!(metrics.success_count, 1);
        assert!(metrics.total_duration >= Duration::from_millis(10));
    }
}
