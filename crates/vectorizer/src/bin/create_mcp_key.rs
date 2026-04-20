//! One-shot helper that mints a fresh API key for an MCP client.
//!
//! Reads `VECTORIZER_JWT_SECRET` from the environment (must be at
//! least 32 chars; generate with `openssl rand -hex 64`), instantiates
//! an in-process [`AuthManager`], creates a never-expiring API key
//! tagged for the `mcp-client` user with Read+Write+ManageApiKeys
//! permissions, and prints the raw key string to stdout. The raw key
//! is shown once and never persisted in plain text — capture it from
//! this output and drop it into your MCP client config.
//!
//! Run with `cargo run --bin create_mcp_key`.

use std::env;

use vectorizer::auth::{AuthConfig, AuthManager, Permission, Secret};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let jwt_secret = env::var("VECTORIZER_JWT_SECRET").map_err(|_| {
        "VECTORIZER_JWT_SECRET env var required (min 32 chars). \
         Generate one with: openssl rand -hex 64"
    })?;

    let config = AuthConfig {
        jwt_secret: Secret::new(jwt_secret),
        enabled: true,
        ..AuthConfig::default()
    };

    let auth_manager = AuthManager::new(config)?;

    let (raw_key, api_key) = auth_manager
        .create_api_key(
            "mcp-client",
            "MCP Server Key",
            vec![
                Permission::Read,
                Permission::Write,
                Permission::ManageApiKeys,
            ],
            None,
        )
        .await?;

    println!("╔════════════════════════════════════════════════════════════╗");
    println!("║                    MCP API KEY GENERATED                   ║");
    println!("╠════════════════════════════════════════════════════════════╣");
    let id = &api_key.id;
    let user_id = &api_key.user_id;
    let name = &api_key.name;
    println!("║  Key ID:  {id:<48} ║");
    println!("║  User ID: {user_id:<48} ║");
    println!("║  Name:    {name:<48} ║");
    println!("╠════════════════════════════════════════════════════════════╣");
    println!("║  Raw key (copy this — shown once, never stored):           ║");
    println!("║  {raw_key:<58} ║");
    println!("╠════════════════════════════════════════════════════════════╣");
    println!("║  Use as Authorization header in your MCP client:           ║");
    println!("║    Authorization: Bearer <raw key>                         ║");
    println!("╚════════════════════════════════════════════════════════════╝");

    Ok(())
}
