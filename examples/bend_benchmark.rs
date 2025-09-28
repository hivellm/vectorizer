//! Bend vs HNSW Performance Benchmark
//! 
//! This benchmark compares the performance of Bend-accelerated vector search
//! against traditional HNSW search in the Vectorizer.

use std::sync::Arc;
use std::time::Instant;
use vectorizer::bend::{BendConfig, collection::BendCollection};
use vectorizer::cuda::{CudaConfig, collection::CudaCollection};
use vectorizer::models::{CollectionConfig, HnswConfig, DistanceMetric, Vector, Payload, CompressionConfig};
use vectorizer::error::Result;

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

/// Run comprehensive Bend vs HNSW vs CUDA benchmark
pub async fn run_bend_benchmark() -> Result<Vec<BenchmarkResults>> {
    println!("🏁 Starting Bend vs HNSW vs CUDA Performance Benchmark");
    println!("=====================================================");

    let mut results = Vec::new();

    // Test configurations
    let test_configs = vec![
        (100, 10, "Small Dataset"),
        (1000, 50, "Medium Dataset"),
        (5000, 100, "Large Dataset"),
        (10000, 200, "Very Large Dataset"),
    ];

    for (vector_count, query_count, test_name) in test_configs {
        println!("\n🧪 Testing: {} ({} vectors, {} queries)", test_name, vector_count, query_count);
        
        let result = run_single_benchmark(vector_count, query_count, test_name).await?;
        results.push(result.clone());
        
        println!("  ✅ Completed: Bend {:.2}x, CUDA {:.2}x speedup", result.bend_speedup_factor, result.cuda_speedup_factor);
    }

    // Print summary
    print_benchmark_summary(&results);
    
    Ok(results)
}

/// Run a single benchmark test
async fn run_single_benchmark(
    vector_count: usize,
    query_count: usize,
    test_name: &str,
) -> Result<BenchmarkResults> {
    let dimension = 384; // Standard embedding dimension
    
    // Create Bend configuration
    let bend_config = BendConfig {
        enabled: true,
        enable_cuda: false,
        max_parallel: 1000,
        fallback_enabled: true,
        bend_path: "bend".to_string(),
    };

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
        quantization: None,
    };

    // Create CUDA configuration
    let cuda_config = CudaConfig {
        enabled: true,
        device_id: 0,
        max_threads_per_block: 1024,
        max_blocks_per_grid: 65535,
        memory_pool_size_mb: 1024,
    };

    // Create Bend-enhanced collection
    let bend_collection = BendCollection::new(
        format!("benchmark_bend_{}", vector_count),
        collection_config.clone(),
        bend_config,
    );

    // Create CUDA-enhanced collection
    let mut cuda_collection = CudaCollection::new(
        format!("benchmark_cuda_{}", vector_count),
        collection_config,
        cuda_config,
    );

    // Generate test vectors
    let test_vectors = generate_test_vectors(vector_count, dimension);
    
    // Add vectors to collections
    bend_collection.batch_add_vectors_with_bend(test_vectors.clone()).await?;
    cuda_collection.batch_add_vectors_with_cuda(test_vectors.clone()).await?;
    
    // Initialize CUHNSW if available (using public method)
    let flat_vectors: Vec<f32> = test_vectors.iter().flat_map(|v| v.data.clone()).collect();
    cuda_collection.initialize_cuhnsw(&flat_vectors, vector_count, dimension)?;

    // Generate test queries
    let test_queries = generate_test_queries(query_count, dimension);

    // Benchmark HNSW search
    let hnsw_start = Instant::now();
    for query in &test_queries {
        let _results = bend_collection.search(query, 10)?;
    }
    let hnsw_duration = hnsw_start.elapsed().as_secs_f64() * 1000.0;

    // Benchmark Bend search
    let bend_start = Instant::now();
    for query in &test_queries {
        let _results = bend_collection.search_with_bend(query, 10).await?;
    }
    let bend_duration = bend_start.elapsed().as_secs_f64() * 1000.0;

    // Benchmark CUDA search
    let cuda_start = Instant::now();
    for query in &test_queries {
        let _results = cuda_collection.search_with_cuda(query, 10).await?;
    }
    let cuda_duration = cuda_start.elapsed().as_secs_f64() * 1000.0;

    // Calculate metrics
    let hnsw_avg = hnsw_duration / query_count as f64;
    let bend_avg = bend_duration / query_count as f64;
    let cuda_avg = cuda_duration / query_count as f64;
    let bend_speedup = if bend_avg > 0.0 { hnsw_avg / bend_avg } else { 0.0 };
    let cuda_speedup = if cuda_avg > 0.0 { hnsw_avg / cuda_avg } else { 0.0 };
    
    // Estimate memory usage
    let memory_usage = (vector_count * dimension * 4) as f64 / (1024.0 * 1024.0); // MB

    Ok(BenchmarkResults {
        test_name: test_name.to_string(),
        vector_count,
        query_count,
        hnsw_total_time_ms: hnsw_duration,
        bend_total_time_ms: bend_duration,
        cuda_total_time_ms: cuda_duration,
        hnsw_avg_time_ms: hnsw_avg,
        bend_avg_time_ms: bend_avg,
        cuda_avg_time_ms: cuda_avg,
        bend_speedup_factor: bend_speedup,
        cuda_speedup_factor: cuda_speedup,
        memory_usage_mb: memory_usage,
    })
}

/// Generate test vectors with realistic patterns
fn generate_test_vectors(count: usize, dimension: usize) -> Vec<Vector> {
    let mut vectors = Vec::with_capacity(count);
    
    for i in 0..count {
        let mut data = Vec::with_capacity(dimension);
        
        // Generate vectors with some structure (not completely random)
        for j in 0..dimension {
            let base_value = (i as f32 / count as f32) * 2.0 - 1.0;
            let noise = (j as f32 / dimension as f32) * 0.1;
            let value = base_value + noise + (rand::random::<f32>() - 0.5) * 0.2;
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
    
    for i in 0..count {
        let mut query = Vec::with_capacity(dimension);
        
        // Generate query vectors with some structure
        for j in 0..dimension {
            let base_value = (i as f32 / count as f32) * 1.5 - 0.75;
            let noise = (j as f32 / dimension as f32) * 0.05;
            let value = base_value + noise + (rand::random::<f32>() - 0.5) * 0.1;
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
    println!("{:<20} {:<10} {:<10} {:<10} {:<10} {:<10} {:<10}", 
             "Test", "Vectors", "Queries", "HNSW(ms)", "Bend(ms)", "CUDA(ms)", "Best");
    println!("{:-<90}", "");
    
    for result in results {
        let best = if result.cuda_speedup_factor > result.bend_speedup_factor {
            format!("CUDA {:.2}x", result.cuda_speedup_factor)
        } else {
            format!("Bend {:.2}x", result.bend_speedup_factor)
        };
        
        println!("{:<20} {:<10} {:<10} {:<10.2} {:<10.2} {:<10.2} {:<10}", 
                 result.test_name,
                 result.vector_count,
                 result.query_count,
                 result.hnsw_avg_time_ms,
                 result.bend_avg_time_ms,
                 result.cuda_avg_time_ms,
                 best);
    }
    
    // Calculate overall statistics
    let avg_bend_speedup: f64 = results.iter().map(|r| r.bend_speedup_factor).sum::<f64>() / results.len() as f64;
    let avg_cuda_speedup: f64 = results.iter().map(|r| r.cuda_speedup_factor).sum::<f64>() / results.len() as f64;
    let max_bend_speedup = results.iter().map(|r| r.bend_speedup_factor).fold(0.0, f64::max);
    let max_cuda_speedup = results.iter().map(|r| r.cuda_speedup_factor).fold(0.0, f64::max);
    
    println!("\n📈 Overall Statistics:");
    println!("  Average Bend Speedup: {:.2}x", avg_bend_speedup);
    println!("  Average CUDA Speedup: {:.2}x", avg_cuda_speedup);
    println!("  Maximum Bend Speedup: {:.2}x", max_bend_speedup);
    println!("  Maximum CUDA Speedup: {:.2}x", max_cuda_speedup);
    
    if avg_cuda_speedup > avg_bend_speedup {
        println!("  🚀 CUDA provides better performance than Bend!");
    } else if avg_bend_speedup > avg_cuda_speedup {
        println!("  ⚡ Bend provides better performance than CUDA!");
    } else {
        println!("  ⚖️  Bend and CUDA performance are similar");
    }
    
    if avg_cuda_speedup > 1.0 {
        println!("  🎯 CUDA shows significant GPU acceleration!");
    } else if avg_bend_speedup > 1.0 {
        println!("  🎯 Bend shows significant parallelization benefits!");
    } else {
        println!("  ⚠️  Both Bend and CUDA may need optimization for this workload");
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt::init();
    
    // Run the benchmark
    let results = run_bend_benchmark().await?;
    
    println!("\n🎯 Benchmark completed successfully!");
    println!("📊 Total tests run: {}", results.len());
    
    Ok(())
}
