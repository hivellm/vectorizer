//! Authentication REST API handlers
//!
//! Provides endpoints for user authentication, API key management,
//! and user information retrieval.

use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use axum::Extension;
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::Json;
use serde::{Deserialize, Serialize};
use tracing::{debug, error, info, warn};

use crate::auth::middleware::AuthState;
use crate::auth::persistence::{AuthPersistence, PersistedUser};
use crate::auth::roles::{Permission, Role};
use crate::auth::{AuthManager, UserClaims};

/// Login request
#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    /// Username
    pub username: String,
    /// Password
    pub password: String,
}

/// Login response
#[derive(Debug, Serialize)]
pub struct LoginResponse {
    /// JWT access token
    pub access_token: String,
    /// Token type (always "Bearer")
    pub token_type: String,
    /// Token expiration in seconds
    pub expires_in: u64,
    /// User information
    pub user: UserInfo,
}

/// User information
#[derive(Debug, Serialize)]
pub struct UserInfo {
    /// User ID
    pub user_id: String,
    /// Username
    pub username: String,
    /// User roles
    pub roles: Vec<String>,
}

/// Create API key request
#[derive(Debug, Deserialize)]
pub struct CreateApiKeyRequest {
    /// API key name/description
    pub name: String,
    /// Permissions for this key (optional, defaults to Read)
    #[serde(default)]
    pub permissions: Vec<String>,
    /// Expiration time in seconds from now (optional, None = never expires)
    pub expires_in: Option<u64>,
}

/// Create API key response
#[derive(Debug, Serialize)]
pub struct CreateApiKeyResponse {
    /// The API key (only shown once!)
    pub api_key: String,
    /// API key ID
    pub id: String,
    /// API key name
    pub name: String,
    /// Permissions
    pub permissions: Vec<String>,
    /// Expiration timestamp (None = never)
    pub expires_at: Option<u64>,
    /// Warning message
    pub warning: String,
}

/// API key info (without the key itself)
#[derive(Debug, Serialize)]
pub struct ApiKeyInfo {
    /// API key ID
    pub id: String,
    /// API key name
    pub name: String,
    /// Permissions
    pub permissions: Vec<String>,
    /// Creation timestamp
    pub created_at: u64,
    /// Last used timestamp
    pub last_used: Option<u64>,
    /// Expiration timestamp
    pub expires_at: Option<u64>,
    /// Whether the key is active
    pub active: bool,
}

/// List API keys response
#[derive(Debug, Serialize)]
pub struct ListApiKeysResponse {
    /// API keys
    pub keys: Vec<ApiKeyInfo>,
}

/// Create user request (admin only)
#[derive(Debug, Deserialize)]
pub struct CreateUserRequest {
    /// Username
    pub username: String,
    /// Password
    pub password: String,
    /// User roles (optional, defaults to User)
    #[serde(default)]
    pub roles: Vec<String>,
}

/// Create user response
#[derive(Debug, Serialize)]
pub struct CreateUserResponse {
    /// User ID
    pub user_id: String,
    /// Username
    pub username: String,
    /// User roles
    pub roles: Vec<String>,
    /// Success message
    pub message: String,
}

/// List users response (admin only)
#[derive(Debug, Serialize)]
pub struct ListUsersResponse {
    /// List of users
    pub users: Vec<UserInfo>,
}

/// Change password request
#[derive(Debug, Deserialize)]
pub struct ChangePasswordRequest {
    /// Current password (required for non-admin)
    pub current_password: Option<String>,
    /// New password
    pub new_password: String,
}

/// Logout response
#[derive(Debug, Serialize)]
pub struct LogoutResponse {
    /// Status
    pub status: String,
    /// Message
    pub message: String,
}

/// Refresh token request
#[derive(Debug, Deserialize)]
pub struct RefreshTokenRequest {
    /// The current access token to refresh (optional, can use Authorization header)
    pub access_token: Option<String>,
}

/// Refresh token response
#[derive(Debug, Serialize)]
pub struct RefreshTokenResponse {
    /// New JWT access token
    pub access_token: String,
    /// Token type (always "Bearer")
    pub token_type: String,
    /// Token expiration in seconds
    pub expires_in: u64,
}

/// Password validation request
#[derive(Debug, Deserialize)]
pub struct ValidatePasswordRequest {
    /// Password to validate
    pub password: String,
}

/// Password validation response
#[derive(Debug, Serialize)]
pub struct ValidatePasswordResponse {
    /// Whether the password meets all requirements
    pub valid: bool,
    /// List of validation errors (empty if valid)
    pub errors: Vec<String>,
    /// Password strength score (0-100)
    pub strength: u8,
    /// Strength label (Very Weak, Weak, Fair, Strong, Very Strong)
    pub strength_label: String,
}

/// Error response
#[derive(Debug, Serialize)]
pub struct AuthErrorResponse {
    /// Error type
    pub error: String,
    /// Error message
    pub message: String,
}

/// Login attempt tracking for rate limiting
#[derive(Debug, Clone)]
pub struct LoginAttempt {
    /// Number of failed attempts
    pub count: u32,
    /// Timestamp of first failed attempt in this window
    pub window_start: std::time::Instant,
    /// Timestamp when lockout expires (if locked)
    pub locked_until: Option<std::time::Instant>,
}

/// Rate limit configuration
const MAX_LOGIN_ATTEMPTS: u32 = 5;
const LOGIN_WINDOW_SECONDS: u64 = 300; // 5 minutes
const LOCKOUT_SECONDS: u64 = 900; // 15 minutes lockout after max attempts

/// Shared state for auth handlers
#[derive(Clone)]
pub struct AuthHandlerState {
    /// Authentication manager
    pub auth_manager: Arc<AuthManager>,
    /// User store (in-memory cache, backed by disk persistence)
    pub users: Arc<tokio::sync::RwLock<HashMap<String, UserRecord>>>,
    /// Persistence manager for saving/loading from disk
    pub persistence: Arc<AuthPersistence>,
    /// Token blacklist for logout (tokens that have been invalidated)
    pub token_blacklist: Arc<tokio::sync::RwLock<HashSet<String>>>,
    /// Login attempt tracking for rate limiting (by IP or username)
    pub login_attempts: Arc<tokio::sync::RwLock<HashMap<String, LoginAttempt>>>,
}

/// User record for authentication
#[derive(Debug, Clone)]
pub struct UserRecord {
    /// User ID
    pub user_id: String,
    /// Username
    pub username: String,
    /// Password hash (bcrypt)
    pub password_hash: String,
    /// User roles
    pub roles: Vec<Role>,
}

impl AuthHandlerState {
    /// Create a new auth handler state
    pub fn new(auth_manager: Arc<AuthManager>) -> Self {
        let users = Arc::new(tokio::sync::RwLock::new(HashMap::new()));
        let persistence = Arc::new(AuthPersistence::with_default_dir());
        let token_blacklist = Arc::new(tokio::sync::RwLock::new(HashSet::new()));
        let login_attempts = Arc::new(tokio::sync::RwLock::new(HashMap::new()));
        Self {
            auth_manager,
            users,
            persistence,
            token_blacklist,
            login_attempts,
        }
    }

    /// Create with persistence - loads from disk or creates default admin
    pub async fn new_with_default_admin(auth_manager: Arc<AuthManager>) -> Self {
        Self::new_with_root_user(auth_manager, None, None).await
    }

    /// Create with persistence and optional root user configuration
    ///
    /// # Arguments
    /// * `auth_manager` - The authentication manager
    /// * `root_user` - Optional root username (defaults to "root")
    /// * `root_password` - Optional root password (generates random if not provided)
    pub async fn new_with_root_user(
        auth_manager: Arc<AuthManager>,
        root_user: Option<String>,
        root_password: Option<String>,
    ) -> Self {
        let persistence = Arc::new(AuthPersistence::with_default_dir());
        let users = Arc::new(tokio::sync::RwLock::new(HashMap::new()));

        // Try to load users from disk
        let mut loaded_users = HashMap::new();
        match persistence.load() {
            Ok(data) => {
                if !data.users.is_empty() {
                    info!("Loaded {} users from disk", data.users.len());
                    for (username, persisted) in data.users {
                        loaded_users.insert(
                            username,
                            UserRecord {
                                user_id: persisted.user_id,
                                username: persisted.username,
                                password_hash: persisted.password_hash,
                                roles: persisted.roles,
                            },
                        );
                    }
                }
            }
            Err(e) => {
                warn!("Failed to load auth data from disk: {}", e);
            }
        }

        // Check if any admin user exists
        let has_admin = loaded_users
            .values()
            .any(|u| u.roles.contains(&Role::Admin));

        // Create root admin if no admin users exist
        if !has_admin {
            let username = root_user.unwrap_or_else(|| "root".to_string());
            let (password, was_generated) = match root_password {
                Some(pwd) => (pwd, false),
                None => (Self::generate_secure_password(), true),
            };

            info!(
                "No admin users found, creating root admin user '{}'",
                username
            );

            let password_hash = bcrypt::hash(&password, bcrypt::DEFAULT_COST)
                .unwrap_or_else(|_| "invalid".to_string());

            let admin = UserRecord {
                user_id: username.clone(),
                username: username.clone(),
                password_hash: password_hash.clone(),
                roles: vec![Role::Admin],
            };

            loaded_users.insert(username.clone(), admin);

            // Save to disk
            let persisted_admin = PersistedUser {
                user_id: username.clone(),
                username: username.clone(),
                password_hash,
                roles: vec![Role::Admin],
                created_at: chrono::Utc::now().timestamp() as u64,
                last_login: None,
            };

            if let Err(e) = persistence.save_user(persisted_admin) {
                error!("Failed to save root admin to disk: {}", e);
            }

            // Print credentials to console (one-time only)
            println!();
            println!(
                "╔════════════════════════════════════════════════════════════════════════════╗"
            );
            println!(
                "║                     🔐 ROOT USER CREDENTIALS                               ║"
            );
            println!(
                "╠════════════════════════════════════════════════════════════════════════════╣"
            );
            println!("║  Username: {:<63} ║", username);
            println!("║  Password: {:<63} ║", password);
            println!(
                "╠════════════════════════════════════════════════════════════════════════════╣"
            );
            if was_generated {
                println!(
                    "║  ⚠️  SAVE THIS PASSWORD NOW - IT WILL NOT BE SHOWN AGAIN!                  ║"
                );
            } else {
                println!(
                    "║  ⚠️  Consider changing this password after first login!                    ║"
                );
            }
            println!(
                "╚════════════════════════════════════════════════════════════════════════════╝"
            );
            println!();

            warn!(
                "Root admin user '{}' created. {}",
                username,
                if was_generated {
                    "Password was auto-generated - see console output above."
                } else {
                    "Please change the password in production!"
                }
            );
        }

        *users.write().await = loaded_users;

        let token_blacklist = Arc::new(tokio::sync::RwLock::new(HashSet::new()));
        let login_attempts = Arc::new(tokio::sync::RwLock::new(HashMap::new()));

        Self {
            auth_manager,
            users,
            persistence,
            token_blacklist,
            login_attempts,
        }
    }

    /// Generate a secure random password
    fn generate_secure_password() -> String {
        use rand::Rng;
        const CHARSET: &[u8] = b"ABCDEFGHJKLMNPQRSTUVWXYZabcdefghjkmnpqrstuvwxyz23456789!@#$%^&*";
        let mut rng = rand::thread_rng();
        let password: String = (0..24)
            .map(|_| {
                let idx = rng.gen_range(0..CHARSET.len());
                CHARSET[idx] as char
            })
            .collect();
        password
    }

    /// Check if a token is blacklisted (logged out)
    pub async fn is_token_blacklisted(&self, token: &str) -> bool {
        self.token_blacklist.read().await.contains(token)
    }

    /// Add a token to the blacklist (logout)
    pub async fn blacklist_token(&self, token: String) {
        self.token_blacklist.write().await.insert(token);
    }

    /// Clean up expired tokens from blacklist (should be called periodically)
    pub async fn cleanup_expired_tokens(&self) {
        let mut blacklist = self.token_blacklist.write().await;
        let before_count = blacklist.len();

        // Remove tokens that have expired (can no longer be used anyway)
        blacklist.retain(|token| {
            // Try to decode without validation to check expiry
            match self.auth_manager.validate_jwt(token) {
                Ok(_) => true,   // Still valid, keep in blacklist
                Err(_) => false, // Expired or invalid, remove from blacklist
            }
        });

        let removed = before_count - blacklist.len();
        if removed > 0 {
            debug!("Cleaned up {} expired tokens from blacklist", removed);
        }
    }

    /// Check if login is rate limited for a given key (username or IP)
    pub async fn check_login_rate_limit(&self, key: &str) -> Result<(), (u32, u64)> {
        let attempts = self.login_attempts.read().await;

        if let Some(attempt) = attempts.get(key) {
            // Check if currently locked out
            if let Some(locked_until) = attempt.locked_until {
                if locked_until > std::time::Instant::now() {
                    let remaining = locked_until
                        .duration_since(std::time::Instant::now())
                        .as_secs();
                    return Err((attempt.count, remaining));
                }
            }

            // Check if within window and exceeded attempts
            let window_elapsed = attempt.window_start.elapsed().as_secs();
            if window_elapsed < LOGIN_WINDOW_SECONDS && attempt.count >= MAX_LOGIN_ATTEMPTS {
                let remaining = LOGIN_WINDOW_SECONDS - window_elapsed;
                return Err((attempt.count, remaining));
            }
        }

        Ok(())
    }

    /// Record a failed login attempt
    pub async fn record_failed_login(&self, key: &str) {
        let mut attempts = self.login_attempts.write().await;
        let now = std::time::Instant::now();

        let attempt = attempts.entry(key.to_string()).or_insert(LoginAttempt {
            count: 0,
            window_start: now,
            locked_until: None,
        });

        // Reset window if expired
        if attempt.window_start.elapsed().as_secs() >= LOGIN_WINDOW_SECONDS {
            attempt.count = 0;
            attempt.window_start = now;
            attempt.locked_until = None;
        }

        attempt.count += 1;

        // Lock out if exceeded max attempts
        if attempt.count >= MAX_LOGIN_ATTEMPTS {
            attempt.locked_until = Some(now + std::time::Duration::from_secs(LOCKOUT_SECONDS));
            warn!(
                "Account '{}' locked out for {} seconds after {} failed login attempts",
                key, LOCKOUT_SECONDS, attempt.count
            );
        }
    }

    /// Clear login attempts on successful login
    pub async fn clear_login_attempts(&self, key: &str) {
        let mut attempts = self.login_attempts.write().await;
        attempts.remove(key);
    }

    /// Clean up expired login attempt records
    pub async fn cleanup_expired_login_attempts(&self) {
        let mut attempts = self.login_attempts.write().await;
        let before_count = attempts.len();

        attempts.retain(|_, attempt| {
            // Keep if within window or still locked
            let window_active = attempt.window_start.elapsed().as_secs() < LOGIN_WINDOW_SECONDS;
            let still_locked = attempt
                .locked_until
                .is_some_and(|until| until > std::time::Instant::now());
            window_active || still_locked
        });

        let removed = before_count - attempts.len();
        if removed > 0 {
            debug!("Cleaned up {} expired login attempt records", removed);
        }
    }

    /// Save current users to disk
    pub async fn save_users_to_disk(&self) -> Result<(), String> {
        let users = self.users.read().await;
        let mut data = self.persistence.load().unwrap_or_default();

        data.users.clear();
        for (username, record) in users.iter() {
            data.users.insert(
                username.clone(),
                PersistedUser {
                    user_id: record.user_id.clone(),
                    username: record.username.clone(),
                    password_hash: record.password_hash.clone(),
                    roles: record.roles.clone(),
                    created_at: chrono::Utc::now().timestamp() as u64,
                    last_login: None,
                },
            );
        }

        self.persistence.save(&data)
    }
}

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
    let valid = bcrypt::verify(&request.password, &user.password_hash).unwrap_or(false);
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

    info!(
        "API key '{}' revoked by user '{}'",
        key_id, auth_state.user_claims.user_id
    );

    Ok(Json(serde_json::json!({
        "status": "ok",
        "message": "API key revoked successfully"
    })))
}

// ============================================================================
// User Management Endpoints (Admin only)
// ============================================================================

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

    // Hash password
    let password_hash = bcrypt::hash(&request.password, bcrypt::DEFAULT_COST).map_err(|e| {
        error!("Failed to hash password: {}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(AuthErrorResponse {
                error: "hash_failed".to_string(),
                message: "Failed to create user".to_string(),
            }),
        )
    })?;

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

        let valid = bcrypt::verify(current_password, &user.password_hash).unwrap_or(false);
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

    // Hash new password
    let new_hash = bcrypt::hash(&request.new_password, bcrypt::DEFAULT_COST).map_err(|e| {
        error!("Failed to hash password: {}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(AuthErrorResponse {
                error: "hash_failed".to_string(),
                message: "Failed to update password".to_string(),
            }),
        )
    })?;

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
        error!("Failed to persist password change: {}", e);
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

/// Authentication middleware that extracts auth state from request
/// and adds it to request extensions
pub async fn auth_middleware(
    State(state): State<AuthHandlerState>,
    mut request: axum::extract::Request,
    next: axum::middleware::Next,
) -> axum::response::Response {
    use axum::http::header::AUTHORIZATION;

    let auth_state = extract_auth_from_request(&state, &request).await;

    // Add auth state to request extensions
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

    let auth_state = extract_auth_from_request(&state, &request).await;

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

    // Add auth state to request extensions
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

    let auth_state = extract_auth_from_request(&state, &request).await;

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

    // Check for admin role
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

    // Add auth state to request extensions
    request.extensions_mut().insert(auth_state);

    next.run(request).await
}

/// Extract authentication state from request headers
async fn extract_auth_from_request(
    state: &AuthHandlerState,
    request: &axum::extract::Request,
) -> AuthState {
    use axum::http::header::AUTHORIZATION;

    // Try to get authorization header
    if let Some(auth_header) = request.headers().get(AUTHORIZATION) {
        if let Ok(auth_str) = auth_header.to_str() {
            // Check for Bearer token (JWT)
            if let Some(token) = auth_str.strip_prefix("Bearer ") {
                // Check if token is blacklisted (logged out)
                if state.is_token_blacklisted(token).await {
                    debug!("Token is blacklisted, rejecting authentication");
                    return AuthState {
                        user_claims: UserClaims {
                            user_id: "anonymous".to_string(),
                            username: "anonymous".to_string(),
                            roles: vec![],
                            iat: 0,
                            exp: 0,
                        },
                        authenticated: false,
                    };
                }

                if let Ok(claims) = state.auth_manager.validate_jwt(token) {
                    return AuthState {
                        user_claims: claims,
                        authenticated: true,
                    };
                }
            }

            // Check for X-API-Key style (direct API key)
            if let Ok(claims) = state.auth_manager.validate_api_key(auth_str).await {
                return AuthState {
                    user_claims: claims,
                    authenticated: true,
                };
            }
        }
    }

    // Check for X-API-Key header
    if let Some(api_key_header) = request.headers().get("X-API-Key") {
        if let Ok(api_key) = api_key_header.to_str() {
            if let Ok(claims) = state.auth_manager.validate_api_key(api_key).await {
                return AuthState {
                    user_claims: claims,
                    authenticated: true,
                };
            }
        }
    }

    // Check for API key in query parameters
    if let Some(query) = request.uri().query() {
        for param in query.split('&') {
            if let Some(api_key) = param.strip_prefix("api_key=") {
                if let Ok(claims) = state.auth_manager.validate_api_key(api_key).await {
                    return AuthState {
                        user_claims: claims,
                        authenticated: true,
                    };
                }
            }
        }
    }

    // No authentication found - return anonymous state
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_login_request_deserialization() {
        let json = r#"{"username": "test", "password": "pass123"}"#;
        let request: LoginRequest = serde_json::from_str(json).unwrap();
        assert_eq!(request.username, "test");
        assert_eq!(request.password, "pass123");
    }

    #[test]
    fn test_create_api_key_request_deserialization() {
        let json = r#"{"name": "my-key", "permissions": ["read", "write"]}"#;
        let request: CreateApiKeyRequest = serde_json::from_str(json).unwrap();
        assert_eq!(request.name, "my-key");
        assert_eq!(request.permissions.len(), 2);
    }

    #[test]
    fn test_login_response_serialization() {
        let response = LoginResponse {
            access_token: "token123".to_string(),
            token_type: "Bearer".to_string(),
            expires_in: 3600,
            user: UserInfo {
                user_id: "user1".to_string(),
                username: "testuser".to_string(),
                roles: vec!["User".to_string()],
            },
        };
        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("token123"));
        assert!(json.contains("Bearer"));
    }
}
