//! MCP `ServerHandler` implementation used by the StreamableHTTP
//! transport. Constructed per-session from
//! [`super::routing::VectorizerServer::create_mcp_router`] and bridges
//! MCP tool calls into the existing [`crate::server::mcp::handlers`]
//! dispatch table.

use std::sync::Arc;

use crate::VectorStore;
use crate::embedding::EmbeddingManager;

/// MCP Service implementation
#[derive(Clone)]
pub(super) struct VectorizerMcpService {
    pub(super) store: Arc<VectorStore>,
    pub(super) embedding_manager: Arc<EmbeddingManager>,
    pub(super) cluster_manager: Option<Arc<crate::cluster::ClusterManager>>,
}

impl rmcp::ServerHandler for VectorizerMcpService {
    fn get_info(&self) -> rmcp::model::ServerInfo {
        use rmcp::model::{Implementation, ProtocolVersion, ServerCapabilities, ServerInfo};

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
    ) -> impl std::future::Future<
        Output = Result<rmcp::model::ListToolsResult, rmcp::model::ErrorData>,
    > + Send
    + '_ {
        async move {
            use rmcp::model::ListToolsResult;

            let tools = crate::server::mcp::tools::get_mcp_tools();

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
    ) -> impl std::future::Future<
        Output = Result<rmcp::model::CallToolResult, rmcp::model::ErrorData>,
    > + Send
    + '_ {
        async move {
            crate::server::mcp::handlers::handle_mcp_tool(
                request,
                self.store.clone(),
                self.embedding_manager.clone(),
                self.cluster_manager.clone(),
            )
            .await
        }
    }

    fn list_resources(
        &self,
        _request: Option<rmcp::model::PaginatedRequestParam>,
        _context: rmcp::service::RequestContext<rmcp::RoleServer>,
    ) -> impl std::future::Future<
        Output = Result<rmcp::model::ListResourcesResult, rmcp::model::ErrorData>,
    > + Send
    + '_ {
        async move {
            use rmcp::model::ListResourcesResult;
            Ok(ListResourcesResult {
                resources: vec![],
                next_cursor: None,
            })
        }
    }
}
