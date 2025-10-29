//! Benchmark Runner
//!
//! Provides utilities for running benchmarks and collecting performance metrics.

use std::time::Instant;

use crate::benchmark::data_generator::TestData;
use crate::benchmark::metrics::{LatencyMeasurer, ThroughputCalculator};
use crate::benchmark::{BenchmarkConfig, OperationMetrics, PerformanceMetrics, TestDataGenerator};
use crate::db::{OptimizedHnswConfig, OptimizedHnswIndex};
use crate::models::DistanceMetric;

/// Benchmark runner for executing various benchmark scenarios
pub struct BenchmarkRunner {
    config: BenchmarkConfig,
    system_monitor: Option<crate::benchmark::SystemMonitor>,
}

impl BenchmarkRunner {
    /// Create new benchmark runner
    pub fn new(config: BenchmarkConfig) -> Self {
        Self {
            config,
            system_monitor: None,
        }
    }

    /// Enable system monitoring
    pub fn with_system_monitoring(mut self) -> Self {
        self.system_monitor = Some(crate::benchmark::SystemMonitor::new());
        self
    }

    /// Run search performance benchmark
    pub fn benchmark_search(
        &mut self,
        test_data: &TestData,
        k_values: &[usize],
    ) -> Result<PerformanceMetrics, Box<dyn std::error::Error>> {
        let mut metrics = PerformanceMetrics::new(
            format!("search_benchmark_dim_{}", test_data.dimension()),
            test_data.vector_count(),
            test_data.dimension(),
        );

        // Build HNSW index
        let hnsw_config = OptimizedHnswConfig {
            max_connections: self.config.hnsw_config.max_connections,
            max_connections_0: self.config.hnsw_config.max_connections_0,
            ef_construction: self.config.hnsw_config.ef_construction,
            distance_metric: DistanceMetric::Cosine,
            parallel: self.config.hnsw_config.parallel,
            initial_capacity: test_data.vector_count(),
            batch_size: self.config.hnsw_config.batch_size,
            ..Default::default()
        };

        let index = OptimizedHnswIndex::new(test_data.dimension(), hnsw_config)?;

        // Insert vectors in batches
        let batch_size = self.config.hnsw_config.batch_size;
        for batch in test_data.vectors().chunks(batch_size) {
            let batch_vectors: Vec<(String, Vec<f32>)> = batch.to_vec();
            index.batch_add(batch_vectors)?;
        }

        index.optimize()?;

        // Benchmark search for different k values
        for &k in k_values {
            let operation_name = format!("search_k_{}", k);
            let operation_metrics = self.benchmark_search_operation(&index, test_data, k)?;
            metrics.add_operation(operation_name, operation_metrics);
        }

        metrics.total_duration_ms =
            metrics.summary.total_operations as f64 / metrics.summary.overall_throughput * 1000.0;
        Ok(metrics)
    }

    /// Run insert performance benchmark
    pub fn benchmark_insert(
        &mut self,
        test_data: &TestData,
        batch_sizes: &[usize],
    ) -> Result<PerformanceMetrics, Box<dyn std::error::Error>> {
        let mut metrics = PerformanceMetrics::new(
            format!("insert_benchmark_dim_{}", test_data.dimension()),
            test_data.vector_count(),
            test_data.dimension(),
        );

        for &batch_size in batch_sizes {
            let operation_name = format!("insert_batch_{}", batch_size);
            let operation_metrics = self.benchmark_insert_operation(test_data, batch_size)?;
            metrics.add_operation(operation_name, operation_metrics);
        }

        metrics.total_duration_ms =
            metrics.summary.total_operations as f64 / metrics.summary.overall_throughput * 1000.0;
        Ok(metrics)
    }

    /// Run update performance benchmark
    pub fn benchmark_update(
        &mut self,
        test_data: &TestData,
        batch_sizes: &[usize],
    ) -> Result<PerformanceMetrics, Box<dyn std::error::Error>> {
        let mut metrics = PerformanceMetrics::new(
            format!("update_benchmark_dim_{}", test_data.dimension()),
            test_data.vector_count(),
            test_data.dimension(),
        );

        // Build initial index
        let hnsw_config = OptimizedHnswConfig {
            max_connections: self.config.hnsw_config.max_connections,
            max_connections_0: self.config.hnsw_config.max_connections_0,
            ef_construction: self.config.hnsw_config.ef_construction,
            distance_metric: DistanceMetric::Cosine,
            parallel: self.config.hnsw_config.parallel,
            initial_capacity: test_data.vector_count(),
            batch_size: self.config.hnsw_config.batch_size,
            ..Default::default()
        };

        let index = OptimizedHnswIndex::new(test_data.dimension(), hnsw_config)?;

        // Insert initial vectors
        let batch_size = self.config.hnsw_config.batch_size;
        for batch in test_data.vectors().chunks(batch_size) {
            index.batch_add(batch.to_vec())?;
        }

        for &batch_size in batch_sizes {
            let operation_name = format!("update_batch_{}", batch_size);
            let operation_metrics =
                self.benchmark_update_operation(&index, test_data, batch_size)?;
            metrics.add_operation(operation_name, operation_metrics);
        }

        metrics.total_duration_ms =
            metrics.summary.total_operations as f64 / metrics.summary.overall_throughput * 1000.0;
        Ok(metrics)
    }

    /// Run delete performance benchmark
    pub fn benchmark_delete(
        &mut self,
        test_data: &TestData,
        batch_sizes: &[usize],
    ) -> Result<PerformanceMetrics, Box<dyn std::error::Error>> {
        let mut metrics = PerformanceMetrics::new(
            format!("delete_benchmark_dim_{}", test_data.dimension()),
            test_data.vector_count(),
            test_data.dimension(),
        );

        for &batch_size in batch_sizes {
            let operation_name = format!("delete_batch_{}", batch_size);
            let operation_metrics = self.benchmark_delete_operation(test_data, batch_size)?;
            metrics.add_operation(operation_name, operation_metrics);
        }

        metrics.total_duration_ms =
            metrics.summary.total_operations as f64 / metrics.summary.overall_throughput * 1000.0;
        Ok(metrics)
    }

    /// Run concurrent mixed workload benchmark
    pub fn benchmark_concurrent_mixed(
        &mut self,
        test_data: &TestData,
        num_threads: usize,
        operations_per_thread: usize,
    ) -> Result<PerformanceMetrics, Box<dyn std::error::Error>> {
        let mut metrics = PerformanceMetrics::new(
            format!("concurrent_mixed_benchmark_dim_{}", test_data.dimension()),
            test_data.vector_count(),
            test_data.dimension(),
        );

        let operation_metrics =
            self.benchmark_concurrent_operation(test_data, num_threads, operations_per_thread)?;

        metrics.add_operation("concurrent_mixed".to_string(), operation_metrics);
        metrics.total_duration_ms =
            metrics.summary.total_operations as f64 / metrics.summary.overall_throughput * 1000.0;
        Ok(metrics)
    }

    /// Run comprehensive benchmark suite
    pub fn run_comprehensive_benchmark(
        &mut self,
        test_data: &TestData,
    ) -> Result<Vec<PerformanceMetrics>, Box<dyn std::error::Error>> {
        let mut all_metrics = Vec::new();

        // Search benchmarks
        let search_k_values = vec![1, 10, 100];
        let search_metrics = self.benchmark_search(test_data, &search_k_values)?;
        all_metrics.push(search_metrics);

        // Insert benchmarks
        let insert_batch_sizes = vec![1, 100, 1000];
        let insert_metrics = self.benchmark_insert(test_data, &insert_batch_sizes)?;
        all_metrics.push(insert_metrics);

        // Update benchmarks
        let update_batch_sizes = vec![1, 100, 1000];
        let update_metrics = self.benchmark_update(test_data, &update_batch_sizes)?;
        all_metrics.push(update_metrics);

        // Delete benchmarks
        let delete_batch_sizes = vec![1, 100, 1000];
        let delete_metrics = self.benchmark_delete(test_data, &delete_batch_sizes)?;
        all_metrics.push(delete_metrics);

        // Concurrent mixed workload
        let concurrent_metrics = self.benchmark_concurrent_mixed(test_data, 4, 1000)?;
        all_metrics.push(concurrent_metrics);

        Ok(all_metrics)
    }

    // Private helper methods

    fn benchmark_search_operation(
        &mut self,
        index: &OptimizedHnswIndex,
        test_data: &TestData,
        k: usize,
    ) -> Result<OperationMetrics, Box<dyn std::error::Error>> {
        let operation_name = format!("Search k={}", k);
        let config = format!(
            "{} queries on {} vectors",
            test_data.queries().len(),
            test_data.vector_count()
        );

        let mut measurer = LatencyMeasurer::new();
        let mut throughput_calc = ThroughputCalculator::new();

        // Warmup
        if !test_data.vectors().is_empty() {
            for _ in 0..3 {
                if let Some(query_vec) = test_data.vectors().first() {
                    let _ = index.search(&query_vec.1, k)?;
                }
            }
        }

        // Actual benchmark
        let mut latencies = Vec::new();
        let start_time = Instant::now();

        if !test_data.vectors().is_empty() {
            for (i, query) in test_data.queries().iter().enumerate() {
                let query_idx = i % test_data.vectors().len();
                let query_vec = &test_data.vectors()[query_idx].1;

                measurer.start();
                let _ = index.search(query_vec, k)?;
                if let Some(latency) = measurer.end() {
                    latencies.push(latency);
                    throughput_calc.record_operation();
                }
            }
        }

        let total_time = start_time.elapsed();
        let memory_before = 0.0; // Would get from system monitor
        let memory_after = 0.0; // Would get from system monitor

        Ok(OperationMetrics::from_latencies(
            operation_name,
            config,
            latencies,
            memory_before,
            memory_after,
        ))
    }

    fn benchmark_insert_operation(
        &mut self,
        test_data: &TestData,
        batch_size: usize,
    ) -> Result<OperationMetrics, Box<dyn std::error::Error>> {
        let operation_name = format!("Insert batch_size={}", batch_size);
        let config = format!(
            "Batch size: {}, parallel: {}",
            batch_size, self.config.hnsw_config.parallel
        );

        let mut measurer = LatencyMeasurer::new();
        let mut throughput_calc = ThroughputCalculator::new();

        let mut latencies = Vec::new();
        let start_time = Instant::now();

        for batch in test_data.vectors().chunks(batch_size) {
            let batch: &[(String, Vec<f32>)] = batch;
            let hnsw_config = OptimizedHnswConfig {
                max_connections: self.config.hnsw_config.max_connections,
                max_connections_0: self.config.hnsw_config.max_connections_0,
                ef_construction: self.config.hnsw_config.ef_construction,
                distance_metric: DistanceMetric::Cosine,
                parallel: self.config.hnsw_config.parallel,
                initial_capacity: batch.len(),
                batch_size,
                ..Default::default()
            };

            let index = OptimizedHnswIndex::new(test_data.dimension(), hnsw_config)?;

            measurer.start();
            let batch_vectors: Vec<(String, Vec<f32>)> = batch.to_vec();
            index.batch_add(batch_vectors)?;
            if let Some(latency) = measurer.end() {
                latencies.push(latency);
                throughput_calc.record_operation();
            }
        }

        let memory_before = 0.0;
        let memory_after = 0.0;

        Ok(OperationMetrics::from_latencies(
            operation_name,
            config,
            latencies,
            memory_before,
            memory_after,
        ))
    }

    fn benchmark_update_operation(
        &mut self,
        index: &OptimizedHnswIndex,
        test_data: &TestData,
        batch_size: usize,
    ) -> Result<OperationMetrics, Box<dyn std::error::Error>> {
        let operation_name = format!("Update batch_size={}", batch_size);
        let config = format!("Batch size: {}", batch_size);

        let mut measurer = LatencyMeasurer::new();
        let mut throughput_calc = ThroughputCalculator::new();

        let mut latencies = Vec::new();
        let start_time = Instant::now();

        for batch in test_data.vectors().chunks(batch_size) {
            let batch: &[(String, Vec<f32>)] = batch;
            measurer.start();

            // Update each vector in the batch
            for (id, vector) in batch {
                let mut modified_vector = vector.clone();
                // Add small modification
                for v in &mut modified_vector {
                    *v *= 1.01;
                }
                index.update(id, &modified_vector)?;
            }

            if let Some(latency) = measurer.end() {
                latencies.push(latency);
                throughput_calc.record_operation();
            }
        }

        let memory_before = 0.0;
        let memory_after = 0.0;

        Ok(OperationMetrics::from_latencies(
            operation_name,
            config,
            latencies,
            memory_before,
            memory_after,
        ))
    }

    fn benchmark_delete_operation(
        &mut self,
        test_data: &TestData,
        batch_size: usize,
    ) -> Result<OperationMetrics, Box<dyn std::error::Error>> {
        let operation_name = format!("Delete batch_size={}", batch_size);
        let config = format!("Batch size: {}", batch_size);

        let mut measurer = LatencyMeasurer::new();
        let mut throughput_calc = ThroughputCalculator::new();

        let mut latencies = Vec::new();
        let start_time = Instant::now();

        // Build index first
        let hnsw_config = OptimizedHnswConfig {
            max_connections: self.config.hnsw_config.max_connections,
            max_connections_0: self.config.hnsw_config.max_connections_0,
            ef_construction: self.config.hnsw_config.ef_construction,
            distance_metric: DistanceMetric::Cosine,
            parallel: self.config.hnsw_config.parallel,
            initial_capacity: test_data.vector_count(),
            batch_size: self.config.hnsw_config.batch_size,
            ..Default::default()
        };

        let index = OptimizedHnswIndex::new(test_data.dimension(), hnsw_config)?;

        // Insert vectors
        for batch in test_data
            .vectors()
            .chunks(self.config.hnsw_config.batch_size)
        {
            let batch_vectors: Vec<(String, Vec<f32>)> = batch.to_vec();
            index.batch_add(batch_vectors)?;
        }

        // Delete in batches
        for batch in test_data.vectors().chunks(batch_size) {
            measurer.start();

            for (id, _) in batch {
                index.remove(id)?;
            }

            if let Some(latency) = measurer.end() {
                latencies.push(latency);
                throughput_calc.record_operation();
            }
        }

        let memory_before = 0.0;
        let memory_after = 0.0;

        Ok(OperationMetrics::from_latencies(
            operation_name,
            config,
            latencies,
            memory_before,
            memory_after,
        ))
    }

    fn benchmark_concurrent_operation(
        &mut self,
        test_data: &TestData,
        num_threads: usize,
        operations_per_thread: usize,
    ) -> Result<OperationMetrics, Box<dyn std::error::Error>> {
        let operation_name = "Concurrent Mixed".to_string();
        let config = format!(
            "{} threads, {} ops/thread",
            num_threads, operations_per_thread
        );

        // Build shared index
        let hnsw_config = OptimizedHnswConfig {
            max_connections: self.config.hnsw_config.max_connections,
            max_connections_0: self.config.hnsw_config.max_connections_0,
            ef_construction: self.config.hnsw_config.ef_construction,
            distance_metric: DistanceMetric::Cosine,
            parallel: self.config.hnsw_config.parallel,
            initial_capacity: test_data.vector_count() + operations_per_thread * num_threads,
            batch_size: self.config.hnsw_config.batch_size,
            ..Default::default()
        };

        let index =
            std::sync::Arc::new(OptimizedHnswIndex::new(test_data.dimension(), hnsw_config)?);

        // Insert initial vectors
        for batch in test_data
            .vectors()
            .chunks(self.config.hnsw_config.batch_size)
        {
            let batch_vectors: Vec<(String, Vec<f32>)> = batch.to_vec();
            index.batch_add(batch_vectors)?;
        }

        let mut latencies = Vec::new();
        let start_time = Instant::now();

        // Run concurrent operations
        let handles: Vec<_> = (0..num_threads)
            .map(|thread_id| {
                let index = std::sync::Arc::clone(&index);
                let test_data = test_data.clone();
                std::thread::spawn(move || {
                    let mut thread_latencies = Vec::new();

                    for i in 0..operations_per_thread {
                        let operation_type = i % 10;
                        let start = Instant::now();

                        match operation_type {
                            0..=6 => {
                                // 70% search operations
                                let query_idx = (thread_id * operations_per_thread + i)
                                    % test_data.vectors().len();
                                let query_vec = &test_data.vectors()[query_idx].1;
                                let _ = index.search(query_vec, 10);
                            }
                            7..=8 => {
                                // 20% insert operations
                                let id = format!("concurrent_{}_{}", thread_id, i);
                                let vector =
                                    test_data.vectors()[i % test_data.vectors().len()].1.clone();
                                let _ = index.add(id, vector);
                            }
                            _ => {
                                // 10% delete operations
                                if i < test_data.vectors().len() {
                                    let id = &test_data.vectors()[i].0;
                                    let _ = index.remove(id);
                                }
                            }
                        }

                        let elapsed = start.elapsed().as_micros() as f64;
                        thread_latencies.push(elapsed);
                    }

                    thread_latencies
                })
            })
            .collect();

        // Collect results
        for handle in handles {
            let thread_latencies = handle.join().unwrap();
            latencies.extend(thread_latencies);
        }

        let total_time = start_time.elapsed();
        let memory_before = 0.0;
        let memory_after = 0.0;

        Ok(OperationMetrics::from_latencies(
            operation_name,
            config,
            latencies,
            memory_before,
            memory_after,
        ))
    }
}
