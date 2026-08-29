//! Collections that store pre-computed vectors
//! (phase6_raw-vector-collections).
//!
//! `POST /insert_vectors` exists for callers who bring their own embeddings,
//! and until this task there was no way to create a collection to put them
//! in: `create_collection` resolves an embedding provider for every
//! collection and rejects any dimension that disagrees with it, while a stock
//! server registers only BM25 at 512. Every real embedding width — 384, 768,
//! 1536 — was refused.
//!
//! `embedding_provider: "none"` is the opt-out. The tests below pin both
//! halves: the widths that were impossible now work, and text operations on
//! such a collection *fail loudly* rather than falling back to BM25. That
//! second half is the point. Embedding text with a provider the stored
//! vectors did not come from yields a collection that searches badly with
//! nothing reporting why — the silent coercion phase33 (issue #306) removed.

#![allow(clippy::unwrap_used, clippy::expect_used)]

mod common;

use axum::http::StatusCode;
use common::TestApp;
use serde_json::json;

/// Widths of models people actually use. None is 512, which is the whole
/// problem: BM25's width is the only one the native endpoint used to accept.
const REAL_MODEL_WIDTHS: [usize; 4] = [100, 384, 768, 1536];

async fn create_raw(
    app: &TestApp,
    name: &str,
    dimension: usize,
) -> (StatusCode, serde_json::Value) {
    app.post_json(
        "/collections",
        json!({
            "name": name,
            "dimension": dimension,
            "metric": "cosine",
            "embedding_provider": "none",
        }),
    )
    .await
}

#[tokio::test]
async fn raw_vector_collections_accept_widths_no_provider_has() {
    let app = TestApp::new().await;

    for (i, width) in REAL_MODEL_WIDTHS.into_iter().enumerate() {
        let name = format!("raw_w{i}");
        let (status, body) = create_raw(&app, &name, width).await;
        assert!(
            status.is_success(),
            "dimension {width} must be allowed for a raw-vector collection, \
             got {status}: {body}"
        );
    }
}

#[tokio::test]
async fn the_same_widths_are_still_refused_without_the_opt_out() {
    let app = TestApp::new().await;

    // The phase33 guard has to stay intact for ordinary collections: a caller
    // who wants BM25 and mistypes the dimension must still be told.
    let (status, body) = app
        .post_json(
            "/collections",
            json!({"name": "still_guarded", "dimension": 384, "metric": "cosine"}),
        )
        .await;

    assert_eq!(status, StatusCode::BAD_REQUEST, "body: {body}");
    assert_eq!(body["error_type"], "provider_dimension_mismatch");
}

#[tokio::test]
async fn an_unknown_provider_is_still_rejected() {
    let app = TestApp::new().await;

    // The sentinel must not turn the provider field into a free-text field.
    let (status, body) = app
        .post_json(
            "/collections",
            json!({
                "name": "bogus_provider",
                "dimension": 384,
                "metric": "cosine",
                "embedding_provider": "definitely-not-registered",
            }),
        )
        .await;

    assert_eq!(status, StatusCode::BAD_REQUEST, "body: {body}");
    assert_eq!(body["error_type"], "unsupported_provider");
}

#[tokio::test]
async fn pre_computed_vectors_round_trip_through_a_raw_collection() {
    let app = TestApp::new().await;
    let (status, body) = create_raw(&app, "raw_roundtrip", 4).await;
    assert!(status.is_success(), "create: {status} {body}");

    let (status, body) = app
        .post_json(
            "/insert_vectors",
            json!({
                "collection": "raw_roundtrip",
                "vectors": [
                    {"id": "0", "embedding": [1.0, 0.0, 0.0, 0.0], "payload": {}},
                    {"id": "1", "embedding": [0.0, 1.0, 0.0, 0.0], "payload": {}},
                ],
            }),
        )
        .await;
    assert!(status.is_success(), "insert: {status} {body}");
    assert_eq!(body["failed"], 0, "body: {body}");
    assert_eq!(body["inserted"], 2, "body: {body}");

    // Vector search is the operation that *does* work here, and the caller's
    // own ids must come back — anything that mangles them makes the results
    // unusable for the workload this collection exists to serve.
    let (status, body) = app
        .post_json(
            "/collections/raw_roundtrip/search",
            json!({"vector": [1.0, 0.0, 0.0, 0.0], "limit": 2}),
        )
        .await;
    assert!(status.is_success(), "search: {status} {body}");
    assert_eq!(
        body["results"][0]["id"], "0",
        "the query vector's own id must rank first: {body}"
    );
}

#[tokio::test]
async fn text_insert_is_refused_rather_than_embedded_with_the_default() {
    let app = TestApp::new().await;
    let (status, _) = create_raw(&app, "raw_no_text", 384).await;
    assert!(status.is_success());

    // Single insert: the refusal is the whole response.
    //
    // Falling back to BM25 here would embed the text with a provider the
    // stored vectors did not come from, leaving the collection searching a
    // space its own contents do not live in.
    let (status, body) = app
        .post_json(
            "/insert",
            json!({"collection": "raw_no_text", "text": "hello"}),
        )
        .await;
    assert_eq!(status, StatusCode::BAD_REQUEST, "body: {body}");
    assert_eq!(body["error_type"], "collection_has_no_embedding_provider");
    assert!(
        body["message"]
            .as_str()
            .unwrap()
            .contains("/insert_vectors"),
        "the error must name the endpoint that works: {body}"
    );

    // Batch inserts report per-row and answer 200 even when every row failed
    // — their documented contract, since one bad row must not discard the
    // rest. The refusal therefore lands in `results[].error_type`, not in the
    // status. Asserting 400 here would be asserting against the batch
    // contract rather than against this fix.
    for route in ["/batch_insert", "/insert_texts"] {
        let (status, body) = app
            .post_json(
                route,
                json!({"collection": "raw_no_text", "texts": [{"text": "hello"}]}),
            )
            .await;

        assert!(status.is_success(), "{route} body: {body}");
        assert_eq!(body["inserted"], 0, "{route} body: {body}");
        assert_eq!(body["failed"], 1, "{route} body: {body}");
        assert_eq!(
            body["results"][0]["error_type"], "collection_has_no_embedding_provider",
            "{route} body: {body}"
        );
    }
}

#[tokio::test]
async fn text_search_is_refused_on_a_raw_collection() {
    let app = TestApp::new().await;
    let (status, _) = create_raw(&app, "raw_no_text_search", 384).await;
    assert!(status.is_success());

    let (status, body) = app
        .post_json(
            "/collections/raw_no_text_search/search/text",
            json!({"query": "hello", "limit": 5}),
        )
        .await;

    assert_eq!(status, StatusCode::BAD_REQUEST, "body: {body}");
    assert_eq!(body["error_type"], "collection_has_no_embedding_provider");
}

#[tokio::test]
async fn hybrid_search_is_refused_on_a_raw_collection() {
    let app = TestApp::new().await;
    let (status, _) = create_raw(&app, "raw_no_hybrid", 384).await;
    assert!(status.is_success());

    let (status, body) = app
        .post_json(
            "/collections/raw_no_hybrid/hybrid_search",
            json!({"query": "hello", "limit": 5}),
        )
        .await;

    assert_eq!(status, StatusCode::BAD_REQUEST, "body: {body}");
    assert_eq!(body["error_type"], "collection_has_no_embedding_provider");
}

#[tokio::test]
async fn the_sentinel_is_discoverable_in_the_stats_inventory() {
    let app = TestApp::new().await;
    let (status, body) = app.get("/stats").await;
    assert!(status.is_success(), "{status}: {body}");

    let providers = body["providers"]
        .as_array()
        .unwrap_or_else(|| panic!("GET /stats must list providers: {body}"));

    // A client is told to pick `embedding_provider` from this list. If the
    // sentinel is not in it, the pre-vectorized workflow is a feature nobody
    // can find — accepted but unadvertised.
    let sentinel = providers
        .iter()
        .find(|p| p["name"] == "none")
        .unwrap_or_else(|| panic!("the raw-vector sentinel must be listed: {body}"));

    assert!(
        sentinel["dimension"].is_null(),
        "any width is allowed, so there is no fixed dimension to report — \
         and 0 already means 'could not read it': {sentinel}"
    );
    assert_eq!(sentinel["supports_text"], false, "{sentinel}");
    assert_eq!(sentinel["default"], false, "{sentinel}");

    // The flag has to be readable on every entry, or a client cannot branch
    // on it and has to special-case the name instead.
    let bm25 = providers
        .iter()
        .find(|p| p["name"] == "bm25")
        .unwrap_or_else(|| panic!("bm25 is always registered: {body}"));
    assert_eq!(bm25["supports_text"], true, "{bm25}");
}

#[tokio::test]
async fn a_raw_collection_reports_no_provider_rather_than_bm25() {
    let app = TestApp::new().await;
    let (status, _) = create_raw(&app, "raw_reported", 384).await;
    assert!(status.is_success());

    let (status, body) = app
        .post_json(
            "/collections",
            json!({"name": "ordinary", "dimension": 512, "metric": "cosine"}),
        )
        .await;
    assert!(status.is_success(), "{status}: {body}");

    // Detail route.
    let (status, body) = app.get("/collections/raw_reported").await;
    assert!(status.is_success(), "{status}: {body}");
    assert!(
        body["embedding_provider"].is_null(),
        "a collection with no provider must not be reported under one: {body}"
    );

    // Listing route. Both used to report the *server default* for every
    // collection regardless of config, so a raw collection read as `bm25` —
    // the exact confusion phase33 (#306) discovery was added to remove.
    let (status, body) = app.get("/collections").await;
    assert!(status.is_success(), "{status}: {body}");
    let collections = body["collections"]
        .as_array()
        .unwrap_or_else(|| panic!("listing must carry collections: {body}"));

    let raw = collections
        .iter()
        .find(|c| c["name"] == "raw_reported")
        .unwrap_or_else(|| panic!("created collection must be listed: {body}"));
    assert!(raw["embedding_provider"].is_null(), "{raw}");

    let ordinary = collections
        .iter()
        .find(|c| c["name"] == "ordinary")
        .unwrap_or_else(|| panic!("created collection must be listed: {body}"));
    assert_eq!(
        ordinary["embedding_provider"], "bm25",
        "an ordinary collection still reports its own provider: {ordinary}"
    );
}

#[tokio::test]
async fn refusal_is_a_bad_request_not_a_not_found() {
    let app = TestApp::new().await;
    let (status, _) = create_raw(&app, "raw_exists", 384).await;
    assert!(status.is_success());

    let (status, body) = app
        .post_json(
            "/collections/raw_exists/search/text",
            json!({"query": "hello"}),
        )
        .await;

    // The collection is right there. A 404 would send the caller looking for
    // something that exists, which is a worse diagnostic than the original
    // silent coercion.
    assert_eq!(status, StatusCode::BAD_REQUEST, "body: {body}");
    assert_ne!(status, StatusCode::NOT_FOUND);
}
