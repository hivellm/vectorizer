//! Live integration tests for `POST /collections/{src}/vectors/move`
//! (issue #265 — tier-demotion API).
//!
//! Asserts:
//!   - Happy path moves N vectors, response contract matches spec.
//!   - A missing src id yields `missing_in_src` and does not touch dst.
//!   - Dim mismatch yields `dst_insert_failed` and the src vector is
//!     NOT deleted (insert-before-delete invariant).
//!   - Partial failures don't abort the batch.
//!
//! Requires a running server at `127.0.0.1:15002`. Run with:
//! `cargo test --test all_tests api::rest::move_vectors_real -- --ignored`

#![allow(clippy::unwrap_used, clippy::expect_used)]
#![allow(clippy::uninlined_format_args)]

use std::time::Duration;

use serde_json::{Value, json};

const BASE: &str = "http://127.0.0.1:15002";

fn client() -> reqwest::blocking::Client {
    reqwest::blocking::Client::builder()
        .timeout(Duration::from_secs(30))
        .build()
        .expect("build reqwest client")
}

fn create_collection(http: &reqwest::blocking::Client, name: &str, dimension: usize) {
    let _ = http.delete(format!("{}/collections/{}", BASE, name)).send();
    let resp = http
        .post(format!("{}/collections", BASE))
        .json(&json!({"name": name, "dimension": dimension, "metric": "cosine"}))
        .send()
        .expect("create collection");
    assert!(
        resp.status().is_success(),
        "create {} status {}",
        name,
        resp.status()
    );
}

fn seed(http: &reqwest::blocking::Client, name: &str, n: usize) -> Vec<String> {
    create_collection(http, name, 512);
    let texts: Vec<Value> = (0..n)
        .map(|i| json!({"text": format!("move probe doc {}", i)}))
        .collect();
    let resp: Value = http
        .post(format!("{}/batch_insert", BASE))
        .json(&json!({"collection": name, "texts": texts}))
        .send()
        .expect("batch_insert")
        .json()
        .expect("decode batch_insert");
    assert_eq!(resp["inserted"].as_u64(), Some(n as u64));
    resp["results"]
        .as_array()
        .unwrap()
        .iter()
        .map(|r| r["vector_ids"][0].as_str().unwrap().to_string())
        .collect()
}

fn vector_count(http: &reqwest::blocking::Client, name: &str) -> u64 {
    let meta: Value = http
        .get(format!("{}/collections/{}", BASE, name))
        .send()
        .expect("GET collection")
        .json()
        .expect("decode meta");
    meta["vector_count"].as_u64().unwrap_or(0)
}

#[test]
#[ignore = "requires running vectorizer server on 127.0.0.1:15002"]
fn move_happy_path_relocates_vectors() {
    let http = client();
    let src = "move_vectors_real_src_happy";
    let dst = "move_vectors_real_dst_happy";
    let ids = seed(&http, src, 3);
    create_collection(&http, dst, 512);

    let resp: Value = http
        .post(format!("{}/collections/{}/vectors/move", BASE, src))
        .json(&json!({"destination": dst, "ids": &ids}))
        .send()
        .expect("POST move")
        .json()
        .expect("decode");

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
    assert_eq!(vector_count(&http, src), 0);
    assert_eq!(vector_count(&http, dst), 3);
}

#[test]
#[ignore = "requires running vectorizer server on 127.0.0.1:15002"]
fn move_reports_missing_in_src_without_aborting() {
    let http = client();
    let src = "move_vectors_real_src_missing";
    let dst = "move_vectors_real_dst_missing";
    let ids = seed(&http, src, 2);
    create_collection(&http, dst, 512);

    let mut request_ids = ids.clone();
    request_ids.push("never-existed-bogus".to_string());

    let resp: Value = http
        .post(format!("{}/collections/{}/vectors/move", BASE, src))
        .json(&json!({"destination": dst, "ids": &request_ids}))
        .send()
        .expect("POST move")
        .json()
        .expect("decode");

    assert_eq!(resp["requested"].as_u64(), Some(3));
    assert_eq!(resp["moved"].as_u64(), Some(2));
    assert_eq!(resp["failed"].as_u64(), Some(1));

    let results = resp["results"].as_array().unwrap();
    let missing = results
        .iter()
        .find(|r| r["id"].as_str() == Some("never-existed-bogus"))
        .expect("missing-id row in results");
    assert_eq!(missing["status"].as_str(), Some("missing_in_src"));

    assert_eq!(vector_count(&http, src), 0);
    assert_eq!(vector_count(&http, dst), 2);
}

#[test]
#[ignore = "requires running vectorizer server on 127.0.0.1:15002"]
fn move_dim_mismatch_keeps_src_vector() {
    let http = client();
    let src = "move_vectors_real_src_dim";
    let dst = "move_vectors_real_dst_dim";
    let ids = seed(&http, src, 1);
    create_collection(&http, dst, 256); // intentional mismatch — src is 512

    let resp: Value = http
        .post(format!("{}/collections/{}/vectors/move", BASE, src))
        .json(&json!({"destination": dst, "ids": &ids}))
        .send()
        .expect("POST move")
        .json()
        .expect("decode");

    assert_eq!(resp["moved"].as_u64(), Some(0));
    assert_eq!(resp["failed"].as_u64(), Some(1));
    let row = &resp["results"].as_array().unwrap()[0];
    assert_eq!(row["status"].as_str(), Some("dst_insert_failed"));

    // Insert-before-delete invariant: the src vector MUST still exist.
    assert_eq!(vector_count(&http, src), 1);
    assert_eq!(vector_count(&http, dst), 0);
}

#[test]
#[ignore = "requires running vectorizer server on 127.0.0.1:15002"]
fn move_rejects_empty_ids_and_same_collection() {
    let http = client();
    let src = "move_vectors_real_src_validation";
    let dst = "move_vectors_real_dst_validation";
    let _ids = seed(&http, src, 1);
    create_collection(&http, dst, 512);

    // empty ids
    let resp = http
        .post(format!("{}/collections/{}/vectors/move", BASE, src))
        .json(&json!({"destination": dst, "ids": []}))
        .send()
        .expect("POST move empty");
    assert_eq!(resp.status().as_u16(), 400);

    // same src and dst
    let resp = http
        .post(format!("{}/collections/{}/vectors/move", BASE, src))
        .json(&json!({"destination": src, "ids": ["x"]}))
        .send()
        .expect("POST move self");
    assert_eq!(resp.status().as_u16(), 400);
}
