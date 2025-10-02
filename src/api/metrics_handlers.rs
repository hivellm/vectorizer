use axum::{
    extract::State,
    http::StatusCode,
    response::Response,
};
use opentelemetry_prometheus::PrometheusExporter;
use std::sync::Arc;

use crate::telemetry::TelemetryState;
use super::handlers::AppState;

/// Handler for Prometheus metrics endpoint
pub async fn metrics_handler(
    State(state): State<AppState>,
) -> Result<Response<String>, StatusCode> {
    if let Some(telemetry_state) = &state.telemetry_state {
        if let Some(exporter) = telemetry_state.manager.get_prometheus_exporter() {
            // For now, return a simple metrics response
            let metrics = "# HELP vectorizer_info Vectorizer service information\n# TYPE vectorizer_info gauge\nvectorizer_info{version=\"0.21.0\"} 1\n";
            
            let response = Response::builder()
                .status(StatusCode::OK)
                .header("Content-Type", "text/plain; version=0.0.4; charset=utf-8")
                .body(metrics.to_string())
                .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
                
                Ok(response)
        } else {
            Err(StatusCode::NOT_FOUND)
        }
    } else {
        Err(StatusCode::NOT_FOUND)
    }
}

/// Health check endpoint that includes telemetry status
pub async fn health_check() -> (StatusCode, String) {
    let status = serde_json::json!({
        "status": "healthy",
        "timestamp": chrono::Utc::now().to_rfc3339(),
    });
    
    (StatusCode::OK, status.to_string())
}
