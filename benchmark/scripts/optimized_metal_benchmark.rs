//! Optimized Metal Native Benchmark
//! 
//! Test optimized Metal Native implementation with buffer pooling and batch processing
//! to achieve performance up to 20k vectors.

use vectorizer::error::Result;
use tracing::{info, error, warn, debug};
use vectorizer::gpu::{OptimizedMetalNativeCollection, MetalNativeContext};
use vectorizer::models::{DistanceMetric, Vector};
use std::time::Instant;
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    tracing::info!("🚀 Optimized Metal Native Benchmark");
    tracing::info!("===================================");
    tracing::info!("Testing optimized implementation with buffer pooling and batch processing");
    tracing::info!("Target: 20k vectors with acceptable performance\n");

    #[cfg(not(target_os = "macos"))]
    {
        tracing::info!("❌ This benchmark requires macOS with Metal support");
        return Ok(());
    }

    #[cfg(target_os = "macos")]
    {
        run_optimized_benchmark().await?;
    }

    Ok(())
}

#[cfg(target_os = "macos")]
async fn run_optimized_benchmark() -> Result<()> {
    use std::time::Instant;

    // Test scenarios with optimized implementation
    let scenarios = vec![
        OptimizedBenchmarkScenario {
            name: "Small Dataset (Optimized)".to_string(),
            dimension: 128,
            vector_count: 1000,
            search_queries: 100,
            k: 10,
            batch_size: 100,
        },
        OptimizedBenchmarkScenario {
            name: "Medium Dataset (Optimized)".to_string(),
            dimension: 256,
            vector_count: 5000,
            search_queries: 200,
            k: 20,
            batch_size: 500,
        },
        OptimizedBenchmarkScenario {
            name: "Large Dataset (Optimized)".to_string(),
            dimension: 512,
            vector_count: 10000,
            search_queries: 500,
            k: 50,
            batch_size: 1000,
        },
        OptimizedBenchmarkScenario {
            name: "XLarge Dataset (Optimized)".to_string(),
            dimension: 512,
            vector_count: 20000,
            search_queries: 1000,
            k: 100,
            batch_size: 2000,
        },
    ];

    let mut all_results = Vec::new();

    for scenario in scenarios {
        tracing::info!("🎯 Running Scenario: {}", scenario.name);
        tracing::info!("=====================================");
        
        let result = run_optimized_scenario_benchmark(&scenario).await?;
        all_results.push(result);
        
        tracing::info!();
    }

    // Generate comprehensive report
    generate_optimized_report(&all_results).await?;

    Ok(())
}

#[derive(Debug, Clone)]
struct OptimizedBenchmarkScenario {
    name: String,
    dimension: usize,
    vector_count: usize,
    search_queries: usize,
    k: usize,
    batch_size: usize,
}

#[cfg(target_os = "macos")]
async fn run_optimized_scenario_benchmark(scenario: &OptimizedBenchmarkScenario) -> Result<OptimizedBenchmarkResult> {
    use std::time::Instant;

    tracing::info!("📊 Test Parameters");
    tracing::info!("------------------");
    tracing::info!("  Dimension: {}", scenario.dimension);
    tracing::info!("  Vector count: {}", scenario.vector_count);
    tracing::info!("  Search queries: {}", scenario.search_queries);
    tracing::info!("  k (results): {}", scenario.k);
    tracing::info!("  Batch size: {}", scenario.batch_size);
    tracing::info!();

    // 1. Generate test vectors
    tracing::info!("🔧 Generating test vectors...");
    let start = Instant::now();
    let vectors = generate_test_vectors(scenario.vector_count, scenario.dimension);
    let generation_time = start.elapsed();
    tracing::info!("  ✅ Generated {} vectors in {:.3}ms", scenario.vector_count, generation_time.as_millis());
    tracing::info!();

    // 2. Create optimized collection
    tracing::info!("📊 Test 1: Create Optimized Metal Native Collection");
    tracing::info!("----------------------------------------------------");
    let start = Instant::now();
    let mut collection = OptimizedMetalNativeCollection::new(
        scenario.dimension,
        DistanceMetric::Cosine,
        scenario.vector_count, // Pre-allocate for expected size
    )?;
    let creation_time = start.elapsed();
    tracing::info!("  ✅ Collection created: {:.3}ms", creation_time.as_millis());
    tracing::info!("  Device: Optimized Metal native (VRAM only)");
    tracing::info!("  Pre-allocated capacity: {}", scenario.vector_count);
    tracing::info!();

    // 3. Add vectors in batches (optimized)
    tracing::info!("📊 Test 2: Add Vectors in Batches (Optimized)");
    tracing::info!("---------------------------------------------");
    let start = Instant::now();
    let mut total_added = 0;
    
    for batch_start in (0..scenario.vector_count).step_by(scenario.batch_size) {
        let batch_end = std::cmp::min(batch_start + scenario.batch_size, scenario.vector_count);
        let batch = &vectors[batch_start..batch_end];
        
        let batch_start_time = Instant::now();
        collection.add_vectors_batch(batch)?;
        let batch_time = batch_start_time.elapsed();
        
        total_added += batch.len();
        tracing::info!("  Added batch {} vectors... ({:.3}ms)", total_added, batch_time.as_millis());
    }
    
    let addition_time = start.elapsed();
    tracing::info!("  ✅ Added {} vectors in batches: {:.3}ms", total_added, addition_time.as_millis());
    tracing::info!("  Throughput: {:.2} vectors/sec", total_added as f64 / addition_time.as_secs_f64());
    tracing::info!();

    // 4. Build HNSW index
    tracing::info!("📊 Test 3: Build HNSW Index on GPU (VRAM)");
    tracing::info!("-----------------------------------------");
    let start = Instant::now();
    collection.build_index()?;
    let construction_time = start.elapsed();
    tracing::info!("  ✅ HNSW index built on GPU: {:.3}ms", construction_time.as_millis());
    tracing::info!("  Storage: VRAM only (no CPU access)");
    tracing::info!("  Nodes: {}", total_added);
    tracing::info!();

    // 5. Search performance
    tracing::info!("📊 Test 4: Search Performance");
    tracing::info!("-----------------------------");
    let start = Instant::now();
    let mut search_times = Vec::new();
    
    for i in 0..std::cmp::min(scenario.search_queries, 50) { // Limit to 50 for testing
        let query_start = Instant::now();
        let query_vector = &vectors[i % scenario.vector_count];
        let results = collection.search(&query_vector.data, scenario.k)?;
        let query_time = query_start.elapsed();
        search_times.push(query_time.as_millis() as f64);
        
        if i % 10 == 0 {
            tracing::info!("  Completed {} searches...", i + 1);
        }
    }
    
    let total_search_time = start.elapsed();
    let avg_search_time = search_times.iter().sum::<f64>() / search_times.len() as f64;
    let min_search_time = search_times.iter().fold(f64::INFINITY, |a, &b| a.min(b));
    let max_search_time = search_times.iter().fold(0.0_f64, |a, &b| a.max(b));
    
    tracing::info!("  ✅ Completed {} searches", search_times.len());
    tracing::info!("  Average search time: {:.3}ms", avg_search_time);
    tracing::info!("  Min search time: {:.3}ms", min_search_time);
    tracing::info!("  Max search time: {:.3}ms", max_search_time);
    tracing::info!("  Total search time: {:.3}s", total_search_time.as_secs_f64());
    tracing::info!("  Throughput: {:.2} searches/sec", search_times.len() as f64 / total_search_time.as_secs_f64());
    tracing::info!();

    // 6. Memory usage analysis
    tracing::info!("📊 Test 5: Memory Usage Analysis");
    tracing::info!("--------------------------------");
    let memory_stats = collection.get_memory_stats();
    tracing::info!("  Vector count: {}", memory_stats.vector_count);
    tracing::info!("  Buffer capacity: {}", memory_stats.buffer_capacity);
    tracing::info!("  Used bytes: {:.2} MB", memory_stats.used_bytes as f64 / 1024.0 / 1024.0);
    tracing::info!("  Allocated bytes: {:.2} MB", memory_stats.allocated_bytes as f64 / 1024.0 / 1024.0);
    tracing::info!("  Utilization: {:.1}%", memory_stats.utilization * 100.0);
    tracing::info!("  Pool utilization: {:.1}%", memory_stats.buffer_pool_stats.pool_utilization * 100.0);
    tracing::info!();

    // 7. Memory compaction test
    tracing::info!("📊 Test 6: Memory Compaction");
    tracing::info!("-----------------------------");
    let start = Instant::now();
    collection.compact_memory()?;
    let compaction_time = start.elapsed();
    tracing::info!("  ✅ Memory compaction: {:.3}ms", compaction_time.as_millis());
    
    let final_memory_stats = collection.get_memory_stats();
    tracing::info!("  Final utilization: {:.1}%", final_memory_stats.utilization * 100.0);
    tracing::info!();

    Ok(OptimizedBenchmarkResult {
        scenario: scenario.clone(),
        generation_time,
        creation_time,
        addition_time,
        construction_time,
        search_times,
        memory_stats: final_memory_stats,
    })
}

#[derive(Debug, Clone)]
struct OptimizedBenchmarkResult {
    scenario: OptimizedBenchmarkScenario,
    generation_time: std::time::Duration,
    creation_time: std::time::Duration,
    addition_time: std::time::Duration,
    construction_time: std::time::Duration,
    search_times: Vec<f64>,
    memory_stats: vectorizer::gpu::CollectionMemoryStats,
}

fn generate_test_vectors(count: usize, dimension: usize) -> Vec<Vector> {
    let mut vectors = Vec::with_capacity(count);
    
    for i in 0..count {
        let mut data = Vec::with_capacity(dimension);
        for _ in 0..dimension {
            data.push(rand::random::<f32>());
        }
        
        vectors.push(Vector {
            id: format!("vector_{}", i),
            data,
            payload: None,
        });
    }
    
    vectors
}

#[cfg(target_os = "macos")]
async fn generate_optimized_report(results: &[OptimizedBenchmarkResult]) -> Result<()> {
    tracing::info!("📊 Optimized Benchmark Report");
    tracing::info!("=============================");
    tracing::info!("Comprehensive analysis of optimized Metal Native performance\n");
    
    for result in results {
        tracing::info!("🎯 Scenario: {}", result.scenario.name);
        tracing::info!("----------------------------------------");
        tracing::info!("  Dimension: {}", result.scenario.dimension);
        tracing::info!("  Vector count: {}", result.scenario.vector_count);
        tracing::info!("  Batch size: {}", result.scenario.batch_size);
        tracing::info!();
        
        tracing::info!("📈 Performance Metrics");
        tracing::info!("----------------------");
        tracing::info!("  Vector generation: {:.3}ms", result.generation_time.as_millis());
        tracing::info!("  Collection creation: {:.3}ms", result.creation_time.as_millis());
        tracing::info!("  Vector addition: {:.3}ms", result.addition_time.as_millis());
        tracing::info!("  Throughput addition: {:.2} vectors/sec", 
            result.scenario.vector_count as f64 / result.addition_time.as_secs_f64());
        tracing::info!("  HNSW construction: {:.3}ms", result.construction_time.as_millis());
        tracing::info!();
        
        if !result.search_times.is_empty() {
            let avg_search = result.search_times.iter().sum::<f64>() / result.search_times.len() as f64;
            let min_search = result.search_times.iter().fold(f64::INFINITY, |a, &b| a.min(b));
            let max_search = result.search_times.iter().fold(0.0_f64, |a, &b| a.max(b));
            
            tracing::info!("🔍 Search Performance");
            tracing::info!("--------------------");
            tracing::info!("  Average search time: {:.3}ms", avg_search);
            tracing::info!("  Min search time: {:.3}ms", min_search);
            tracing::info!("  Max search time: {:.3}ms", max_search);
            tracing::info!("  Search queries: {}", result.search_times.len());
            tracing::info!();
        }
        
        tracing::info!("💾 Memory Usage");
        tracing::info!("---------------");
        tracing::info!("  Vector count: {}", result.memory_stats.vector_count);
        tracing::info!("  Buffer capacity: {}", result.memory_stats.buffer_capacity);
        tracing::info!("  Used memory: {:.2} MB", result.memory_stats.used_bytes as f64 / 1024.0 / 1024.0);
        tracing::info!("  Allocated memory: {:.2} MB", result.memory_stats.allocated_bytes as f64 / 1024.0 / 1024.0);
        tracing::info!("  Memory utilization: {:.1}%", result.memory_stats.utilization * 100.0);
        tracing::info!("  Pool utilization: {:.1}%", result.memory_stats.buffer_pool_stats.pool_utilization * 100.0);
        tracing::info!();
        
        // Performance assessment
        let throughput = result.scenario.vector_count as f64 / result.addition_time.as_secs_f64();
        let search_throughput = if !result.search_times.is_empty() {
            result.search_times.len() as f64 / result.search_times.iter().sum::<f64>() * 1000.0
        } else {
            0.0
        };
        
        tracing::info!("🎯 Performance Assessment");
        tracing::info!("-------------------------");
        if throughput > 2000.0 {
            tracing::info!("  ✅ Vector addition: EXCELLENT ({:.0} vectors/sec)", throughput);
        } else if throughput > 1000.0 {
            tracing::info!("  ✅ Vector addition: GOOD ({:.0} vectors/sec)", throughput);
        } else if throughput > 500.0 {
            tracing::info!("  ⚠️ Vector addition: ACCEPTABLE ({:.0} vectors/sec)", throughput);
        } else {
            tracing::info!("  ❌ Vector addition: POOR ({:.0} vectors/sec)", throughput);
        }
        
        if search_throughput > 10.0 {
            tracing::info!("  ✅ Search performance: EXCELLENT ({:.1} searches/sec)", search_throughput);
        } else if search_throughput > 5.0 {
            tracing::info!("  ✅ Search performance: GOOD ({:.1} searches/sec)", search_throughput);
        } else if search_throughput > 1.0 {
            tracing::info!("  ⚠️ Search performance: ACCEPTABLE ({:.1} searches/sec)", search_throughput);
        } else {
            tracing::info!("  ❌ Search performance: POOR ({:.1} searches/sec)", search_throughput);
        }
        
        if result.memory_stats.utilization > 0.8 {
            tracing::info!("  ✅ Memory efficiency: EXCELLENT ({:.1}%)", result.memory_stats.utilization * 100.0);
        } else if result.memory_stats.utilization > 0.6 {
            tracing::info!("  ✅ Memory efficiency: GOOD ({:.1}%)", result.memory_stats.utilization * 100.0);
        } else {
            tracing::info!("  ⚠️ Memory efficiency: LOW ({:.1}%)", result.memory_stats.utilization * 100.0);
        }
        
        tracing::info!();
    }
    
    Ok(())
}

