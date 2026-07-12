//! In-process migration of
//! `crates/vectorizer/tests/api/rest/embedding_provider_real.rs`
//! (phase39 §1.2) onto the shared harness in `tests/common/mod.rs`.
//!
//! Same assertions as the live-server suite, but dispatched through the
//! real production router via `tower::ServiceExt::oneshot` instead of
//! `reqwest` against `127.0.0.1:15002` — no `#[ignore]`, runs in CI.
//!
//! Covers phase33 / issue #306 — the `embedding_provider` and `model`
//! contract on `POST /collections` and `POST /embed`, plus the `GET
//! /stats` discovery surface. Before phase33 the server silently coerced
//! every collection to BM25-512 regardless of the requested
//! `embedding_provider`, and `/embed` ignored the `model` param.
//!
//! Delta vs the live suite: [`common::TestApp`] only registers a single
//! `bm25` provider (see `tests/common/mod.rs`), so `stats.providers` /
//! `details.available` only ever list `bm25` here — the live suite runs
//! against a fully-configured server that may register additional
//! providers (fastembed, etc.). The assertions below only require the
//! arrays to be non-empty and contain `bm25`, which both environments
//! satisfy.

#![allow(clippy::unwrap_used, clippy::expect_used)]
#![allow(clippy::uninlined_format_args)]

mod common;

use axum::http::StatusCode;
use common::TestApp;
use serde_json::json;

/// Phase33 §4 (GET /stats discovery surface): every running server MUST
/// expose its registered providers and the currently-selected default,
/// so callers can decide what to post before tripping a 400 on
/// `embedding_provider`.
#[tokio::test]
async fn stats_advertises_providers_block() {
    let app = TestApp::new().await;
    let (status, body) = app.get("/stats").await;
    assert_eq!(status, StatusCode::OK, "GET /stats status: {body}");

    let providers = body
        .get("providers")
        .and_then(|p| p.as_array())
        .expect("stats.providers must be an array");
    assert!(
        !providers.is_empty(),
        "stats.providers is empty — register_all_providers did not run: {body}"
    );
    for p in providers {
        assert!(p.get("name").and_then(|v| v.as_str()).is_some());
        assert!(p.get("dimension").and_then(|v| v.as_u64()).is_some());
        assert!(p.get("default").and_then(|v| v.as_bool()).is_some());
    }

    let default = body
        .get("default_provider")
        .and_then(|v| v.as_str())
        .expect("default_provider must be a string");
    assert!(!default.is_empty(), "default_provider is empty");
}

/// Phase33 §2: `bm25` is always-on so `POST /collections
/// {embedding_provider: "bm25"}` succeeds regardless of which other
/// providers are compiled in, and the response reads back with the
/// same provider it was created with.
#[tokio::test]
async fn create_collection_honours_bm25_provider() {
    let app = TestApp::new().await;
    let name = "phase33_bm25_in_process";

    let (status, resp) = app
        .post_json(
            "/collections",
            json!({
                "name": name,
                "dimension": 512,
                "metric": "cosine",
                "embedding_provider": "bm25",
            }),
        )
        .await;
    assert!(status.is_success(), "create status {status}: {resp}");

    let (status, info) = app.get(&format!("/collections/{name}")).await;
    assert_eq!(status, StatusCode::OK, "get collection status: {info}");

    let provider = info
        .get("embedding_provider")
        .and_then(|v| v.as_str())
        .expect("embedding_provider field missing");
    assert_eq!(
        provider, "bm25",
        "round-trip lost embedding_provider (phase33 regression): {info}"
    );
}

/// Phase33 §2.2: unknown `embedding_provider` returns `400
/// unsupported_provider` with `requested` + `available` in the response
/// details. This is the SDK contract that downstream (Rust / TypeScript
/// / Python / Go / C#) clients match on.
#[tokio::test]
async fn create_collection_rejects_unknown_provider() {
    let app = TestApp::new().await;
    let name = "phase33_bogus_in_process";

    let (status, body) = app
        .post_json(
            "/collections",
            json!({
                "name": name,
                "dimension": 768,
                "metric": "cosine",
                "embedding_provider": "definitely-not-a-real-provider",
            }),
        )
        .await;

    assert_eq!(
        status.as_u16(),
        400,
        "expected 400 unsupported_provider, got {status}: {body}"
    );
    assert_eq!(
        body.get("error_type").and_then(|v| v.as_str()),
        Some("unsupported_provider"),
        "error_type drift — SDKs match on this string: {body}",
    );
    let details = body
        .get("details")
        .expect("error body must carry structured details");
    assert_eq!(
        details.get("requested").and_then(|v| v.as_str()),
        Some("definitely-not-a-real-provider"),
    );
    assert!(
        details
            .get("available")
            .and_then(|v| v.as_array())
            .map(|a| !a.is_empty())
            .unwrap_or(false),
        "details.available must list at least bm25: {body}",
    );

    // Collection MUST NOT exist (the 400 has to be pre-create).
    let (probe_status, _) = app.get(&format!("/collections/{name}")).await;
    assert_eq!(
        probe_status.as_u16(),
        404,
        "rejected collection somehow got created (phase33 regression)",
    );
}

/// Phase33 §3: `/embed` honours `model`. Unknown model → 400
/// unsupported_model with the same `requested`/`available` shape as the
/// create-collection branch.
#[tokio::test]
async fn embed_rejects_unknown_model() {
    let app = TestApp::new().await;
    let (status, body) = app
        .post_json(
            "/embed",
            json!({
                "text": "phase33 embed contract probe",
                "model": "definitely-not-a-real-model",
            }),
        )
        .await;

    assert_eq!(
        status.as_u16(),
        400,
        "expected 400 unsupported_model, got {status}: {body}"
    );
    assert_eq!(
        body.get("error_type").and_then(|v| v.as_str()),
        Some("unsupported_model"),
        "error_type drift — SDKs match on this string: {body}",
    );
    let details = body
        .get("details")
        .expect("error body must carry structured details");
    assert_eq!(
        details.get("requested").and_then(|v| v.as_str()),
        Some("definitely-not-a-real-model"),
    );
    assert!(
        details
            .get("available")
            .and_then(|v| v.as_array())
            .map(|a| !a.is_empty())
            .unwrap_or(false),
        "details.available must list at least bm25: {body}",
    );
}

/// Phase33 §3.2: `/embed` without an explicit `model` uses the server's
/// default provider and the response echoes the resolved `model` so
/// callers can confirm which provider produced the vector.
#[tokio::test]
async fn embed_without_model_echoes_resolved_model() {
    let app = TestApp::new().await;
    let (status, resp) = app
        .post_json("/embed", json!({ "text": "phase33 default-model probe" }))
        .await;
    assert_eq!(status, StatusCode::OK, "POST /embed status: {resp}");

    let model = resp
        .get("model")
        .and_then(|v| v.as_str())
        .expect("/embed response must carry the resolved `model` field (phase33)");
    assert!(!model.is_empty(), "resolved model is empty: {resp}");

    let dim = resp
        .get("dimension")
        .and_then(|v| v.as_u64())
        .expect("/embed response missing dimension");
    assert!(dim > 0);
}
