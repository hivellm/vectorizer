#![allow(warnings)]
#![allow(clippy::unwrap_used, clippy::expect_used)]
#![allow(clippy::absurd_extreme_comparisons, clippy::nonminimal_bool)]

//! End-to-end integration test for the SDK's RPC client.
//!
//! Stands up an in-test Thunder listener on `127.0.0.1:0` — the same server
//! half `vectorizer-server` runs, with a fake dispatch table standing in for
//! the engine — and drives it from `RpcClient` to prove:
//!
//! - HELLO produces the expected `HelloResponse` shape.
//! - `PING` works pre-HELLO (auth-exempt per wire spec § 4).
//! - A data-plane command on an un-credentialed session against an
//!   auth-enabled server returns `RpcClientError::NotAuthenticated`.
//! - `hello` with a token authenticates the session, so later commands pass
//!   the gate.
//! - Two concurrent calls on the same connection get correctly
//!   demultiplexed by frame id.
//! - The typed wrappers (`list_collections`, `get_collection_info`,
//!   `search_basic`) round-trip over the wire.
//! - `RpcClient::connect_url` accepts every documented URL form and
//!   rejects REST URLs with a clear error.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use std::sync::Arc;

use thunder::Value;
use thunder::server::{
    AuthError, Credentials, Dispatch, ListenerConfig, ListenerHandle, Principal, ServerInfo,
    Session, spawn_listener,
};
use vectorizer_sdk::rpc::{HelloPayload, RpcClient, RpcClientError, protocol_config};

/// The one credential the fake server accepts.
const GOOD_TOKEN: &str = "good-token";

/// In-test dispatch table that mimics the production server closely
/// enough to exercise the SDK's wire layer end-to-end.
struct FakeVectorizer;

impl Dispatch for FakeVectorizer {
    type Identity = ();

    async fn dispatch(
        &self,
        _session: &Session<()>,
        command: &str,
        args: Vec<Value>,
    ) -> Result<Value, String> {
        match command {
            // `HelloStyle::NotUsed`: HELLO is Vectorizer's own command, so it
            // reaches the product dispatch exactly as on the real server.
            "HELLO" => Ok(Value::Map(vec![
                (
                    Value::Str("server_version".into()),
                    Value::Str("test-fixture/0.0.0".into()),
                ),
                (Value::Str("protocol_version".into()), Value::Int(1)),
                (Value::Str("authenticated".into()), Value::Bool(true)),
                (Value::Str("admin".into()), Value::Bool(true)),
                (
                    Value::Str("capabilities".into()),
                    Value::Array(vec![
                        Value::Str("PING".into()),
                        Value::Str("collections.list".into()),
                        Value::Str("collections.get_info".into()),
                        Value::Str("vectors.get".into()),
                        Value::Str("search.basic".into()),
                    ]),
                ),
            ])),
            "PING" => Ok(Value::Str("PONG".into())),
            "collections.list" => Ok(Value::Array(vec![
                Value::Str("alpha-docs".into()),
                Value::Str("beta-source".into()),
            ])),
            "collections.get_info" => {
                let name = args.first().and_then(|v| v.as_str()).unwrap_or("unknown");
                Ok(Value::Map(vec![
                    (Value::Str("name".into()), Value::Str(name.to_owned())),
                    (Value::Str("vector_count".into()), Value::Int(42)),
                    (Value::Str("document_count".into()), Value::Int(10)),
                    (Value::Str("dimension".into()), Value::Int(384)),
                    (Value::Str("metric".into()), Value::Str("Cosine".into())),
                    (
                        Value::Str("created_at".into()),
                        Value::Str("2026-04-19T00:00:00Z".into()),
                    ),
                    (
                        Value::Str("updated_at".into()),
                        Value::Str("2026-04-19T00:00:00Z".into()),
                    ),
                ]))
            }
            "search.basic" => Ok(Value::Array(vec![
                Value::Map(vec![
                    (Value::Str("id".into()), Value::Str("vec-0".into())),
                    (Value::Str("score".into()), Value::Float(0.95)),
                    (
                        Value::Str("payload".into()),
                        Value::Str(r#"{"title":"hit one"}"#.into()),
                    ),
                ]),
                Value::Map(vec![
                    (Value::Str("id".into()), Value::Str("vec-1".into())),
                    (Value::Str("score".into()), Value::Float(0.81)),
                ]),
            ])),
            other => Err(format!("unknown command '{other}'")),
        }
    }

    async fn authenticate(&self, creds: Credentials) -> Result<Principal<()>, AuthError> {
        let secret = match creds {
            Credentials::Token(t) | Credentials::ApiKey(t) => t,
            Credentials::UserPass(_, pass) => pass,
            Credentials::None => return Err(AuthError::InvalidCredentials),
        };
        if secret == GOOD_TOKEN {
            Ok(Principal::new("fake-admin"))
        } else {
            Err(AuthError::InvalidCredentials)
        }
    }
}

/// Boot the fake server. `auth_required` mirrors the deployment posture:
/// `false` opens the listener (single-user mode), `true` makes Thunder refuse
/// un-credentialed sessions.
///
/// The returned handle must stay alive for the test — dropping it stops the
/// accept loop.
async fn spawn_fake_server(auth_required: bool) -> (ListenerHandle, String) {
    let mut config = ListenerConfig::new("127.0.0.1:0".parse().unwrap());
    if !auth_required {
        config = config.open();
    }
    let info = ServerInfo {
        name: "vectorizer-test-fixture".to_owned(),
        version: "0.0.0".to_owned(),
    };
    let handle = spawn_listener(Arc::new(FakeVectorizer), protocol_config(), info, config)
        .await
        .unwrap();
    let endpoint = format!("vectorizer://{}", handle.local_addr());
    (handle, endpoint)
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn hello_then_ping_then_typed_commands() {
    let (_handle, endpoint) = spawn_fake_server(false).await;
    let client = RpcClient::connect(&endpoint).await.unwrap();

    // PING is auth-exempt per wire spec § 4.
    let pong = client.ping().await.unwrap();
    assert_eq!(pong, "PONG");

    // HELLO reports the server's capabilities and auth flags.
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
async fn data_plane_call_without_credentials_is_rejected() {
    let (_handle, endpoint) = spawn_fake_server(true).await;
    let client = RpcClient::connect(&endpoint).await.unwrap();
    assert!(!client.is_authenticated());

    // The session sent no `AUTH`, so the server gates the command before it
    // reaches the dispatch table.
    let err = client.list_collections().await.unwrap_err();
    match err {
        RpcClientError::NotAuthenticated(_) => {}
        other => panic!("expected NotAuthenticated, got {other:?}"),
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn hello_with_token_authenticates_the_session() {
    let (_handle, endpoint) = spawn_fake_server(true).await;
    let client = RpcClient::connect(&endpoint).await.unwrap();

    // A credential-carrying HELLO re-dials so the token travels in Thunder's
    // `AUTH` handshake; the session is authenticated from there on.
    let hello = client
        .hello(HelloPayload::new("rpc-integration-test").with_token(GOOD_TOKEN))
        .await
        .unwrap();
    assert!(hello.authenticated);
    assert!(client.is_authenticated());

    let cols = client.list_collections().await.unwrap();
    assert_eq!(cols.len(), 2);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn bad_credentials_fail_the_handshake() {
    let (_handle, endpoint) = spawn_fake_server(true).await;
    let client = RpcClient::connect(&endpoint).await.unwrap();

    let err = client
        .hello(HelloPayload::new("rpc-integration-test").with_token("wrong-token"))
        .await
        .unwrap_err();
    match err {
        RpcClientError::NotAuthenticated(_) => {}
        other => panic!("expected NotAuthenticated, got {other:?}"),
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn concurrent_calls_on_one_connection_are_demultiplexed_by_id() {
    let (_handle, endpoint) = spawn_fake_server(false).await;
    let client = Arc::new(RpcClient::connect(&endpoint).await.unwrap());
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
    let (_handle, endpoint) = spawn_fake_server(false).await;
    let client = RpcClient::connect_url(&endpoint).await.unwrap();
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
