//! Live integration tests for the Qdrant-compat REST surface
//! against the minimal request shape that real Qdrant clients send.
//!
//! Covers `phase8_qdrant-compat-minimal-request-shape` (probe 3.6):
//! the handler at `crates/vectorizer-server/src/server/qdrant/handlers.rs`
//! used to require a wrapped `{config: {...}}` request with 9 mandatory
//! sub-fields, so any real qdrant-client (-python / -js / ...) failed
//! at collection creation with a 422 about a missing `config` field.
//! The contract is now: only `vectors` is required, everything else
//! defaults server-side, and both the wrapped + flat request shapes
//! parse.
//!
//! Require a running server at `127.0.0.1:15002`. Run with:
//! `cargo test --test all_tests api::rest::qdrant_compat_minimal_real -- --ignored`

#![allow(clippy::unwrap_used, clippy::expect_used)]
#![allow(clippy::uninlined_format_args)]

use std::time::Duration;

use serde_json::json;

const VECTORIZER_API_URL: &str = "http://127.0.0.1:15002";

fn client() -> reqwest::blocking::Client {
    reqwest::blocking::Client::builder()
        .timeout(Duration::from_secs(30))
        .build()
        .expect("build reqwest client")
}

fn delete_collection(http: &reqwest::blocking::Client, name: &str) {
    let _ = http
        .delete(format!(
            "{}/qdrant/collections/{}",
            VECTORIZER_API_URL, name
        ))
        .send();
}

#[test]
#[ignore = "requires running vectorizer server on 127.0.0.1:15002"]
fn flat_minimal_shape_creates_collection() {
    let http = client();
    let name = "qdrant_compat_minimal_flat";
    delete_collection(&http, name);

    // This is exactly the payload qdrant-client-python sends when a
    // caller does `client.create_collection(name, vectors_config=
    // VectorParams(size=4, distance=Distance.COSINE))`. Before the
    // fix, the handler rejected it with
    // "missing field `config`" (422).
    let resp = http
        .put(format!(
            "{}/qdrant/collections/{}",
            VECTORIZER_API_URL, name
        ))
        .json(&json!({
            "vectors": { "size": 4, "distance": "Cosine" }
        }))
        .send()
        .expect("PUT /qdrant/collections/<name>");
    assert!(
        resp.status().is_success(),
        "flat {{vectors: ...}} payload must be accepted, got status {}",
        resp.status()
    );
}

#[test]
#[ignore = "requires running vectorizer server on 127.0.0.1:15002"]
fn wrapped_legacy_shape_still_creates_collection() {
    let http = client();
    let name = "qdrant_compat_minimal_wrapped";
    delete_collection(&http, name);

    // The historic `{config: {vectors: ...}}` shape must keep parsing
    // so every operator script and SDK test pinned to the old shape
    // does not regress.
    let resp = http
        .put(format!(
            "{}/qdrant/collections/{}",
            VECTORIZER_API_URL, name
        ))
        .json(&json!({
            "config": {
                "vectors": { "size": 8, "distance": "Euclid" }
            }
        }))
        .send()
        .expect("PUT /qdrant/collections/<name>");
    assert!(
        resp.status().is_success(),
        "wrapped {{config: {{vectors: ...}}}} payload must keep working, got status {}",
        resp.status()
    );
}

#[test]
#[ignore = "requires running vectorizer server on 127.0.0.1:15002"]
fn partial_flat_shape_uses_upstream_qdrant_defaults() {
    let http = client();
    let name = "qdrant_compat_minimal_partial";
    delete_collection(&http, name);

    // Only `vectors` + `shard_number` + `hnsw_config.m` — everything
    // else must default server-side to the upstream Qdrant values.
    let resp = http
        .put(format!(
            "{}/qdrant/collections/{}",
            VECTORIZER_API_URL, name
        ))
        .json(&json!({
            "vectors": { "size": 16, "distance": "Dot" },
            "shard_number": 3,
            "hnsw_config": { "m": 32, "ef_construct": 200, "full_scan_threshold": 5000 }
        }))
        .send()
        .expect("PUT /qdrant/collections/<name>");
    assert!(
        resp.status().is_success(),
        "partially-populated flat payload must resolve defaults server-side, got status {}",
        resp.status()
    );
}
