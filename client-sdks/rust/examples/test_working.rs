//! Simple and functional test example

use vectorizer_rust_sdk::*;
use std::collections::HashMap;

#[tokio::main]
async fn main() -> vectorizer_rust_sdk::Result<()> {
    println!("🦀 Testing Rust SDK for Vectorizer");
    println!("===================================");

    // Create client
    let client = VectorizerClient::new_default()?;
    println!("✅ Client created successfully");

    // Health check
    println!("\n📊 Health Check:");
    match client.health_check().await {
        Ok(health) => {
            println!("✅ Service: {}", health.status);
            println!("   Status: {}", health.status);
            println!("   Version: {}", health.version);
        }
        Err(e) => {
            println!("❌ Health check failed: {}", e);
            return Ok(());
        }
    }

    // List collections
    println!("\n📚 Available Collections:");
    match client.list_collections().await {
        Ok(collections) => {
            println!("✅ Found {} collections:", collections.len());
            for collection in collections.iter().take(3) {
                println!("   - {} ({} vectors)", collection.name, collection.vector_count);
            }
        }
        Err(e) => {
            println!("❌ Error listing collections: {}", e);
        }
    }

    // Test search
    println!("\n🔍 Testing Search:");
    match client.search_vectors("gov-bips", "bitcoin", Some(2), None).await {
        Ok(results) => {
            println!("✅ Search successful: {} results", results.results.len());
            for result in results.results {
                println!("   - {} (score: {:.3})", result.id, result.score);
            }
        }
        Err(e) => {
            println!("❌ Search error: {}", e);
        }
    }

    // Test collection creation
    println!("\n🆕 Testing Collection Creation:");
    let test_collection = format!("test_rust_{}", uuid::Uuid::new_v4());
    
    match client.create_collection(&test_collection, 384, Some(SimilarityMetric::Cosine)).await {
        Ok(info) => {
            println!("✅ Collection '{}' created:", info.name);
            println!("   Dimension: {}", info.dimension);
            println!("   Status: {}", info.indexing_status.status);
        }
        Err(e) => {
            println!("❌ Error creating collection: {}", e);
            return Ok(());
        }
    }

    // Test text insertion
    println!("\n📝 Testing Text Insertion:");
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
            println!("✅ Insertion successful:");
            println!("   Total operations: {}", response.total_operations);
            println!("   Successful: {}", response.successful_operations);
            println!("   Failed: {}", response.failed_operations);
        }
        Err(e) => {
            println!("❌ Insertion error: {}", e);
        }
    }

    // Wait for indexing
    println!("\n⏳ Waiting for indexing...");
    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

    // Test search in test collection
    println!("\n🔍 Testing Search in Test Collection:");
    match client.search_vectors(&test_collection, "vectorizer performance", Some(5), None).await {
        Ok(results) => {
            println!("✅ Search successful:");
            println!("   Found {} results", results.results.len());
            for (i, result) in results.results.iter().enumerate() {
                println!("   {}. {} (score: {:.3})", i + 1, result.id, result.score);
            }
        }
        Err(e) => {
            println!("❌ Search error: {}", e);
        }
    }

    // Test embedding generation
    println!("\n🧠 Testing Embedding Generation:");
    match client.embed_text("This is a test text for embedding.", None).await {
        Ok(response) => {
            println!("✅ Embedding generated:");
            println!("   Text: {}", response.text);
            println!("   Model: {}", response.model);
            println!("   Dimension: {}", response.dimension);
            println!("   Provider: {}", response.provider);
        }
        Err(e) => {
            println!("❌ Embedding generation error: {}", e);
        }
    }

    // Cleanup
    println!("\n🧹 Cleaning up test collection...");
    match client.delete_collection(&test_collection).await {
        Ok(_) => {
            println!("✅ Collection '{}' deleted successfully", test_collection);
        }
        Err(e) => {
            println!("❌ Error deleting collection: {}", e);
        }
    }

    println!("\n🎉 Test completed successfully!");
    Ok(())
}