//! Test script to create collection with graph enabled and test MCP graph functions
//! 
//! Run with: cargo run --bin vectorizer -- test-graph

use std::sync::Arc;
use vectorizer::models::{
    CollectionConfig, DistanceMetric, GraphConfig, AutoRelationshipConfig,
    QuantizationConfig, HnswConfig, CompressionConfig, CompressionAlgorithm, StorageType
};
use vectorizer::VectorStore;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Creating collection with graph enabled...");
    
    let store = Arc::new(VectorStore::new());
    
    // Create collection with graph enabled
    let config = CollectionConfig {
        dimension: 512,
        metric: DistanceMetric::Cosine,
        hnsw_config: HnswConfig::default(),
        quantization: QuantizationConfig::SQ { bits: 8 },
        compression: CompressionConfig {
            enabled: false,
            threshold_bytes: 1024,
            algorithm: CompressionAlgorithm::Lz4,
        },
        normalization: None,
        storage_type: Some(StorageType::Memory),
        sharding: None,
        graph: Some(GraphConfig {
            enabled: true,
            auto_relationship: AutoRelationshipConfig::default(),
        }),
    };
    
    let collection_name = "graph-enabled-collection";
    
    // Delete if exists
    let _ = store.delete_collection(collection_name);
    
    match store.create_collection(collection_name, config) {
        Ok(_) => {
            println!("✅ Collection '{}' created successfully with graph enabled!", collection_name);
        }
        Err(e) => {
            eprintln!("❌ Failed to create collection: {}", e);
            return Err(e.into());
        }
    }
    
    // Verify graph is enabled
    let collection = store.get_collection(collection_name)?;
    
    match collection {
        vectorizer::db::CollectionType::Cpu(coll) => {
            if let Some(graph) = coll.get_graph() {
                println!("✅ Graph is enabled in collection!");
                
                let nodes = graph.get_all_nodes();
                println!("📊 Graph has {} nodes (initial)", nodes.len());
                
                println!("\n✅ Collection with graph created successfully!");
                println!("You can now test MCP graph functions on this collection:");
                println!("  - graph_list_nodes");
                println!("  - graph_get_neighbors");
                println!("  - graph_find_related");
                println!("  - graph_find_path");
                println!("  - graph_create_edge");
                println!("  - graph_delete_edge");
            } else {
                println!("❌ Graph is NOT enabled in collection!");
                return Err("Graph not enabled".into());
            }
        }
        _ => {
            println!("❌ Unexpected collection type!");
            return Err("Unexpected collection type".into());
        }
    }
    
    Ok(())
}

