//! Vectorizer Server - Unified MCP + REST API
//!
//! This is the new unified server that eliminates GRPC complexity
//! and provides direct MCP + REST API access.

use vectorizer::server::VectorizerServer;
use clap::{Parser, Subcommand};
use tracing::error;

#[derive(Parser)]
#[command(name = "vectorizer")]
#[command(about = "Vectorizer Server - Unified MCP + REST API")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Start the unified server
    Start {
        /// Server host
        #[arg(long, default_value = "127.0.0.1")]
        host: String,

        /// Server port
        #[arg(long, default_value = "15002")]
        port: u16,
    },
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize logging
    vectorizer::logging::init_logging("vectorizer");

    let cli = Cli::parse();

    match cli.command {
        Commands::Start { host, port } => {
            println!("🚀 Starting Vectorizer Server");
            println!("🌐 Server: {}:{}", host, port);

            // Create and start the server
            let server = VectorizerServer::new().await?;
            
            println!("✅ Server initialized successfully");
            println!("🎯 Press Ctrl+C to stop the server");
            
            // Start the server (this will block)
            if let Err(e) = server.start(&host, port).await {
                error!("❌ Server failed: {}", e);
                eprintln!("❌ Server failed: {}", e);
                std::process::exit(1);
            }

            println!("✅ Server completed successfully");
        }
    }

    Ok(())
}
