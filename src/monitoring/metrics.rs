//! Prometheus Metrics Definitions
//!
//! This module defines all Prometheus metrics used for monitoring the vector database.
//! Metrics are organized by subsystem for clarity and maintainability.

use once_cell::sync::Lazy;
use prometheus::{
    Counter, CounterVec, Gauge, GaugeVec, Histogram, HistogramOpts, HistogramVec, Opts, Registry,
};

/// Global metrics instance
pub static METRICS: Lazy<Metrics> = Lazy::new(Metrics::new);

/// Centralized metrics structure
#[derive(Clone)]
pub struct Metrics {
    // ═══════════════════════════════════════════════════════════════════════
    // Search Metrics
    // ═══════════════════════════════════════════════════════════════════════
    /// Total number of search requests
    pub search_requests_total: CounterVec,

    /// Search request latency in seconds
    pub search_latency_seconds: HistogramVec,

    /// Number of results returned per search
    pub search_results_count: HistogramVec,

    // ═══════════════════════════════════════════════════════════════════════
    // Indexing Metrics
    // ═══════════════════════════════════════════════════════════════════════
    /// Total number of vectors stored
    pub vectors_total: Gauge,

    /// Total number of collections
    pub collections_total: Gauge,

    /// Alias operations counter (create/delete/rename)
    pub alias_operations_total: CounterVec,

    /// Total number of insert requests
    pub insert_requests_total: CounterVec,

    /// Insert request latency in seconds
    pub insert_latency_seconds: Histogram,

    // ═══════════════════════════════════════════════════════════════════════
    // Replication Metrics
    // ═══════════════════════════════════════════════════════════════════════
    /// Replication lag in milliseconds
    pub replication_lag_ms: Gauge,

    /// Total bytes sent via replication
    pub replication_bytes_sent_total: Counter,

    /// Total bytes received via replication
    pub replication_bytes_received_total: Counter,

    /// Number of operations pending replication
    pub replication_operations_pending: Gauge,

    // ═══════════════════════════════════════════════════════════════════════
    // System Metrics
    // ═══════════════════════════════════════════════════════════════════════
    /// Memory usage in bytes
    pub memory_usage_bytes: Gauge,

    /// Total cache requests
    pub cache_requests_total: CounterVec,

    /// Total API errors
    pub api_errors_total: CounterVec,

    // ═══════════════════════════════════════════════════════════════════════
    // GPU Metrics (Metal GPU - macOS only)
    // ═══════════════════════════════════════════════════════════════════════
    /// GPU backend type (metal/cpu)
    pub gpu_backend_type: Gauge,

    /// GPU memory usage in bytes
    pub gpu_memory_usage_bytes: Gauge,

    /// GPU search requests total
    pub gpu_search_requests_total: Counter,

    /// GPU search latency in seconds
    pub gpu_search_latency_seconds: Histogram,

    /// GPU batch operations total
    pub gpu_batch_operations_total: CounterVec,

    /// GPU batch operation latency in seconds
    pub gpu_batch_latency_seconds: HistogramVec,
}

impl Metrics {
    /// Create a new metrics instance
    pub fn new() -> Self {
        Self {
            // Search metrics
            search_requests_total: CounterVec::new(
                Opts::new(
                    "vectorizer_search_requests_total",
                    "Total number of search requests",
                ),
                &["collection", "search_type", "status"],
            )
            .unwrap(),

            search_latency_seconds: HistogramVec::new(
                HistogramOpts::new(
                    "vectorizer_search_latency_seconds",
                    "Search request latency in seconds",
                )
                .buckets(vec![
                    0.001, 0.003, 0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0,
                ]),
                &["collection", "search_type"],
            )
            .unwrap(),

            search_results_count: HistogramVec::new(
                HistogramOpts::new(
                    "vectorizer_search_results_count",
                    "Number of results returned per search",
                )
                .buckets(vec![
                    0.0, 1.0, 5.0, 10.0, 25.0, 50.0, 100.0, 250.0, 500.0, 1000.0,
                ]),
                &["collection", "search_type"],
            )
            .unwrap(),

            // Indexing metrics
            vectors_total: Gauge::new("vectorizer_vectors_total", "Total number of vectors stored")
                .unwrap(),

            collections_total: Gauge::new(
                "vectorizer_collections_total",
                "Total number of collections",
            )
            .unwrap(),

            alias_operations_total: CounterVec::new(
                Opts::new(
                    "vectorizer_alias_operations_total",
                    "Total number of alias operations grouped by status",
                ),
                &["operation", "status"],
            )
            .unwrap(),

            insert_requests_total: CounterVec::new(
                Opts::new(
                    "vectorizer_insert_requests_total",
                    "Total number of insert requests",
                ),
                &["collection", "status"],
            )
            .unwrap(),

            insert_latency_seconds: Histogram::with_opts(
                HistogramOpts::new(
                    "vectorizer_insert_latency_seconds",
                    "Insert request latency in seconds",
                )
                .buckets(vec![
                    0.001, 0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0, 2.5,
                ]),
            )
            .unwrap(),

            // Replication metrics
            replication_lag_ms: Gauge::new(
                "vectorizer_replication_lag_ms",
                "Replication lag in milliseconds",
            )
            .unwrap(),

            replication_bytes_sent_total: Counter::new(
                "vectorizer_replication_bytes_sent_total",
                "Total bytes sent via replication",
            )
            .unwrap(),

            replication_bytes_received_total: Counter::new(
                "vectorizer_replication_bytes_received_total",
                "Total bytes received via replication",
            )
            .unwrap(),

            replication_operations_pending: Gauge::new(
                "vectorizer_replication_operations_pending",
                "Number of operations pending replication",
            )
            .unwrap(),

            // System metrics
            memory_usage_bytes: Gauge::new(
                "vectorizer_memory_usage_bytes",
                "Memory usage in bytes",
            )
            .unwrap(),

            cache_requests_total: CounterVec::new(
                Opts::new("vectorizer_cache_requests_total", "Total cache requests"),
                &["cache_type", "result"],
            )
            .unwrap(),

            api_errors_total: CounterVec::new(
                Opts::new("vectorizer_api_errors_total", "Total API errors"),
                &["endpoint", "error_type", "status_code"],
            )
            .unwrap(),

            // GPU metrics
            gpu_backend_type: Gauge::new(
                "vectorizer_gpu_backend_type",
                "GPU backend type (0=None, 1=Metal)",
            )
            .unwrap(),

            gpu_memory_usage_bytes: Gauge::new(
                "vectorizer_gpu_memory_usage_bytes",
                "GPU memory usage in bytes",
            )
            .unwrap(),

            gpu_search_requests_total: Counter::new(
                "vectorizer_gpu_search_requests_total",
                "Total GPU search requests",
            )
            .unwrap(),

            gpu_search_latency_seconds: Histogram::with_opts(
                HistogramOpts::new(
                    "vectorizer_gpu_search_latency_seconds",
                    "GPU search latency in seconds",
                )
                .buckets(vec![0.001, 0.005, 0.01, 0.05, 0.1, 0.5, 1.0, 5.0]),
            )
            .unwrap(),

            gpu_batch_operations_total: CounterVec::new(
                Opts::new(
                    "vectorizer_gpu_batch_operations_total",
                    "Total GPU batch operations",
                ),
                &["operation_type"],
            )
            .unwrap(),

            gpu_batch_latency_seconds: HistogramVec::new(
                HistogramOpts::new(
                    "vectorizer_gpu_batch_latency_seconds",
                    "GPU batch operation latency in seconds",
                )
                .buckets(vec![0.01, 0.05, 0.1, 0.5, 1.0, 5.0, 10.0]),
                &["operation_type"],
            )
            .unwrap(),
        }
    }

    /// Register all metrics with the given registry
    pub fn register(&self, registry: &Registry) -> Result<(), prometheus::Error> {
        // Search metrics
        registry.register(Box::new(self.search_requests_total.clone()))?;
        registry.register(Box::new(self.search_latency_seconds.clone()))?;
        registry.register(Box::new(self.search_results_count.clone()))?;

        // Indexing metrics
        registry.register(Box::new(self.vectors_total.clone()))?;
        registry.register(Box::new(self.collections_total.clone()))?;
        registry.register(Box::new(self.alias_operations_total.clone()))?;
        registry.register(Box::new(self.insert_requests_total.clone()))?;
        registry.register(Box::new(self.insert_latency_seconds.clone()))?;

        // Replication metrics
        registry.register(Box::new(self.replication_lag_ms.clone()))?;
        registry.register(Box::new(self.replication_bytes_sent_total.clone()))?;
        registry.register(Box::new(self.replication_bytes_received_total.clone()))?;
        registry.register(Box::new(self.replication_operations_pending.clone()))?;

        // System metrics
        registry.register(Box::new(self.memory_usage_bytes.clone()))?;
        registry.register(Box::new(self.cache_requests_total.clone()))?;
        registry.register(Box::new(self.api_errors_total.clone()))?;

        // GPU metrics
        registry.register(Box::new(self.gpu_backend_type.clone()))?;
        registry.register(Box::new(self.gpu_memory_usage_bytes.clone()))?;
        registry.register(Box::new(self.gpu_search_requests_total.clone()))?;
        registry.register(Box::new(self.gpu_search_latency_seconds.clone()))?;
        registry.register(Box::new(self.gpu_batch_operations_total.clone()))?;
        registry.register(Box::new(self.gpu_batch_latency_seconds.clone()))?;

        Ok(())
    }
}

impl Default for Metrics {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_metrics_creation() {
        let metrics = Metrics::new();

        // Verify search metrics
        metrics
            .search_requests_total
            .with_label_values(&["test", "basic", "success"])
            .inc();
        assert_eq!(
            metrics
                .search_requests_total
                .with_label_values(&["test", "basic", "success"])
                .get(),
            1.0
        );
    }

    #[test]
    fn test_metrics_registration() {
        let metrics = Metrics::new();
        let registry = Registry::new();

        let result = metrics.register(&registry);
        assert!(result.is_ok());
    }

    #[test]
    fn test_histogram_buckets() {
        let metrics = Metrics::new();

        // Test search latency recording
        let timer = metrics
            .search_latency_seconds
            .with_label_values(&["test", "basic"])
            .start_timer();
        drop(timer); // Simulate completion
    }

    #[test]
    fn test_gauge_operations() {
        let metrics = Metrics::new();

        // Test vector count gauge
        metrics.vectors_total.set(1000.0);
        assert_eq!(metrics.vectors_total.get(), 1000.0);

        metrics.vectors_total.inc();
        assert_eq!(metrics.vectors_total.get(), 1001.0);

        metrics.vectors_total.dec();
        assert_eq!(metrics.vectors_total.get(), 1000.0);
    }

    #[test]
    fn test_counter_operations() {
        let metrics = Metrics::new();

        // Test insert requests counter
        metrics
            .insert_requests_total
            .with_label_values(&["test", "success"])
            .inc();
        metrics
            .insert_requests_total
            .with_label_values(&["test", "success"])
            .inc_by(5.0);

        assert_eq!(
            metrics
                .insert_requests_total
                .with_label_values(&["test", "success"])
                .get(),
            6.0
        );
    }
}
