//! HTTP + MCP + gRPC server façade.
//!
//! The struct definitions and tiny helpers live here; everything else
//! is split by concern:
//!
//! - [`core`]           — bootstrap, routing, grpc, mcp service, shared
//!                        request helpers, workspace loader
//! - [`auth_handlers`]  — `/auth/*` REST handlers + middleware (already
//!                        split into its own directory)
//! - [`rest_handlers`]  — the main REST API (already split into its own
//!                        directory)
//! - [`mcp`]            — MCP dispatch table + tool catalog
//! - [`qdrant`]         — Qdrant-compatible REST handlers
//! - [`hub_handlers`]   — HiveHub backup / tenant / usage handlers
//! - [`files`]          — file-operation REST handlers + upload
//! - [`graph_handlers`], [`graphql_handlers`], [`replication_handlers`],
//!   [`discovery_handlers`], [`setup_handlers`], [`error_middleware`],
//!   [`embedded_assets`] — each a single-concern file at this level
//!
//! Downstream callers still see everything at its historic
//! `crate::server::X` path thanks to the `pub use` aliases below.

// Internal data-layout file: public fields are self-documenting; the
// blanket allow keeps `cargo doc -W missing-docs` clean without padding
// every field with a tautological `///` comment. See
// phase4_enforce-public-api-docs.
#![allow(missing_docs)]
// Internal data-layout file: public fields are self-documenting; the
// blanket allow keeps `cargo doc -W missing-docs` clean without padding
// every field with a tautological `///` comment. See
// phase4_enforce-public-api-docs.
#![allow(missing_docs)]

use std::sync::Arc;

mod auth_handlers;
pub mod capabilities;
mod core;
mod discovery_handlers;
mod embedded_assets;
pub mod error_middleware;
pub mod files;
mod graph_handlers;
mod graphql_handlers;
mod hub_handlers;
pub mod mcp;
pub mod metrics_middleware;
mod qdrant;
pub mod replication_handlers;
pub mod rest_handlers;
pub mod runtime_metrics;
mod setup_handlers;
pub mod ws;

pub use core::get_file_watcher_metrics;

pub use auth_handlers::csrf::require_csrf_middleware;
pub use auth_handlers::{
    AuthHandlerState, CreateApiKeyResponse, LoginRequest, LoginResponse, UserRecord,
    auth_middleware, create_api_key, login, require_admin_middleware, require_auth_middleware,
};
// `file_operations_handlers` is referenced as
// `crate::server::file_operations_handlers` by at least one external caller;
// keep the alias until that caller is migrated.
pub use files::operations as file_operations_handlers;
// Keep the old `crate::server::mcp_handlers::X` / `crate::server::mcp_tools::X`
// paths working for external callers (src/umicp, tests/api/mcp/*). `pub use`
// doesn't duplicate code, it re-exports.
pub use mcp::handlers as mcp_handlers;
pub use mcp::handlers::handle_mcp_tool;
pub use mcp::tools as mcp_tools;
pub use mcp::tools::get_mcp_tools;
use vectorizer::VectorStore;
use vectorizer::cache::SlowQueryRing;
use vectorizer::embedding::EmbeddingManager;
use vectorizer::file_watcher::{FileWatcherSystem, MetricsCollector};

/// Global server state shared with the `/metrics` handler.
#[derive(Clone)]
pub struct ServerState {
    pub file_watcher_system: Arc<tokio::sync::Mutex<Option<FileWatcherSystem>>>,
}

/// Vectorizer server state
#[derive(Clone)]
pub struct VectorizerServer {
    pub store: Arc<VectorStore>,
    pub embedding_manager: Arc<EmbeddingManager>,
    pub start_time: std::time::Instant,
    pub file_watcher_system:
        Arc<tokio::sync::Mutex<Option<vectorizer::file_watcher::FileWatcherSystem>>>,
    pub metrics_collector: Arc<MetricsCollector>,
    pub auto_save_manager: Option<Arc<vectorizer::db::AutoSaveManager>>,
    pub master_node: Option<Arc<vectorizer::replication::MasterNode>>,
    pub replica_node: Option<Arc<vectorizer::replication::ReplicaNode>>,
    pub query_cache: Arc<vectorizer::cache::query_cache::QueryCache<serde_json::Value>>,
    /// In-memory slow-query ring buffer (phase-14).
    pub slow_query_ring: SlowQueryRing,
    pub(super) background_task: Arc<
        tokio::sync::Mutex<
            Option<(
                tokio::task::JoinHandle<()>,
                tokio::sync::watch::Sender<bool>,
            )>,
        >,
    >,
    pub(super) system_collector_task: Arc<tokio::sync::Mutex<Option<tokio::task::JoinHandle<()>>>>,
    pub(super) file_watcher_task: Arc<tokio::sync::Mutex<Option<tokio::task::JoinHandle<()>>>>,
    pub(super) file_watcher_cancel:
        Arc<tokio::sync::Mutex<Option<tokio::sync::watch::Sender<bool>>>>,
    pub(super) grpc_task: Arc<tokio::sync::Mutex<Option<tokio::task::JoinHandle<()>>>>,
    pub(super) auto_save_task: Arc<tokio::sync::Mutex<Option<tokio::task::JoinHandle<()>>>>,
    /// Cluster manager (optional, only if cluster is enabled)
    pub cluster_manager: Option<Arc<vectorizer::cluster::ClusterManager>>,
    /// Cluster client pool (optional, only if cluster is enabled)
    pub cluster_client_pool: Option<Arc<vectorizer::cluster::ClusterClientPool>>,
    /// Maximum request body size in MB (from config)
    pub max_request_size_mb: usize,
    /// Snapshot manager (optional, for Qdrant snapshot API)
    pub snapshot_manager: Option<Arc<vectorizer::storage::SnapshotManager>>,
    /// Authentication handler state (optional, only if auth is enabled)
    pub auth_handler_state: Option<AuthHandlerState>,
    /// HiveHub manager (optional, only if hub integration is enabled)
    pub hub_manager: Option<Arc<vectorizer::hub::HubManager>>,
    /// User backup manager (optional, only if hub integration is enabled)
    pub backup_manager: Option<Arc<vectorizer::hub::UserBackupManager>>,
    /// MCP Hub Gateway for multi-tenant MCP operations
    pub mcp_hub_gateway: Option<Arc<vectorizer::hub::McpHubGateway>>,
    /// Raft consensus manager (optional, for HA mode)
    pub raft_manager: Option<Arc<vectorizer::cluster::raft_node::RaftManager>>,
    /// HA lifecycle manager (optional, for HA mode)
    pub ha_manager: Option<Arc<vectorizer::cluster::HaManager>>,
    /// Per-collection upsert queue admission tracker (issue #263).
    /// REST / gRPC / MCP upsert handlers call `try_admit` and return
    /// 429 / `RESOURCE_EXHAUSTED` on `AdmissionError::QueueFull`.
    pub upsert_queue: Arc<vectorizer::db::UpsertQueue>,
    /// Shared BackpressureGuard for the BM25 vocab-build path
    /// (issue #263). Held here so the metrics handler can sample
    /// `vectorizer_vocab_build_permits_available` at scrape time.
    pub backpressure_guard: Arc<vectorizer::db::BackpressureGuard>,
    /// Runtime metrics sampler (phase25): CPU, memory, connections, QPS,
    /// per-route latency. Wraps a 1-second background tick task.
    pub runtime_sampler: Arc<runtime_metrics::RuntimeSampler>,
    /// Dashboard broadcast bus sender (phase29 + phase30). Held here so
    /// collection-mutation handlers can publish an immediate
    /// `Collections` snapshot on create / delete / rename without
    /// waiting for the 30 s tick. `send` returns an error when there
    /// are no live receivers, which is the normal idle state — every
    /// caller drops it on the floor.
    pub dashboard_tx: tokio::sync::broadcast::Sender<runtime_metrics::DashboardEvent>,
}

/// Configuration for root user credentials.
#[derive(Debug, Clone, Default)]
pub struct RootUserConfig {
    /// Root username (defaults to "root" if not set)
    pub root_user: Option<String>,
    /// Root password (generates random if not set)
    pub root_password: Option<String>,
    /// Path to config file (defaults to "config.yml" if not set)
    pub config_path: Option<String>,
    /// When true and `auth.jwt_secret` is empty, generate a cryptographically
    /// random key on first boot and persist it under the auth data directory
    /// as `jwt_secret.key`. Opt-in so production deployments fail fast instead
    /// of silently running with an unconfigured secret.
    pub auto_generate_jwt_secret: bool,
}

impl VectorizerServer {
    /// Check if a request is a write operation that should be redirected to the leader
    pub(super) fn is_write_request(method: &axum::http::Method) -> bool {
        matches!(
            method,
            &axum::http::Method::POST
                | &axum::http::Method::PUT
                | &axum::http::Method::DELETE
                | &axum::http::Method::PATCH
        )
    }

    /// Check if authentication should be required based on host binding.
    /// Returns true if host is 0.0.0.0 (production mode) and auth is not enabled.
    #[allow(dead_code)]
    pub(super) fn should_require_auth(host: &str, auth_enabled: bool) -> bool {
        let is_production_bind = host == "0.0.0.0";
        is_production_bind && !auth_enabled
    }
}
