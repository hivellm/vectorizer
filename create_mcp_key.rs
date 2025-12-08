use vectorizer::auth::{AuthConfig, AuthManager};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize auth manager
    let config = AuthConfig {
        enabled: true,
        jwt_secret: "your-secret-key-change-in-production".to_string(),
        jwt_expiration: 3600,
        api_key_length: 32,
        rate_limit_per_minute: 100,
        rate_limit_per_hour: 1000,
    };

    let auth_manager = AuthManager::new(config)?;

    // Create MCP API key
    let api_key = auth_manager.create_api_key(
        "mcp-client",
        "MCP Server Key",
        vec!["Read".to_string(), "Write".to_string(), "Admin".to_string()],
        0, // never expires
    ).await?;

    println!("╔════════════════════════════════════════════════════════════╗");
    println!("║                    MCP API KEY GENERATED                   ║");
    println!("╠════════════════════════════════════════════════════════════╣");
    println!("║  Key ID: {:<50} ║", api_key.id);
    println!("║  API Key: {:<48} ║", api_key.key);
    println!("║  User ID: {:<49} ║", api_key.user_id);
    println!("║  Name: {:<52} ║", api_key.name);
    println!("╠════════════════════════════════════════════════════════════╣");
    println!("║  Use this key in your MCP client configuration:            ║");
    println!("║  Authorization: {}  ║", api_key.key);
    println!("╚════════════════════════════════════════════════════════════╝");

    Ok(())
}
