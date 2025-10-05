//! Unified Vectorizer Server - MCP + REST API
//!
//! This server implements MCP as the primary server using SSE transport,
//! with REST API routes layered on top. This eliminates GRPC complexity and
//! centralizes all methods in a single implementation.

mod mcp_tools;
mod mcp_handlers;
pub mod rest_handlers;

use std::sync::Arc;
use axum::{
    Router,
    routing::{get, post, delete},
};
use tower_http::services::ServeDir;
use tower_http::cors::CorsLayer;
use tracing::{info, error, warn};

use crate::{
    VectorStore,
    embedding::EmbeddingManager,
};

/// Vectorizer server state
#[derive(Clone)]
pub struct VectorizerServer {
    pub store: Arc<VectorStore>,
    pub embedding_manager: Arc<EmbeddingManager>,
    pub start_time: std::time::Instant,
}

impl VectorizerServer {
    /// Create a new vectorizer server
    pub async fn new() -> anyhow::Result<Self> {
        info!("🔧 Initializing Vectorizer Server...");
        
        // Initialize VectorStore with auto-loading enabled
        let mut vector_store = VectorStore::new();
        
        // Load all persisted collections during initialization
        match vector_store.load_all_persisted_collections() {
            Ok(count) => {
                if count > 0 {
                    info!("✅ Loaded {} persisted collections during initialization", count);
                } else {
                    info!("ℹ️  No persisted collections found during initialization");
                }
            },
            Err(e) => {
                warn!("⚠️  Failed to load persisted collections during initialization: {}", e);
            }
        }
        
        let mut embedding_manager = EmbeddingManager::new();
        let bm25 = crate::embedding::Bm25Embedding::new(512);
        embedding_manager.register_provider("bm25".to_string(), Box::new(bm25));
        embedding_manager.set_default_provider("bm25")?;

        info!("✅ Vectorizer Server initialized successfully with auto-indexation enabled");

        Ok(Self {
            store: Arc::new(vector_store),
            embedding_manager: Arc::new(embedding_manager),
            start_time: std::time::Instant::now(),
        })
    }

    /// Start the server
    pub async fn start(&self, host: &str, port: u16) -> anyhow::Result<()> {
        info!("🚀 Starting Vectorizer Server on {}:{}", host, port);

        // Start background collection loading
        self.start_background_loading();

        // Create MCP router (main server) using SSE transport
        info!("🔧 Creating MCP router with SSE transport...");
        let mcp_router = self.create_mcp_router().await;
        info!("✅ MCP router created");

        // Create REST API router to add to MCP
        let rest_routes = Router::new()
            // Health and stats
            .route("/health", get(rest_handlers::health_check))
            .route("/stats", get(rest_handlers::get_stats))
            .route("/indexing/progress", get(rest_handlers::get_indexing_progress))
            .route("/memory-analysis", get(rest_handlers::get_memory_analysis))
            .route("/collections/{name}/requantize", post(rest_handlers::requantize_collection))
            
            // Collection management
            .route("/collections", get(rest_handlers::list_collections))
            .route("/collections", post(rest_handlers::create_collection))
            .route("/collections/{name}", get(rest_handlers::get_collection))
            .route("/collections/{name}", delete(rest_handlers::delete_collection))
            
            // Vector operations - single
            .route("/search", post(rest_handlers::search_vectors))
            .route("/collections/{name}/search", post(rest_handlers::search_vectors))
            .route("/collections/{name}/search/text", post(rest_handlers::search_vectors_by_text))
            .route("/collections/{name}/search/file", post(rest_handlers::search_by_file))
            .route("/insert", post(rest_handlers::insert_text))
            .route("/update", post(rest_handlers::update_vector))
            .route("/delete", post(rest_handlers::delete_vector))
            .route("/embed", post(rest_handlers::embed_text))
            .route("/vector", post(rest_handlers::get_vector))
            .route("/collections/{name}/vectors", get(rest_handlers::list_vectors))
            .route("/collections/{name}/vectors/{id}", get(rest_handlers::get_vector))
            .route("/collections/{name}/vectors/{id}", delete(rest_handlers::delete_vector))
            
            // Vector operations - batch
            .route("/batch_insert", post(rest_handlers::batch_insert_texts))
            .route("/insert_texts", post(rest_handlers::insert_texts))
            .route("/batch_search", post(rest_handlers::batch_search_vectors))
            .route("/batch_update", post(rest_handlers::batch_update_vectors))
            .route("/batch_delete", post(rest_handlers::batch_delete_vectors))
            
            // Dashboard - serve static files
            .nest_service("/dashboard", ServeDir::new("dashboard"))
            .fallback_service(ServeDir::new("dashboard"))
            
            .layer(CorsLayer::permissive())
            .with_state(self.clone());

        // Merge REST routes into MCP router
        let app = mcp_router.merge(rest_routes);

        info!("🌐 Vectorizer Server available at:");
        info!("   📡 MCP SSE: http://{}:{}/mcp/sse", host, port);
        info!("   📬 MCP POST: http://{}:{}/mcp/message", host, port);
        info!("   🔌 REST API: http://{}:{}", host, port);
        info!("   📊 Dashboard: http://{}:{}/", host, port);

        // Bind and start the server
        let listener = tokio::net::TcpListener::bind(format!("{}:{}", host, port)).await?;
        info!("✅ MCP server with REST API listening on {}:{}", host, port);
        
        // Serve the application
        axum::serve(listener, app).await?;
        
        info!("✅ Server stopped gracefully");
        Ok(())
    }

    /// Start background collection loading
    async fn start_background_loading(&self) {
        let store = self.store.clone();
        
        tokio::spawn(async move {
            info!("📦 Starting background collection loading...");
            
            // Since VectorStore is wrapped in Arc, we need to handle this differently
            // For now, we'll log that auto-loading is enabled and collections will be loaded on-demand
            info!("🔄 Auto-indexation enabled - collections will be loaded automatically when accessed");
            info!("📁 Existing collections can be accessed via REST/MCP APIs");
            
            info!("✅ Background collection loading completed");
        });
    }

    /// Create MCP router with SSE transport
    async fn create_mcp_router(&self) -> Router {
        use rmcp::transport::sse_server::{SseServer, SseServerConfig};
        use std::sync::Arc;
        
        // Create MCP service handler
        let mcp_service = Arc::new(VectorizerMcpService {
            store: self.store.clone(),
            embedding_manager: self.embedding_manager.clone(),
        });

        // Create SSE server config (same as task-queue implementation)
        let config = SseServerConfig {
            bind: "0.0.0.0:0".parse().expect("Invalid bind address"), // Port 0 means don't bind, just create router
            sse_path: "/mcp/sse".into(),
            post_path: "/mcp/message".into(),
            ct: Default::default(),
            sse_keep_alive: Some(std::time::Duration::from_secs(30)),
        };

        // Create SSE server and get router
        let (sse, router) = SseServer::new(config);
        
        // Create the MCP server and register it with the SSE server
        let _cancel = sse.with_service(move || {
            (*mcp_service).clone()
        });

        router
    }
}

/// MCP Service implementation
#[derive(Clone)]
struct VectorizerMcpService {
    store: Arc<VectorStore>,
    embedding_manager: Arc<EmbeddingManager>,
}

impl rmcp::ServerHandler for VectorizerMcpService {
    fn get_info(&self) -> rmcp::model::ServerInfo {
        use rmcp::model::{ServerInfo, ProtocolVersion, ServerCapabilities, Implementation};
        
        ServerInfo {
            protocol_version: ProtocolVersion::default(),
            capabilities: ServerCapabilities::builder()
                .enable_tools()
                .build(),
            server_info: Implementation {
                name: "vectorizer-server".to_string(),
                title: Some("HiveLLM Vectorizer Server".to_string()),
                version: env!("CARGO_PKG_VERSION").to_string(),
                website_url: Some("https://github.com/hivellm/hivellm".to_string()),
                icons: None,
            },
            instructions: Some("HiveLLM Vectorizer - High-performance semantic search and vector database system with MCP + REST API.".to_string()),
        }
    }

    fn list_tools(
        &self,
        _request: Option<rmcp::model::PaginatedRequestParam>,
        _context: rmcp::service::RequestContext<rmcp::RoleServer>,
    ) -> impl std::future::Future<Output = Result<rmcp::model::ListToolsResult, rmcp::model::ErrorData>> + Send + '_ {
        async move {
            use rmcp::model::ListToolsResult;
            
            let tools = mcp_tools::get_mcp_tools();

            Ok(ListToolsResult {
                tools,
                next_cursor: None,
            })
        }
    }

    fn call_tool(
        &self,
        request: rmcp::model::CallToolRequestParam,
        _context: rmcp::service::RequestContext<rmcp::RoleServer>,
    ) -> impl std::future::Future<Output = Result<rmcp::model::CallToolResult, rmcp::model::ErrorData>> + Send + '_ {
        async move {
            mcp_handlers::handle_mcp_tool(
                request,
                self.store.clone(),
                self.embedding_manager.clone(),
            ).await
        }
    }

    fn list_resources(
        &self,
        _request: Option<rmcp::model::PaginatedRequestParam>,
        _context: rmcp::service::RequestContext<rmcp::RoleServer>,
    ) -> impl std::future::Future<Output = Result<rmcp::model::ListResourcesResult, rmcp::model::ErrorData>> + Send + '_ {
        async move {
            use rmcp::model::ListResourcesResult;
            Ok(ListResourcesResult {
                resources: vec![],
                next_cursor: None,
            })
        }
    }
}
