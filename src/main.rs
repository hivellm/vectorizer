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

    /// Project directory to load and index automatically
    #[arg(long)]
    project: Option<String>,
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

    // Load project documents if specified
    let embedding_manager = if let Some(project_path) = &args.project {
        info!("Loading project from: {}", project_path);

        let config = vectorizer::document_loader::LoaderConfig::default();
        let mut loader = vectorizer::document_loader::DocumentLoader::new(config);

            match loader.load_project(project_path, &store) {
                Ok(count) => {
                    info!("Successfully loaded {} document chunks", count);

                    // Print collection statistics
                    if let Ok(stats) = loader.get_stats(&store) {
                        info!(
                            "Collection stats: {}",
                            serde_json::to_string_pretty(&stats)?
                        );
                    }

                    // Extract the embedding manager from the loader
                    let mut manager = loader.into_embedding_manager();

                    // If a tokenizer exists, load it to ensure vocabulary persistence
                    let base = std::path::Path::new(project_path).join(".vectorizer");
                    let try_load = |name: &str, f: &mut dyn FnMut(&std::path::Path) -> bool| {
                        let path = base.join(name);
                        if path.exists() {
                            let ok = f(&path);
                            if ok { eprintln!("Tokenizer loaded: {}", path.to_string_lossy()); }
                        }
                    };

                    // Try load tokenizer for the configured provider
                    let _ = (|| {
                        let cfg = vectorizer::document_loader::LoaderConfig::default();
                        match cfg.embedding_type.as_str() {
                            "bm25" => {
                                try_load("tokenizer.bm25.json", &mut |p| {
                                    if let Some(prov) = manager.get_provider_mut("bm25") {
                                        if let Some(bm25) = prov.as_any_mut().downcast_mut::<vectorizer::embedding::Bm25Embedding>() {
                                            return bm25.load_vocabulary_json(p).is_ok();
                                        }
                                    }
                                    false
                                });
                            }
                            "tfidf" => {
                                try_load("tokenizer.tfidf.json", &mut |p| {
                                    if let Some(prov) = manager.get_provider_mut("tfidf") {
                                        if let Some(tfidf) = prov.as_any_mut().downcast_mut::<vectorizer::embedding::TfIdfEmbedding>() {
                                            return tfidf.load_vocabulary_json(p).is_ok();
                                        }
                                    }
                                    false
                                });
                            }
                            "bagofwords" => {
                                try_load("tokenizer.bow.json", &mut |p| {
                                    if let Some(prov) = manager.get_provider_mut("bagofwords") {
                                        if let Some(bow) = prov.as_any_mut().downcast_mut::<vectorizer::embedding::BagOfWordsEmbedding>() {
                                            return bow.load_vocabulary_json(p).is_ok();
                                        }
                                    }
                                    false
                                });
                            }
                            "charngram" => {
                                try_load("tokenizer.charngram.json", &mut |p| {
                                    if let Some(prov) = manager.get_provider_mut("charngram") {
                                        if let Some(cng) = prov.as_any_mut().downcast_mut::<vectorizer::embedding::CharNGramEmbedding>() {
                                            return cng.load_vocabulary_json(p).is_ok();
                                        }
                                    }
                                    false
                                });
                            }
                            _ => {}
                        }
                        Some(())
                    })();

                    manager
                }
                Err(e) => {
                    eprintln!("Failed to load project: {}", e);
                    std::process::exit(1);
                }
            }
    } else {
        // Create a default embedding manager if no project is loaded
        let config = vectorizer::document_loader::LoaderConfig::default();
        let loader = vectorizer::document_loader::DocumentLoader::new(config);
        loader.into_embedding_manager()
    };

    // Create and start the HTTP server
        let server = vectorizer::api::VectorizerServer::new(&args.host, args.port, store.into(), embedding_manager);

    info!("Starting REST API server...");
    server.start().await?;

    Ok(())
}
