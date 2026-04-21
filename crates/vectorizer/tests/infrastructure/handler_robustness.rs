//! Handler robustness guards.
//!
//! Regression tests for the rule "no panic on untrusted input": a
//! malformed request body, an invalid path parameter, or a missing
//! header must come back as 4xx, never as 5xx or a crash. See
//! `.rulebook/specs/RUST.md#the-unwrapexpect-policy-tightened-in-phase3`
//! and the full sweep tracked under `phase4_enforce-no-unwrap-policy`.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use axum::body::Body;
use axum::http::{Request, StatusCode};
use axum::routing::post;
use axum::{Json, Router};
use serde::Deserialize;
use tower::ServiceExt;

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct InsertBody {
    collection: String,
    dimension: usize,
}

/// Minimal handler that takes a typed `Json<InsertBody>` — matches the
/// shape of many real REST handlers. If axum's Json extractor ever
/// starts returning 500 instead of 400 on a malformed body, this test
/// catches it.
async fn insert_handler(Json(_body): Json<InsertBody>) -> StatusCode {
    StatusCode::OK
}

fn router() -> Router {
    Router::new().route("/insert", post(insert_handler))
}

#[tokio::test]
async fn malformed_json_returns_400_not_500() {
    let response = router()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/insert")
                .header("content-type", "application/json")
                .body(Body::from("{ this is not valid json"))
                .unwrap(),
        )
        .await
        .expect("router handles request");

    let status = response.status();
    assert!(
        status.is_client_error(),
        "malformed JSON must produce 4xx, got {status}"
    );
    assert_ne!(
        status,
        StatusCode::INTERNAL_SERVER_ERROR,
        "malformed JSON must not panic into 500"
    );
}

#[tokio::test]
async fn missing_required_field_returns_400_not_500() {
    // Body is valid JSON but missing the `dimension` field. The Json
    // extractor must reject with 422 (unprocessable entity) or 400,
    // never 500.
    let response = router()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/insert")
                .header("content-type", "application/json")
                .body(Body::from(r#"{"collection": "x"}"#))
                .unwrap(),
        )
        .await
        .expect("router handles request");

    let status = response.status();
    assert!(
        status.is_client_error(),
        "missing field must produce 4xx, got {status}"
    );
}

#[tokio::test]
async fn wrong_content_type_returns_client_error_not_500() {
    let response = router()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/insert")
                .header("content-type", "text/plain")
                .body(Body::from(r#"{"collection": "x", "dimension": 8}"#))
                .unwrap(),
        )
        .await
        .expect("router handles request");

    let status = response.status();
    assert!(
        status.is_client_error(),
        "wrong content-type must produce 4xx, got {status}"
    );
}

#[tokio::test]
async fn empty_body_returns_client_error_not_500() {
    let response = router()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/insert")
                .header("content-type", "application/json")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .expect("router handles request");

    let status = response.status();
    assert!(
        status.is_client_error(),
        "empty body must produce 4xx, got {status}"
    );
}
