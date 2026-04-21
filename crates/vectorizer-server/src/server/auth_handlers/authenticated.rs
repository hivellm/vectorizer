//! Authenticated (non-admin) endpoints.
//!
//! Every handler here pulls `AuthState` from a request extension and
//! short-circuits with a 401 if `authenticated == false`. The extension
//! is populated by the [`auth_middleware`](super::middleware::auth_middleware)
//! layer registered on the protected router.
//!
//! Endpoints:
//! - `GET  /auth/me` — return the calling user's claims
//! - `POST /auth/logout` — blacklist the presented JWT until natural expiry
//! - `POST /auth/refresh` — mint a fresh JWT for the current claims
//! - `POST /auth/keys` — create an API key for the calling user
//! - `GET  /auth/keys` — list the calling user's API keys
//! - `DELETE /auth/keys/{id}` — revoke an API key (must belong to caller or admin)

use axum::Extension;
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::Json;
use tracing::{error, info};

use super::state::AuthHandlerState;
use super::types::{
    ApiKeyInfo, AuthErrorResponse, CreateApiKeyRequest, CreateApiKeyResponse, ListApiKeysResponse,
    LogoutResponse, RefreshTokenResponse, UserInfo,
};
use vectorizer::auth::middleware::AuthState;
use vectorizer::auth::persistence::PersistedApiKey;
use vectorizer::auth::roles::{Permission, Role};

/// Get current user info - GET /auth/me
pub async fn get_me(
    Extension(auth_state): Extension<AuthState>,
) -> Result<Json<UserInfo>, (StatusCode, Json<AuthErrorResponse>)> {
    if !auth_state.authenticated {
        return Err((
            StatusCode::UNAUTHORIZED,
            Json(AuthErrorResponse {
                error: "unauthorized".to_string(),
                message: "Authentication required".to_string(),
            }),
        ));
    }

    Ok(Json(UserInfo {
        user_id: auth_state.user_claims.user_id,
        username: auth_state.user_claims.username,
        roles: auth_state
            .user_claims
            .roles
            .iter()
            .map(|r| format!("{:?}", r))
            .collect(),
    }))
}

/// Logout endpoint - POST /auth/logout
///
/// Invalidates the current JWT token by adding it to a blacklist.
/// The token will remain invalid until it expires naturally.
pub async fn logout(
    State(state): State<AuthHandlerState>,
    Extension(auth_state): Extension<AuthState>,
    request: axum::extract::Request,
) -> Result<Json<LogoutResponse>, (StatusCode, Json<AuthErrorResponse>)> {
    if !auth_state.authenticated {
        return Err((
            StatusCode::UNAUTHORIZED,
            Json(AuthErrorResponse {
                error: "unauthorized".to_string(),
                message: "Authentication required".to_string(),
            }),
        ));
    }

    // Extract the token from the Authorization header to blacklist it
    if let Some(auth_header) = request.headers().get(axum::http::header::AUTHORIZATION) {
        if let Ok(auth_str) = auth_header.to_str() {
            if let Some(token) = auth_str.strip_prefix("Bearer ") {
                state.blacklist_token(token.to_string()).await;
                info!(
                    "User '{}' logged out, token blacklisted",
                    auth_state.user_claims.username
                );
            }
        }
    }

    Ok(Json(LogoutResponse {
        status: "ok".to_string(),
        message: "Logged out successfully".to_string(),
    }))
}

/// Refresh token endpoint - POST /auth/refresh
///
/// Generates a new JWT token with extended expiration.
/// The old token remains valid until it expires (unless logout is called).
pub async fn refresh_token(
    State(state): State<AuthHandlerState>,
    Extension(auth_state): Extension<AuthState>,
    request: axum::extract::Request,
) -> Result<Json<RefreshTokenResponse>, (StatusCode, Json<AuthErrorResponse>)> {
    if !auth_state.authenticated {
        return Err((
            StatusCode::UNAUTHORIZED,
            Json(AuthErrorResponse {
                error: "unauthorized".to_string(),
                message: "Authentication required".to_string(),
            }),
        ));
    }

    // Get the current token from Authorization header
    let current_token = request
        .headers()
        .get(axum::http::header::AUTHORIZATION)
        .and_then(|h| h.to_str().ok())
        .and_then(|s| s.strip_prefix("Bearer "))
        .map(|s| s.to_string());

    // Check if the current token is blacklisted
    if let Some(ref token) = current_token {
        if state.is_token_blacklisted(token).await {
            return Err((
                StatusCode::UNAUTHORIZED,
                Json(AuthErrorResponse {
                    error: "token_revoked".to_string(),
                    message: "Token has been revoked. Please login again.".to_string(),
                }),
            ));
        }
    }

    // Generate a new token with same user info but fresh expiration
    let new_token = state
        .auth_manager
        .generate_jwt(
            &auth_state.user_claims.user_id,
            &auth_state.user_claims.username,
            auth_state.user_claims.roles.clone(),
        )
        .map_err(|e| {
            error!("Failed to generate refresh token: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(AuthErrorResponse {
                    error: "token_error".to_string(),
                    message: "Failed to generate new access token".to_string(),
                }),
            )
        })?;

    info!(
        "Token refreshed for user '{}'",
        auth_state.user_claims.username
    );

    Ok(Json(RefreshTokenResponse {
        access_token: new_token,
        token_type: "Bearer".to_string(),
        expires_in: state.auth_manager.config().jwt_expiration,
    }))
}

/// Create API key - POST /auth/keys
pub async fn create_api_key(
    State(state): State<AuthHandlerState>,
    Extension(auth_state): Extension<AuthState>,
    Json(request): Json<CreateApiKeyRequest>,
) -> Result<Json<CreateApiKeyResponse>, (StatusCode, Json<AuthErrorResponse>)> {
    if !auth_state.authenticated {
        return Err((
            StatusCode::UNAUTHORIZED,
            Json(AuthErrorResponse {
                error: "unauthorized".to_string(),
                message: "Authentication required".to_string(),
            }),
        ));
    }

    // Parse permissions
    let permissions: Vec<Permission> = request
        .permissions
        .iter()
        .filter_map(|p| match p.to_lowercase().as_str() {
            "read" => Some(Permission::Read),
            "write" => Some(Permission::Write),
            "delete" => Some(Permission::Delete),
            "create_collection" => Some(Permission::CreateCollection),
            "delete_collection" => Some(Permission::DeleteCollection),
            "manage_users" => Some(Permission::ManageUsers),
            "manage_api_keys" => Some(Permission::ManageApiKeys),
            "view_logs" => Some(Permission::ViewLogs),
            "system_config" => Some(Permission::SystemConfig),
            _ => None,
        })
        .collect();

    let permissions = if permissions.is_empty() {
        vec![Permission::Read]
    } else {
        permissions
    };

    // Calculate expiration
    let expires_at = request
        .expires_in
        .map(|secs| chrono::Utc::now().timestamp() as u64 + secs);

    // Create the API key
    let (api_key, key_info) = state
        .auth_manager
        .create_api_key(
            &auth_state.user_claims.user_id,
            &request.name,
            permissions.clone(),
            expires_at,
        )
        .await
        .map_err(|e| {
            error!("Failed to create API key: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(AuthErrorResponse {
                    error: "key_creation_failed".to_string(),
                    message: format!("Failed to create API key: {}", e),
                }),
            )
        })?;

    // Persist the API key to disk (encrypted)
    let persisted_key = PersistedApiKey {
        id: key_info.id.clone(),
        name: key_info.name.clone(),
        key_hash: key_info.key_hash.clone(),
        user_id: key_info.user_id.clone(),
        permissions: key_info.permissions.clone(),
        created_at: key_info.created_at,
        last_used: key_info.last_used,
        expires_at: key_info.expires_at,
        active: key_info.active,
    };
    if let Err(e) = state.persistence.save_api_key(persisted_key) {
        error!("Failed to persist API key to disk: {}", e);
        // Continue anyway - key is in memory, just won't survive restart
    }

    info!(
        "API key '{}' created for user '{}'",
        request.name, auth_state.user_claims.user_id
    );

    Ok(Json(CreateApiKeyResponse {
        api_key,
        id: key_info.id,
        name: key_info.name,
        permissions: permissions.iter().map(|p| format!("{:?}", p)).collect(),
        expires_at: key_info.expires_at,
        warning: "Save this API key now! It will not be shown again.".to_string(),
    }))
}

/// List API keys - GET /auth/keys
pub async fn list_api_keys(
    State(state): State<AuthHandlerState>,
    Extension(auth_state): Extension<AuthState>,
) -> Result<Json<ListApiKeysResponse>, (StatusCode, Json<AuthErrorResponse>)> {
    if !auth_state.authenticated {
        return Err((
            StatusCode::UNAUTHORIZED,
            Json(AuthErrorResponse {
                error: "unauthorized".to_string(),
                message: "Authentication required".to_string(),
            }),
        ));
    }

    let keys = state
        .auth_manager
        .list_api_keys(&auth_state.user_claims.user_id)
        .await
        .map_err(|e| {
            error!("Failed to list API keys: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(AuthErrorResponse {
                    error: "list_failed".to_string(),
                    message: format!("Failed to list API keys: {}", e),
                }),
            )
        })?;

    let keys = keys
        .into_iter()
        .map(|k| ApiKeyInfo {
            id: k.id,
            name: k.name,
            permissions: k.permissions.iter().map(|p| format!("{:?}", p)).collect(),
            created_at: k.created_at,
            last_used: k.last_used,
            expires_at: k.expires_at,
            active: k.active,
        })
        .collect();

    Ok(Json(ListApiKeysResponse { keys }))
}

/// Revoke API key - DELETE /auth/keys/{id}
pub async fn revoke_api_key(
    State(state): State<AuthHandlerState>,
    Extension(auth_state): Extension<AuthState>,
    axum::extract::Path(key_id): axum::extract::Path<String>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<AuthErrorResponse>)> {
    if !auth_state.authenticated {
        return Err((
            StatusCode::UNAUTHORIZED,
            Json(AuthErrorResponse {
                error: "unauthorized".to_string(),
                message: "Authentication required".to_string(),
            }),
        ));
    }

    // Verify the key belongs to the user (or user is admin)
    let keys = state
        .auth_manager
        .list_api_keys(&auth_state.user_claims.user_id)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(AuthErrorResponse {
                    error: "list_failed".to_string(),
                    message: format!("Failed to verify key ownership: {}", e),
                }),
            )
        })?;

    let is_admin = auth_state.user_claims.roles.contains(&Role::Admin);
    let owns_key = keys.iter().any(|k| k.id == key_id);

    if !is_admin && !owns_key {
        return Err((
            StatusCode::FORBIDDEN,
            Json(AuthErrorResponse {
                error: "forbidden".to_string(),
                message: "You can only revoke your own API keys".to_string(),
            }),
        ));
    }

    state
        .auth_manager
        .revoke_api_key(&key_id)
        .await
        .map_err(|e| {
            error!("Failed to revoke API key: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(AuthErrorResponse {
                    error: "revoke_failed".to_string(),
                    message: format!("Failed to revoke API key: {}", e),
                }),
            )
        })?;

    // Remove from persistent storage
    if let Err(e) = state.persistence.remove_api_key(&key_id) {
        error!("Failed to remove API key from disk: {}", e);
        // Continue anyway - key is revoked in memory
    }

    info!(
        "API key '{}' revoked by user '{}'",
        key_id, auth_state.user_claims.user_id
    );

    Ok(Json(serde_json::json!({
        "status": "ok",
        "message": "API key revoked successfully"
    })))
}
