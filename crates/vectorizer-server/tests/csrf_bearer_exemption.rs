//! Integration tests for the CSRF Bearer-without-cookie exemption (phase21).
//!
//! Regression coverage for the rule described in
//! `crates/vectorizer-server/src/server/auth_handlers/csrf.rs`:
//!
//! - A caller that presents only `Authorization: Bearer <jwt>` (no
//!   `vectorizer_session` cookie) MUST pass the CSRF gate and reach the
//!   handler (`POST /auth/keys` → 200).
//! - A caller that presents the `vectorizer_session` cookie WITHOUT an
//!   `X-CSRF-Token` header MUST be rejected with 403 `missing_csrf_token`.
//! - A caller that presents BOTH the cookie AND a Bearer header but NO
//!   `X-CSRF-Token` MUST also be rejected (cookie path takes precedence;
//!   closes a potential Bearer-header downgrade attack).
//!
//! Each test builds a minimal Axum router that mirrors the auth middleware
//! stack from `src/server/core/routing.rs`, so the tests exercise the real
//! middleware composition without requiring the full server binary.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use std::sync::Arc;

use axum::Router;
use axum::body::{Body, to_bytes};
use axum::http::{Request, StatusCode, header};
use axum::routing::post;
use serde_json::Value;
use tower::ServiceExt;
use vectorizer::auth::roles::Role;
use vectorizer::auth::{AuthConfig, AuthManager, Secret};
use vectorizer_server::server::{
    AuthHandlerState, UserRecord, create_api_key, login, require_auth_middleware,
    require_csrf_middleware,
};

// ---------------------------------------------------------------------------
// Test helpers
// ---------------------------------------------------------------------------

/// Build an `AuthHandlerState` with a root admin user pre-populated so
/// `POST /auth/login` returns a real JWT.
async fn state_with_admin(username: &str, password: &str) -> AuthHandlerState {
    let config = AuthConfig {
        jwt_secret: Secret::new("z".repeat(64)),
        enabled: true,
        ..AuthConfig::default()
    };
    let manager = Arc::new(AuthManager::new(config).expect("valid auth config"));
    let state = AuthHandlerState::new(manager.clone());

    // Pre-hash the password and insert the user directly (bcrypt cost 4 for speed).
    let hash = bcrypt::hash(password, 4).expect("bcrypt hash");
    let user = UserRecord {
        user_id: username.to_string(),
        username: username.to_string(),
        password_hash: Secret::new(hash),
        roles: vec![Role::Admin],
    };
    state.users.write().await.insert(username.to_string(), user);

    state
}

/// Build the minimal router under test:
///
/// ```text
/// POST /auth/login  → login handler (public — no auth, no CSRF)
/// POST /auth/keys   → create_api_key handler
///                     ↑ wrapped in:
///                       • require_csrf_middleware
///                       • require_auth_middleware
/// ```
///
/// The middleware layer ordering mirrors `routing.rs`: CSRF outer, auth inner,
/// handler innermost.
fn build_router(state: AuthHandlerState) -> Router {
    // `/auth/keys` is the protected route under test.
    //
    // Axum layer order: the LAST `.layer()` call is outermost (executes
    // first). We need CSRF to execute first (so it can reject cookie-only
    // requests before auth injects an extension), then auth middleware
    // (so it populates Extension<AuthState> for the handler).
    let protected = Router::new()
        .route("/auth/keys", post(create_api_key))
        .layer(axum::middleware::from_fn_with_state(
            state.clone(),
            require_auth_middleware, // innermost — runs second, populates AuthState
        ))
        .layer(axum::middleware::from_fn_with_state(
            state.clone(),
            require_csrf_middleware, // outermost — runs first, gates on CSRF
        ))
        .with_state(state.clone());

    // `/auth/login` is the public bootstrap route — no CSRF, no auth guard.
    let public = Router::new()
        .route("/auth/login", post(login))
        .with_state(state);

    Router::new().merge(protected).merge(public)
}

/// POST `/auth/login` and return the `access_token` from the JSON body.
async fn do_login(router: &Router, username: &str, password: &str) -> String {
    let body = serde_json::json!({
        "username": username,
        "password": password,
    });

    let req = Request::builder()
        .method("POST")
        .uri("/auth/login")
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(serde_json::to_vec(&body).unwrap()))
        .unwrap();

    let response = router
        .clone()
        .oneshot(req)
        .await
        .expect("login request must complete");

    assert_eq!(
        response.status(),
        StatusCode::OK,
        "login must succeed with 200"
    );

    let bytes = to_bytes(response.into_body(), 64 * 1024).await.unwrap();
    let json: Value = serde_json::from_slice(&bytes).expect("login response is valid JSON");
    json.get("access_token")
        .and_then(|v| v.as_str())
        .expect("login response must contain access_token")
        .to_string()
}

// ---------------------------------------------------------------------------
// Test 1 — Bearer-only (no cookie) skips CSRF gate → 200
// ---------------------------------------------------------------------------

/// SDK / service-to-service callers present only `Authorization: Bearer`.
/// No `vectorizer_session` cookie is sent, therefore no CSRF token is
/// needed. The request MUST reach the handler and return 200.
#[tokio::test]
async fn bearer_without_cookie_skips_csrf() {
    let state = state_with_admin("admin", "hunter2").await;
    let router = build_router(state);

    let jwt = do_login(&router, "admin", "hunter2").await;

    let key_body = serde_json::json!({
        "name": "test-key",
        "permissions": ["read"]
    });

    let req = Request::builder()
        .method("POST")
        .uri("/auth/keys")
        .header(header::AUTHORIZATION, format!("Bearer {jwt}"))
        .header(header::CONTENT_TYPE, "application/json")
        // Deliberately: NO Cookie header, NO X-CSRF-Token
        .body(Body::from(serde_json::to_vec(&key_body).unwrap()))
        .unwrap();

    let response = router.oneshot(req).await.expect("request must complete");

    assert_eq!(
        response.status(),
        StatusCode::OK,
        "Bearer-only caller must not be blocked by CSRF gate"
    );

    // Parse the response body and confirm it contains the expected API key fields.
    let bytes = to_bytes(response.into_body(), 64 * 1024).await.unwrap();
    let parsed: Value = serde_json::from_slice(&bytes).expect("response must be valid JSON");
    assert_eq!(
        parsed.get("name").and_then(|v| v.as_str()),
        Some("test-key"),
        "response name must be test-key, got: {parsed}"
    );
    let api_key = parsed
        .get("api_key")
        .and_then(|v| v.as_str())
        .expect("response must contain api_key field");
    assert!(
        !api_key.is_empty(),
        "api_key must be a non-empty string, got empty"
    );
}

// ---------------------------------------------------------------------------
// Test 2 — Cookie WITHOUT CSRF token → 403 missing_csrf_token
// ---------------------------------------------------------------------------

/// Browser callers present the `vectorizer_session` cookie issued at login.
/// Without the matching `X-CSRF-Token` header the CSRF gate MUST reject the
/// request with 403 and the error code `missing_csrf_token`.
#[tokio::test]
async fn cookie_without_csrf_still_403() {
    let state = state_with_admin("admin", "hunter2").await;
    let router = build_router(state);

    // Login binds the CSRF token inside state automatically via the login handler.
    let jwt = do_login(&router, "admin", "hunter2").await;

    let key_body = serde_json::json!({
        "name": "should-not-be-created",
        "permissions": ["read"]
    });

    let req = Request::builder()
        .method("POST")
        .uri("/auth/keys")
        // Cookie present — activates the cookie path in the CSRF middleware.
        .header(header::COOKIE, format!("vectorizer_session={jwt}"))
        .header(header::CONTENT_TYPE, "application/json")
        // Deliberately: NO X-CSRF-Token header
        .body(Body::from(serde_json::to_vec(&key_body).unwrap()))
        .unwrap();

    let response = router.oneshot(req).await.expect("request must complete");

    assert_eq!(
        response.status(),
        StatusCode::FORBIDDEN,
        "Cookie-only caller without X-CSRF-Token must be rejected with 403"
    );

    let bytes = to_bytes(response.into_body(), 64 * 1024).await.unwrap();
    let json: Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(
        json.get("error").and_then(|v| v.as_str()),
        Some("missing_csrf_token"),
        "error code must be missing_csrf_token, got: {json}"
    );
}

// ---------------------------------------------------------------------------
// Test 3 — Both Bearer AND cookie present, no CSRF → 403 (downgrade guard)
// ---------------------------------------------------------------------------

/// Edge case: a confused or adversarial client sends both
/// `Authorization: Bearer <jwt>` AND `Cookie: vectorizer_session=<jwt>`.
///
/// The cookie presence MUST activate the cookie path (not the Bearer-only
/// exemption), and the absence of `X-CSRF-Token` MUST therefore return 403.
/// This prevents a downgrade attack where an attacker adds a Bearer header
/// hoping to suppress the CSRF check while the browser still auto-attaches
/// the session cookie.
#[tokio::test]
async fn bearer_plus_cookie_requires_csrf() {
    let state = state_with_admin("admin", "hunter2").await;
    let router = build_router(state);

    let jwt = do_login(&router, "admin", "hunter2").await;

    let key_body = serde_json::json!({
        "name": "downgrade-attempt",
        "permissions": ["read"]
    });

    let req = Request::builder()
        .method("POST")
        .uri("/auth/keys")
        // Both credentials present — cookie must win and activate CSRF gate.
        .header(header::AUTHORIZATION, format!("Bearer {jwt}"))
        .header(header::COOKIE, format!("vectorizer_session={jwt}"))
        .header(header::CONTENT_TYPE, "application/json")
        // Deliberately: NO X-CSRF-Token header
        .body(Body::from(serde_json::to_vec(&key_body).unwrap()))
        .unwrap();

    let response = router.oneshot(req).await.expect("request must complete");

    assert_eq!(
        response.status(),
        StatusCode::FORBIDDEN,
        "Bearer + Cookie without X-CSRF-Token must be rejected with 403 \
         (cookie path must win over Bearer exemption)"
    );

    let bytes = to_bytes(response.into_body(), 64 * 1024).await.unwrap();
    let json: Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(
        json.get("error").and_then(|v| v.as_str()),
        Some("missing_csrf_token"),
        "error code must be missing_csrf_token, got: {json}"
    );
}
