// Quick test script to verify all routes work before running full benchmark
use reqwest::Client;
use serde_json::json;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::new();

    println!("=== Testing Vectorizer Routes ===\n");

    // 1. Health check
    println!("1. Health check:");
    let resp = client.get("http://localhost:15002/health").send().await?;
    println!("   Status: {}", resp.status());
    println!("   Response: {}", resp.text().await?);

    // 2. Create collection
    println!("\n2. Create collection:");
    let resp = client
        .post("http://localhost:15002/collections")
        .json(&json!({"name": "test_route", "dimension": 384}))
        .send()
        .await?;
    println!("   Status: {}", resp.status());
    println!("   Response: {}", resp.text().await?);

    // 3. Insert vector
    println!("\n3. Insert vector:");
    let vector: Vec<f32> = vec![0.1; 384];
    let payload = json!({
        "points": [{
            "id": "test1",
            "vector": vector
        }]
    });
    let resp = client
        .put("http://localhost:15002/qdrant/collections/test_route/points")
        .json(&payload)
        .send()
        .await?;
    println!("   Status: {}", resp.status());
    let text = resp.text().await?;
    println!(
        "   Response: {}",
        if text.len() > 200 {
            &text[..200]
        } else {
            &text
        }
    );

    // 4. Search
    println!("\n4. Search:");
    let query: Vec<f32> = vec![0.1; 384];
    let search_payload = json!({
        "vector": query,
        "limit": 5
    });
    let resp = client
        .post("http://localhost:15002/collections/test_route/search")
        .json(&search_payload)
        .send()
        .await?;
    println!("   Status: {}", resp.status());
    let text = resp.text().await?;
    println!(
        "   Response: {}",
        if text.len() > 200 {
            &text[..200]
        } else {
            &text
        }
    );

    println!("\n=== Testing Qdrant Routes ===\n");

    // 1. Health check
    println!("1. Health check:");
    let resp = client.get("http://localhost:6333/").send().await?;
    println!("   Status: {}", resp.status());
    println!("   Response: {}", resp.text().await?);

    // 2. Create collection
    println!("\n2. Create collection:");
    let resp = client
        .put("http://localhost:6333/collections/test_qdrant_route")
        .json(&json!({
            "vectors": {
                "size": 384,
                "distance": "Cosine"
            }
        }))
        .send()
        .await?;
    println!("   Status: {}", resp.status());
    println!("   Response: {}", resp.text().await?);

    // 3. Insert vector
    println!("\n3. Insert vector:");
    let vector: Vec<f32> = vec![0.1; 384];
    let payload = json!({
        "points": [{
            "id": 1,
            "vector": vector
        }]
    });
    let resp = client
        .put("http://localhost:6333/collections/test_qdrant_route/points")
        .json(&payload)
        .send()
        .await?;
    println!("   Status: {}", resp.status());
    let text = resp.text().await?;
    println!(
        "   Response: {}",
        if text.len() > 200 {
            &text[..200]
        } else {
            &text
        }
    );

    // 4. Search
    println!("\n4. Search:");
    let query: Vec<f32> = vec![0.1; 384];
    let search_payload = json!({
        "vector": query,
        "limit": 5
    });
    let resp = client
        .post("http://localhost:6333/collections/test_qdrant_route/points/search")
        .json(&search_payload)
        .send()
        .await?;
    println!("   Status: {}", resp.status());
    let text = resp.text().await?;
    println!(
        "   Response: {}",
        if text.len() > 200 {
            &text[..200]
        } else {
            &text
        }
    );

    println!("\n✅ All routes tested!");
    Ok(())
}
