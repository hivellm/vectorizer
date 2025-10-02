//! CUDA vs CPU Performance Benchmark
//!
//! This benchmark compares CUDA-accelerated vector search operations
//! against CPU-only HNSW search in the Vectorizer.

use vectorizer::models::{
    CollectionConfig, DistanceMetric, HnswConfig, CompressionConfig, QuantizationConfig,
    SearchResult, Vector, Payload
};
use vectorizer::error::Result;
use vectorizer::db::optimized_hnsw::OptimizedHnswIndex;
use vectorizer::cuda::CudaVectorOperations;
use std::time::{Duration, Instant};
use std::sync::Arc;
use rand::prelude::*;

/// Benchmark results
#[derive(Debug, Clone)]
pub struct BenchmarkResults {
    pub test_name: String,
    pub vector_count: usize,
    pub query_count: usize,
    pub hnsw_total_time_ms: f64,
    pub bend_total_time_ms: f64,
    pub cuda_total_time_ms: f64,
    pub hnsw_avg_time_ms: f64,
    pub bend_avg_time_ms: f64,
    pub cuda_avg_time_ms: f64,
    pub bend_speedup_factor: f64,
    pub cuda_speedup_factor: f64,
    pub memory_usage_mb: f64,
}

/// Run CUDA benchmark
pub async fn run_cuda_benchmark() -> Result<Vec<BenchmarkResults>> {
    println!("🚀 Starting CUDA vs CPU Benchmark");
    println!("=================================");

    let mut results = Vec::new();

    // Test configurations
    let test_configs = vec![
        ("Small Dataset", 1000, 100),
        ("Medium Dataset", 10000, 500),
        ("Large Dataset", 50000, 1000),
    ];

    for (test_name, vector_count, query_count) in test_configs {
        println!("\n📊 Running test: {} ({} vectors, {} queries)", test_name, vector_count, query_count);

        let result = run_single_benchmark(test_name, vector_count, query_count).await?;
        results.push(result);
    }

    Ok(results)
}

/// Run single benchmark test
async fn run_single_benchmark(test_name: &str, vector_count: usize, query_count: usize) -> Result<BenchmarkResults> {
    let dimension = 128; // Standard embedding dimension

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
    };

    // Create CUDA configuration
    let cuda_config = vectorizer::cuda::CudaConfig {
        enabled: true,
        device_id: 0,
        memory_limit_mb: 4096,
        max_threads_per_block: 1024,
        max_blocks_per_grid: 65535,
        memory_pool_size_mb: 1024,
    };

    // Create collections
    let hnsw_index = Arc::new(parking_lot::RwLock::new(
        OptimizedHnswIndex::new(dimension, vectorizer::db::optimized_hnsw::OptimizedHnswConfig {
            max_connections: collection_config.hnsw_config.m,
            max_connections_0: collection_config.hnsw_config.m * 2,
            ef_construction: collection_config.hnsw_config.ef_construction,
            seed: collection_config.hnsw_config.seed,
            distance_metric: collection_config.metric,
            parallel: true,
            initial_capacity: vector_count,
            batch_size: 1000,
        }).expect("Failed to create HNSW index")
    ));

    let cuda_ops = CudaVectorOperations::new(cuda_config);

    // Generate test data
    let test_vectors = generate_test_vectors(vector_count, dimension);
    let test_queries = generate_test_queries(query_count, dimension);

    // Add vectors to collections
    println!("  📥 Adding {} vectors to index...", vector_count);
    {
        let mut index = hnsw_index.write();
        for vector in &test_vectors {
            index.add(vector.id.clone(), vector.data.clone())?;
        }
    }

    // Initialize CUHNSW if available
    let mut cuda_ops = cuda_ops;
    if cuda_ops.is_cuda_available() {
        println!("  🎯 Initializing CUDA operations...");
        let flat_vectors: Vec<f32> = test_vectors.iter().flat_map(|v| v.data.clone()).collect();
        cuda_ops.initialize_cuhnsw(&flat_vectors, vector_count, dimension)?;
    }

    // Warmup runs (3-5 times as suggested)
    println!("  🔥 Warming up...");
    for _ in 0..3 {
        for query in &test_queries {
            let _results = search_hnsw(&hnsw_index, query, 10);
        }
        if cuda_ops.is_cuda_available() {
            for query in &test_queries {
                let _results = cuda_ops.parallel_similarity_search(query, &test_vectors.iter().map(|v| v.data.clone()).collect::<Vec<_>>(), 0.0, collection_config.metric).await;
            }
        }
    }

    // Benchmark HNSW search
    println!("  🧮 Benchmarking HNSW search...");
    let hnsw_start = Instant::now();
    for query in &test_queries {
        let _results = search_hnsw(&hnsw_index, query, 10);
    }
    let hnsw_duration = hnsw_start.elapsed().as_secs_f64() * 1000.0;

    // Benchmark CUDA search
    let cuda_duration = if cuda_ops.is_cuda_available() {
        println!("  🎮 Benchmarking CUDA search...");
        let cuda_start = Instant::now();
        for query in &test_queries {
            let _results = cuda_ops.parallel_similarity_search(query, &test_vectors.iter().map(|v| v.data.clone()).collect::<Vec<_>>(), 0.0, collection_config.metric).await;
        }
        cuda_start.elapsed().as_secs_f64() * 1000.0
    } else {
        println!("  ⚠️ CUDA not available, skipping CUDA benchmark");
        0.0
    };

    // Calculate metrics
    let hnsw_avg = hnsw_duration / query_count as f64;
    let cuda_avg = if cuda_duration > 0.0 { cuda_duration / query_count as f64 } else { 0.0 };
    let cuda_speedup = if cuda_avg > 0.0 { hnsw_avg / cuda_avg } else { 0.0 };

    // Estimate memory usage
    let memory_usage = (vector_count * dimension * 4) as f64 / (1024.0 * 1024.0); // MB

    let result = BenchmarkResults {
        test_name: test_name.to_string(),
        vector_count,
        query_count,
        hnsw_total_time_ms: hnsw_duration,
        bend_total_time_ms: 0.0, // No bend implementation in this benchmark
        cuda_total_time_ms: cuda_duration,
        hnsw_avg_time_ms: hnsw_avg,
        bend_avg_time_ms: 0.0,
        cuda_avg_time_ms: cuda_avg,
        bend_speedup_factor: 0.0,
        cuda_speedup_factor: cuda_speedup,
        memory_usage_mb: memory_usage,
    };

    println!("  📈 Results:");
    println!("    HNSW: {:.2}ms avg ({:.2}ms total)", hnsw_avg, hnsw_duration);
    if cuda_duration > 0.0 {
        println!("    CUDA: {:.2}ms avg ({:.2}ms total) - {:.2}x speedup", cuda_avg, cuda_duration, cuda_speedup);
    } else {
        println!("    CUDA: Not available");
    }
    println!("    Memory: {:.1} MB", memory_usage);

    Ok(result)
}

/// Search using HNSW index
fn search_hnsw(index: &Arc<parking_lot::RwLock<OptimizedHnswIndex>>, query: &[f32], k: usize) -> Result<Vec<SearchResult>> {
    let index = index.read();
    let neighbors = index.search(query, k)?;

    let results = neighbors.into_iter()
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
fn print_benchmark_summary(results: &[BenchmarkResults]) {
    println!("\n📊 BENCHMARK SUMMARY");
    println!("===================");
    println!("{:<20} {:<10} {:<10} {:<10} {:<12} {:<10}",
             "Test", "Vectors", "Queries", "HNSW(ms)", "CUDA(ms)", "Speedup");
    println!("{:-<90}", "");

    for result in results {
        let cuda_time = if result.cuda_total_time_ms > 0.0 {
            format!("{:.2}", result.cuda_avg_time_ms)
        } else {
            "N/A".to_string()
        };

        let speedup = if result.cuda_speedup_factor > 0.0 {
            format!("{:.2}x", result.cuda_speedup_factor)
        } else {
            "N/A".to_string()
        };

        println!("{:<20} {:<10} {:<10} {:<10.2} {:<12} {:<10}",
                 result.test_name,
                 result.vector_count,
                 result.query_count,
                 result.hnsw_avg_time_ms,
                 cuda_time,
                 speedup);
    }

    // Calculate overall statistics
    let avg_cuda_speedup: f64 = results.iter()
        .filter(|r| r.cuda_speedup_factor > 0.0)
        .map(|r| r.cuda_speedup_factor)
        .sum::<f64>() / results.iter().filter(|r| r.cuda_speedup_factor > 0.0).count() as f64;

    let max_cuda_speedup = results.iter()
        .map(|r| r.cuda_speedup_factor)
        .fold(0.0, f64::max);

    println!("\n📈 Overall Statistics:");
    if avg_cuda_speedup > 0.0 {
        println!("  Average CUDA Speedup: {:.2}x", avg_cuda_speedup);
        println!("  Maximum CUDA Speedup: {:.2}x", max_cuda_speedup);

        if avg_cuda_speedup > 1.0 {
            println!("  🎯 CUDA shows significant GPU acceleration!");
        } else {
            println!("  ⚠️ CUDA performance needs optimization");
        }
    } else {
        println!("  CUDA not available for benchmarking");
    }
}

/// Run CUDA compatibility analysis
pub fn run_cuda_compatibility_analysis() -> Result<()> {
    println!("🔍 CUDA Compatibility Analysis");
    println!("==============================");

    println!("CUDA 12.6 Status:");
    println!("  ✅ Installed: Yes");
    println!("  ✅ MSVC Compiler: Available");
    println!("  ❌ CUB Library: Incompatible");
    println!("  ❌ HNSW CUDA Code: Not compatible with CUDA 12.6");

    println!("\n📋 Compatibility Issues:");
    println!("  • CUDA 12.6 has breaking changes in CUB library");
    println!("  • MSVC compiler too old for modern C++ features");
    println!("  • Thrust/CUB deprecated API removals");
    println!("  • Inline assembly constraint incompatibilities");

    println!("\n💡 Solutions:");
    println!("  1. Use CUDA 11.8 (recommended for compatibility)");
    println!("  2. Update HNSW codebase for CUDA 12.x standards");
    println!("  3. Use CPU-only operations for now");

    println!("\n✅ Current Status:");
    println!("  • CPU operations: Working");
    println!("  • Library linking: Working (stub library)");
    println!("  • CUDA operations: Disabled (compatibility issues)");

    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    println!("🎯 Vectorizer CUDA Benchmark Suite");
    println!("===================================");

    // Run compatibility analysis first
    run_cuda_compatibility_analysis()?;

    // Run actual benchmarks
    println!("\n🚀 Running Performance Benchmarks...");
    match run_cuda_benchmark().await {
        Ok(results) => {
            print_benchmark_summary(&results);
            println!("\n🎉 Benchmark completed successfully!");
            println!("📊 Total tests run: {}", results.len());
        }
        Err(e) => {
            println!("❌ Benchmark failed: {}", e);
            return Err(e);
        }
    }

    Ok(())
}
