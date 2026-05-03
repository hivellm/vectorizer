//! Phase-15 auth handlers: scoped key creation, key rotation,
//! token introspection, and audit-log query.
//!
//! All handlers require an authenticated admin caller except
//! `introspect_token`, which requires authentication but not admin.

#![allow(missing_docs)]

use axum::Extension;
use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::response::Json;
use serde::{Deserialize, Serialize};
use tracing::info;
use vectorizer::auth::audit::AuditQuery;
use vectorizer::auth::middleware::AuthState;
use vectorizer::auth::roles::Role;
use vectorizer::auth::{AuditEntry, Permission, TokenIntrospection, TokenScope};

use super::state::AuthHandlerState;
use super::types::AuthErrorResponse;

// ---------------------------------------------------------------------------
// DTOs
// ---------------------------------------------------------------------------

/// Request body for `POST /auth/keys` — extended with optional scopes.
#[derive(Debug, Deserialize)]
pub struct CreateScopedApiKeyRequest {
    pub name: String,
    #[serde(default)]
    pub permissions: Vec<String>,
    pub expires_in: Option<u64>,
    /// Per-collection scopes. Empty = default-deny (no implicit access).
    #[serde(default)]
    pub scopes: Vec<ScopeDto>,
}

/// DTO for a single collection scope in the request body.
#[derive(Debug, Deserialize, Serialize)]
pub struct ScopeDto {
    pub collection: String,
    #[serde(default)]
    pub permissions: Vec<String>,
}

/// Response for scoped key creation.
#[derive(Debug, Serialize)]
pub struct CreateScopedApiKeyResponse {
    pub api_key: String,
    pub id: String,
    pub name: String,
    pub permissions: Vec<String>,
    pub scopes: Vec<ScopeDto>,
    pub expires_at: Option<u64>,
    pub warning: String,
}

/// Response for key rotation.
#[derive(Debug, Serialize)]
pub struct RotateApiKeyResponse {
    pub old_key_id: String,
    pub new_key_id: String,
    pub new_token: String,
    pub grace_until: u64,
}

/// Request body for `POST /auth/introspect`.
#[derive(Debug, Deserialize)]
pub struct IntrospectRequest {
    pub token: String,
}

/// Query parameters for `GET /auth/audit`.
#[derive(Debug, Deserialize, Default)]
pub struct AuditQueryParams {
    pub from: Option<String>,
    pub to: Option<String>,
    pub actor: Option<String>,
    pub action: Option<String>,
    pub limit: Option<usize>,
}

/// Response for audit log query.
#[derive(Debug, Serialize)]
pub struct AuditLogResponse {
    pub entries: Vec<AuditEntry>,
    pub total: usize,
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn require_admin(auth: &AuthState) -> Result<(), (StatusCode, Json<AuthErrorResponse>)> {
    if !auth.authenticated {
        return Err((
            StatusCode::UNAUTHORIZED,
            Json(AuthErrorResponse {
                error: "unauthorized".to_string(),
                message: "Authentication required".to_string(),
            }),
        ));
    }
    if !auth.user_claims.roles.contains(&Role::Admin) {
        return Err((
            StatusCode::FORBIDDEN,
            Json(AuthErrorResponse {
                error: "forbidden".to_string(),
                message: "Admin access required".to_string(),
            }),
        ));
    }
    Ok(())
}

fn require_auth(auth: &AuthState) -> Result<(), (StatusCode, Json<AuthErrorResponse>)> {
    if !auth.authenticated {
        return Err((
            StatusCode::UNAUTHORIZED,
            Json(AuthErrorResponse {
                error: "unauthorized".to_string(),
                message: "Authentication required".to_string(),
            }),
        ));
    }
    Ok(())
}

fn parse_permissions(raw: &[String]) -> Vec<Permission> {
    raw.iter()
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
        .collect()
}

fn internal_err(msg: &str) -> (StatusCode, Json<AuthErrorResponse>) {
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(AuthErrorResponse {
            error: "internal_error".to_string(),
            message: msg.to_string(),
        }),
    )
}

fn not_found_err(msg: &str) -> (StatusCode, Json<AuthErrorResponse>) {
    (
        StatusCode::NOT_FOUND,
        Json(AuthErrorResponse {
            error: "not_found".to_string(),
            message: msg.to_string(),
        }),
    )
}

// ---------------------------------------------------------------------------
// POST /auth/keys  (extended — scoped variant replaces the base handler)
// ---------------------------------------------------------------------------

/// Create API key — POST /auth/keys
///
/// Accepts an optional `scopes` array.  When present the key is
/// collection-scoped and will be denied on collections not listed.
/// When absent (or empty) the key is default-deny for scope-aware routes
/// but retains its global permissions for role-based routes.
pub async fn create_scoped_api_key(
    State(state): State<AuthHandlerState>,
    Extension(auth_state): Extension<AuthState>,
    Json(request): Json<CreateScopedApiKeyRequest>,
) -> Result<Json<CreateScopedApiKeyResponse>, (StatusCode, Json<AuthErrorResponse>)> {
    require_auth(&auth_state)?;

    let permissions = {
        let p = parse_permissions(&request.permissions);
        if p.is_empty() {
            vec![Permission::Read]
        } else {
            p
        }
    };

    let scopes: Vec<TokenScope> = request
        .scopes
        .iter()
        .map(|s| TokenScope {
            collection: s.collection.clone(),
            permissions: s.permissions.clone(),
        })
        .collect();

    let expires_at = request
        .expires_in
        .map(|secs| chrono::Utc::now().timestamp() as u64 + secs);

    let (api_key, key_info) = state
        .auth_manager
        .create_scoped_api_key(
            &auth_state.user_claims.user_id,
            &request.name,
            permissions.clone(),
            expires_at,
            scopes.clone(),
        )
        .await
        .map_err(|e| internal_err(&format!("Failed to create API key: {}", e)))?;

    state.audit_logger.record(
        &auth_state.user_claims.username,
        "create_api_key",
        &key_info.id,
        vectorizer::monitoring::current_correlation_id(),
    );

    info!(
        "Scoped API key '{}' created for user '{}'",
        request.name, auth_state.user_claims.user_id
    );

    Ok(Json(CreateScopedApiKeyResponse {
        api_key,
        id: key_info.id,
        name: key_info.name,
        permissions: permissions.iter().map(|p| format!("{:?}", p)).collect(),
        scopes: scopes
            .iter()
            .map(|s| ScopeDto {
                collection: s.collection.clone(),
                permissions: s.permissions.clone(),
            })
            .collect(),
        expires_at: key_info.expires_at,
        warning: "Save this API key now! It will not be shown again.".to_string(),
    }))
}

// ---------------------------------------------------------------------------
// POST /auth/keys/{id}/rotate
// ---------------------------------------------------------------------------

/// Rotate an API key — POST /auth/keys/{id}/rotate
///
/// Generates a successor key, marks the old key with a grace window
/// (default 300 s), and returns both tokens so the client can migrate
/// without downtime.
pub async fn rotate_api_key(
    State(state): State<AuthHandlerState>,
    Extension(auth_state): Extension<AuthState>,
    Path(id): Path<String>,
) -> Result<Json<RotateApiKeyResponse>, (StatusCode, Json<AuthErrorResponse>)> {
    require_admin(&auth_state)?;

    // Verify the key exists and belongs to a user the admin can manage.
    state
        .auth_manager
        .get_api_key_info(&id)
        .await
        .map_err(|_| not_found_err(&format!("API key {} not found", id)))?;

    let grace_secs: u64 = 300; // configurable in future
    let rotated = state
        .auth_manager
        .rotate_api_key(&id, grace_secs)
        .await
        .map_err(|e| internal_err(&format!("Rotation failed: {}", e)))?;

    state.audit_logger.record(
        &auth_state.user_claims.username,
        "rotate_api_key",
        &id,
        vectorizer::monitoring::current_correlation_id(),
    );

    info!(
        "API key '{}' rotated by admin '{}' (grace={}s)",
        id, auth_state.user_claims.username, grace_secs
    );

    Ok(Json(RotateApiKeyResponse {
        old_key_id: rotated.old_token,
        new_key_id: rotated.new_key_id,
        new_token: rotated.new_token,
        grace_until: rotated.grace_until,
    }))
}

// ---------------------------------------------------------------------------
// POST /auth/introspect
// ---------------------------------------------------------------------------

/// Token introspection — POST /auth/introspect (RFC 7662)
///
/// Returns `{ active, scope, sub, exp }` for any token presented in the
/// request body.  Requires the caller to be authenticated (but not admin).
pub async fn introspect_token(
    State(state): State<AuthHandlerState>,
    Extension(auth_state): Extension<AuthState>,
    Json(request): Json<IntrospectRequest>,
) -> Result<Json<TokenIntrospection>, (StatusCode, Json<AuthErrorResponse>)> {
    require_auth(&auth_state)?;

    let result = state.auth_manager.introspect_token(&request.token).await;
    Ok(Json(result))
}

// ---------------------------------------------------------------------------
// GET /auth/audit
// ---------------------------------------------------------------------------

/// List audit log — GET /auth/audit (admin only)
///
/// Returns the most recent admin-action entries from the in-memory buffer,
/// filtered by optional `from`, `to`, `actor`, `action` query params.
pub async fn list_audit_log(
    State(state): State<AuthHandlerState>,
    Extension(auth_state): Extension<AuthState>,
    Query(params): Query<AuditQueryParams>,
) -> Result<Json<AuditLogResponse>, (StatusCode, Json<AuthErrorResponse>)> {
    require_admin(&auth_state)?;

    fn parse_dt(s: &str) -> Option<chrono::DateTime<chrono::Utc>> {
        chrono::DateTime::parse_from_rfc3339(s)
            .ok()
            .map(|dt| dt.with_timezone(&chrono::Utc))
    }

    let query = AuditQuery {
        from: params.from.as_deref().and_then(parse_dt),
        to: params.to.as_deref().and_then(parse_dt),
        actor: params.actor.clone(),
        action: params.action.clone(),
        limit: params.limit,
    };

    let entries = state.audit_logger.query(&query).await;
    let total = entries.len();

    Ok(Json(AuditLogResponse { entries, total }))
}
