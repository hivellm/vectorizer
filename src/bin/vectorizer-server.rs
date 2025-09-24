//! Vectorizer Server - Main server with MCP integration
//!
//! This binary provides the main Vectorizer server with integrated MCP support
//! for IDE integration and AI model communication.

use clap::Parser;
use tracing::info;
use std::sync::Arc;
use vectorizer::{
    api::VectorizerServer,
    db::VectorStore,
    document_loader::{DocumentLoader, LoaderConfig},
};

#[derive(Parser)]
#[command(name = "vectorizer-server")]
#[command(about = "Vectorizer HTTP Server with document loading capabilities")]
struct Args {
    /// Host to bind the server to
    #[arg(long, default_value = "127.0.0.1")]
    host: String,

    /// Port to bind the server to
    #[arg(long, default_value = "15001")]
    port: u16,

    /// Project directory to load and vectorize
    #[arg(long)]
    project: Option<String>,

    /// Configuration file path
    #[arg(long, default_value = "config.yml")]
    config: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter("vectorizer=info")
        .init();

    info!("Starting Vectorizer Server with dashboard");

    // Configuration loading disabled for now
    let config = serde_yaml::Value::Null;

    // Initialize vector store
    let vector_store = Arc::new(VectorStore::new());
    info!("Vector store initialized");

    // Load project documents if specified
    if let Some(project_path) = &args.project {
        info!("Loading project from: {}", project_path);

            // Create optimized configuration for better relevance
            let loader_config = LoaderConfig {
                max_chunk_size: 4000, // Chunks maiores para mais contexto
                chunk_overlap: 200,   // Overlap maior para melhor continuidade
                allowed_extensions: vec![
                    "md".to_string(),
                    "txt".to_string(), 
                    "json".to_string(),
                    "rs".to_string(),
                    "js".to_string(),
                    "ts".to_string(),
                    "py".to_string(),
                ],
                embedding_dimension: 512, // Dimensão maior para melhor precisão
                embedding_type: "bm25".to_string(), // BM25 como padrão
                collection_name: "documents".to_string(),
                max_file_size: 5 * 1024 * 1024, // 5MB para arquivos maiores
            };
        let mut loader = DocumentLoader::new(loader_config);

        match loader.load_project(project_path, &vector_store) {
            Ok(count) => {
                info!("Successfully loaded {} document chunks", count);

                // Print collection statistics
                if let Ok(stats) = loader.get_stats(&vector_store) {
                    info!(
                        "Collection stats: {}",
                        serde_json::to_string_pretty(&stats)?
                    );
                }
            }
            Err(e) => {
                eprintln!("Failed to load project: {}", e);
                std::process::exit(1);
            }
        }
        // Extract embedding manager from the same loader used for indexing
        let mut embedding_manager = loader.into_embedding_manager();

        // If tokenizer exists, load it to ensure vocabulary persistence (for bm25)
        let tokenizer_path = std::path::Path::new(project_path).join(".vectorizer").join("tokenizer.bm25.json");
        if tokenizer_path.exists() {
            eprintln!("Loading tokenizer from {}", tokenizer_path.to_string_lossy());
            if let Some(provider) = embedding_manager.get_provider_mut("bm25") {
                if let Some(bm25) = provider.as_any_mut().downcast_mut::<vectorizer::embedding::Bm25Embedding>() {
                    if let Err(e) = bm25.load_vocabulary_json(&tokenizer_path) {
                        eprintln!("Failed to load tokenizer: {}", e);
                    } else {
                        eprintln!("Tokenizer loaded, vocabulary size: {}", bm25.vocabulary_size());
                    }
                }
            }
        } else {
            eprintln!("Tokenizer file not found at {} (will use in-memory vocabulary)", tokenizer_path.to_string_lossy());
        }

        // Start HTTP server
        let server = VectorizerServer::new(&args.host, args.port, vector_store, embedding_manager);
        server.start().await?;
        return Ok(());
    }

    // No project: create default embedding manager
    let embedding_manager = {
        let config = LoaderConfig::default();
        let loader = DocumentLoader::new(config);
        loader.into_embedding_manager()
    };

    // Start HTTP server
    let server = VectorizerServer::new(&args.host, args.port, vector_store, embedding_manager);
    server.start().await?;

    Ok(())
}
