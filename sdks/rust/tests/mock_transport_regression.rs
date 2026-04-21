#![allow(warnings)]
#![allow(clippy::unwrap_used, clippy::expect_used)]
#![allow(clippy::absurd_extreme_comparisons, clippy::nonminimal_bool)]

//! RPC-readiness regression guard for the per-surface client split.
//!
//! Constructs `VectorizerClient` from an in-memory mock `Transport`
//! (no `reqwest`, no actual HTTP) and pokes a method from each of
//! the eight per-surface impl files. If any per-surface module
//! accidentally hard-codes `HttpTransport` or `reqwest::Client`,
//! one of these calls stops compiling — proving the surface
//! modules are decoupled from the concrete REST backend.
//!
//! This is the **phase 4 task 2.4 + 3.2 RPC-readiness regression
//! guard**: when `phase6_sdk-rust-rpc` plugs an `RpcTransport` into
//! the same `Transport` trait, every per-surface call will route
//! through it without a single per-method edit. This test pins
//! that contract.

#![cfg(feature = "http")]
#![allow(clippy::unwrap_used)]

use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use serde_json::Value;
use vectorizer_sdk::error::Result;
use vectorizer_sdk::transport::{Protocol, Transport};
use vectorizer_sdk::{ReadOptions, VectorizerClient};

/// Mock `Transport` that records every call and returns a canned
/// JSON body. The `responses` lookup is keyed by `<method> <path>`.
struct MockTransport {
    responses: Mutex<std::collections::HashMap<String, String>>,
    call_count: AtomicUsize,
}

impl MockTransport {
    fn new() -> Self {
        Self {
            responses: Mutex::new(std::collections::HashMap::new()),
            call_count: AtomicUsize::new(0),
        }
    }

    fn with_response(self, method: &str, path: &str, body: &str) -> Self {
        self.responses
            .lock()
            .unwrap()
            .insert(format!("{method} {path}"), body.to_string());
        self
    }

    fn calls(&self) -> usize {
        self.call_count.load(Ordering::SeqCst)
    }

    fn dispatch(&self, method: &str, path: &str) -> Result<String> {
        self.call_count.fetch_add(1, Ordering::SeqCst);
        let key = format!("{method} {path}");
        Ok(self
            .responses
            .lock()
            .unwrap()
            .get(&key)
            .cloned()
            .unwrap_or_else(|| "{}".to_string()))
    }
}

#[async_trait]
impl Transport for MockTransport {
    async fn get(&self, path: &str) -> Result<String> {
        self.dispatch("GET", path)
    }
    async fn post(&self, path: &str, _data: Option<&Value>) -> Result<String> {
        self.dispatch("POST", path)
    }
    async fn put(&self, path: &str, _data: Option<&Value>) -> Result<String> {
        self.dispatch("PUT", path)
    }
    async fn delete(&self, path: &str) -> Result<String> {
        self.dispatch("DELETE", path)
    }
    fn protocol(&self) -> Protocol {
        Protocol::Http
    }
}

#[tokio::test]
async fn collections_surface_routes_through_mock() {
    let mock = Arc::new(MockTransport::new().with_response(
        "GET",
        "/collections",
        r#"{"collections":[]}"#,
    ));
    let calls_before = mock.calls();
    let client = VectorizerClient::with_transport(mock.clone(), "http://mock");
    let cols = client.list_collections().await.unwrap();
    assert!(cols.is_empty());
    assert_eq!(mock.calls(), calls_before + 1);
}

#[tokio::test]
async fn core_surface_routes_through_mock() {
    let mock = Arc::new(MockTransport::new().with_response(
        "GET",
        "/health",
        r#"{"status":"healthy","timestamp":"2026-04-19T00:00:00Z","service":"v","version":"3.0.0"}"#,
    ));
    let client = VectorizerClient::with_transport(mock.clone(), "http://mock");
    let _ = client.health_check().await.unwrap();
    assert_eq!(mock.calls(), 1);
}

#[tokio::test]
async fn vectors_surface_routes_through_mock() {
    let mock = Arc::new(MockTransport::new().with_response(
        "POST",
        "/embed",
        r#"{"embedding":[0.1,0.2],"dimension":2,"model":"test","provider":"test","text":"hello"}"#,
    ));
    let client = VectorizerClient::with_transport(mock.clone(), "http://mock");
    let resp = client.embed_text("hello", None).await.unwrap();
    assert_eq!(resp.dimension, 2);
}

#[tokio::test]
async fn search_surface_routes_through_mock() {
    let mock = Arc::new(MockTransport::new().with_response(
        "POST",
        "/collections/c/search/text",
        r#"{"results":[],"query_time_ms":0.5}"#,
    ));
    let client = VectorizerClient::with_transport(mock.clone(), "http://mock");
    let resp = client.search_vectors("c", "q", None, None).await.unwrap();
    assert!(resp.results.is_empty());
}

#[tokio::test]
async fn discovery_surface_routes_through_mock() {
    let mock = Arc::new(MockTransport::new().with_response("POST", "/discover", r#"{"ok":true}"#));
    let client = VectorizerClient::with_transport(mock.clone(), "http://mock");
    let _ = client
        .discover("q", None, None, None, None, None)
        .await
        .unwrap();
    assert_eq!(mock.calls(), 1);
}

#[tokio::test]
async fn files_surface_routes_through_mock() {
    let mock = Arc::new(MockTransport::new().with_response(
        "POST",
        "/file/content",
        r#"{"content":"x","file_path":"f","collection":"c"}"#,
    ));
    let client = VectorizerClient::with_transport(mock.clone(), "http://mock");
    let _ = client.get_file_content("c", "f", None).await.unwrap();
    assert_eq!(mock.calls(), 1);
}

#[tokio::test]
async fn graph_surface_routes_through_mock() {
    let mock = Arc::new(MockTransport::new().with_response(
        "GET",
        "/graph/nodes/c",
        r#"{"nodes":[],"count":0}"#,
    ));
    let client = VectorizerClient::with_transport(mock.clone(), "http://mock");
    let resp = client.list_graph_nodes("c").await.unwrap();
    assert_eq!(resp.count, 0);
}

#[tokio::test]
async fn qdrant_surface_routes_through_mock() {
    let mock = Arc::new(MockTransport::new().with_response(
        "GET",
        "/qdrant/collections",
        r#"{"result":{"collections":[]},"status":"ok","time":0}"#,
    ));
    let client = VectorizerClient::with_transport(mock.clone(), "http://mock");
    let _ = client.qdrant_list_collections().await.unwrap();
    assert_eq!(mock.calls(), 1);
}

#[tokio::test]
async fn read_options_dont_break_mock_dispatch() {
    // Smoke test for the read-options plumbing: a custom transport
    // that knows nothing about replicas should still serve calls
    // when the client is built without master/replica wiring.
    let mock = Arc::new(MockTransport::new().with_response(
        "GET",
        "/collections/c",
        r#"{"name":"c","dimension":3,"metric":"cosine","vector_count":0,"document_count":0,"created_at":"","updated_at":""}"#,
    ));
    let client = VectorizerClient::with_transport(mock.clone(), "http://mock");
    let info = client.get_collection_info("c").await.unwrap();
    assert_eq!(info.name, "c");
    let _ = ReadOptions::default(); // ensure the public type is reachable
}
