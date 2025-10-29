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
<<<<<<< HEAD
=======

    // ═══════════════════════════════════════════════════════════════════════
    // Performance Metrics
    // ═══════════════════════════════════════════════════════════════════════
    /// CPU usage percentage
    pub cpu_usage_percent: Gauge,

    /// Disk I/O operations per second
    pub disk_io_ops_per_second: Gauge,

    /// Network I/O bytes per second
    pub network_io_bytes_per_second: Gauge,

    /// Database connection pool size
    pub db_connection_pool_size: Gauge,

    /// Active database connections
    pub db_active_connections: Gauge,

    /// Database query latency in seconds
    pub db_query_latency_seconds: HistogramVec,

    /// Cache hit ratio
    pub cache_hit_ratio: Gauge,

    /// Cache size in bytes
    pub cache_size_bytes: Gauge,

    /// Cache eviction count
    pub cache_evictions_total: Counter,

    // ═══════════════════════════════════════════════════════════════════════
    // Business Metrics
    // ═══════════════════════════════════════════════════════════════════════
    /// Total documents processed
    pub documents_processed_total: Counter,

    /// Total embeddings generated
    pub embeddings_generated_total: Counter,

    /// Average embedding dimension
    pub avg_embedding_dimension: Gauge,

    /// Quantization compression ratio
    pub quantization_compression_ratio: Gauge,

    /// Index build time in seconds
    pub index_build_time_seconds: Histogram,

    /// Index size in bytes
    pub index_size_bytes: Gauge,

    // ═══════════════════════════════════════════════════════════════════════
    // Error Metrics
    // ═══════════════════════════════════════════════════════════════════════
    /// Total errors by type
    pub errors_by_type_total: CounterVec,

    /// Error rate percentage
    pub error_rate_percent: Gauge,

    /// Last error timestamp
    pub last_error_timestamp: Gauge,
>>>>>>> 09c343e7a158de5fc41739e0d5798846bca10a44
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
<<<<<<< HEAD
=======

            // Performance metrics
            cpu_usage_percent: Gauge::new("vectorizer_cpu_usage_percent", "CPU usage percentage")
                .unwrap(),

            disk_io_ops_per_second: Gauge::new(
                "vectorizer_disk_io_ops_per_second",
                "Disk I/O operations per second",
            )
            .unwrap(),

            network_io_bytes_per_second: Gauge::new(
                "vectorizer_network_io_bytes_per_second",
                "Network I/O bytes per second",
            )
            .unwrap(),

            db_connection_pool_size: Gauge::new(
                "vectorizer_db_connection_pool_size",
                "Database connection pool size",
            )
            .unwrap(),

            db_active_connections: Gauge::new(
                "vectorizer_db_active_connections",
                "Active database connections",
            )
            .unwrap(),

            db_query_latency_seconds: HistogramVec::new(
                HistogramOpts::new(
                    "vectorizer_db_query_latency_seconds",
                    "Database query latency in seconds",
                )
                .buckets(vec![
                    0.001, 0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0, 2.5, 5.0,
                ]),
                &["query_type", "collection"],
            )
            .unwrap(),

            cache_hit_ratio: Gauge::new("vectorizer_cache_hit_ratio", "Cache hit ratio").unwrap(),

            cache_size_bytes: Gauge::new("vectorizer_cache_size_bytes", "Cache size in bytes")
                .unwrap(),

            cache_evictions_total: Counter::new(
                "vectorizer_cache_evictions_total",
                "Cache eviction count",
            )
            .unwrap(),

            // Business metrics
            documents_processed_total: Counter::new(
                "vectorizer_documents_processed_total",
                "Total documents processed",
            )
            .unwrap(),

            embeddings_generated_total: Counter::new(
                "vectorizer_embeddings_generated_total",
                "Total embeddings generated",
            )
            .unwrap(),

            avg_embedding_dimension: Gauge::new(
                "vectorizer_avg_embedding_dimension",
                "Average embedding dimension",
            )
            .unwrap(),

            quantization_compression_ratio: Gauge::new(
                "vectorizer_quantization_compression_ratio",
                "Quantization compression ratio",
            )
            .unwrap(),

            index_build_time_seconds: Histogram::with_opts(
                HistogramOpts::new(
                    "vectorizer_index_build_time_seconds",
                    "Index build time in seconds",
                )
                .buckets(vec![
                    0.1, 0.5, 1.0, 2.5, 5.0, 10.0, 25.0, 50.0, 100.0, 250.0, 500.0,
                ]),
            )
            .unwrap(),

            index_size_bytes: Gauge::new("vectorizer_index_size_bytes", "Index size in bytes")
                .unwrap(),

            // Error metrics
            errors_by_type_total: CounterVec::new(
                Opts::new("vectorizer_errors_by_type_total", "Total errors by type"),
                &["error_type", "severity"],
            )
            .unwrap(),

            error_rate_percent: Gauge::new(
                "vectorizer_error_rate_percent",
                "Error rate percentage",
            )
            .unwrap(),

            last_error_timestamp: Gauge::new(
                "vectorizer_last_error_timestamp",
                "Last error timestamp",
            )
            .unwrap(),
>>>>>>> 09c343e7a158de5fc41739e0d5798846bca10a44
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

<<<<<<< HEAD
        Ok(())
    }
=======
        // Performance metrics
        registry.register(Box::new(self.cpu_usage_percent.clone()))?;
        registry.register(Box::new(self.disk_io_ops_per_second.clone()))?;
        registry.register(Box::new(self.network_io_bytes_per_second.clone()))?;
        registry.register(Box::new(self.db_connection_pool_size.clone()))?;
        registry.register(Box::new(self.db_active_connections.clone()))?;
        registry.register(Box::new(self.db_query_latency_seconds.clone()))?;
        registry.register(Box::new(self.cache_hit_ratio.clone()))?;
        registry.register(Box::new(self.cache_size_bytes.clone()))?;
        registry.register(Box::new(self.cache_evictions_total.clone()))?;

        // Business metrics
        registry.register(Box::new(self.documents_processed_total.clone()))?;
        registry.register(Box::new(self.embeddings_generated_total.clone()))?;
        registry.register(Box::new(self.avg_embedding_dimension.clone()))?;
        registry.register(Box::new(self.quantization_compression_ratio.clone()))?;
        registry.register(Box::new(self.index_build_time_seconds.clone()))?;
        registry.register(Box::new(self.index_size_bytes.clone()))?;

        // Error metrics
        registry.register(Box::new(self.errors_by_type_total.clone()))?;
        registry.register(Box::new(self.error_rate_percent.clone()))?;
        registry.register(Box::new(self.last_error_timestamp.clone()))?;

        Ok(())
    }

    /// Record a search operation with timing
    pub fn record_search(
        &self,
        collection: &str,
        search_type: &str,
        duration: f64,
        result_count: usize,
        success: bool,
    ) {
        let status = if success { "success" } else { "error" };

        self.search_requests_total
            .with_label_values(&[collection, search_type, status])
            .inc();

        self.search_latency_seconds
            .with_label_values(&[collection, search_type])
            .observe(duration);

        self.search_results_count
            .with_label_values(&[collection, search_type])
            .observe(result_count as f64);
    }

    /// Record an insert operation with timing
    pub fn record_insert(&self, collection: &str, duration: f64, success: bool) {
        let status = if success { "success" } else { "error" };

        self.insert_requests_total
            .with_label_values(&[collection, status])
            .inc();

        self.insert_latency_seconds.observe(duration);
    }

    /// Record a database query with timing
    pub fn record_db_query(&self, query_type: &str, collection: &str, duration: f64) {
        self.db_query_latency_seconds
            .with_label_values(&[query_type, collection])
            .observe(duration);
    }

    /// Record an API error
    pub fn record_api_error(&self, endpoint: &str, error_type: &str, status_code: u16) {
        self.api_errors_total
            .with_label_values(&[endpoint, error_type, &status_code.to_string()])
            .inc();
    }

    /// Record an error by type and severity
    pub fn record_error(&self, error_type: &str, severity: &str) {
        self.errors_by_type_total
            .with_label_values(&[error_type, severity])
            .inc();

        // Update last error timestamp
        self.last_error_timestamp
            .set(chrono::Utc::now().timestamp() as f64);
    }

    /// Update cache metrics
    pub fn update_cache_metrics(&self, hit: bool, cache_type: &str, size_bytes: u64) {
        let result = if hit { "hit" } else { "miss" };

        self.cache_requests_total
            .with_label_values(&[cache_type, result])
            .inc();

        self.cache_size_bytes.set(size_bytes as f64);
    }

    /// Update system performance metrics
    pub fn update_system_metrics(
        &self,
        cpu_percent: f64,
        memory_bytes: u64,
        disk_io_ops: f64,
        network_bytes: f64,
    ) {
        self.cpu_usage_percent.set(cpu_percent);
        self.memory_usage_bytes.set(memory_bytes as f64);
        self.disk_io_ops_per_second.set(disk_io_ops);
        self.network_io_bytes_per_second.set(network_bytes);
    }

    /// Update business metrics
    pub fn update_business_metrics(
        &self,
        documents_processed: u64,
        embeddings_generated: u64,
        avg_dimension: f64,
        compression_ratio: f64,
    ) {
        self.documents_processed_total
            .inc_by(documents_processed as f64);
        self.embeddings_generated_total
            .inc_by(embeddings_generated as f64);
        self.avg_embedding_dimension.set(avg_dimension);
        self.quantization_compression_ratio.set(compression_ratio);
    }

    /// Record index build time
    pub fn record_index_build(&self, duration: f64, size_bytes: u64) {
        self.index_build_time_seconds.observe(duration);
        self.index_size_bytes.set(size_bytes as f64);
    }

    /// Update collection and vector counts
    pub fn update_collection_metrics(&self, collections: usize, vectors: usize) {
        self.collections_total.set(collections as f64);
        self.vectors_total.set(vectors as f64);
    }
>>>>>>> 09c343e7a158de5fc41739e0d5798846bca10a44
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
<<<<<<< HEAD
=======

    #[test]
    fn test_record_search() {
        let metrics = Metrics::new();

        // Test successful search
        metrics.record_search("test_collection", "basic", 0.1, 5, true);

        assert_eq!(
            metrics
                .search_requests_total
                .with_label_values(&["test_collection", "basic", "success"])
                .get(),
            1.0
        );

        // Test failed search
        metrics.record_search("test_collection", "basic", 0.05, 0, false);

        assert_eq!(
            metrics
                .search_requests_total
                .with_label_values(&["test_collection", "basic", "error"])
                .get(),
            1.0
        );
    }

    #[test]
    fn test_record_insert() {
        let metrics = Metrics::new();

        // Test successful insert
        metrics.record_insert("test_collection", 0.2, true);

        assert_eq!(
            metrics
                .insert_requests_total
                .with_label_values(&["test_collection", "success"])
                .get(),
            1.0
        );
    }

    #[test]
    fn test_record_api_error() {
        let metrics = Metrics::new();

        metrics.record_api_error("/api/search", "validation_error", 400);

        assert_eq!(
            metrics
                .api_errors_total
                .with_label_values(&["/api/search", "validation_error", "400"])
                .get(),
            1.0
        );
    }

    #[test]
    fn test_record_error() {
        let metrics = Metrics::new();

        metrics.record_error("validation_error", "warning");

        assert_eq!(
            metrics
                .errors_by_type_total
                .with_label_values(&["validation_error", "warning"])
                .get(),
            1.0
        );

        // Check that last error timestamp was updated
        assert!(metrics.last_error_timestamp.get() > 0.0);
    }

    #[test]
    fn test_update_cache_metrics() {
        let metrics = Metrics::new();

        // Test cache hit
        metrics.update_cache_metrics(true, "query_cache", 1024);

        assert_eq!(
            metrics
                .cache_requests_total
                .with_label_values(&["query_cache", "hit"])
                .get(),
            1.0
        );

        assert_eq!(metrics.cache_size_bytes.get(), 1024.0);

        // Test cache miss
        metrics.update_cache_metrics(false, "query_cache", 2048);

        assert_eq!(
            metrics
                .cache_requests_total
                .with_label_values(&["query_cache", "miss"])
                .get(),
            1.0
        );
    }

    #[test]
    fn test_update_system_metrics() {
        let metrics = Metrics::new();

        metrics.update_system_metrics(75.5, 1024 * 1024 * 100, 150.0, 1024.0 * 1024.0);

        assert_eq!(metrics.cpu_usage_percent.get(), 75.5);
        assert_eq!(metrics.memory_usage_bytes.get(), 1024.0 * 1024.0 * 100.0);
        assert_eq!(metrics.disk_io_ops_per_second.get(), 150.0);
        assert_eq!(metrics.network_io_bytes_per_second.get(), 1024.0 * 1024.0);
    }

    #[test]
    fn test_update_business_metrics() {
        let metrics = Metrics::new();

        metrics.update_business_metrics(1000, 5000, 512.0, 0.5);

        assert_eq!(metrics.documents_processed_total.get(), 1000.0);
        assert_eq!(metrics.embeddings_generated_total.get(), 5000.0);
        assert_eq!(metrics.avg_embedding_dimension.get(), 512.0);
        assert_eq!(metrics.quantization_compression_ratio.get(), 0.5);
    }

    #[test]
    fn test_record_index_build() {
        let metrics = Metrics::new();

        metrics.record_index_build(10.5, 1024 * 1024 * 50);

        assert_eq!(metrics.index_size_bytes.get(), 1024.0 * 1024.0 * 50.0);
    }

    #[test]
    fn test_update_collection_metrics() {
        let metrics = Metrics::new();

        metrics.update_collection_metrics(5, 10000);

        assert_eq!(metrics.collections_total.get(), 5.0);
        assert_eq!(metrics.vectors_total.get(), 10000.0);
    }
>>>>>>> 09c343e7a158de5fc41739e0d5798846bca10a44
}
