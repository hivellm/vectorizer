//! Test Master/Replica routing functionality

use vectorizer_sdk::{ClientConfig, HostConfig, ReadPreference, VectorizerClient};

const MASTER_URL: &str = "http://localhost:15002";
const REPLICA_URL: &str = "http://localhost:17780";
const API_KEY: &str = "eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9.eyJ1c2VyX2lkIjoiYWRtaW4iLCJ1c2VybmFtZSI6ImFkbWluIiwicm9sZXMiOlsiQWRtaW4iXSwiaWF0IjoxNzY0Mzk0MzI3LCJleHAiOjE3NjQzOTc5Mjd9.AnLPTgdHRfCFMdMp6VFemhcIZPUfpzjwB5r6xOkCxNQ";

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Rust SDK Master/Replica Test ===\n");

    // 1. Test with hosts configuration
    println!("1. Creating client with hosts configuration...");
    let config = ClientConfig {
        hosts: Some(HostConfig {
            master: MASTER_URL.to_string(),
            replicas: vec![REPLICA_URL.to_string()],
        }),
        read_preference: Some(ReadPreference::Replica),
        api_key: Some(API_KEY.to_string()),
        ..Default::default()
    };

    let client = VectorizerClient::new(config)?;
    println!("   Client created with master/replica topology");

    // 2. Test health check (read operation - should go to replica)
    println!("2. Testing health check (read - should go to replica)...");
    match client.health_check().await {
        Ok(health) => println!("   Health status: {}", health.status),
        Err(e) => println!("   Health failed: {}", e),
    }

    // 3. Test listing collections (read operation)
    println!("3. Listing collections (read)...");
    match client.list_collections().await {
        Ok(collections) => println!("   Found {} collections", collections.len()),
        Err(e) => println!("   List failed: {}", e),
    }

    // 4. Test with_master callback
    println!("4. Testing with_master() callback...");
    match client
        .with_master(|master_client| async move {
            println!("   Inside with_master callback");
            let health = master_client.health_check().await?;
            Ok(health.status)
        })
        .await
    {
        Ok(status) => println!("   Master health: {}", status),
        Err(e) => println!("   with_master failed: {}", e),
    }

    // 5. Test backward compatibility with single base_url
    println!("\n5. Testing backward compatibility (single base_url)...");
    let single_config = ClientConfig {
        base_url: Some(MASTER_URL.to_string()),
        api_key: Some(API_KEY.to_string()),
        ..Default::default()
    };

    let single_client = VectorizerClient::new(single_config)?;
    match single_client.health_check().await {
        Ok(health) => println!("   Single URL mode works: {}", health.status),
        Err(e) => println!("   Single URL failed: {}", e),
    }

    println!("\n=== Rust SDK Test Complete ===");
    Ok(())
}
