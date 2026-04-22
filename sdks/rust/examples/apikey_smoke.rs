use std::env;

use vectorizer_sdk::{ClientConfig, VectorizerClient};
#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let url = env::var("VECTORIZER_URL").unwrap_or_else(|_| "http://localhost:15001".to_string());
    let key = env::args().nth(1).ok_or("pass api_key as arg 1")?;
    let cfg = ClientConfig {
        base_url: Some(url),
        api_key: Some(key),
        ..ClientConfig::default()
    };
    let c = VectorizerClient::new(cfg)?;
    let cols = c.list_collections().await?;
    println!(
        "list_collections via X-API-Key OK: {} collections",
        cols.len()
    );
    Ok(())
}
