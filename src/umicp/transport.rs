//! UMICP Transport - Simple HTTP handler
//! Updated for v0.2.1: Native JSON types + Tool Discovery

use axum::Json;
use axum::extract::{Request, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde_json::json;
use tracing::{debug, error, info};
use umicp_core::{DiscoverableService, Envelope};

use super::{UmicpState, VectorizerDiscoveryService};

/// Main UMICP HTTP handler
pub async fn umicp_handler(State(state): State<UmicpState>, request: Request) -> Response {
    info!("🔌 UMICP request received");

    // Read body
    let body_bytes = match axum::body::to_bytes(request.into_body(), usize::MAX).await {
        Ok(bytes) => bytes,
        Err(e) => {
            error!("Failed to read body: {}", e);
            return (
                StatusCode::BAD_REQUEST,
                format!(r#"{{"error":"Failed to read body: {}"}}"#, e),
            )
                .into_response();
        }
    };

    let body_str = match std::str::from_utf8(&body_bytes) {
        Ok(s) => s,
        Err(e) => {
            error!("Invalid UTF-8: {}", e);
            return (StatusCode::BAD_REQUEST, r#"{"error":"Invalid UTF-8"}"#).into_response();
        }
    };

    debug!("Received body: {} bytes", body_str.len());

    // Parse envelope
    let envelope = match Envelope::deserialize(body_str) {
        Ok(env) => env,
        Err(e) => {
            error!("Failed to parse envelope: {}", e);
            return (
                StatusCode::BAD_REQUEST,
                format!(r#"{{"error":"Invalid UMICP envelope: {}"}}"#, e),
            )
                .into_response();
        }
    };

    // Validate
    if let Err(e) = envelope.validate() {
        error!("Envelope validation failed: {}", e);
        return (
            StatusCode::BAD_REQUEST,
            format!(r#"{{"error":"Validation failed: {}"}}"#, e),
        )
            .into_response();
    }

    debug!(
        "✅ Valid envelope - from: {}, to: {}, op: {:?}",
        envelope.from(),
        envelope.to(),
        envelope.operation()
    );

    // Process request
    let response_envelope = super::handlers::handle_umicp_request(state, envelope).await;

    match response_envelope {
        Ok(response) => match response.serialize() {
            Ok(json_str) => (StatusCode::OK, json_str).into_response(),
            Err(e) => {
                error!("Failed to serialize response: {}", e);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    r#"{"error":"Failed to serialize response"}"#,
                )
                    .into_response()
            }
        },
        Err(e) => {
            error!("Handler error: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!(r#"{{"error":"{}"}}"#, e),
            )
                .into_response()
        }
    }
}

/// UMICP Discovery Handler - Returns all 38+ available operations
/// Endpoint: GET /umicp/discover
pub async fn umicp_discover_handler(State(_state): State<UmicpState>) -> Json<serde_json::Value> {
    info!("🔍 UMICP discovery request received");

    let service = VectorizerDiscoveryService;
    let server_info = service.server_info();
    let operations = service.list_operations();

    // Operations are already serializable OperationSchema structs
    let operations_json: Vec<serde_json::Value> = operations
        .iter()
        .map(|op| serde_json::to_value(op).unwrap())
        .collect();

    Json(json!({
        "protocol": "UMICP",
        "version": "0.2.1",
        "server_info": {
            "server": server_info.server,
            "version": server_info.version,
            "protocol": server_info.protocol,
            "features": server_info.features,
            "operations_count": server_info.operations_count,
            "mcp_compatible": server_info.mcp_compatible,
            "metadata": server_info.metadata,
        },
        "operations": operations_json,
        "total_operations": operations.len(),
    }))
}
