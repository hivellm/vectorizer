//! Quick Benchmark - CPU only, minimal tests
//!
//! Fast benchmark to validate basic performance

use anyhow::Result;
use tracing::{info, error, warn, debug};
use std::time::Instant;
use vectorizer::VectorStore;
use vectorizer::models::{CollectionConfig, DistanceMetric, HnswConfig, Vector};

fn generate_random_vectors(count: usize, dimension: usize) -> Vec<Vector> {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    
    (0..count)
        .map(|i| {
            let data: Vec<f32> = (0..dimension)
                .map(|_| rng.gen_range(-1.0..1.0))
                .collect();
            
            Vector {
                id: format!("vec_{}", i),
                data,
                payload: None,
            }
        })
        .collect()
}

fn main() -> Result<()> {
    tracing::info!("⚡ Quick Benchmark - CPU Performance");
    tracing::info!("====================================\n");
    
    // Test configurations (reduced)
    let test_configs = vec![
        (100, 128, "Small dataset, low dimension"),
        (500, 128, "Medium dataset, low dimension"),
    ];
    
    for (vector_count, dimension, description) in test_configs {
        tracing::info!("📊 Test: {} ({} vectors, {}D)", description, vector_count, dimension);
        
        let config = CollectionConfig {
            dimension,
            metric: DistanceMetric::Cosine,
            hnsw_config: HnswConfig {
                m: 16,
                ef_construction: 200,
                ef_search: 100,
                seed: Some(42),
            },
            ..Default::default()
        };
        
        let vectors = generate_random_vectors(vector_count, dimension);
        tracing::info!("  • Generated {} vectors", vectors.len());
        
        // Create store and collection
        let store = VectorStore::new();
        let collection_name = format!("bench_{}", uuid::Uuid::new_v4());
        
        let start = Instant::now();
        store.create_collection(&collection_name, config.clone())?;
        tracing::info!("  ✅ Collection created: {:.2}ms", start.elapsed().as_secs_f64() * 1000.0);
        
        // IMPORTANT: Use collection in a scope to release the lock before delete
        {
            let mut collection = store.get_collection_mut(&collection_name)?;
            
            // Add vectors
            let insert_start = Instant::now();
            for vector in &vectors {
                collection.add_vector(vector.id.clone(), vector.clone())?;
            }
            let insert_time = insert_start.elapsed();
            let insert_ms = insert_time.as_secs_f64() * 1000.0;
            
            tracing::info!("  ✅ Insertion: {:.2}ms ({:.0} vectors/sec)", 
                     insert_ms, 
                     vectors.len() as f64 / (insert_ms / 1000.0));
            
            // Search benchmark (reduced to 10 queries)
            let query = vectors[0].data.clone();
            let search_start = Instant::now();
            
            for _ in 0..10 {
                let results = collection.search(&query, 10)?;
                assert!(!results.is_empty(), "Search should return results");
            }
            
            let search_time = search_start.elapsed();
            let search_ms = search_time.as_secs_f64() * 1000.0 / 10.0;
            
            tracing::info!("  ✅ Search: {:.4}ms per query ({:.0} QPS)", 
                     search_ms,
                     1000.0 / search_ms);
        } // collection dropped here, lock released
        
        // Cleanup
        store.delete_collection(&collection_name)?;
        tracing::info!("  ✅ Cleanup done\n");
    }
    
    tracing::info!("✅ Benchmark completed successfully!");
    
    Ok(())
}

