//! VRAM Validation Benchmark
//! 
//! This benchmark validates that all operations are truly using VRAM
//! and not falling back to RAM, with comprehensive monitoring and reporting.

use vectorizer::error::Result;
use vectorizer::gpu::{VramMonitor, VramValidator, OptimizedMetalNativeCollection};
use vectorizer::models::{DistanceMetric, Vector};
use std::time::Instant;
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    println!("🔍 VRAM Validation Benchmark");
    println!("============================");
    println!("Comprehensive validation that all operations use VRAM, not RAM");
    println!("Includes performance monitoring and fallback detection\n");

    #[cfg(not(target_os = "macos"))]
    {
        println!("❌ This benchmark requires macOS with Metal support");
        return Ok(());
    }

    #[cfg(target_os = "macos")]
    {
        run_vram_validation_benchmark().await?;
    }

    Ok(())
}

#[cfg(target_os = "macos")]
async fn run_vram_validation_benchmark() -> Result<()> {
    use std::time::Instant;

    println!("🔍 Phase 1: Device VRAM Validation");
    println!("===================================");
    
    // 1. Validate Metal device VRAM capabilities
    let device = metal::Device::system_default().unwrap();
    VramValidator::validate_device_vram(&device)?;
    println!("  ✅ Device VRAM validation passed\n");
    
    // 2. Benchmark VRAM vs RAM performance
    println!("🔍 Phase 2: VRAM vs RAM Performance Benchmark");
    println!("==============================================");
    let benchmark_result = VramValidator::benchmark_vram_vs_ram(&device)?;
    
    if benchmark_result.vram_faster {
        println!("  ✅ VRAM is faster than RAM: {:.2}x speedup", benchmark_result.performance_ratio);
    } else {
        println!("  ⚠️ VRAM is slower than RAM: {:.2}x slowdown", 1.0 / benchmark_result.performance_ratio);
    }
    println!();
    
    // 3. Create VRAM monitor
    println!("🔍 Phase 3: VRAM Monitor Setup");
    println!("==============================");
    let mut vram_monitor = VramMonitor::new(device.clone());
    vram_monitor.force_vram_validation()?;
    println!("  ✅ VRAM monitor initialized and validated\n");
    
    // 4. Test scenarios with VRAM monitoring
    let scenarios = vec![
        VramTestScenario {
            name: "Small Dataset VRAM Test".to_string(),
            dimension: 128,
            vector_count: 1000,
            batch_size: 100,
        },
        VramTestScenario {
            name: "Medium Dataset VRAM Test".to_string(),
            dimension: 256,
            vector_count: 5000,
            batch_size: 500,
        },
        VramTestScenario {
            name: "Large Dataset VRAM Test".to_string(),
            dimension: 512,
            vector_count: 10000,
            batch_size: 1000,
        },
        VramTestScenario {
            name: "XLarge Dataset VRAM Test".to_string(),
            dimension: 512,
            vector_count: 20000,
            batch_size: 2000,
        },
    ];
    
    let mut all_results = Vec::new();
    
    for scenario in scenarios {
        println!("🔍 Phase 4: {}", scenario.name);
        println!("{}", "=".repeat(scenario.name.len() + 15));
        
        let result = run_vram_test_scenario(&mut vram_monitor, &scenario).await?;
        all_results.push(result);
        
        println!();
    }
    
    // 5. Generate comprehensive VRAM report
    println!("🔍 Phase 5: Comprehensive VRAM Report");
    println!("======================================");
    generate_vram_report(&all_results, &vram_monitor).await?;
    
    Ok(())
}

#[derive(Debug, Clone)]
struct VramTestScenario {
    name: String,
    dimension: usize,
    vector_count: usize,
    batch_size: usize,
}

#[cfg(target_os = "macos")]
async fn run_vram_test_scenario(
    vram_monitor: &mut VramMonitor,
    scenario: &VramTestScenario,
) -> Result<VramTestResult> {
    use std::time::Instant;

    println!("📊 Test Parameters");
    println!("------------------");
    println!("  Dimension: {}", scenario.dimension);
    println!("  Vector count: {}", scenario.vector_count);
    println!("  Batch size: {}", scenario.batch_size);
    println!();

    // 1. Generate test vectors
    println!("🔧 Generating test vectors...");
    let start = Instant::now();
    let vectors = generate_test_vectors(scenario.vector_count, scenario.dimension);
    let generation_time = start.elapsed();
    println!("  ✅ Generated {} vectors in {:.3}ms", scenario.vector_count, generation_time.as_millis());
    println!();

    // 2. Create collection with VRAM monitoring
    println!("📊 Test 1: Create Collection with VRAM Monitoring");
    println!("--------------------------------------------------");
    let start = Instant::now();
    let mut collection = vram_monitor.monitor_operation("collection_creation", || {
        OptimizedMetalNativeCollection::new(
            scenario.dimension,
            DistanceMetric::Cosine,
            scenario.vector_count,
        )
    })?;
    let creation_time = start.elapsed();
    println!("  ✅ Collection created with VRAM monitoring: {:.3}ms", creation_time.as_millis());
    println!();

    // 3. Add vectors with VRAM monitoring
    println!("📊 Test 2: Add Vectors with VRAM Monitoring");
    println!("-------------------------------------------");
    let start = Instant::now();
    let mut total_added = 0;
    
    for batch_start in (0..scenario.vector_count).step_by(scenario.batch_size) {
        let batch_end = std::cmp::min(batch_start + scenario.batch_size, scenario.vector_count);
        let batch = &vectors[batch_start..batch_end];
        
        let batch_start_time = Instant::now();
        vram_monitor.monitor_operation("vector_batch_addition", || {
            collection.add_vectors_batch(batch)
        })?;
        let batch_time = batch_start_time.elapsed();
        
        total_added += batch.len();
        println!("  Added batch {} vectors... ({:.3}ms)", total_added, batch_time.as_millis());
    }
    
    let addition_time = start.elapsed();
    println!("  ✅ Added {} vectors with VRAM monitoring: {:.3}ms", total_added, addition_time.as_millis());
    println!("  Throughput: {:.2} vectors/sec", total_added as f64 / addition_time.as_secs_f64());
    println!();

    // 4. Build index with VRAM monitoring
    println!("📊 Test 3: Build HNSW Index with VRAM Monitoring");
    println!("------------------------------------------------");
    let start = Instant::now();
    vram_monitor.monitor_operation("hnsw_construction", || {
        collection.build_index()
    })?;
    let construction_time = start.elapsed();
    println!("  ✅ HNSW index built with VRAM monitoring: {:.3}ms", construction_time.as_millis());
    println!();

    // 5. Search with VRAM monitoring
    println!("📊 Test 4: Search with VRAM Monitoring");
    println!("--------------------------------------");
    let start = Instant::now();
    let mut search_times = Vec::new();
    
    for i in 0..std::cmp::min(50, scenario.vector_count / 100) { // Limit searches for testing
        let query_start = Instant::now();
        let query_vector = &vectors[i % scenario.vector_count];
        
        vram_monitor.monitor_operation("vector_search", || {
            collection.search(&query_vector.data, 10)
        })?;
        
        let query_time = query_start.elapsed();
        search_times.push(query_time.as_millis() as f64);
        
        if i % 10 == 0 {
            println!("  Completed {} searches...", i + 1);
        }
    }
    
    let total_search_time = start.elapsed();
    let avg_search_time = search_times.iter().sum::<f64>() / search_times.len() as f64;
    
    println!("  ✅ Completed {} searches with VRAM monitoring", search_times.len());
    println!("  Average search time: {:.3}ms", avg_search_time);
    println!("  Total search time: {:.3}s", total_search_time.as_secs_f64());
    println!();

    // 6. Validate all VRAM usage
    println!("📊 Test 5: Validate All VRAM Usage");
    println!("----------------------------------");
    let start = Instant::now();
    vram_monitor.validate_all_vram()?;
    let validation_time = start.elapsed();
    println!("  ✅ All VRAM usage validated: {:.3}ms", validation_time.as_millis());
    println!();

    // 7. Get VRAM statistics
    let vram_stats = vram_monitor.get_vram_stats();
    let memory_stats = collection.get_memory_stats();
    
    println!("📊 Test 6: VRAM Statistics");
    println!("--------------------------");
    println!("  Total VRAM allocated: {:.2} MB", vram_stats.total_allocated as f64 / 1024.0 / 1024.0);
    println!("  Peak VRAM usage: {:.2} MB", vram_stats.peak_usage as f64 / 1024.0 / 1024.0);
    println!("  Buffer count: {}", vram_stats.buffer_count);
    println!("  RAM fallback detected: {}", vram_stats.ram_fallback_detected);
    println!("  VRAM efficiency: {:.1}%", vram_stats.vram_efficiency * 100.0);
    println!("  Average buffer size: {:.2} KB", vram_stats.average_buffer_size / 1024.0);
    println!();

    Ok(VramTestResult {
        scenario: scenario.clone(),
        generation_time,
        creation_time,
        addition_time,
        construction_time,
        search_times,
        validation_time,
        vram_stats,
        memory_stats,
    })
}

#[derive(Debug, Clone)]
struct VramTestResult {
    scenario: VramTestScenario,
    generation_time: std::time::Duration,
    creation_time: std::time::Duration,
    addition_time: std::time::Duration,
    construction_time: std::time::Duration,
    search_times: Vec<f64>,
    validation_time: std::time::Duration,
    vram_stats: vectorizer::gpu::VramStats,
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
async fn generate_vram_report(
    results: &[VramTestResult],
    vram_monitor: &VramMonitor,
) -> Result<()> {
    println!("📊 Comprehensive VRAM Validation Report");
    println!("======================================");
    println!("Complete analysis of VRAM usage and performance\n");
    
    // Overall VRAM statistics
    let total_vram = results.iter().map(|r| r.vram_stats.total_allocated).sum::<usize>();
    let peak_vram = results.iter().map(|r| r.vram_stats.peak_usage).max().unwrap_or(0);
    let total_buffers = results.iter().map(|r| r.vram_stats.buffer_count).sum::<usize>();
    let ram_fallback_detected = results.iter().any(|r| r.vram_stats.ram_fallback_detected);
    
    println!("🎯 Overall VRAM Statistics");
    println!("--------------------------");
    println!("  Total VRAM allocated: {:.2} MB", total_vram as f64 / 1024.0 / 1024.0);
    println!("  Peak VRAM usage: {:.2} MB", peak_vram as f64 / 1024.0 / 1024.0);
    println!("  Total buffers: {}", total_buffers);
    println!("  RAM fallback detected: {}", ram_fallback_detected);
    println!();
    
    // Per-scenario analysis
    for result in results {
        println!("🎯 Scenario: {}", result.scenario.name);
        println!("{}", "-".repeat(result.scenario.name.len() + 10));
        
        println!("📊 Performance Metrics");
        println!("----------------------");
        println!("  Vector generation: {:.3}ms", result.generation_time.as_millis());
        println!("  Collection creation: {:.3}ms", result.creation_time.as_millis());
        println!("  Vector addition: {:.3}ms", result.addition_time.as_millis());
        println!("  HNSW construction: {:.3}ms", result.construction_time.as_millis());
        println!("  VRAM validation: {:.3}ms", result.validation_time.as_millis());
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
        
        println!("💾 VRAM Usage");
        println!("-------------");
        println!("  VRAM allocated: {:.2} MB", result.vram_stats.total_allocated as f64 / 1024.0 / 1024.0);
        println!("  Peak VRAM usage: {:.2} MB", result.vram_stats.peak_usage as f64 / 1024.0 / 1024.0);
        println!("  Buffer count: {}", result.vram_stats.buffer_count);
        println!("  RAM fallback: {}", result.vram_stats.ram_fallback_detected);
        println!("  VRAM efficiency: {:.1}%", result.vram_stats.vram_efficiency * 100.0);
        println!("  Average buffer size: {:.2} KB", result.vram_stats.average_buffer_size / 1024.0);
        println!();
        
        println!("💾 Memory Usage");
        println!("---------------");
        println!("  Vector count: {}", result.memory_stats.vector_count);
        println!("  Buffer capacity: {}", result.memory_stats.buffer_capacity);
        println!("  Used memory: {:.2} MB", result.memory_stats.used_bytes as f64 / 1024.0 / 1024.0);
        println!("  Allocated memory: {:.2} MB", result.memory_stats.allocated_bytes as f64 / 1024.0 / 1024.0);
        println!("  Memory utilization: {:.1}%", result.memory_stats.utilization * 100.0);
        println!();
        
        // VRAM validation assessment
        println!("🎯 VRAM Validation Assessment");
        println!("-----------------------------");
        if result.vram_stats.ram_fallback_detected {
            println!("  ❌ RAM fallback detected - NOT using VRAM");
        } else {
            println!("  ✅ No RAM fallback detected - using VRAM");
        }
        
        if result.vram_stats.vram_efficiency > 0.8 {
            println!("  ✅ VRAM efficiency: EXCELLENT ({:.1}%)", result.vram_stats.vram_efficiency * 100.0);
        } else if result.vram_stats.vram_efficiency > 0.6 {
            println!("  ✅ VRAM efficiency: GOOD ({:.1}%)", result.vram_stats.vram_efficiency * 100.0);
        } else {
            println!("  ⚠️ VRAM efficiency: LOW ({:.1}%)", result.vram_stats.vram_efficiency * 100.0);
        }
        
        if result.vram_stats.buffer_count > 0 {
            println!("  ✅ Buffer management: ACTIVE ({} buffers)", result.vram_stats.buffer_count);
        } else {
            println!("  ⚠️ Buffer management: INACTIVE");
        }
        
        println!();
    }
    
    // Final VRAM report
    println!("📊 Final VRAM Report");
    println!("====================");
    println!("{}", vram_monitor.generate_vram_report());
    
    // Final assessment
    println!("🎯 Final VRAM Validation Assessment");
    println!("===================================");
    if ram_fallback_detected {
        println!("❌ VRAM validation FAILED - RAM fallback detected");
        println!("   Some operations may be using RAM instead of VRAM");
    } else {
        println!("✅ VRAM validation PASSED - All operations using VRAM");
        println!("   No RAM fallback detected in any operation");
    }
    
    println!("📊 Summary Statistics");
    println!("--------------------");
    println!("  Total VRAM used: {:.2} MB", total_vram as f64 / 1024.0 / 1024.0);
    println!("  Peak VRAM usage: {:.2} MB", peak_vram as f64 / 1024.0 / 1024.0);
    println!("  Total buffers: {}", total_buffers);
    println!("  Scenarios tested: {}", results.len());
    println!("  VRAM validation: {}", if ram_fallback_detected { "FAILED" } else { "PASSED" });
    
    Ok(())
}

