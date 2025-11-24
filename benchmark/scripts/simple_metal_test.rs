//! Simple Metal Native Test
//!
//! Basic test to validate Metal Native collection creation and vector addition using hive-gpu

#[cfg(target_os = "macos")]
use std::time::Instant;

#[cfg(target_os = "macos")]
use hive_gpu::metal::{MetalNativeContext, MetalNativeVectorStorage};
#[cfg(target_os = "macos")]
use hive_gpu::{GpuContext, GpuDistanceMetric, GpuVector, GpuVectorStorage};

#[cfg(target_os = "macos")]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔧 Simple Metal Native Test");
    println!("==========================");

    // Test parameters
    let vector_count = 100;
    let dimension = 128;

    println!("📊 Test 1: Create Collection");
    println!("----------------------------");

    let start = Instant::now();
    let context = MetalNativeContext::new()?;
    let mut storage = context.create_storage(dimension, GpuDistanceMetric::Cosine)?;
    let create_time = start.elapsed();

    println!("  ✅ Collection created: {:?}", create_time);
    println!("  Dimension: {}D", dimension);
    println!("  Metric: {:?}", GpuDistanceMetric::Cosine);
    println!();

    println!("📊 Test 2: Add Vectors");
    println!("----------------------");

    let start = Instant::now();
    for i in 0..vector_count {
        let vector = GpuVector {
            id: format!("vector_{}", i),
            data: vec![i as f32; dimension],
            metadata: std::collections::HashMap::new(),
        };

        storage.add_vectors(&[vector])?;

        if (i + 1) % 10 == 0 {
            println!("  Added {} vectors...", i + 1);
        }
    }
    let add_time = start.elapsed();

    println!("  ✅ Added {} vectors: {:?}", vector_count, add_time);
    println!(
        "  Throughput: {:.2} vectors/sec",
        vector_count as f64 / add_time.as_secs_f64()
    );
    println!();

    println!("📊 Test 3: Basic Search");
    println!("-----------------------");

    let query = vec![50.0; dimension];
    let start = Instant::now();
    let results = storage.search(&query, 5)?;
    let search_time = start.elapsed();

    println!("  ✅ Search completed: {:?}", search_time);
    println!("  Results: {} found", results.len());
    for (i, result) in results.iter().enumerate() {
        println!(
            "    {}. ID: {}, Score: {:.4}",
            i + 1,
            result.id,
            result.score
        );
    }
    println!();

    println!("🎉 All tests passed!");
    Ok(())
}

#[cfg(not(target_os = "macos"))]
fn main() {
    eprintln!("⚠️  This benchmark is only available on macOS (Metal backend)");
    std::process::exit(1);
}
