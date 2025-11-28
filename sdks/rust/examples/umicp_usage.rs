//! Example: Using the Vectorizer client with UMICP protocol
//!
//! UMICP (Universal Messaging and Inter-process Communication Protocol) provides:
//! - High-performance binary protocol
//! - Efficient transport layer
//! - Built on umicp-core crate

use tracing::{debug, error, info, warn};
use vectorizer_rust_sdk::{ClientConfig, VectorizerClient};

#[cfg(feature = "umicp")]
use vectorizer_rust_sdk::{Protocol, UmicpConfig};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing::info!("=== Vectorizer Client with UMICP ===\n");

    // Option 1: Using connection string
    #[cfg(feature = "umicp")]
    {
        tracing::info!("Option 1: Connection string");
        let client1 = VectorizerClient::from_connection_string(
            "umicp://localhost:15003",
            Some("your-api-key-here"),
        )?;

        tracing::info!("Protocol: {}", client1.protocol());
    }

    // Option 2: Using explicit configuration
    #[cfg(feature = "umicp")]
    {
        tracing::info!("\nOption 2: Explicit configuration");
        let client2 = VectorizerClient::new(ClientConfig {
            protocol: Some(Protocol::Umicp),
            api_key: Some("your-api-key-here".to_string()),
            umicp: Some(UmicpConfig {
                host: "localhost".to_string(),
                port: 15003,
            }),
            ..Default::default()
        })?;

        tracing::info!("Protocol: {}", client2.protocol());

        // Health check
        tracing::info!("\n1. Health Check");
        match client2.health_check().await {
            Ok(health) => tracing::info!("Server status: {:?}", health.status),
            Err(e) => tracing::info!("Health check failed: {}", e),
        }

        // List collections
        tracing::info!("\n2. List Collections");
        match client2.list_collections().await {
            Ok(collections) => tracing::info!("Found {} collection(s)", collections.len()),
            Err(e) => tracing::info!("Failed to list collections: {}", e),
        }

        // Search example
        if let Ok(collections) = client2.list_collections().await {
            if !collections.is_empty() {
                let collection_name = &collections[0].name;
                tracing::info!("\n3. Searching in collection: {}", collection_name);

                match client2
                    .search_vectors(collection_name, "example search", Some(5), None)
                    .await
                {
                    Ok(results) => {
                        tracing::info!("Found {} result(s)", results.results.len());
                        for (i, result) in results.results.iter().enumerate() {
                            tracing::info!("  {}. Score: {:.4}", i + 1, result.score);
                        }
                    }
                    Err(e) => tracing::info!("Search failed: {}", e),
                }
            }
        }
    }

    #[cfg(not(feature = "umicp"))]
    {
        tracing::info!("UMICP feature is not enabled.");
        tracing::info!("To use UMICP, rebuild with: cargo build --features umicp");
        tracing::info!("\nUsing HTTP transport instead:");

        let client = VectorizerClient::new_with_url("http://localhost:15002")?;
        tracing::info!("Protocol: {}", client.protocol());

        match client.health_check().await {
            Ok(health) => tracing::info!("Server status: {:?}", health.status),
            Err(e) => tracing::info!("Health check failed: {}", e),
        }
    }

    tracing::info!("\n=== UMICP Demo Complete ===");
    Ok(())
}
