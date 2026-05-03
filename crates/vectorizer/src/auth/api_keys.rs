//! API Key management system
//!
//! Provides secure API key generation, storage, and validation

use std::collections::HashMap;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

use dashmap::DashMap;
// serde traits are used in the ApiKey struct
use sha2::{Digest, Sha256};
use tokio::sync::RwLock;
use uuid::Uuid;

use crate::auth::ApiKey;
use crate::error::{Result, VectorizerError};

/// API Key manager for secure key operations
#[derive(Debug)]
pub struct ApiKeyManager {
    /// Storage for API keys (in production, this would be a database)
    keys: Arc<RwLock<HashMap<String, ApiKey>>>,
    /// Per-key atomic usage counters kept OUTSIDE the `RwLock<HashMap>`
    /// so the hot validation path can bump a key's counter without
    /// acquiring a write lock. The map itself is sharded (DashMap), so
    /// concurrent bumps on different keys never contend. On
    /// `register_key` (load from disk) the atomic is seeded from the
    /// persisted `usage_count`; on `snapshot_usage_counts` (called
    /// before persistence flush) the atomics are stamped back into the
    /// in-memory `ApiKey` records so the next save round-trips the
    /// up-to-date totals.
    usage_counters: Arc<DashMap<String, Arc<AtomicU64>>>,
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
            usage_counters: Arc::new(DashMap::new()),
            key_length,
        })
    }

    /// Stamp the atomic counter for `key_id` into a cloned `ApiKey`
    /// value before returning it to a caller. Read paths (validate_key,
    /// list_*, get_key_info) call this so observers see the live
    /// counter value rather than the value last persisted to disk.
    fn stamp_usage_count(&self, mut key: ApiKey) -> ApiKey {
        if let Some(counter) = self.usage_counters.get(&key.id) {
            key.usage_count = counter.load(Ordering::Relaxed);
        }
        key
    }

    /// Bump the per-key usage counter atomically. Called on every
    /// successful credential acceptance from
    /// `AuthManager::validate_api_key`. Returns the new counter value.
    pub fn increment_usage(&self, key_id: &str) -> u64 {
        let counter = self
            .usage_counters
            .entry(key_id.to_string())
            .or_insert_with(|| Arc::new(AtomicU64::new(0)));
        counter.fetch_add(1, Ordering::Relaxed) + 1
    }

    /// Read the current usage counter for a key. Returns 0 if the key
    /// has never been validated since boot AND was never loaded from
    /// disk with a non-zero `usage_count`.
    pub fn current_usage(&self, key_id: &str) -> u64 {
        self.usage_counters
            .get(key_id)
            .map(|c| c.load(Ordering::Relaxed))
            .unwrap_or(0)
    }

    /// Snapshot atomic counters into the in-memory `ApiKey` records so
    /// the next persistence flush round-trips the up-to-date totals.
    /// Takes the write lock briefly; safe to call from a periodic flush
    /// task.
    pub async fn snapshot_usage_counts(&self) {
        let mut keys = self.keys.write().await;
        for (id, key) in keys.iter_mut() {
            if let Some(counter) = self.usage_counters.get(id) {
                key.usage_count = counter.load(Ordering::Relaxed);
            }
        }
    }

    /// Replace a key's `permissions` and (optionally) `scopes`.
    /// `key_hash`, `created_at`, `id`, and `user_id` are immutable.
    /// Returns the updated `ApiKey` snapshot (with live `usage_count`
    /// stamped in) so the handler can echo it to the caller.
    pub async fn update_permissions(
        &self,
        key_id: &str,
        permissions: Vec<crate::auth::roles::Permission>,
        scopes: Option<Vec<crate::auth::TokenScope>>,
    ) -> Result<ApiKey> {
        let mut keys = self.keys.write().await;
        let key = keys
            .get_mut(key_id)
            .ok_or_else(|| VectorizerError::NotFound(format!("API key {} not found", key_id)))?;
        key.permissions = permissions;
        if let Some(s) = scopes {
            key.scopes = s;
        }
        Ok(self.stamp_usage_count(key.clone()))
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
        hex::encode(hasher.finalize())
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
            key_hash: crate::auth::Secret::new(key_hash),
            user_id: user_id.to_string(),
            permissions,
            created_at: now,
            last_used: None,
            expires_at,
            active: true,
            scopes: Vec::new(),
            grace_until: None,
            rotated_to: None,
            usage_count: 0,
        };

        let mut keys = self.keys.write().await;
        keys.insert(key_id.clone(), key_info.clone());
        drop(keys);
        self.usage_counters
            .insert(key_id, Arc::new(AtomicU64::new(0)));

        Ok((api_key, key_info))
    }

    /// Validate an API key and return key information.
    ///
    /// A rotated key whose `grace_until` has not yet passed is still
    /// accepted so that deployed clients can roll over without a 401 gap.
    pub async fn validate_key(&self, api_key: &str) -> Result<ApiKey> {
        let key_hash = self.hash_key(api_key);
        let now = chrono::Utc::now().timestamp() as u64;
        let keys = self.keys.read().await;

        for key_info in keys.values() {
            // Hash comparison against a freshly computed hash. Same algorithm on
            // both sides, so a simple byte-equal is correct (the hash itself is
            // already a constant-size digest; no plaintext leaks).
            if key_info.key_hash.expose_secret() != &key_hash {
                continue;
            }

            if !key_info.active {
                // Rotated-but-within-grace: still accept.
                if let Some(grace) = key_info.grace_until {
                    if now <= grace {
                        return Ok(self.stamp_usage_count(key_info.clone()));
                    }
                }
                return Err(VectorizerError::AuthenticationError(
                    "API key has been revoked".to_string(),
                ));
            }

            // Check expiration
            if let Some(expires_at) = key_info.expires_at {
                if now > expires_at {
                    return Err(VectorizerError::AuthenticationError(
                        "API key has expired".to_string(),
                    ));
                }
            }

            return Ok(self.stamp_usage_count(key_info.clone()));
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
            .map(|k| self.stamp_usage_count(k.clone()))
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
            .map(|k| self.stamp_usage_count(k))
            .ok_or_else(|| VectorizerError::NotFound(format!("API key {} not found", key_id)))
    }

    /// List all API keys (admin function)
    pub async fn list_all_keys(&self) -> Result<Vec<ApiKey>> {
        let keys = self.keys.read().await;
        Ok(keys
            .values()
            .map(|k| self.stamp_usage_count(k.clone()))
            .collect())
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

    /// Register an existing API key (for loading from persistence).
    /// This is used when loading keys from disk on startup.
    ///
    /// Seeds the per-key atomic counter from the persisted
    /// `usage_count` so totals survive restarts.
    pub async fn register_key(&self, key_info: ApiKey) -> Result<()> {
        let id = key_info.id.clone();
        let seed = key_info.usage_count;
        let mut keys = self.keys.write().await;
        keys.insert(id.clone(), key_info);
        drop(keys);
        self.usage_counters
            .insert(id, Arc::new(AtomicU64::new(seed)));
        Ok(())
    }

    /// Hash a key (exposed for persistence layer)
    pub fn hash_key_value(&self, key: &str) -> String {
        self.hash_key(key)
    }

    /// Delete a key completely (for sync with persistence)
    pub async fn delete_key(&self, key_id: &str) -> Result<()> {
        let mut keys = self.keys.write().await;
        keys.remove(key_id);
        drop(keys);
        self.usage_counters.remove(key_id);
        Ok(())
    }

    /// Create a new API key with per-collection scopes.
    ///
    /// Identical to `create_key` except the resulting key carries the
    /// supplied `scopes`.  An empty `scopes` vec means default-deny on
    /// scope-enforced routes.
    pub async fn create_key_with_scopes(
        &self,
        user_id: &str,
        name: &str,
        permissions: Vec<crate::auth::roles::Permission>,
        expires_at: Option<u64>,
        scopes: Vec<crate::auth::TokenScope>,
    ) -> Result<(String, ApiKey)> {
        let key_id = Uuid::new_v4().to_string();
        let api_key = self.generate_key();
        let key_hash = self.hash_key(&api_key);
        let now = chrono::Utc::now().timestamp() as u64;

        let key_info = ApiKey {
            id: key_id.clone(),
            name: name.to_string(),
            key_hash: crate::auth::Secret::new(key_hash),
            user_id: user_id.to_string(),
            permissions,
            created_at: now,
            last_used: None,
            expires_at,
            active: true,
            scopes,
            grace_until: None,
            rotated_to: None,
            usage_count: 0,
        };

        let mut keys = self.keys.write().await;
        keys.insert(key_id.clone(), key_info.clone());
        drop(keys);
        self.usage_counters
            .insert(key_id, Arc::new(AtomicU64::new(0)));

        Ok((api_key, key_info))
    }

    /// Record that `old_key_id` was rotated to `new_key_id` and set its
    /// grace window.  The old key remains `active = true` until
    /// `grace_until` so callers can roll over without a 401 gap.
    pub async fn set_rotation_metadata(
        &self,
        old_key_id: &str,
        new_key_id: String,
        grace_until: u64,
    ) -> Result<()> {
        let mut keys = self.keys.write().await;
        let key = keys.get_mut(old_key_id).ok_or_else(|| {
            VectorizerError::NotFound(format!("API key {} not found", old_key_id))
        })?;
        key.rotated_to = Some(new_key_id);
        key.grace_until = Some(grace_until);
        // Keep active = true; the validate_key path checks grace_until.
        Ok(())
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
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
    async fn usage_count_is_zero_on_a_brand_new_key() {
        let manager = ApiKeyManager::new(32).unwrap();
        let (_token, key) = manager
            .create_key("u", "k", vec![Permission::Read], None)
            .await
            .unwrap();
        assert_eq!(key.usage_count, 0);
        assert_eq!(manager.current_usage(&key.id), 0);
    }

    #[tokio::test]
    async fn validate_key_does_not_increment_counter_directly() {
        // The increment hook lives on `AuthManager::validate_api_key`,
        // not on `ApiKeyManager::validate_key` — so calling validate_key
        // alone leaves the counter at 0. That separation lets
        // introspection (read-only observation of a token) skip the
        // bump. This test pins the contract.
        let manager = ApiKeyManager::new(32).unwrap();
        let (token, key) = manager
            .create_key("u", "k", vec![Permission::Read], None)
            .await
            .unwrap();
        manager.validate_key(&token).await.unwrap();
        assert_eq!(manager.current_usage(&key.id), 0);
    }

    #[tokio::test]
    async fn one_hundred_concurrent_increments_yield_one_hundred() {
        let manager = std::sync::Arc::new(ApiKeyManager::new(32).unwrap());
        let (_token, key) = manager
            .create_key("u", "k", vec![Permission::Read], None)
            .await
            .unwrap();
        let key_id = key.id.clone();

        let mut handles = Vec::with_capacity(100);
        for _ in 0..100 {
            let m = manager.clone();
            let id = key_id.clone();
            handles.push(tokio::spawn(async move {
                m.increment_usage(&id);
            }));
        }
        for h in handles {
            h.await.unwrap();
        }
        assert_eq!(manager.current_usage(&key_id), 100);

        let stamped = manager.get_key_info(&key_id).await.unwrap();
        assert_eq!(stamped.usage_count, 100);
    }

    #[tokio::test]
    async fn register_key_seeds_counter_from_persisted_value() {
        let manager = ApiKeyManager::new(32).unwrap();
        let key = ApiKey {
            id: "persisted".to_string(),
            name: "n".to_string(),
            key_hash: crate::auth::Secret::new("h".to_string()),
            user_id: "u".to_string(),
            permissions: vec![Permission::Read],
            created_at: 0,
            last_used: None,
            expires_at: None,
            active: true,
            scopes: Vec::new(),
            grace_until: None,
            rotated_to: None,
            usage_count: 42,
        };
        manager.register_key(key).await.unwrap();
        assert_eq!(manager.current_usage("persisted"), 42);

        manager.increment_usage("persisted");
        assert_eq!(manager.current_usage("persisted"), 43);
    }

    #[tokio::test]
    async fn snapshot_usage_counts_writes_back_to_in_memory_record() {
        let manager = ApiKeyManager::new(32).unwrap();
        let (_t, key) = manager
            .create_key("u", "k", vec![Permission::Read], None)
            .await
            .unwrap();
        for _ in 0..10 {
            manager.increment_usage(&key.id);
        }
        manager.snapshot_usage_counts().await;
        let raw = manager.list_all_keys().await.unwrap();
        assert_eq!(raw.len(), 1);
        assert_eq!(raw[0].usage_count, 10);
    }

    #[tokio::test]
    async fn update_permissions_replaces_perms_and_keeps_counter() {
        let manager = ApiKeyManager::new(32).unwrap();
        let (_t, key) = manager
            .create_key("u", "k", vec![Permission::Read], None)
            .await
            .unwrap();
        for _ in 0..5 {
            manager.increment_usage(&key.id);
        }
        let updated = manager
            .update_permissions(
                &key.id,
                vec![Permission::Read, Permission::Write],
                Some(Vec::new()),
            )
            .await
            .unwrap();
        assert_eq!(updated.permissions.len(), 2);
        assert!(updated.scopes.is_empty());
        assert_eq!(updated.usage_count, 5);
        assert_eq!(updated.id, key.id);
        assert_eq!(updated.created_at, key.created_at);
    }

    #[tokio::test]
    async fn update_permissions_returns_not_found_for_unknown_key() {
        let manager = ApiKeyManager::new(32).unwrap();
        let err = manager
            .update_permissions("missing", vec![Permission::Read], None)
            .await
            .unwrap_err();
        assert!(matches!(err, VectorizerError::NotFound(_)));
    }

    #[tokio::test]
    async fn delete_key_drops_the_usage_counter() {
        let manager = ApiKeyManager::new(32).unwrap();
        let (_t, key) = manager
            .create_key("u", "k", vec![Permission::Read], None)
            .await
            .unwrap();
        manager.increment_usage(&key.id);
        manager.delete_key(&key.id).await.unwrap();
        assert_eq!(manager.current_usage(&key.id), 0);
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
