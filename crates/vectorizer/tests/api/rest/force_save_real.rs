//! Live integration tests for `POST /collections/{name}/force-save`.
//!
//! Cover phase8_fix-force-save-endpoint: the handler used to return
//! success without ever writing `vectorizer.vecdb`. These tests seed a
//! collection via `/batch_insert`, trigger `/force-save`, and assert the
//! on-disk `.vecdb` file appears and grows.
//!
//! Require a running server at `127.0.0.1:15002`. Run with:
//! `cargo test --test all_tests api::rest::force_save_real -- --ignored`

#![allow(clippy::unwrap_used, clippy::expect_used)]
#![allow(clippy::uninlined_format_args)]

use std::time::Duration;

use serde_json::{Value, json};

const VECTORIZER_API_URL: &str = "http://127.0.0.1:15002";

fn client() -> reqwest::blocking::Client {
    reqwest::blocking::Client::builder()
        .timeout(Duration::from_secs(60))
        .build()
        .expect("build reqwest client")
}

fn ensure_collection(http: &reqwest::blocking::Client, name: &str) {
    let _ = http
        .delete(format!("{}/collections/{}", VECTORIZER_API_URL, name))
        .send();
    let resp = http
        .post(format!("{}/collections", VECTORIZER_API_URL))
        .json(&json!({"name": name, "dimension": 512, "metric": "cosine"}))
        .send()
        .expect("create collection");
    assert!(
        resp.status().is_success(),
        "create status {}",
        resp.status()
    );
}

fn seed_texts(http: &reqwest::blocking::Client, name: &str, n: usize) {
    let texts: Vec<Value> = (0..n)
        .map(|i| json!({"text": format!("force-save probe doc {}", i)}))
        .collect();
    let resp: Value = http
        .post(format!("{}/batch_insert", VECTORIZER_API_URL))
        .json(&json!({"collection": name, "texts": texts}))
        .send()
        .expect("batch_insert")
        .json()
        .expect("decode batch_insert");
    assert_eq!(resp["inserted"].as_u64(), Some(n as u64));
}

/// `vectorizer_core::paths::data_dir()` on the live host — matches the
/// library resolution (honoring `VECTORIZER_DATA_DIR`, otherwise
/// `dirs::data_dir().join("vectorizer")`).
fn vecdb_path() -> std::path::PathBuf {
    if let Ok(p) = std::env::var("VECTORIZER_DATA_DIR") {
        return std::path::PathBuf::from(p).join("vectorizer.vecdb");
    }
    dirs::data_dir()
        .expect("data_dir")
        .join("vectorizer")
        .join("vectorizer.vecdb")
}

#[test]
#[ignore = "requires running vectorizer server on 127.0.0.1:15002"]
fn force_save_flushes_vecdb_to_disk() {
    let http = client();
    ensure_collection(&http, "force_save_real_ok");
    seed_texts(&http, "force_save_real_ok", 20);

    // Capture the .vecdb state before the flush (it may or may not exist
    // depending on whether the 5min timer has run since boot).
    let path = vecdb_path();
    let before_size = std::fs::metadata(&path).map(|m| m.len()).unwrap_or(0);

    let resp: Value = http
        .post(format!(
            "{}/collections/force_save_real_ok/force-save",
            VECTORIZER_API_URL
        ))
        .json(&json!({}))
        .send()
        .expect("POST /force-save")
        .json()
        .expect("decode response");

    assert_eq!(resp["success"].as_bool(), Some(true));
    assert_eq!(
        resp["flushed"].as_bool(),
        Some(true),
        "flushed flag must be true on servers with an auto_save_manager"
    );

    // File must exist after the flush.
    assert!(
        path.exists(),
        "vectorizer.vecdb should exist at {:?} after force-save",
        path
    );

    // And must be at least as large as before — compaction may rewrite
    // with the same content but shouldn't shrink to zero.
    let after_size = std::fs::metadata(&path).expect("vecdb metadata").len();
    assert!(
        after_size > 0,
        "vectorizer.vecdb is empty (size={}) after force-save",
        after_size
    );
    assert!(
        after_size >= before_size || before_size == 0,
        "vectorizer.vecdb shrank unexpectedly (before={}, after={})",
        before_size,
        after_size
    );
}

#[test]
#[ignore = "requires running vectorizer server on 127.0.0.1:15002"]
fn force_save_rejects_unknown_collection() {
    let http = client();
    let resp = http
        .post(format!(
            "{}/collections/force_save_real_missing_xyz/force-save",
            VECTORIZER_API_URL
        ))
        .json(&json!({}))
        .send()
        .expect("POST /force-save");
    assert!(
        !resp.status().is_success(),
        "unknown collection should not return 2xx (got {})",
        resp.status()
    );
}
