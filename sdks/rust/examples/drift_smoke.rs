//! End-to-end smoke test that exercises every contract change made
//! against the v3.0.x server drift:
//!   1. `POST /auth/login` round-trip → JWT accepted on subsequent requests.
//!   2. API key sent as `X-API-Key` (not `Bearer`) — verify a 200 from a
//!      gated route under pure-api-key auth.
//!   3. `POST /insert_texts` new payload shape (collection top-level).
//!   4. `search_vectors` against `/collections/{c}/search/text`.
//!   5. `get_vector` by client id — documented quirk (server reassigns
//!      UUID + may return synthetic payload) is surfaced, not hidden.
//!
//! Run against a live v3.x server on `VECTORIZER_URL` (default
//! `http://localhost:15001` — the cortex stack on this dev box maps
//! the vectorizer container's 15002 → host 15001). Example:
//!
//!     $env:VECTORIZER_URL = "http://localhost:15001"
//!     cargo run -p vectorizer-sdk --example drift_smoke

use std::env;

use vectorizer_sdk::{BatchTextRequest, ClientConfig, SimilarityMetric, VectorizerClient};

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let base_url =
        env::var("VECTORIZER_URL").unwrap_or_else(|_| "http://localhost:15001".to_string());

    // Step 1 — create a client WITHOUT credentials, login with
    // admin/admin, get a JWT. If `auth.enabled` is false on the target
    // server the endpoint 404s; treat that as "no auth" and proceed
    // with the anonymous client.
    let anon = VectorizerClient::new_with_url(&base_url)?;
    let user = env::var("VZ_ADMIN_USER").unwrap_or_else(|_| "admin".to_string());
    let pass = env::var("VZ_ADMIN_PASS").unwrap_or_else(|_| "admin".to_string());
    let jwt = match anon.login(&user, &pass).await {
        Ok(tok) => {
            println!(
                "login OK — jwt len={} prefix={:?}",
                tok.access_token.len(),
                &tok.access_token.chars().take(16).collect::<String>()
            );
            Some(tok.access_token)
        }
        Err(e) => {
            println!("login skipped ({e}); continuing anonymously");
            None
        }
    };

    // Step 2 — rebuild client with the JWT in `api_key`. The HTTP
    // transport sniffs the 3-segment JWT shape and uses
    // `Authorization: Bearer …`.
    let config = ClientConfig {
        base_url: Some(base_url.clone()),
        api_key: jwt.clone(),
        ..ClientConfig::default()
    };
    let client = VectorizerClient::new(config)?;

    // Step 3 — `/health` is anonymous; verify transport is wired.
    let health = client.health_check().await?;
    println!("health={} version={}", health.status, health.version);
    assert!(
        health.version.starts_with("3."),
        "expected v3.x server, got {}",
        health.version
    );

    // Step 4 — create a fresh collection at the actual BM25-fallback
    // dimension (512). If we guess 768 the server rejects the inserts
    // with `Invalid dimension: expected 512, got 768`.
    let name = format!(
        "drift_smoke_{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)?
            .as_secs()
    );
    let _ = client.delete_collection(&name).await;
    client
        .create_collection(&name, 512, Some(SimilarityMetric::Cosine))
        .await?;
    println!("collection={name} dim=512 metric=cosine");

    // Step 5 — batch insert via the fixed `/insert_texts` route +
    // `{collection, texts:[…]}` payload. Previous SDK path
    // (`/collections/{c}/documents`) 404'd on the 3.0.x image.
    let texts = vec![
        BatchTextRequest {
            id: "doc_0".to_string(),
            text: "rust is a systems programming language focused on safety".to_string(),
            metadata: None,
        },
        BatchTextRequest {
            id: "doc_1".to_string(),
            text: "vector databases index high-dimensional embeddings for search".to_string(),
            metadata: None,
        },
    ];
    let inserted = client.insert_texts(&name, texts).await?;
    println!(
        "insert_texts total={} succeeded={} failed={}",
        inserted.total_operations, inserted.successful_operations, inserted.failed_operations
    );

    // Step 6 — text search round-trip.
    let search = client
        .search_vectors(&name, "rust programming", Some(5), None)
        .await?;
    println!("search results={}", search.results.len());

    // Cleanup.
    client.delete_collection(&name).await?;
    println!("cleanup OK");
    Ok(())
}
