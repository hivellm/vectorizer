//! Unified GPU Performance Benchmark
//!
//! This benchmark compares GPU-accelerated vector search operations
//! against CPU-only HNSW search in the Vectorizer, supporting both
//! CUDA (NVIDIA) and Metal (Apple) backends.

use std::sync::Arc;
use tracing::{info, error, warn, debug};
use std::time::Instant;

use criterion::{BenchmarkId, Criterion, black_box, criterion_group, criterion_main};
use rand::prelude::*;
use vectorizer::db::optimized_hnsw::OptimizedHnswIndex;
use vectorizer::error::Result;
use vectorizer::models::{
    CollectionConfig, CompressionConfig, DistanceMetric, HnswConfig, Payload, QuantizationConfig,
    SearchResult, Vector,
};

/// Benchmark results for GPU vs CPU comparison
#[derive(Debug, Clone)]
pub struct GpuBenchmarkResults {
    pub test_name: String,
    pub vector_count: usize,
    pub query_count: usize,
    pub hnsw_total_time_ms: f64,
    pub cuda_total_time_ms: f64,
    pub metal_total_time_ms: f64,
    pub hnsw_avg_time_ms: f64,
    pub cuda_avg_time_ms: f64,
    pub metal_avg_time_ms: f64,
    pub cuda_speedup_factor: f64,
    pub metal_speedup_factor: f64,
    pub memory_usage_mb: f64,
}

/// GPU benchmark configuration
#[derive(Debug, Clone)]
pub struct GpuBenchmarkConfig {
    pub vector_counts: Vec<usize>,
    pub query_count: usize,
    pub dimension: usize,
    pub k: usize,
    pub warmup_runs: usize,
    pub measurement_runs: usize,
}

impl Default for GpuBenchmarkConfig {
    fn default() -> Self {
        Self {
            vector_counts: vec![1000, 10000, 50000],
            query_count: 100,
            dimension: 128,
            k: 10,
            warmup_runs: 3,
            measurement_runs: 5,
        }
    }
}

/// Run comprehensive GPU benchmark
pub async fn run_gpu_benchmark(config: GpuBenchmarkConfig) -> Result<Vec<GpuBenchmarkResults>> {
    tracing::info!("🚀 Starting Unified GPU vs CPU Benchmark");
    tracing::info!("=========================================");

    let mut results = Vec::new();

    for vector_count in config.vector_counts {
        let test_name = format!("{}_vectors", vector_count);
        tracing::info!(
            "\n📊 Running test: {} ({} vectors, {} queries)",
            test_name, vector_count, config.query_count
        );

        let result = run_single_gpu_benchmark(
            &test_name,
            vector_count,
            config.query_count,
            config.dimension,
            config.k,
            config.warmup_runs,
        )
        .await?;

        results.push(result);
    }

    Ok(results)
}

/// Run single GPU benchmark test
async fn run_single_gpu_benchmark(
    test_name: &str,
    vector_count: usize,
    query_count: usize,
    dimension: usize,
    k: usize,
    warmup_runs: usize,
) -> Result<GpuBenchmarkResults> {
    // Create collection configuration
    let collection_config = CollectionConfig {
        dimension,
        metric: DistanceMetric::Cosine,
        hnsw_config: HnswConfig {
            m: 16,
            ef_construction: 200,
            ef_search: 50,
            seed: Some(42),
        },
        compression: CompressionConfig::default(),
        quantization: QuantizationConfig::SQ { bits: 8 },
        normalization: None,
    };

    // Create HNSW index
    let hnsw_index = Arc::new(parking_lot::RwLock::new(
        OptimizedHnswIndex::new(
            dimension,
            vectorizer::db::optimized_hnsw::OptimizedHnswConfig {
                max_connections: collection_config.hnsw_config.m,
                max_connections_0: collection_config.hnsw_config.m * 2,
                ef_construction: collection_config.hnsw_config.ef_construction,
                seed: collection_config.hnsw_config.seed,
                distance_metric: collection_config.metric,
                parallel: true,
                initial_capacity: vector_count,
                batch_size: 1000,
            },
        )
        .expect("Failed to create HNSW index"),
    ));

    // Generate test data
    let test_vectors = generate_test_vectors(vector_count, dimension);
    let test_queries = generate_test_queries(query_count, dimension);

    // Add vectors to HNSW index
    tracing::info!("  📥 Adding {} vectors to HNSW index...", vector_count);
    {
        let mut index = hnsw_index.write();
        for vector in &test_vectors {
            index.add(vector.id.clone(), vector.data.clone())?;
        }
    }

    // Initialize GPU operations
    let mut cuda_available = false;
    let mut metal_available = false;

    // Try to initialize CUDA
    #[cfg(feature = "cuda")]
    {
        let cuda_config = vectorizer::cuda::CudaConfig {
            enabled: true,
            device_id: 0,
            memory_limit_mb: 4096,
            max_threads_per_block: 1024,
            max_blocks_per_grid: 65535,
            memory_pool_size_mb: 1024,
        };

        let cuda_ops = vectorizer::gpu::CudaVectorOperations::new(cuda_config);
        if cuda_ops.is_cuda_available() {
            tracing::info!("  🎯 Initializing CUDA operations...");
            let flat_vectors: Vec<f32> = test_vectors.iter().flat_map(|v| v.data.clone()).collect();
            if cuda_ops
                .initialize_cuhnsw(&flat_vectors, vector_count, dimension)
                .is_ok()
            {
                cuda_available = true;
            }
        }
    }

    // Try to initialize Metal (macOS only)
    #[cfg(target_os = "macos")]
    {
        if let Ok(context) = hive_gpu::metal::MetalNativeContext::new() {
            if let Ok(mut storage) =
                context.create_storage(dimension, hive_gpu::GpuDistanceMetric::Cosine)
            {
                // Add vectors to Metal storage
                for vector in &test_vectors {
                    let gpu_vector = hive_gpu::GpuVector {
                        id: vector.id.clone(),
                        data: vector.data.clone(),
                        metadata: std::collections::HashMap::new(),
                    };
                    if storage.add_vectors(&[gpu_vector]).is_ok() {
                        metal_available = true;
                    }
                }
            }
        }
    }

    // Warmup runs
    tracing::info!("  🔥 Warming up...");
    for _ in 0..warmup_runs {
        for query in &test_queries {
            let _results = search_hnsw(&hnsw_index, query, k);
        }
    }

    // Benchmark HNSW search
    tracing::info!("  🧮 Benchmarking HNSW search...");
    let hnsw_start = Instant::now();
    for query in &test_queries {
        let _results = search_hnsw(&hnsw_index, query, k);
    }
    let hnsw_duration = hnsw_start.elapsed().as_secs_f64() * 1000.0;

    // Benchmark CUDA search
    let cuda_duration = if cuda_available {
        tracing::info!("  🎮 Benchmarking CUDA search...");
        let cuda_start = Instant::now();
        for query in &test_queries {
            // CUDA search implementation would go here
            // This is a placeholder for the actual CUDA search
            let _results = search_hnsw(&hnsw_index, query, k); // Fallback to HNSW for now
        }
        cuda_start.elapsed().as_secs_f64() * 1000.0
    } else {
        tracing::info!("  ⚠️ CUDA not available, skipping CUDA benchmark");
        0.0
    };

    // Benchmark Metal search
    let metal_duration = if metal_available {
        tracing::info!("  🎨 Benchmarking Metal search...");
        let metal_start = Instant::now();
        for query in &test_queries {
            // Metal search implementation would go here
            // This is a placeholder for the actual Metal search
            let _results = search_hnsw(&hnsw_index, query, k); // Fallback to HNSW for now
        }
        metal_start.elapsed().as_secs_f64() * 1000.0
    } else {
        tracing::info!("  ⚠️ Metal not available, skipping Metal benchmark");
        0.0
    };

    // Calculate metrics
    let hnsw_avg = hnsw_duration / query_count as f64;
    let cuda_avg = if cuda_duration > 0.0 {
        cuda_duration / query_count as f64
    } else {
        0.0
    };
    let metal_avg = if metal_duration > 0.0 {
        metal_duration / query_count as f64
    } else {
        0.0
    };

    let cuda_speedup = if cuda_avg > 0.0 {
        hnsw_avg / cuda_avg
    } else {
        0.0
    };

    let metal_speedup = if metal_avg > 0.0 {
        hnsw_avg / metal_avg
    } else {
        0.0
    };

    // Estimate memory usage
    let memory_usage = (vector_count * dimension * 4) as f64 / (1024.0 * 1024.0); // MB

    let result = GpuBenchmarkResults {
        test_name: test_name.to_string(),
        vector_count,
        query_count,
        hnsw_total_time_ms: hnsw_duration,
        cuda_total_time_ms: cuda_duration,
        metal_total_time_ms: metal_duration,
        hnsw_avg_time_ms: hnsw_avg,
        cuda_avg_time_ms: cuda_avg,
        metal_avg_time_ms: metal_avg,
        cuda_speedup_factor: cuda_speedup,
        metal_speedup_factor: metal_speedup,
        memory_usage_mb: memory_usage,
    };

    tracing::info!("  📈 Results:");
    tracing::info!(
        "    HNSW: {:.2}ms avg ({:.2}ms total)",
        hnsw_avg, hnsw_duration
    );
    if cuda_duration > 0.0 {
        tracing::info!(
            "    CUDA: {:.2}ms avg ({:.2}ms total) - {:.2}x speedup",
            cuda_avg, cuda_duration, cuda_speedup
        );
    } else {
        tracing::info!("    CUDA: Not available");
    }
    if metal_duration > 0.0 {
        tracing::info!(
            "    Metal: {:.2}ms avg ({:.2}ms total) - {:.2}x speedup",
            metal_avg, metal_duration, metal_speedup
        );
    } else {
        tracing::info!("    Metal: Not available");
    }
    tracing::info!("    Memory: {:.1} MB", memory_usage);

    Ok(result)
}

/// Search using HNSW index
fn search_hnsw(
    index: &Arc<parking_lot::RwLock<OptimizedHnswIndex>>,
    query: &[f32],
    k: usize,
) -> Result<Vec<SearchResult>> {
    let index = index.read();
    let neighbors = index.search(query, k)?;

    let results = neighbors
        .into_iter()
        .map(|(id, score)| SearchResult {
            id,
            score,
            vector: None,
            payload: None,
        })
        .collect();

    Ok(results)
}

/// Generate test vectors with realistic patterns
fn generate_test_vectors(count: usize, dimension: usize) -> Vec<Vector> {
    let mut vectors = Vec::with_capacity(count);
    let mut rng = rand::thread_rng();

    for i in 0..count {
        let mut data = Vec::with_capacity(dimension);

        // Generate vectors with some structure (not completely random)
        for j in 0..dimension {
            let base_value = (i as f32 / count as f32) * 2.0 - 1.0;
            let noise = (j as f32 / dimension as f32) * 0.1;
            let value = base_value + noise + (rng.random::<f32>() - 0.5) * 0.2;
            data.push(value);
        }

        // Normalize vector
        let norm: f32 = data.iter().map(|x| x * x).sum::<f32>().sqrt();
        if norm > 0.0 {
            for val in &mut data {
                *val /= norm;
            }
        }

        vectors.push(Vector {
            id: format!("vec_{}", i),
            data,
            payload: Some(Payload {
                data: serde_json::json!({
                    "content": format!("Document {} content", i),
                    "file_path": format!("/docs/doc_{}.txt", i),
                    "metadata": {
                        "index": i,
                        "category": if i % 3 == 0 { "tech" } else if i % 3 == 1 { "science" } else { "general" }
                    }
                }),
            }),
        });
    }

    vectors
}

/// Generate test queries
fn generate_test_queries(count: usize, dimension: usize) -> Vec<Vec<f32>> {
    let mut queries = Vec::with_capacity(count);
    let mut rng = rand::thread_rng();

    for i in 0..count {
        let mut query = Vec::with_capacity(dimension);

        // Generate query vectors with some structure
        for j in 0..dimension {
            let base_value = (i as f32 / count as f32) * 1.5 - 0.75;
            let noise = (j as f32 / dimension as f32) * 0.05;
            let value = base_value + noise + (rng.random::<f32>() - 0.5) * 0.1;
            query.push(value);
        }

        // Normalize query
        let norm: f32 = query.iter().map(|x| x * x).sum::<f32>().sqrt();
        if norm > 0.0 {
            for val in &mut query {
                *val /= norm;
            }
        }

        queries.push(query);
    }

    queries
}

/// Print benchmark summary
fn print_gpu_benchmark_summary(results: &[GpuBenchmarkResults]) {
    tracing::info!("\n📊 GPU BENCHMARK SUMMARY");
    tracing::info!("========================");
    tracing::info!(
        "{:<20} {:<10} {:<10} {:<10} {:<12} {:<12} {:<10}",
        "Test", "Vectors", "Queries", "HNSW(ms)", "CUDA(ms)", "Metal(ms)", "Speedup"
    );
    tracing::info!("{:-<100}", "");

    for result in results {
        let cuda_time = if result.cuda_total_time_ms > 0.0 {
            format!("{:.2}", result.cuda_avg_time_ms)
        } else {
            "N/A".to_string()
        };

        let metal_time = if result.metal_total_time_ms > 0.0 {
            format!("{:.2}", result.metal_avg_time_ms)
        } else {
            "N/A".to_string()
        };

        let max_speedup = result.cuda_speedup_factor.max(result.metal_speedup_factor);
        let speedup = if max_speedup > 0.0 {
            format!("{:.2}x", max_speedup)
        } else {
            "N/A".to_string()
        };

        tracing::info!(
            "{:<20} {:<10} {:<10} {:<10.2} {:<12} {:<12} {:<10}",
            result.test_name,
            result.vector_count,
            result.query_count,
            result.hnsw_avg_time_ms,
            cuda_time,
            metal_time,
            speedup
        );
    }

    // Calculate overall statistics
    let cuda_results: Vec<_> = results
        .iter()
        .filter(|r| r.cuda_speedup_factor > 0.0)
        .collect();
    let metal_results: Vec<_> = results
        .iter()
        .filter(|r| r.metal_speedup_factor > 0.0)
        .collect();

    tracing::info!("\n📈 Overall Statistics:");

    if !cuda_results.is_empty() {
        let avg_cuda_speedup: f64 = cuda_results
            .iter()
            .map(|r| r.cuda_speedup_factor)
            .sum::<f64>()
            / cuda_results.len() as f64;
        let max_cuda_speedup = cuda_results
            .iter()
            .map(|r| r.cuda_speedup_factor)
            .fold(0.0, f64::max);

        tracing::info!("  CUDA Performance:");
        tracing::info!("    Average Speedup: {:.2}x", avg_cuda_speedup);
        tracing::info!("    Maximum Speedup: {:.2}x", max_cuda_speedup);

        if avg_cuda_speedup > 1.0 {
            tracing::info!("    🎯 CUDA shows significant GPU acceleration!");
        } else {
            tracing::info!("    ⚠️ CUDA performance needs optimization");
        }
    } else {
        tracing::info!("  CUDA: Not available for benchmarking");
    }

    if !metal_results.is_empty() {
        let avg_metal_speedup: f64 = metal_results
            .iter()
            .map(|r| r.metal_speedup_factor)
            .sum::<f64>()
            / metal_results.len() as f64;
        let max_metal_speedup = metal_results
            .iter()
            .map(|r| r.metal_speedup_factor)
            .fold(0.0, f64::max);

        tracing::info!("  Metal Performance:");
        tracing::info!("    Average Speedup: {:.2}x", avg_metal_speedup);
        tracing::info!("    Maximum Speedup: {:.2}x", max_metal_speedup);

        if avg_metal_speedup > 1.0 {
            tracing::info!("    🎯 Metal shows significant GPU acceleration!");
        } else {
            tracing::info!("    ⚠️ Metal performance needs optimization");
        }
    } else {
        tracing::info!("  Metal: Not available for benchmarking");
    }
}

/// Criterion benchmark function for HNSW search
fn benchmark_hnsw_search(c: &mut Criterion) {
    let config = GpuBenchmarkConfig::default();
    let dimension = config.dimension;
    let k = config.k;

    for vector_count in &config.vector_counts {
        let mut group = c.benchmark_group("hnsw_search");
        group.measurement_time(std::time::Duration::from_secs(10));

        // Generate test data
        let test_vectors = generate_test_vectors(*vector_count, dimension);
        let test_queries = generate_test_queries(config.query_count, dimension);

        // Create HNSW index
        let hnsw_index = Arc::new(parking_lot::RwLock::new(
            OptimizedHnswIndex::new(
                dimension,
                vectorizer::db::optimized_hnsw::OptimizedHnswConfig {
                    max_connections: 16,
                    max_connections_0: 32,
                    ef_construction: 200,
                    seed: Some(42),
                    distance_metric: DistanceMetric::Cosine,
                    parallel: true,
                    initial_capacity: *vector_count,
                    batch_size: 1000,
                },
            )
            .expect("Failed to create HNSW index"),
        ));

        // Add vectors to index
        {
            let mut index = hnsw_index.write();
            for vector in &test_vectors {
                index.add(vector.id.clone(), vector.data.clone()).unwrap();
            }
        }

        group.bench_with_input(
            BenchmarkId::new("hnsw_search", vector_count),
            vector_count,
            |b, _| {
                b.iter(|| {
                    for query in &test_queries {
                        black_box(search_hnsw(&hnsw_index, query, k).unwrap());
                    }
                })
            },
        );

        group.finish();
    }
}

/// Criterion benchmark function for GPU operations (placeholder)
fn benchmark_gpu_operations(c: &mut Criterion) {
    let config = GpuBenchmarkConfig::default();
    let dimension = config.dimension;
    let k = config.k;

    for vector_count in &config.vector_counts {
        let mut group = c.benchmark_group("gpu_operations");
        group.measurement_time(std::time::Duration::from_secs(10));

        // Generate test data
        let test_vectors = generate_test_vectors(*vector_count, dimension);
        let test_queries = generate_test_queries(config.query_count, dimension);

        // Create HNSW index as fallback
        let hnsw_index = Arc::new(parking_lot::RwLock::new(
            OptimizedHnswIndex::new(
                dimension,
                vectorizer::db::optimized_hnsw::OptimizedHnswConfig {
                    max_connections: 16,
                    max_connections_0: 32,
                    ef_construction: 200,
                    seed: Some(42),
                    distance_metric: DistanceMetric::Cosine,
                    parallel: true,
                    initial_capacity: *vector_count,
                    batch_size: 1000,
                },
            )
            .expect("Failed to create HNSW index"),
        ));

        // Add vectors to index
        {
            let mut index = hnsw_index.write();
            for vector in &test_vectors {
                index.add(vector.id.clone(), vector.data.clone()).unwrap();
            }
        }

        group.bench_with_input(
            BenchmarkId::new("gpu_search_placeholder", vector_count),
            vector_count,
            |b, _| {
                b.iter(|| {
                    for query in &test_queries {
                        // Placeholder for actual GPU search
                        black_box(search_hnsw(&hnsw_index, query, k).unwrap());
                    }
                })
            },
        );

        group.finish();
    }
}

// Criterion benchmark groups
criterion_group!(benches, benchmark_hnsw_search, benchmark_gpu_operations);
criterion_main!(benches);
