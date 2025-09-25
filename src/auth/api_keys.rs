//! API Key management system
//!
//! Provides secure API key generation, storage, and validation

use crate::auth::ApiKey;
use crate::error::{Result, VectorizerError};
// serde traits are used in the ApiKey struct
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

/// API Key manager for secure key operations
#[derive(Debug)]
pub struct ApiKeyManager {
    /// Storage for API keys (in production, this would be a database)
    keys: Arc<RwLock<HashMap<String, ApiKey>>>,
    /// API key length
    key_length: usize,
}

impl ApiKeyManager {
    /// Create a new API key manager
    pub fn new(key_length: usize) -> Result<Self> {
        if key_length < 16 {
            return Err(VectorizerError::InvalidConfiguration {
                message: "API key length must be at least 16 characters".to_string(),
            });
        }

        Ok(Self {
            keys: Arc::new(RwLock::new(HashMap::new())),
            key_length,
        })
    }

    /// Generate a new API key
    fn generate_key(&self) -> String {
        use rand::Rng;
        const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789";

        let mut rng = rand::thread_rng();
        (0..self.key_length)
            .map(|_| {
                let idx = rng.gen_range(0..CHARSET.len());
                CHARSET[idx] as char
            })
            .collect()
    }

    /// Hash an API key for secure storage
    fn hash_key(&self, key: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(key.as_bytes());
        format!("{:x}", hasher.finalize())
    }

    /// Create a new API key for a user
    pub async fn create_key(
        &self,
        user_id: &str,
        name: &str,
        permissions: Vec<crate::auth::roles::Permission>,
        expires_at: Option<u64>,
    ) -> Result<(String, ApiKey)> {
        let key_id = Uuid::new_v4().to_string();
        let api_key = self.generate_key();
        let key_hash = self.hash_key(&api_key);

        let now = chrono::Utc::now().timestamp() as u64;

        let key_info = ApiKey {
            id: key_id.clone(),
            name: name.to_string(),
            key_hash,
            user_id: user_id.to_string(),
            permissions,
            created_at: now,
            last_used: None,
            expires_at,
            active: true,
        };

        let mut keys = self.keys.write().await;
        keys.insert(key_id, key_info.clone());

        Ok((api_key, key_info))
    }

    /// Validate an API key and return key information
    pub async fn validate_key(&self, api_key: &str) -> Result<ApiKey> {
        let key_hash = self.hash_key(api_key);
        let keys = self.keys.read().await;

        // Find the key by hash
        for (_, key_info) in keys.iter() {
            if key_info.key_hash == key_hash && key_info.active {
                // Check expiration
                if let Some(expires_at) = key_info.expires_at {
                    let now = chrono::Utc::now().timestamp() as u64;
                    if now > expires_at {
                        return Err(VectorizerError::AuthenticationError(
                            "API key has expired".to_string(),
                        ));
                    }
                }

                return Ok(key_info.clone());
            }
        }

        Err(VectorizerError::AuthenticationError(
            "Invalid API key".to_string(),
        ))
    }

    /// Update the last used timestamp for an API key
    pub async fn update_last_used(&self, key_id: &str) -> Result<()> {
        let mut keys = self.keys.write().await;

        if let Some(key_info) = keys.get_mut(key_id) {
            key_info.last_used = Some(chrono::Utc::now().timestamp() as u64);
        }

        Ok(())
    }

    /// List all API keys for a user
    pub async fn list_keys_for_user(&self, user_id: &str) -> Result<Vec<ApiKey>> {
        let keys = self.keys.read().await;

        let user_keys: Vec<ApiKey> = keys
            .values()
            .filter(|key| key.user_id == user_id)
            .cloned()
            .collect();

        Ok(user_keys)
    }

    /// Revoke an API key
    pub async fn revoke_key(&self, key_id: &str) -> Result<()> {
        let mut keys = self.keys.write().await;

        if let Some(key_info) = keys.get_mut(key_id) {
            key_info.active = false;
        } else {
            return Err(VectorizerError::NotFound(format!(
                "API key {} not found",
                key_id
            )));
        }

        Ok(())
    }

    /// Get API key information by ID
    pub async fn get_key_info(&self, key_id: &str) -> Result<ApiKey> {
        let keys = self.keys.read().await;

        keys.get(key_id)
            .cloned()
            .ok_or_else(|| VectorizerError::NotFound(format!("API key {} not found", key_id)))
    }

    /// List all API keys (admin function)
    pub async fn list_all_keys(&self) -> Result<Vec<ApiKey>> {
        let keys = self.keys.read().await;
        Ok(keys.values().cloned().collect())
    }

    /// Clean up expired keys
    pub async fn cleanup_expired_keys(&self) -> Result<usize> {
        let mut keys = self.keys.write().await;
        let now = chrono::Utc::now().timestamp() as u64;

        let expired_keys: Vec<String> = keys
            .iter()
            .filter(|(_, key)| key.expires_at.is_some_and(|expires_at| now > expires_at))
            .map(|(id, _)| id.clone())
            .collect();

        for key_id in &expired_keys {
            keys.remove(key_id);
        }

        Ok(expired_keys.len())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::auth::roles::Permission;

    #[tokio::test]
    async fn test_api_key_manager_creation() {
        let manager = ApiKeyManager::new(32).unwrap();
        assert_eq!(manager.key_length, 32);
    }

    #[tokio::test]
    async fn test_api_key_manager_short_length() {
        let result = ApiKeyManager::new(8);
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            VectorizerError::InvalidConfiguration { .. }
        ));
    }

    #[tokio::test]
    async fn test_api_key_creation() {
        let manager = ApiKeyManager::new(32).unwrap();

        let (api_key, key_info) = manager
            .create_key("user123", "test_key", vec![Permission::Read], None)
            .await
            .unwrap();

        assert_eq!(key_info.user_id, "user123");
        assert_eq!(key_info.name, "test_key");
        assert_eq!(key_info.permissions, vec![Permission::Read]);
        assert!(key_info.active);
        assert_eq!(api_key.len(), 32);
    }

    #[tokio::test]
    async fn test_api_key_validation() {
        let manager = ApiKeyManager::new(32).unwrap();

        let (api_key, key_info) = manager
            .create_key("user123", "test_key", vec![Permission::Read], None)
            .await
            .unwrap();

        let validated_key = manager.validate_key(&api_key).await.unwrap();
        assert_eq!(validated_key.id, key_info.id);
        assert_eq!(validated_key.user_id, "user123");
    }

    #[tokio::test]
    async fn test_invalid_api_key() {
        let manager = ApiKeyManager::new(32).unwrap();

        let result = manager.validate_key("invalid_key").await;
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            VectorizerError::AuthenticationError { .. }
        ));
    }

    #[tokio::test]
    async fn test_api_key_expiration() {
        let manager = ApiKeyManager::new(32).unwrap();

        // Create key that expires in 1 second
        let expires_at = chrono::Utc::now().timestamp() as u64 + 1;
        let (api_key, _) = manager
            .create_key(
                "user123",
                "test_key",
                vec![Permission::Read],
                Some(expires_at),
            )
            .await
            .unwrap();

        // Should be valid initially
        manager.validate_key(&api_key).await.unwrap();

        // Wait for expiration
        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

        // Should be invalid now
        let result = manager.validate_key(&api_key).await;
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            VectorizerError::AuthenticationError { .. }
        ));
    }

    #[tokio::test]
    async fn test_api_key_revocation() {
        let manager = ApiKeyManager::new(32).unwrap();

        let (api_key, key_info) = manager
            .create_key("user123", "test_key", vec![Permission::Read], None)
            .await
            .unwrap();

        // Should be valid initially
        manager.validate_key(&api_key).await.unwrap();

        // Revoke the key
        manager.revoke_key(&key_info.id).await.unwrap();

        // Should be invalid now
        let result = manager.validate_key(&api_key).await;
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            VectorizerError::AuthenticationError { .. }
        ));
    }

    #[tokio::test]
    async fn test_list_keys_for_user() {
        let manager = ApiKeyManager::new(32).unwrap();

        // Create multiple keys for user123
        manager
            .create_key("user123", "key1", vec![Permission::Read], None)
            .await
            .unwrap();
        manager
            .create_key("user123", "key2", vec![Permission::Write], None)
            .await
            .unwrap();

        // Create key for different user
        manager
            .create_key("user456", "key3", vec![Permission::Read], None)
            .await
            .unwrap();

        let user_keys = manager.list_keys_for_user("user123").await.unwrap();
        assert_eq!(user_keys.len(), 2);

        let user_keys = manager.list_keys_for_user("user456").await.unwrap();
        assert_eq!(user_keys.len(), 1);
    }

    #[tokio::test]
    async fn test_cleanup_expired_keys() {
        let manager = ApiKeyManager::new(32).unwrap();

        // Create expired key
        let expired_time = chrono::Utc::now().timestamp() as u64 - 1;
        manager
            .create_key(
                "user123",
                "expired_key",
                vec![Permission::Read],
                Some(expired_time),
            )
            .await
            .unwrap();

        // Create non-expired key
        manager
            .create_key("user123", "valid_key", vec![Permission::Read], None)
            .await
            .unwrap();

        let cleaned_count = manager.cleanup_expired_keys().await.unwrap();
        assert_eq!(cleaned_count, 1);

        let all_keys = manager.list_all_keys().await.unwrap();
        assert_eq!(all_keys.len(), 1);
        assert_eq!(all_keys[0].name, "valid_key");
    }
}
