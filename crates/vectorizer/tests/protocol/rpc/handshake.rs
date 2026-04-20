//! End-to-end handshake + PING round-trip over a real TCP socket.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use std::sync::Arc;
use std::time::Duration;

use tokio::io::BufReader;
use tokio::net::TcpStream;
use vectorizer::db::VectorStore;
use vectorizer::embedding::EmbeddingManager;
use vectorizer::protocol::rpc::codec::{read_response, write_request};
use vectorizer::protocol::rpc::types::{Request, VectorizerValue};
use vectorizer_server::protocol::rpc::server::{RpcState, spawn_rpc_listener};

/// Bind the listener on `127.0.0.1:0` so the OS picks a free port,
/// then return the bound address. The listener task lives for the
/// duration of the test process.
async fn boot_listener() -> std::net::SocketAddr {
    // Pick a free ephemeral port by binding once, reading the port,
    // dropping the bind, then handing the port to the listener. There
    // is a small race window here — acceptable for a single-test
    // smoke check; a hardened test harness would loop on EADDRINUSE.
    let probe = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = probe.local_addr().unwrap();
    drop(probe);

    let state = RpcState {
        store: Arc::new(VectorStore::new()),
        embedding_manager: Arc::new(EmbeddingManager::new()),
        // `auth: None` puts the dispatch in single-user mode — every
        // HELLO succeeds and the principal is the implicit local
        // admin. Adequate for the handshake smoke test; a real
        // auth-enforcement test would build an `AuthHandlerState`.
        auth: None,
    };
    spawn_rpc_listener(state, addr).await.unwrap();
    // Give the listener a moment to actually start accepting.
    tokio::time::sleep(Duration::from_millis(50)).await;
    addr
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn hello_then_ping_roundtrip() {
    let addr = boot_listener().await;
    let stream = TcpStream::connect(addr).await.unwrap();
    let (read_half, mut write_half) = stream.into_split();
    let mut reader = BufReader::new(read_half);

    // 1. HELLO with the v1 protocol version. No token because the
    //    listener is in single-user mode.
    write_request(
        &mut write_half,
        &Request {
            id: 1,
            command: "HELLO".into(),
            args: vec![VectorizerValue::Map(vec![(
                VectorizerValue::Str("version".into()),
                VectorizerValue::Int(1),
            )])],
        },
    )
    .await
    .unwrap();
    let resp = read_response(&mut reader).await.unwrap();
    assert_eq!(resp.id, 1);
    let payload = resp.result.expect("HELLO must succeed in single-user mode");
    let auth_flag = payload.map_get("authenticated").and_then(|v| v.as_bool());
    assert_eq!(auth_flag, Some(true));
    let admin_flag = payload.map_get("admin").and_then(|v| v.as_bool());
    assert_eq!(admin_flag, Some(true));
    let proto_version = payload.map_get("protocol_version").and_then(|v| v.as_int());
    assert_eq!(proto_version, Some(1));

    // 2. PING — confirms a post-HELLO command goes through.
    write_request(
        &mut write_half,
        &Request {
            id: 2,
            command: "PING".into(),
            args: vec![],
        },
    )
    .await
    .unwrap();
    let pong = read_response(&mut reader).await.unwrap();
    assert_eq!(pong.id, 2);
    assert_eq!(pong.result.as_ref().unwrap().as_str(), Some("PONG"));

    // 3. collections.list on an empty store returns an empty array
    //    (not an error). Confirms the auth state persisted across
    //    requests and the dispatch reaches the registry-backed handler.
    write_request(
        &mut write_half,
        &Request {
            id: 3,
            command: "collections.list".into(),
            args: vec![],
        },
    )
    .await
    .unwrap();
    let listing = read_response(&mut reader).await.unwrap();
    assert_eq!(listing.id, 3);
    let arr = listing.result.unwrap();
    assert_eq!(arr.as_array().map(|s| s.len()), Some(0));
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn unauthenticated_command_is_rejected() {
    let addr = boot_listener().await;
    let stream = TcpStream::connect(addr).await.unwrap();
    let (read_half, mut write_half) = stream.into_split();
    let mut reader = BufReader::new(read_half);

    // Skip HELLO and go straight to a data-plane command. The dispatch
    // must reject with a clear error rather than crash or hang.
    write_request(
        &mut write_half,
        &Request {
            id: 1,
            command: "collections.list".into(),
            args: vec![],
        },
    )
    .await
    .unwrap();
    let resp = read_response(&mut reader).await.unwrap();
    assert_eq!(resp.id, 1);
    let err = resp.result.unwrap_err();
    assert!(
        err.contains("authentication required"),
        "expected auth-required error, got: {err}"
    );
}
