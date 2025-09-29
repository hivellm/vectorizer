//! Basic example of using the Vectorizer Rust SDK

use vectorizer_rust_sdk::*;

#[tokio::main]
async fn main() -> Result<()> {
    println!("🦀 Vectorizer Rust SDK Basic Example");
    println!("====================================");

    // Create a client
    let client = VectorizerClient::new_default()?;
    println!("✅ Client created successfully");

    // Health check
    println!("\n📊 Health Check:");
    match client.health_check().await {
        Ok(health) => {
            println!("✅ Status: {}", health.status);
            println!("   Version: {}", health.version);
            println!("   Collections: {}", health.collections.unwrap_or(0));
            println!("   Total Vectors: {}", health.total_vectors.unwrap_or(0));
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

    println!("\n🎉 Basic example completed!");
    Ok(())
}
