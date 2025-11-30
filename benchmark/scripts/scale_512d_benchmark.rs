//! Scale Benchmark for 512D Vectors
//! 
//! Test vector generation and search performance with 512 dimensions
//! across different scales: 5k, 10k, 15k, 25k vectors

use vectorizer::error::Result;
use tracing::{info, error, warn, debug};
use vectorizer::gpu::{OptimizedMetalNativeCollection, VramMonitor, VramValidator};
use vectorizer::models::{DistanceMetric, Vector};
use std::time::Instant;
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    tracing::info!("🚀 Scale Benchmark - 512D Vectors");
    tracing::info!("=================================");
    tracing::info!("Testing vector generation and search performance");
    tracing::info!("Dimensions: 512, Scales: 5k, 10k, 15k, 25k vectors\n");

    #[cfg(not(target_os = "macos"))]
    {
        tracing::info!("❌ This benchmark requires macOS with Metal support");
        return Ok(());
    }

    #[cfg(target_os = "macos")]
    {
        run_scale_512d_benchmark().await?;
    }

    Ok(())
}

#[cfg(target_os = "macos")]
async fn run_scale_512d_benchmark() -> Result<()> {
    use std::time::Instant;

    // Test scenarios with 512 dimensions
    let scenarios = vec![
        Scale512DScenario {
            name: "5K Vectors (512D)".to_string(),
            dimension: 512,
            vector_count: 5000,
            search_queries: 100,
            k: 20,
            batch_size: 500,
        },
        Scale512DScenario {
            name: "10K Vectors (512D)".to_string(),
            dimension: 512,
            vector_count: 10000,
            search_queries: 200,
            k: 50,
            batch_size: 1000,
        },
        Scale512DScenario {
            name: "15K Vectors (512D)".to_string(),
            dimension: 512,
            vector_count: 15000,
            search_queries: 300,
            k: 75,
            batch_size: 1500,
        },
        Scale512DScenario {
            name: "25K Vectors (512D)".to_string(),
            dimension: 512,
            vector_count: 25000,
            search_queries: 500,
            k: 100,
            batch_size: 2500,
        },
    ];

    let mut all_results = Vec::new();

    for scenario in scenarios {
        tracing::info!("🎯 Running Scenario: {}", scenario.name);
        tracing::info!("{}", "=".repeat(scenario.name.len() + 20));
        
        let result = run_scale_512d_scenario_benchmark(&scenario).await?;
        all_results.push(result);
        
        tracing::info!();
    }

    // Generate comprehensive report
    generate_scale_512d_report(&all_results).await?;

    Ok(())
}

#[derive(Debug, Clone)]
struct Scale512DScenario {
    name: String,
    dimension: usize,
    vector_count: usize,
    search_queries: usize,
    k: usize,
    batch_size: usize,
}

#[cfg(target_os = "macos")]
async fn run_scale_512d_scenario_benchmark(scenario: &Scale512DScenario) -> Result<Scale512DResult> {
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
    tracing::info!("🔧 Phase 1: Vector Generation");
    tracing::info!("-----------------------------");
    let start = Instant::now();
    let vectors = generate_test_vectors_512d(scenario.vector_count, scenario.dimension);
    let generation_time = start.elapsed();
    tracing::info!("  ✅ Generated {} vectors in {:.3}ms", scenario.vector_count, generation_time.as_millis());
    tracing::info!("  Memory per vector: {:.2} KB", (scenario.dimension * 4) as f64 / 1024.0);
    tracing::info!("  Total memory: {:.2} MB", (scenario.vector_count * scenario.dimension * 4) as f64 / 1024.0 / 1024.0);
    tracing::info!();

    // 2. Create optimized collection
    tracing::info!("📊 Phase 2: Create Optimized Collection");
    tracing::info!("--------------------------------------");
    let start = Instant::now();
    let mut collection = OptimizedMetalNativeCollection::new(
        scenario.dimension,
        DistanceMetric::Cosine,
        scenario.vector_count,
    )?;
    let creation_time = start.elapsed();
    tracing::info!("  ✅ Collection created: {:.3}ms", creation_time.as_millis());
    tracing::info!("  Pre-allocated capacity: {}", scenario.vector_count);
    tracing::info!();

    // 3. Add vectors in batches
    tracing::info!("📊 Phase 3: Add Vectors in Batches");
    tracing::info!("----------------------------------");
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
    tracing::info!("📊 Phase 4: Build HNSW Index");
    tracing::info!("---------------------------");
    let start = Instant::now();
    collection.build_index()?;
    let construction_time = start.elapsed();
    tracing::info!("  ✅ HNSW index built: {:.3}ms", construction_time.as_millis());
    tracing::info!("  Nodes: {}", total_added);
    tracing::info!();

    // 5. Search performance
    tracing::info!("📊 Phase 5: Search Performance");
    tracing::info!("--------------------------");
    let start = Instant::now();
    let mut search_times = Vec::new();
    
    for i in 0..scenario.search_queries {
        let query_start = Instant::now();
        let query_vector = &vectors[i % scenario.vector_count];
        let results = collection.search(&query_vector.data, scenario.k)?;
        let query_time = query_start.elapsed();
        search_times.push(query_time.as_millis() as f64);
        
        if i % 50 == 0 {
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
    tracing::info!("📊 Phase 6: Memory Usage Analysis");
    tracing::info!("---------------------------------");
    let memory_stats = collection.get_memory_stats();
    tracing::info!("  Vector count: {}", memory_stats.vector_count);
    tracing::info!("  Buffer capacity: {}", memory_stats.buffer_capacity);
    tracing::info!("  Used bytes: {:.2} MB", memory_stats.used_bytes as f64 / 1024.0 / 1024.0);
    tracing::info!("  Allocated bytes: {:.2} MB", memory_stats.allocated_bytes as f64 / 1024.0 / 1024.0);
    tracing::info!("  Utilization: {:.1}%", memory_stats.utilization * 100.0);
    tracing::info!("  Pool utilization: {:.1}%", memory_stats.buffer_pool_stats.pool_utilization * 100.0);
    tracing::info!();

    // 7. Performance assessment
    let throughput = scenario.vector_count as f64 / addition_time.as_secs_f64();
    let search_throughput = search_times.len() as f64 / total_search_time.as_secs_f64();
    
    tracing::info!("📊 Phase 7: Performance Assessment");
    tracing::info!("----------------------------------");
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
    
    if memory_stats.utilization > 0.8 {
        tracing::info!("  ✅ Memory efficiency: EXCELLENT ({:.1}%)", memory_stats.utilization * 100.0);
    } else if memory_stats.utilization > 0.6 {
        tracing::info!("  ✅ Memory efficiency: GOOD ({:.1}%)", memory_stats.utilization * 100.0);
    } else {
        tracing::info!("  ⚠️ Memory efficiency: LOW ({:.1}%)", memory_stats.utilization * 100.0);
    }
    tracing::info!();

    Ok(Scale512DResult {
        scenario: scenario.clone(),
        generation_time,
        creation_time,
        addition_time,
        construction_time,
        search_times,
        memory_stats,
        throughput,
        search_throughput,
    })
}

fn generate_test_vectors_512d(count: usize, dimension: usize) -> Vec<Vector> {
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

#[derive(Debug, Clone)]
struct Scale512DResult {
    scenario: Scale512DScenario,
    generation_time: std::time::Duration,
    creation_time: std::time::Duration,
    addition_time: std::time::Duration,
    construction_time: std::time::Duration,
    search_times: Vec<f64>,
    memory_stats: vectorizer::gpu::CollectionMemoryStats,
    throughput: f64,
    search_throughput: f64,
}

#[cfg(target_os = "macos")]
async fn generate_scale_512d_report(results: &[Scale512DResult]) -> Result<()> {
    tracing::info!("📊 Scale 512D Benchmark Report");
    tracing::info!("==============================");
    tracing::info!("Comprehensive analysis of 512D vector performance across scales\n");
    
    // Summary table
    tracing::info!("📈 Performance Summary Table");
    tracing::info!("----------------------------");
    tracing::info!("| Scale    | Vectors | Gen(ms) | Add(ms) | Search(ms) | Add/sec | Search/sec | Memory(MB) |");
    tracing::info!("|----------|---------|---------|---------|------------|---------|------------|------------|");
    
    for result in results {
        let avg_search = result.search_times.iter().sum::<f64>() / result.search_times.len() as f64;
        let memory_mb = result.memory_stats.used_bytes as f64 / 1024.0 / 1024.0;
        
        tracing::info!(
            "| {:8} | {:7} | {:7.1} | {:7.1} | {:10.3} | {:7.0} | {:10.1} | {:10.1} |",
            result.scenario.name,
            result.scenario.vector_count,
            result.generation_time.as_millis(),
            result.addition_time.as_millis(),
            avg_search,
            result.throughput,
            result.search_throughput,
            memory_mb
        );
    }
    tracing::info!();
    
    // Detailed analysis
    for result in results {
        tracing::info!("🎯 Scenario: {}", result.scenario.name);
        tracing::info!("{}", "-".repeat(result.scenario.name.len() + 10));
        
        tracing::info!("📊 Generation Performance");
        tracing::info!("-------------------------");
        tracing::info!("  Vector count: {}", result.scenario.vector_count);
        tracing::info!("  Dimension: {}", result.scenario.dimension);
        tracing::info!("  Generation time: {:.3}ms", result.generation_time.as_millis());
        tracing::info!("  Memory per vector: {:.2} KB", (result.scenario.dimension * 4) as f64 / 1024.0);
        tracing::info!();
        
        tracing::info!("📊 Addition Performance");
        tracing::info!("-----------------------");
        tracing::info!("  Addition time: {:.3}ms", result.addition_time.as_millis());
        tracing::info!("  Throughput: {:.2} vectors/sec", result.throughput);
        tracing::info!("  Batch size: {}", result.scenario.batch_size);
        tracing::info!();
        
        if !result.search_times.is_empty() {
            let avg_search = result.search_times.iter().sum::<f64>() / result.search_times.len() as f64;
            let min_search = result.search_times.iter().fold(f64::INFINITY, |a, &b| a.min(b));
            let max_search = result.search_times.iter().fold(0.0_f64, |a, &b| a.max(b));
            
            tracing::info!("🔍 Search Performance");
            tracing::info!("--------------------");
            tracing::info!("  Search queries: {}", result.search_times.len());
            tracing::info!("  Average search time: {:.3}ms", avg_search);
            tracing::info!("  Min search time: {:.3}ms", min_search);
            tracing::info!("  Max search time: {:.3}ms", max_search);
            tracing::info!("  Search throughput: {:.2} searches/sec", result.search_throughput);
            tracing::info!();
        }
        
        tracing::info!("💾 Memory Usage");
        tracing::info!("---------------");
        tracing::info!("  Vector count: {}", result.memory_stats.vector_count);
        tracing::info!("  Buffer capacity: {}", result.memory_stats.buffer_capacity);
        tracing::info!("  Used memory: {:.2} MB", result.memory_stats.used_bytes as f64 / 1024.0 / 1024.0);
        tracing::info!("  Allocated memory: {:.2} MB", result.memory_stats.allocated_bytes as f64 / 1024.0 / 1024.0);
        tracing::info!("  Memory utilization: {:.1}%", result.memory_stats.utilization * 100.0);
        tracing::info!();
        
        // Performance assessment
        tracing::info!("🎯 Performance Assessment");
        tracing::info!("-------------------------");
        if result.throughput > 2000.0 {
            tracing::info!("  ✅ Vector addition: EXCELLENT ({:.0} vectors/sec)", result.throughput);
        } else if result.throughput > 1000.0 {
            tracing::info!("  ✅ Vector addition: GOOD ({:.0} vectors/sec)", result.throughput);
        } else if result.throughput > 500.0 {
            tracing::info!("  ⚠️ Vector addition: ACCEPTABLE ({:.0} vectors/sec)", result.throughput);
        } else {
            tracing::info!("  ❌ Vector addition: POOR ({:.0} vectors/sec)", result.throughput);
        }
        
        if result.search_throughput > 10.0 {
            tracing::info!("  ✅ Search performance: EXCELLENT ({:.1} searches/sec)", result.search_throughput);
        } else if result.search_throughput > 5.0 {
            tracing::info!("  ✅ Search performance: GOOD ({:.1} searches/sec)", result.search_throughput);
        } else if result.search_throughput > 1.0 {
            tracing::info!("  ⚠️ Search performance: ACCEPTABLE ({:.1} searches/sec)", result.search_throughput);
        } else {
            tracing::info!("  ❌ Search performance: POOR ({:.1} searches/sec)", result.search_throughput);
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
    
    // Scale analysis
    tracing::info!("📈 Scale Analysis");
    tracing::info!("-----------------");
    let mut scales = results.iter().map(|r| r.scenario.vector_count).collect::<Vec<_>>();
    scales.sort();
    
    for i in 1..scales.len() {
        let prev_scale = scales[i-1];
        let curr_scale = scales[i];
        let scale_ratio = curr_scale as f64 / prev_scale as f64;
        
        let prev_result = results.iter().find(|r| r.scenario.vector_count == prev_scale).unwrap();
        let curr_result = results.iter().find(|r| r.scenario.vector_count == curr_scale).unwrap();
        
        let throughput_ratio = curr_result.throughput / prev_result.throughput;
        let search_ratio = curr_result.search_throughput / prev_result.search_throughput;
        
        tracing::info!("  {} -> {} vectors ({}x scale)", prev_scale, curr_scale, scale_ratio);
        tracing::info!("    Throughput ratio: {:.2}x", throughput_ratio);
        tracing::info!("    Search ratio: {:.2}x", search_ratio);
        
        if throughput_ratio > 0.8 {
            tracing::info!("    ✅ Addition scales well");
        } else {
            tracing::info!("    ⚠️ Addition performance degrades");
        }
        
        if search_ratio > 0.8 {
            tracing::info!("    ✅ Search scales well");
        } else {
            tracing::info!("    ⚠️ Search performance degrades");
        }
        tracing::info!();
    }
    
    Ok(())
}

