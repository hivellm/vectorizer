//! Route-reachability check for the capability registry (phase40 §1.3).
//!
//! `crates/vectorizer/tests/api/parity.rs` validates the registry
//! *structurally* (ids unique, `Transport` tags self-consistent, MCP
//! schemas match). It cannot boot a server, so it can't tell whether a
//! `rest: Some((method, path))` entry actually resolves against the
//! real router — which is exactly the bug class phase40 §1.1 found:
//! `capabilities.rs` declared `graph.find_related` as `GET` while
//! `api/graph.rs` only ever registered `POST`, so a registry-driven
//! caller got a silent 405.
//!
//! This suite closes that gap by dispatching every
//! [`inventory`] REST route *and* every
//! [`documented_rest_exclusions`] route against the REAL production
//! router (`VectorizerServer::build_router`, via the [`common::TestApp`]
//! harness) and asserting the response is neither:
//!
//! - a **405** (Method Not Allowed) — the path pattern matched a
//!   registered route, but not on the declared method, i.e. exactly the
//!   §1.1 bug; or
//! - a **true 404** (no route matched at all, empty/non-JSON body) —
//!   distinguished from a *business* 404 (e.g. "collection not found"),
//!   which every handler in this codebase returns as a JSON error body
//!   (see `error/mapping.rs`). A route that resolves to a handler which
//!   then reports "resource not found" for our placeholder path
//!   parameters is proof the route exists; a route axum couldn't match
//!   at all falls through to axum's default empty-body 404.
//!
//! ## What this does **not** catch
//!
//! A brand-new route added to `core/routing.rs` / `api/graph.rs` that
//! nobody ever adds to [`inventory`] or [`documented_rest_exclusions`].
//! Axum's `Router` has no stable public API to enumerate its own
//! registered routes, so there is no way to diff "the real router's
//! full route set" against the registry without hand-maintaining a
//! second copy of every route the server exposes (including ones far
//! outside this registry's stated scope — auth, admin, Qdrant
//! compatibility, replication, cluster, GraphQL, etc. — see the
//! exclusion list in `capabilities.rs`'s module doc comment). Catching
//! *that* direction of drift is deliberately left as follow-up work
//! rather than accepting either a stale, broad static route list, or
//! silently expanding this test's scope into unrelated route groups.

#![allow(clippy::unwrap_used, clippy::expect_used)]

mod common;

use common::TestApp;
use serde_json::{Value, json};
use vectorizer_server::server::capabilities::{documented_rest_exclusions, inventory};

/// Replace every axum `{param}` path segment with a fixed placeholder.
///
/// The registry only records the path *pattern*, not a value that
/// resolves to a real resource, so every probe necessarily targets a
/// nonexistent collection/node/edge. That's fine here — this suite only
/// asserts the route resolves to a handler at all (see module doc for
/// how a business 404 is told apart from a routing 404).
fn concretize(path: &str) -> String {
    let mut out = String::new();
    let mut chars = path.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '{' {
            for c2 in chars.by_ref() {
                if c2 == '}' {
                    break;
                }
            }
            out.push_str("phase40-probe");
        } else {
            out.push(c);
        }
    }
    out
}

/// Dispatch `method` against `path` through the real router, returning
/// `(status, body)` exactly like the other `TestApp` helpers.
///
/// Only the HTTP verbs actually used by the capability registry today
/// are wired up; a registry entry using anything else is a signal to
/// extend this match arm rather than silently skip verification.
async fn probe(app: &TestApp, method: &str, path: &str) -> (axum::http::StatusCode, Value) {
    match method {
        "GET" => app.get(path).await,
        "POST" => app.post_json(path, json!({})).await,
        "PUT" => app.put_json(path, json!({})).await,
        "PATCH" => app.patch_json(path, json!({})).await,
        "DELETE" => app.delete(path).await,
        other => panic!(
            "capability_registry_route_reachability: unsupported HTTP method '{other}' — \
             add a dispatch arm in `probe()` before adding a registry entry that uses it"
        ),
    }
}

/// `true` if `(status, body)` represents axum's own "no route matched"
/// 404 (empty/non-JSON body) rather than a handler-produced business
/// 404 (a JSON error object, e.g. `{"error": "Collection '...' not
/// found"}`).
fn is_true_routing_404(status: axum::http::StatusCode, body: &Value) -> bool {
    status == axum::http::StatusCode::NOT_FOUND && *body == Value::Null
}

#[tokio::test]
async fn every_inventory_rest_route_resolves_on_the_declared_method() {
    // Deliberately the no-auth harness. `require_auth_middleware` (see
    // `TestApp::with_auth`) is a tower `Layer` wrapped *around* the
    // entire `rest_routes` sub-router: it runs before delegating to the
    // inner service, and short-circuits with 401 on every unauthenticated
    // request — matched route or not, right method or not. That masks
    // exactly the 404/405 signal this test needs, so auth must stay off
    // here. The one route that lives outside `rest_routes` entirely when
    // auth is disabled — `POST /auth/login`, only mounted by
    // `build_router` when an `AuthHandlerState` exists — is verified
    // separately below in `auth_login_route_resolves_on_the_auth_enabled_router`.
    let app = TestApp::new().await;
    let mut failures: Vec<String> = Vec::new();

    for cap in inventory() {
        if cap.id == "auth.login" {
            continue;
        }
        let Some((method, path)) = cap.rest else {
            continue;
        };
        let concrete_path = concretize(path);
        let (status, body) = probe(&app, method, &concrete_path).await;

        if status == axum::http::StatusCode::METHOD_NOT_ALLOWED {
            failures.push(format!(
                "{}: {method} {path} -> 405 Method Not Allowed (registry declares the wrong \
                 HTTP method for this route)",
                cap.id
            ));
        } else if is_true_routing_404(status, &body) {
            failures.push(format!(
                "{}: {method} {path} -> 404 with an empty/non-JSON body (no route matched at \
                 all — the registry points at a route that doesn't exist)",
                cap.id
            ));
        }
    }

    assert!(
        failures.is_empty(),
        "capability registry entries that don't resolve against the real router:\n{}",
        failures.join("\n")
    );
}

#[tokio::test]
async fn auth_login_route_resolves_on_the_auth_enabled_router() {
    // `POST /auth/login` (the `auth.login` capability) is only mounted
    // when `AuthHandlerState` exists, so it needs the auth-enabled
    // harness. Unlike every other data route, `/auth/login` is merged
    // back onto the router *outside* the `require_auth_middleware`
    // layer (see `routing.rs`: `gated.merge(public_auth_router)`), so it
    // is not subject to the blanket-401 short-circuit described above —
    // a bad/empty login attempt still reaches the real handler and
    // returns a normal 4xx login-failure body, not 404/405.
    let (app, _auth_fixture) = TestApp::with_auth().await;
    let cap = inventory()
        .into_iter()
        .find(|c| c.id == "auth.login")
        .expect("auth.login must exist in the registry");
    let (method, path) = cap
        .rest
        .expect("auth.login is RestOnly and must carry a rest route");

    let (status, body) = probe(&app, method, &concretize(path)).await;
    assert_ne!(
        status,
        axum::http::StatusCode::METHOD_NOT_ALLOWED,
        "auth.login: {method} {path} -> 405 Method Not Allowed"
    );
    assert!(
        !is_true_routing_404(status, &body),
        "auth.login: {method} {path} -> 404 with an empty/non-JSON body (no route matched)"
    );
}

#[tokio::test]
async fn every_documented_exclusion_route_still_resolves() {
    let app = TestApp::new().await;
    let mut failures: Vec<String> = Vec::new();

    for (method, path, reason) in documented_rest_exclusions() {
        let concrete_path = concretize(path);
        let (status, body) = probe(&app, method, &concrete_path).await;

        if status == axum::http::StatusCode::METHOD_NOT_ALLOWED {
            failures.push(format!(
                "{method} {path} -> 405 Method Not Allowed (exclusion reason: {reason})"
            ));
        } else if is_true_routing_404(status, &body) {
            failures.push(format!(
                "{method} {path} -> 404 with an empty/non-JSON body — this exclusion no longer \
                 points at a live route; remove it from documented_rest_exclusions() (reason \
                 was: {reason})"
            ));
        }
    }

    assert!(
        failures.is_empty(),
        "documented_rest_exclusions() entries that no longer resolve against the real router:\n{}",
        failures.join("\n")
    );
}
