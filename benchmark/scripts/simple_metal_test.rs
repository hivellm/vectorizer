//! Simple Metal Native Test
//! 
//! Basic test to validate Metal Native collection creation and vector addition

use vectorizer::gpu::metal_native::MetalNativeCollection;
use vectorizer::models::{Vector, DistanceMetric};
use std::time::Instant;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔧 Simple Metal Native Test");
    println!("==========================");
    
    // Test parameters
    let vector_count = 100;
    let dimension = 128;
    
    println!("📊 Test 1: Create Collection");
    println!("----------------------------");
    
    let start = Instant::now();
    let mut collection = MetalNativeCollection::new(dimension, DistanceMetric::Cosine)?;
    let create_time = start.elapsed();
    
    println!("  ✅ Collection created: {:?}", create_time);
    println!("  Dimension: {}D", dimension);
    println!("  Metric: {:?}", DistanceMetric::Cosine);
    println!();
    
    println!("📊 Test 2: Add Vectors");
    println!("----------------------");
    
    let start = Instant::now();
    for i in 0..vector_count {
        let vector = Vector {
            id: format!("vector_{}", i),
            data: vec![i as f32; dimension],
            payload: None,
        };
        
        collection.add_vector(vector)?;
        
        if (i + 1) % 10 == 0 {
            println!("  Added {} vectors...", i + 1);
        }
    }
    let add_time = start.elapsed();
    
    println!("  ✅ Added {} vectors: {:?}", vector_count, add_time);
    println!("  Throughput: {:.2} vectors/sec", 
        vector_count as f64 / add_time.as_secs_f64());
    println!();
    
    println!("📊 Test 3: Basic Search");
    println!("-----------------------");
    
    let query = vec![50.0; dimension];
    let start = Instant::now();
    let results = collection.search(&query, 5)?;
    let search_time = start.elapsed();
    
    println!("  ✅ Search completed: {:?}", search_time);
    println!("  Results: {} found", results.len());
    for (i, (id, score)) in results.iter().enumerate() {
        println!("    {}. ID: {}, Score: {:.4}", i + 1, id, score);
    }
    println!();
    
    println!("🎉 All tests passed!");
    Ok(())
}

