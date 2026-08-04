//! End-to-end RPC round-trips over Thunder's transport, driven by
//! Thunder's own client — the same client the Rust SDK ships, so these
//! tests exercise the real client/server pair rather than a synthetic
//! socket peer.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use std::sync::Arc;

use thunder::client::{Client, ClientConfig, ClientError};
use vectorizer::auth::roles::Role;
use vectorizer::auth::{AuthConfig, AuthManager, Secret};
use vectorizer::db::VectorStore;
use vectorizer::embedding::EmbeddingManager;
use vectorizer::protocol::rpc::Value;
use vectorizer_server::protocol::rpc::server::{RpcState, spawn_rpc_listener, vectorizer_config};
use vectorizer_server::server::AuthHandlerState;

/// Bind the listener on an ephemeral port and return the endpoint URL the
/// Thunder client dials. `auth` decides the deployment posture: `None`
/// opens the listener (single-user mode, every caller is the implicit
/// local admin), `Some` makes Thunder refuse un-credentialed sessions.
///
/// The listener task lives for the duration of the test process.
async fn boot_listener(auth: Option<AuthHandlerState>) -> String {
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
        auth,
        master_node: None,
        replica_node: None,
        cluster_manager: None,
        slow_query_ring: vectorizer::cache::slow_query::SlowQueryRing::new(
            vectorizer::cache::slow_query::SlowQueryConfig::default(),
        ),
        auto_save_manager: None,
    };
    // `spawn_rpc_listener` returns once the socket is bound, so a client
    // may dial immediately.
    spawn_rpc_listener(state, addr).await.unwrap();
    format!("vectorizer://{addr}")
}

/// An auth-enabled handler state plus an admin JWT that validates
/// against it. No user record is seeded: JWT validation is signature +
/// claims only, and the admin bit comes from the claims' roles.
fn auth_state_with_admin_jwt() -> (AuthHandlerState, String) {
    let config = AuthConfig {
        jwt_secret: Secret::new("t".repeat(64)),
        enabled: true,
        ..AuthConfig::default()
    };
    let manager = Arc::new(AuthManager::new(config).expect("valid auth config"));
    let jwt = manager
        .generate_jwt("rpc-admin", "rpc-admin", vec![Role::Admin])
        .expect("generate admin JWT");
    (AuthHandlerState::new(manager), jwt)
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn ping_then_dispatch_roundtrip() {
    let endpoint = boot_listener(None).await;
    let client = Client::connect(&endpoint, vectorizer_config())
        .await
        .unwrap();

    // PING is answered by our dispatch table (Thunder only intercepts it
    // pre-auth), so a PONG proves the Dispatch binding is wired up.
    let pong = client.call("PING", vec![]).await.unwrap();
    assert_eq!(pong.as_str(), Some("PONG"));

    // collections.list on an empty store returns an empty array (not an
    // error) — the dispatch reaches the registry-backed handler.
    let listing = client.call("collections.list", vec![]).await.unwrap();
    assert_eq!(listing.as_array().map(|s| s.len()), Some(0));

    // HELLO stays in the command catalog for pre-Thunder clients that
    // still lead with it. In single-user mode it reports the implicit
    // local admin and the v1 protocol version.
    let hello = client
        .call(
            "HELLO",
            vec![Value::Map(vec![(
                Value::Str("version".into()),
                Value::Int(1),
            )])],
        )
        .await
        .unwrap();
    assert_eq!(
        hello.map_get("authenticated").and_then(|v| v.as_bool()),
        Some(true)
    );
    assert_eq!(hello.map_get("admin").and_then(|v| v.as_bool()), Some(true));
    assert_eq!(
        hello.map_get("protocol_version").and_then(|v| v.as_int()),
        Some(1)
    );

    client.close().await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn unauthenticated_command_is_rejected() {
    let (auth_state, _jwt) = auth_state_with_admin_jwt();
    let endpoint = boot_listener(Some(auth_state)).await;

    // No credentials on the client config: under `Handshake::AuthCommand`
    // that means no `AUTH` frame, so the session stays gated. The dial
    // still succeeds — the refusal happens per command.
    let client = Client::connect(&endpoint, vectorizer_config())
        .await
        .unwrap();
    assert!(!client.is_authenticated());

    let err = client.call("collections.list", vec![]).await.unwrap_err();
    assert!(
        matches!(err, ClientError::Auth { .. }),
        "expected an auth-class error, got: {err:?}"
    );

    client.close().await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn admin_jwt_authenticates_the_session() {
    let (auth_state, jwt) = auth_state_with_admin_jwt();
    let endpoint = boot_listener(Some(auth_state)).await;

    // `Dispatch::authenticate` validates the secret as a JWT and lifts
    // the admin bit out of its claims, so the session is authenticated
    // before the first command and passes the admin ACL.
    let client = Client::connect_with(
        &endpoint,
        vectorizer_config(),
        ClientConfig::new().token(jwt),
    )
    .await
    .unwrap();
    assert!(client.is_authenticated());

    let listing = client.call("collections.list", vec![]).await.unwrap();
    assert_eq!(listing.as_array().map(|s| s.len()), Some(0));

    client.close().await;
}
