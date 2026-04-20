#![allow(warnings)]
#![allow(clippy::unwrap_used, clippy::expect_used)]
#![allow(clippy::absurd_extreme_comparisons, clippy::nonminimal_bool)]

//! End-to-end integration test for the SDK's RPC client.
//!
//! Spins up an in-test server on `127.0.0.1:0` that speaks the
//! VectorizerRPC wire format using the SDK's own codec + types
//! (because the server crate isn't a dev-dependency of this SDK), and
//! drives it from `RpcClient` to prove:
//!
//! - HELLO handshake produces the expected `HelloResponse` shape.
//! - `PING` works pre-HELLO (auth-exempt per wire spec § 4).
//! - A data-plane command (`collections.list`) before HELLO returns
//!   `RpcClientError::NotAuthenticated`.
//! - Two concurrent calls on the same connection get correctly
//!   demultiplexed by `Request.id`.
//! - The typed wrappers (`list_collections`, `get_collection_info`,
//!   `search_basic`) round-trip through the codec.
//! - `RpcClient::connect_url` accepts every documented URL form and
//!   rejects REST URLs with a clear error.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use std::sync::Arc;
use std::time::Duration;

use tokio::io::BufReader;
use tokio::net::TcpListener;

use vectorizer_sdk::rpc::codec::{read_request, write_response};
use vectorizer_sdk::rpc::types::{Request, Response, VectorizerValue};
use vectorizer_sdk::rpc::{HelloPayload, RpcClient, RpcClientError};

/// In-test server that mimics the production dispatcher closely
/// enough to exercise the SDK's wire layer end-to-end.
async fn spawn_fake_server() -> std::net::SocketAddr {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        loop {
            let (stream, _peer) = match listener.accept().await {
                Ok(x) => x,
                Err(_) => break,
            };
            tokio::spawn(handle_connection(stream));
        }
    });
    // Give the listener a moment to start.
    tokio::time::sleep(Duration::from_millis(20)).await;
    addr
}

async fn handle_connection(stream: tokio::net::TcpStream) {
    let (read_half, mut write_half) = stream.into_split();
    let mut reader = BufReader::new(read_half);
    let authenticated = Arc::new(parking_lot::RwLock::new(false));

    loop {
        let req: Request = match read_request(&mut reader).await {
            Ok(r) => r,
            Err(_) => break,
        };
        let resp = dispatch(&req, &authenticated).await;
        if write_response(&mut write_half, &resp).await.is_err() {
            break;
        }
    }
}

async fn dispatch(req: &Request, authenticated: &Arc<parking_lot::RwLock<bool>>) -> Response {
    match req.command.as_str() {
        "HELLO" => {
            *authenticated.write() = true;
            Response::ok(
                req.id,
                VectorizerValue::Map(vec![
                    (
                        VectorizerValue::Str("server_version".into()),
                        VectorizerValue::Str("test-fixture/0.0.0".into()),
                    ),
                    (
                        VectorizerValue::Str("protocol_version".into()),
                        VectorizerValue::Int(1),
                    ),
                    (
                        VectorizerValue::Str("authenticated".into()),
                        VectorizerValue::Bool(true),
                    ),
                    (
                        VectorizerValue::Str("admin".into()),
                        VectorizerValue::Bool(true),
                    ),
                    (
                        VectorizerValue::Str("capabilities".into()),
                        VectorizerValue::Array(vec![
                            VectorizerValue::Str("PING".into()),
                            VectorizerValue::Str("collections.list".into()),
                            VectorizerValue::Str("collections.get_info".into()),
                            VectorizerValue::Str("vectors.get".into()),
                            VectorizerValue::Str("search.basic".into()),
                        ]),
                    ),
                ]),
            )
        }
        "PING" => Response::ok(req.id, VectorizerValue::Str("PONG".into())),
        // Auth gate for data-plane commands — mirrors the production
        // server's behaviour described in wire spec § 4.
        cmd if !*authenticated.read() => Response::err(
            req.id,
            format!("authentication required: send HELLO first ({cmd})"),
        ),
        "collections.list" => Response::ok(
            req.id,
            VectorizerValue::Array(vec![
                VectorizerValue::Str("alpha-docs".into()),
                VectorizerValue::Str("beta-source".into()),
            ]),
        ),
        "collections.get_info" => {
            let name = req
                .args
                .first()
                .and_then(|v| v.as_str())
                .unwrap_or("unknown");
            Response::ok(
                req.id,
                VectorizerValue::Map(vec![
                    (
                        VectorizerValue::Str("name".into()),
                        VectorizerValue::Str(name.to_owned()),
                    ),
                    (
                        VectorizerValue::Str("vector_count".into()),
                        VectorizerValue::Int(42),
                    ),
                    (
                        VectorizerValue::Str("document_count".into()),
                        VectorizerValue::Int(10),
                    ),
                    (
                        VectorizerValue::Str("dimension".into()),
                        VectorizerValue::Int(384),
                    ),
                    (
                        VectorizerValue::Str("metric".into()),
                        VectorizerValue::Str("Cosine".into()),
                    ),
                    (
                        VectorizerValue::Str("created_at".into()),
                        VectorizerValue::Str("2026-04-19T00:00:00Z".into()),
                    ),
                    (
                        VectorizerValue::Str("updated_at".into()),
                        VectorizerValue::Str("2026-04-19T00:00:00Z".into()),
                    ),
                ]),
            )
        }
        "search.basic" => Response::ok(
            req.id,
            VectorizerValue::Array(vec![
                VectorizerValue::Map(vec![
                    (
                        VectorizerValue::Str("id".into()),
                        VectorizerValue::Str("vec-0".into()),
                    ),
                    (
                        VectorizerValue::Str("score".into()),
                        VectorizerValue::Float(0.95),
                    ),
                    (
                        VectorizerValue::Str("payload".into()),
                        VectorizerValue::Str(r#"{"title":"hit one"}"#.into()),
                    ),
                ]),
                VectorizerValue::Map(vec![
                    (
                        VectorizerValue::Str("id".into()),
                        VectorizerValue::Str("vec-1".into()),
                    ),
                    (
                        VectorizerValue::Str("score".into()),
                        VectorizerValue::Float(0.81),
                    ),
                ]),
            ]),
        ),
        other => Response::err(req.id, format!("unknown command '{other}'")),
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn hello_then_ping_then_typed_commands() {
    let addr = spawn_fake_server().await;
    let client = RpcClient::connect(addr.to_string()).await.unwrap();

    // PING is auth-exempt per wire spec § 4.
    let pong = client.ping().await.unwrap();
    assert_eq!(pong, "PONG");

    // HELLO completes the handshake.
    let hello = client
        .hello(HelloPayload::new("rpc-integration-test"))
        .await
        .unwrap();
    assert!(hello.authenticated);
    assert!(hello.admin);
    assert_eq!(hello.protocol_version, 1);
    assert_eq!(hello.server_version, "test-fixture/0.0.0");
    assert!(hello.capabilities.contains(&"collections.list".to_owned()));

    // Typed wrappers.
    let cols = client.list_collections().await.unwrap();
    assert_eq!(
        cols,
        vec!["alpha-docs".to_owned(), "beta-source".to_owned()]
    );

    let info = client.get_collection_info("alpha-docs").await.unwrap();
    assert_eq!(info.name, "alpha-docs");
    assert_eq!(info.vector_count, 42);
    assert_eq!(info.dimension, 384);
    assert_eq!(info.metric, "Cosine");

    let hits = client
        .search_basic("alpha-docs", "anything", 10)
        .await
        .unwrap();
    assert_eq!(hits.len(), 2);
    assert_eq!(hits[0].id, "vec-0");
    assert!((hits[0].score - 0.95).abs() < 1e-9);
    assert_eq!(hits[0].payload.as_deref(), Some(r#"{"title":"hit one"}"#));
    assert_eq!(hits[1].id, "vec-1");
    assert!(hits[1].payload.is_none());
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn data_plane_call_before_hello_is_rejected_locally() {
    let addr = spawn_fake_server().await;
    let client = RpcClient::connect(addr.to_string()).await.unwrap();

    // The SDK's local auth gate fails fast before even sending the
    // request — saves an unnecessary round-trip.
    let err = client.list_collections().await.unwrap_err();
    match err {
        RpcClientError::NotAuthenticated => {}
        other => panic!("expected NotAuthenticated, got {other:?}"),
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn concurrent_calls_on_one_connection_are_demultiplexed_by_id() {
    let addr = spawn_fake_server().await;
    let client = Arc::new(RpcClient::connect(addr.to_string()).await.unwrap());
    client
        .hello(HelloPayload::new("concurrent-test"))
        .await
        .unwrap();

    // Fire 16 list_collections in parallel; every call must get the
    // right shape back. If response demultiplexing were broken,
    // calls would either hang (no response) or get the wrong payload.
    let mut handles = Vec::new();
    for _ in 0..16 {
        let c = Arc::clone(&client);
        handles.push(tokio::spawn(async move { c.list_collections().await }));
    }
    for h in handles {
        let cols = h.await.unwrap().unwrap();
        assert_eq!(cols.len(), 2);
        assert_eq!(cols[0], "alpha-docs");
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn connect_url_accepts_canonical_vectorizer_scheme() {
    let addr = spawn_fake_server().await;
    let url = format!("vectorizer://{}", addr);
    let client = RpcClient::connect_url(&url).await.unwrap();
    let pong = client.ping().await.unwrap();
    assert_eq!(pong, "PONG");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn connect_url_rejects_http_scheme_with_clear_error() {
    let result = RpcClient::connect_url("http://localhost:15002").await;
    let err = match result {
        Err(e) => e,
        Ok(_) => panic!("connect_url with http scheme must fail"),
    };
    let msg = err.to_string();
    assert!(
        msg.contains("REST URL") && msg.contains("HTTP client"),
        "expected the error to point the caller at the HTTP client; got: {msg}"
    );
}
