//! Basic usage example for the Hive Vectorizer Rust SDK.
//! This example demonstrates all core operations available in the SDK.

use std::collections::HashMap;

use tracing::{debug, error, info, warn};
use vectorizer_sdk::*;

#[tokio::main]
async fn main() -> Result<()> {
    tracing::info!("🦀 Vectorizer Rust SDK Basic Example");
    tracing::info!("====================================");

    // Create client
    let client = VectorizerClient::new_default()?;
    tracing::info!("✅ Client created successfully");

    let collection_name = "example-documents";

    // Health check
    tracing::info!("\n🔍 Checking server health...");
    match client.health_check().await {
        Ok(health) => {
            tracing::info!("✅ Server status: {}", health.status);
            tracing::info!("   Version: {}", health.version);
            if let Some(collections) = health.collections {
                tracing::info!("   Collections: {}", collections);
            }
            if let Some(vectors) = health.total_vectors {
                tracing::info!("   Total Vectors: {}", vectors);
            }
        }
        Err(e) => {
            tracing::info!("⚠️ Health check failed: {}", e);
        }
    }

    // List existing collections
    tracing::info!("\n📋 Listing collections...");
    match client.list_collections().await {
        Ok(collections) => {
            tracing::info!("📁 Found {} collections:", collections.len());
            for collection in collections.iter().take(5) {
                tracing::info!(
                    "   - {} ({} vectors)",
                    collection.name,
                    collection.vector_count
                );
            }
        }
        Err(e) => {
            tracing::info!("⚠️ Error listing collections: {}", e);
        }
    }

    // Create a new collection
    tracing::info!("\n🆕 Creating collection...");
    match client
        .create_collection(collection_name, 384, Some(SimilarityMetric::Cosine))
        .await
    {
        Ok(collection) => {
            tracing::info!("✅ Collection created: {}", collection.name);
            tracing::info!("   Dimension: {}", collection.dimension);
            tracing::info!("   Metric: {}", collection.metric);
        }
        Err(e) => {
            tracing::info!("⚠️ Collection creation failed (may already exist): {}", e);
        }
    }

    // Insert texts
    tracing::info!("\n📥 Inserting texts...");
    let texts = vec![
        BatchTextRequest {
            id: "doc_1".to_string(),
            text: "Introduction to Machine Learning".to_string(),
            metadata: Some({
                let mut meta = HashMap::new();
                meta.insert(
                    "source".to_string(),
                    serde_json::Value::String("document1.pdf".to_string()),
                );
                meta.insert(
                    "title".to_string(),
                    serde_json::Value::String("Introduction to Machine Learning".to_string()),
                );
                meta.insert(
                    "category".to_string(),
                    serde_json::Value::String("AI".to_string()),
                );
                meta
            }),
        },
        BatchTextRequest {
            id: "doc_2".to_string(),
            text: "Deep Learning Fundamentals".to_string(),
            metadata: Some({
                let mut meta = HashMap::new();
                meta.insert(
                    "source".to_string(),
                    serde_json::Value::String("document2.pdf".to_string()),
                );
                meta.insert(
                    "title".to_string(),
                    serde_json::Value::String("Deep Learning Fundamentals".to_string()),
                );
                meta.insert(
                    "category".to_string(),
                    serde_json::Value::String("AI".to_string()),
                );
                meta
            }),
        },
        BatchTextRequest {
            id: "doc_3".to_string(),
            text: "Data Science Best Practices".to_string(),
            metadata: Some({
                let mut meta = HashMap::new();
                meta.insert(
                    "source".to_string(),
                    serde_json::Value::String("document3.pdf".to_string()),
                );
                meta.insert(
                    "title".to_string(),
                    serde_json::Value::String("Data Science Best Practices".to_string()),
                );
                meta.insert(
                    "category".to_string(),
                    serde_json::Value::String("Data".to_string()),
                );
                meta
            }),
        },
    ];

    match client.insert_texts(collection_name, texts).await {
        Ok(result) => {
            tracing::info!("✅ Texts inserted: {}", result.inserted);
        }
        Err(e) => {
            tracing::info!("⚠️ Insert texts failed: {}", e);
        }
    }

    // Search for similar vectors
    tracing::info!("\n🔍 Searching for similar vectors...");
    match client
        .search_vectors(
            collection_name,
            "machine learning algorithms",
            Some(3),
            None,
        )
        .await
    {
        Ok(results) => {
            tracing::info!("🎯 Search results:");
            for (index, result) in results.results.iter().enumerate() {
                tracing::info!("   {}. Score: {:.4}", index + 1, result.score);
                if let Some(metadata) = &result.metadata {
                    if let Some(title) = metadata.get("title") {
                        tracing::info!("      Title: {}", title);
                    }
                    if let Some(category) = metadata.get("category") {
                        tracing::info!("      Category: {}", category);
                    }
                }
            }
        }
        Err(e) => {
            tracing::info!("⚠️ Search failed: {}", e);
        }
    }

    // Generate embeddings
    tracing::info!("\n🧠 Generating embeddings...");
    match client
        .embed_text("artificial intelligence and machine learning", None)
        .await
    {
        Ok(embedding) => {
            tracing::info!("✅ Embedding generated:");
            tracing::info!("   Text: {}", embedding.text);
            tracing::info!("   Model: {}", embedding.model);
            tracing::info!("   Dimension: {}", embedding.dimension);
            tracing::info!("   Provider: {}", embedding.provider);
        }
        Err(e) => {
            tracing::info!("⚠️ Embedding generation failed: {}", e);
        }
    }

    // Get collection info
    tracing::info!("\n📊 Getting collection information...");
    match client.get_collection_info(collection_name).await {
        Ok(info) => {
            tracing::info!("📈 Collection info:");
            tracing::info!("   Name: {}", info.name);
            tracing::info!("   Dimension: {}", info.dimension);
            tracing::info!("   Vector count: {}", info.vector_count);
            if let Some(size_bytes) = info.size_bytes {
                tracing::info!("   Size: {} KB", size_bytes / 1024);
            }
        }
        Err(e) => {
            tracing::info!("⚠️ Get collection info failed: {}", e);
        }
    }

    tracing::info!("\n🌐 All operations completed successfully!");

    // Clean up
    tracing::info!("\n🧹 Cleaning up...");
    match client.delete_collection(collection_name).await {
        Ok(_) => {
            tracing::info!("✅ Collection deleted");
        }
        Err(e) => {
            tracing::info!("⚠️ Delete collection failed: {}", e);
        }
    }

    tracing::info!("\n👋 Example completed!");
    Ok(())
}
