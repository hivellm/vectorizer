//! In-process migration of
//! `crates/vectorizer/tests/api/rest/auth_enforcement_real.rs`
//! (phase39 §1.2 final batch) onto the shared harness in
//! `tests/common/mod.rs`.
//!
//! Same assertions as the live-server suite (phase8_gate-data-routes-when-
//! auth-enabled), but built on [`common::TestApp::with_auth`] — a real
//! `AuthManager` + `AuthHandlerState` wired into the production router via
//! `tower::ServiceExt::oneshot` — instead of `reqwest` against a running
//! server on `127.0.0.1:15002`. No `#[ignore]`, runs in CI.
//!
//! - Anonymous calls to the public surface (`/health`,
//!   `/prometheus/metrics`, `/umicp/discover`, `/dashboard/`) still
//!   return 200.
//! - Anonymous calls to data routes (`/collections`, `/stats`, `/logs`,
//!   `/auth/me`) return 401 with the `{"error": "unauthorized", ...}`
//!   shape `require_auth_middleware` emits.
//! - A JWT minted by the real `POST /auth/login` handler unlocks
//!   `/collections`; a garbage bearer token is still rejected with 401.
//! - `POST /auth/login` with the wrong password returns 401
//!   `invalid_credentials`.

#![allow(clippy::unwrap_used, clippy::expect_used)]
#![allow(clippy::uninlined_format_args)]

mod common;

use std::sync::LazyLock;

use axum::http::StatusCode;
use common::TestApp;
use serde_json::json;
use tokio::sync::Mutex as AsyncMutex;

/// `TestApp::with_auth` writes `AuthPersistence`'s encryption key + data
/// file under `VECTORIZER_DATA_DIR` (see `common/mod.rs`'s "Known
/// limitation" section) — process-global and therefore unsafe to mutate
/// concurrently from two tests in the same binary. Every test in this
/// file calls `with_auth`, so each one holds this lock for its entire
/// body, mirroring the `ENV_DIR_LOCK` pattern in
/// `tests/rest_lifecycle_handlers.rs`.
static ENV_DIR_LOCK: LazyLock<AsyncMutex<()>> = LazyLock::new(|| AsyncMutex::new(()));

#[tokio::test]
async fn public_routes_stay_anonymous_with_auth_enabled() {
    let _guard = ENV_DIR_LOCK.lock().await;
    let (app, _creds) = TestApp::with_auth().await;

    for path in [
        "/health",
        "/prometheus/metrics",
        "/umicp/discover",
        "/dashboard/",
    ] {
        let (status, body) = app.get(path).await;
        assert_eq!(
            status,
            StatusCode::OK,
            "{path} should stay anonymous (got {status}: {body})"
        );
    }
}

#[tokio::test]
async fn data_routes_require_auth_when_auth_enabled() {
    let _guard = ENV_DIR_LOCK.lock().await;
    let (app, _creds) = TestApp::with_auth().await;

    for path in ["/collections", "/stats", "/logs", "/auth/me"] {
        let (status, body) = app.get(path).await;
        assert_eq!(
            status.as_u16(),
            401,
            "{path} should reject anonymous callers (got {status}: {body})"
        );
        assert_eq!(
            body["error"].as_str(),
            Some("unauthorized"),
            "{path} error body: {body}"
        );
    }
}

#[tokio::test]
async fn valid_jwt_unlocks_data_routes() {
    let _guard = ENV_DIR_LOCK.lock().await;
    let (app, creds) = TestApp::with_auth().await;

    let (status, login_resp) = app
        .post_json(
            "/auth/login",
            json!({ "username": creds.username, "password": creds.password }),
        )
        .await;
    assert_eq!(
        status,
        StatusCode::OK,
        "login status {status}: {login_resp}"
    );
    let token = login_resp["access_token"]
        .as_str()
        .expect("login response must contain access_token")
        .to_string();
    assert!(!token.is_empty(), "login returned empty access_token");

    let (status, resp) = app.get_with_bearer("/collections", &token).await;
    assert_eq!(
        status,
        StatusCode::OK,
        "authenticated call should succeed (got {status}: {resp})"
    );

    let (status, _resp) = app.get_with_bearer("/collections", "not-a-real-jwt").await;
    assert_eq!(status.as_u16(), 401, "a garbage token must be rejected");
}

#[tokio::test]
async fn auth_login_rejects_wrong_password() {
    let _guard = ENV_DIR_LOCK.lock().await;
    let (app, creds) = TestApp::with_auth().await;

    let (status, body) = app
        .post_json(
            "/auth/login",
            json!({ "username": creds.username, "password": "definitely-not-the-password" }),
        )
        .await;
    assert_eq!(status.as_u16(), 401);
    assert_eq!(body["error"].as_str(), Some("invalid_credentials"));
}
