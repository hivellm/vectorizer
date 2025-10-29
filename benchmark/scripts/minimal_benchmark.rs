//! Minimal Benchmark - Just insertion, no search

use anyhow::Result;
use std::time::Instant;
use vectorizer::VectorStore;
use vectorizer::models::{CollectionConfig, DistanceMetric, HnswConfig, Vector};

fn generate_vectors(count: usize, dimension: usize) -> Vec<Vector> {
    (0..count)
        .map(|i| {
            let data: Vec<f32> = (0..dimension).map(|j| (i + j) as f32).collect();
            Vector {
                id: format!("vec_{}", i),
                data,
                payload: None,
            }
        })
        .collect()
}

fn main() -> Result<()> {
    println!("🔬 Minimal Benchmark");
    println!("===================\n");
    
    let counts = vec![10, 50, 100];
    
    for count in counts {
        println!("📊 Testing {} vectors (128D)", count);
        
        let config = CollectionConfig {
            dimension: 128,
            metric: DistanceMetric::Cosine,
            hnsw_config: HnswConfig {
                m: 8,
                ef_construction: 100,
                ef_search: 50,
                seed: Some(42),
            },
            ..Default::default()
        };
        
        let vectors = generate_vectors(count, 128);
        
        let store = VectorStore::new();
        let name = format!("test_{}", count);
        
        let start = Instant::now();
        store.create_collection(&name, config)?;
        
        // IMPORTANT: Use collection in a scope to release the lock before delete
        {
            let mut collection = store.get_collection_mut(&name)?;
            
            for (i, vector) in vectors.iter().enumerate() {
                collection.add_vector(vector.id.clone(), vector.clone())?;
                if (i + 1) % 10 == 0 {
                    println!("  • Inserted {} vectors", i + 1);
                }
            }
        } // collection dropped here, lock released
        
        let elapsed = start.elapsed().as_secs_f64() * 1000.0;
        println!("  ✅ Total: {:.2}ms ({:.0} vectors/sec)\n", 
                 elapsed, 
                 count as f64 / (elapsed / 1000.0));
        
        store.delete_collection(&name)?;
    }
    
    println!("✅ Done!");
    Ok(())
}

