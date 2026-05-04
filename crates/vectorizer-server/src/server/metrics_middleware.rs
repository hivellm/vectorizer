//! Connection counter + latency middleware (phase25).
//!
//! Wraps every HTTP request to:
//! - increment/decrement the active-connection `AtomicUsize`
//! - record `(path, duration_ms, status_code)` in the `LatencyAggregator`

use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Instant;

use axum::body::Body;
use axum::http::Request;
use axum::middleware::Next;
use axum::response::Response;

use super::runtime_metrics::LatencyAggregator;

/// Axum middleware that tracks active connections and records per-route
/// latency samples into the shared `LatencyAggregator`.
pub async fn metrics_middleware(
    req: Request<Body>,
    next: Next,
    counter: Arc<AtomicUsize>,
    aggregator: Arc<LatencyAggregator>,
) -> Response {
    let route = req.uri().path().to_string();
    let start = Instant::now();
    counter.fetch_add(1, Ordering::Relaxed);

    let response = next.run(req).await;

    counter.fetch_sub(1, Ordering::Relaxed);
    let elapsed_ms = start.elapsed().as_millis().min(u32::MAX as u128) as u32;
    aggregator.record(&route, elapsed_ms, response.status().as_u16());

    response
}
