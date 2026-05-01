//! Integration tests for the per-collection upsert backpressure 429
//! response surface (issue #263, phase9 §3).
//!
//! Exercises the wiring path used by REST upsert handlers:
//!   `UpsertQueue::try_admit` → `create_queue_full_error` →
//!   `ErrorResponse::into_response`.
//!
//! Asserts that:
//!   * status is exactly `429 Too Many Requests`
//!   * `Retry-After` header carries the configured value (in seconds)
//!   * the JSON body contains the structured `queue_full` reason
//!     callers can parse without HTML scraping
//!
//! gRPC and MCP have analogous wiring; the cross-protocol invariants
//! they share are covered by the `UpsertQueue` unit tests in
//! `crates/vectorizer/tests/core/upsert_queue.rs`.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use axum::body::to_bytes;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use serde_json::Value;
use vectorizer::config::BackpressureConfig;
use vectorizer::db::{AdmissionError, UpsertQueue};
use vectorizer_server::server::error_middleware::create_queue_full_error;

#[tokio::test]
async fn queue_full_response_is_429_with_retry_after_and_json_body() {
    // Force a hard limit of 1 so the second admission is refused.
    let cfg = BackpressureConfig {
        upsert_queue_high_water: 0, // < hard_limit, valid
        upsert_queue_hard_limit: 1,
        retry_after_seconds: 7,
        ..BackpressureConfig::default()
    };
    let queue = UpsertQueue::from_config(&cfg);

    let _first = queue.try_admit("docs").expect("first admission succeeds");
    let err = queue
        .try_admit("docs")
        .expect_err("second admission must hit hard limit");

    let AdmissionError::QueueFull {
        depth,
        hard_limit,
        retry_after_seconds,
    } = err;
    assert_eq!(depth, 1);
    assert_eq!(hard_limit, 1);
    assert_eq!(retry_after_seconds, 7);

    // Build the same response the REST handler would return.
    let response =
        create_queue_full_error("docs", depth, hard_limit, retry_after_seconds).into_response();

    assert_eq!(
        response.status(),
        StatusCode::TOO_MANY_REQUESTS,
        "queue-full response must be 429",
    );

    let retry_after = response
        .headers()
        .get(axum::http::header::RETRY_AFTER)
        .expect("Retry-After must be set on 429 responses")
        .to_str()
        .expect("Retry-After must be ASCII");
    assert_eq!(
        retry_after, "7",
        "Retry-After value must mirror retry_after_seconds",
    );

    // Body shape: structured queue_full reason.
    let body_bytes = to_bytes(response.into_body(), 64 * 1024)
        .await
        .expect("collect response body");
    let body: Value = serde_json::from_slice(&body_bytes).expect("body is JSON");

    assert_eq!(
        body.get("error_type").and_then(|v| v.as_str()),
        Some("queue_full")
    );
    assert_eq!(body.get("status_code").and_then(|v| v.as_u64()), Some(429));

    let details = body.get("details").expect("details present");
    assert_eq!(
        details.get("collection").and_then(|v| v.as_str()),
        Some("docs")
    );
    assert_eq!(details.get("depth").and_then(|v| v.as_u64()), Some(1));
    assert_eq!(details.get("hard_limit").and_then(|v| v.as_u64()), Some(1));
    assert_eq!(
        details.get("retry_after_seconds").and_then(|v| v.as_u64()),
        Some(7),
    );
}
