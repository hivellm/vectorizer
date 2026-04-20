//! Typed axum extractors that front-load admin / authentication checks
//! onto the handler signature instead of requiring each handler to call
//! `require_admin_for_rest(...)` in its body.
//!
//! A handler that declares `_admin: AdminAuth` in its signature is
//! admin-only by construction: axum runs the extractor before the
//! handler body and short-circuits with a 401 / 403 `ErrorResponse` if
//! the caller is not an admin. This mirrors the pattern in
//! [`middleware`](super::middleware) but keeps the check next to the
//! handler signature (so it is reviewable next to the route
//! declaration) and removes the need to plumb `HeaderMap` through.
//!
//! Backward-compat: when the server boots with no `AuthHandlerState`
//! (auth globally disabled for single-user local setups), both
//! extractors succeed for every caller — identical to the existing
//! `require_admin_for_rest` semantics.

use axum::extract::FromRequestParts;
use axum::http::HeaderMap;
use axum::http::request::Parts;
use axum::response::Json;

use super::middleware::require_admin_from_headers;
use super::state::AuthHandlerState;
use super::types::AuthErrorResponse;
use crate::auth::middleware::AuthState;
use crate::server::VectorizerServer;
use crate::server::error_middleware::ErrorResponse;

/// Admin-auth extractor.
///
/// Wraps the authenticated `AuthState` after verifying `Role::Admin`.
/// The inner `Option` is `None` when auth is globally disabled, which
/// callers can treat as "every caller is admin" (matches the legacy
/// behaviour preserved by `require_admin_for_rest`).
#[derive(Debug)]
pub struct AdminAuth(pub Option<AuthState>);

impl FromRequestParts<VectorizerServer> for AdminAuth {
    type Rejection = ErrorResponse;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &VectorizerServer,
    ) -> Result<Self, Self::Rejection> {
        extract_admin(state.auth_handler_state.as_ref(), &parts.headers).await
    }
}

/// Any-authenticated extractor (companion to `AdminAuth`).
///
/// Succeeds when the caller presents a valid JWT or API key, regardless
/// of role. Fails with `401` otherwise. When auth is globally disabled
/// the extractor yields `Authenticated(None)` — callers that need to
/// distinguish that case should inspect the inner `Option`.
#[derive(Debug)]
pub struct Authenticated(pub Option<AuthState>);

impl FromRequestParts<VectorizerServer> for Authenticated {
    type Rejection = ErrorResponse;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &VectorizerServer,
    ) -> Result<Self, Self::Rejection> {
        extract_authenticated(state.auth_handler_state.as_ref(), &parts.headers).await
    }
}

/// Internal testable helper for `AdminAuth`. Split out so unit tests
/// can exercise it without standing up a full `VectorizerServer`.
pub(crate) async fn extract_admin(
    auth_handler_state: Option<&AuthHandlerState>,
    headers: &HeaderMap,
) -> Result<AdminAuth, ErrorResponse> {
    let Some(ahs) = auth_handler_state else {
        return Ok(AdminAuth(None));
    };

    match require_admin_from_headers(ahs, headers).await {
        Ok(auth) => Ok(AdminAuth(Some(auth))),
        Err((status, Json(err))) => Err(ErrorResponse::new(err.error, err.message, status)),
    }
}

/// Internal testable helper for `Authenticated`.
pub(crate) async fn extract_authenticated(
    auth_handler_state: Option<&AuthHandlerState>,
    headers: &HeaderMap,
) -> Result<Authenticated, ErrorResponse> {
    let Some(ahs) = auth_handler_state else {
        return Ok(Authenticated(None));
    };

    match validate_authenticated(ahs, headers).await {
        Ok(auth) => Ok(Authenticated(Some(auth))),
        Err((status, err)) => Err(ErrorResponse::new(err.error, err.message, status)),
    }
}

async fn validate_authenticated(
    state: &AuthHandlerState,
    headers: &HeaderMap,
) -> Result<AuthState, (axum::http::StatusCode, AuthErrorResponse)> {
    use axum::http::StatusCode;
    use axum::http::header::AUTHORIZATION;

    let unauthenticated = || {
        (
            StatusCode::UNAUTHORIZED,
            AuthErrorResponse {
                error: "unauthorized".to_string(),
                message: "Authentication required. Provide a valid JWT token or API key."
                    .to_string(),
            },
        )
    };

    let mut auth_state: Option<AuthState> = None;

    if let Some(auth_header) = headers.get(AUTHORIZATION)
        && let Ok(auth_str) = auth_header.to_str()
    {
        if let Some(token) = auth_str.strip_prefix("Bearer ") {
            if !state.is_token_blacklisted(token).await
                && let Ok(claims) = state.auth_manager.validate_jwt(token)
            {
                auth_state = Some(AuthState {
                    user_claims: claims,
                    authenticated: true,
                });
            }
        } else if let Ok(claims) = state.auth_manager.validate_api_key(auth_str).await {
            auth_state = Some(AuthState {
                user_claims: claims,
                authenticated: true,
            });
        }
    }

    if auth_state.is_none()
        && let Some(api_key_header) = headers.get("X-API-Key")
        && let Ok(api_key) = api_key_header.to_str()
        && let Ok(claims) = state.auth_manager.validate_api_key(api_key).await
    {
        auth_state = Some(AuthState {
            user_claims: claims,
            authenticated: true,
        });
    }

    auth_state.ok_or_else(unauthenticated)
}
