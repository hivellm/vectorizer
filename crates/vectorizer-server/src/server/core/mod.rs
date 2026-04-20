//! Internal lifecycle of the Vectorizer HTTP server.
//!
//! This subdirectory carries everything that used to sit in the 3300-line
//! `src/server/mod.rs`. It is split by runtime phase rather than by
//! endpoint, because those are the natural boundaries for review:
//!
//! - [`bootstrap`]        — `VectorizerServer::new` / `new_with_root_config`:
//!                          config load, subsystem init (embedding,
//!                          watcher, auto-save, cluster, replication,
//!                          Raft, hub, auth)
//! - [`routing`]          — `VectorizerServer::start` and
//!                          `create_mcp_router`: route composition,
//!                          middleware layering, graceful shutdown
//! - [`grpc`]             — `VectorizerServer::start_grpc_server`
//! - [`mcp_service`]      — the rmcp `ServerHandler` implementation
//!                          (`VectorizerMcpService`)
//! - [`helpers`]          — shared request-parsing + response helpers
//!                          (auth extractor, security headers, file-watcher
//!                          metrics endpoint)
//! - [`workspace_loader`] — workspace / file-watcher config loaders
//!                          used during bootstrap
//!
//! All of these files add `impl VectorizerServer { … }` blocks or free
//! functions against the types declared in `src/server/mod.rs`. The
//! module tree is a submodule of `crate::server`, so paths into the
//! rest of the server tree use `crate::server::…` rather than `super::`.

mod bootstrap;
mod grpc;
pub(super) mod helpers;
mod mcp_service;
mod routing;
mod workspace_loader;

// Re-export the one public handler so `src/server/mod.rs` can keep the
// existing `/metrics` route referencing `get_file_watcher_metrics`
// without knowing it moved.
pub use helpers::get_file_watcher_metrics;
