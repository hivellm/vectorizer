use rmcp::transport::sse_server::{SseServer, SseServerConfig};
use tracing_subscriber::{
    layer::SubscriberExt,
    util::SubscriberInitExt,
};
use std::sync::Arc;
use tokio::sync::Mutex;
use vectorizer::VectorStore;
use vectorizer::mcp_service::VectorizerService;

fn get_bind_address() -> String {
    let port = std::env::var("VECTORIZER_SERVER_PORT").unwrap_or_else(|_| "15002".to_string());
    format!("127.0.0.1:{}", port)
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize logging to file (unique per port to avoid conflicts)
    let port = std::env::var("VECTORIZER_SERVER_PORT").unwrap_or_else(|_| "15002".to_string());
    let log_filename = format!("vectorizer-mcp-server-{}.log", port);
    let log_file = match std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&log_filename) {
        Ok(file) => file,
        Err(e) => {
            eprintln!("Failed to open log file {}: {}", log_filename, e);
            std::process::exit(1);
        }
    };

    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info".to_string().into()),
        )
        .with(tracing_subscriber::fmt::layer().with_writer(move || log_file.try_clone().expect("Failed to clone log file")))
        .init();

    // Check if we have workspace configuration
    let workspace_info = std::env::var("VECTORIZER_WORKSPACE_INFO").ok();

    let vector_store_path = if workspace_info.is_some() {
        // Multi-project mode - use shared vector store
        std::path::PathBuf::from(".vectorizer").join("vector_store.bin")
    } else {
        // Legacy single project mode
        let project_path = std::env::args().nth(1).unwrap_or_else(|| "../gov".to_string());
        tracing::info!("Loading project from: {}", project_path);
        std::path::PathBuf::from(&project_path).join(".vectorizer").join("vector_store.bin")
    };

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
    
    // Load documents based on mode
    if workspace_info.is_some() {
        // Multi-project workspace mode
        let workspace_config_path = workspace_info.unwrap();
        tracing::info!("Loading workspace configuration from: {}", workspace_config_path);
        match load_workspace_projects_mcp(&workspace_config_path, Arc::clone(&vector_store)) {
            Ok(loaded_collections) => {
                tracing::info!("Successfully loaded {} collections from workspace", loaded_collections);
            }
            Err(e) => {
                tracing::error!("Failed to load workspace projects: {}", e);
                return Err(e.into());
            }
        }
    } else {
        // Legacy single project mode
        load_single_project_mcp(Arc::clone(&vector_store), vector_store_path)?;
    }

    // Create embedding manager from the loaded documents
    let embedding_manager = {
        // Create a dummy loader just to get the embedding manager
        let config = vectorizer::document_loader::LoaderConfig {
            max_chunk_size: 512,
            chunk_overlap: 50,
            allowed_extensions: vec![".md".to_string(), ".txt".to_string(), ".json".to_string()],
            include_patterns: vec![],
            exclude_patterns: vec![],
            embedding_dimension: 512,
            embedding_type: "bm25".to_string(),
            collection_name: "dummy".to_string(),
            max_file_size: 10 * 1024 * 1024, // 10MB
        };
        let loader = vectorizer::document_loader::DocumentLoader::new(config);
        loader.into_embedding_manager()
    };

    tracing::info!("Project loaded successfully");

    let bind_address = get_bind_address();
    let config = SseServerConfig {
        bind: bind_address.parse()?,
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
        // Create a new embedding manager for each service instance
        let service_embedding_manager = {
            let config = vectorizer::document_loader::LoaderConfig {
                max_chunk_size: 512,
                chunk_overlap: 50,
                allowed_extensions: vec![".md".to_string(), ".txt".to_string(), ".json".to_string()],
                include_patterns: vec![],
                exclude_patterns: vec![],
                embedding_dimension: 512,
                embedding_type: "bm25".to_string(),
                collection_name: "service".to_string(),
                max_file_size: 10 * 1024 * 1024, // 10MB
            };
            let loader = vectorizer::document_loader::DocumentLoader::new(config);
            Arc::new(Mutex::new(loader.into_embedding_manager()))
        };
        VectorizerService::new(vector_store.clone(), service_embedding_manager)
    });

    tracing::info!("Vectorizer MCP SSE Server started on {}", bind_address);

    tokio::signal::ctrl_c().await?;
    ct.cancel();
    Ok(())
}

/// Load all projects from a workspace configuration (MCP server version)
fn load_workspace_projects_mcp(workspace_path: &str, vector_store: Arc<VectorStore>) -> anyhow::Result<usize> {
    tracing::info!("Loading workspace info from: {}", workspace_path);

    let workspace_content = std::fs::read_to_string(workspace_path)
        .map_err(|e| anyhow::anyhow!("Failed to read workspace file: {}", e))?;

    let workspace_json: serde_json::Value = serde_json::from_str(&workspace_content)
        .map_err(|e| anyhow::anyhow!("Failed to parse workspace JSON: {}", e))?;

    let projects = workspace_json.get("projects")
        .and_then(|p| p.as_array())
        .ok_or_else(|| anyhow::anyhow!("No projects found in workspace configuration"))?;

    let mut total_collections = 0;

    for project_value in projects {
        if let Some(project_name) = project_value.get("name").and_then(|n| n.as_str()) {
            tracing::info!("Loading project: {}", project_name);

            // Load collections for this project
            if let Some(collections) = project_value.get("collections").and_then(|c| c.as_array()) {
                for collection in collections {
                    if let Some(collection_name) = collection.get("name").and_then(|n| n.as_str()) {
                        let project_path = project_value.get("path")
                            .and_then(|p| p.as_str())
                            .unwrap_or(".");

                        tracing::info!("🔄 Starting collection '{}' for project '{}' ({}D, {})",
                            collection_name, project_name,
                            collection.get("dimension").and_then(|d| d.as_u64()).unwrap_or(512),
                            collection.get("embedding").and_then(|e| e.get("model")).and_then(|m| m.as_str()).unwrap_or("bm25"));

                        // Create loader config for this collection
                        let loader_config = vectorizer::document_loader::LoaderConfig {
                            max_chunk_size: collection.get("chunk_size")
                                .and_then(|c| c.as_u64())
                                .unwrap_or(512) as usize,
                            chunk_overlap: collection.get("chunk_overlap")
                                .and_then(|c| c.as_u64())
                                .unwrap_or(50) as usize,
                            allowed_extensions: vec![], // Will use include_patterns instead
                            include_patterns: collection.get("processing")
                                .and_then(|p| p.get("include_patterns"))
                                .and_then(|i| i.as_array())
                                .map(|arr| arr.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect())
                                .unwrap_or_else(|| vec!["**/*.md".to_string(), "**/*.txt".to_string()]),
                            exclude_patterns: collection.get("processing")
                                .and_then(|p| p.get("exclude_patterns"))
                                .and_then(|e| e.as_array())
                                .map(|arr| arr.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect())
                                .unwrap_or_else(|| vec!["**/node_modules/**".to_string(), "**/target/**".to_string()]),
                            embedding_dimension: 512,
                            embedding_type: "bm25".to_string(),
                            collection_name: collection_name.to_string(),
                            max_file_size: 10 * 1024 * 1024, // 10MB
                        };

                        let mut loader = vectorizer::document_loader::DocumentLoader::new(loader_config);
                        match loader.load_project(project_path, &vector_store) {
                            Ok(_) => {
                                tracing::info!("✅ Collection '{}' completed successfully", collection_name);
                                total_collections += 1;
                            }
                            Err(e) => {
                                tracing::warn!("Failed to load collection '{}': {}", collection_name, e);
                            }
                        }
                    }
                }
            }
        }
    }

    Ok(total_collections)
}

/// Load a single project (legacy mode for MCP server)
fn load_single_project_mcp(vector_store: Arc<VectorStore>, vector_store_path: std::path::PathBuf) -> anyhow::Result<()> {
    let project_path = std::env::args().nth(1).unwrap_or_else(|| "../gov".to_string());

    let config = vectorizer::document_loader::LoaderConfig {
        max_chunk_size: 512,
        chunk_overlap: 50,
        allowed_extensions: vec![".md".to_string(), ".txt".to_string(), ".json".to_string()],
        include_patterns: vec![],
        exclude_patterns: vec![],
        embedding_dimension: 512,
        embedding_type: "bm25".to_string(),
        collection_name: "documents".to_string(),
        max_file_size: 10 * 1024 * 1024, // 10MB
    };

    let mut loader = vectorizer::document_loader::DocumentLoader::new(config.clone());
    let collection_name = config.collection_name.clone();

    // Check if we need to load documents
    let should_load_documents = !vector_store.list_collections().contains(&collection_name);

    if should_load_documents {
        tracing::info!("Loading documents for collection '{}'", collection_name);
        match loader.load_project(&project_path, &vector_store) {
            Ok(count) => {
                tracing::info!("Successfully loaded {} document chunks", count);

                // Save the vector store after loading documents
                let vector_store_dir = std::path::PathBuf::from(&project_path).join(".vectorizer");
                if let Err(e) = std::fs::create_dir_all(&vector_store_dir) {
                    tracing::warn!("Failed to create .vectorizer directory: {}", e);
                }
                if let Err(e) = vector_store.save(&vector_store_path) {
                    tracing::warn!("Failed to save vector store: {}", e);
                } else {
                    tracing::info!("Vector store saved successfully");
                }
            }
            Err(e) => {
                tracing::error!("Failed to load documents: {}", e);
                return Err(e.into());
            }
        }
    } else {
        tracing::info!("Collection '{}' already loaded", collection_name);
    }

    Ok(())
}
