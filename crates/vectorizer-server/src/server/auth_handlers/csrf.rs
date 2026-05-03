//! CSRF token validation middleware.
//!
//! Every mutating dashboard request (POST / PUT / PATCH / DELETE) under
//! `/auth/*` and `/admin/*` MUST carry an `X-CSRF-Token` header whose
//! value matches the CSRF token bound to the caller's JWT session at
//! login time. Mismatches return HTTP 403; missing tokens return 403.
//!
//! Two paths are exempt because they bootstrap the CSRF flow itself:
//!
//! - `POST /auth/login` — mints the JWT + CSRF token in the first place
//! - `POST /auth/validate-password` — public, no session yet
//!
//! GET / HEAD / OPTIONS bypass CSRF entirely (read-only).

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

/// Axum middleware that rejects mutating `/auth/*` and `/admin/*` requests
/// missing or carrying a mismatched `X-CSRF-Token` header.
///
/// Lookup order for the JWT that pins the CSRF binding:
/// 1. `Cookie: vectorizer_session=<jwt>` — set by the new login flow
/// 2. `Authorization: Bearer <jwt>` — for legacy/REST API consumers that
///    happen to hit `/auth/*` mutating routes (kept so the change does not
///    break programmatic users)
///
/// API-key requests (no JWT, only `X-API-Key`) bypass CSRF — they are
/// header-bearer credentials not subject to the cross-origin attack the
/// CSRF token defends against. The dashboard never uses API keys for its
/// own session.
pub async fn require_csrf_middleware(
    State(state): State<AuthHandlerState>,
    request: axum::extract::Request,
    next: axum::middleware::Next,
) -> axum::response::Response {
    let method = request.method();
    if matches!(*method, Method::GET | Method::HEAD | Method::OPTIONS) {
        return next.run(request).await;
    }

    let path = request.uri().path();
    if CSRF_EXEMPT_PATHS.iter().any(|p| *p == path) {
        return next.run(request).await;
    }

    let headers = request.headers();

    // API-key requests bypass CSRF (header-bearer credentials).
    if headers.contains_key("X-API-Key") {
        return next.run(request).await;
    }

    // Locate the JWT that pins the CSRF binding.
    let jwt = read_cookie(headers, SESSION_COOKIE_NAME)
        .map(|s| s.to_string())
        .or_else(|| {
            headers
                .get(axum::http::header::AUTHORIZATION)
                .and_then(|h| h.to_str().ok())
                .and_then(|s| s.strip_prefix("Bearer "))
                .map(|s| s.to_string())
        });

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
}
