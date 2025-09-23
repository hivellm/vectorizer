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
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter("vectorizer=debug,tower_http=debug,axum=debug")
        .init();

    // Parse arguments
    let args = Args::parse();

    info!("Starting Vectorizer v{}", vectorizer::VERSION);
    info!("Binding to {}:{}", args.host, args.port);

    // Initialize vector store
    let store = vectorizer::VectorStore::new();
    info!("Vector store initialized");

    // Create and start the HTTP server
    let server = vectorizer::api::VectorizerServer::new(&args.host, args.port, store);
    
    info!("Starting REST API server...");
    server.start().await?;

    Ok(())
}
