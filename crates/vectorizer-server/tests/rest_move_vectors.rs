//! In-process migration of
//! `crates/vectorizer/tests/api/rest/move_vectors_real.rs` (phase39 §1.2)
//! onto the shared harness in `tests/common/mod.rs`.
//!
//! Same assertions as the live-server suite, but dispatched through the
//! real production router via `tower::ServiceExt::oneshot` instead of
//! `reqwest` against `127.0.0.1:15002` — no `#[ignore]`, runs in CI.
//!
//! Covers `POST /collections/{src}/vectors/move` (issue #265 —
//! tier-demotion API): happy-path relocation, missing-in-src reporting,
//! the insert-before-delete invariant on destination-dimension mismatch,
//! and empty-ids / same-collection validation.

#![allow(clippy::unwrap_used, clippy::expect_used)]
#![allow(clippy::uninlined_format_args)]

mod common;

use common::TestApp;
use serde_json::{Value, json};

/// Delete-then-create `name` as a `dimension`-dim cosine collection.
async fn create_collection(app: &TestApp, name: &str, dimension: usize) {
    let _ = app.delete(&format!("/collections/{name}")).await;
    let (status, resp) = app
        .post_json(
            "/collections",
            json!({"name": name, "dimension": dimension, "metric": "cosine"}),
        )
        .await;
    assert!(status.is_success(), "create {name} status {status}: {resp}");
}

/// Create `name` (512-dim) and batch insert `n` probe texts, returning
/// their vector ids.
async fn seed(app: &TestApp, name: &str, n: usize) -> Vec<String> {
    create_collection(app, name, 512).await;
    let texts: Vec<Value> = (0..n)
        .map(|i| json!({"text": format!("move probe doc {}", i)}))
        .collect();
    let (status, resp) = app
        .post_json("/batch_insert", json!({"collection": name, "texts": texts}))
        .await;
    assert!(status.is_success(), "batch_insert status {status}: {resp}");
    assert_eq!(resp["inserted"].as_u64(), Some(n as u64));
    resp["results"]
        .as_array()
        .unwrap()
        .iter()
        .map(|r| r["vector_ids"][0].as_str().unwrap().to_string())
        .collect()
}

/// `GET /collections/{name}` and pull out `vector_count`.
async fn vector_count(app: &TestApp, name: &str) -> u64 {
    let (status, meta) = app.get(&format!("/collections/{name}")).await;
    assert!(
        status.is_success(),
        "get collection status {status}: {meta}"
    );
    meta["vector_count"].as_u64().unwrap_or(0)
}

#[tokio::test]
async fn move_happy_path_relocates_vectors() {
    let app = TestApp::new().await;
    let src = "move_vectors_real_src_happy";
    let dst = "move_vectors_real_dst_happy";
    let ids = seed(&app, src, 3).await;
    create_collection(&app, dst, 512).await;

    let (status, resp) = app
        .post_json(
            &format!("/collections/{src}/vectors/move"),
            json!({"destination": dst, "ids": &ids}),
        )
        .await;
    assert!(status.is_success(), "POST move status {status}: {resp}");

    assert_eq!(resp["src"].as_str(), Some(src));
    assert_eq!(resp["dst"].as_str(), Some(dst));
    assert_eq!(resp["requested"].as_u64(), Some(3));
    assert_eq!(resp["moved"].as_u64(), Some(3));
    assert_eq!(resp["failed"].as_u64(), Some(0));
    let results = resp["results"].as_array().unwrap();
    assert_eq!(results.len(), 3);
    for r in results {
        assert_eq!(r["status"].as_str(), Some("ok"));
    }
    assert_eq!(vector_count(&app, src).await, 0);
    assert_eq!(vector_count(&app, dst).await, 3);
}

#[tokio::test]
async fn move_reports_missing_in_src_without_aborting() {
    let app = TestApp::new().await;
    let src = "move_vectors_real_src_missing";
    let dst = "move_vectors_real_dst_missing";
    let ids = seed(&app, src, 2).await;
    create_collection(&app, dst, 512).await;

    let mut request_ids = ids.clone();
    request_ids.push("never-existed-bogus".to_string());

    let (status, resp) = app
        .post_json(
            &format!("/collections/{src}/vectors/move"),
            json!({"destination": dst, "ids": &request_ids}),
        )
        .await;
    assert!(status.is_success(), "POST move status {status}: {resp}");

    assert_eq!(resp["requested"].as_u64(), Some(3));
    assert_eq!(resp["moved"].as_u64(), Some(2));
    assert_eq!(resp["failed"].as_u64(), Some(1));

    let results = resp["results"].as_array().unwrap();
    let missing = results
        .iter()
        .find(|r| r["id"].as_str() == Some("never-existed-bogus"))
        .expect("missing-id row in results");
    assert_eq!(missing["status"].as_str(), Some("missing_in_src"));

    assert_eq!(vector_count(&app, src).await, 0);
    assert_eq!(vector_count(&app, dst).await, 2);
}

#[tokio::test]
async fn move_dim_mismatch_keeps_src_vector() {
    // Delta vs. the live-server suite: that suite creates `dst` at
    // dimension 256 (src is 512) to force a genuine vector-dimension
    // mismatch inside `state.store.insert`. This harness registers a
    // single embedding provider (`bm25`, fixed at dimension 512 — see
    // `tests/common/mod.rs`), and `POST /collections` rejects any
    // `dimension` that doesn't match the resolved provider's native
    // dimension (phase33 #306). There is therefore no in-process way to
    // create a schema-incompatible destination collection through the
    // public API. Instead, `dst` is never created at all, so
    // `state.store.insert` fails for the same reason the handler cares
    // about — the destination collection cannot accept the vector — and
    // the response is classified identically as `dst_insert_failed`.
    // This still validates the behavior the module doc calls out: the
    // src vector is NOT deleted when the destination insert fails
    // (insert-before-delete invariant).
    let app = TestApp::new().await;
    let src = "move_vectors_real_src_dim";
    let dst = "move_vectors_real_dst_dim";
    let ids = seed(&app, src, 1).await;

    let (status, resp) = app
        .post_json(
            &format!("/collections/{src}/vectors/move"),
            json!({"destination": dst, "ids": &ids}),
        )
        .await;
    assert!(status.is_success(), "POST move status {status}: {resp}");

    assert_eq!(resp["moved"].as_u64(), Some(0));
    assert_eq!(resp["failed"].as_u64(), Some(1));
    let row = &resp["results"].as_array().unwrap()[0];
    assert_eq!(row["status"].as_str(), Some("dst_insert_failed"));

    // Insert-before-delete invariant: the src vector MUST still exist.
    assert_eq!(vector_count(&app, src).await, 1);
}

#[tokio::test]
async fn move_rejects_empty_ids_and_same_collection() {
    let app = TestApp::new().await;
    let src = "move_vectors_real_src_validation";
    let dst = "move_vectors_real_dst_validation";
    let _ids = seed(&app, src, 1).await;
    create_collection(&app, dst, 512).await;

    // empty ids
    let (status, _) = app
        .post_json(
            &format!("/collections/{src}/vectors/move"),
            json!({"destination": dst, "ids": []}),
        )
        .await;
    assert_eq!(status.as_u16(), 400);

    // same src and dst
    let (status, _) = app
        .post_json(
            &format!("/collections/{src}/vectors/move"),
            json!({"destination": src, "ids": ["x"]}),
        )
        .await;
    assert_eq!(status.as_u16(), 400);
}
