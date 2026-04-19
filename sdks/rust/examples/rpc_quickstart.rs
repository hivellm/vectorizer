//! RPC quickstart example for the Vectorizer Rust SDK (v3.x default).
//!
//! Connects to a server on `127.0.0.1:15503` (the default RPC port),
//! does the HELLO handshake, lists collections, and runs a search
//! against the first one.
//!
//! Run a Vectorizer server with RPC enabled (the v3.x default config
//! does this automatically), then:
//!
//! ```bash
//! cargo run --example rpc_quickstart
//! ```

use vectorizer_sdk::rpc::{HelloPayload, RpcClient};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Two equivalent ways to dial:
    //   1. By host:port (no scheme).
    //   2. By the canonical vectorizer:// URL.
    // The canonical URL is the recommended form because it makes the
    // transport explicit and round-trips through configuration files
    // unchanged.
    let url = std::env::var("VECTORIZER_URL")
        .unwrap_or_else(|_| "vectorizer://127.0.0.1:15503".to_string());
    println!("→ Dialing {url}");
    let client = RpcClient::connect_url(&url).await?;

    // HELLO handshake. In single-user mode (auth.enabled: false on
    // the server side), credentials are accepted-but-ignored. When
    // auth is enabled, attach a JWT or API key:
    //   HelloPayload::new("rpc-quickstart").with_token("<jwt>")
    let hello = client.hello(HelloPayload::new("rpc-quickstart")).await?;
    println!(
        "✓ HELLO ok — server={}  protocol_version={}  authenticated={}  admin={}",
        hello.server_version, hello.protocol_version, hello.authenticated, hello.admin
    );
    println!("  capabilities: {:?}", hello.capabilities);

    // PING (auth-exempt — works pre-HELLO too).
    let pong = client.ping().await?;
    println!("✓ PING → {pong}");

    // List collections.
    let collections = client.list_collections().await?;
    println!("✓ {} collection(s): {:?}", collections.len(), collections);

    // If we have at least one collection, run a search against it.
    if let Some(first) = collections.first() {
        println!("→ Searching '{first}' for 'vector database'");
        let info = client.get_collection_info(first).await?;
        println!(
            "  collection has {} vectors across {} documents (dim={})",
            info.vector_count, info.document_count, info.dimension
        );

        let hits = client.search_basic(first, "vector database", 5).await?;
        println!("  top {} hit(s):", hits.len());
        for hit in &hits {
            println!("    {} (score={:.4})", hit.id, hit.score);
        }
    } else {
        println!("  (no collections to search — create one via the REST/MCP API or the dashboard)");
    }

    Ok(())
}
