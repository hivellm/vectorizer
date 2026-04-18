//! Admin-gated user management endpoints.
//!
//! Every handler here enforces `Role::Admin` on the caller's claims in
//! addition to requiring authentication. `change_password` is a partial
//! exception: non-admin callers may change their OWN password by also
//! presenting `current_password`, but never someone else's.
//!
//! Endpoints:
//! - `POST   /auth/users` — create user (admin)
//! - `GET    /auth/users` — list users (admin)
//! - `DELETE /auth/users/{username}` — delete user (admin, refuses to
//!   delete self or the last admin)
//! - `PUT    /auth/users/{username}/password` — change password (admin
//!   for anyone; user for themselves with current-password challenge)

use axum::Extension;
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::Json;
use tracing::{error, info};

use super::state::{AuthHandlerState, UserRecord};
use super::types::{
    AuthErrorResponse, ChangePasswordRequest, CreateUserRequest, CreateUserResponse,
    ListUsersResponse, UserInfo,
};
use crate::auth::middleware::AuthState;
use crate::auth::persistence::PersistedUser;
use crate::auth::roles::Role;

/// Create user - POST /auth/users (admin only)
pub async fn create_user(
    State(state): State<AuthHandlerState>,
    Extension(auth_state): Extension<AuthState>,
    Json(request): Json<CreateUserRequest>,
) -> Result<Json<CreateUserResponse>, (StatusCode, Json<AuthErrorResponse>)> {
    // Require authentication
    if !auth_state.authenticated {
        return Err((
            StatusCode::UNAUTHORIZED,
            Json(AuthErrorResponse {
                error: "unauthorized".to_string(),
                message: "Authentication required".to_string(),
            }),
        ));
    }

    // Require admin role
    let is_admin = auth_state.user_claims.roles.contains(&Role::Admin);
    if !is_admin {
        return Err((
            StatusCode::FORBIDDEN,
            Json(AuthErrorResponse {
                error: "forbidden".to_string(),
                message: "Admin access required to create users".to_string(),
            }),
        ));
    }

    // Validate username
    if request.username.is_empty() || request.username.len() < 3 {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(AuthErrorResponse {
                error: "invalid_username".to_string(),
                message: "Username must be at least 3 characters".to_string(),
            }),
        ));
    }

    // Validate password complexity
    let password_validation = crate::auth::validate_password(&request.password);
    if !password_validation.valid {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(AuthErrorResponse {
                error: "invalid_password".to_string(),
                message: password_validation.errors.join(". "),
            }),
        ));
    }

    // Check if user already exists
    let users = state.users.read().await;
    if users.contains_key(&request.username) {
        return Err((
            StatusCode::CONFLICT,
            Json(AuthErrorResponse {
                error: "user_exists".to_string(),
                message: "User with this username already exists".to_string(),
            }),
        ));
    }
    drop(users);

    // Parse roles
    let roles: Vec<Role> = request
        .roles
        .iter()
        .filter_map(|r| match r.to_lowercase().as_str() {
            "admin" => Some(Role::Admin),
            "user" => Some(Role::User),
            "readonly" => Some(Role::ReadOnly),
            "apiuser" => Some(Role::ApiUser),
            _ => None,
        })
        .collect();

    let roles = if roles.is_empty() {
        vec![Role::User]
    } else {
        roles
    };

    // Hash password and wrap as Secret immediately so the plaintext hash does
    // not float around as a bare String.
    let password_hash = crate::auth::Secret::new(
        bcrypt::hash(&request.password, bcrypt::DEFAULT_COST).map_err(|e| {
            error!("Failed to hash password: {}", e); // logging-allow(label): "{}" is the bcrypt error, not the password
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(AuthErrorResponse {
                    error: "hash_failed".to_string(),
                    message: "Failed to create user".to_string(),
                }),
            )
        })?,
    );

    // Create user ID
    let user_id = uuid::Uuid::new_v4().to_string();

    // Create user record
    let user = UserRecord {
        user_id: user_id.clone(),
        username: request.username.clone(),
        password_hash: password_hash.clone(),
        roles: roles.clone(),
    };

    // Save to memory
    state
        .users
        .write()
        .await
        .insert(request.username.clone(), user);

    // Save to disk
    let persisted_user = PersistedUser {
        user_id: user_id.clone(),
        username: request.username.clone(),
        password_hash,
        roles: roles.clone(),
        created_at: chrono::Utc::now().timestamp() as u64,
        last_login: None,
    };

    if let Err(e) = state.persistence.save_user(persisted_user) {
        error!("Failed to persist user: {}", e);
        // Don't fail the request, user is in memory
    }

    info!(
        "User '{}' created by admin '{}'",
        request.username, auth_state.user_claims.username
    );

    Ok(Json(CreateUserResponse {
        user_id,
        username: request.username,
        roles: roles.iter().map(|r| format!("{:?}", r)).collect(),
        message: "User created successfully".to_string(),
    }))
}

/// List users - GET /auth/users (admin only)
pub async fn list_users(
    State(state): State<AuthHandlerState>,
    Extension(auth_state): Extension<AuthState>,
) -> Result<Json<ListUsersResponse>, (StatusCode, Json<AuthErrorResponse>)> {
    // Require authentication
    if !auth_state.authenticated {
        return Err((
            StatusCode::UNAUTHORIZED,
            Json(AuthErrorResponse {
                error: "unauthorized".to_string(),
                message: "Authentication required".to_string(),
            }),
        ));
    }

    // Require admin role
    let is_admin = auth_state.user_claims.roles.contains(&Role::Admin);
    if !is_admin {
        return Err((
            StatusCode::FORBIDDEN,
            Json(AuthErrorResponse {
                error: "forbidden".to_string(),
                message: "Admin access required to list users".to_string(),
            }),
        ));
    }

    let users = state.users.read().await;
    let user_list: Vec<UserInfo> = users
        .values()
        .map(|u| UserInfo {
            user_id: u.user_id.clone(),
            username: u.username.clone(),
            roles: u.roles.iter().map(|r| format!("{:?}", r)).collect(),
        })
        .collect();

    Ok(Json(ListUsersResponse { users: user_list }))
}

/// Delete user - DELETE /auth/users/{username} (admin only)
pub async fn delete_user(
    State(state): State<AuthHandlerState>,
    Extension(auth_state): Extension<AuthState>,
    axum::extract::Path(username): axum::extract::Path<String>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<AuthErrorResponse>)> {
    // Require authentication
    if !auth_state.authenticated {
        return Err((
            StatusCode::UNAUTHORIZED,
            Json(AuthErrorResponse {
                error: "unauthorized".to_string(),
                message: "Authentication required".to_string(),
            }),
        ));
    }

    // Require admin role
    let is_admin = auth_state.user_claims.roles.contains(&Role::Admin);
    if !is_admin {
        return Err((
            StatusCode::FORBIDDEN,
            Json(AuthErrorResponse {
                error: "forbidden".to_string(),
                message: "Admin access required to delete users".to_string(),
            }),
        ));
    }

    // Prevent deleting yourself
    if username == auth_state.user_claims.username {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(AuthErrorResponse {
                error: "cannot_delete_self".to_string(),
                message: "You cannot delete your own account".to_string(),
            }),
        ));
    }

    // Check if user exists and if they're the last admin
    let mut users = state.users.write().await;

    // Check if user exists
    let user_to_delete = match users.get(&username) {
        Some(user) => user.clone(),
        None => {
            return Err((
                StatusCode::NOT_FOUND,
                Json(AuthErrorResponse {
                    error: "user_not_found".to_string(),
                    message: "User not found".to_string(),
                }),
            ));
        }
    };

    // Prevent deleting the last admin user
    if user_to_delete.roles.contains(&Role::Admin) {
        let admin_count = users
            .values()
            .filter(|u| u.roles.contains(&Role::Admin))
            .count();
        if admin_count <= 1 {
            return Err((
                StatusCode::BAD_REQUEST,
                Json(AuthErrorResponse {
                    error: "cannot_delete_last_admin".to_string(),
                    message: "Cannot delete the last admin user. Create another admin first."
                        .to_string(),
                }),
            ));
        }
    }

    // Remove from memory
    users.remove(&username);
    drop(users);

    // Remove from disk
    if let Err(e) = state.persistence.remove_user(&username) {
        error!("Failed to remove user from disk: {}", e);
    }

    info!(
        "User '{}' deleted by admin '{}'",
        username, auth_state.user_claims.username
    );

    Ok(Json(serde_json::json!({
        "status": "ok",
        "message": format!("User '{}' deleted successfully", username)
    })))
}

/// Change password - PUT /auth/users/{username}/password
pub async fn change_password(
    State(state): State<AuthHandlerState>,
    Extension(auth_state): Extension<AuthState>,
    axum::extract::Path(username): axum::extract::Path<String>,
    Json(request): Json<ChangePasswordRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<AuthErrorResponse>)> {
    // Require authentication
    if !auth_state.authenticated {
        return Err((
            StatusCode::UNAUTHORIZED,
            Json(AuthErrorResponse {
                error: "unauthorized".to_string(),
                message: "Authentication required".to_string(),
            }),
        ));
    }

    let is_admin = auth_state.user_claims.roles.contains(&Role::Admin);
    let is_self = username == auth_state.user_claims.username;

    // Only admin or the user themselves can change password
    if !is_admin && !is_self {
        return Err((
            StatusCode::FORBIDDEN,
            Json(AuthErrorResponse {
                error: "forbidden".to_string(),
                message: "You can only change your own password".to_string(),
            }),
        ));
    }

    // Validate new password complexity
    let password_validation = crate::auth::validate_password(&request.new_password);
    if !password_validation.valid {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(AuthErrorResponse {
                error: "invalid_password".to_string(),
                message: password_validation.errors.join(". "),
            }),
        ));
    }

    // Get user
    let mut users = state.users.write().await;
    let user = users.get_mut(&username).ok_or_else(|| {
        (
            StatusCode::NOT_FOUND,
            Json(AuthErrorResponse {
                error: "user_not_found".to_string(),
                message: "User not found".to_string(),
            }),
        )
    })?;

    // If not admin, verify current password
    if !is_admin {
        let current_password = request.current_password.as_ref().ok_or_else(|| {
            (
                StatusCode::BAD_REQUEST,
                Json(AuthErrorResponse {
                    error: "missing_current_password".to_string(),
                    message: "Current password is required".to_string(),
                }),
            )
        })?;

        let valid =
            bcrypt::verify(current_password, user.password_hash.expose_secret()).unwrap_or(false);
        if !valid {
            return Err((
                StatusCode::UNAUTHORIZED,
                Json(AuthErrorResponse {
                    error: "invalid_password".to_string(),
                    message: "Current password is incorrect".to_string(),
                }),
            ));
        }
    }

    // Hash new password and wrap as Secret immediately.
    let new_hash = crate::auth::Secret::new(
        bcrypt::hash(&request.new_password, bcrypt::DEFAULT_COST).map_err(|e| {
            error!("Failed to hash password: {}", e); // logging-allow(label): "{}" is the bcrypt error, not the password
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(AuthErrorResponse {
                    error: "hash_failed".to_string(),
                    message: "Failed to update password".to_string(),
                }),
            )
        })?,
    );

    // Update password
    user.password_hash = new_hash.clone();

    // Save to disk
    let persisted_user = PersistedUser {
        user_id: user.user_id.clone(),
        username: user.username.clone(),
        password_hash: new_hash,
        roles: user.roles.clone(),
        created_at: chrono::Utc::now().timestamp() as u64,
        last_login: None,
    };

    drop(users);

    if let Err(e) = state.persistence.save_user(persisted_user) {
        error!("Failed to persist password change: {}", e); // logging-allow(label): "{}" is the I/O error, not the password
    }

    info!(
        "Password changed for user '{}' by '{}'",
        username, auth_state.user_claims.username
    );

    Ok(Json(serde_json::json!({
        "status": "ok",
        "message": "Password changed successfully"
    })))
}
