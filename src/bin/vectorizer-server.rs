//! Vectorizer Server - Main server with MCP integration
//!
//! This binary provides the main Vectorizer server with integrated MCP support
//! for IDE integration and AI model communication.

use clap::Parser;
use tracing::{info, warn};
use std::sync::Arc;
use std::path::PathBuf;
use std::fs;
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

    // Try to load existing vector store first
    let vector_store_path = if let Some(project_path) = &args.project {
        PathBuf::from(project_path).join(".vectorizer").join("vector_store.bin")
    } else {
        PathBuf::from(".vectorizer").join("vector_store.bin")
    };

    let mut vector_store = if vector_store_path.exists() {
        info!("Loading existing vector store from: {:?}", vector_store_path);
        match VectorStore::load(&vector_store_path) {
            Ok(store) => {
                info!("Successfully loaded vector store with {} collections", store.list_collections().len());
                Arc::new(store)
            }
            Err(e) => {
                warn!("Failed to load vector store from {:?}: {}, creating new one", vector_store_path, e);
                Arc::new(VectorStore::new())
            }
        }
    } else {
        info!("No existing vector store found, creating new one");
        Arc::new(VectorStore::new())
    };

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
        let mut loader = DocumentLoader::new(loader_config.clone());

        // Check if we need to load documents (only if collection doesn't exist or cache is invalid)
        let collection_name = loader_config.collection_name.clone();
        let should_load_documents = if vector_store.list_collections().contains(&collection_name) {
            info!("Collection '{}' already exists in loaded vector store", collection_name);
            // Check if cache is still valid
            let cache_path = PathBuf::from(project_path).join(".vectorizer").join("cache.bin");
            if cache_path.exists() {
                match loader.is_cache_valid(&cache_path.to_string_lossy()) {
                    Ok(is_valid) => {
                        if is_valid {
                            info!("Document cache is valid, skipping document loading");
                            false
                        } else {
                            info!("Document cache is outdated, reloading documents");
                            true
                        }
                    }
                    Err(_) => {
                        info!("Could not validate document cache, reloading documents");
                        true
                    }
                }
            } else {
                info!("No document cache found, reloading documents");
                true
            }
        } else {
            info!("Collection '{}' not found in vector store, loading documents", collection_name);
            true
        };

        if should_load_documents {
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

                    // Save the vector store after loading documents
                    let vector_store_dir = PathBuf::from(project_path).join(".vectorizer");
                    if let Err(e) = fs::create_dir_all(&vector_store_dir) {
                        warn!("Failed to create .vectorizer directory for vector store: {}", e);
                    } else {
                        match Arc::get_mut(&mut vector_store) {
                            Some(store) => {
                                if let Err(e) = store.save(&vector_store_path) {
                                    warn!("Failed to save vector store to {:?}: {}", vector_store_path, e);
                                } else {
                                    info!("Vector store saved successfully to {:?}", vector_store_path);
                                }
                            }
                            None => {
                                warn!("Could not get mutable reference to vector store for saving");
                            }
                        }
                    }
                }
                Err(e) => {
                    eprintln!("Failed to load project: {}", e);
                    std::process::exit(1);
                }
            }
        } else {
            // Load tokenizer from saved files even if not reloading documents
            let vectorizer_dir = PathBuf::from(project_path).join(".vectorizer");
            match loader_config.embedding_type.as_str() {
                "bm25" => {
                    let tokenizer_path = vectorizer_dir.join("tokenizer.bm25.json");
                    if tokenizer_path.exists() {
                        if let Some(provider) = loader.get_embedding_manager_mut().get_provider_mut("bm25") {
                            if let Some(bm25) = provider.as_any_mut().downcast_mut::<vectorizer::embedding::Bm25Embedding>() {
                                if let Err(e) = bm25.load_vocabulary_json(&tokenizer_path) {
                                    warn!("Failed to load BM25 tokenizer from {}: {}", tokenizer_path.display(), e);
                                } else {
                                    info!("Loaded BM25 tokenizer from: {}", tokenizer_path.display());
                                }
                            }
                        }
                    }
                }
                _ => {} // Other embedding types don't need special loading
            }

            // Print existing collection statistics
            if let Ok(stats) = loader.get_stats(&vector_store) {
                info!(
                    "Existing collection stats: {}",
                    serde_json::to_string_pretty(&stats)?
                );
            }
        }
        // Extract embedding manager from the same loader used for indexing
        let mut embedding_manager = loader.into_embedding_manager();

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
