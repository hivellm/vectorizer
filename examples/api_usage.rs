//! Example demonstrating how to use the Vectorizer REST API

use serde_json::json;
use std::time::Duration;
use tokio::time::sleep;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 Vectorizer API Usage Example");

    // Wait a moment for the server to start (if running separately)
    sleep(Duration::from_secs(1)).await;

    let client = reqwest::Client::new();
    let base_url = "http://127.0.0.1:15001/api/v1";

    // 1. Health check
    println!("\n📊 Checking server health...");
    let health_response = client.get(&format!("{}/health", base_url)).send().await?;

    if health_response.status().is_success() {
        let health: serde_json::Value = health_response.json().await?;
        println!(
            "✅ Server is healthy: {}",
            serde_json::to_string_pretty(&health)?
        );
    } else {
        println!("❌ Server health check failed");
        return Ok(());
    }

    // 2. Create a collection
    println!("\n📁 Creating a collection...");
    let create_collection_request = json!({
        "name": "documents",
        "dimension": 384,
        "metric": "cosine",
        "hnsw_config": {
            "m": 16,
            "ef_construction": 200,
            "ef_search": 64
        }
    });

    let create_response = client
        .post(&format!("{}/collections", base_url))
        .json(&create_collection_request)
        .send()
        .await?;

    if create_response.status().is_success() {
        let result: serde_json::Value = create_response.json().await?;
        println!(
            "✅ Collection created: {}",
            serde_json::to_string_pretty(&result)?
        );
    } else {
        println!(
            "❌ Failed to create collection: {}",
            create_response.status()
        );
        let error: serde_json::Value = create_response.json().await?;
        println!("Error: {}", serde_json::to_string_pretty(&error)?);
    }

    // 3. List collections
    println!("\n📋 Listing collections...");
    let list_response = client
        .get(&format!("{}/collections", base_url))
        .send()
        .await?;

    if list_response.status().is_success() {
        let collections: serde_json::Value = list_response.json().await?;
        println!(
            "✅ Collections: {}",
            serde_json::to_string_pretty(&collections)?
        );
    }

    // 4. Insert vectors
    println!("\n📝 Inserting vectors...");
    let insert_request = json!({
        "vectors": [
            {
                "id": "doc1",
                "vector": vec![0.1; 384],
                "payload": {
                    "title": "Machine Learning Basics",
                    "content": "Introduction to ML concepts",
                    "category": "tutorial"
                }
            },
            {
                "id": "doc2",
                "vector": vec![0.2; 384],
                "payload": {
                    "title": "Deep Learning Advanced",
                    "content": "Advanced neural network techniques",
                    "category": "advanced"
                }
            },
            {
                "id": "doc3",
                "vector": vec![0.15; 384],
                "payload": {
                    "title": "Vector Databases",
                    "content": "Understanding vector storage and retrieval",
                    "category": "infrastructure"
                }
            }
        ]
    });

    let insert_response = client
        .post(&format!("{}/collections/documents/vectors", base_url))
        .json(&insert_request)
        .send()
        .await?;

    if insert_response.status().is_success() {
        let result: serde_json::Value = insert_response.json().await?;
        println!(
            "✅ Vectors inserted: {}",
            serde_json::to_string_pretty(&result)?
        );
    } else {
        println!("❌ Failed to insert vectors: {}", insert_response.status());
        let error: serde_json::Value = insert_response.json().await?;
        println!("Error: {}", serde_json::to_string_pretty(&error)?);
    }

    // 5. Search for similar vectors
    println!("\n🔍 Searching for similar vectors...");
    let search_request = json!({
        "vector": vec![0.12; 384],
        "limit": 2,
        "score_threshold": 0.0
    });

    let search_response = client
        .post(&format!("{}/collections/documents/search", base_url))
        .json(&search_request)
        .send()
        .await?;

    if search_response.status().is_success() {
        let results: serde_json::Value = search_response.json().await?;
        println!(
            "✅ Search results: {}",
            serde_json::to_string_pretty(&results)?
        );
    } else {
        println!("❌ Search failed: {}", search_response.status());
        let error: serde_json::Value = search_response.json().await?;
        println!("Error: {}", serde_json::to_string_pretty(&error)?);
    }

    // 6. Get a specific vector
    println!("\n📄 Getting specific vector...");
    let get_response = client
        .get(&format!("{}/collections/documents/vectors/doc1", base_url))
        .send()
        .await?;

    if get_response.status().is_success() {
        let vector: serde_json::Value = get_response.json().await?;
        println!(
            "✅ Vector retrieved: {}",
            serde_json::to_string_pretty(&vector)?
        );
    } else {
        println!("❌ Failed to get vector: {}", get_response.status());
    }

    // 7. Get collection info
    println!("\n📊 Getting collection information...");
    let info_response = client
        .get(&format!("{}/collections/documents", base_url))
        .send()
        .await?;

    if info_response.status().is_success() {
        let info: serde_json::Value = info_response.json().await?;
        println!(
            "✅ Collection info: {}",
            serde_json::to_string_pretty(&info)?
        );
    }

    println!("\n🎉 API example completed successfully!");

    Ok(())
}
