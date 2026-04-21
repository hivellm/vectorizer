//! `vectorizer-server` — HTTP / gRPC / MCP server layer for Vectorizer.
//!
//! Sub-phase 4 of `phase4_split-vectorizer-workspace` extracts the
//! transport-facing surface (REST handlers, gRPC handlers, MCP tools,
//! WebSocket dashboard, replication TCP server, security middleware,
//! UMICP integration, structured logging) from the umbrella
//! `vectorizer` crate. The main `vectorizer` binary lives here.
//!
//! The umbrella `vectorizer` crate keeps the engine modules
//! (`db`, `embedding`, `models`, `cache`, `normalization`,
//! `persistence`, `file_*`, `discovery`, `hybrid_search`,
//! `intelligent_search`, `search`, `evaluation`, `batch`, `config`,
//! `workspace`, …) plus the four modules with bidirectional deps
//! into the engine (`auth`, `cluster`, `hub`, `monitoring`). A
//! follow-up sub-phase resolves those circulars and finishes the
//! split. Until then, this crate depends on the umbrella for that
//! engine + tied infrastructure.

#![allow(warnings)]
#![allow(clippy::unwrap_used, clippy::expect_used)]

pub mod api;
pub mod grpc;
pub mod logging;
pub mod protocol;
pub mod server;
pub mod umicp;
