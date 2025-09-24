use rmcp::transport::sse_server::{SseServer, SseServerConfig};
use tracing_subscriber::{
    layer::SubscriberExt,
    util::SubscriberInitExt,
};
use std::sync::Arc;
use tokio::sync::Mutex;
use vectorizer::embedding::EmbeddingManager;
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

    // Initialize vector store and embedding manager
    let vector_store = Arc::new(VectorStore::new());

    // Load project data
    let project_path = std::env::args().nth(1).unwrap_or_else(|| "../gov".to_string());

    tracing::info!("Loading project from: {}", project_path);
    
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
    
    let mut loader = vectorizer::document_loader::DocumentLoader::new(config);

    if let Err(e) = loader.load_project(&project_path, &vector_store) {
        tracing::error!("Failed to load project: {}", e);
        return Err(e.into());
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
