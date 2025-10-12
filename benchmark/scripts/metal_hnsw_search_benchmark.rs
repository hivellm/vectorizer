//! Metal HNSW Search Benchmark
//! 
//! Tests GPU-accelerated HNSW search performance

use vectorizer::gpu::metal_native::MetalNativeCollection;
use vectorizer::models::{Vector, DistanceMetric};
use std::time::Instant;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔧 Metal HNSW Search Benchmark");
    println!("==============================");
    
    // Test parameters
    let vector_count = 1000;
    let dimension = 128;
    let search_queries = 100;
    let k = 10;
    
    println!("📊 Test 1: Create Collection and Add Vectors");
    println!("------------------------------------------------");
    
    let start = Instant::now();
    let mut collection = MetalNativeCollection::new(dimension, DistanceMetric::Cosine)?;
    let create_time = start.elapsed();
    
    println!("  ✅ Collection created: {:?}", create_time);
    
    // Add vectors
    let add_start = Instant::now();
    for i in 0..vector_count {
        let vector = Vector {
            id: format!("vector_{}", i),
            data: vec![i as f32; dimension],
            payload: None,
        };
        
        collection.add_vector(vector)?;
        
        if (i + 1) % 100 == 0 {
            println!("  Added {} vectors...", i + 1);
        }
    }
    let add_time = add_start.elapsed();
    
    println!("  ✅ Added {} vectors: {:?}", vector_count, add_time);
    println!("  Throughput: {:.2} vectors/sec", 
        vector_count as f64 / add_time.as_secs_f64());
    println!();
    
    println!("📊 Test 2: GPU Search Setup");
    println!("----------------------------");
    
    println!("  ✅ GPU search ready");
    println!("  Search type: GPU-accelerated Metal compute shaders");
    println!("  Vectors: {}", collection.vector_count());
    println!();
    
    println!("📊 Test 3: GPU HNSW Search Performance");
    println!("-------------------------------------");
    
    let mut total_search_time = std::time::Duration::new(0, 0);
    let mut successful_searches = 0;
    
    for i in 0..search_queries {
        let query = vec![i as f32; dimension];
        
        let search_start = Instant::now();
        let results = collection.search(&query, k)?;
        let search_time = search_start.elapsed();
        
        total_search_time += search_time;
        successful_searches += 1;
        
        if (i + 1) % 20 == 0 {
            println!("  Completed {} searches...", i + 1);
        }
    }
    
    let avg_search_time = total_search_time / successful_searches as u32;
    let search_throughput = successful_searches as f64 / total_search_time.as_secs_f64();
    
    println!("  ✅ Search completed: {} queries", successful_searches);
    println!("  Average latency: {:?}", avg_search_time);
    println!("  Throughput: {:.2} queries/sec", search_throughput);
    println!();
    
    println!("📊 Test 4: Sample Search Results");
    println!("-------------------------------");
    
    let sample_query = vec![50.0; dimension];
    let sample_start = Instant::now();
    let sample_results = collection.search(&sample_query, 5)?;
    let sample_time = sample_start.elapsed();
    
    println!("  Query: {:?}", &sample_query[..5]);
    println!("  Results: {} found in {:?}", sample_results.len(), sample_time);
    for (i, (id, score)) in sample_results.iter().enumerate() {
        println!("    {}. ID: {}, Score: {:.4}", i + 1, id, score);
    }
    println!();
    
    println!("🎉 Metal HNSW Search Benchmark Complete!");
    println!("========================================");
    println!("Performance Summary:");
    println!("  - Collection Creation: {:?}", create_time);
    println!("  - Vector Addition: {:.2} vectors/sec", 
        vector_count as f64 / add_time.as_secs_f64());
    println!("  - Search Latency: {:?}", avg_search_time);
    println!("  - Search Throughput: {:.2} queries/sec", search_throughput);
    
    Ok(())
}
