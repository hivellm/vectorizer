//! In-process migration of
//! `crates/vectorizer/tests/api/rest/file_upload_real.rs` (3 tests) +
//! `crates/vectorizer/tests/api/rest/file_upload.rs` (6 tests) (phase39
//! §1.2 final batch) onto the shared harness in `tests/common/mod.rs`.
//!
//! Same assertions as the two live-server suites, but dispatched through
//! the real production router via `tower::ServiceExt::oneshot` instead of
//! a `reqwest` multipart client against `127.0.0.1:15002` — no
//! `#[ignore]`, runs in CI. Multipart bodies are hand-encoded by
//! `common::TestApp::post_multipart` (axum's `Multipart` extractor only
//! needs a syntactically valid stream, not a real HTTP client).
//!
//! Covers `POST /files/upload`, `GET /files/config`, and the
//! phase8_fix-file-upload-payload-schema (F8) regression: uploads must
//! write per-chunk metadata under a nested `metadata:` sub-object so
//! `FileOperations::list_files_in_collection` / `get_file_chunks_ordered`
//! can find `payload.data.metadata.file_path`, while the legacy flat-shape
//! fallback (metadata keys at the payload root, written by the
//! non-chunked `POST /insert` path) stays readable for one release cycle.

#![allow(clippy::unwrap_used, clippy::expect_used)]
#![allow(clippy::uninlined_format_args)]

mod common;

use axum::http::StatusCode;
use common::{MultipartField, TestApp};
use serde_json::{Value, json};

/// Upload `body` as `filename` (`content_type`) into `collection` via the
/// real `POST /files/upload` multipart handler. Asserts the upload
/// succeeded and returns the decoded `FileUploadResponse` JSON.
async fn upload_file(
    app: &TestApp,
    collection: &str,
    filename: &str,
    content_type: &str,
    body: &[u8],
) -> Value {
    let fields = vec![
        MultipartField::file("file", filename, content_type, body.to_vec()),
        MultipartField::text("collection_name", collection),
    ];
    let (status, resp) = app.post_multipart("/files/upload", &fields).await;
    assert!(status.is_success(), "upload status {status}: {resp}");
    resp
}

/// Create `name` as a 512-dim cosine collection (512 matches the
/// harness's fixed BM25 provider dimension — see `common/mod.rs`).
async fn create_collection(app: &TestApp, name: &str) {
    let (status, resp) = app
        .post_json(
            "/collections",
            json!({"name": name, "dimension": 512, "metric": "cosine"}),
        )
        .await;
    assert!(status.is_success(), "create {name} status {status}: {resp}");
}

// ─── GET /files/config ──────────────────────────────────────────────────────

#[tokio::test]
async fn get_upload_config_returns_expected_fields() {
    let app = TestApp::new().await;

    let (status, body) = app.get("/files/config").await;
    assert_eq!(
        status,
        StatusCode::OK,
        "GET /files/config status {status}: {body}"
    );

    for field in [
        "max_file_size",
        "max_file_size_mb",
        "allowed_extensions",
        "reject_binary",
        "default_chunk_size",
        "default_chunk_overlap",
    ] {
        assert!(
            body.get(field).is_some(),
            "missing field '{field}' in {body}"
        );
    }
}

// ─── POST /files/upload ─────────────────────────────────────────────────────

#[tokio::test]
async fn upload_text_file_creates_chunks_and_vectors() {
    let app = TestApp::new().await;
    let content = b"# Test Document\n\nThis is a test document for the file upload API.\n\n\
                     ## Section 1\n\nSome content here.\n\n## Section 2\n\nMore content here.";

    let resp = upload_file(
        &app,
        "upload_inprocess_text",
        "test_document.md",
        "text/markdown",
        content,
    )
    .await;

    assert_eq!(resp["success"].as_bool(), Some(true));
    assert_eq!(resp["filename"].as_str(), Some("test_document.md"));
    assert!(
        resp["chunks_created"].as_u64().unwrap_or(0) > 0,
        "expected >0 chunks, got {resp}"
    );
    assert!(
        resp["vectors_created"].as_u64().unwrap_or(0) > 0,
        "expected >0 vectors, got {resp}"
    );
    assert_eq!(resp["language"].as_str(), Some("markdown"));
}

#[tokio::test]
async fn upload_code_file_detects_rust_language() {
    let app = TestApp::new().await;
    let content = br"
//! A test module

/// Calculate the factorial of a number
pub fn factorial(n: u64) -> u64 {
    match n {
        0 | 1 => 1,
        _ => n * factorial(n - 1),
    }
}

/// Check if a number is prime
pub fn is_prime(n: u64) -> bool {
    if n <= 1 {
        return false;
    }
    for i in 2..=((n as f64).sqrt() as u64) {
        if n % i == 0 {
            return false;
        }
    }
    true
}
";

    let fields = vec![
        MultipartField::file("file", "test_math.rs", "text/x-rust", content.to_vec()),
        MultipartField::text("collection_name", "upload_inprocess_code"),
        MultipartField::text("chunk_size", "512"),
        MultipartField::text("chunk_overlap", "64"),
    ];
    let (status, resp) = app.post_multipart("/files/upload", &fields).await;
    assert!(status.is_success(), "upload status {status}: {resp}");

    assert_eq!(resp["success"].as_bool(), Some(true));
    assert_eq!(resp["language"].as_str(), Some("rust"));
}

#[tokio::test]
async fn upload_rejects_disallowed_extension() {
    let app = TestApp::new().await;
    let content = b"\x89PNG\r\n\x1a\n\x00\x00\x00\rIHDR";

    let fields = vec![
        MultipartField::file("file", "image.png", "image/png", content.to_vec()),
        MultipartField::text("collection_name", "upload_inprocess_invalid_ext"),
    ];
    let (status, body) = app.post_multipart("/files/upload", &fields).await;
    assert_eq!(
        status.as_u16(),
        400,
        "expected 400 for a disallowed extension, got {status}: {body}"
    );
    assert!(body.get("error_type").is_some());
}

#[tokio::test]
async fn upload_rejects_missing_collection_name() {
    let app = TestApp::new().await;

    let fields = vec![MultipartField::file(
        "file",
        "test.txt",
        "text/plain",
        b"Hello, World!".to_vec(),
    )];
    let (status, body) = app.post_multipart("/files/upload", &fields).await;
    assert_eq!(
        status.as_u16(),
        400,
        "expected 400 for missing collection_name, got {status}: {body}"
    );
}

#[tokio::test]
async fn upload_with_metadata_succeeds() {
    let app = TestApp::new().await;
    let content = b"# README\n\nThis is a project readme file.";
    let metadata = json!({
        "project": "test-project",
        "version": "1.0.0",
        "tags": ["documentation", "readme"],
    });

    let fields = vec![
        MultipartField::file("file", "README.md", "text/markdown", content.to_vec()),
        MultipartField::text("collection_name", "upload_inprocess_metadata"),
        MultipartField::text("metadata", metadata.to_string()),
    ];
    let (status, resp) = app.post_multipart("/files/upload", &fields).await;
    assert!(status.is_success(), "upload status {status}: {resp}");
    assert_eq!(resp["success"].as_bool(), Some(true));
}

// ─── POST /files/upload → /file/list, /file/chunks (F8 regression) ─────────

#[tokio::test]
async fn upload_then_file_list_returns_the_file() {
    let app = TestApp::new().await;
    let coll = "upload_inprocess_list";
    create_collection(&app, coll).await;

    let upload = upload_file(
        &app,
        coll,
        "f8_probe.md",
        "text/markdown",
        b"# F8 probe\n\nPhase 8 payload-shape regression guard.\n",
    )
    .await;
    assert_eq!(upload["success"].as_bool(), Some(true));
    let chunks_created = upload["chunks_created"].as_u64().expect("chunks_created");
    assert!(
        chunks_created >= 1,
        "expected \u{2265}1 chunk, got {chunks_created}"
    );

    // /file/list must surface the uploaded file — pre-F8-fix this returned
    // an empty `files` array because the reader looked under `metadata.*`
    // but the writer wrote at the payload root.
    let (status, listing) = app
        .post_json("/file/list", json!({ "collection": coll }))
        .await;
    assert!(status.is_success(), "/file/list status {status}: {listing}");
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

#[tokio::test]
async fn upload_then_file_chunks_returns_chunks_in_order() {
    let app = TestApp::new().await;
    let coll = "upload_inprocess_chunks";
    create_collection(&app, coll).await;

    // Large enough to produce multiple chunks with the default 2048-char
    // chunk size. Synthetic content that chunks deterministically.
    let mut body = String::new();
    for i in 0..5 {
        body.push_str(&format!(
            "## Section {i}\n\n{}\n\n",
            "lorem ipsum ".repeat(200)
        ));
    }

    let upload = upload_file(&app, coll, "chunked.md", "text/markdown", body.as_bytes()).await;
    let chunks_created = upload["chunks_created"].as_u64().expect("chunks_created");
    assert!(
        chunks_created >= 2,
        "expected \u{2265}2 chunks for multi-section body, got {chunks_created}"
    );

    let (status, chunks_resp) = app
        .post_json(
            "/file/chunks",
            json!({
                "collection": coll,
                "file_path": "chunked.md",
                "start_chunk": 0,
                "limit": 100,
            }),
        )
        .await;
    assert!(
        status.is_success(),
        "/file/chunks status {status}: {chunks_resp}"
    );

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

#[tokio::test]
async fn legacy_flat_shape_is_still_readable_via_fallback() {
    // Seed a collection via the non-chunked `POST /insert` path, which
    // writes metadata keys flat at the payload root (the legacy shape
    // the fallback in `FileOperations::list_files_in_collection` exists
    // to bridge).
    let app = TestApp::new().await;
    let coll = "upload_inprocess_flat_fallback";
    create_collection(&app, coll).await;

    let (status, resp) = app
        .post_json(
            "/insert",
            json!({
                "collection": coll,
                "text": "legacy flat-shape probe content",
                "auto_chunk": false,
                "metadata": {
                    "file_path": "legacy_flat.md",
                    "chunk_index": "0",
                    "source": "legacy-flat-fallback-test",
                },
            }),
        )
        .await;
    assert!(status.is_success(), "/insert status {status}: {resp}");
    assert_eq!(resp["vectors_created"].as_u64(), Some(1));

    // The dual-shape reader in `FileOperations::list_files_in_collection`
    // must still surface this legacy-shape row.
    let (status, listing) = app
        .post_json("/file/list", json!({ "collection": coll }))
        .await;
    assert!(status.is_success(), "/file/list status {status}: {listing}");
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
