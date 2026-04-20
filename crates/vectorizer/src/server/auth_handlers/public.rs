//! Public endpoints — no authentication required.
//!
//! - `POST /auth/login` mints a JWT for valid credentials, rate-limits by
//!   username, and hashes failed attempts equally for missing-user and
//!   wrong-password cases to avoid enumeration side-channels.
//! - `POST /auth/validate-password` is a pure strength/complexity check.
//!   It does NOT persist anything and is safe to expose to unauthenticated
//!   clients (e.g. sign-up forms showing live strength feedback).

use axum::extract::State;
use axum::http::StatusCode;
use axum::response::Json;
use tracing::{debug, error, info, warn};

use super::state::AuthHandlerState;
use super::types::{
    AuthErrorResponse, LoginRequest, LoginResponse, UserInfo, ValidatePasswordRequest,
    ValidatePasswordResponse,
};

/// Validate password strength - POST /auth/validate-password
///
/// This is a public endpoint (no auth required) that checks password
/// complexity and returns strength information.
pub async fn validate_password_endpoint(
    Json(request): Json<ValidatePasswordRequest>,
) -> Json<ValidatePasswordResponse> {
    let result = crate::auth::validate_password(&request.password);

    Json(ValidatePasswordResponse {
        valid: result.valid,
        errors: result.errors,
        strength: result.strength,
        strength_label: result.strength_label,
    })
}

/// Login endpoint - POST /auth/login
pub async fn login(
    State(state): State<AuthHandlerState>,
    Json(request): Json<LoginRequest>,
) -> Result<Json<LoginResponse>, (StatusCode, Json<AuthErrorResponse>)> {
    debug!("Login attempt for user: {}", request.username);

    // Check rate limiting first
    if let Err((attempts, remaining_secs)) = state.check_login_rate_limit(&request.username).await {
        warn!(
            "Login rate limited for user '{}': {} attempts, {} seconds remaining",
            request.username, attempts, remaining_secs
        );
        return Err((
            StatusCode::TOO_MANY_REQUESTS,
            Json(AuthErrorResponse {
                error: "rate_limited".to_string(),
                message: format!(
                    "Too many login attempts. Please try again in {} seconds.",
                    remaining_secs
                ),
            }),
        ));
    }

    // Look up user
    let users = state.users.read().await;
    let user = match users.get(&request.username) {
        Some(u) => u.clone(),
        None => {
            warn!("Login failed: user '{}' not found", request.username);
            // Record failed attempt even for non-existent users (prevent enumeration)
            drop(users);
            state.record_failed_login(&request.username).await;
            return Err((
                StatusCode::UNAUTHORIZED,
                Json(AuthErrorResponse {
                    error: "invalid_credentials".to_string(),
                    message: "Invalid username or password".to_string(),
                }),
            ));
        }
    };
    drop(users);

    // Verify password
    let valid =
        bcrypt::verify(&request.password, user.password_hash.expose_secret()).unwrap_or(false);
    if !valid {
        warn!(
            "Login failed: invalid password for user '{}'",
            request.username
        );
        // Record failed attempt
        state.record_failed_login(&request.username).await;
        return Err((
            StatusCode::UNAUTHORIZED,
            Json(AuthErrorResponse {
                error: "invalid_credentials".to_string(),
                message: "Invalid username or password".to_string(),
            }),
        ));
    }

    // Clear failed attempts on successful login
    state.clear_login_attempts(&request.username).await;

    // Generate JWT token
    let token = state
        .auth_manager
        .generate_jwt(&user.user_id, &user.username, user.roles.clone())
        .map_err(|e| {
            error!("Failed to generate JWT: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(AuthErrorResponse {
                    error: "token_error".to_string(),
                    message: "Failed to generate access token".to_string(),
                }),
            )
        })?;

    info!("User '{}' logged in successfully", request.username);

    Ok(Json(LoginResponse {
        access_token: token,
        token_type: "Bearer".to_string(),
        expires_in: state.auth_manager.config().jwt_expiration,
        user: UserInfo {
            user_id: user.user_id.clone(),
            username: user.username.clone(),
            roles: user.roles.iter().map(|r| format!("{:?}", r)).collect(),
        },
    }))
}
