//! Bend Integration Example with Real Vectorizer Data
//! 
//! This example demonstrates how Bend accelerates vector similarity search
//! in the Vectorizer project.

use std::sync::Arc;
use crate::bend::{BendConfig, BendCollection};
use crate::models::{CollectionConfig, HnswConfig, DistanceMetric, Vector, Payload};
use crate::error::Result;

/// Example of using Bend with real vectorizer data
pub async fn bend_vectorizer_example() -> Result<()> {
    println!("🚀 Bend Vectorizer Integration Example");
    println!("=====================================");

    // Create Bend configuration
    let bend_config = BendConfig {
        enabled: true,
        enable_cuda: false, // Set to true if CUDA is available
        max_parallel: 1000,
        fallback_enabled: true,
        bend_path: "bend".to_string(),
    };

    // Create collection configuration
    let collection_config = CollectionConfig {
        dimension: 384, // Standard embedding dimension
        metric: DistanceMetric::Cosine,
        hnsw_config: HnswConfig {
            m: 16,
            ef_construction: 200,
            seed: Some(42),
        },
    };

    // Create Bend-enhanced collection
    let collection = BendCollection::new(
        "test_collection".to_string(),
        collection_config,
        bend_config,
    );

    println!("✅ Created Bend-enhanced collection");

    // Create sample vectors (simulating real embeddings)
    let sample_vectors = vec![
        Vector {
            id: "doc1".to_string(),
            data: vec![0.1, 0.2, 0.3, 0.4, 0.5, 0.6, 0.7, 0.8, 0.9, 1.0],
            payload: Some(Payload {
                data: serde_json::json!({
                    "content": "This is a document about machine learning",
                    "file_path": "/docs/ml_guide.txt",
                    "metadata": {
                        "author": "AI Researcher",
                        "topic": "machine_learning"
                    }
                }),
            }),
        },
        Vector {
            id: "doc2".to_string(),
            data: vec![0.2, 0.3, 0.4, 0.5, 0.6, 0.7, 0.8, 0.9, 1.0, 0.1],
            payload: Some(Payload {
                data: serde_json::json!({
                    "content": "Deep learning algorithms and neural networks",
                    "file_path": "/docs/deep_learning.txt",
                    "metadata": {
                        "author": "ML Engineer",
                        "topic": "deep_learning"
                    }
                }),
            }),
        },
        Vector {
            id: "doc3".to_string(),
            data: vec![0.3, 0.4, 0.5, 0.6, 0.7, 0.8, 0.9, 1.0, 0.1, 0.2],
            payload: Some(Payload {
                data: serde_json::json!({
                    "content": "Natural language processing techniques",
                    "file_path": "/docs/nlp_guide.txt",
                    "metadata": {
                        "author": "NLP Specialist",
                        "topic": "natural_language_processing"
                    }
                }),
            }),
        },
    ];

    // Batch add vectors with Bend acceleration
    println!("📝 Adding {} vectors to collection...", sample_vectors.len());
    collection.batch_add_vectors_with_bend(sample_vectors).await?;
    println!("✅ Vectors added successfully");

    // Perform search with Bend acceleration
    let query_vector = vec![0.15, 0.25, 0.35, 0.45, 0.55, 0.65, 0.75, 0.85, 0.95, 0.05];
    
    println!("🔍 Performing similarity search with Bend acceleration...");
    let start_time = std::time::Instant::now();
    
    let results = collection.search_with_bend(&query_vector, 3).await?;
    
    let search_time = start_time.elapsed().as_secs_f64() * 1000.0;
    
    println!("✅ Search completed in {:.2}ms", search_time);
    println!("📊 Found {} results:", results.len());
    
    for (i, result) in results.iter().enumerate() {
        println!("  {}. ID: {}, Score: {:.4}", i + 1, result.id, result.score);
        
        if let Some(payload) = &result.payload {
            if let Some(content) = payload.data.get("content") {
                if let Some(content_str) = content.as_str() {
                    println!("     Content: {}", content_str);
                }
            }
        }
    }

    // Get Bend statistics
    let stats = collection.get_bend_stats();
    println!("\n📈 Bend Statistics:");
    println!("  Total searches: {}", stats.total_searches);
    println!("  Bend searches: {}", stats.bend_searches);
    println!("  Fallback searches: {}", stats.fallback_searches);
    println!("  Average speedup: {:.2}x", stats.average_speedup);

    println!("\n🎉 Bend Vectorizer integration example completed successfully!");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_bend_vectorizer_example() {
        // This test would run the example
        // Note: Requires Bend to be installed and available
        if std::process::Command::new("bend").arg("--version").output().is_ok() {
            bend_vectorizer_example().await.unwrap();
        } else {
            println!("Bend not available, skipping test");
        }
    }
}
