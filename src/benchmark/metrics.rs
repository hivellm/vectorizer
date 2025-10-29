//! Benchmark Metrics and Results
//!
//! Provides data structures for collecting and analyzing benchmark performance metrics.

use std::time::{Duration, Instant};

use serde::{Deserialize, Serialize};

/// Performance metrics for a single operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OperationMetrics {
    /// Name of the operation
    pub operation: String,
    /// Configuration details
    pub config: String,
    /// Total number of operations performed
    pub total_operations: usize,
    /// Total time for all operations
    pub total_time_ms: f64,
    /// Throughput in operations per second
    pub throughput_ops_per_sec: f64,
    /// Average latency in microseconds
    pub avg_latency_us: f64,
    /// P50 latency in microseconds
    pub p50_latency_us: f64,
    /// P95 latency in microseconds
    pub p95_latency_us: f64,
    /// P99 latency in microseconds
    pub p99_latency_us: f64,
    /// Minimum latency in microseconds
    pub min_latency_us: f64,
    /// Maximum latency in microseconds
    pub max_latency_us: f64,
    /// Memory usage before operation (MB)
    pub memory_before_mb: f64,
    /// Memory usage after operation (MB)
    pub memory_after_mb: f64,
    /// Memory delta (MB)
    pub memory_delta_mb: f64,
    /// CPU usage percentage
    pub cpu_usage_percent: f64,
    /// Additional custom metrics
    pub custom_metrics: std::collections::HashMap<String, f64>,
}

impl OperationMetrics {
    /// Create new operation metrics
    pub fn new(operation: String, config: String) -> Self {
        Self {
            operation,
            config,
            total_operations: 0,
            total_time_ms: 0.0,
            throughput_ops_per_sec: 0.0,
            avg_latency_us: 0.0,
            p50_latency_us: 0.0,
            p95_latency_us: 0.0,
            p99_latency_us: 0.0,
            min_latency_us: 0.0,
            max_latency_us: 0.0,
            memory_before_mb: 0.0,
            memory_after_mb: 0.0,
            memory_delta_mb: 0.0,
            cpu_usage_percent: 0.0,
            custom_metrics: std::collections::HashMap::new(),
        }
    }

    /// Calculate metrics from latency measurements
    pub fn from_latencies(
        operation: String,
        config: String,
        latencies: Vec<f64>,
        memory_before_mb: f64,
        memory_after_mb: f64,
    ) -> Self {
        if latencies.is_empty() {
            return Self::new(operation, config);
        }

        let total_operations = latencies.len();
        let total_time_ms = latencies.iter().sum::<f64>() / 1000.0; // Convert μs to ms
        let throughput_ops_per_sec = if total_time_ms > 0.0 {
            total_operations as f64 / (total_time_ms / 1000.0)
        } else {
            0.0
        };

        let mut sorted_latencies = latencies.clone();
        sorted_latencies.sort_by(|a, b| a.partial_cmp(b).unwrap());

        let avg_latency_us = latencies.iter().sum::<f64>() / latencies.len() as f64;
        let p50_latency_us = percentile(&sorted_latencies, 50);
        let p95_latency_us = percentile(&sorted_latencies, 95);
        let p99_latency_us = percentile(&sorted_latencies, 99);
        let min_latency_us = sorted_latencies[0];
        let max_latency_us = sorted_latencies[sorted_latencies.len() - 1];

        Self {
            operation,
            config,
            total_operations,
            total_time_ms,
            throughput_ops_per_sec,
            avg_latency_us,
            p50_latency_us,
            p95_latency_us,
            p99_latency_us,
            min_latency_us,
            max_latency_us,
            memory_before_mb,
            memory_after_mb,
            memory_delta_mb: memory_after_mb - memory_before_mb,
            cpu_usage_percent: 0.0, // Will be set by system monitor
            custom_metrics: std::collections::HashMap::new(),
        }
    }

    /// Add a custom metric
    pub fn add_custom_metric(&mut self, name: String, value: f64) {
        self.custom_metrics.insert(name, value);
    }

    /// Get a custom metric
    pub fn get_custom_metric(&self, name: &str) -> Option<f64> {
        self.custom_metrics.get(name).copied()
    }
}

/// Comprehensive performance metrics for a benchmark run
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMetrics {
    /// Benchmark configuration used
    pub config: String,
    /// Dataset size (number of vectors)
    pub dataset_size: usize,
    /// Vector dimension
    pub dimension: usize,
    /// Timestamp when benchmark was run
    pub timestamp: String,
    /// Duration of the entire benchmark
    pub total_duration_ms: f64,
    /// System information
    pub system_info: SystemInfo,
    /// Individual operation metrics
    pub operations: std::collections::HashMap<String, OperationMetrics>,
    /// Overall benchmark summary
    pub summary: BenchmarkSummary,
}

/// System information captured during benchmark
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemInfo {
    /// CPU model/name
    pub cpu_model: String,
    /// Number of CPU cores
    pub cpu_cores: usize,
    /// Total memory in MB
    pub total_memory_mb: f64,
    /// Available memory in MB
    pub available_memory_mb: f64,
    /// Operating system
    pub os: String,
    /// Rust version
    pub rust_version: String,
    /// Vectorizer version
    pub vectorizer_version: String,
}

/// Benchmark summary statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkSummary {
    /// Total operations performed
    pub total_operations: usize,
    /// Overall throughput (ops/sec)
    pub overall_throughput: f64,
    /// Average latency across all operations (μs)
    pub avg_latency_us: f64,
    /// P95 latency across all operations (μs)
    pub p95_latency_us: f64,
    /// P99 latency across all operations (μs)
    pub p99_latency_us: f64,
    /// Peak memory usage (MB)
    pub peak_memory_mb: f64,
    /// Average CPU usage (%)
    pub avg_cpu_usage: f64,
    /// Success rate (0.0 to 1.0)
    pub success_rate: f64,
}

impl PerformanceMetrics {
    /// Create new performance metrics
    pub fn new(config: String, dataset_size: usize, dimension: usize) -> Self {
        Self {
            config,
            dataset_size,
            dimension,
            timestamp: chrono::Utc::now()
                .format("%Y-%m-%d %H:%M:%S UTC")
                .to_string(),
            total_duration_ms: 0.0,
            system_info: SystemInfo::default(),
            operations: std::collections::HashMap::new(),
            summary: BenchmarkSummary::default(),
        }
    }

    /// Add operation metrics
    pub fn add_operation(&mut self, name: String, metrics: OperationMetrics) {
        self.operations.insert(name, metrics);
        self.update_summary();
    }

    /// Update summary statistics
    fn update_summary(&mut self) {
        if self.operations.is_empty() {
            return;
        }

        let mut total_operations = 0;
        let mut total_latencies = Vec::new();
        let mut peak_memory: f64 = 0.0;
        let mut total_cpu = 0.0;
        let mut operation_count = 0;

        for metrics in self.operations.values() {
            total_operations += metrics.total_operations;
            peak_memory = peak_memory.max(metrics.memory_after_mb);
            total_cpu += metrics.cpu_usage_percent;
            operation_count += 1;

            // Add individual latencies to overall distribution
            // This is a simplified approach - in practice you might want to store
            // individual latency measurements for more accurate percentiles
            total_latencies.push(metrics.avg_latency_us);
        }

        let overall_throughput = total_operations as f64 / (self.total_duration_ms / 1000.0);
        let avg_latency = total_latencies.iter().sum::<f64>() / total_latencies.len() as f64;
        let mut sorted_latencies = total_latencies.clone();
        sorted_latencies.sort_by(|a, b| a.partial_cmp(b).unwrap());

        self.summary = BenchmarkSummary {
            total_operations,
            overall_throughput,
            avg_latency_us: avg_latency,
            p95_latency_us: percentile(&sorted_latencies, 95),
            p99_latency_us: percentile(&sorted_latencies, 99),
            peak_memory_mb: peak_memory,
            avg_cpu_usage: if operation_count > 0 {
                total_cpu / operation_count as f64
            } else {
                0.0
            },
            success_rate: 1.0, // Assume 100% success unless specified otherwise
        };
    }

    /// Get operation metrics by name
    pub fn get_operation(&self, name: &str) -> Option<&OperationMetrics> {
        self.operations.get(name)
    }

    /// Get all operation names
    pub fn operation_names(&self) -> Vec<&String> {
        self.operations.keys().collect()
    }
}

/// Complete benchmark result including all metrics and metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkResult {
    /// Performance metrics
    pub metrics: PerformanceMetrics,
    /// Benchmark configuration used
    pub benchmark_config: crate::benchmark::BenchmarkConfig,
    /// Test data information
    pub test_data_info: TestDataInfo,
    /// Quality metrics (if applicable)
    pub quality_metrics: Option<QualityMetrics>,
    /// Error information (if any)
    pub errors: Vec<String>,
    /// Additional metadata
    pub metadata: std::collections::HashMap<String, String>,
}

/// Test data information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestDataInfo {
    /// Number of vectors in test dataset
    pub vector_count: usize,
    /// Vector dimension
    pub dimension: usize,
    /// Data generation method
    pub generation_method: String,
    /// Data source (e.g., "synthetic", "real", "workspace")
    pub data_source: String,
    /// Generation time in seconds
    pub generation_time_sec: f64,
}

/// Quality metrics for search benchmarks
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityMetrics {
    /// Mean Average Precision
    pub map: f64,
    /// Mean Reciprocal Rank
    pub mrr: f64,
    /// Precision at 1
    pub precision_at_1: f64,
    /// Precision at 5
    pub precision_at_5: f64,
    /// Precision at 10
    pub precision_at_10: f64,
    /// Recall at 1
    pub recall_at_1: f64,
    /// Recall at 5
    pub recall_at_5: f64,
    /// Recall at 10
    pub recall_at_10: f64,
    /// Normalized Discounted Cumulative Gain at 10
    pub ndcg_at_10: f64,
}

impl Default for SystemInfo {
    fn default() -> Self {
        Self {
            cpu_model: "Unknown".to_string(),
            cpu_cores: num_cpus::get(),
            total_memory_mb: 0.0,
            available_memory_mb: 0.0,
            os: std::env::consts::OS.to_string(),
            rust_version: env!("CARGO_PKG_VERSION").to_string(),
            vectorizer_version: env!("CARGO_PKG_VERSION").to_string(),
        }
    }
}

impl Default for BenchmarkSummary {
    fn default() -> Self {
        Self {
            total_operations: 0,
            overall_throughput: 0.0,
            avg_latency_us: 0.0,
            p95_latency_us: 0.0,
            p99_latency_us: 0.0,
            peak_memory_mb: 0.0,
            avg_cpu_usage: 0.0,
            success_rate: 1.0,
        }
    }
}

impl Default for QualityMetrics {
    fn default() -> Self {
        Self {
            map: 0.0,
            mrr: 0.0,
            precision_at_1: 0.0,
            precision_at_5: 0.0,
            precision_at_10: 0.0,
            recall_at_1: 0.0,
            recall_at_5: 0.0,
            recall_at_10: 0.0,
            ndcg_at_10: 0.0,
        }
    }
}

/// Calculate percentile from sorted values
pub fn percentile(sorted_values: &[f64], p: usize) -> f64 {
    if sorted_values.is_empty() {
        return 0.0;
    }

    let idx = ((p as f64 / 100.0) * sorted_values.len() as f64) as usize;
    sorted_values[idx.min(sorted_values.len() - 1)]
}

/// Latency measurement helper
pub struct LatencyMeasurer {
    measurements: Vec<f64>,
    start_time: Option<Instant>,
}

impl LatencyMeasurer {
    /// Create new latency measurer
    pub fn new() -> Self {
        Self {
            measurements: Vec::new(),
            start_time: None,
        }
    }

    /// Start timing an operation
    pub fn start(&mut self) {
        self.start_time = Some(Instant::now());
    }

    /// End timing and record the measurement
    pub fn end(&mut self) -> Option<f64> {
        if let Some(start) = self.start_time {
            let elapsed_us = start.elapsed().as_micros() as f64;
            self.measurements.push(elapsed_us);
            self.start_time = None;
            Some(elapsed_us)
        } else {
            None
        }
    }

    /// Get all measurements
    pub fn measurements(&self) -> &[f64] {
        &self.measurements
    }

    /// Clear all measurements
    pub fn clear(&mut self) {
        self.measurements.clear();
        self.start_time = None;
    }

    /// Get number of measurements
    pub fn count(&self) -> usize {
        self.measurements.len()
    }
}

impl Default for LatencyMeasurer {
    fn default() -> Self {
        Self::new()
    }
}

/// Throughput calculator
pub struct ThroughputCalculator {
    start_time: Instant,
    operation_count: usize,
}

impl ThroughputCalculator {
    /// Create new throughput calculator
    pub fn new() -> Self {
        Self {
            start_time: Instant::now(),
            operation_count: 0,
        }
    }

    /// Record an operation
    pub fn record_operation(&mut self) {
        self.operation_count += 1;
    }

    /// Get current throughput (ops/sec)
    pub fn current_throughput(&self) -> f64 {
        let elapsed_sec = self.start_time.elapsed().as_secs_f64();
        if elapsed_sec > 0.0 {
            self.operation_count as f64 / elapsed_sec
        } else {
            0.0
        }
    }

    /// Get total operations
    pub fn total_operations(&self) -> usize {
        self.operation_count
    }

    /// Get elapsed time
    pub fn elapsed_time(&self) -> Duration {
        self.start_time.elapsed()
    }

    /// Reset the calculator
    pub fn reset(&mut self) {
        self.start_time = Instant::now();
        self.operation_count = 0;
    }
}

impl Default for ThroughputCalculator {
    fn default() -> Self {
        Self::new()
    }
}
