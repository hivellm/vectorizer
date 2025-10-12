//! Optimized Metal Native Benchmark
//! 
//! Test optimized Metal Native implementation with buffer pooling and batch processing
//! to achieve performance up to 20k vectors.

use vectorizer::error::Result;
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

    println!("🚀 Optimized Metal Native Benchmark");
    println!("===================================");
    println!("Testing optimized implementation with buffer pooling and batch processing");
    println!("Target: 20k vectors with acceptable performance\n");

    #[cfg(not(target_os = "macos"))]
    {
        println!("❌ This benchmark requires macOS with Metal support");
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
        println!("🎯 Running Scenario: {}", scenario.name);
        println!("=====================================");
        
        let result = run_optimized_scenario_benchmark(&scenario).await?;
        all_results.push(result);
        
        println!();
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

    println!("📊 Test Parameters");
    println!("------------------");
    println!("  Dimension: {}", scenario.dimension);
    println!("  Vector count: {}", scenario.vector_count);
    println!("  Search queries: {}", scenario.search_queries);
    println!("  k (results): {}", scenario.k);
    println!("  Batch size: {}", scenario.batch_size);
    println!();

    // 1. Generate test vectors
    println!("🔧 Generating test vectors...");
    let start = Instant::now();
    let vectors = generate_test_vectors(scenario.vector_count, scenario.dimension);
    let generation_time = start.elapsed();
    println!("  ✅ Generated {} vectors in {:.3}ms", scenario.vector_count, generation_time.as_millis());
    println!();

    // 2. Create optimized collection
    println!("📊 Test 1: Create Optimized Metal Native Collection");
    println!("----------------------------------------------------");
    let start = Instant::now();
    let mut collection = OptimizedMetalNativeCollection::new(
        scenario.dimension,
        DistanceMetric::Cosine,
        scenario.vector_count, // Pre-allocate for expected size
    )?;
    let creation_time = start.elapsed();
    println!("  ✅ Collection created: {:.3}ms", creation_time.as_millis());
    println!("  Device: Optimized Metal native (VRAM only)");
    println!("  Pre-allocated capacity: {}", scenario.vector_count);
    println!();

    // 3. Add vectors in batches (optimized)
    println!("📊 Test 2: Add Vectors in Batches (Optimized)");
    println!("---------------------------------------------");
    let start = Instant::now();
    let mut total_added = 0;
    
    for batch_start in (0..scenario.vector_count).step_by(scenario.batch_size) {
        let batch_end = std::cmp::min(batch_start + scenario.batch_size, scenario.vector_count);
        let batch = &vectors[batch_start..batch_end];
        
        let batch_start_time = Instant::now();
        collection.add_vectors_batch(batch)?;
        let batch_time = batch_start_time.elapsed();
        
        total_added += batch.len();
        println!("  Added batch {} vectors... ({:.3}ms)", total_added, batch_time.as_millis());
    }
    
    let addition_time = start.elapsed();
    println!("  ✅ Added {} vectors in batches: {:.3}ms", total_added, addition_time.as_millis());
    println!("  Throughput: {:.2} vectors/sec", total_added as f64 / addition_time.as_secs_f64());
    println!();

    // 4. Build HNSW index
    println!("📊 Test 3: Build HNSW Index on GPU (VRAM)");
    println!("-----------------------------------------");
    let start = Instant::now();
    collection.build_index()?;
    let construction_time = start.elapsed();
    println!("  ✅ HNSW index built on GPU: {:.3}ms", construction_time.as_millis());
    println!("  Storage: VRAM only (no CPU access)");
    println!("  Nodes: {}", total_added);
    println!();

    // 5. Search performance
    println!("📊 Test 4: Search Performance");
    println!("-----------------------------");
    let start = Instant::now();
    let mut search_times = Vec::new();
    
    for i in 0..std::cmp::min(scenario.search_queries, 50) { // Limit to 50 for testing
        let query_start = Instant::now();
        let query_vector = &vectors[i % scenario.vector_count];
        let results = collection.search(&query_vector.data, scenario.k)?;
        let query_time = query_start.elapsed();
        search_times.push(query_time.as_millis() as f64);
        
        if i % 10 == 0 {
            println!("  Completed {} searches...", i + 1);
        }
    }
    
    let total_search_time = start.elapsed();
    let avg_search_time = search_times.iter().sum::<f64>() / search_times.len() as f64;
    let min_search_time = search_times.iter().fold(f64::INFINITY, |a, &b| a.min(b));
    let max_search_time = search_times.iter().fold(0.0_f64, |a, &b| a.max(b));
    
    println!("  ✅ Completed {} searches", search_times.len());
    println!("  Average search time: {:.3}ms", avg_search_time);
    println!("  Min search time: {:.3}ms", min_search_time);
    println!("  Max search time: {:.3}ms", max_search_time);
    println!("  Total search time: {:.3}s", total_search_time.as_secs_f64());
    println!("  Throughput: {:.2} searches/sec", search_times.len() as f64 / total_search_time.as_secs_f64());
    println!();

    // 6. Memory usage analysis
    println!("📊 Test 5: Memory Usage Analysis");
    println!("--------------------------------");
    let memory_stats = collection.get_memory_stats();
    println!("  Vector count: {}", memory_stats.vector_count);
    println!("  Buffer capacity: {}", memory_stats.buffer_capacity);
    println!("  Used bytes: {:.2} MB", memory_stats.used_bytes as f64 / 1024.0 / 1024.0);
    println!("  Allocated bytes: {:.2} MB", memory_stats.allocated_bytes as f64 / 1024.0 / 1024.0);
    println!("  Utilization: {:.1}%", memory_stats.utilization * 100.0);
    println!("  Pool utilization: {:.1}%", memory_stats.buffer_pool_stats.pool_utilization * 100.0);
    println!();

    // 7. Memory compaction test
    println!("📊 Test 6: Memory Compaction");
    println!("-----------------------------");
    let start = Instant::now();
    collection.compact_memory()?;
    let compaction_time = start.elapsed();
    println!("  ✅ Memory compaction: {:.3}ms", compaction_time.as_millis());
    
    let final_memory_stats = collection.get_memory_stats();
    println!("  Final utilization: {:.1}%", final_memory_stats.utilization * 100.0);
    println!();

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
    println!("📊 Optimized Benchmark Report");
    println!("=============================");
    println!("Comprehensive analysis of optimized Metal Native performance\n");
    
    for result in results {
        println!("🎯 Scenario: {}", result.scenario.name);
        println!("----------------------------------------");
        println!("  Dimension: {}", result.scenario.dimension);
        println!("  Vector count: {}", result.scenario.vector_count);
        println!("  Batch size: {}", result.scenario.batch_size);
        println!();
        
        println!("📈 Performance Metrics");
        println!("----------------------");
        println!("  Vector generation: {:.3}ms", result.generation_time.as_millis());
        println!("  Collection creation: {:.3}ms", result.creation_time.as_millis());
        println!("  Vector addition: {:.3}ms", result.addition_time.as_millis());
        println!("  Throughput addition: {:.2} vectors/sec", 
            result.scenario.vector_count as f64 / result.addition_time.as_secs_f64());
        println!("  HNSW construction: {:.3}ms", result.construction_time.as_millis());
        println!();
        
        if !result.search_times.is_empty() {
            let avg_search = result.search_times.iter().sum::<f64>() / result.search_times.len() as f64;
            let min_search = result.search_times.iter().fold(f64::INFINITY, |a, &b| a.min(b));
            let max_search = result.search_times.iter().fold(0.0_f64, |a, &b| a.max(b));
            
            println!("🔍 Search Performance");
            println!("--------------------");
            println!("  Average search time: {:.3}ms", avg_search);
            println!("  Min search time: {:.3}ms", min_search);
            println!("  Max search time: {:.3}ms", max_search);
            println!("  Search queries: {}", result.search_times.len());
            println!();
        }
        
        println!("💾 Memory Usage");
        println!("---------------");
        println!("  Vector count: {}", result.memory_stats.vector_count);
        println!("  Buffer capacity: {}", result.memory_stats.buffer_capacity);
        println!("  Used memory: {:.2} MB", result.memory_stats.used_bytes as f64 / 1024.0 / 1024.0);
        println!("  Allocated memory: {:.2} MB", result.memory_stats.allocated_bytes as f64 / 1024.0 / 1024.0);
        println!("  Memory utilization: {:.1}%", result.memory_stats.utilization * 100.0);
        println!("  Pool utilization: {:.1}%", result.memory_stats.buffer_pool_stats.pool_utilization * 100.0);
        println!();
        
        // Performance assessment
        let throughput = result.scenario.vector_count as f64 / result.addition_time.as_secs_f64();
        let search_throughput = if !result.search_times.is_empty() {
            result.search_times.len() as f64 / result.search_times.iter().sum::<f64>() * 1000.0
        } else {
            0.0
        };
        
        println!("🎯 Performance Assessment");
        println!("-------------------------");
        if throughput > 2000.0 {
            println!("  ✅ Vector addition: EXCELLENT ({:.0} vectors/sec)", throughput);
        } else if throughput > 1000.0 {
            println!("  ✅ Vector addition: GOOD ({:.0} vectors/sec)", throughput);
        } else if throughput > 500.0 {
            println!("  ⚠️ Vector addition: ACCEPTABLE ({:.0} vectors/sec)", throughput);
        } else {
            println!("  ❌ Vector addition: POOR ({:.0} vectors/sec)", throughput);
        }
        
        if search_throughput > 10.0 {
            println!("  ✅ Search performance: EXCELLENT ({:.1} searches/sec)", search_throughput);
        } else if search_throughput > 5.0 {
            println!("  ✅ Search performance: GOOD ({:.1} searches/sec)", search_throughput);
        } else if search_throughput > 1.0 {
            println!("  ⚠️ Search performance: ACCEPTABLE ({:.1} searches/sec)", search_throughput);
        } else {
            println!("  ❌ Search performance: POOR ({:.1} searches/sec)", search_throughput);
        }
        
        if result.memory_stats.utilization > 0.8 {
            println!("  ✅ Memory efficiency: EXCELLENT ({:.1}%)", result.memory_stats.utilization * 100.0);
        } else if result.memory_stats.utilization > 0.6 {
            println!("  ✅ Memory efficiency: GOOD ({:.1}%)", result.memory_stats.utilization * 100.0);
        } else {
            println!("  ⚠️ Memory efficiency: LOW ({:.1}%)", result.memory_stats.utilization * 100.0);
        }
        
        println!();
    }
    
    Ok(())
}

