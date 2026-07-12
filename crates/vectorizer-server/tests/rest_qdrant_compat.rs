//! In-process migration of
//! `crates/vectorizer/tests/api/rest/qdrant_compat_minimal_real.rs`
//! (phase39 §1.2) onto the shared harness in `tests/common/mod.rs`.
//!
//! Same assertions as the live-server suite, but dispatched through the
//! real production router via `tower::ServiceExt::oneshot` instead of
//! `reqwest` against `127.0.0.1:15002` — no `#[ignore]`, runs in CI.
//!
//! Covers `phase8_qdrant-compat-minimal-request-shape` (probe 3.6): the
//! handler at `crates/vectorizer-server/src/server/qdrant/handlers.rs`
//! used to require a wrapped `{config: {...}}` request with 9 mandatory
//! sub-fields, so any real qdrant-client (-python / -js / ...) failed at
//! collection creation with a 422 about a missing `config` field. The
//! contract is now: only `vectors` is required, everything else defaults
//! server-side, and both the wrapped + flat request shapes parse.
//!
//! Delta vs the live suite: each test gets a brand-new [`common::TestApp`]
//! (fresh in-memory store), so the live suite's `delete_collection`
//! pre-test cleanup call (needed because the live server persists state
//! across test runs) is unnecessary here and omitted.

#![allow(clippy::unwrap_used, clippy::expect_used)]
#![allow(clippy::uninlined_format_args)]

mod common;

use common::TestApp;
use serde_json::json;

#[tokio::test]
async fn flat_minimal_shape_creates_collection() {
    let app = TestApp::new().await;
    let name = "qdrant_compat_minimal_flat";

    // This is exactly the payload qdrant-client-python sends when a
    // caller does `client.create_collection(name, vectors_config=
    // VectorParams(size=4, distance=Distance.COSINE))`. Before the fix,
    // the handler rejected it with "missing field `config`" (422).
    let (status, body) = app
        .put_json(
            &format!("/qdrant/collections/{name}"),
            json!({
                "vectors": { "size": 4, "distance": "Cosine" }
            }),
        )
        .await;
    assert!(
        status.is_success(),
        "flat {{vectors: ...}} payload must be accepted, got status {status}: {body}"
    );
}

#[tokio::test]
async fn wrapped_legacy_shape_still_creates_collection() {
    let app = TestApp::new().await;
    let name = "qdrant_compat_minimal_wrapped";

    // The historic `{config: {vectors: ...}}` shape must keep parsing so
    // every operator script and SDK test pinned to the old shape does
    // not regress.
    let (status, body) = app
        .put_json(
            &format!("/qdrant/collections/{name}"),
            json!({
                "config": {
                    "vectors": { "size": 8, "distance": "Euclid" }
                }
            }),
        )
        .await;
    assert!(
        status.is_success(),
        "wrapped {{config: {{vectors: ...}}}} payload must keep working, got status {status}: {body}"
    );
}

#[tokio::test]
async fn partial_flat_shape_uses_upstream_qdrant_defaults() {
    let app = TestApp::new().await;
    let name = "qdrant_compat_minimal_partial";

    // Only `vectors` + `shard_number` + `hnsw_config.m` — everything
    // else must default server-side to the upstream Qdrant values.
    let (status, body) = app
        .put_json(
            &format!("/qdrant/collections/{name}"),
            json!({
                "vectors": { "size": 16, "distance": "Dot" },
                "shard_number": 3,
                "hnsw_config": { "m": 32, "ef_construct": 200, "full_scan_threshold": 5000 }
            }),
        )
        .await;
    assert!(
        status.is_success(),
        "partially-populated flat payload must resolve defaults server-side, got status {status}: {body}"
    );
}
