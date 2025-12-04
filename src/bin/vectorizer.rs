//! Vectorizer Server - Unified MCP + REST API
//!
//! This is the unified server that provides MCP + REST API access
//! for all vector operations.

#![allow(clippy::uninlined_format_args)]

use clap::Parser;
use tracing::{error, info, warn};
use vectorizer::config::VectorizerConfig;
use vectorizer::server::VectorizerServer;

#[derive(Parser)]
#[command(name = "vectorizer")]
#[command(about = "Vectorizer Server - MCP + REST API")]
struct Cli {
    /// Server host (overrides config.yml)
    #[arg(long)]
    host: Option<String>,

    /// Server port (overrides config.yml)
    #[arg(long)]
    port: Option<u16>,

    /// Enable verbose logging (default: only warnings and errors)
    #[arg(long)]
    verbose: bool,

    /// Path to config file
    #[arg(long, default_value = "config.yml")]
    config: String,
}

/// Load configuration from config.yml, falling back to defaults
fn load_config(config_path: &str) -> VectorizerConfig {
    match std::fs::read_to_string(config_path) {
        Ok(content) => match serde_yaml::from_str::<VectorizerConfig>(&content) {
            Ok(config) => {
                info!("✅ Loaded configuration from {}", config_path);
                config
            }
            Err(e) => {
                warn!("⚠️  Failed to parse {}: {}, using defaults", config_path, e);
                VectorizerConfig::default()
            }
        },
        Err(_) => {
            warn!("⚠️  Config file {} not found, using defaults", config_path);
            VectorizerConfig::default()
        }
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Install panic handler to log panics before aborting
    std::panic::set_hook(Box::new(|panic_info| {
        let location = panic_info
            .location()
            .map(|loc| format!("{}:{}:{}", loc.file(), loc.line(), loc.column()))
            .unwrap_or_else(|| "unknown".to_string());

        let message = if let Some(s) = panic_info.payload().downcast_ref::<&str>() {
            (*s).to_string()
        } else if let Some(s) = panic_info.payload().downcast_ref::<String>() {
            s.clone()
        } else {
            "unknown panic".to_string()
        };

        error!("❌ PANIC: {} at {}", message, location);

        // Log to file if possible
        if let Ok(mut file) = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(".logs/panic.log")
        {
            use std::io::Write;
            let _ = writeln!(
                file,
                "[{}] PANIC: {} at {}",
                chrono::Utc::now().to_rfc3339(),
                message,
                location
            );
        }
    }));

    let cli = Cli::parse();

    // Initialize logging with verbose flag (do this early for config loading messages)
    let log_level = if cli.verbose { "debug" } else { "warn" };
    let _ = vectorizer::logging::init_logging_with_level("vectorizer", log_level);

    // Load configuration from config.yml first
    let config = load_config(&cli.config);

    // CLI arguments override config.yml values
    let host = cli.host.unwrap_or(config.server.host);
    let port = cli.port.unwrap_or(config.server.port);

    info!("🚀 Starting Vectorizer Server");
    info!("🌐 Server: {}:{}", host, port);

    // Create and start the server
    let server = VectorizerServer::new().await?;

    // Start the server (this will block)
    if let Err(e) = server.start(&host, port).await {
        error!("❌ Server failed: {}", e);
        std::process::exit(1);
    }

    info!("✅ Server completed successfully");

    // Force exit to ensure process terminates
    // This prevents hanging if any background tasks are still running
    std::process::exit(0);
}
