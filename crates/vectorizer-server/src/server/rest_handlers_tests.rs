//! Unit tests for REST handlers — extracted from
//! `src/server/rest_handlers.rs` under
//! `phase3_split-rest-handlers-monolith` via the `#[path]` attribute.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use super::*;

#[test]
fn collection_metrics_uuid_is_deterministic() {
    let a = collection_metrics_uuid("docs");
    let b = collection_metrics_uuid("docs");
    assert_eq!(a, b, "same name must yield same UUID across calls");
}

#[test]
fn collection_metrics_uuid_differs_between_names() {
    let docs = collection_metrics_uuid("docs");
    let products = collection_metrics_uuid("products");
    assert_ne!(docs, products, "different names must yield different UUIDs");
}

#[test]
fn collection_metrics_uuid_is_v5() {
    let id = collection_metrics_uuid("any-collection");
    assert_eq!(
        id.get_version_num(),
        5,
        "expected UUIDv5, got v{}",
        id.get_version_num()
    );
}

#[test]
fn collection_metrics_uuid_handles_empty_and_unicode() {
    // Edge cases that previously went through `new_v4` without a problem
    // and must keep round-tripping to themselves under v5.
    assert_eq!(
        collection_metrics_uuid(""),
        collection_metrics_uuid(""),
        "empty name must be stable"
    );
    assert_eq!(
        collection_metrics_uuid("coleção"),
        collection_metrics_uuid("coleção"),
        "unicode name must be stable"
    );
    assert_ne!(
        collection_metrics_uuid(""),
        collection_metrics_uuid("coleção"),
        "empty and unicode must collide only by accident, which v5 avoids"
    );
}

// --- phase9: validate_client_id contract --------------------------------

use super::insert::{MAX_CLIENT_ID_LEN, validate_client_id};

#[test]
fn validate_client_id_accepts_typical_ids() {
    // Real-world ids the user reported: `camara:2257511`, `doc:42`,
    // ISO-timestamped slugs, slash paths.
    for id in [
        "doc:42",
        "camara:2257511",
        "camara:2023-07-06T16:32",
        "tenant/org/document-9",
        "abc123",
        "x",
    ] {
        validate_client_id(id).unwrap_or_else(|e| panic!("expected '{id}' to be valid, got: {e}"));
    }
}

#[test]
fn validate_client_id_rejects_empty() {
    let err = validate_client_id("").unwrap_err();
    assert!(err.contains("empty"), "unexpected error message: {err}");
}

#[test]
fn validate_client_id_rejects_chunk_separator() {
    // `#` is reserved for the `parent#chunk_index` chunk-id derivation;
    // accepting it from clients would let two distinct documents collide
    // on the same vector id once chunked.
    let err = validate_client_id("doc#42").unwrap_err();
    assert!(err.contains('#'), "unexpected error message: {err}");
}

#[test]
fn validate_client_id_rejects_edge_whitespace() {
    for id in [" doc:42", "doc:42 ", "\tdoc:42", "doc:42\n"] {
        let err = validate_client_id(id)
            .unwrap_err_or_else_panic_if_unexpected("whitespace at edges must be rejected");
        assert!(
            err.contains("whitespace"),
            "unexpected error message for {id:?}: {err}"
        );
    }
}

#[test]
fn validate_client_id_rejects_overlong() {
    let too_long = "x".repeat(MAX_CLIENT_ID_LEN + 1);
    let err = validate_client_id(&too_long).unwrap_err();
    assert!(
        err.contains("characters"),
        "unexpected error message: {err}"
    );

    // Boundary: exactly MAX_CLIENT_ID_LEN must be accepted.
    let at_limit = "x".repeat(MAX_CLIENT_ID_LEN);
    validate_client_id(&at_limit).expect("id at the limit must be accepted");
}

trait ResultUnwrapErrPanic<E> {
    fn unwrap_err_or_else_panic_if_unexpected(self, msg: &str) -> E;
}

impl<T: std::fmt::Debug, E> ResultUnwrapErrPanic<E> for Result<T, E> {
    fn unwrap_err_or_else_panic_if_unexpected(self, msg: &str) -> E {
        match self {
            Ok(v) => panic!("{msg}: got Ok({v:?})"),
            Err(e) => e,
        }
    }
}

// --- phase9: chunk payload shape ----------------------------------------

use std::collections::HashMap;

use super::insert::build_chunk_payload;

fn meta(pairs: &[(&str, &str)]) -> HashMap<String, String> {
    pairs
        .iter()
        .map(|(k, v)| ((*k).to_string(), (*v).to_string()))
        .collect()
}

#[test]
fn chunk_payload_is_flat_at_root() {
    let user = meta(&[
        ("_id", "camara:2023-07-06T16:32"),
        ("casa", "camara"),
        ("parlamentar", "Jack Rocha"),
    ]);
    let payload = build_chunk_payload("hello world", "text_input", 0, "doc:42", &user);

    let obj = payload.as_object().expect("payload is an object");

    // Server-provided fields at root.
    assert_eq!(obj["content"].as_str(), Some("hello world"));
    assert_eq!(obj["file_path"].as_str(), Some("text_input"));
    assert_eq!(obj["chunk_index"].as_u64(), Some(0));
    assert_eq!(obj["parent_id"].as_str(), Some("doc:42"));

    // User metadata at root, not under a `metadata` sub-object.
    assert_eq!(obj["casa"].as_str(), Some("camara"));
    assert_eq!(obj["parlamentar"].as_str(), Some("Jack Rocha"));
    assert_eq!(obj["_id"].as_str(), Some("camara:2023-07-06T16:32"));

    assert!(
        obj.get("metadata").is_none(),
        "phase9 chunk payloads must not nest user fields under `metadata`"
    );
}

#[test]
fn chunk_payload_server_keys_win_collisions() {
    // If the user passes a `content`/`file_path`/`chunk_index`/`parent_id`
    // key in metadata, the server's value MUST win — the user's key is
    // silently dropped, never overwriting a server invariant.
    let user = meta(&[
        ("content", "USER_OVERRIDE_BAD"),
        ("file_path", "USER_OVERRIDE_BAD"),
        ("chunk_index", "USER_OVERRIDE_BAD"),
        ("parent_id", "USER_OVERRIDE_BAD"),
        ("ok", "kept"),
    ]);
    let payload = build_chunk_payload("real content", "real_path.txt", 7, "doc:9", &user);

    let obj = payload.as_object().unwrap();
    assert_eq!(obj["content"].as_str(), Some("real content"));
    assert_eq!(obj["file_path"].as_str(), Some("real_path.txt"));
    assert_eq!(obj["chunk_index"].as_u64(), Some(7));
    assert_eq!(obj["parent_id"].as_str(), Some("doc:9"));
    assert_eq!(obj["ok"].as_str(), Some("kept"));
}

#[test]
fn chunk_payload_handles_empty_user_metadata() {
    let payload = build_chunk_payload("c", "p", 0, "doc:1", &HashMap::new());
    let obj = payload.as_object().unwrap();
    // Exactly four server keys, no extras.
    assert_eq!(obj.len(), 4);
    assert!(obj.contains_key("content"));
    assert!(obj.contains_key("file_path"));
    assert!(obj.contains_key("chunk_index"));
    assert!(obj.contains_key("parent_id"));
}

// --- phase9: /insert_vectors payload assembly ---------------------------

use serde_json::json;

use super::insert::build_vector_payload;

#[test]
fn vector_payload_returns_explicit_payload_verbatim() {
    let entry = json!({
        "id": "doc:1",
        "embedding": [0.1, 0.2],
        "payload": {
            "casa": "camara",
            "deeply": {"nested": [1, 2, 3]},
            "n": 42
        }
    });
    let p = build_vector_payload(&entry);
    let obj = p.as_object().expect("payload is an object");
    assert_eq!(obj["casa"].as_str(), Some("camara"));
    assert_eq!(obj["n"].as_u64(), Some(42));
    assert_eq!(obj["deeply"]["nested"][2].as_u64(), Some(3));
}

#[test]
fn vector_payload_falls_back_to_metadata_when_payload_absent() {
    let entry = json!({
        "id": "doc:1",
        "embedding": [0.1, 0.2],
        "metadata": {"casa": "camara", "ano": "2020"}
    });
    let p = build_vector_payload(&entry);
    let obj = p.as_object().unwrap();
    assert_eq!(obj["casa"].as_str(), Some("camara"));
    assert_eq!(obj["ano"].as_str(), Some("2020"));
}

#[test]
fn vector_payload_prefers_payload_over_metadata() {
    let entry = json!({
        "id": "doc:1",
        "embedding": [0.1, 0.2],
        "payload": {"from_payload": "yes"},
        "metadata": {"from_metadata": "ignored"}
    });
    let p = build_vector_payload(&entry);
    let obj = p.as_object().unwrap();
    assert_eq!(obj["from_payload"].as_str(), Some("yes"));
    assert!(
        !obj.contains_key("from_metadata"),
        "metadata must be ignored when payload is provided"
    );
}

#[test]
fn vector_payload_is_empty_object_when_neither_present() {
    let entry = json!({"id": "doc:1", "embedding": [0.1]});
    let p = build_vector_payload(&entry);
    assert_eq!(p.as_object().map(|o| o.len()), Some(0));
}
