//! Request / response / error DTOs for the `/auth/*` REST surface.
//!
//! All types derive `Serialize` / `Deserialize` and carry the field
//! documentation consumed by the OpenAPI generator. Nothing in this
//! module depends on handler state, persistence, or cryptographic
//! primitives — keeping the type surface standalone makes it easy to
//! reuse these structs from SDK tests or ad-hoc tooling.

use serde::{Deserialize, Serialize};

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
