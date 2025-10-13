//! Example: Using the Vectorizer client with UMICP protocol
//!
//! UMICP (Universal Messaging and Inter-process Communication Protocol) provides:
//! - High-performance binary protocol
//! - Efficient transport layer
//! - Built on umicp-core crate

use vectorizer_rust_sdk::{VectorizerClient, ClientConfig};

#[cfg(feature = "umicp")]
use vectorizer_rust_sdk::{Protocol, UmicpConfig};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Vectorizer Client with UMICP ===\n");

    // Option 1: Using connection string
    #[cfg(feature = "umicp")]
    {
        println!("Option 1: Connection string");
        let client1 = VectorizerClient::from_connection_string(
            "umicp://localhost:15003",
            Some("your-api-key-here"),
        )?;
        
        println!("Protocol: {}", client1.protocol());
    }

    // Option 2: Using explicit configuration
    #[cfg(feature = "umicp")]
    {
        println!("\nOption 2: Explicit configuration");
        let client2 = VectorizerClient::new(ClientConfig {
            protocol: Some(Protocol::Umicp),
            api_key: Some("your-api-key-here".to_string()),
            umicp: Some(UmicpConfig {
                host: "localhost".to_string(),
                port: 15003,
            }),
            ..Default::default()
        })?;

        println!("Protocol: {}", client2.protocol());

        // Health check
        println!("\n1. Health Check");
        match client2.health_check().await {
            Ok(health) => println!("Server status: {:?}", health.status),
            Err(e) => eprintln!("Health check failed: {}", e),
        }

        // List collections
        println!("\n2. List Collections");
        match client2.list_collections().await {
            Ok(collections) => println!("Found {} collection(s)", collections.len()),
            Err(e) => eprintln!("Failed to list collections: {}", e),
        }

        // Search example
        if let Ok(collections) = client2.list_collections().await {
            if !collections.is_empty() {
                let collection_name = &collections[0].name;
                println!("\n3. Searching in collection: {}", collection_name);
                
                match client2.search_vectors(collection_name, "example search", Some(5), None).await {
                    Ok(results) => {
                        println!("Found {} result(s)", results.results.len());
                        for (i, result) in results.results.iter().enumerate() {
                            println!("  {}. Score: {:.4}", i + 1, result.score);
                        }
                    },
                    Err(e) => eprintln!("Search failed: {}", e),
                }
            }
        }
    }

    #[cfg(not(feature = "umicp"))]
    {
        println!("UMICP feature is not enabled.");
        println!("To use UMICP, rebuild with: cargo build --features umicp");
        println!("\nUsing HTTP transport instead:");
        
        let client = VectorizerClient::new_with_url("http://localhost:15002")?;
        println!("Protocol: {}", client.protocol());
        
        match client.health_check().await {
            Ok(health) => println!("Server status: {:?}", health.status),
            Err(e) => eprintln!("Health check failed: {}", e),
        }
    }

    println!("\n=== UMICP Demo Complete ===");
    Ok(())
}

