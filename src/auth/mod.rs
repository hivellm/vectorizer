//! Authentication and authorization system for Vectorizer
//!
//! Provides JWT-based authentication, API key management, and role-based access control
//! for production deployment of the vector database.

pub mod api_keys;
pub mod jwt;
pub mod middleware;
pub mod password;
pub mod persistence;
pub mod roles;

use std::collections::HashMap;
use std::sync::Arc;

pub use api_keys::ApiKeyManager;
pub use jwt::JwtManager;
pub use middleware::AuthMiddleware;
pub use password::{
    PasswordRequirements, PasswordValidationResult, validate_password,
    validate_password_with_requirements,
};
pub use roles::{Permission, Role};
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;

use crate::error::{Result, VectorizerError};

/// The insecure literal that shipped as the hardcoded default in every
/// Vectorizer release up to 2.x. Rejected at boot so a misconfigured operator
/// can never silently run with a known-weak secret.
pub const LEGACY_INSECURE_DEFAULT_SECRET: &str =
    "vectorizer-default-secret-key-change-in-production";

/// Minimum accepted length for a JWT secret. Matches the length check in
/// `JwtManager::new` and `ConfigManager::validate_config`.
pub const MIN_JWT_SECRET_LEN: usize = 32;

/// Authentication configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthConfig {
    /// JWT secret key for token signing. Must be set explicitly (via config
    /// file or `VECTORIZER_JWT_SECRET` env var) to a value that is at least
    /// `MIN_JWT_SECRET_LEN` characters long and is NOT the legacy default.
    pub jwt_secret: String,
    /// JWT token expiration time in seconds (default: 3600 = 1 hour)
    pub jwt_expiration: u64,
    /// API key length (default: 32)
    pub api_key_length: usize,
    /// Rate limiting: requests per minute per API key
    pub rate_limit_per_minute: u32,
    /// Rate limiting: requests per hour per API key
    pub rate_limit_per_hour: u32,
    /// Enable authentication (default: true)
    pub enabled: bool,
}

impl Default for AuthConfig {
    /// Returns a default config with an **empty** JWT secret. Callers must
    /// populate `jwt_secret` before instantiating an `AuthManager`, or boot
    /// will fail via `AuthConfig::validate` / `AuthManager::new`. This is
    /// intentional: shipping a real default value historically caused a known
    /// auth-bypass vulnerability.
    fn default() -> Self {
        Self {
            jwt_secret: String::new(),
            jwt_expiration: 3600, // 1 hour
            api_key_length: 32,
            rate_limit_per_minute: 100,
            rate_limit_per_hour: 1000,
            enabled: true,
        }
    }
}

impl AuthConfig {
    /// Fail-fast validation called during `AuthManager::new`. Rejects three
    /// classes of misconfiguration:
    ///
    /// 1. empty `jwt_secret` (typical when the operator forgot to set one),
    /// 2. `jwt_secret` equal to `LEGACY_INSECURE_DEFAULT_SECRET` (the old
    ///    hardcoded literal),
    /// 3. `jwt_secret` shorter than `MIN_JWT_SECRET_LEN`.
    ///
    /// Validation only runs when `enabled == true`; running auth-disabled
    /// (dev/testing) has no secret requirement.
    pub fn validate(&self) -> Result<()> {
        if !self.enabled {
            return Ok(());
        }
        if self.jwt_secret.is_empty() {
            return Err(VectorizerError::InvalidConfiguration {
                message: "auth.jwt_secret is empty. Set it in config.yml or via \
                          VECTORIZER_JWT_SECRET env var. Generate one with: \
                          openssl rand -hex 64"
                    .to_string(),
            });
        }
        if self.jwt_secret == LEGACY_INSECURE_DEFAULT_SECRET {
            return Err(VectorizerError::InvalidConfiguration {
                message: "auth.jwt_secret is the legacy insecure default. This \
                          value is publicly known and must never be used. \
                          Generate a new secret with: openssl rand -hex 64"
                    .to_string(),
            });
        }
        if self.jwt_secret.len() < MIN_JWT_SECRET_LEN {
            return Err(VectorizerError::InvalidConfiguration {
                message: format!(
                    "auth.jwt_secret is {} chars; must be at least {} chars. \
                     Generate one with: openssl rand -hex 64",
                    self.jwt_secret.len(),
                    MIN_JWT_SECRET_LEN
                ),
            });
        }
        Ok(())
    }
}

/// User information stored in JWT token
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserClaims {
    /// User ID
    pub user_id: String,
    /// Username
    pub username: String,
    /// User roles
    pub roles: Vec<Role>,
    /// Token issued at (Unix timestamp)
    pub iat: u64,
    /// Token expiration (Unix timestamp)
    pub exp: u64,
}

/// API Key information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiKey {
    /// API key ID
    pub id: String,
    /// API key name/description
    pub name: String,
    /// API key value (hashed)
    pub key_hash: String,
    /// Associated user ID
    pub user_id: String,
    /// Permissions for this API key
    pub permissions: Vec<Permission>,
    /// Creation timestamp
    pub created_at: u64,
    /// Last used timestamp
    pub last_used: Option<u64>,
    /// Expiration timestamp (None = never expires)
    pub expires_at: Option<u64>,
    /// Whether the key is active
    pub active: bool,
}

/// Rate limiting information
#[derive(Debug, Clone)]
pub struct RateLimitInfo {
    /// Requests in current minute
    pub requests_per_minute: u32,
    /// Requests in current hour
    pub requests_per_hour: u32,
    /// Last reset time for minute counter
    pub minute_reset: u64,
    /// Last reset time for hour counter
    pub hour_reset: u64,
}

/// Authentication manager that handles both JWT and API key authentication
#[derive(Debug)]
pub struct AuthManager {
    /// JWT manager for token operations
    jwt_manager: JwtManager,
    /// API key manager
    api_key_manager: ApiKeyManager,
    /// Rate limiting storage
    rate_limits: Arc<RwLock<HashMap<String, RateLimitInfo>>>,
    /// Configuration
    config: AuthConfig,
}

impl AuthManager {
    /// Create a new authentication manager.
    ///
    /// Runs `AuthConfig::validate` before constructing any sub-manager so the
    /// process refuses to boot with an empty/default/short JWT secret.
    pub fn new(config: AuthConfig) -> Result<Self> {
        config.validate()?;

        let jwt_manager = JwtManager::new(&config.jwt_secret, config.jwt_expiration)?;
        let api_key_manager = ApiKeyManager::new(config.api_key_length)?;

        Ok(Self {
            jwt_manager,
            api_key_manager,
            rate_limits: Arc::new(RwLock::new(HashMap::new())),
            config,
        })
    }

    /// Create a new authentication manager with default configuration.
    ///
    /// Deprecated: `AuthConfig::default()` now yields an empty JWT secret and
    /// fails `validate()`. Use `AuthManager::new(config)` with a populated
    /// config instead. Kept only for test call sites that explicitly inject a
    /// valid secret before invoking.
    #[deprecated(
        note = "Default AuthConfig has an empty jwt_secret; construct a full AuthConfig \
                         and call AuthManager::new directly."
    )]
    pub fn new_default() -> Result<Self> {
        Self::new(AuthConfig::default())
    }

    /// Generate a new API key for a user
    pub async fn create_api_key(
        &self,
        user_id: &str,
        name: &str,
        permissions: Vec<Permission>,
        expires_at: Option<u64>,
    ) -> Result<(String, ApiKey)> {
        self.api_key_manager
            .create_key(user_id, name, permissions, expires_at)
            .await
    }

    /// Validate an API key and return user information
    pub async fn validate_api_key(&self, api_key: &str) -> Result<UserClaims> {
        let key_info = self.api_key_manager.validate_key(api_key).await?;

        // Check rate limiting
        self.check_rate_limit(&key_info.id).await?;

        // Update last used timestamp
        self.api_key_manager.update_last_used(&key_info.id).await?;

        // Create user claims from API key
        Ok(UserClaims {
            user_id: key_info.user_id.clone(),
            username: format!("api_key_{}", &key_info.id[..8]),
            roles: vec![Role::ApiUser],
            iat: chrono::Utc::now().timestamp() as u64,
            exp: key_info.expires_at.unwrap_or(u64::MAX),
        })
    }

    /// Generate a JWT token for a user
    pub fn generate_jwt(&self, user_id: &str, username: &str, roles: Vec<Role>) -> Result<String> {
        self.jwt_manager.generate_token(user_id, username, roles)
    }

    /// Validate a JWT token
    pub fn validate_jwt(&self, token: &str) -> Result<UserClaims> {
        self.jwt_manager.validate_token(token)
    }

    /// Check rate limiting for an API key
    async fn check_rate_limit(&self, api_key_id: &str) -> Result<()> {
        let mut rate_limits = self.rate_limits.write().await;
        let now = chrono::Utc::now().timestamp() as u64;

        let rate_info = rate_limits
            .entry(api_key_id.to_string())
            .or_insert_with(|| RateLimitInfo {
                requests_per_minute: 0,
                requests_per_hour: 0,
                minute_reset: now,
                hour_reset: now,
            });

        // Reset counters if needed
        if now - rate_info.minute_reset >= 60 {
            rate_info.requests_per_minute = 0;
            rate_info.minute_reset = now;
        }

        if now - rate_info.hour_reset >= 3600 {
            rate_info.requests_per_hour = 0;
            rate_info.hour_reset = now;
        }

        // Check limits
        if rate_info.requests_per_minute >= self.config.rate_limit_per_minute {
            return Err(VectorizerError::RateLimitExceeded {
                limit_type: "per_minute".to_string(),
                limit: self.config.rate_limit_per_minute,
            });
        }

        if rate_info.requests_per_hour >= self.config.rate_limit_per_hour {
            return Err(VectorizerError::RateLimitExceeded {
                limit_type: "per_hour".to_string(),
                limit: self.config.rate_limit_per_hour,
            });
        }

        // Increment counters
        rate_info.requests_per_minute += 1;
        rate_info.requests_per_hour += 1;

        Ok(())
    }

    /// List all API keys for a user
    pub async fn list_api_keys(&self, user_id: &str) -> Result<Vec<ApiKey>> {
        self.api_key_manager.list_keys_for_user(user_id).await
    }

    /// Revoke an API key
    pub async fn revoke_api_key(&self, api_key_id: &str) -> Result<()> {
        self.api_key_manager.revoke_key(api_key_id).await
    }

    /// Get authentication configuration
    pub fn config(&self) -> &AuthConfig {
        &self.config
    }

    /// Register an existing API key (for loading from persistence)
    pub async fn register_api_key(&self, key_info: ApiKey) -> Result<()> {
        self.api_key_manager.register_key(key_info).await
    }

    /// Delete an API key completely
    pub async fn delete_api_key(&self, key_id: &str) -> Result<()> {
        self.api_key_manager.delete_key(key_id).await
    }

    /// Get API key info by ID
    pub async fn get_api_key_info(&self, key_id: &str) -> Result<ApiKey> {
        self.api_key_manager.get_key_info(key_id).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Build a valid `AuthConfig` for unit tests: injects a deterministic
    /// 64-char secret that satisfies `validate()` without leaking into the
    /// release build. Do NOT use this value in any production config.
    fn test_config() -> AuthConfig {
        AuthConfig {
            jwt_secret: "t".repeat(64),
            ..AuthConfig::default()
        }
    }

    #[tokio::test]
    async fn test_auth_manager_creation() {
        let auth_manager = AuthManager::new(test_config()).unwrap();

        assert!(auth_manager.config().enabled);
        assert_eq!(auth_manager.config().jwt_expiration, 3600);
    }

    #[tokio::test]
    async fn test_api_key_creation_and_validation() {
        let auth_manager = AuthManager::new(test_config()).unwrap();

        let (api_key, key_info) = auth_manager
            .create_api_key("user123", "test_key", vec![Permission::Read], None)
            .await
            .unwrap();

        assert_eq!(key_info.user_id, "user123");
        assert_eq!(key_info.name, "test_key");
        assert!(key_info.active);

        let user_claims = auth_manager.validate_api_key(&api_key).await.unwrap();
        assert_eq!(user_claims.user_id, "user123");
    }

    #[tokio::test]
    async fn test_jwt_generation_and_validation() {
        let auth_manager = AuthManager::new(test_config()).unwrap();

        let token = auth_manager
            .generate_jwt("user123", "testuser", vec![Role::Admin])
            .unwrap();

        let claims = auth_manager.validate_jwt(&token).unwrap();
        assert_eq!(claims.user_id, "user123");
        assert_eq!(claims.username, "testuser");
        assert!(claims.roles.contains(&Role::Admin));
    }

    #[tokio::test]
    async fn test_rate_limiting() {
        let mut config = test_config();
        config.rate_limit_per_minute = 2; // Very low limit for testing

        let auth_manager = AuthManager::new(config).unwrap();

        let (api_key, _) = auth_manager
            .create_api_key("user123", "test_key", vec![Permission::Read], None)
            .await
            .unwrap();

        // First two requests should succeed
        auth_manager.validate_api_key(&api_key).await.unwrap();
        auth_manager.validate_api_key(&api_key).await.unwrap();

        // Third request should fail
        let result = auth_manager.validate_api_key(&api_key).await;
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            VectorizerError::RateLimitExceeded { .. }
        ));
    }

    #[test]
    fn validate_rejects_empty_secret() {
        let config = AuthConfig::default(); // empty secret
        let err = config.validate().unwrap_err();
        assert!(
            matches!(err, VectorizerError::InvalidConfiguration { ref message } if message.contains("empty")),
            "expected InvalidConfiguration about empty secret, got {err:?}"
        );
    }

    #[test]
    fn validate_rejects_legacy_default_secret() {
        let config = AuthConfig {
            jwt_secret: LEGACY_INSECURE_DEFAULT_SECRET.to_string(),
            ..AuthConfig::default()
        };
        let err = config.validate().unwrap_err();
        assert!(
            matches!(err, VectorizerError::InvalidConfiguration { ref message } if message.contains("legacy")),
            "expected InvalidConfiguration about legacy default, got {err:?}"
        );
    }

    #[test]
    fn validate_rejects_short_secret() {
        let config = AuthConfig {
            jwt_secret: "short-but-nonempty".to_string(),
            ..AuthConfig::default()
        };
        let err = config.validate().unwrap_err();
        assert!(
            matches!(err, VectorizerError::InvalidConfiguration { ref message } if message.contains("chars")),
            "expected InvalidConfiguration about length, got {err:?}"
        );
    }

    #[test]
    fn validate_accepts_valid_secret() {
        let config = AuthConfig {
            jwt_secret: "a".repeat(64),
            ..AuthConfig::default()
        };
        config.validate().unwrap();
    }

    #[test]
    fn validate_skipped_when_disabled() {
        let config = AuthConfig {
            enabled: false,
            ..AuthConfig::default() // empty secret, but enabled=false
        };
        config.validate().unwrap();
    }

    #[test]
    fn manager_new_refuses_legacy_default() {
        let config = AuthConfig {
            jwt_secret: LEGACY_INSECURE_DEFAULT_SECRET.to_string(),
            ..AuthConfig::default()
        };
        assert!(AuthManager::new(config).is_err());
    }
}
