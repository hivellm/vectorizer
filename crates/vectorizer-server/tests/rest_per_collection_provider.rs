//! Per-collection embedding providers (3.8).
//!
//! Before 3.8 every text insert and text search embedded with the server's
//! default provider, whatever `embedding_provider` the collection was created
//! with. A server whose default is BM25-512 could therefore not host a
//! `fastembed:multilingual-e5-small` (384) collection: the create succeeded,
//! then every text insert produced a 512-wide vector the collection rejected.
//!
//! These tests register a second, deterministic provider next to the default
//! `bm25` — the shape a server gets from `embedding.additional_models` — and
//! pin that text operations on a collection created with it go through *that*
//! provider (documents via `embed`, queries via `embed_query`), while a BM25
//! collection on the same server keeps using BM25. Loading a real ONNX model
//! is impractical here, so the extra provider is a recording stand-in.

#![allow(clippy::unwrap_used, clippy::expect_used)]

mod common;

use std::sync::{Arc, Mutex};

use axum::http::StatusCode;
use common::TestApp;
use serde_json::json;
use vectorizer::embedding::EmbeddingProvider;
use vectorizer::error::Result;

/// Name the stand-in provider is registered under — the same shape
/// `embedding.additional_models` registers fastembed models with.
const EXTRA: &str = "fastembed:test-multilingual";
/// Width of the stand-in provider; deliberately not BM25's 512.
const EXTRA_DIM: usize = 8;

/// Which entry point served a call, and with what text.
type Calls = Arc<Mutex<Vec<(&'static str, String)>>>;

/// Deterministic provider that records every call. Documents and queries
/// map to the same vector for the same text (so a query equal to a stored
/// document finds it), but are recorded separately so a test can tell the
/// query path from the document path.
struct RecordingProvider {
    calls: Calls,
}

fn vector_for(text: &str) -> Vec<f32> {
    let mut v = [0.0_f32; EXTRA_DIM];
    for (i, b) in text.bytes().enumerate() {
        v[(b as usize + i) % EXTRA_DIM] += 1.0;
    }
    let norm = v
        .iter()
        .map(|x| x * x)
        .sum::<f32>()
        .sqrt()
        .max(f32::EPSILON);
    v.iter().map(|x| x / norm).collect()
}

impl EmbeddingProvider for RecordingProvider {
    fn embed_batch(&self, texts: &[&str]) -> Result<Vec<Vec<f32>>> {
        let mut calls = self.calls.lock().unwrap();
        Ok(texts
            .iter()
            .map(|t| {
                calls.push(("passage", (*t).to_string()));
                vector_for(t)
            })
            .collect())
    }

    fn embed_query_batch(&self, texts: &[&str]) -> Result<Vec<Vec<f32>>> {
        let mut calls = self.calls.lock().unwrap();
        Ok(texts
            .iter()
            .map(|t| {
                calls.push(("query", (*t).to_string()));
                vector_for(t)
            })
            .collect())
    }

    fn embed_query(&self, text: &str) -> Result<Vec<f32>> {
        Ok(self.embed_query_batch(&[text])?.remove(0))
    }

    fn dimension(&self) -> usize {
        EXTRA_DIM
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

async fn app_with_extra_provider() -> (TestApp, Arc<vectorizer::VectorStore>, Calls) {
    let calls: Calls = Arc::default();
    let provider = RecordingProvider {
        calls: Arc::clone(&calls),
    };
    let (app, store) =
        TestApp::with_additional_providers(vec![(EXTRA.to_string(), Box::new(provider))]).await;
    (app, store, calls)
}

async fn create(app: &TestApp, name: &str, dimension: usize, provider: &str) {
    let (status, body) = app
        .post_json(
            "/collections",
            json!({
                "name": name,
                "dimension": dimension,
                "metric": "cosine",
                "embedding_provider": provider,
            }),
        )
        .await;
    assert!(status.is_success(), "create {name}: {status} {body}");
}

fn stored_width(store: &vectorizer::VectorStore, collection: &str, id: &str) -> usize {
    store.get_vector(collection, id).unwrap().data.len()
}

#[tokio::test]
async fn text_insert_and_search_use_the_collections_provider() {
    let (app, store, calls) = app_with_extra_provider().await;
    create(&app, "pt_docs", EXTRA_DIM, EXTRA).await;
    create(&app, "en_docs", 512, "bm25").await;

    // Insert into the non-default collection: embedded as a document by the
    // collection's provider, at the collection's width.
    let doc = "como cancelar meu pedido";
    let (status, body) = app
        .post_json("/insert", json!({"collection": "pt_docs", "text": doc}))
        .await;
    assert!(status.is_success(), "insert: {status} {body}");
    let pt_id = body["vector_ids"][0].as_str().unwrap().to_string();
    assert_eq!(stored_width(&store, "pt_docs", &pt_id), EXTRA_DIM);
    assert_eq!(
        *calls.lock().unwrap(),
        vec![("passage", doc.to_string())],
        "the text must be embedded once, as a document, by the collection's provider"
    );

    // Text search: embedded as a query by the same provider, and finds it.
    let (status, body) = app
        .post_json(
            "/collections/pt_docs/search/text",
            json!({"query": doc, "limit": 3}),
        )
        .await;
    assert!(status.is_success(), "search: {status} {body}");
    assert_eq!(body["results"][0]["id"], pt_id.as_str(), "{body}");
    assert_eq!(
        calls.lock().unwrap().last().cloned(),
        Some(("query", doc.to_string())),
        "search must embed through the query entry point"
    );

    // The BM25 collection on the same server is untouched: default provider,
    // BM25 width, and the extra provider sees none of its traffic.
    let before = calls.lock().unwrap().len();
    let (status, body) = app
        .post_json(
            "/insert",
            json!({"collection": "en_docs", "text": "vector databases store embeddings"}),
        )
        .await;
    assert!(status.is_success(), "bm25 insert: {status} {body}");
    let en_id = body["vector_ids"][0].as_str().unwrap().to_string();
    assert_eq!(stored_width(&store, "en_docs", &en_id), 512);
    let (status, body) = app
        .post_json(
            "/collections/en_docs/search/text",
            json!({"query": "vector databases", "limit": 3}),
        )
        .await;
    assert!(status.is_success(), "bm25 search: {status} {body}");
    assert_eq!(body["results"][0]["id"], en_id.as_str(), "{body}");
    assert_eq!(calls.lock().unwrap().len(), before);
}

#[tokio::test]
async fn hybrid_and_batch_search_use_the_collections_provider() {
    let (app, _store, calls) = app_with_extra_provider().await;
    create(&app, "pt_hybrid", EXTRA_DIM, EXTRA).await;
    let (status, body) = app
        .post_json(
            "/insert",
            json!({"collection": "pt_hybrid", "text": "reembolso do pagamento"}),
        )
        .await;
    assert!(status.is_success(), "insert: {status} {body}");

    // A default-provider (512) query vector would fail against this 8-wide
    // collection, so success here already rules out the old behavior; the
    // recorder pins that the query entry point was the one used.
    let (status, body) = app
        .post_json(
            "/collections/pt_hybrid/hybrid_search",
            json!({"query": "reembolso", "limit": 3}),
        )
        .await;
    assert!(status.is_success(), "hybrid: {status} {body}");
    assert_eq!(
        calls.lock().unwrap().last().cloned(),
        Some(("query", "reembolso".to_string()))
    );

    let (status, body) = app
        .post_json(
            "/batch_search",
            json!({"collection": "pt_hybrid", "queries": [{"query": "pagamento"}]}),
        )
        .await;
    assert!(status.is_success(), "batch: {status} {body}");
    assert_eq!(body["results"][0]["status"], "ok", "{body}");
    assert_eq!(
        calls.lock().unwrap().last().cloned(),
        Some(("query", "pagamento".to_string()))
    );
}

#[tokio::test]
async fn create_collection_validates_against_the_additional_provider() {
    let (app, _store, _calls) = app_with_extra_provider().await;

    // Registered, so accepted at its own width...
    create(&app, "pt_ok", EXTRA_DIM, EXTRA).await;

    // ...and the dimension is checked against it, not against the default.
    let (status, body) = app
        .post_json(
            "/collections",
            json!({
                "name": "pt_wrong_width",
                "dimension": 512,
                "metric": "cosine",
                "embedding_provider": EXTRA,
            }),
        )
        .await;
    assert_eq!(status, StatusCode::BAD_REQUEST, "{body}");
    assert_eq!(body["error_type"], "provider_dimension_mismatch", "{body}");

    // The inventory lists it next to bm25 so callers can discover it.
    let (status, body) = app.get("/stats").await;
    assert_eq!(status, StatusCode::OK);
    let names: Vec<&str> = body["providers"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|p| p["name"].as_str())
        .collect();
    assert!(names.contains(&EXTRA) && names.contains(&"bm25"), "{body}");
}

#[tokio::test]
async fn raw_vector_collections_still_refuse_text() {
    let (app, _store, calls) = app_with_extra_provider().await;
    create(&app, "raw_8", EXTRA_DIM, "none").await;

    let (status, body) = app
        .post_json("/insert", json!({"collection": "raw_8", "text": "olá"}))
        .await;
    assert_eq!(status, StatusCode::BAD_REQUEST, "{body}");
    assert_eq!(body["error_type"], "collection_has_no_embedding_provider");

    let (status, body) = app
        .post_json(
            "/collections/raw_8/search/text",
            json!({"query": "olá", "limit": 1}),
        )
        .await;
    assert_eq!(status, StatusCode::BAD_REQUEST, "{body}");
    assert_eq!(body["error_type"], "collection_has_no_embedding_provider");

    // Same width as the extra provider, yet it was never consulted.
    assert!(calls.lock().unwrap().is_empty());
}

#[tokio::test]
async fn omitted_dimension_takes_the_providers_width() {
    let (app, _store, _calls) = app_with_extra_provider().await;

    // A client that names only the provider must not have to know its width:
    // before 3.8.2 an omitted dimension meant 512 and was then rejected as a
    // mismatch against any non-512 provider.
    let (status, body) = app
        .post_json(
            "/collections",
            json!({"name": "pt_auto", "embedding_provider": EXTRA}),
        )
        .await;
    assert!(status.is_success(), "create: {status} {body}");
    let (_, info) = app.get("/collections/pt_auto").await;
    assert_eq!(info["dimension"], EXTRA_DIM, "{info}");

    // No provider either: the server default (BM25) and its width.
    let (status, body) = app
        .post_json("/collections", json!({"name": "default_auto"}))
        .await;
    assert!(status.is_success(), "create: {status} {body}");
    let (_, info) = app.get("/collections/default_auto").await;
    assert_eq!(info["dimension"], 512, "{info}");

    // An explicit width that disagrees is still refused.
    let (status, body) = app
        .post_json(
            "/collections",
            json!({"name": "pt_bad", "dimension": 512, "embedding_provider": EXTRA}),
        )
        .await;
    assert_eq!(status, StatusCode::BAD_REQUEST, "{body}");
}
