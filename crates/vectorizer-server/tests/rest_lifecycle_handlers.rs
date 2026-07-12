//! Router-level coverage for the phase39 §2 batch A collection/vector
//! lifecycle handlers that had no dedicated REST test yet, dispatched
//! through the shared in-process harness in `tests/common/mod.rs`.
//!
//! Handlers covered (routes from
//! `crates/vectorizer-server/src/server/core/routing.rs`):
//! - `POST /collections/{name}/explain` — `explain_search`
//! - `POST /collections/{name}/vectors/copy` — `copy_vectors`
//! - `PATCH /collections/{name}/vectors/{id}/expiry` — `set_vector_expiry`
//! - `POST /collections/{name}/vectors/delete_by_filter` — `delete_by_filter`
//! - `POST /collections/{name}/vectors/bulk_update_metadata` — `bulk_update_metadata`
//! - `POST /collections/{name}/ttl` — `set_collection_ttl`
//! - `POST /collections/{name}/rename` — `rename_collection`
//! - `POST /collections/{name}/reindex` — `reindex_collection`
//! - `POST /collections/{name}/reencode` — `reencode_collection`
//! - `POST /collections/{name}/snapshot`,
//!   `GET /collections/{name}/snapshots`,
//!   `POST /collections/{name}/snapshots/{id}/restore` —
//!   `create_native_snapshot` / `list_native_snapshots` /
//!   `restore_native_snapshot` (grouped into a single round-trip test —
//!   see `ENV_DIR_LOCK` below)
//! - `GET /collections/empty` — `list_empty_collections`
//!
//! ## Serializing the native-snapshot tests (`ENV_DIR_LOCK`)
//!
//! `tests/common/mod.rs`'s "Known limitation" section flags that
//! `TestApp::new()` mutates the process-global `VECTORIZER_DATA_DIR` env
//! var, which is safe for tests that only exercise in-memory operations
//! but NOT for a suite that reads/writes on-disk paths derived from that
//! var after construction — which the native-snapshot handlers do
//! (`VectorStore::get_data_dir()` reads the var at call time, not just at
//! `TestApp::new()` time). If another test in this binary calls
//! `TestApp::new()` concurrently while a snapshot test's request is in
//! flight, the snapshot test could silently read/write the WRONG
//! directory.
//!
//! `ENV_DIR_LOCK` fixes this without adding a `serial_test`-style
//! dependency: every test in this file constructs its `TestApp` while
//! holding the lock (via [`new_app`], which acquires-then-immediately-
//! releases it). The three snapshot tests instead acquire the lock
//! manually and hold it for their ENTIRE body — construction through the
//! last disk-dependent assertion — so no other test's `TestApp::new()`
//! call can run concurrently and shift the env var out from under them.

#![allow(clippy::unwrap_used, clippy::expect_used)]
#![allow(clippy::uninlined_format_args)]

mod common;

use std::sync::LazyLock;

use axum::http::StatusCode;
use common::TestApp;
use serde_json::{Value, json};
use tokio::sync::Mutex as AsyncMutex;

/// See the module doc comment above for the full rationale.
static ENV_DIR_LOCK: LazyLock<AsyncMutex<()>> = LazyLock::new(|| AsyncMutex::new(()));

/// Construct a [`TestApp`] while briefly holding [`ENV_DIR_LOCK`] so
/// construction never races with a disk-dependent test's longer-held
/// guard. Every test in this file EXCEPT the native-snapshot trio uses
/// this helper.
async fn new_app() -> TestApp {
    let _guard = ENV_DIR_LOCK.lock().await;
    TestApp::new().await
}

/// Delete-then-create `name` as a 512-dim cosine collection (512 matches
/// the harness's fixed BM25 provider dimension — see `common/mod.rs`).
async fn create_collection(app: &TestApp, name: &str) {
    let _ = app.delete(&format!("/collections/{name}")).await;
    let (status, resp) = app
        .post_json(
            "/collections",
            json!({"name": name, "dimension": 512, "metric": "cosine"}),
        )
        .await;
    assert!(status.is_success(), "create {name} status {status}: {resp}");
}

/// Create `name` and batch-insert one probe text per entry in `tags`,
/// storing `{"tag": tags[i]}` as vector metadata (surfaced as a
/// top-level payload key by the chunked insert path). Returns the
/// assigned vector ids in the same order as `tags`.
async fn seed_with_tags(app: &TestApp, name: &str, tags: &[&str]) -> Vec<String> {
    create_collection(app, name).await;
    let texts: Vec<Value> = tags
        .iter()
        .enumerate()
        .map(|(i, tag)| {
            json!({
                "text": format!("lifecycle probe doc {i}"),
                "metadata": {"tag": tag},
            })
        })
        .collect();
    let (status, resp) = app
        .post_json("/batch_insert", json!({"collection": name, "texts": texts}))
        .await;
    assert!(status.is_success(), "batch_insert status {status}: {resp}");
    assert_eq!(resp["inserted"].as_u64(), Some(tags.len() as u64));
    resp["results"]
        .as_array()
        .expect("results array")
        .iter()
        .map(|r| r["vector_ids"][0].as_str().expect("vector id").to_string())
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

/// `GET /collections/{name}/vectors?limit=50` and return the raw
/// `vectors` array (each entry has `id`, `vector`, `payload`).
async fn list_all_vectors(app: &TestApp, name: &str) -> Vec<Value> {
    let (status, resp) = app
        .get(&format!("/collections/{name}/vectors?limit=50"))
        .await;
    assert!(status.is_success(), "list_vectors status {status}: {resp}");
    resp["vectors"].as_array().cloned().unwrap_or_default()
}

// ─── explain_search ─────────────────────────────────────────────────────────

#[tokio::test]
async fn explain_search_happy_path_returns_results_and_trace() {
    let app = new_app().await;
    let name = "lifecycle_explain_happy";
    seed_with_tags(&app, name, &["a", "b", "c"]).await;

    let vectors = list_all_vectors(&app, name).await;
    let query_vector: Vec<f32> = vectors[0]["vector"]
        .as_array()
        .expect("vector array")
        .iter()
        .map(|v| v.as_f64().unwrap_or(0.0) as f32)
        .collect();

    let (status, resp) = app
        .post_json(
            &format!("/collections/{name}/explain"),
            json!({"vector": query_vector, "k": 2}),
        )
        .await;
    assert_eq!(status, StatusCode::OK, "explain status {status}: {resp}");
    assert_eq!(resp["collection"].as_str(), Some(name));
    let results = resp["results"].as_array().expect("results array");
    assert!(!results.is_empty(), "explain returned zero results");

    let trace = &resp["trace"];
    assert!(trace["ef_search"].is_number(), "trace.ef_search missing");
    assert!(trace["total_ms"].is_number(), "trace.total_ms missing");
    assert!(
        trace["visited_nodes"].is_number(),
        "trace.visited_nodes missing"
    );
}

#[tokio::test]
async fn explain_search_rejects_missing_collection() {
    let app = new_app().await;
    let (status, body) = app
        .post_json(
            "/collections/lifecycle_explain_missing/explain",
            json!({"vector": vec![0.1_f32; 512], "k": 1}),
        )
        .await;
    assert_eq!(status.as_u16(), 404);
    assert_eq!(body["error_type"].as_str(), Some("collection_not_found"));
}

// ─── copy_vectors ───────────────────────────────────────────────────────────

#[tokio::test]
async fn copy_vectors_happy_path_duplicates_without_touching_source() {
    let app = new_app().await;
    let src = "lifecycle_copy_src_happy";
    let dst = "lifecycle_copy_dst_happy";
    let ids = seed_with_tags(&app, src, &["a", "b", "c"]).await;
    create_collection(&app, dst).await;

    let (status, resp) = app
        .post_json(
            &format!("/collections/{src}/vectors/copy"),
            json!({"destination": dst, "ids": &ids}),
        )
        .await;
    assert!(status.is_success(), "copy status {status}: {resp}");
    assert_eq!(resp["copied"].as_u64(), Some(3));
    assert_eq!(resp["failed"].as_u64(), Some(0));

    assert_eq!(vector_count(&app, src).await, 3, "source must be untouched");
    assert_eq!(vector_count(&app, dst).await, 3);
}

#[tokio::test]
async fn copy_vectors_rejects_empty_ids_and_same_collection() {
    let app = new_app().await;
    let src = "lifecycle_copy_src_validation";
    let ids = seed_with_tags(&app, src, &["a"]).await;
    let dst = "lifecycle_copy_dst_validation";
    create_collection(&app, dst).await;

    let (status, _) = app
        .post_json(
            &format!("/collections/{src}/vectors/copy"),
            json!({"destination": dst, "ids": []}),
        )
        .await;
    assert_eq!(status.as_u16(), 400);

    let (status, _) = app
        .post_json(
            &format!("/collections/{src}/vectors/copy"),
            json!({"destination": src, "ids": &ids}),
        )
        .await;
    assert_eq!(status.as_u16(), 400);
}

// ─── set_vector_expiry ──────────────────────────────────────────────────────

#[tokio::test]
async fn set_vector_expiry_happy_path_sets_and_clears() {
    let app = new_app().await;
    let name = "lifecycle_expiry_happy";
    let ids = seed_with_tags(&app, name, &["a"]).await;
    let id = &ids[0];

    let expires_at = 4_102_444_800_000_i64; // arbitrary far-future unix ms timestamp
    let (status, resp) = app
        .patch_json(
            &format!("/collections/{name}/vectors/{id}/expiry"),
            json!({"expires_at": expires_at}),
        )
        .await;
    assert!(status.is_success(), "set expiry status {status}: {resp}");
    assert_eq!(resp["expires_at"].as_i64(), Some(expires_at));
    assert_eq!(resp["status"].as_str(), Some("ok"));

    let vectors = list_all_vectors(&app, name).await;
    let payload = vectors
        .iter()
        .find(|v| v["id"].as_str() == Some(id.as_str()))
        .map(|v| v["payload"].clone())
        .expect("vector present in list");
    assert_eq!(payload["__expires_at"].as_i64(), Some(expires_at));

    let (status, resp) = app
        .patch_json(
            &format!("/collections/{name}/vectors/{id}/expiry"),
            json!({"expires_at": null}),
        )
        .await;
    assert!(status.is_success(), "clear expiry status {status}: {resp}");
    assert!(resp["expires_at"].is_null());
}

#[tokio::test]
async fn set_vector_expiry_rejects_missing_vector() {
    let app = new_app().await;
    let name = "lifecycle_expiry_missing";
    create_collection(&app, name).await;

    let (status, body) = app
        .patch_json(
            &format!("/collections/{name}/vectors/does-not-exist/expiry"),
            json!({"expires_at": 123}),
        )
        .await;
    assert_eq!(status.as_u16(), 404);
    assert_eq!(body["error_type"].as_str(), Some("vector_not_found"));
}

// ─── delete_by_filter ───────────────────────────────────────────────────────

#[tokio::test]
async fn delete_by_filter_happy_path_deletes_matching_vectors() {
    let app = new_app().await;
    let name = "lifecycle_delete_by_filter_happy";
    seed_with_tags(&app, name, &["drop", "drop", "keep", "keep"]).await;

    let (status, resp) = app
        .post_json(
            &format!("/collections/{name}/vectors/delete_by_filter"),
            json!({
                "filter": {
                    "must": [
                        {"type": "match", "key": "tag", "match_value": "drop"}
                    ]
                }
            }),
        )
        .await;
    assert!(
        status.is_success(),
        "delete_by_filter status {status}: {resp}"
    );
    assert_eq!(resp["scanned"].as_u64(), Some(4));
    assert_eq!(resp["matched"].as_u64(), Some(2));
    assert_eq!(resp["deleted"].as_u64(), Some(2));

    assert_eq!(vector_count(&app, name).await, 2);
}

#[tokio::test]
async fn delete_by_filter_rejects_empty_filter() {
    let app = new_app().await;
    let name = "lifecycle_delete_by_filter_empty";
    seed_with_tags(&app, name, &["a"]).await;

    let (status, body) = app
        .post_json(
            &format!("/collections/{name}/vectors/delete_by_filter"),
            json!({"filter": {}}),
        )
        .await;
    assert_eq!(status.as_u16(), 400);
    assert_eq!(body["error_type"].as_str(), Some("validation_error"));
    assert_eq!(
        vector_count(&app, name).await,
        1,
        "empty filter must not wipe the collection"
    );
}

// ─── bulk_update_metadata ───────────────────────────────────────────────────

#[tokio::test]
async fn bulk_update_metadata_happy_path_patches_matching_payloads() {
    let app = new_app().await;
    let name = "lifecycle_bulk_update_happy";
    seed_with_tags(&app, name, &["target", "target", "other"]).await;

    let (status, resp) = app
        .post_json(
            &format!("/collections/{name}/vectors/bulk_update_metadata"),
            json!({
                "filter": {
                    "must": [
                        {"type": "match", "key": "tag", "match_value": "target"}
                    ]
                },
                "patch": {"reviewed": true},
            }),
        )
        .await;
    assert!(
        status.is_success(),
        "bulk_update_metadata status {status}: {resp}"
    );
    assert_eq!(resp["scanned"].as_u64(), Some(3));
    assert_eq!(resp["matched"].as_u64(), Some(2));
    assert_eq!(resp["updated"].as_u64(), Some(2));

    let vectors = list_all_vectors(&app, name).await;
    let reviewed_count = vectors
        .iter()
        .filter(|v| v["payload"]["reviewed"].as_bool() == Some(true))
        .count();
    assert_eq!(reviewed_count, 2);
}

#[tokio::test]
async fn bulk_update_metadata_rejects_missing_patch_field() {
    let app = new_app().await;
    let name = "lifecycle_bulk_update_missing_patch";
    seed_with_tags(&app, name, &["a"]).await;

    let (status, body) = app
        .post_json(
            &format!("/collections/{name}/vectors/bulk_update_metadata"),
            json!({
                "filter": {"must": [{"type": "match", "key": "tag", "match_value": "a"}]},
            }),
        )
        .await;
    assert_eq!(status.as_u16(), 400);
    assert_eq!(body["error_type"].as_str(), Some("validation_error"));
}

// ─── set_collection_ttl ─────────────────────────────────────────────────────

#[tokio::test]
async fn set_collection_ttl_happy_path_sets_and_clears() {
    let app = new_app().await;
    let name = "lifecycle_ttl_happy";
    create_collection(&app, name).await;

    let (status, resp) = app
        .post_json(
            &format!("/collections/{name}/ttl"),
            json!({"ttl_secs": 3600}),
        )
        .await;
    assert!(status.is_success(), "set ttl status {status}: {resp}");
    assert_eq!(resp["ttl_secs"].as_u64(), Some(3600));
    assert_eq!(resp["status"].as_str(), Some("ok"));

    let (status, resp) = app
        .post_json(
            &format!("/collections/{name}/ttl"),
            json!({"ttl_secs": null}),
        )
        .await;
    assert!(status.is_success(), "clear ttl status {status}: {resp}");
    assert!(resp["ttl_secs"].is_null());
}

#[tokio::test]
async fn set_collection_ttl_rejects_missing_collection() {
    let app = new_app().await;
    let (status, body) = app
        .post_json(
            "/collections/lifecycle_ttl_missing/ttl",
            json!({"ttl_secs": 60}),
        )
        .await;
    assert_eq!(status.as_u16(), 404);
    assert_eq!(body["error_type"].as_str(), Some("collection_not_found"));
}

// ─── rename_collection ──────────────────────────────────────────────────────

#[tokio::test]
async fn rename_collection_happy_path_renames_and_keeps_alias() {
    let app = new_app().await;
    let old_name = "lifecycle_rename_happy_old";
    let new_name = "lifecycle_rename_happy_new";
    create_collection(&app, old_name).await;
    let _ = app.delete(&format!("/collections/{new_name}")).await;

    let (status, resp) = app
        .post_json(
            &format!("/collections/{old_name}/rename"),
            json!({"new_name": new_name}),
        )
        .await;
    assert!(status.is_success(), "rename status {status}: {resp}");
    assert_eq!(resp["old_name"].as_str(), Some(old_name));
    assert_eq!(resp["new_name"].as_str(), Some(new_name));
    assert_eq!(resp["alias_retained"].as_str(), Some(old_name));
    assert_eq!(resp["status"].as_str(), Some("ok"));

    let (status, meta) = app.get(&format!("/collections/{new_name}")).await;
    assert!(
        status.is_success(),
        "get renamed collection status {status}: {meta}"
    );
}

#[tokio::test]
async fn rename_collection_rejects_missing_new_name() {
    let app = new_app().await;
    let name = "lifecycle_rename_invalid";
    create_collection(&app, name).await;

    let (status, body) = app
        .post_json(&format!("/collections/{name}/rename"), json!({}))
        .await;
    assert_eq!(status.as_u16(), 400);
    assert_eq!(body["error_type"].as_str(), Some("validation_error"));
}

// ─── reindex_collection ─────────────────────────────────────────────────────

#[tokio::test]
async fn reindex_collection_happy_path_rebuilds_index() {
    let app = new_app().await;
    let name = "lifecycle_reindex_happy";
    seed_with_tags(&app, name, &["a", "b", "c"]).await;

    let (status, resp) = app
        .post_json(
            &format!("/collections/{name}/reindex"),
            json!({"m": 16, "ef_construction": 100, "ef_search": 50}),
        )
        .await;
    assert!(status.is_success(), "reindex status {status}: {resp}");
    assert_eq!(resp["state"].as_str(), Some("completed"));
    assert_eq!(resp["params"]["m"].as_u64(), Some(16));

    assert_eq!(
        vector_count(&app, name).await,
        3,
        "reindex must not lose vectors"
    );
}

#[tokio::test]
async fn reindex_collection_rejects_missing_collection() {
    let app = new_app().await;
    let (status, body) = app
        .post_json("/collections/lifecycle_reindex_missing/reindex", json!({}))
        .await;
    assert_eq!(status.as_u16(), 404);
    assert_eq!(body["error_type"].as_str(), Some("collection_not_found"));
}

// ─── reencode_collection ────────────────────────────────────────────────────

#[tokio::test]
async fn reencode_collection_happy_path_requantizes_in_place() {
    let app = new_app().await;
    let name = "lifecycle_reencode_happy";
    seed_with_tags(&app, name, &["a", "b"]).await;

    let (status, resp) = app
        .post_json(
            &format!("/collections/{name}/reencode"),
            json!({"target_encoding": "sq8"}),
        )
        .await;
    assert!(status.is_success(), "reencode status {status}: {resp}");
    assert_eq!(resp["state"].as_str(), Some("completed"));
    assert_eq!(resp["target_encoding"].as_str(), Some("sq8"));

    assert_eq!(
        vector_count(&app, name).await,
        2,
        "reencode must not lose vectors"
    );
}

#[tokio::test]
async fn reencode_collection_rejects_missing_target_encoding() {
    let app = new_app().await;
    let name = "lifecycle_reencode_invalid";
    create_collection(&app, name).await;

    let (status, body) = app
        .post_json(&format!("/collections/{name}/reencode"), json!({}))
        .await;
    assert_eq!(status.as_u16(), 400);
    assert_eq!(body["error_type"].as_str(), Some("validation_error"));
}

// ─── list_empty_collections ─────────────────────────────────────────────────

#[tokio::test]
async fn list_empty_collections_returns_only_truly_empty_collections() {
    // No genuine error branch: the handler only takes `State` (no
    // path/body params to invalidate). This covers the happy path plus
    // the filtering boundary between an empty and a populated collection.
    let app = new_app().await;
    let empty_name = "lifecycle_list_empty_target";
    let populated_name = "lifecycle_list_empty_populated";
    create_collection(&app, empty_name).await;
    seed_with_tags(&app, populated_name, &["a"]).await;

    let (status, resp) = app.get("/collections/empty").await;
    assert!(
        status.is_success(),
        "list_empty_collections status {status}: {resp}"
    );
    let empties = resp["empty_collections"]
        .as_array()
        .expect("empty_collections array");
    let names: Vec<&str> = empties.iter().filter_map(|v| v.as_str()).collect();
    assert!(
        names.contains(&empty_name),
        "expected {empty_name} in {names:?}"
    );
    assert!(
        !names.contains(&populated_name),
        "did not expect {populated_name} in {names:?}"
    );
}

// ─── create_native_snapshot / list_native_snapshots / restore_native_snapshot ──
//
// Disk-dependent: every test below holds `ENV_DIR_LOCK` for its entire
// body (see the module doc comment).

#[tokio::test]
async fn create_native_snapshot_list_native_snapshots_restore_native_snapshot_round_trip() {
    let _env_guard = ENV_DIR_LOCK.lock().await;
    let app = TestApp::new().await;
    let name = "lifecycle_snapshot_happy";
    let ids = seed_with_tags(&app, name, &["a", "b", "c"]).await;

    // POST /collections/{name}/snapshot
    let (status, resp) = app
        .post_json(&format!("/collections/{name}/snapshot"), json!({}))
        .await;
    assert!(
        status.is_success(),
        "create snapshot status {status}: {resp}"
    );
    assert_eq!(resp["collection"].as_str(), Some(name));
    assert!(resp["size_bytes"].as_u64().unwrap_or(0) > 0);
    let snapshot_id = resp["id"].as_str().expect("snapshot id").to_string();

    // GET /collections/{name}/snapshots
    let (status, list_resp) = app.get(&format!("/collections/{name}/snapshots")).await;
    assert!(
        status.is_success(),
        "list snapshots status {status}: {list_resp}"
    );
    let snapshots = list_resp["snapshots"].as_array().expect("snapshots array");
    assert!(
        snapshots
            .iter()
            .any(|s| s["id"].as_str() == Some(snapshot_id.as_str())),
        "expected snapshot {snapshot_id} in {snapshots:?}"
    );

    // Mutate collection state after the snapshot so restore has
    // something real to roll back.
    let (status, _) = app
        .delete(&format!("/collections/{name}/vectors/{}", ids[0]))
        .await;
    assert!(status.is_success());
    assert_eq!(vector_count(&app, name).await, 2);

    // POST /collections/{name}/snapshots/{id}/restore
    let (status, restore_resp) = app
        .post_json(
            &format!("/collections/{name}/snapshots/{snapshot_id}/restore"),
            json!({}),
        )
        .await;
    assert!(
        status.is_success(),
        "restore status {status}: {restore_resp}"
    );
    assert_eq!(restore_resp["status"].as_str(), Some("restored"));
    assert_eq!(
        vector_count(&app, name).await,
        3,
        "restore must bring back the deleted vector"
    );
}

#[tokio::test]
async fn create_native_snapshot_rejects_missing_collection() {
    let _env_guard = ENV_DIR_LOCK.lock().await;
    let app = TestApp::new().await;
    let (status, body) = app
        .post_json(
            "/collections/lifecycle_snapshot_missing/snapshot",
            json!({}),
        )
        .await;
    assert_eq!(status.as_u16(), 404);
    assert_eq!(body["error_type"].as_str(), Some("collection_not_found"));
}

#[tokio::test]
async fn restore_native_snapshot_rejects_unknown_snapshot_id() {
    let _env_guard = ENV_DIR_LOCK.lock().await;
    let app = TestApp::new().await;
    let name = "lifecycle_snapshot_restore_missing";
    create_collection(&app, name).await;

    let (status, body) = app
        .post_json(
            &format!("/collections/{name}/snapshots/does-not-exist/restore"),
            json!({}),
        )
        .await;
    assert_eq!(status.as_u16(), 404);
    assert_eq!(body["error_type"].as_str(), Some("not_found"));
}

#[tokio::test]
async fn list_native_snapshots_returns_empty_for_collection_without_snapshots() {
    // No genuine error branch: `VectorStore::list_native_snapshots`
    // resolves an unknown name through unchanged (alias resolution is a
    // no-op for names it doesn't recognize) and a missing snapshot
    // directory on disk short-circuits to `Ok(Vec::new())` rather than an
    // error. This test documents that 200-with-empty-list behavior in
    // place of a 4xx/5xx branch.
    let _env_guard = ENV_DIR_LOCK.lock().await;
    let app = TestApp::new().await;
    let name = "lifecycle_snapshot_list_empty";
    create_collection(&app, name).await;

    let (status, resp) = app.get(&format!("/collections/{name}/snapshots")).await;
    assert!(
        status.is_success(),
        "list snapshots status {status}: {resp}"
    );
    assert_eq!(resp["snapshots"].as_array().map(Vec::len), Some(0));
    assert_eq!(resp["total"].as_u64(), Some(0));
}
