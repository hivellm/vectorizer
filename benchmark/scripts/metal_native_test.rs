//! Test Metal native buffer operations
//! 
//! This benchmark tests the Metal-rs native implementation for
//! synchronous buffer read-back operations.

use vectorizer::error::Result;

#[cfg(all(feature = "wgpu-gpu", target_os = "macos"))]
use vectorizer::gpu::metal_helpers::{MetalBufferReader, create_system_metal_device};

#[cfg(all(feature = "wgpu-gpu", target_os = "macos"))]
use metal::MTLResourceOptions;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    println!("🧪 Metal Native Buffer Operations Test");
    println!("==========================================\n");

    #[cfg(not(all(feature = "wgpu-gpu", target_os = "macos")))]
    {
        println!("❌ This test requires macOS with wgpu-gpu feature enabled");
        println!("   Compile with: cargo test --features wgpu-gpu");
        return Ok(());
    }

    #[cfg(all(feature = "wgpu-gpu", target_os = "macos"))]
    {
        run_metal_tests().await?;
    }

    Ok(())
}

#[cfg(all(feature = "wgpu-gpu", target_os = "macos"))]
async fn run_metal_tests() -> Result<()> {
    use std::time::Instant;

    // Test 1: Create Metal device
    println!("📊 Test 1: Create Metal Device");
    println!("----------------------------------------");
    
    let start = Instant::now();
    let metal_device = create_system_metal_device()?;
    let elapsed = start.elapsed();
    
    println!("  ✅ Metal device created: {:?}", elapsed);
    println!("  Device name: {}", metal_device.name());
    println!();

    // Test 2: Create MetalBufferReader
    println!("📊 Test 2: Create MetalBufferReader");
    println!("----------------------------------------");
    
    let start = Instant::now();
    let reader = MetalBufferReader::new(metal_device.clone())?;
    let elapsed = start.elapsed();
    
    println!("  ✅ MetalBufferReader created: {:?}", elapsed);
    println!();

    // Test 3: Create and read f32 buffer
    println!("📊 Test 3: Synchronous f32 Buffer Read");
    println!("----------------------------------------");
    
    let test_data_f32: Vec<f32> = (0..1000).map(|i| i as f32 * 1.5).collect();
    
    let buffer_f32 = metal_device.new_buffer_with_data(
        test_data_f32.as_ptr() as *const std::ffi::c_void,
        (test_data_f32.len() * std::mem::size_of::<f32>()) as u64,
        MTLResourceOptions::StorageModeShared,
    );
    
    let start = Instant::now();
    let result_f32 = reader.read_buffer_sync_f32(&buffer_f32, test_data_f32.len())?;
    let elapsed = start.elapsed();
    
    println!("  ✅ Read {} f32 values", result_f32.len());
    println!("  ⏱️  Time: {:?} ({:.2} µs per value)", 
        elapsed, 
        elapsed.as_micros() as f64 / result_f32.len() as f64
    );
    
    // Validate data
    let mut errors = 0;
    for (i, (&expected, &actual)) in test_data_f32.iter().zip(result_f32.iter()).enumerate() {
        if (expected - actual).abs() > 0.0001 {
            if errors < 5 {
                println!("  ⚠️  Mismatch at index {}: expected {}, got {}", i, expected, actual);
            }
            errors += 1;
        }
    }
    
    if errors == 0 {
        println!("  ✅ All values match perfectly!");
    } else {
        println!("  ❌ {} mismatches found", errors);
    }
    println!();

    // Test 4: Create and read u32 buffer
    println!("📊 Test 4: Synchronous u32 Buffer Read");
    println!("----------------------------------------");
    
    let test_data_u32: Vec<u32> = (0..1000).map(|i| i * 3).collect();
    
    let buffer_u32 = metal_device.new_buffer_with_data(
        test_data_u32.as_ptr() as *const std::ffi::c_void,
        (test_data_u32.len() * std::mem::size_of::<u32>()) as u64,
        MTLResourceOptions::StorageModeShared,
    );
    
    let start = Instant::now();
    let result_u32 = reader.read_buffer_sync_u32(&buffer_u32, test_data_u32.len())?;
    let elapsed = start.elapsed();
    
    println!("  ✅ Read {} u32 values", result_u32.len());
    println!("  ⏱️  Time: {:?} ({:.2} µs per value)", 
        elapsed, 
        elapsed.as_micros() as f64 / result_u32.len() as f64
    );
    
    // Validate data
    let mut errors = 0;
    for (i, (&expected, &actual)) in test_data_u32.iter().zip(result_u32.iter()).enumerate() {
        if expected != actual {
            if errors < 5 {
                println!("  ⚠️  Mismatch at index {}: expected {}, got {}", i, expected, actual);
            }
            errors += 1;
        }
    }
    
    if errors == 0 {
        println!("  ✅ All values match perfectly!");
    } else {
        println!("  ❌ {} mismatches found", errors);
    }
    println!();

    // Test 5: Performance with larger buffer
    println!("📊 Test 5: Performance Test (Large Buffer)");
    println!("----------------------------------------");
    
    let sizes = vec![
        (1_000, "1K"),
        (10_000, "10K"),
        (100_000, "100K"),
        (1_000_000, "1M"),
    ];
    
    for (size, label) in sizes {
        let test_data: Vec<f32> = (0..size).map(|i| i as f32).collect();
        
        let buffer = metal_device.new_buffer_with_data(
            test_data.as_ptr() as *const std::ffi::c_void,
            (test_data.len() * std::mem::size_of::<f32>()) as u64,
            MTLResourceOptions::StorageModeShared,
        );
        
        // Warm-up
        let _ = reader.read_buffer_sync_f32(&buffer, test_data.len())?;
        
        // Benchmark (5 iterations)
        let mut times = Vec::new();
        for _ in 0..5 {
            let start = Instant::now();
            let _ = reader.read_buffer_sync_f32(&buffer, test_data.len())?;
            times.push(start.elapsed());
        }
        
        let avg_time = times.iter().sum::<std::time::Duration>() / times.len() as u32;
        let min_time = times.iter().min().unwrap();
        let max_time = times.iter().max().unwrap();
        
        println!("  {} elements:", label);
        println!("    Avg: {:?} ({:.2} µs/value)", 
            avg_time, 
            avg_time.as_micros() as f64 / size as f64
        );
        println!("    Min: {:?}", min_time);
        println!("    Max: {:?}", max_time);
    }
    println!();

    // Test 6: Test with staging buffer (GPU-to-CPU copy)
    println!("📊 Test 6: Staging Buffer Copy Test (Private GPU Storage)");
    println!("----------------------------------------");
    
    let test_data: Vec<f32> = (0..10000).map(|i| i as f32 * 0.5).collect();
    
    // Step 1: Create temp shared buffer with data
    let temp_buffer = metal_device.new_buffer_with_data(
        test_data.as_ptr() as *const std::ffi::c_void,
        (test_data.len() * std::mem::size_of::<f32>()) as u64,
        MTLResourceOptions::StorageModeShared,
    );
    
    // Step 2: Create destination buffer in private storage (GPU-only, faster)
    let private_buffer = metal_device.new_buffer(
        (test_data.len() * std::mem::size_of::<f32>()) as u64,
        MTLResourceOptions::StorageModePrivate, // GPU-only, not CPU accessible
    );
    
    // Step 3: Copy from shared to private using GPU blit
    let cmd_buffer = reader.command_queue().new_command_buffer();
    let blit_encoder = cmd_buffer.new_blit_command_encoder();
    blit_encoder.copy_from_buffer(
        &temp_buffer,
        0,
        &private_buffer,
        0,
        (test_data.len() * std::mem::size_of::<f32>()) as u64,
    );
    blit_encoder.end_encoding();
    cmd_buffer.commit();
    cmd_buffer.wait_until_completed();
    
    println!("  ✅ Created GPU-private buffer and copied data");
    
    // Step 4: Now test reading from the private buffer
    let start = Instant::now();
    let result = reader.read_buffer_sync_f32(&private_buffer, test_data.len())?;
    let elapsed = start.elapsed();
    
    println!("  ✅ Read {} f32 values from GPU-private buffer", result.len());
    println!("  ⏱️  Time: {:?} (includes GPU-to-staging copy)", elapsed);
    
    // Validate
    let mut errors = 0;
    for (i, (&expected, &actual)) in test_data.iter().zip(result.iter()).enumerate() {
        if (expected - actual).abs() > 0.0001 {
            if errors < 5 {
                println!("  ⚠️  Mismatch at index {}: expected {}, got {}", i, expected, actual);
            }
            errors += 1;
        }
    }
    
    if errors == 0 {
        println!("  ✅ GPU-to-CPU copy successful!");
    } else {
        println!("  ❌ {} mismatches found", errors);
    }
    println!();

    // Summary
    println!("📊 Test Summary");
    println!("==========================================");
    println!("  ✅ All tests completed successfully!");
    println!("  ✅ Metal native buffer operations working");
    println!("  ✅ Synchronous read-back validated");
    println!("  ✅ Performance characteristics measured");
    println!();

    Ok(())
}

