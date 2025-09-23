//! Vectorizer Server - Main server with MCP integration
//!
//! This binary provides the main Vectorizer server with integrated MCP support
//! for IDE integration and AI model communication.

use axum::{Router, extract::State, response::Json, routing::get};
use serde_json::json;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{error, info};
use vectorizer::{
    auth::{AuthConfig, AuthManager},
    db::VectorStore,
    embedding::EmbeddingManager,
    mcp::{McpConfig, McpServer},
};

/// Application state
#[derive(Clone)]
struct AppState {
    vector_store: Arc<VectorStore>,
    #[allow(dead_code)]
    embedding_manager: Arc<RwLock<EmbeddingManager>>,
    auth_manager: Option<Arc<AuthManager>>,
    mcp_server: Option<Arc<McpServer>>,
}

/// Health check endpoint
async fn health_check() -> Json<serde_json::Value> {
    Json(json!({
        "status": "healthy",
        "service": "vectorizer",
        "version": env!("CARGO_PKG_VERSION"),
        "timestamp": chrono::Utc::now().to_rfc3339()
    }))
}

/// Get server status
async fn get_status(State(state): State<AppState>) -> Json<serde_json::Value> {
    let collections = state.vector_store.list_collections();
    let total_collections = collections.len();

    let mut total_vectors = 0;
    for collection_name in &collections {
        if let Ok(metadata) = state.vector_store.get_collection_metadata(collection_name) {
            total_vectors += metadata.vector_count;
        }
    }

    Json(json!({
        "status": "running",
        "collections": {
            "count": total_collections,
            "names": collections
        },
        "vectors": {
            "total": total_vectors
        },
        "mcp": {
            "enabled": state.mcp_server.is_some()
        },
        "auth": {
            "enabled": state.auth_manager.is_some()
        }
    }))
}

/// List collections endpoint
async fn list_collections(State(state): State<AppState>) -> Json<serde_json::Value> {
    let collections = state.vector_store.list_collections();

    let collection_info: Vec<serde_json::Value> = collections
        .into_iter()
        .map(
            |name| match state.vector_store.get_collection_metadata(&name) {
                Ok(metadata) => {
                    json!({
                        "name": name,
                        "vector_count": metadata.vector_count,
                        "dimension": metadata.config.dimension,
                        "metric": metadata.config.metric
                    })
                }
                Err(_) => {
                    json!({
                        "name": name,
                        "error": "Failed to get metadata"
                    })
                }
            },
        )
        .collect();

    Json(json!({
        "collections": collection_info
    }))
}

/// Create application router
fn create_app(state: AppState) -> Router {
    Router::new()
        .route("/health", get(health_check))
        .route("/status", get(get_status))
        .route("/collections", get(list_collections))
        .with_state(state)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter("vectorizer=info")
        .init();

    info!("Starting Vectorizer Server with MCP integration");

    // Initialize vector store
    let vector_store = Arc::new(VectorStore::new());
    info!("Vector store initialized");

    // Initialize embedding manager
    let embedding_manager = Arc::new(RwLock::new(EmbeddingManager::new()));
    info!("Embedding manager initialized");

    // Initialize authentication (optional)
    let auth_config = AuthConfig::default();
    let auth_manager = if auth_config.enabled {
        Some(Arc::new(AuthManager::new(auth_config)?))
    } else {
        None
    };

    if auth_manager.is_some() {
        info!("Authentication enabled");
    } else {
        info!("Authentication disabled");
    }

    // Initialize MCP server
    let mcp_config = McpConfig::default();
    let mcp_server = if mcp_config.enabled {
        Some(Arc::new(McpServer::new(
            mcp_config.clone(),
            Arc::clone(&vector_store),
            auth_manager.clone(),
        )))
    } else {
        None
    };

    if mcp_server.is_some() {
        info!(
            "MCP server enabled on {}:{}",
            mcp_config.host, mcp_config.port
        );
    } else {
        info!("MCP server disabled");
    }

    // Create application state
    let app_state = AppState {
        vector_store,
        embedding_manager,
        auth_manager,
        mcp_server,
    };

    // Start MCP server if enabled
    if let Some(mcp_server) = &app_state.mcp_server {
        let mcp_server_clone = Arc::clone(mcp_server);
        tokio::spawn(async move {
            if let Err(e) = mcp_server_clone.start().await {
                error!("Failed to start MCP server: {}", e);
            }
        });
    }

    // Create router
    let app = create_app(app_state);

    // Start HTTP server
    let listener = tokio::net::TcpListener::bind("127.0.0.1:15001").await?;
    info!("HTTP server listening on http://127.0.0.1:15001");

    axum::serve(listener, app).await?;

    Ok(())
}
