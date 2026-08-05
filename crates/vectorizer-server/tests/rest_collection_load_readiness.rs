//! A partial collection list says it is partial
//! (phase1_collections-list-hides-lazy-load-progress, issue #391).
//!
//! The server fills its store from a background task, one collection at a
//! time. `GET /collections` used to answer from whatever had landed so far
//! with no hint that more was coming — and `total_collections` was the length
//! of that partial list, so it *agreed* with it. On a 181-collection store a
//! client 20s after boot got 11 collections and a total of 11: a partial
//! answer wearing a complete answer's clothes. That reads as catastrophic
//! data loss, and during a 3.5→3.6 upgrade it was acted on as such.
//!
//! These tests drive the load-progress handle directly rather than racing a
//! real background load, so the warm-up window — normally a few seconds on a
//! large store and impossible to hit reliably — is observable on demand.

#![allow(clippy::unwrap_used, clippy::expect_used)]

mod common;

use axum::http::{StatusCode, header};
use common::TestApp;
use serde_json::json;

/// Seed `n` collections through the real router so the listing has content.
///
/// 512 dimensions because the harness's default provider is BM25, and
/// `create_collection` rejects a dimension that disagrees with the provider
/// (`provider_dimension_mismatch`, issue #306).
async fn seed(app: &TestApp, n: usize) {
    for i in 0..n {
        let (status, body) = app
            .post_json(
                "/collections",
                json!({"name": format!("warmup_c{i}"), "dimension": 512, "metric": "cosine"}),
            )
            .await;
        assert!(status.is_success(), "seed {i} failed {status}: {body}");
    }
}

#[tokio::test]
async fn collections_list_announces_that_it_is_partial() {
    let (app, progress) = TestApp::new_with_load_progress().await;

    // A store mid-load: the catalog holds 5, two have landed.
    seed(&app, 2).await;
    progress.begin(5);
    progress.record_loaded();
    progress.record_loaded();

    let (status, body) = app.get("/collections").await;
    assert_eq!(status, StatusCode::OK, "body: {body}");

    assert_eq!(
        body["loading"], true,
        "a listing taken mid-load must admit it is provisional: {body}"
    );
    assert_eq!(body["loaded_collections"], 2);
    assert_eq!(
        body["expected_collections"], 5,
        "the denominator is what turns a short list into a readable one"
    );

    // The compatibility half of the contract: `total_collections` still counts
    // the items in *this* response, because published SDKs read it. The new
    // fields carry the load story instead of redefining an existing one.
    assert_eq!(
        body["total_collections"],
        body["collections"].as_array().unwrap().len(),
        "total_collections must keep meaning 'items in this response': {body}"
    );
}

#[tokio::test]
async fn collections_list_stops_claiming_partial_once_loaded() {
    let (app, progress) = TestApp::new_with_load_progress().await;

    seed(&app, 2).await;
    progress.begin(2);
    progress.record_loaded();
    progress.record_loaded();
    progress.finish();

    let (status, body) = app.get("/collections").await;
    assert_eq!(status, StatusCode::OK, "body: {body}");
    assert_eq!(
        body["loading"], false,
        "a complete catalog must not keep flagging itself as loading: {body}"
    );
    assert_eq!(body["loaded_collections"], 2);
    assert_eq!(body["expected_collections"], 2);
}

#[tokio::test]
async fn ready_gates_traffic_until_the_catalog_is_in() {
    let (app, progress) = TestApp::new_with_load_progress().await;

    progress.begin(5);
    progress.record_loaded();

    let (status, headers, body) = app.get_with_headers("/ready").await;
    assert_eq!(
        status,
        StatusCode::SERVICE_UNAVAILABLE,
        "readiness must fail while the catalog loads: {body}"
    );
    assert_eq!(
        headers
            .get(header::RETRY_AFTER)
            .map(|v| v.to_str().unwrap()),
        Some("5"),
        "without Retry-After a probe loop hammers a server that is busy loading"
    );
    assert_eq!(body["ready"], false);
    assert_eq!(body["loaded_collections"], 1);
    assert_eq!(body["expected_collections"], 5);

    progress.finish();

    let (status, _headers, body) = app.get_with_headers("/ready").await;
    assert_eq!(status, StatusCode::OK, "body: {body}");
    assert_eq!(body["ready"], true);
}

#[tokio::test]
async fn ready_stays_unavailable_after_a_failed_load() {
    let (app, progress) = TestApp::new_with_load_progress().await;

    progress.begin(5);
    progress.fail("vectorizer.vecdb is corrupt");

    let (status, _headers, body) = app.get_with_headers("/ready").await;
    // A failed load is settled — nothing more is coming — but the catalog was
    // never delivered, so the node is not ready. Reporting 200 here would send
    // traffic to a server that cannot serve it.
    assert_eq!(status, StatusCode::SERVICE_UNAVAILABLE, "body: {body}");
    assert_eq!(body["ready"], false);
    assert_eq!(
        body["loading"], false,
        "a failed load must stop reporting as in-progress: {body}"
    );
    assert!(
        body["load_state"].to_string().contains("corrupt"),
        "the reason is what separates \"wait\" from \"investigate\": {body}"
    );
}

#[tokio::test]
async fn health_keeps_answering_ok_during_warm_up() {
    let (app, progress) = TestApp::new_with_load_progress().await;

    progress.begin(181);

    let (status, body) = app.get("/health").await;
    // Load-bearing: the Dockerfile HEALTHCHECK probes /health with
    // `--start-period=40s --interval=30s --retries=3`. Failing it during the
    // load would mark a container unhealthy after roughly two minutes — and
    // the stores slow enough to hit that are exactly the large ones this
    // issue is about, so the orchestrator would restart them in a loop.
    assert_eq!(
        status,
        StatusCode::OK,
        "liveness must survive warm-up: {body}"
    );
    assert_eq!(body["status"], "healthy");

    // Readiness still travels with it, for anyone reading one endpoint.
    assert_eq!(body["readiness"]["ready"], false);
    assert_eq!(body["readiness"]["expected_collections"], 181);
}
