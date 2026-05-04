//! CSRF token validation middleware.
//!
//! Every mutating dashboard request (POST / PUT / PATCH / DELETE) under
//! `/auth/*` and `/admin/*` MUST carry an `X-CSRF-Token` header whose
//! value matches the CSRF token bound to the caller's JWT session at
//! login time. Mismatches return HTTP 403; missing tokens return 403.
//!
//! Exemptions (in evaluation order):
//!
//! 1. Dev-mode loopback (`auth.dev_mode_skip_loopback`) — no auth at all.
//! 2. GET / HEAD / OPTIONS — non-mutating; no CSRF risk.
//! 3. `CSRF_EXEMPT_PATHS` — `/auth/login` and `/auth/validate-password`
//!    bootstrap the session flow before any CSRF token exists.
//! 4. `X-API-Key` header — header-bearer credential; not subject to the
//!    cross-origin cookie-hijack attack CSRF defends against.
//! 5. `Authorization: Bearer` **without** a `vectorizer_session` cookie —
//!    SDK / service-to-service callers that explicitly attach a JWT via the
//!    `Authorization` header and have no browser session cookie. The attack
//!    vector (browser silently re-attaching a credential) does not apply
//!    when the credential must be explicitly set by the caller.
//!
//! Cookie-path callers (browser dashboard sessions) still pay the full
//! CSRF gate: a `vectorizer_session` cookie present in the request
//! requires a matching `X-CSRF-Token` header.

use axum::extract::State;
use axum::http::{Method, StatusCode};
use axum::response::{IntoResponse, Json};

use super::cookies::{CSRF_HEADER_NAME, SESSION_COOKIE_NAME, read_cookie};
use super::state::AuthHandlerState;
use super::types::AuthErrorResponse;

/// Paths that MUST stay reachable without a CSRF token because they
/// bootstrap the session flow. `/auth/login` mints the very token the
/// SPA needs to call any other mutating endpoint; `/auth/validate-password`
/// is documented as a public, unauthenticated strength check.
const CSRF_EXEMPT_PATHS: &[&str] = &["/auth/login", "/auth/validate-password"];

/// CSRF gate for mutating requests under `/auth/*` and `/admin/*`.
///
/// Exemptions (in order):
/// 1. Dev-mode loopback (`auth.dev_mode_skip_loopback`) — no auth at all.
/// 2. GET / HEAD / OPTIONS — non-mutating.
/// 3. `CSRF_EXEMPT_PATHS` — login + validate-password (bootstrap routes).
/// 4. `X-API-Key` header — header-bearer credential, not subject to CSRF.
/// 5. `Authorization: Bearer` header without `vectorizer_session` cookie —
///    SDK / service-to-service callers (phase21).
///
/// Otherwise: requires an `X-CSRF-Token` header that matches the token
/// bound to the JWT in the `vectorizer_session` cookie.
pub async fn require_csrf_middleware(
    State(state): State<AuthHandlerState>,
    request: axum::extract::Request,
    next: axum::middleware::Next,
) -> axum::response::Response {
    // Exemption 1 — Dev-mode short-circuit: when `auth.dev_mode_skip_loopback`
    // is on (and the boot guard has confirmed we are bound to a loopback
    // address) the CSRF check would have nothing to validate against —
    // there is no session JWT, no cookie, no header. Skip the gate so
    // the rest of the dev-mode story (no Authorization header, no CSRF
    // token) is internally consistent.
    if super::middleware::dev_mode_active(&state) {
        return next.run(request).await;
    }

    // Exemption 2 — non-mutating HTTP methods carry no CSRF risk.
    let method = request.method();
    if matches!(*method, Method::GET | Method::HEAD | Method::OPTIONS) {
        return next.run(request).await;
    }

    // Exemption 3 — bootstrap paths that exist before any CSRF token is minted.
    let path = request.uri().path();
    if CSRF_EXEMPT_PATHS.iter().any(|p| *p == path) {
        return next.run(request).await;
    }

    let headers = request.headers();

    // Exemption 4 — API-key requests bypass CSRF (header-bearer credentials).
    if headers.contains_key("X-API-Key") {
        return next.run(request).await;
    }

    // Exemption 5 (phase21) — Bearer-without-cookie exemption.
    //
    // SDK + service-to-service callers attach the JWT via
    // `Authorization: Bearer` and have no `vectorizer_session` cookie.
    // CSRF tokens defend against cross-origin attacks where the browser
    // auto-attaches a cookie credential — Bearer is explicitly set by the
    // caller, so the attack vector does not apply. The cookie path below
    // stays fully gated.
    let has_session_cookie = read_cookie(headers, SESSION_COOKIE_NAME).is_some();
    let has_bearer_header = headers
        .get(axum::http::header::AUTHORIZATION)
        .and_then(|h| h.to_str().ok())
        .is_some_and(|s| s.starts_with("Bearer "));
    if has_bearer_header && !has_session_cookie {
        return next.run(request).await;
    }

    // Cookie path: locate the JWT from the session cookie only.
    // (By this point either the cookie is present, or neither credential
    // was supplied — both cases require the full CSRF gate.)
    let jwt = read_cookie(headers, SESSION_COOKIE_NAME).map(|s| s.to_string());

    let Some(jwt) = jwt else {
        return forbidden("missing_session", "No session token found for CSRF check");
    };

    let Some(provided) = headers
        .get(CSRF_HEADER_NAME)
        .and_then(|h| h.to_str().ok())
        .map(|s| s.to_string())
    else {
        return forbidden(
            "missing_csrf_token",
            "X-CSRF-Token header is required on mutating requests",
        );
    };

    let Some(expected) = state.get_csrf_token(&jwt).await else {
        return forbidden(
            "invalid_csrf_token",
            "No CSRF token bound to this session; re-login to refresh",
        );
    };

    if !constant_time_eq(provided.as_bytes(), expected.as_bytes()) {
        return forbidden("invalid_csrf_token", "X-CSRF-Token mismatch");
    }

    next.run(request).await
}

fn forbidden(error: &'static str, message: &'static str) -> axum::response::Response {
    (
        StatusCode::FORBIDDEN,
        Json(AuthErrorResponse {
            error: error.to_string(),
            message: message.to_string(),
        }),
    )
        .into_response()
}

/// Length-aware constant-time byte comparison. Avoids `==` short-circuit
/// timing leaks on the CSRF token comparison path.
fn constant_time_eq(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() {
        return false;
    }
    let mut diff: u8 = 0;
    for (x, y) in a.iter().zip(b.iter()) {
        diff |= x ^ y;
    }
    diff == 0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn constant_time_eq_returns_false_on_length_mismatch() {
        assert!(!constant_time_eq(b"abc", b"abcd"));
    }

    #[test]
    fn constant_time_eq_returns_true_on_equal_bytes() {
        assert!(constant_time_eq(b"abc123", b"abc123"));
    }

    #[test]
    fn constant_time_eq_returns_false_on_one_byte_diff() {
        assert!(!constant_time_eq(b"abc123", b"abc124"));
    }

    // --- Phase21: Bearer-without-cookie exemption unit tests
    //
    // Build the same minimal router already used by auth_handlers_tests.rs
    // so the middleware is exercised through the full Axum stack.

    use std::sync::Arc;

    use axum::Router;
    use axum::body::Body;
    use axum::http::Request;
    use axum::routing::post;
    use tower::ServiceExt;
    use vectorizer::auth::{AuthConfig, AuthManager, Secret};

    async fn test_state_with_admin_jwt() -> (super::super::state::AuthHandlerState, String) {
        use vectorizer::auth::roles::Role;

        let config = AuthConfig {
            jwt_secret: Secret::new("z".repeat(64)),
            enabled: true,
            ..AuthConfig::default()
        };
        let manager = Arc::new(AuthManager::new(config).expect("valid auth config"));
        let token = manager
            .generate_jwt("user-test", "tester", vec![Role::Admin])
            .expect("generate_jwt");
        (super::super::state::AuthHandlerState::new(manager), token)
    }

    async fn csrf_router(state: super::super::state::AuthHandlerState) -> Router {
        async fn ok_handler() -> &'static str {
            "ok"
        }
        Router::new().route("/auth/keys", post(ok_handler)).layer(
            axum::middleware::from_fn_with_state(state, require_csrf_middleware),
        )
    }

    async fn post_with_headers(
        router: Router,
        authorization: Option<&str>,
        cookie: Option<&str>,
        csrf_header: Option<&str>,
    ) -> axum::http::StatusCode {
        let mut builder = Request::builder().method("POST").uri("/auth/keys");
        if let Some(v) = authorization {
            builder = builder.header(axum::http::header::AUTHORIZATION, v);
        }
        if let Some(v) = cookie {
            builder = builder.header(axum::http::header::COOKIE, v);
        }
        if let Some(v) = csrf_header {
            builder = builder.header("X-CSRF-Token", v);
        }
        let req = builder.body(Body::empty()).expect("build request");
        router.oneshot(req).await.expect("router oneshot").status()
    }

    /// Exemption 4: X-API-Key bypasses CSRF (existing behaviour, verified).
    #[tokio::test]
    async fn api_key_header_bypasses_csrf_gate() {
        let (state, _jwt) = test_state_with_admin_jwt().await;
        let router = csrf_router(state).await;
        let req = Request::builder()
            .method("POST")
            .uri("/auth/keys")
            .header("X-API-Key", "some-api-key")
            .body(Body::empty())
            .unwrap();
        let status = router.oneshot(req).await.unwrap().status();
        assert_eq!(status, axum::http::StatusCode::OK);
    }

    /// Exemption 5: Bearer header WITHOUT session cookie → exempt (phase21).
    #[tokio::test]
    async fn bearer_without_cookie_bypasses_csrf_gate() {
        let (state, jwt) = test_state_with_admin_jwt().await;
        let router = csrf_router(state).await;
        let status = post_with_headers(
            router,
            Some(&format!("Bearer {jwt}")),
            None, // no session cookie
            None, // no X-CSRF-Token
        )
        .await;
        assert_eq!(status, axum::http::StatusCode::OK);
    }

    /// Cookie WITHOUT CSRF token → still 403 (cookie path fully gated).
    #[tokio::test]
    async fn cookie_without_csrf_header_returns_403() {
        let (state, jwt) = test_state_with_admin_jwt().await;
        // Bind a CSRF token so the session lookup does not fail at the
        // "unknown session" step — we want to hit missing_csrf_token, not
        // invalid_csrf_token.
        state
            .store_csrf_token(jwt.clone(), "some-csrf".to_string())
            .await;
        let router = csrf_router(state).await;
        let status = post_with_headers(
            router,
            None,
            Some(&format!("vectorizer_session={jwt}")),
            None, // deliberately omit X-CSRF-Token
        )
        .await;
        assert_eq!(status, axum::http::StatusCode::FORBIDDEN);
    }

    /// Edge case: both Bearer AND cookie present → cookie path wins → CSRF required.
    ///
    /// This protects against a downgrade attack where a confused or adversarial
    /// client sets an `Authorization: Bearer` header hoping to bypass the cookie
    /// path while also sending a `vectorizer_session` cookie.
    #[tokio::test]
    async fn bearer_plus_cookie_requires_csrf() {
        let (state, jwt) = test_state_with_admin_jwt().await;
        state
            .store_csrf_token(jwt.clone(), "expected-csrf".to_string())
            .await;
        let router = csrf_router(state).await;
        // Both present but no X-CSRF-Token → must still be 403.
        let status = post_with_headers(
            router,
            Some(&format!("Bearer {jwt}")),
            Some(&format!("vectorizer_session={jwt}")),
            None, // no X-CSRF-Token
        )
        .await;
        assert_eq!(status, axum::http::StatusCode::FORBIDDEN);
    }

    /// Cookie + correct CSRF header → 200 (full cookie path still works).
    #[tokio::test]
    async fn cookie_with_correct_csrf_header_succeeds() {
        let (state, jwt) = test_state_with_admin_jwt().await;
        state
            .store_csrf_token(jwt.clone(), "correct-csrf".to_string())
            .await;
        let router = csrf_router(state).await;
        let status = post_with_headers(
            router,
            None,
            Some(&format!("vectorizer_session={jwt}")),
            Some("correct-csrf"),
        )
        .await;
        assert_eq!(status, axum::http::StatusCode::OK);
    }
}
