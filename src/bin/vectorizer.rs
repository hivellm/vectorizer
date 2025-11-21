//! Vectorizer Server - Unified MCP + REST API
//!
//! This is the unified server that provides MCP + REST API access
//! for all vector operations.

#![allow(clippy::uninlined_format_args)]

use clap::Parser;
use tracing::error;
use vectorizer::server::VectorizerServer;

#[derive(Parser)]
#[command(name = "vectorizer")]
#[command(about = "Vectorizer Server - MCP + REST API")]
struct Cli {
    /// Server host
    #[arg(long, default_value = "0.0.0.0")]
    host: String,

    /// Server port
    #[arg(long, default_value = "15002")]
    port: u16,

    /// Enable verbose logging (default: only warnings and errors)
    #[arg(long)]
    verbose: bool,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    // Initialize logging with verbose flag
    let log_level = if cli.verbose { "debug" } else { "warn" };
    let _ = vectorizer::logging::init_logging_with_level("vectorizer", log_level);

    println!("🚀 Starting Vectorizer Server");
    println!("🌐 Server: {}:{}", cli.host, cli.port);

    // Create and start the server
    let server = VectorizerServer::new().await?;

    // Start the server (this will block)
    if let Err(e) = server.start(&cli.host, cli.port).await {
        error!("❌ Server failed: {}", e);
        eprintln!("❌ Server failed: {}", e);
        std::process::exit(1);
    }

    println!("✅ Server completed successfully");

    Ok(())
}
