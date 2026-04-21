//! Live integration tests for `POST /files/upload` + the file-navigation
//! REST surface (`/file/list`, `/file/chunks`).
//!
//! Covers the phase8_fix-file-upload-payload-schema (F8) fix: uploads must
//! write their per-chunk metadata under a nested `metadata:` sub-object so
//! `FileOperations::list_files_in_collection` / `get_file_chunks_ordered`
//! can find `payload.data.metadata.file_path`. Also asserts the legacy
//! flat-shape fallback still works for one release cycle (v3.0.x).
//!
//! Require a running server at `127.0.0.1:15002`. Run with:
//! `cargo test --test all_tests api::rest::file_upload_real -- --ignored`

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

fn ensure_clean_collection(http: &reqwest::blocking::Client, name: &str) {
    let _ = http
        .delete(format!("{}/collections/{}", VECTORIZER_API_URL, name))
        .send();
    let resp = http
        .post(format!("{}/collections", VECTORIZER_API_URL))
        .json(&json!({
            "name": name,
            "dimension": 512,
            "metric": "cosine",
        }))
        .send()
        .expect("create collection");
    assert!(
        resp.status().is_success(),
        "unexpected create_collection status {}",
        resp.status()
    );
}

/// Upload `body` as `filename` into `collection` via `POST /files/upload`.
fn upload_file(
    http: &reqwest::blocking::Client,
    collection: &str,
    filename: &str,
    body: &str,
) -> Value {
    let form = reqwest::blocking::multipart::Form::new()
        .text("collection_name", collection.to_string())
        .part(
            "file",
            reqwest::blocking::multipart::Part::text(body.to_string())
                .file_name(filename.to_string())
                .mime_str("text/markdown")
                .expect("mime"),
        );

    let resp = http
        .post(format!("{}/files/upload", VECTORIZER_API_URL))
        .multipart(form)
        .send()
        .expect("POST /files/upload");
    assert!(
        resp.status().is_success(),
        "unexpected upload status {}",
        resp.status()
    );
    resp.json().expect("decode upload response")
}

#[test]
#[ignore = "requires running vectorizer server on 127.0.0.1:15002"]
fn upload_then_file_list_returns_the_file() {
    let http = client();
    let coll = "file_upload_real_list";
    ensure_clean_collection(&http, coll);

    let upload = upload_file(
        &http,
        coll,
        "f8_probe.md",
        "# F8 probe\n\nPhase 8 payload-shape regression guard.\n",
    );
    assert_eq!(upload["success"].as_bool(), Some(true));
    let chunks_created = upload["chunks_created"].as_u64().expect("chunks_created");
    assert!(
        chunks_created >= 1,
        "expected ≥1 chunk, got {chunks_created}"
    );

    // /file/list must now surface the uploaded file — pre-fix this returned
    // an empty `files` array because the reader looked under `metadata.*`
    // but the writer wrote at the payload root.
    let listing: Value = http
        .post(format!("{}/file/list", VECTORIZER_API_URL))
        .json(&json!({ "collection": coll }))
        .send()
        .expect("POST /file/list")
        .json()
        .expect("decode /file/list");
    let files = listing["files"]
        .as_array()
        .expect("files array in /file/list response");
    assert_eq!(
        files.len(),
        1,
        "expected exactly 1 file, got {} ({:?})",
        files.len(),
        files
    );
    assert_eq!(files[0]["path"].as_str(), Some("f8_probe.md"));
    assert_eq!(
        files[0]["chunk_count"].as_u64(),
        Some(chunks_created),
        "file listing chunk_count must match upload chunks_created"
    );
}

#[test]
#[ignore = "requires running vectorizer server on 127.0.0.1:15002"]
fn upload_then_file_chunks_returns_chunks_in_order() {
    let http = client();
    let coll = "file_upload_real_chunks";
    ensure_clean_collection(&http, coll);

    // Large enough to produce multiple chunks with the default 2048-char
    // chunk size. Use a synthetic content that chunks deterministically.
    let mut body = String::new();
    for i in 0..5 {
        body.push_str(&format!(
            "## Section {i}\n\n{}\n\n",
            "lorem ipsum ".repeat(200)
        ));
    }

    let upload = upload_file(&http, coll, "chunked.md", &body);
    let chunks_created = upload["chunks_created"].as_u64().expect("chunks_created");
    assert!(
        chunks_created >= 2,
        "expected ≥2 chunks for multi-section body, got {chunks_created}"
    );

    let chunks_resp: Value = http
        .post(format!("{}/file/chunks", VECTORIZER_API_URL))
        .json(&json!({
            "collection": coll,
            "file_path": "chunked.md",
            "start_chunk": 0,
            "limit": 100,
        }))
        .send()
        .expect("POST /file/chunks")
        .json()
        .expect("decode /file/chunks");

    // Accept either shape the handler may surface (the key is that it
    // returns the chunks, not an empty `files:[]` or a not-found error).
    let chunks = chunks_resp["chunks"]
        .as_array()
        .expect("chunks array in /file/chunks response");
    assert_eq!(
        chunks.len() as u64,
        chunks_created,
        "/file/chunks must surface every chunk the upload created"
    );
    // Chunks must be ordered by chunk_index ascending.
    let indices: Vec<u64> = chunks
        .iter()
        .filter_map(|c| c["chunk_index"].as_u64())
        .collect();
    for w in indices.windows(2) {
        assert!(
            w[0] <= w[1],
            "chunks must be ordered by chunk_index (saw {} then {})",
            w[0],
            w[1]
        );
    }
}

#[test]
#[ignore = "requires running vectorizer server on 127.0.0.1:15002"]
fn legacy_flat_shape_is_still_readable_via_fallback() {
    // Seed a collection via the non-chunked `POST /insert` path, which
    // writes metadata keys flat at the payload root (the legacy shape
    // the v3.0.x fallback exists to bridge).
    let http = client();
    let coll = "file_upload_real_flat_fallback";
    ensure_clean_collection(&http, coll);

    let resp: Value = http
        .post(format!("{}/insert", VECTORIZER_API_URL))
        .json(&json!({
            "collection": coll,
            "text": "legacy flat-shape probe content",
            "auto_chunk": false,
            "metadata": {
                "file_path": "legacy_flat.md",
                "chunk_index": "0",
                "source": "legacy-flat-fallback-test",
            },
        }))
        .send()
        .expect("POST /insert")
        .json()
        .expect("decode /insert");
    assert_eq!(resp["vectors_created"].as_u64(), Some(1));

    // The dual-shape reader in `FileOperations::list_files_in_collection`
    // must still surface this legacy-shape row.
    let listing: Value = http
        .post(format!("{}/file/list", VECTORIZER_API_URL))
        .json(&json!({ "collection": coll }))
        .send()
        .expect("POST /file/list")
        .json()
        .expect("decode /file/list");
    let files = listing["files"]
        .as_array()
        .expect("files array in /file/list response");
    assert!(
        files
            .iter()
            .any(|f| f["path"].as_str() == Some("legacy_flat.md")),
        "legacy flat-shape vector must be discoverable via /file/list fallback: {:?}",
        files
    );
}
