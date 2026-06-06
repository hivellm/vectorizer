//! Live integration tests for phase33 / issue #306 — the
//! `embedding_provider` and `model` contract on `POST /collections`
//! and `POST /embed`, plus the `GET /stats` discovery surface.
//!
//! Before phase33 the 3.3.0 server silently coerced every collection
//! to BM25-512 regardless of the requested `embedding_provider`,
//! and `/embed` ignored the `model` param. These tests pin the new
//! behavior:
//!
//! - Unknown provider on create-collection → `400 unsupported_provider`
//!   with `requested` + `available` in the response body.
//! - Unknown model on `/embed` → `400 unsupported_model` with the
//!   same shape.
//! - `GET /stats` carries `providers[]` and `default_provider` so
//!   callers can discover what the deployment supports.
//! - A collection that asks for `bm25` (always-on, registered by
//!   `register_all_providers`) round-trips with the same provider
//!   it was created with.
//!
//! Require a running server at `127.0.0.1:15002`. Run with:
//! `cargo test --test all_tests api::rest::embedding_provider_real -- --ignored`

#![allow(clippy::unwrap_used, clippy::expect_used)]
#![allow(clippy::uninlined_format_args)]

use std::time::Duration;

use serde_json::{Value, json};

const VECTORIZER_API_URL: &str = "http://127.0.0.1:15002";

fn client() -> reqwest::blocking::Client {
    reqwest::blocking::Client::builder()
        .timeout(Duration::from_secs(30))
        .build()
        .expect("build reqwest client")
}

fn cleanup(http: &reqwest::blocking::Client, name: &str) {
    let _ = http
        .delete(format!("{}/collections/{}", VECTORIZER_API_URL, name))
        .send();
}

/// Phase33 §4 (GET /stats discovery surface): every running server
/// MUST expose its registered providers and the currently-selected
/// default, so callers can decide what to post before tripping a
/// 400 on `embedding_provider`.
#[test]
#[ignore = "requires live server at 127.0.0.1:15002"]
fn stats_advertises_providers_block() {
    let http = client();
    let body: Value = http
        .get(format!("{}/stats", VECTORIZER_API_URL))
        .send()
        .expect("GET /stats")
        .json()
        .expect("decode /stats");

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
#[test]
#[ignore = "requires live server at 127.0.0.1:15002"]
fn create_collection_honours_bm25_provider() {
    let http = client();
    let name = format!("phase33_bm25_{}", std::process::id());
    cleanup(&http, &name);

    let resp = http
        .post(format!("{}/collections", VECTORIZER_API_URL))
        .json(&json!({
            "name": &name,
            "dimension": 512,
            "metric": "cosine",
            "embedding_provider": "bm25",
        }))
        .send()
        .expect("create collection");
    assert!(
        resp.status().is_success(),
        "create status {} (body: {:?})",
        resp.status(),
        resp.text().ok(),
    );

    let info: Value = http
        .get(format!("{}/collections/{}", VECTORIZER_API_URL, name))
        .send()
        .expect("read collection")
        .json()
        .expect("decode collection");

    let provider = info
        .get("embedding_provider")
        .and_then(|v| v.as_str())
        .expect("embedding_provider field missing");
    assert_eq!(
        provider, "bm25",
        "round-trip lost embedding_provider (phase33 regression): {info}"
    );

    cleanup(&http, &name);
}

/// Phase33 §2.2: unknown `embedding_provider` returns `400
/// unsupported_provider` with `requested` + `available` in the
/// response details. This is the SDK contract that downstream
/// (Rust / TypeScript / Python / Go / C#) clients match on.
#[test]
#[ignore = "requires live server at 127.0.0.1:15002"]
fn create_collection_rejects_unknown_provider() {
    let http = client();
    let name = format!("phase33_bogus_{}", std::process::id());
    cleanup(&http, &name);

    let resp = http
        .post(format!("{}/collections", VECTORIZER_API_URL))
        .json(&json!({
            "name": &name,
            "dimension": 768,
            "metric": "cosine",
            "embedding_provider": "definitely-not-a-real-provider",
        }))
        .send()
        .expect("create collection");

    assert_eq!(
        resp.status().as_u16(),
        400,
        "expected 400 unsupported_provider, got {}",
        resp.status()
    );

    let body: Value = resp.json().expect("decode error body");
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
    let probe = http
        .get(format!("{}/collections/{}", VECTORIZER_API_URL, name))
        .send()
        .expect("probe collection");
    assert_eq!(
        probe.status().as_u16(),
        404,
        "rejected collection somehow got created (phase33 regression)",
    );
}

/// Phase33 §3: `/embed` honours `model`. Unknown model → 400
/// unsupported_model with the same `requested`/`available` shape as
/// the create-collection branch.
#[test]
#[ignore = "requires live server at 127.0.0.1:15002"]
fn embed_rejects_unknown_model() {
    let http = client();
    let resp = http
        .post(format!("{}/embed", VECTORIZER_API_URL))
        .json(&json!({
            "text": "phase33 embed contract probe",
            "model": "definitely-not-a-real-model",
        }))
        .send()
        .expect("POST /embed");

    assert_eq!(
        resp.status().as_u16(),
        400,
        "expected 400 unsupported_model, got {}",
        resp.status()
    );

    let body: Value = resp.json().expect("decode error body");
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

/// Phase33 §3.2: `/embed` without an explicit `model` uses the
/// server's default provider and the response echoes the resolved
/// `model` so callers can confirm which provider produced the vector.
#[test]
#[ignore = "requires live server at 127.0.0.1:15002"]
fn embed_without_model_echoes_resolved_model() {
    let http = client();
    let resp: Value = http
        .post(format!("{}/embed", VECTORIZER_API_URL))
        .json(&json!({ "text": "phase33 default-model probe" }))
        .send()
        .expect("POST /embed")
        .json()
        .expect("decode /embed");

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
