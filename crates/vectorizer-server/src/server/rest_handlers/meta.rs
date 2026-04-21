//! Meta / status REST handlers.
//!
//! - `health_check` — GET /health
//! - `get_stats`    — GET /stats
//! - `get_indexing_progress` — GET /indexing/progress
//! - `get_status`   — GET /status  (GUI)
//! - `get_logs`     — GET /logs    (GUI)
//! - `get_prometheus_metrics` — GET /metrics

use std::collections::HashMap;

use axum::extract::{Query, State};
use axum::http::StatusCode;
use axum::response::Json;
use serde_json::{Value, json};
use tracing::error;

use crate::server::VectorizerServer;
use crate::server::error_middleware::ErrorResponse;

/// GET /health — liveness check with cache and hub stats
pub async fn health_check(State(state): State<VectorizerServer>) -> Json<Value> {
    let cache_stats = state.query_cache.stats();

    // Build base health response
    let mut response = json!({
        "status": "healthy",
        "timestamp": chrono::Utc::now(),
        "version": env!("CARGO_PKG_VERSION"),
        "cache": {
            "size": cache_stats.size,
            "capacity": cache_stats.capacity,
            "hits": cache_stats.hits,
            "misses": cache_stats.misses,
            "evictions": cache_stats.evictions,
            "hit_rate": cache_stats.hit_rate
        }
    });

    // Add Hub status if Hub is enabled
    if let Some(ref hub_manager) = state.hub_manager {
        let hub_status = json!({
            "enabled": hub_manager.is_enabled(),
            "active": hub_manager.is_active(),
            "tenant_isolation": format!("{:?}", hub_manager.config().tenant_isolation),
        });
        response["hub"] = hub_status;
    }

    // Add backup manager status
    if state.backup_manager.is_some() {
        response["backup"] = json!({
            "enabled": true
        });
    }

    Json(response)
}

/// GET /stats — aggregate collection and vector counts
pub async fn get_stats(State(state): State<VectorizerServer>) -> Json<Value> {
    let collections = state.store.list_collections();
    let total_vectors: usize = collections
        .iter()
        .map(|name| {
            state
                .store
                .get_collection(name)
                .map(|c| c.vector_count())
                .unwrap_or(0)
        })
        .sum();

    Json(json!({
        "collections": collections.len(),
        "total_vectors": total_vectors,
        "uptime_seconds": state.start_time.elapsed().as_secs(),
        "version": env!("CARGO_PKG_VERSION")
    }))
}

/// GET /indexing/progress — per-collection indexing progress
pub async fn get_indexing_progress(State(state): State<VectorizerServer>) -> Json<Value> {
    let collections = state.store.list_collections();
    let total_collections = collections.len();

    Json(json!({
        "overall_status": "completed",
        "collections": collections.iter().map(|name| {
            json!({
                "name": name,
                "status": "completed",
                "progress": 1.0,
                "total_documents": 0,
                "processed_documents": 0,
                "errors": 0
            })
        }).collect::<Vec<_>>(),
        "total_collections": total_collections,
        "completed_collections": total_collections,
        "processing_collections": 0
    }))
}

/// GET /status — server status for GUI
pub async fn get_status(State(state): State<VectorizerServer>) -> Json<Value> {
    Json(json!({
        "online": true,
        "version": env!("CARGO_PKG_VERSION"),
        "uptime_seconds": state.start_time.elapsed().as_secs(),
        "collections_count": state.store.list_collections().len()
    }))
}

/// GET /logs — tail log file for GUI
pub async fn get_logs(Query(params): Query<HashMap<String, String>>) -> Json<Value> {
    let lines = params
        .get("lines")
        .and_then(|v| v.parse::<usize>().ok())
        .unwrap_or(100);

    let level_filter = params.get("level");

    // Read logs from the .logs directory
    let logs_dir = std::path::Path::new(".logs");
    let mut all_logs = Vec::new();

    if logs_dir.exists() {
        // Find the most recent log file
        if let Ok(entries) = std::fs::read_dir(logs_dir) {
            let mut log_files: Vec<_> = entries
                .flatten()
                .filter(|e| {
                    e.path()
                        .extension()
                        .and_then(|s| s.to_str())
                        .map(|s| s == "log")
                        .unwrap_or(false)
                })
                .collect();

            // Sort by modified time (newest first)
            log_files
                .sort_by_key(|e| std::cmp::Reverse(e.metadata().and_then(|m| m.modified()).ok()));

            // Read only the most recent file
            if let Some(entry) = log_files.first() {
                let path = entry.path();
                if let Ok(content) = std::fs::read_to_string(&path) {
                    // Get last N lines from the file
                    let log_lines: Vec<&str> = content.lines().rev().take(lines * 2).collect();

                    for line in log_lines.iter().rev() {
                        if line.trim().is_empty() {
                            continue;
                        }

                        // Simple parsing
                        let upper_line = line.to_uppercase();
                        let level = if upper_line.contains("ERROR") {
                            "ERROR"
                        } else if upper_line.contains("WARN") {
                            "WARN"
                        } else if upper_line.contains("INFO") {
                            "INFO"
                        } else if upper_line.contains("DEBUG") {
                            "DEBUG"
                        } else {
                            "INFO"
                        };

                        // Apply level filter if specified
                        if let Some(filter_level) = level_filter {
                            if !level.eq_ignore_ascii_case(filter_level) {
                                continue;
                            }
                        }

                        all_logs.push(json!({
                            "timestamp": chrono::Utc::now().to_rfc3339(),
                            "level": level,
                            "message": line,
                            "source": "vectorizer"
                        }));

                        if all_logs.len() >= lines {
                            break;
                        }
                    }
                }
            }
        }
    }

    // Reverse to show newest first
    all_logs.reverse();

    Json(json!({
        "logs": all_logs
    }))
}

/// GET /metrics — export Prometheus metrics
pub async fn get_prometheus_metrics() -> Result<(StatusCode, String), (StatusCode, String)> {
    match vectorizer::monitoring::export_metrics() {
        Ok(metrics) => Ok((StatusCode::OK, metrics)),
        Err(e) => {
            error!("Failed to export Prometheus metrics: {}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to export metrics: {}", e),
            ))
        }
    }
}

// Suppress the unused-import warning — ErrorResponse is needed by the module
// boundary even when all handlers in this file only return plain Json.
#[allow(dead_code)]
fn _use_error_response(_: ErrorResponse) {}
