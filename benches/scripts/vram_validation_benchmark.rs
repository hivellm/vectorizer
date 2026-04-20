//! VRAM Validation Benchmark
//! 
//! This benchmark validates that all operations are truly using VRAM
//! and not falling back to RAM, with comprehensive monitoring and reporting.

use vectorizer::error::Result;
use tracing::{info, error, warn, debug};
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

    tracing::info!("🔍 VRAM Validation Benchmark");
    tracing::info!("============================");
    tracing::info!("Comprehensive validation that all operations use VRAM, not RAM");
    tracing::info!("Includes performance monitoring and fallback detection\n");

    #[cfg(not(target_os = "macos"))]
    {
        tracing::info!("❌ This benchmark requires macOS with Metal support");
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

    tracing::info!("🔍 Phase 1: Device VRAM Validation");
    tracing::info!("===================================");
    
    // 1. Validate Metal device VRAM capabilities
    let device = metal::Device::system_default().unwrap();
    VramValidator::validate_device_vram(&device)?;
    tracing::info!("  ✅ Device VRAM validation passed\n");
    
    // 2. Benchmark VRAM vs RAM performance
    tracing::info!("🔍 Phase 2: VRAM vs RAM Performance Benchmark");
    tracing::info!("==============================================");
    let benchmark_result = VramValidator::benchmark_vram_vs_ram(&device)?;
    
    if benchmark_result.vram_faster {
        tracing::info!("  ✅ VRAM is faster than RAM: {:.2}x speedup", benchmark_result.performance_ratio);
    } else {
        tracing::info!("  ⚠️ VRAM is slower than RAM: {:.2}x slowdown", 1.0 / benchmark_result.performance_ratio);
    }
    tracing::info!();
    
    // 3. Create VRAM monitor
    tracing::info!("🔍 Phase 3: VRAM Monitor Setup");
    tracing::info!("==============================");
    let mut vram_monitor = VramMonitor::new(device.clone());
    vram_monitor.force_vram_validation()?;
    tracing::info!("  ✅ VRAM monitor initialized and validated\n");
    
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
        tracing::info!("🔍 Phase 4: {}", scenario.name);
        tracing::info!("{}", "=".repeat(scenario.name.len() + 15));
        
        let result = run_vram_test_scenario(&mut vram_monitor, &scenario).await?;
        all_results.push(result);
        
        tracing::info!();
    }
    
    // 5. Generate comprehensive VRAM report
    tracing::info!("🔍 Phase 5: Comprehensive VRAM Report");
    tracing::info!("======================================");
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

    tracing::info!("📊 Test Parameters");
    tracing::info!("------------------");
    tracing::info!("  Dimension: {}", scenario.dimension);
    tracing::info!("  Vector count: {}", scenario.vector_count);
    tracing::info!("  Batch size: {}", scenario.batch_size);
    tracing::info!();

    // 1. Generate test vectors
    tracing::info!("🔧 Generating test vectors...");
    let start = Instant::now();
    let vectors = generate_test_vectors(scenario.vector_count, scenario.dimension);
    let generation_time = start.elapsed();
    tracing::info!("  ✅ Generated {} vectors in {:.3}ms", scenario.vector_count, generation_time.as_millis());
    tracing::info!();

    // 2. Create collection with VRAM monitoring
    tracing::info!("📊 Test 1: Create Collection with VRAM Monitoring");
    tracing::info!("--------------------------------------------------");
    let start = Instant::now();
    let mut collection = vram_monitor.monitor_operation("collection_creation", || {
        OptimizedMetalNativeCollection::new(
            scenario.dimension,
            DistanceMetric::Cosine,
            scenario.vector_count,
        )
    })?;
    let creation_time = start.elapsed();
    tracing::info!("  ✅ Collection created with VRAM monitoring: {:.3}ms", creation_time.as_millis());
    tracing::info!();

    // 3. Add vectors with VRAM monitoring
    tracing::info!("📊 Test 2: Add Vectors with VRAM Monitoring");
    tracing::info!("-------------------------------------------");
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
        tracing::info!("  Added batch {} vectors... ({:.3}ms)", total_added, batch_time.as_millis());
    }
    
    let addition_time = start.elapsed();
    tracing::info!("  ✅ Added {} vectors with VRAM monitoring: {:.3}ms", total_added, addition_time.as_millis());
    tracing::info!("  Throughput: {:.2} vectors/sec", total_added as f64 / addition_time.as_secs_f64());
    tracing::info!();

    // 4. Build index with VRAM monitoring
    tracing::info!("📊 Test 3: Build HNSW Index with VRAM Monitoring");
    tracing::info!("------------------------------------------------");
    let start = Instant::now();
    vram_monitor.monitor_operation("hnsw_construction", || {
        collection.build_index()
    })?;
    let construction_time = start.elapsed();
    tracing::info!("  ✅ HNSW index built with VRAM monitoring: {:.3}ms", construction_time.as_millis());
    tracing::info!();

    // 5. Search with VRAM monitoring
    tracing::info!("📊 Test 4: Search with VRAM Monitoring");
    tracing::info!("--------------------------------------");
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
            tracing::info!("  Completed {} searches...", i + 1);
        }
    }
    
    let total_search_time = start.elapsed();
    let avg_search_time = search_times.iter().sum::<f64>() / search_times.len() as f64;
    
    tracing::info!("  ✅ Completed {} searches with VRAM monitoring", search_times.len());
    tracing::info!("  Average search time: {:.3}ms", avg_search_time);
    tracing::info!("  Total search time: {:.3}s", total_search_time.as_secs_f64());
    tracing::info!();

    // 6. Validate all VRAM usage
    tracing::info!("📊 Test 5: Validate All VRAM Usage");
    tracing::info!("----------------------------------");
    let start = Instant::now();
    vram_monitor.validate_all_vram()?;
    let validation_time = start.elapsed();
    tracing::info!("  ✅ All VRAM usage validated: {:.3}ms", validation_time.as_millis());
    tracing::info!();

    // 7. Get VRAM statistics
    let vram_stats = vram_monitor.get_vram_stats();
    let memory_stats = collection.get_memory_stats();
    
    tracing::info!("📊 Test 6: VRAM Statistics");
    tracing::info!("--------------------------");
    tracing::info!("  Total VRAM allocated: {:.2} MB", vram_stats.total_allocated as f64 / 1024.0 / 1024.0);
    tracing::info!("  Peak VRAM usage: {:.2} MB", vram_stats.peak_usage as f64 / 1024.0 / 1024.0);
    tracing::info!("  Buffer count: {}", vram_stats.buffer_count);
    tracing::info!("  RAM fallback detected: {}", vram_stats.ram_fallback_detected);
    tracing::info!("  VRAM efficiency: {:.1}%", vram_stats.vram_efficiency * 100.0);
    tracing::info!("  Average buffer size: {:.2} KB", vram_stats.average_buffer_size / 1024.0);
    tracing::info!();

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
    tracing::info!("📊 Comprehensive VRAM Validation Report");
    tracing::info!("======================================");
    tracing::info!("Complete analysis of VRAM usage and performance\n");
    
    // Overall VRAM statistics
    let total_vram = results.iter().map(|r| r.vram_stats.total_allocated).sum::<usize>();
    let peak_vram = results.iter().map(|r| r.vram_stats.peak_usage).max().unwrap_or(0);
    let total_buffers = results.iter().map(|r| r.vram_stats.buffer_count).sum::<usize>();
    let ram_fallback_detected = results.iter().any(|r| r.vram_stats.ram_fallback_detected);
    
    tracing::info!("🎯 Overall VRAM Statistics");
    tracing::info!("--------------------------");
    tracing::info!("  Total VRAM allocated: {:.2} MB", total_vram as f64 / 1024.0 / 1024.0);
    tracing::info!("  Peak VRAM usage: {:.2} MB", peak_vram as f64 / 1024.0 / 1024.0);
    tracing::info!("  Total buffers: {}", total_buffers);
    tracing::info!("  RAM fallback detected: {}", ram_fallback_detected);
    tracing::info!();
    
    // Per-scenario analysis
    for result in results {
        tracing::info!("🎯 Scenario: {}", result.scenario.name);
        tracing::info!("{}", "-".repeat(result.scenario.name.len() + 10));
        
        tracing::info!("📊 Performance Metrics");
        tracing::info!("----------------------");
        tracing::info!("  Vector generation: {:.3}ms", result.generation_time.as_millis());
        tracing::info!("  Collection creation: {:.3}ms", result.creation_time.as_millis());
        tracing::info!("  Vector addition: {:.3}ms", result.addition_time.as_millis());
        tracing::info!("  HNSW construction: {:.3}ms", result.construction_time.as_millis());
        tracing::info!("  VRAM validation: {:.3}ms", result.validation_time.as_millis());
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
        
        tracing::info!("💾 VRAM Usage");
        tracing::info!("-------------");
        tracing::info!("  VRAM allocated: {:.2} MB", result.vram_stats.total_allocated as f64 / 1024.0 / 1024.0);
        tracing::info!("  Peak VRAM usage: {:.2} MB", result.vram_stats.peak_usage as f64 / 1024.0 / 1024.0);
        tracing::info!("  Buffer count: {}", result.vram_stats.buffer_count);
        tracing::info!("  RAM fallback: {}", result.vram_stats.ram_fallback_detected);
        tracing::info!("  VRAM efficiency: {:.1}%", result.vram_stats.vram_efficiency * 100.0);
        tracing::info!("  Average buffer size: {:.2} KB", result.vram_stats.average_buffer_size / 1024.0);
        tracing::info!();
        
        tracing::info!("💾 Memory Usage");
        tracing::info!("---------------");
        tracing::info!("  Vector count: {}", result.memory_stats.vector_count);
        tracing::info!("  Buffer capacity: {}", result.memory_stats.buffer_capacity);
        tracing::info!("  Used memory: {:.2} MB", result.memory_stats.used_bytes as f64 / 1024.0 / 1024.0);
        tracing::info!("  Allocated memory: {:.2} MB", result.memory_stats.allocated_bytes as f64 / 1024.0 / 1024.0);
        tracing::info!("  Memory utilization: {:.1}%", result.memory_stats.utilization * 100.0);
        tracing::info!();
        
        // VRAM validation assessment
        tracing::info!("🎯 VRAM Validation Assessment");
        tracing::info!("-----------------------------");
        if result.vram_stats.ram_fallback_detected {
            tracing::info!("  ❌ RAM fallback detected - NOT using VRAM");
        } else {
            tracing::info!("  ✅ No RAM fallback detected - using VRAM");
        }
        
        if result.vram_stats.vram_efficiency > 0.8 {
            tracing::info!("  ✅ VRAM efficiency: EXCELLENT ({:.1}%)", result.vram_stats.vram_efficiency * 100.0);
        } else if result.vram_stats.vram_efficiency > 0.6 {
            tracing::info!("  ✅ VRAM efficiency: GOOD ({:.1}%)", result.vram_stats.vram_efficiency * 100.0);
        } else {
            tracing::info!("  ⚠️ VRAM efficiency: LOW ({:.1}%)", result.vram_stats.vram_efficiency * 100.0);
        }
        
        if result.vram_stats.buffer_count > 0 {
            tracing::info!("  ✅ Buffer management: ACTIVE ({} buffers)", result.vram_stats.buffer_count);
        } else {
            tracing::info!("  ⚠️ Buffer management: INACTIVE");
        }
        
        tracing::info!();
    }
    
    // Final VRAM report
    tracing::info!("📊 Final VRAM Report");
    tracing::info!("====================");
    tracing::info!("{}", vram_monitor.generate_vram_report());
    
    // Final assessment
    tracing::info!("🎯 Final VRAM Validation Assessment");
    tracing::info!("===================================");
    if ram_fallback_detected {
        tracing::info!("❌ VRAM validation FAILED - RAM fallback detected");
        tracing::info!("   Some operations may be using RAM instead of VRAM");
    } else {
        tracing::info!("✅ VRAM validation PASSED - All operations using VRAM");
        tracing::info!("   No RAM fallback detected in any operation");
    }
    
    tracing::info!("📊 Summary Statistics");
    tracing::info!("--------------------");
    tracing::info!("  Total VRAM used: {:.2} MB", total_vram as f64 / 1024.0 / 1024.0);
    tracing::info!("  Peak VRAM usage: {:.2} MB", peak_vram as f64 / 1024.0 / 1024.0);
    tracing::info!("  Total buffers: {}", total_buffers);
    tracing::info!("  Scenarios tested: {}", results.len());
    tracing::info!("  VRAM validation: {}", if ram_fallback_detected { "FAILED" } else { "PASSED" });
    
    Ok(())
}

