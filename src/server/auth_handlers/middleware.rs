//! Axum middlewares and auth-extraction helpers.
//!
//! Five entry points are exposed:
//!
//! - [`auth_middleware`] — annotate every request with an `AuthState`
//!   extension (anonymous allowed). Use for optionally-authenticated routes.
//! - [`require_auth_middleware`] — 401 if the caller has no valid JWT /
//!   API key; otherwise installs the `AuthState` extension.
//! - [`require_admin_middleware`] — 401 without auth, 403 without
//!   `Role::Admin`; otherwise installs the `AuthState`.
//! - [`require_admin_from_headers`] — handler-level admin gate used when
//!   router-level middleware doesn't fit (e.g. when the handler needs
//!   to branch on auth presence vs. return a preformatted error).
//! - [`require_admin_for_rest`] — the REST-flavoured variant that
//!   returns [`crate::server::error_middleware::ErrorResponse`] and
//!   treats a globally-disabled auth subsystem as "every caller is admin"
//!   for backward compatibility with single-user local setups.
//!
//! [`extract_auth_from_request`] does the header parsing (Authorization,
//! X-API-Key, `?api_key=` query) and is shared by all of the above.

use axum::extract::State;
use axum::response::Json;
use tracing::debug;

use super::state::AuthHandlerState;
use super::types::AuthErrorResponse;
use crate::auth::UserClaims;
use crate::auth::middleware::AuthState;
use crate::auth::roles::Role;

/// Authentication middleware that extracts auth state from request
/// and adds it to request extensions
pub async fn auth_middleware(
    State(state): State<AuthHandlerState>,
    mut request: axum::extract::Request,
    next: axum::middleware::Next,
) -> axum::response::Response {
    let headers = request.headers().clone();
    let query = request.uri().query().map(str::to_string);
    let auth_state = extract_auth_from_parts(&state, &headers, query.as_deref()).await;

    request.extensions_mut().insert(auth_state);

    next.run(request).await
}

/// Middleware that requires authentication - returns 401 if not authenticated
pub async fn require_auth_middleware(
    State(state): State<AuthHandlerState>,
    mut request: axum::extract::Request,
    next: axum::middleware::Next,
) -> axum::response::Response {
    use axum::http::StatusCode;
    use axum::response::IntoResponse;

    let headers = request.headers().clone();
    let query = request.uri().query().map(str::to_string);
    let auth_state = extract_auth_from_parts(&state, &headers, query.as_deref()).await;

    if !auth_state.authenticated {
        return (
            StatusCode::UNAUTHORIZED,
            Json(AuthErrorResponse {
                error: "unauthorized".to_string(),
                message: "Authentication required. Provide a valid JWT token or API key."
                    .to_string(),
            }),
        )
            .into_response();
    }

    request.extensions_mut().insert(auth_state);

    next.run(request).await
}

/// Middleware that requires admin role - returns 403 if not admin
pub async fn require_admin_middleware(
    State(state): State<AuthHandlerState>,
    mut request: axum::extract::Request,
    next: axum::middleware::Next,
) -> axum::response::Response {
    use axum::http::StatusCode;
    use axum::response::IntoResponse;

    let headers = request.headers().clone();
    let query = request.uri().query().map(str::to_string);
    let auth_state = extract_auth_from_parts(&state, &headers, query.as_deref()).await;

    if !auth_state.authenticated {
        return (
            StatusCode::UNAUTHORIZED,
            Json(AuthErrorResponse {
                error: "unauthorized".to_string(),
                message: "Authentication required".to_string(),
            }),
        )
            .into_response();
    }

    let is_admin = auth_state.user_claims.roles.contains(&Role::Admin);
    if !is_admin {
        return (
            StatusCode::FORBIDDEN,
            Json(AuthErrorResponse {
                error: "forbidden".to_string(),
                message: "Admin access required".to_string(),
            }),
        )
            .into_response();
    }

    request.extensions_mut().insert(auth_state);

    next.run(request).await
}

/// Handler-level admin gate. Prefer this over router-level middleware for
/// admin-only endpoints: it compiles against any handler state type, is
/// cheap at call time, and keeps the admin check next to the handler body.
///
/// Returns `Ok(AuthState)` only when the caller presents a valid JWT / API
/// key AND the claims contain `Role::Admin`. Returns a preformatted 401 or
/// 403 response otherwise.
pub async fn require_admin_from_headers(
    state: &AuthHandlerState,
    headers: &axum::http::HeaderMap,
) -> Result<AuthState, (axum::http::StatusCode, Json<AuthErrorResponse>)> {
    use axum::http::StatusCode;
    use axum::http::header::AUTHORIZATION;

    let unauthenticated = || {
        (
            StatusCode::UNAUTHORIZED,
            Json(AuthErrorResponse {
                error: "unauthorized".to_string(),
                message: "Authentication required. Provide a valid JWT token or API key."
                    .to_string(),
            }),
        )
    };
    let forbidden = || {
        (
            StatusCode::FORBIDDEN,
            Json(AuthErrorResponse {
                error: "forbidden".to_string(),
                message: "Admin role required for this endpoint.".to_string(),
            }),
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

    let Some(auth) = auth_state else {
        return Err(unauthenticated());
    };
    if !auth.user_claims.roles.contains(&Role::Admin) {
        return Err(forbidden());
    }
    Ok(auth)
}

/// REST-flavored variant of `require_admin_from_headers`.
///
/// Backward-compat behavior: if the server booted with no
/// `AuthHandlerState` (auth globally disabled), every caller is treated
/// as admin — preserves single-user local setups that never configured
/// authentication. When auth IS configured, admin role is enforced.
pub async fn require_admin_for_rest(
    auth_handler_state: &Option<AuthHandlerState>,
    headers: &axum::http::HeaderMap,
) -> Result<(), crate::server::error_middleware::ErrorResponse> {
    use crate::server::error_middleware::ErrorResponse;

    let Some(state) = auth_handler_state else {
        return Ok(());
    };

    match require_admin_from_headers(state, headers).await {
        Ok(_) => Ok(()),
        Err((status, Json(err))) => Err(ErrorResponse::new(err.error, err.message, status)),
    }
}

/// Extract authentication state from a request.
///
/// Takes the headers + query string by reference rather than
/// `&axum::extract::Request` so the resulting future is `Send`. The
/// `Request` body type is a `dyn HttpBody` trait object that is `Send`
/// but not `Sync`, which makes any `&Request` held across an `.await`
/// non-Send and breaks `axum::middleware::from_fn` composition. Splitting
/// the borrowed surface into `&HeaderMap` + `Option<&str>` (both
/// `Send + Sync`) sidesteps the whole problem and lets the same helper
/// be reused from both `axum::middleware`-style middleware and standalone
/// callers.
async fn extract_auth_from_parts(
    state: &AuthHandlerState,
    headers: &axum::http::HeaderMap,
    query: Option<&str>,
) -> AuthState {
    use axum::http::header::AUTHORIZATION;

    if let Some(auth_header) = headers.get(AUTHORIZATION)
        && let Ok(auth_str) = auth_header.to_str()
    {
        if let Some(token) = auth_str.strip_prefix("Bearer ") {
            if state.is_token_blacklisted(token).await {
                debug!("Token is blacklisted, rejecting authentication");
                return anonymous_auth_state();
            }
            if let Ok(claims) = state.auth_manager.validate_jwt(token) {
                return AuthState {
                    user_claims: claims,
                    authenticated: true,
                };
            }
        }

        if let Ok(claims) = state.auth_manager.validate_api_key(auth_str).await {
            return AuthState {
                user_claims: claims,
                authenticated: true,
            };
        }
    }

    if let Some(api_key_header) = headers.get("X-API-Key")
        && let Ok(api_key) = api_key_header.to_str()
        && let Ok(claims) = state.auth_manager.validate_api_key(api_key).await
    {
        return AuthState {
            user_claims: claims,
            authenticated: true,
        };
    }

    if let Some(query) = query {
        for param in query.split('&') {
            if let Some(api_key) = param.strip_prefix("api_key=")
                && let Ok(claims) = state.auth_manager.validate_api_key(api_key).await
            {
                return AuthState {
                    user_claims: claims,
                    authenticated: true,
                };
            }
        }
    }

    anonymous_auth_state()
}

fn anonymous_auth_state() -> AuthState {
    AuthState {
        user_claims: UserClaims {
            user_id: "anonymous".to_string(),
            username: "anonymous".to_string(),
            roles: vec![],
            iat: 0,
            exp: 0,
        },
        authenticated: false,
    }
}
