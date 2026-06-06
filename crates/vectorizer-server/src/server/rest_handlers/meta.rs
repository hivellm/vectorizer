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

/// GET /stats — aggregate collection and vector counts.
///
/// Phase25 §5 additions: `default_quantization` (most-common quantization
/// label across active collections) and `compression_ratio` (mean ratio
/// across the collections sharing that label). `none` / `1.0` when the
/// store is empty.
pub async fn get_stats(State(state): State<VectorizerServer>) -> Json<Value> {
    let collections = state.store.list_collections();
    let mut total_vectors: usize = 0;

    // Counts per quantization label and the matching ratios so we can
    // pick the most-common label and average its ratio in one pass.
    let mut by_label: HashMap<&'static str, (usize, f64)> = HashMap::new();

    for name in &collections {
        let Ok(coll) = state.store.get_collection(name) else {
            continue;
        };
        total_vectors += coll.vector_count();

        let cfg = coll.config();
        let label = quantization_label(&cfg.quantization);
        let ratio = compression_ratio(&cfg.quantization, cfg.dimension) as f64;
        let entry = by_label.entry(label).or_insert((0, 0.0));
        entry.0 += 1;
        entry.1 += ratio;
    }

    let (default_quantization, compression_ratio) = by_label
        .into_iter()
        .max_by_key(|(_, (count, _))| *count)
        .map(|(label, (count, ratio_sum))| {
            let mean = if count > 0 {
                ratio_sum / count as f64
            } else {
                1.0
            };
            (label.to_string(), mean as f32)
        })
        .unwrap_or_else(|| ("none".to_string(), 1.0));

    // phase33 (#306): expose every registered embedding provider so
    // callers can discover what the deployment actually supports
    // before posting a collection. Without this, the only way to
    // notice that `fastembed` was never registered was to watch a
    // POST get coerced to bm25 — the silent-coercion bug we just
    // closed.
    let default_provider = state
        .embedding_manager
        .get_default_provider_name()
        .map(|s| s.to_string());
    let providers: Vec<Value> = state
        .embedding_manager
        .list_providers()
        .into_iter()
        .map(|name| {
            let dimension = state
                .embedding_manager
                .get_provider_dimension(&name)
                .unwrap_or(0);
            let is_default = default_provider.as_deref() == Some(name.as_str());
            json!({
                "name": name,
                "dimension": dimension,
                "default": is_default,
            })
        })
        .collect();

    Json(json!({
        "collections": collections.len(),
        "total_vectors": total_vectors,
        "uptime_seconds": state.start_time.elapsed().as_secs(),
        "version": env!("CARGO_PKG_VERSION"),
        "default_quantization": default_quantization,
        "compression_ratio": compression_ratio,
        "providers": providers,
        "default_provider": default_provider,
    }))
}

/// Stable label used by `default_quantization` in `GET /stats`.
fn quantization_label(q: &vectorizer::models::QuantizationConfig) -> &'static str {
    use vectorizer::models::QuantizationConfig;
    match q {
        QuantizationConfig::None => "none",
        QuantizationConfig::Binary => "binary",
        QuantizationConfig::SQ { bits: 4 } => "sq-4bit",
        QuantizationConfig::SQ { bits: 8 } => "sq-8bit",
        QuantizationConfig::SQ { bits: 16 } => "sq-16bit",
        QuantizationConfig::SQ { .. } => "sq",
        QuantizationConfig::PQ { .. } => "pq",
    }
}

/// Static compression ratio (uncompressed_bytes / compressed_bytes) for a
/// given quantization config. PQ depends on the collection's dimension;
/// the others are dimension-independent.
fn compression_ratio(q: &vectorizer::models::QuantizationConfig, dimension: usize) -> f32 {
    use vectorizer::models::QuantizationConfig;
    match q {
        QuantizationConfig::None => 1.0,
        QuantizationConfig::Binary => 32.0,
        QuantizationConfig::SQ { bits } if *bits > 0 => 32.0 / (*bits as f32),
        QuantizationConfig::SQ { .. } => 1.0,
        QuantizationConfig::PQ {
            n_centroids,
            n_subquantizers,
        } if *n_centroids > 1 && *n_subquantizers > 0 && dimension > 0 => {
            let bits_per_sub = (*n_centroids as f32).log2();
            let total_bits = (*n_subquantizers as f32) * bits_per_sub;
            if total_bits > 0.0 {
                (dimension as f32 * 32.0) / total_bits
            } else {
                1.0
            }
        }
        QuantizationConfig::PQ { .. } => 1.0,
    }
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
pub async fn get_prometheus_metrics(
    State(state): State<VectorizerServer>,
) -> Result<(StatusCode, String), (StatusCode, String)> {
    use vectorizer::monitoring::metrics::METRICS;

    // Issue #263: refresh per-collection backpressure gauges right
    // before we encode the snapshot Prometheus will scrape, so they
    // reflect the live in-flight depth (atomic counters change
    // independently of admission events). This keeps the gauges
    // monotonically accurate without spawning a periodic sampler.
    for (collection, depth) in state.upsert_queue.snapshot_depths() {
        let depth_f64 = depth as f64;
        METRICS
            .upsert_queue_depth
            .with_label_values(&[&collection])
            .set(depth_f64);
        METRICS
            .upsert_in_flight
            .with_label_values(&[&collection])
            .set(depth_f64);
    }
    METRICS
        .vocab_build_permits_available
        .set(state.backpressure_guard.available_permits() as f64);

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

#[cfg(test)]
mod tests {
    use vectorizer::models::QuantizationConfig;

    use super::{compression_ratio, quantization_label};

    #[test]
    fn quantization_label_covers_known_variants() {
        assert_eq!(quantization_label(&QuantizationConfig::None), "none");
        assert_eq!(quantization_label(&QuantizationConfig::Binary), "binary");
        assert_eq!(
            quantization_label(&QuantizationConfig::SQ { bits: 4 }),
            "sq-4bit"
        );
        assert_eq!(
            quantization_label(&QuantizationConfig::SQ { bits: 8 }),
            "sq-8bit"
        );
        assert_eq!(
            quantization_label(&QuantizationConfig::SQ { bits: 16 }),
            "sq-16bit"
        );
        assert_eq!(
            quantization_label(&QuantizationConfig::SQ { bits: 5 }),
            "sq"
        );
        assert_eq!(
            quantization_label(&QuantizationConfig::PQ {
                n_centroids: 256,
                n_subquantizers: 8
            }),
            "pq"
        );
    }

    #[test]
    fn compression_ratio_static_variants() {
        assert!((compression_ratio(&QuantizationConfig::None, 768) - 1.0).abs() < 1e-6);
        assert!((compression_ratio(&QuantizationConfig::Binary, 768) - 32.0).abs() < 1e-6);
        assert!((compression_ratio(&QuantizationConfig::SQ { bits: 8 }, 768) - 4.0).abs() < 1e-6);
        assert!((compression_ratio(&QuantizationConfig::SQ { bits: 16 }, 768) - 2.0).abs() < 1e-6);
    }

    #[test]
    fn compression_ratio_pq_depends_on_dimension() {
        // 256 centroids => 8 bits per sub-quantizer.
        // 8 subquantizers * 8 bits = 64 compressed bits.
        // dimension 512 * 32 = 16384 uncompressed bits.
        // ratio = 16384 / 64 = 256.
        let pq = QuantizationConfig::PQ {
            n_centroids: 256,
            n_subquantizers: 8,
        };
        assert!((compression_ratio(&pq, 512) - 256.0).abs() < 1e-3);

        // dimension 0 short-circuits to 1.0 to avoid divide-by-zero.
        assert!((compression_ratio(&pq, 0) - 1.0).abs() < 1e-6);
    }

    #[test]
    fn compression_ratio_handles_degenerate_configs() {
        assert!((compression_ratio(&QuantizationConfig::SQ { bits: 0 }, 768) - 1.0).abs() < 1e-6);
        let bad_pq = QuantizationConfig::PQ {
            n_centroids: 1,
            n_subquantizers: 8,
        };
        assert!((compression_ratio(&bad_pq, 512) - 1.0).abs() < 1e-6);
    }
}
