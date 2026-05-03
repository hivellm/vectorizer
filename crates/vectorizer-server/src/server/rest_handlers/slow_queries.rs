//! Slow-query log REST handlers.
//!
//! - `list_slow_queries`      — GET  /slow_queries
//! - `set_slow_query_config`  — POST /slow_queries/config

#![allow(missing_docs)]

use axum::extract::State;
use axum::response::Json;
use serde_json::{Value, json};
use tracing::info;
use vectorizer::cache::{SlowQueryConfig, SlowQueryRing};

use crate::server::VectorizerServer;
use crate::server::error_middleware::{ErrorResponse, create_validation_error};

/// GET /slow_queries
///
/// Returns all entries currently held in the in-memory slow-query ring
/// buffer, oldest first.
pub async fn list_slow_queries(State(state): State<VectorizerServer>) -> Json<Value> {
    let ring: &SlowQueryRing = &state.slow_query_ring;
    let entries = ring.entries();
    let config = ring.config();

    let items: Vec<Value> = entries
        .iter()
        .map(|e| {
            json!({
                "timestamp": e.timestamp.to_rfc3339(),
                "collection": e.collection,
                "k": e.k,
                "duration_ms": e.duration_ms,
            })
        })
        .collect();

    Json(json!({
        "entries": items,
        "total": items.len(),
        "config": {
            "threshold_ms": config.threshold_ms,
            "capacity": config.capacity,
        },
    }))
}

/// POST /slow_queries/config
///
/// Body: `{"threshold_ms": 200, "capacity": 500}`
///
/// Reconfigures the slow-query ring buffer. Existing entries are
/// retained; if the new capacity is smaller than the current entry
/// count, the oldest entries are evicted.
pub async fn set_slow_query_config(
    State(state): State<VectorizerServer>,
    Json(payload): Json<Value>,
) -> Result<Json<Value>, ErrorResponse> {
    let threshold_ms = match payload.get("threshold_ms").and_then(|v| v.as_u64()) {
        Some(t) => t,
        None => {
            return Err(create_validation_error(
                "threshold_ms",
                "missing or invalid threshold_ms; must be a non-negative integer",
            ));
        }
    };
    let capacity = payload
        .get("capacity")
        .and_then(|v| v.as_u64())
        .unwrap_or(1_000) as usize;

    if capacity == 0 {
        return Err(create_validation_error(
            "capacity",
            "capacity must be at least 1",
        ));
    }

    let new_config = SlowQueryConfig {
        threshold_ms,
        capacity,
    };

    state.slow_query_ring.set_config(new_config.clone());

    info!(
        "set_slow_query_config: threshold_ms={}, capacity={}",
        threshold_ms, capacity
    );

    Ok(Json(json!({
        "threshold_ms": new_config.threshold_ms,
        "capacity": new_config.capacity,
        "status": "ok",
    })))
}
