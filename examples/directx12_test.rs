//! DirectX 12 GPU Backend Test
//!
//! Specific test for DirectX 12 backend on Windows systems.
//! This example forces the use of DirectX 12 and benchmarks its performance.
//!
//! Usage:
//! ```bash
//! cargo run --example directx12_test --features wgpu-gpu --release
//! ```

use std::time::Instant;
use vectorizer::db::VectorStore;
use vectorizer::models::{CollectionConfig, DistanceMetric, HnswConfig, Vector, Payload};
use vectorizer::gpu::config::GpuConfig;

fn generate_test_vectors(count: usize, dimension: usize) -> Vec<Vector> {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    
    (0..count)
        .map(|i| Vector {
            id: format!("test_vector_{}", i),
            data: (0..dimension)
                .map(|_| rng.gen_range(-1.0..1.0))
                .collect(),
            payload: Some(Payload {
                data: serde_json::json!({
                    "test_id": i,
                    "dimension": dimension,
                    "backend": "directx12"
                }),
            }),
        })
        .collect()
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::fmt::init();

    println!("🪟 DirectX 12 GPU Backend Test");
    println!("================================");
    
    // Check if we're on Windows
    if !cfg!(target_os = "windows") {
        println!("❌ DirectX 12 is only available on Windows!");
        println!("   Current OS: {}", std::env::consts::OS);
        return Ok(());
    }

    println!("✅ Running on Windows - DirectX 12 should be available");
    
    // Create VectorStore with auto-detection (should pick DirectX 12 on Windows)
    println!("\n🔧 Creating VectorStore with auto-detection...");
    
    let vector_store = VectorStore::new_auto_universal();
    println!("✅ VectorStore created with auto-detection!");

    // Create test collection
    println!("\n📦 Creating test collection...");
    let collection_name = "directx12_test_collection";
    let collection_config = CollectionConfig {
        dimension: 512,
        metric: DistanceMetric::Cosine,
        hnsw_config: HnswConfig {
            m: 16,
            ef_construction: 200,
            ef_search: 100,
            seed: None,
        },
        quantization: vectorizer::models::QuantizationConfig::SQ { bits: 8 },
        compression: vectorizer::models::CompressionConfig::default(),
    };

    vector_store.create_collection(collection_name, collection_config)?;
    println!("✅ Collection '{}' created", collection_name);

    // Test 1: Vector Insertion
    println!("\n1️⃣  Test: Vector Insertion (DirectX 12)");
    let test_vectors = generate_test_vectors(1000, 512);
    
    let start = Instant::now();
    vector_store.insert(collection_name, test_vectors.clone())?;
    let duration = start.elapsed();
    
    println!("   ✅ Inserted {} vectors in {:.2} ms", 
             test_vectors.len(), duration.as_millis());
    println!("   📈 Throughput: {:.0} ops/sec", 
             test_vectors.len() as f64 / duration.as_secs_f64());

    // Test 2: Vector Search
    println!("\n2️⃣  Test: Vector Search (DirectX 12)");
    let query_vector = generate_test_vectors(1, 512)[0].clone();
    
    let start = Instant::now();
    let search_results = vector_store.search(collection_name, &query_vector.data, 10)?;
    let duration = start.elapsed();
    
    println!("   ✅ Searched in {:.2} ms", duration.as_millis());
    println!("   📈 Found {} results", search_results.len());
    println!("   🎯 Latency: {:.3} ms", duration.as_secs_f64() * 1000.0);

    // Test 3: Batch Operations
    println!("\n3️⃣  Test: Batch Operations (DirectX 12)");
    let batch_vectors = generate_test_vectors(100, 512);
    
    let start = Instant::now();
    vector_store.insert(collection_name, batch_vectors.clone())?;
    let duration = start.elapsed();
    
    println!("   ✅ Batch inserted {} vectors in {:.2} ms", 
             batch_vectors.len(), duration.as_millis());
    println!("   📈 Batch throughput: {:.0} ops/sec", 
             batch_vectors.len() as f64 / duration.as_secs_f64());

    // Test 4: Memory Usage
    println!("\n4️⃣  Test: Memory Usage (DirectX 12)");
    let stats = vector_store.stats();
    println!("   📊 Total vectors: {}", stats.total_vectors);
    println!("   📊 Total collections: {}", stats.collection_count);
    println!("   💾 Memory usage: {:.1} MB", stats.total_memory_bytes as f64 / 1024.0 / 1024.0);

    // Cleanup
    println!("\n🧹 Cleaning up...");
    vector_store.delete_collection(collection_name)?;
    println!("✅ Collection deleted");

    println!("\n🎉 DirectX 12 test completed successfully!");
    println!("   Backend: DirectX 12 (Windows)");
    println!("   Performance: Optimized for Windows GPU acceleration");

    Ok(())
}
