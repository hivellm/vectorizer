//! Simple and functional test example

use std::collections::HashMap;
use tracing::{info, error, warn, debug};
use vectorizer_rust_sdk::*;

#[tokio::main]
async fn main() -> vectorizer_rust_sdk::Result<()> {
    tracing::info!("🦀 Testing Rust SDK for Vectorizer");
    tracing::info!("===================================");

    // Create client
    let client = VectorizerClient::new_default()?;
    tracing::info!("✅ Client created successfully");

    // Health check
    tracing::info!("\n📊 Health Check:");
    match client.health_check().await {
        Ok(health) => {
            tracing::info!("✅ Service: {}", health.status);
            tracing::info!("   Status: {}", health.status);
            tracing::info!("   Version: {}", health.version);
        }
        Err(e) => {
            tracing::info!("❌ Health check failed: {}", e);
            return Ok(());
        }
    }

    // List collections
    tracing::info!("\n📚 Available Collections:");
    match client.list_collections().await {
        Ok(collections) => {
            tracing::info!("✅ Found {} collections:", collections.len());
            for collection in collections.iter().take(3) {
                tracing::info!(
                    "   - {} ({} vectors)",
                    collection.name, collection.vector_count
                );
            }
        }
        Err(e) => {
            tracing::info!("❌ Error listing collections: {}", e);
        }
    }

    // Test search
    tracing::info!("\n🔍 Testing Search:");
    match client
        .search_vectors("gov-bips", "bitcoin", Some(2), None)
        .await
    {
        Ok(results) => {
            tracing::info!("✅ Search successful: {} results", results.results.len());
            for result in results.results {
                tracing::info!("   - {} (score: {:.3})", result.id, result.score);
            }
        }
        Err(e) => {
            tracing::info!("❌ Search error: {}", e);
        }
    }

    // Test collection creation
    tracing::info!("\n🆕 Testing Collection Creation:");
    let test_collection = format!("test_rust_{}", uuid::Uuid::new_v4());

    match client
        .create_collection(&test_collection, 384, Some(SimilarityMetric::Cosine))
        .await
    {
        Ok(info) => {
            tracing::info!("✅ Collection '{}' created:", info.name);
            tracing::info!("   Dimension: {}", info.dimension);
            tracing::info!("   Status: {}", info.indexing_status.status);
        }
        Err(e) => {
            tracing::info!("❌ Error creating collection: {}", e);
            return Ok(());
        }
    }

    // Test text insertion
    tracing::info!("\n📝 Testing Text Insertion:");
    let test_texts = vec![
        BatchTextRequest {
            id: "rust_test_1".to_string(),
            text: "This is a test of the Rust SDK for Vectorizer.".to_string(),
            metadata: Some({
                let mut meta = HashMap::new();
                meta.insert("language".to_string(), "english".to_string());
                meta.insert("test".to_string(), "rust_sdk".to_string());
                meta
            }),
        },
        BatchTextRequest {
            id: "rust_test_2".to_string(),
            text: "Vectorizer is a high-performance vector database.".to_string(),
            metadata: Some({
                let mut meta = HashMap::new();
                meta.insert("language".to_string(), "english".to_string());
                meta.insert("test".to_string(), "rust_sdk".to_string());
                meta
            }),
        },
    ];

    match client.insert_texts(&test_collection, test_texts).await {
        Ok(response) => {
            tracing::info!("✅ Insertion successful:");
            tracing::info!("   Total operations: {}", response.total_operations);
            tracing::info!("   Successful: {}", response.successful_operations);
            tracing::info!("   Failed: {}", response.failed_operations);
        }
        Err(e) => {
            tracing::info!("❌ Insertion error: {}", e);
        }
    }

    // Wait for indexing
    tracing::info!("\n⏳ Waiting for indexing...");
    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

    // Test search in test collection
    tracing::info!("\n🔍 Testing Search in Test Collection:");
    match client
        .search_vectors(&test_collection, "vectorizer performance", Some(5), None)
        .await
    {
        Ok(results) => {
            tracing::info!("✅ Search successful:");
            tracing::info!("   Found {} results", results.results.len());
            for (i, result) in results.results.iter().enumerate() {
                tracing::info!("   {}. {} (score: {:.3})", i + 1, result.id, result.score);
            }
        }
        Err(e) => {
            tracing::info!("❌ Search error: {}", e);
        }
    }

    // Test embedding generation
    tracing::info!("\n🧠 Testing Embedding Generation:");
    match client
        .embed_text("This is a test text for embedding.", None)
        .await
    {
        Ok(response) => {
            tracing::info!("✅ Embedding generated:");
            tracing::info!("   Text: {}", response.text);
            tracing::info!("   Model: {}", response.model);
            tracing::info!("   Dimension: {}", response.dimension);
            tracing::info!("   Provider: {}", response.provider);
        }
        Err(e) => {
            tracing::info!("❌ Embedding generation error: {}", e);
        }
    }

    // Cleanup
    tracing::info!("\n🧹 Cleaning up test collection...");
    match client.delete_collection(&test_collection).await {
        Ok(_) => {
            tracing::info!("✅ Collection '{}' deleted successfully", test_collection);
        }
        Err(e) => {
            tracing::info!("❌ Error deleting collection: {}", e);
        }
    }

    tracing::info!("\n🎉 Test completed successfully!");
    Ok(())
}
