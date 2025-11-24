//! Simple Metal Native Test
//!
//! Basic test to validate Metal Native collection creation and vector addition using hive-gpu

#[cfg(target_os = "macos")]
use std::time::Instant;
use tracing::{info, error, warn, debug};

#[cfg(target_os = "macos")]
use hive_gpu::metal::{MetalNativeContext, MetalNativeVectorStorage};
#[cfg(target_os = "macos")]
use hive_gpu::{GpuContext, GpuDistanceMetric, GpuVector, GpuVectorStorage};

#[cfg(target_os = "macos")]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing::info!("🔧 Simple Metal Native Test");
    tracing::info!("==========================");

    // Test parameters
    let vector_count = 100;
    let dimension = 128;

    tracing::info!("📊 Test 1: Create Collection");
    tracing::info!("----------------------------");

    let start = Instant::now();
    let context = MetalNativeContext::new()?;
    let mut storage = context.create_storage(dimension, GpuDistanceMetric::Cosine)?;
    let create_time = start.elapsed();

    tracing::info!("  ✅ Collection created: {:?}", create_time);
    tracing::info!("  Dimension: {}D", dimension);
    tracing::info!("  Metric: {:?}", GpuDistanceMetric::Cosine);
    tracing::info!();

    tracing::info!("📊 Test 2: Add Vectors");
    tracing::info!("----------------------");

    let start = Instant::now();
    for i in 0..vector_count {
        let vector = GpuVector {
            id: format!("vector_{}", i),
            data: vec![i as f32; dimension],
            metadata: std::collections::HashMap::new(),
        };

        storage.add_vectors(&[vector])?;

        if (i + 1) % 10 == 0 {
            tracing::info!("  Added {} vectors...", i + 1);
        }
    }
    let add_time = start.elapsed();

    tracing::info!("  ✅ Added {} vectors: {:?}", vector_count, add_time);
    tracing::info!(
        "  Throughput: {:.2} vectors/sec",
        vector_count as f64 / add_time.as_secs_f64()
    );
    tracing::info!();

    tracing::info!("📊 Test 3: Basic Search");
    tracing::info!("-----------------------");

    let query = vec![50.0; dimension];
    let start = Instant::now();
    let results = storage.search(&query, 5)?;
    let search_time = start.elapsed();

    tracing::info!("  ✅ Search completed: {:?}", search_time);
    tracing::info!("  Results: {} found", results.len());
    for (i, result) in results.iter().enumerate() {
        tracing::info!(
            "    {}. ID: {}, Score: {:.4}",
            i + 1,
            result.id,
            result.score
        );
    }
    tracing::info!();

    tracing::info!("🎉 All tests passed!");
    Ok(())
}

#[cfg(not(target_os = "macos"))]
fn main() {
    etracing::info!("⚠️  This benchmark is only available on macOS (Metal backend)");
    std::process::exit(1);
}
