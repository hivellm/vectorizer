//! UMICP Protocol Integration for Vectorizer
//! 
//! This module provides UMICP protocol support for the vectorizer,
//! enabling high-performance streaming communication over HTTP.
//! 
//! Version 0.2.1: Native JSON types + Tool Discovery

use axum::response::Json;
use serde_json::Value;

pub mod handlers;
pub mod transport;
pub mod discovery;

pub use discovery::VectorizerDiscoveryService;

/// UMICP server state
#[derive(Clone)]
pub struct UmicpState {
    /// Vectorizer store reference
    pub store: std::sync::Arc<crate::db::VectorStore>,
    /// Embedding manager
    pub embedding_manager: std::sync::Arc<crate::embedding::EmbeddingManager>,
}

/// Health check for UMICP endpoint
pub async fn health_check() -> Json<Value> {
    Json(serde_json::json!({
        "protocol": "UMICP",
        "version": "1.0",
        "transport": "streamable-http",
        "status": "ok",
        "vectorizer_version": env!("CARGO_PKG_VERSION")
    }))
}
