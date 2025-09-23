//! Vectorizer server entry point

use clap::Parser;
use tracing::info;

/// Vectorizer - High-performance vector database
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Port to listen on
    #[arg(short, long, default_value_t = 15001)]
    port: u16,

    /// Host to bind to
    #[arg(short = 'H', long, default_value = "127.0.0.1")]
    host: String,

    /// Configuration file path
    #[arg(short, long)]
    config: Option<String>,
}

#[tokio::main]
async fn main() {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter("vectorizer=debug,info")
        .init();

    // Parse arguments
    let args = Args::parse();

    info!("Starting Vectorizer v{}", vectorizer::VERSION);
    info!("Binding to {}:{}", args.host, args.port);

    // TODO: Initialize server with configuration
    // TODO: Start REST API server
    // TODO: Start dashboard server

    // For now, just create a simple vector store to test compilation
    let _store = vectorizer::VectorStore::new();
    info!("Vector store initialized");

    // Keep the server running
    tokio::signal::ctrl_c().await.unwrap();
    info!("Shutting down...");
}
