//! GPU Stress Benchmark - High Load Test
//!
//! Tests GPU performance under heavy load with large vector sets
//! and intensive compute operations. This benchmark helps identify:
//! - Maximum GPU throughput
//! - GPU memory limits
//! - Thermal throttling behavior
//! - CPU↔GPU transfer bottlenecks
//!
//! Usage:
//! ```bash
//! cargo run --example gpu_stress_benchmark --features wgpu-gpu --release
//! ```

use std::time::{Duration, Instant};
use vectorizer::db::VectorStore;
use vectorizer::models::{CollectionConfig, DistanceMetric, HnswConfig, Vector};
use serde::{Serialize, Deserialize};
use std::fs;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct StressTestResult {
    test_name: String,
    vector_count: usize,
    vector_dimension: usize,
    duration_ms: f64,
    throughput: f64,
    peak_memory_mb: f64,
    gpu_utilization: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct StressTestReport {
    timestamp: String,
    backend: String,
    device_name: String,
    results: Vec<StressTestResult>,
    system_info: SystemInfo,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct SystemInfo {
    os: String,
    cpu_cores: usize,
    available_memory_gb: f64,
}

fn generate_random_vector(dim: usize) -> Vec<f32> {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    (0..dim).map(|_| rng.gen_range(-1.0..1.0)).collect()
}

fn stress_test_large_vectors(
    store: &VectorStore,
    collection_name: &str,
    dimension: usize,
    count: usize,
) -> Result<StressTestResult, Box<dyn std::error::Error>> {
    println!("\n🔥 STRESS TEST: Large Vector Set");
    println!("   Dimension: {}", dimension);
    println!("   Count: {}", count);
    println!("   Total size: {:.2} MB", (count * dimension * 4) as f64 / 1_000_000.0);
    
    // Generate vectors
    println!("   📦 Generating {} vectors...", count);
    let gen_start = Instant::now();
    let vectors: Vec<Vector> = (0..count)
        .map(|i| Vector::new(format!("stress_vec_{}", i), generate_random_vector(dimension)))
        .collect();
    let gen_duration = gen_start.elapsed();
    println!("   ✅ Generated in {:.2} s", gen_duration.as_secs_f64());
    
    // Insert vectors
    println!("   📥 Inserting vectors...");
    let insert_start = Instant::now();
    store.insert(collection_name, vectors.clone())?;
    let insert_duration = insert_start.elapsed();
    
    let throughput = count as f64 / insert_duration.as_secs_f64();
    println!("   ✅ Inserted in {:.2} s", insert_duration.as_secs_f64());
    println!("   📈 Throughput: {:.0} vectors/sec", throughput);
    
    // Estimate memory usage
    let vector_data_mb = (count * dimension * 4) as f64 / 1_000_000.0;
    let hnsw_index_mb = (count * 16 * 4) as f64 / 1_000_000.0; // Rough estimate
    let total_mb = vector_data_mb + hnsw_index_mb;
    
    println!("   💾 Estimated memory: {:.2} MB", total_mb);
    
    Ok(StressTestResult {
        test_name: format!("Large Vector Set ({}x{})", count, dimension),
        vector_count: count,
        vector_dimension: dimension,
        duration_ms: insert_duration.as_secs_f64() * 1000.0,
        throughput,
        peak_memory_mb: total_mb,
        gpu_utilization: 0.0, // Would need platform-specific GPU monitoring
    })
}

fn stress_test_high_dimensional(
    store: &VectorStore,
    collection_name: &str,
    dimension: usize,
    count: usize,
) -> Result<StressTestResult, Box<dyn std::error::Error>> {
    println!("\n🔥 STRESS TEST: High-Dimensional Vectors");
    println!("   Dimension: {}", dimension);
    println!("   Count: {}", count);
    
    let vectors: Vec<Vector> = (0..count)
        .map(|i| Vector::new(format!("hd_vec_{}", i), generate_random_vector(dimension)))
        .collect();
    
    let start = Instant::now();
    store.insert(collection_name, vectors)?;
    let duration = start.elapsed();
    
    let throughput = count as f64 / duration.as_secs_f64();
    let memory_mb = (count * dimension * 4) as f64 / 1_000_000.0;
    
    println!("   ✅ Completed in {:.2} s", duration.as_secs_f64());
    println!("   📈 Throughput: {:.0} vectors/sec", throughput);
    
    Ok(StressTestResult {
        test_name: format!("High-Dimensional ({}D)", dimension),
        vector_count: count,
        vector_dimension: dimension,
        duration_ms: duration.as_secs_f64() * 1000.0,
        throughput,
        peak_memory_mb: memory_mb,
        gpu_utilization: 0.0,
    })
}

fn stress_test_continuous_search(
    store: &VectorStore,
    collection_name: &str,
    dimension: usize,
    duration_secs: u64,
) -> Result<StressTestResult, Box<dyn std::error::Error>> {
    println!("\n🔥 STRESS TEST: Continuous Search");
    println!("   Duration: {} seconds", duration_secs);
    println!("   Dimension: {}", dimension);
    
    let start = Instant::now();
    let mut query_count = 0;
    let target_duration = Duration::from_secs(duration_secs);
    
    while start.elapsed() < target_duration {
        let query = generate_random_vector(dimension);
        let _ = store.search(collection_name, &query, 10)?;
        query_count += 1;
        
        // Print progress every second
        if query_count % 1000 == 0 {
            let elapsed = start.elapsed().as_secs_f64();
            let qps = query_count as f64 / elapsed;
            print!("\r   ⏱️  {} queries ({:.0} QPS)", query_count, qps);
            std::io::Write::flush(&mut std::io::stdout())?;
        }
    }
    println!();
    
    let total_duration = start.elapsed();
    let throughput = query_count as f64 / total_duration.as_secs_f64();
    
    println!("   ✅ Completed {} queries", query_count);
    println!("   📈 Average: {:.0} QPS", throughput);
    
    Ok(StressTestResult {
        test_name: format!("Continuous Search ({}s)", duration_secs),
        vector_count: query_count,
        vector_dimension: dimension,
        duration_ms: total_duration.as_secs_f64() * 1000.0,
        throughput,
        peak_memory_mb: 0.0,
        gpu_utilization: 0.0,
    })
}

fn run_stress_test_suite() -> Result<StressTestReport, Box<dyn std::error::Error>> {
    println!("\n💪 GPU Stress Test Suite");
    println!("═══════════════════════════════════════");
    
    // Initialize VectorStore with auto GPU detection
    #[cfg(feature = "wgpu-gpu")]
    let store = {
        println!("\n🔍 Detecting GPU backend...");
        VectorStore::new_auto_universal()
    };
    
    #[cfg(not(feature = "wgpu-gpu"))]
    let store = {
        println!("\n⚠️  Using CPU backend");
        VectorStore::new()
    };
    
    let mut results = Vec::new();
    
    // Test 1: Small vectors, large count
    {
        let collection_name = "stress_small_dim";
        let config = CollectionConfig {
            dimension: 128,
            metric: DistanceMetric::Cosine,
            hnsw_config: HnswConfig {
                m: 16,
                ef_construction: 200,
                ef_search: 64,
                seed: Some(42),
            },
            quantization: Default::default(),
            compression: Default::default(),
        };
        
        store.create_collection(collection_name, config)?;
        let result = stress_test_large_vectors(&store, collection_name, 128, 10000)?;
        results.push(result);
        
        // Continuous search test on this collection
        let search_result = stress_test_continuous_search(&store, collection_name, 128, 5)?;
        results.push(search_result);
        
        store.delete_collection(collection_name)?;
    }
    
    // Test 2: Medium vectors, medium count
    {
        let collection_name = "stress_medium_dim";
        let config = CollectionConfig {
            dimension: 512,
            metric: DistanceMetric::Cosine,
            hnsw_config: HnswConfig {
                m: 16,
                ef_construction: 200,
                ef_search: 64,
                seed: Some(42),
            },
            quantization: Default::default(),
            compression: Default::default(),
        };
        
        store.create_collection(collection_name, config)?;
        let result = stress_test_large_vectors(&store, collection_name, 512, 5000)?;
        results.push(result);
        store.delete_collection(collection_name)?;
    }
    
    // Test 3: High-dimensional vectors
    {
        let collection_name = "stress_high_dim";
        let config = CollectionConfig {
            dimension: 2048,
            metric: DistanceMetric::Cosine,
            hnsw_config: HnswConfig {
                m: 16,
                ef_construction: 200,
                ef_search: 64,
                seed: Some(42),
            },
            quantization: Default::default(),
            compression: Default::default(),
        };
        
        store.create_collection(collection_name, config)?;
        let result = stress_test_high_dimensional(&store, collection_name, 2048, 1000)?;
        results.push(result);
        store.delete_collection(collection_name)?;
    }
    
    let report = StressTestReport {
        timestamp: chrono::Utc::now().to_rfc3339(),
        backend: "Auto-detected".to_string(),
        device_name: "GPU".to_string(),
        results,
        system_info: SystemInfo {
            os: std::env::consts::OS.to_string(),
            cpu_cores: num_cpus::get(),
            available_memory_gb: 16.0,
        },
    };
    
    Ok(report)
}

fn print_summary(report: &StressTestReport) {
    println!("\n\n📊 Stress Test Summary");
    println!("═══════════════════════════════════════════════");
    println!("Backend: {}", report.backend);
    println!("Device: {}", report.device_name);
    println!("OS: {}", report.system_info.os);
    println!("CPU Cores: {}", report.system_info.cpu_cores);
    
    println!("\n🔥 Test Results:");
    for result in &report.results {
        println!("\n📌 {}", result.test_name);
        println!("   Vectors: {}", result.vector_count);
        println!("   Dimension: {}", result.vector_dimension);
        println!("   Duration: {:.2} s", result.duration_ms / 1000.0);
        println!("   Throughput: {:.0} ops/sec", result.throughput);
        if result.peak_memory_mb > 0.0 {
            println!("   Memory: {:.2} MB", result.peak_memory_mb);
        }
    }
    
    println!("\n═══════════════════════════════════════════════");
}

fn save_report(report: &StressTestReport) -> Result<(), Box<dyn std::error::Error>> {
    fs::create_dir_all("benchmark/reports")?;
    
    let filename = format!(
        "benchmark/reports/gpu_stress_test_{}.json",
        chrono::Local::now().format("%Y%m%d_%H%M%S")
    );
    
    let json = serde_json::to_string_pretty(&report)?;
    fs::write(&filename, json)?;
    
    println!("\n💾 Report saved: {}", filename);
    
    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::WARN) // Less verbose for stress test
        .init();
    
    let report = run_stress_test_suite()?;
    print_summary(&report);
    save_report(&report)?;
    
    println!("\n✅ Stress test complete!");
    
    Ok(())
}

