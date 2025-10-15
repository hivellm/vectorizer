//! JWT (JSON Web Token) implementation for authentication
//!
//! Provides secure token generation and validation using HMAC-SHA256

use crate::error::{Result, VectorizerError};
use jsonwebtoken::{Algorithm, DecodingKey, EncodingKey, Header, Validation, decode, encode};
// serde traits are used in the UserClaims struct
use std::time::{SystemTime, UNIX_EPOCH};

/// JWT manager for token operations
#[derive(Debug)]
pub struct JwtManager {
    /// Secret key for signing tokens
    secret: String,
    /// Token expiration time in seconds
    expiration: u64,
}

impl JwtManager {
    /// Create a new JWT manager
    pub fn new(secret: &str, expiration: u64) -> Result<Self> {
        if secret.len() < 32 {
            return Err(VectorizerError::InvalidConfiguration {
                message: "JWT secret must be at least 32 characters long".to_string(),
            });
        }

        Ok(Self {
            secret: secret.to_string(),
            expiration,
        })
    }

    /// Generate a JWT token for a user
    pub fn generate_token(
        &self,
        user_id: &str,
        username: &str,
        roles: Vec<crate::auth::roles::Role>,
    ) -> Result<String> {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|_| VectorizerError::InternalError("Failed to get current time".to_string()))?
            .as_secs();

        let claims = crate::auth::UserClaims {
            user_id: user_id.to_string(),
            username: username.to_string(),
            roles,
            iat: now,
            exp: now + self.expiration,
        };

        let header = Header::new(Algorithm::HS256);
        let encoding_key = EncodingKey::from_secret(self.secret.as_bytes());

        encode(&header, &claims, &encoding_key)
            .map_err(|e| VectorizerError::InternalError(format!("Failed to encode JWT: {}", e)))
    }

    /// Validate a JWT token and return claims
    pub fn validate_token(&self, token: &str) -> Result<crate::auth::UserClaims> {
        let decoding_key = DecodingKey::from_secret(self.secret.as_bytes());
        let validation = Validation::new(Algorithm::HS256);

        let token_data = decode::<crate::auth::UserClaims>(token, &decoding_key, &validation)
            .map_err(|e| match e.kind() {
                jsonwebtoken::errors::ErrorKind::ExpiredSignature => {
                    VectorizerError::AuthenticationError("Token has expired".to_string())
                }
                jsonwebtoken::errors::ErrorKind::InvalidToken => {
                    VectorizerError::AuthenticationError("Invalid token".to_string())
                }
                _ => {
                    VectorizerError::AuthenticationError(format!("Token validation failed: {}", e))
                }
            })?;

        Ok(token_data.claims)
    }

    /// Refresh a token (generate new token with same claims but new expiration)
    pub fn refresh_token(&self, old_token: &str) -> Result<String> {
        let claims = self.validate_token(old_token)?;

        // Generate new token with same user info but new expiration
        self.generate_token(&claims.user_id, &claims.username, claims.roles)
    }

    /// Get token expiration time
    pub fn expiration(&self) -> u64 {
        self.expiration
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::auth::roles::Role;

    #[test]
    fn test_jwt_manager_creation() {
        let secret = "this-is-a-very-long-secret-key-for-testing-purposes-only";
        let manager = JwtManager::new(secret, 3600).unwrap();

        assert_eq!(manager.expiration(), 3600);
    }

    #[test]
    fn test_jwt_manager_short_secret() {
        let secret = "short";
        let result = JwtManager::new(secret, 3600);

        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            VectorizerError::InvalidConfiguration { .. }
        ));
    }

    #[test]
    fn test_token_generation_and_validation() {
        let secret = "this-is-a-very-long-secret-key-for-testing-purposes-only";
        let manager = JwtManager::new(secret, 3600).unwrap();

        let token = manager
            .generate_token("user123", "testuser", vec![Role::Admin, Role::User])
            .unwrap();

        let claims = manager.validate_token(&token).unwrap();
        assert_eq!(claims.user_id, "user123");
        assert_eq!(claims.username, "testuser");
        assert!(claims.roles.contains(&Role::Admin));
        assert!(claims.roles.contains(&Role::User));
    }

    #[test]
    fn test_invalid_token() {
        let secret = "this-is-a-very-long-secret-key-for-testing-purposes-only";
        let manager = JwtManager::new(secret, 3600).unwrap();

        let result = manager.validate_token("invalid.token.here");
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            VectorizerError::AuthenticationError { .. }
        ));
    }

    #[test]
    fn test_token_refresh() {
        let secret = "this-is-a-very-long-secret-key-for-testing-purposes-only";
        let manager = JwtManager::new(secret, 1).unwrap(); // Very short expiration

        let original_token = manager
            .generate_token("user123", "testuser", vec![Role::User])
            .unwrap();

        let refreshed_token = manager.refresh_token(&original_token).unwrap();

        // Both tokens should be valid
        let original_claims = manager.validate_token(&original_token).unwrap();
        let refreshed_claims = manager.validate_token(&refreshed_token).unwrap();

        assert_eq!(original_claims.user_id, refreshed_claims.user_id);
        assert_eq!(original_claims.username, refreshed_claims.username);
        assert_eq!(original_claims.roles, refreshed_claims.roles);
    }
}
