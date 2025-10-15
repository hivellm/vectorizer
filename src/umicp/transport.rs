//! UMICP Transport - Simple HTTP handler

use axum::{
    extract::{Request, State},
    http::StatusCode,
    response::{IntoResponse, Response},
};
use umicp_core::Envelope;
use tracing::{debug, error, info};

use super::UmicpState;

/// Main UMICP HTTP handler
pub async fn umicp_handler(
    State(state): State<UmicpState>,
    request: Request,
) -> Response {
    info!("🔌 UMICP request received");
    
    // Read body
    let body_bytes = match axum::body::to_bytes(request.into_body(), usize::MAX).await {
        Ok(bytes) => bytes,
        Err(e) => {
            error!("Failed to read body: {}", e);
            return (StatusCode::BAD_REQUEST, format!(r#"{{"error":"Failed to read body: {}"}}"#, e)).into_response();
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
            return (StatusCode::BAD_REQUEST, format!(r#"{{"error":"Invalid UMICP envelope: {}"}}"#, e)).into_response();
        }
    };
    
    // Validate
    if let Err(e) = envelope.validate() {
        error!("Envelope validation failed: {}", e);
        return (StatusCode::BAD_REQUEST, format!(r#"{{"error":"Validation failed: {}"}}"#, e)).into_response();
    }
    
    debug!("✅ Valid envelope - from: {}, to: {}, op: {:?}", 
           envelope.from(), envelope.to(), envelope.operation());
    
    // Process request
    let response_envelope = super::handlers::handle_umicp_request(state, envelope).await;
    
    match response_envelope {
        Ok(response) => {
            match response.serialize() {
                Ok(json_str) => (StatusCode::OK, json_str).into_response(),
                Err(e) => {
                    error!("Failed to serialize response: {}", e);
                    (StatusCode::INTERNAL_SERVER_ERROR, r#"{"error":"Failed to serialize response"}"#).into_response()
                }
            }
        },
        Err(e) => {
            error!("Handler error: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, format!(r#"{{"error":"{}"}}"#, e)).into_response()
        }
    }
}
