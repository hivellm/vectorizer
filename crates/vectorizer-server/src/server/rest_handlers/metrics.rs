//! `GET /metrics/runtime` handler (phase25).
//!
//! Returns a JSON snapshot of process-level and per-route metrics for
//! the dashboard. Requires admin authentication when auth is enabled.

use axum::extract::State;
use axum::response::Json;

use crate::server::VectorizerServer;
use crate::server::error_middleware::ErrorResponse;
use crate::server::runtime_metrics::RuntimeSnapshot;

/// GET /metrics/runtime — live process + request metrics for the dashboard.
///
/// Returns process CPU, memory, active connection count, rolling QPS,
/// per-route latency percentiles, and 5xx error rate over the last 60 s.
pub async fn get_runtime_metrics(
    State(state): State<VectorizerServer>,
) -> Result<Json<RuntimeSnapshot>, ErrorResponse> {
    Ok(Json(state.runtime_sampler.snapshot()))
}
