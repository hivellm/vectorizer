use rmcp::transport::sse_server::{SseServer, SseServerConfig};
use tracing_subscriber::{
    layer::SubscriberExt,
    util::SubscriberInitExt,
};
use std::sync::Arc;
use tokio::sync::Mutex;
use vectorizer::VectorStore;
use vectorizer::mcp_service::VectorizerService;

const BIND_ADDRESS: &str = "127.0.0.1:15002";

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info".to_string().into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Load project data
    let project_path = std::env::args().nth(1).unwrap_or_else(|| "../gov".to_string());

    tracing::info!("Loading project from: {}", project_path);

    // Try to load existing vector store first
    let vector_store_path = std::path::PathBuf::from(&project_path).join(".vectorizer").join("vector_store.bin");

    let mut vector_store = if vector_store_path.exists() {
        tracing::info!("Loading existing vector store from: {:?}", vector_store_path);
        match VectorStore::load(&vector_store_path) {
            Ok(store) => {
                tracing::info!("Successfully loaded vector store with {} collections", store.list_collections().len());
                Arc::new(store)
            }
            Err(e) => {
                tracing::warn!("Failed to load vector store from {:?}: {}, creating new one", vector_store_path, e);
                Arc::new(VectorStore::new())
            }
        }
    } else {
        tracing::info!("No existing vector store found, creating new one");
        Arc::new(VectorStore::new())
    };
    
    // Load documents
    let config = vectorizer::document_loader::LoaderConfig {
        max_chunk_size: 512,
        chunk_overlap: 50,
        allowed_extensions: vec![".md".to_string(), ".txt".to_string(), ".json".to_string()],
        embedding_dimension: 512,
        embedding_type: "bm25".to_string(),
        collection_name: "documents".to_string(),
        max_file_size: 10 * 1024 * 1024, // 10MB
    };
    
    let mut loader = vectorizer::document_loader::DocumentLoader::new(config.clone());

    // Check if we need to load documents (only if collection doesn't exist or cache is invalid)
    let collection_name = config.collection_name.clone();
    let should_load_documents = if vector_store.list_collections().contains(&collection_name) {
        tracing::info!("Collection '{}' already exists in loaded vector store", collection_name);
        // Check if cache is still valid
        let cache_path = std::path::PathBuf::from(&project_path).join(".vectorizer").join("cache.bin");
        if cache_path.exists() {
            match loader.is_cache_valid(&cache_path.to_string_lossy()) {
                Ok(is_valid) => {
                    if is_valid {
                        tracing::info!("Document cache is valid, skipping document loading");
                        false
                    } else {
                        tracing::info!("Document cache is outdated, reloading documents");
                        true
                    }
                }
                Err(_) => {
                    tracing::info!("Could not validate document cache, reloading documents");
                    true
                }
            }
        } else {
            tracing::info!("No document cache found, reloading documents");
            true
        }
    } else {
        tracing::info!("Collection '{}' not found in vector store, loading documents", collection_name);
        true
    };

    if should_load_documents {
        if let Err(e) = loader.load_project(&project_path, &vector_store) {
            tracing::error!("Failed to load project: {}", e);
            return Err(e.into());
        }

        // Save the vector store after loading documents
        let vector_store_dir = std::path::PathBuf::from(&project_path).join(".vectorizer");
        if let Err(e) = std::fs::create_dir_all(&vector_store_dir) {
            tracing::warn!("Failed to create .vectorizer directory for vector store: {}", e);
        } else {
            match std::sync::Arc::get_mut(&mut vector_store) {
                Some(store) => {
                    if let Err(e) = store.save(&vector_store_path) {
                        tracing::warn!("Failed to save vector store to {:?}: {}", vector_store_path, e);
                    } else {
                        tracing::info!("Vector store saved successfully to {:?}", vector_store_path);
                    }
                }
                None => {
                    tracing::warn!("Could not get mutable reference to vector store for saving");
                }
            }
        }
    } else {
        // Load tokenizer from saved files even if not reloading documents
        let vectorizer_dir = std::path::PathBuf::from(&project_path).join(".vectorizer");
        match config.embedding_type.as_str() {
            "bm25" => {
                let tokenizer_path = vectorizer_dir.join("tokenizer.bm25.json");
                if tokenizer_path.exists() {
                    if let Some(provider) = loader.get_embedding_manager_mut().get_provider_mut("bm25") {
                        if let Some(bm25) = provider.as_any_mut().downcast_mut::<vectorizer::embedding::Bm25Embedding>() {
                            if let Err(e) = bm25.load_vocabulary_json(&tokenizer_path) {
                                tracing::warn!("Failed to load BM25 tokenizer from {}: {}", tokenizer_path.display(), e);
                            } else {
                                tracing::info!("Loaded BM25 tokenizer from: {}", tokenizer_path.display());
                            }
                        }
                    }
                }
            }
            _ => {} // Other embedding types don't need special loading
        }
    }

    // Extract the embedding manager with the loaded vocabulary
    let embedding_manager = Arc::new(Mutex::new(loader.into_embedding_manager()));

    tracing::info!("Project loaded successfully");

    let config = SseServerConfig {
        bind: BIND_ADDRESS.parse()?,
        sse_path: "/sse".to_string(),
        post_path: "/message".to_string(),
        ct: tokio_util::sync::CancellationToken::new(),
        sse_keep_alive: None,
    };

    let (sse_server, router) = SseServer::new(config);

    let listener = tokio::net::TcpListener::bind(sse_server.config.bind).await?;

    let ct = sse_server.config.ct.child_token();

    let router_svc = router.into_make_service();
    let server = axum::serve(listener, router_svc).with_graceful_shutdown(async move {
        ct.cancelled().await;
        tracing::info!("sse server cancelled");
    });

    tokio::spawn(async move {
        if let Err(e) = server.await {
            tracing::error!(error = %e, "sse server shutdown with error");
        }
    });

    let ct = sse_server.with_service(move || {
        VectorizerService::new(vector_store.clone(), embedding_manager.clone())
    });

    tracing::info!("Vectorizer MCP SSE Server started on {}", BIND_ADDRESS);

    tokio::signal::ctrl_c().await?;
    ct.cancel();
    Ok(())
}
