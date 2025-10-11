//! UMICP Protocol Integration for Vectorizer
//! 
//! This module provides UMICP protocol support for the vectorizer,
//! enabling high-performance streaming communication over HTTP.

use axum::response::Json;
use serde_json::Value;

pub mod handlers;
pub mod transport;

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

/// Example UMICP envelope for testing
pub async fn example_envelope() -> Json<Value> {
    use umicp_core::{Envelope, OperationType, Capabilities};
    
    let mut caps = Capabilities::new();
    caps.insert("operation".to_string(), "list_collections".to_string());
    
    let envelope = Envelope::builder()
        .from("client-test")
        .to("vectorizer")
        .operation(OperationType::Data)
        .message_id("msg-001")
        .capabilities(caps)
        .build()
        .unwrap();
    
    let json_str = envelope.serialize().unwrap();
    let json_value: Value = serde_json::from_str(&json_str).unwrap();
    
    Json(json_value)
}
