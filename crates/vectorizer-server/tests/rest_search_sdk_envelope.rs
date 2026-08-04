//! The search wire fields the published SDKs validate on
//! (phase2_update-gui-to-thunder-sdk).
//!
//! The TypeScript SDK does not merely deserialise a search response, it
//! *validates* it: `validateSearchResponse` requires a numeric `total`, and
//! `validateSearchResult` requires a non-empty `data` array on every hit. The
//! server answered `total_results` and `vector`, so a **successful** search
//! threw inside the client — first `Search response total must be a
//! non-negative number`, then `Search result data must be a non-empty array`.
//! Reproduced with `@hivehub/vectorizer-sdk` 3.6.0 against a running 3.6.0
//! server while porting the GUI onto the Thunder SDK.
//!
//! The handlers now emit both spellings. These tests pin that, because the
//! failure mode is invisible from the server's side: the response looks
//! perfectly good in `curl`, every REST test passes, and only a client that
//! validates rejects it. Dropping either field again would break every
//! published SDK while this repo's own suite stayed green.

#![allow(clippy::unwrap_used, clippy::expect_used)]
#![allow(clippy::uninlined_format_args)]

mod common;

use common::TestApp;
use serde_json::{Value, json};

async fn seed(app: &TestApp, name: &str) {
    let _ = app.delete(&format!("/collections/{name}")).await;
    let (status, resp) = app
        .post_json(
            "/collections",
            json!({"name": name, "dimension": 512, "metric": "cosine"}),
        )
        .await;
    assert!(status.is_success(), "create status {status}: {resp}");

    let (status, resp) = app
        .post_json(
            "/batch_insert",
            json!({
                "collection": name,
                "texts": [
                    {"text": "alpha search envelope probe", "metadata": {"tag": "a"}},
                    {"text": "beta search envelope probe",  "metadata": {"tag": "b"}},
                ],
            }),
        )
        .await;
    assert!(status.is_success(), "batch_insert status {status}: {resp}");
}

/// Assert both spellings of the count, and both spellings of the embedding on
/// every hit — the exact pair of checks the SDK validators perform.
fn assert_sdk_validatable(resp: &Value, route: &str) {
    let total = resp["total"].as_u64();
    assert!(
        total.is_some(),
        "{route}: `total` missing — the SDK rejects the envelope with \
         \"Search response total must be a non-negative number\". Response: {resp}"
    );
    assert_eq!(
        total,
        resp["total_results"].as_u64(),
        "{route}: `total` and `total_results` must agree"
    );

    let hits = resp["results"].as_array().expect("results array");
    for hit in hits {
        let data = hit["data"].as_array();
        assert!(
            data.is_some_and(|d| !d.is_empty()),
            "{route}: hit `data` missing or empty — the SDK rejects it with \
             \"Search result data must be a non-empty array\". Hit: {hit}"
        );
        assert_eq!(
            data,
            hit["vector"].as_array(),
            "{route}: `data` must mirror `vector`"
        );
        assert!(
            hit["id"].is_string(),
            "{route}: hit needs a string id: {hit}"
        );
        assert!(
            hit["score"].is_number(),
            "{route}: hit needs a score: {hit}"
        );
    }
}

#[tokio::test]
async fn text_search_envelope_is_sdk_validatable() {
    let app = TestApp::new().await;
    let name = "sdk_envelope_text";
    seed(&app, name).await;

    let (status, resp) = app
        .post_json(
            &format!("/collections/{name}/search/text"),
            json!({"query": "alpha", "limit": 5}),
        )
        .await;
    assert!(status.is_success(), "search/text status {status}: {resp}");
    assert!(
        !resp["results"]
            .as_array()
            .expect("results array")
            .is_empty(),
        "search/text must return hits for a seeded collection: {resp}"
    );
    assert_sdk_validatable(&resp, "search/text");
}

#[tokio::test]
async fn hybrid_search_envelope_is_sdk_validatable() {
    let app = TestApp::new().await;
    let name = "sdk_envelope_hybrid";
    seed(&app, name).await;

    let (status, resp) = app
        .post_json(
            &format!("/collections/{name}/hybrid_search"),
            json!({"query": "alpha", "limit": 5}),
        )
        .await;
    assert!(status.is_success(), "hybrid_search status {status}: {resp}");
    assert!(
        !resp["results"]
            .as_array()
            .expect("results array")
            .is_empty(),
        "hybrid_search must return hits for a seeded collection: {resp}"
    );
    assert_sdk_validatable(&resp, "hybrid_search");
}

#[tokio::test]
async fn file_search_envelope_is_sdk_validatable() {
    let app = TestApp::new().await;
    let name = "sdk_envelope_file";
    seed(&app, name).await;

    // `search/file` is still a stub: it validates the collection and answers an
    // empty result set (`// For now, return empty results` in the handler). The
    // envelope has to be SDK-validatable anyway — a client that throws on a
    // missing `total` cannot tell "no matches" from "request failed". When the
    // handler grows a real implementation, add the hits assertion the other two
    // tests carry.
    let (status, resp) = app
        .post_json(
            &format!("/collections/{name}/search/file"),
            json!({"file_path": "envelope_probe.md", "limit": 5}),
        )
        .await;
    assert!(status.is_success(), "search/file status {status}: {resp}");
    assert!(
        resp["results"]
            .as_array()
            .expect("results array")
            .is_empty(),
        "search/file is a stub today; update this test when it returns hits: {resp}"
    );
    assert_sdk_validatable(&resp, "search/file");
}
