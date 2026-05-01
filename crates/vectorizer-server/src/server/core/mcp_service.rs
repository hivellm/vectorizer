//! MCP `ServerHandler` implementation used by the StreamableHTTP
//! transport. Constructed per-session from
//! [`super::routing::VectorizerServer::create_mcp_router`] and bridges
//! MCP tool calls into the existing [`crate::server::mcp::handlers`]
//! dispatch table.

use std::sync::Arc;

use vectorizer::VectorStore;
use vectorizer::db::UpsertQueue;
use vectorizer::embedding::EmbeddingManager;

/// MCP Service implementation
#[derive(Clone)]
pub(super) struct VectorizerMcpService {
    pub(super) store: Arc<VectorStore>,
    pub(super) embedding_manager: Arc<EmbeddingManager>,
    pub(super) cluster_manager: Option<Arc<vectorizer::cluster::ClusterManager>>,
    /// Per-collection upsert admission tracker (issue #263). Mirrors
    /// REST/gRPC: an upsert tool call that would push the
    /// collection's in-flight depth past the configured hard limit
    /// returns a structured error.
    pub(super) upsert_queue: Arc<UpsertQueue>,
}

impl rmcp::ServerHandler for VectorizerMcpService {
    fn get_info(&self) -> rmcp::model::ServerInfo {
        use rmcp::model::{Implementation, ProtocolVersion, ServerCapabilities, ServerInfo};

        // rmcp 1.x marked `Implementation` + `ServerInfo` as
        // `#[non_exhaustive]`, so struct-literal syntax is no longer
        // legal — build them through the `Implementation::new` +
        // `InitializeResult::new` builder chains instead.
        ServerInfo::new(ServerCapabilities::builder().enable_tools().build())
            .with_protocol_version(ProtocolVersion::default())
            .with_server_info(
                Implementation::new("vectorizer-server", env!("CARGO_PKG_VERSION"))
                    .with_title("HiveLLM Vectorizer Server")
                    .with_website_url("https://github.com/hivellm/hivellm"),
            )
            .with_instructions("HiveLLM Vectorizer - High-performance semantic search and vector database system with MCP + REST API.")
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

            Ok(ListToolsResult::with_all_items(tools))
        }
    }

    fn call_tool(
        &self,
        request: rmcp::model::CallToolRequestParams,
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
                self.upsert_queue.clone(),
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
            Ok(ListResourcesResult::with_all_items(vec![]))
        }
    }
}
