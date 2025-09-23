//! Vectorizer CLI - Administrative command-line interface
//! 
//! This binary provides administrative tools for managing the Vectorizer vector database

use vectorizer::cli;
use tracing_subscriber;

#[tokio::main]
async fn main() {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter("vectorizer=info")
        .init();

    // Run CLI
    if let Err(e) = cli::run().await {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}
