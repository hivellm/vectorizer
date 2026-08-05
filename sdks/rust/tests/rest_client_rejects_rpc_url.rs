//! The REST client refuses an RPC URL at construction
//! (phase2_rest-client-accepts-rpc-scheme, issue #392).
//!
//! This is the mirror of `rpc_integration.rs::connect_url_rejects_http_scheme_with_clear_error`.
//! The RPC client already taught callers who handed it an `http://` URL; the
//! REST facade accepted `vectorizer://` without complaint and failed only at
//! the first request, from inside reqwest:
//!
//! ```text
//! Network error: HTTP request failed:
//! builder error for url (vectorizer://127.0.0.1:15503/auth/login)
//! ```
//!
//! That message names neither the scheme nor the client that would have
//! worked, and it arrives far from the line that caused it.
//!
//! Driven through `VectorizerClient::new` rather than `HttpTransport::new`
//! (which has its own unit tests) because the public entry point is what the
//! issue used, and because the guard lives one layer down — a refactor that
//! stopped routing construction through `HttpTransport::new` would pass those
//! unit tests while regressing this.

#![cfg(feature = "http")]
#![allow(clippy::unwrap_used, clippy::expect_used)]

use vectorizer_sdk::{ClientConfig, VectorizerClient};

/// The exact shape reported in #392.
#[test]
fn new_rejects_a_vectorizer_url_instead_of_failing_at_first_request() {
    let err = VectorizerClient::new(ClientConfig {
        base_url: Some("vectorizer://127.0.0.1:15503".into()),
        ..Default::default()
    })
    .err()
    .expect("constructing the REST client with an RPC URL must fail");

    let msg = err.to_string();
    assert!(
        msg.contains("RpcClient"),
        "the error must name the client that would have worked; got: {msg}"
    );
    assert!(
        msg.contains("vectorizer://"),
        "the error must name the offending scheme; got: {msg}"
    );
    assert!(
        !msg.contains("builder error"),
        "the reqwest internals error is what this fix replaces; got: {msg}"
    );
}

#[test]
fn new_with_url_is_guarded_too() {
    // The convenience constructors funnel into the same transport, so they
    // inherit the guard. Pinned because they are the shortest path a caller
    // reaches for.
    let err = VectorizerClient::new_with_url("vectorizer://host:15503")
        .err()
        .expect("new_with_url must reject an RPC URL as well");
    assert!(err.to_string().contains("RpcClient"), "{err}");
}

#[test]
fn http_urls_still_construct() {
    // No server is contacted here — construction is offline, which is exactly
    // why rejecting a bad scheme at this point is worth doing.
    VectorizerClient::new(ClientConfig {
        base_url: Some("http://localhost:15002".into()),
        ..Default::default()
    })
    .expect("an http base URL is the normal case");

    VectorizerClient::new_with_url("localhost:15002")
        .expect("scheme-less base URLs must keep working");
}
