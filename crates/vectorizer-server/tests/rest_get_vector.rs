//! `GET /collections/{name}/vectors/{id}` and `POST /vector` must return the
//! stored vector.
//!
//! Both routes used to share a handler that answered `200 OK` with a
//! fabricated body — `vec![0.1; 512]` for any id, in any collection, whether
//! or not the vector existed. `POST /vector` was worse: the handler's
//! `Path<(String, String)>` extractor could not be satisfied on a route with
//! no path parameters, so the request never reached a body.
//!
//! These tests pin the two properties a fabricated response cannot have: the
//! data matches what was inserted, and an unknown id is reported as missing.

#![allow(clippy::unwrap_used, clippy::expect_used)]

mod common;

use axum::http::StatusCode;
use common::TestApp;
use serde_json::{Value, json};

/// Insert one text vector into a fresh collection and return its id.
async fn seed_one(app: &TestApp, collection: &str) -> String {
    let (status, resp) = app
        .post_json(
            "/collections",
            json!({"name": collection, "dimension": 512, "metric": "cosine"}),
        )
        .await;
    assert!(status.is_success(), "create status {status}: {resp}");

    let (status, resp) = app
        .post_json(
            "/batch_insert",
            json!({
                "collection": collection,
                "texts": [{"text": "a probe document for vector retrieval",
                           "metadata": {"tag": "probe"}}],
            }),
        )
        .await;
    assert!(status.is_success(), "batch_insert status {status}: {resp}");
    resp["results"][0]["vector_ids"][0]
        .as_str()
        .expect("vector id")
        .to_string()
}

/// A stored vector is not a constant: assert the body carries real data of the
/// collection's dimension, and that it is not the old `[0.1; 512]` filler.
fn assert_real_vector(body: &Value, expected_id: &str) {
    assert_eq!(body["id"].as_str(), Some(expected_id));
    let data = body["vector"].as_array().expect("vector array");
    assert_eq!(data.len(), 512, "dimension must match the collection");
    let values: Vec<f64> = data.iter().map(|v| v.as_f64().expect("float")).collect();
    assert!(
        values.iter().any(|v| (*v - 0.1).abs() > 1e-9),
        "every component is 0.1 — this is the old fabricated body, not stored data"
    );
    assert!(
        values.iter().any(|v| *v != 0.0),
        "an all-zero embedding means nothing was retrieved"
    );
}

#[tokio::test]
async fn get_vector_by_path_returns_the_stored_vector() {
    let app = TestApp::new().await;
    let collection = "get_vector_path";
    let id = seed_one(&app, collection).await;

    let (status, body) = app
        .get(&format!("/collections/{collection}/vectors/{id}"))
        .await;
    assert!(status.is_success(), "get status {status}: {body}");
    assert_real_vector(&body, &id);
    assert_eq!(body["collection"].as_str(), Some(collection));
    assert_eq!(
        body["payload"]["tag"].as_str(),
        Some("probe"),
        "the stored payload must come back: {body}"
    );
}

#[tokio::test]
async fn get_vector_by_path_reports_an_unknown_id_as_missing() {
    let app = TestApp::new().await;
    let collection = "get_vector_path_missing";
    seed_one(&app, collection).await;

    let (status, body) = app
        .get(&format!("/collections/{collection}/vectors/no_such_vector"))
        .await;
    assert_eq!(
        status,
        StatusCode::NOT_FOUND,
        "an absent vector must not be answered with a body: {body}"
    );
}

#[tokio::test]
async fn post_vector_returns_the_stored_vector() {
    let app = TestApp::new().await;
    let collection = "get_vector_body";
    let id = seed_one(&app, collection).await;

    // The registry/MCP field name.
    let (status, body) = app
        .post_json(
            "/vector",
            json!({"collection": collection, "vector_id": id}),
        )
        .await;
    assert!(status.is_success(), "post status {status}: {body}");
    assert_real_vector(&body, &id);

    // `id` is accepted too, since that is what every reply calls the field.
    let (status, body) = app
        .post_json("/vector", json!({"collection": collection, "id": id}))
        .await;
    assert!(
        status.is_success(),
        "post with 'id' status {status}: {body}"
    );
    assert_real_vector(&body, &id);
}

#[tokio::test]
async fn post_vector_validates_its_body() {
    let app = TestApp::new().await;
    let collection = "get_vector_body_validation";
    seed_one(&app, collection).await;

    let (status, _) = app.post_json("/vector", json!({"vector_id": "x"})).await;
    assert_eq!(
        status,
        StatusCode::BAD_REQUEST,
        "a missing collection must be a validation error"
    );

    let (status, _) = app
        .post_json("/vector", json!({"collection": collection}))
        .await;
    assert_eq!(
        status,
        StatusCode::BAD_REQUEST,
        "a missing vector_id must be a validation error"
    );

    let (status, _) = app
        .post_json(
            "/vector",
            json!({"collection": collection, "vector_id": "no_such_vector"}),
        )
        .await;
    assert_eq!(
        status,
        StatusCode::NOT_FOUND,
        "an absent vector must be reported as missing"
    );
}
