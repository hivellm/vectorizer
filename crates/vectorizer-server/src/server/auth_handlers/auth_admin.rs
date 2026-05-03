//! Extended `/auth/*` admin handlers.
//!
//! Six routes live here, all admin-gated except `introspect_token`:
//!
//! - `POST /auth/keys` (scoped variant) — create an API key with
//!   per-collection scopes (extends the basic creation handler in
//!   `authenticated.rs`).
//! - `POST /auth/keys/{id}/rotate` — issue a successor key, mark the
//!   old one with a grace window so deployed clients can roll over.
//! - `PUT /auth/keys/{id}/permissions` — replace `permissions` (and
//!   optionally `scopes`) on an existing key without rotating the
//!   credential. `key_hash`, `id`, `user_id`, `created_at` are
//!   immutable.
//! - `GET /auth/keys/{id}/usage?window=<n>` — return the per-day
//!   counter ring for the last `window` days (default 7, max 30) plus
//!   the live `usage_count`.
//! - `POST /auth/introspect` — RFC 7662 token introspection. Requires
//!   authentication but not admin.
//! - `GET /auth/audit` — query the in-memory admin-action audit log.

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
use vectorizer::monitoring::api_key_usage::UsageBucket;

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

/// Request body for `PUT /auth/keys/{id}/permissions`.
#[derive(Debug, Deserialize)]
pub struct UpdateApiKeyPermissionsRequest {
    /// New permission list. Rejected with 400 when empty — operators
    /// revoke a key with `DELETE /auth/keys/{id}` instead of stripping
    /// it of permissions.
    pub permissions: Vec<String>,
    /// `None` leaves the existing scopes untouched. `Some([])` clears
    /// scopes (default-deny on scope-aware routes).
    #[serde(default)]
    pub scopes: Option<Vec<ScopeDto>>,
}

/// Flattened view of an `ApiKey` returned by both the permission-update
/// endpoint and the usage endpoint. Drops `key_hash` from the wire — no
/// caller needs the hash.
#[derive(Debug, Serialize)]
pub struct ApiKeyView {
    pub id: String,
    pub name: String,
    pub user_id: String,
    pub permissions: Vec<String>,
    pub scopes: Vec<ScopeDto>,
    pub created_at: u64,
    pub last_used: Option<u64>,
    pub expires_at: Option<u64>,
    pub active: bool,
    pub usage_count: u64,
}

/// Query parameters for `GET /auth/keys/{id}/usage`.
#[derive(Debug, Deserialize, Default)]
pub struct UsageQueryParams {
    /// Number of days back from today. Defaults to 7, clamped to 30.
    pub window: Option<usize>,
}

/// Response body for `GET /auth/keys/{id}/usage`.
#[derive(Debug, Serialize)]
pub struct ApiKeyUsageResponse {
    pub key: ApiKeyView,
    /// Daily counter buckets, oldest first, exactly `window` long.
    /// Days with zero validations are still included so the SPA can
    /// render a continuous sparkline without gap-fill logic.
    pub buckets: Vec<UsageBucket>,
    /// Total validations across all returned buckets.
    pub window_total: u64,
}

const MAX_USAGE_WINDOW_DAYS: usize = 30;
const DEFAULT_USAGE_WINDOW_DAYS: usize = 7;

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

fn into_view(key: vectorizer::auth::ApiKey) -> ApiKeyView {
    ApiKeyView {
        id: key.id,
        name: key.name,
        user_id: key.user_id,
        permissions: key.permissions.iter().map(|p| format!("{:?}", p)).collect(),
        scopes: key
            .scopes
            .iter()
            .map(|s| ScopeDto {
                collection: s.collection.clone(),
                permissions: s.permissions.clone(),
            })
            .collect(),
        created_at: key.created_at,
        last_used: key.last_used,
        expires_at: key.expires_at,
        active: key.active,
        usage_count: key.usage_count,
    }
}

// ---------------------------------------------------------------------------
// POST /auth/keys (scoped variant)
// ---------------------------------------------------------------------------

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

/// Generates a successor key, marks the old key with a grace window
/// (default 300 s), and returns both tokens so the client can migrate
/// without downtime.
pub async fn rotate_api_key(
    State(state): State<AuthHandlerState>,
    Extension(auth_state): Extension<AuthState>,
    Path(id): Path<String>,
) -> Result<Json<RotateApiKeyResponse>, (StatusCode, Json<AuthErrorResponse>)> {
    require_admin(&auth_state)?;

    state
        .auth_manager
        .get_api_key_info(&id)
        .await
        .map_err(|_| not_found_err(&format!("API key {} not found", id)))?;

    let grace_secs: u64 = 300;
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
// PUT /auth/keys/{id}/permissions
// ---------------------------------------------------------------------------

/// Replace permissions (and optionally scopes) on an existing key
/// without rotating the credential.
pub async fn update_api_key_permissions(
    State(state): State<AuthHandlerState>,
    Extension(auth_state): Extension<AuthState>,
    Path(id): Path<String>,
    Json(request): Json<UpdateApiKeyPermissionsRequest>,
) -> Result<Json<ApiKeyView>, (StatusCode, Json<AuthErrorResponse>)> {
    require_admin(&auth_state)?;

    if request.permissions.is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(AuthErrorResponse {
                error: "invalid_permissions".to_string(),
                message: "permissions must contain at least one entry; revoke the key with \
                          DELETE /auth/keys/{id} if you want to disable it"
                    .to_string(),
            }),
        ));
    }

    let permissions = parse_permissions(&request.permissions);
    if permissions.is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(AuthErrorResponse {
                error: "invalid_permissions".to_string(),
                message: format!(
                    "no recognized permission in {:?}; valid: read, write, delete, \
                     create_collection, delete_collection, manage_users, manage_api_keys, \
                     view_logs, system_config",
                    request.permissions
                ),
            }),
        ));
    }

    let scopes = request.scopes.as_ref().map(|raw| {
        raw.iter()
            .map(|s| TokenScope {
                collection: s.collection.clone(),
                permissions: s.permissions.clone(),
            })
            .collect::<Vec<_>>()
    });

    let updated = state
        .auth_manager
        .update_api_key_permissions(&id, permissions, scopes)
        .await
        .map_err(|e| not_found_err(&format!("API key {} not found: {}", id, e)))?;

    state.audit_logger.record(
        &auth_state.user_claims.username,
        "update_api_key_permissions",
        &id,
        vectorizer::monitoring::current_correlation_id(),
    );

    info!(
        "API key '{}' permissions updated by admin '{}'",
        id, auth_state.user_claims.username
    );

    Ok(Json(into_view(updated)))
}

// ---------------------------------------------------------------------------
// GET /auth/keys/{id}/usage
// ---------------------------------------------------------------------------

/// Returns the per-day counter ring for the last `window` days, the
/// total over that window, and the live `usage_count` from the
/// in-memory atomic stamped onto `key.usage_count`.
pub async fn get_api_key_usage(
    State(state): State<AuthHandlerState>,
    Extension(auth_state): Extension<AuthState>,
    Path(id): Path<String>,
    Query(params): Query<UsageQueryParams>,
) -> Result<Json<ApiKeyUsageResponse>, (StatusCode, Json<AuthErrorResponse>)> {
    require_admin(&auth_state)?;

    let window = params
        .window
        .unwrap_or(DEFAULT_USAGE_WINDOW_DAYS)
        .clamp(1, MAX_USAGE_WINDOW_DAYS);

    let key = state
        .auth_manager
        .get_api_key_info(&id)
        .await
        .map_err(|e| not_found_err(&format!("API key {} not found: {}", id, e)))?;

    let recorder = state.auth_manager.usage_recorder();
    let buckets = match recorder.as_ref() {
        Some(r) => r.snapshot(&id, window),
        None => Vec::new(),
    };
    let window_total: u64 = buckets.iter().map(|b| b.count).sum();

    Ok(Json(ApiKeyUsageResponse {
        key: into_view(key),
        buckets,
        window_total,
    }))
}

// ---------------------------------------------------------------------------
// POST /auth/introspect
// ---------------------------------------------------------------------------

/// RFC 7662 token introspection.  Returns `{ active, scope, sub, exp }`
/// for any token presented in the request body. Requires the caller to
/// be authenticated (but not admin).
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
