//! Basic usage example for the Hive Vectorizer Rust SDK.
//! This example demonstrates all core operations available in the SDK.

use vectorizer_sdk::*;
use std::collections::HashMap;

#[tokio::main]
async fn main() -> Result<()> {
    println!("🦀 Vectorizer Rust SDK Basic Example");
    println!("====================================");

    // Create client
    let client = VectorizerClient::new_default()?;
    println!("✅ Client created successfully");

    let collection_name = "example-documents";

    // Health check
    println!("\n🔍 Checking server health...");
    match client.health_check().await {
        Ok(health) => {
            println!("✅ Server status: {}", health.status);
            println!("   Version: {}", health.version);
            if let Some(collections) = health.collections {
                println!("   Collections: {}", collections);
            }
            if let Some(vectors) = health.total_vectors {
                println!("   Total Vectors: {}", vectors);
            }
        }
        Err(e) => {
            println!("⚠️ Health check failed: {}", e);
        }
    }

    // List existing collections
    println!("\n📋 Listing collections...");
    match client.list_collections().await {
        Ok(collections) => {
            println!("📁 Found {} collections:", collections.len());
            for collection in collections.iter().take(5) {
                println!("   - {} ({} vectors)", collection.name, collection.vector_count);
            }
        }
        Err(e) => {
            println!("⚠️ Error listing collections: {}", e);
        }
    }

    // Create a new collection
    println!("\n🆕 Creating collection...");
    match client.create_collection(collection_name, 384, Some(SimilarityMetric::Cosine)).await {
        Ok(collection) => {
            println!("✅ Collection created: {}", collection.name);
            println!("   Dimension: {}", collection.dimension);
            println!("   Metric: {}", collection.metric);
        }
        Err(e) => {
            println!("⚠️ Collection creation failed (may already exist): {}", e);
        }
    }

    // Insert texts
    println!("\n📥 Inserting texts...");
    let texts = vec![
        BatchTextRequest {
            id: "doc_1".to_string(),
            text: "Introduction to Machine Learning".to_string(),
            metadata: Some({
                let mut meta = HashMap::new();
                meta.insert("source".to_string(), serde_json::Value::String("document1.pdf".to_string()));
                meta.insert("title".to_string(), serde_json::Value::String("Introduction to Machine Learning".to_string()));
                meta.insert("category".to_string(), serde_json::Value::String("AI".to_string()));
                meta
            }),
        },
        BatchTextRequest {
            id: "doc_2".to_string(),
            text: "Deep Learning Fundamentals".to_string(),
            metadata: Some({
                let mut meta = HashMap::new();
                meta.insert("source".to_string(), serde_json::Value::String("document2.pdf".to_string()));
                meta.insert("title".to_string(), serde_json::Value::String("Deep Learning Fundamentals".to_string()));
                meta.insert("category".to_string(), serde_json::Value::String("AI".to_string()));
                meta
            }),
        },
        BatchTextRequest {
            id: "doc_3".to_string(),
            text: "Data Science Best Practices".to_string(),
            metadata: Some({
                let mut meta = HashMap::new();
                meta.insert("source".to_string(), serde_json::Value::String("document3.pdf".to_string()));
                meta.insert("title".to_string(), serde_json::Value::String("Data Science Best Practices".to_string()));
                meta.insert("category".to_string(), serde_json::Value::String("Data".to_string()));
                meta
            }),
        },
    ];

    match client.insert_texts(collection_name, texts).await {
        Ok(result) => {
            println!("✅ Texts inserted: {}", result.inserted);
        }
        Err(e) => {
            println!("⚠️ Insert texts failed: {}", e);
        }
    }

    // Search for similar vectors
    println!("\n🔍 Searching for similar vectors...");
    match client.search_vectors(collection_name, "machine learning algorithms", Some(3), None).await {
        Ok(results) => {
            println!("🎯 Search results:");
            for (index, result) in results.results.iter().enumerate() {
                println!("   {}. Score: {:.4}", index + 1, result.score);
                if let Some(metadata) = &result.metadata {
                    if let Some(title) = metadata.get("title") {
                        println!("      Title: {}", title);
                    }
                    if let Some(category) = metadata.get("category") {
                        println!("      Category: {}", category);
                    }
                }
            }
        }
        Err(e) => {
            println!("⚠️ Search failed: {}", e);
        }
    }

    // Generate embeddings
    println!("\n🧠 Generating embeddings...");
    match client.embed_text("artificial intelligence and machine learning", None).await {
        Ok(embedding) => {
            println!("✅ Embedding generated:");
            println!("   Text: {}", embedding.text);
            println!("   Model: {}", embedding.model);
            println!("   Dimension: {}", embedding.dimension);
            println!("   Provider: {}", embedding.provider);
        }
        Err(e) => {
            println!("⚠️ Embedding generation failed: {}", e);
        }
    }

    // Get collection info
    println!("\n📊 Getting collection information...");
    match client.get_collection_info(collection_name).await {
        Ok(info) => {
            println!("📈 Collection info:");
            println!("   Name: {}", info.name);
            println!("   Dimension: {}", info.dimension);
            println!("   Vector count: {}", info.vector_count);
            if let Some(size_bytes) = info.size_bytes {
                println!("   Size: {} KB", size_bytes / 1024);
            }
        }
        Err(e) => {
            println!("⚠️ Get collection info failed: {}", e);
        }
    }

    println!("\n🌐 All operations completed successfully!");

    // Clean up
    println!("\n🧹 Cleaning up...");
    match client.delete_collection(collection_name).await {
        Ok(_) => {
            println!("✅ Collection deleted");
        }
        Err(e) => {
            println!("⚠️ Delete collection failed: {}", e);
        }
    }

    println!("\n👋 Example completed!");
    Ok(())
}
