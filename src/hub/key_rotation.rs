//! API Key Rotation Module
//!
//! Provides secure API key rotation capabilities for HiveHub integration.
//! Supports graceful key rotation with overlap periods to prevent service disruption.
//!
//! ## Key Rotation Flow
//!
//! 1. Generate new API key while old key is still valid
//! 2. Both keys work during grace period (default: 7 days)
//! 3. Update clients to use new key
//! 4. Old key expires after grace period
//! 5. Optionally revoke old key immediately after migration

use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, SystemTime};

use chrono::{DateTime, Utc};
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use tracing::{debug, info, warn};
use uuid::Uuid;

use super::auth::TenantContext;
use super::client::HubClient;
use crate::error::{Result, VectorizerError};

/// Default grace period for key rotation (7 days)
pub const DEFAULT_GRACE_PERIOD_SECS: u64 = 7 * 24 * 60 * 60;

/// API key rotation status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum KeyStatus {
    /// Key is active and can be used
    Active,
    /// Key is being rotated (both old and new are valid)
    Rotating,
    /// Key is deprecated but still valid during grace period
    Deprecated,
    /// Key has been revoked and cannot be used
    Revoked,
    /// Key has expired
    Expired,
}

/// Information about an API key rotation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyRotation {
    /// Tenant ID
    pub tenant_id: String,
    /// Old API key ID (hash, not the actual key)
    pub old_key_id: String,
    /// New API key ID (hash, not the actual key)
    pub new_key_id: String,
    /// Rotation initiated timestamp
    pub initiated_at: DateTime<Utc>,
    /// Grace period end timestamp
    pub grace_period_end: DateTime<Utc>,
    /// Status of the rotation
    pub status: KeyStatus,
    /// Whether old key has been revoked
    pub old_key_revoked: bool,
}

impl KeyRotation {
    /// Check if rotation is complete
    pub fn is_complete(&self) -> bool {
        self.old_key_revoked || Utc::now() > self.grace_period_end
    }

    /// Check if old key is still valid
    pub fn old_key_valid(&self) -> bool {
        !self.old_key_revoked && Utc::now() <= self.grace_period_end
    }

    /// Get days remaining in grace period
    pub fn days_remaining(&self) -> i64 {
        (self.grace_period_end - Utc::now()).num_days()
    }
}

/// Key rotation manager
#[derive(Debug)]
pub struct KeyRotationManager {
    /// HiveHub client
    client: Arc<HubClient>,
    /// Active rotations (tenant_id -> rotation)
    active_rotations: Arc<RwLock<HashMap<String, KeyRotation>>>,
    /// Deprecated keys that are still valid (key_hash -> expiry_time)
    deprecated_keys: Arc<RwLock<HashMap<String, SystemTime>>>,
    /// Default grace period
    grace_period: Duration,
}

impl KeyRotationManager {
    /// Create a new key rotation manager
    pub fn new(client: Arc<HubClient>, grace_period_secs: Option<u64>) -> Self {
        let grace_period =
            Duration::from_secs(grace_period_secs.unwrap_or(DEFAULT_GRACE_PERIOD_SECS));

        Self {
            client,
            active_rotations: Arc::new(RwLock::new(HashMap::new())),
            deprecated_keys: Arc::new(RwLock::new(HashMap::new())),
            grace_period,
        }
    }

    /// Initiate key rotation for a tenant
    ///
    /// Returns the new key ID (not the actual key - that should be retrieved from HiveHub)
    pub async fn initiate_rotation(&self, tenant_id: &str, old_key_hash: &str) -> Result<String> {
        info!("Initiating key rotation for tenant: {}", tenant_id);

        // Check if there's already an active rotation
        {
            let rotations = self.active_rotations.read();
            if let Some(rotation) = rotations.get(tenant_id) {
                if !rotation.is_complete() {
                    warn!(
                        "Tenant {} already has an active rotation (old_key: {})",
                        tenant_id, rotation.old_key_id
                    );
                    return Err(VectorizerError::ConfigurationError(
                        "Key rotation already in progress".to_string(),
                    ));
                }
            }
        }

        // Generate new key ID
        let new_key_id = format!("key_{}", Uuid::new_v4().simple());

        let now = Utc::now();
        let grace_period_end = now
            + chrono::Duration::from_std(self.grace_period).map_err(|e| {
                VectorizerError::ConfigurationError(format!("Invalid grace period: {}", e))
            })?;

        // Create rotation record
        let rotation = KeyRotation {
            tenant_id: tenant_id.to_string(),
            old_key_id: old_key_hash.to_string(),
            new_key_id: new_key_id.clone(),
            initiated_at: now,
            grace_period_end,
            status: KeyStatus::Rotating,
            old_key_revoked: false,
        };

        // Store rotation info
        {
            let mut rotations = self.active_rotations.write();
            rotations.insert(tenant_id.to_string(), rotation);
        }

        // Mark old key as deprecated but still valid
        {
            let mut deprecated = self.deprecated_keys.write();
            let expiry = SystemTime::now() + self.grace_period;
            deprecated.insert(old_key_hash.to_string(), expiry);
        }

        info!(
            "Key rotation initiated for tenant {} (grace period: {} days)",
            tenant_id,
            self.grace_period.as_secs() / 86400
        );

        Ok(new_key_id)
    }

    /// Complete a key rotation by revoking the old key
    pub async fn complete_rotation(&self, tenant_id: &str) -> Result<()> {
        info!("Completing key rotation for tenant: {}", tenant_id);

        let old_key_id = {
            let mut rotations = self.active_rotations.write();

            if let Some(rotation) = rotations.get_mut(tenant_id) {
                if rotation.old_key_revoked {
                    return Err(VectorizerError::ConfigurationError(
                        "Key rotation already completed".to_string(),
                    ));
                }

                rotation.old_key_revoked = true;
                rotation.status = KeyStatus::Revoked;

                // Remove from deprecated keys immediately
                let mut deprecated = self.deprecated_keys.write();
                deprecated.remove(&rotation.old_key_id);

                rotation.old_key_id.clone()
            } else {
                return Err(VectorizerError::NotFound(
                    "No active rotation found".to_string(),
                ));
            }
        };

        info!(
            "Key rotation completed for tenant {} (old key revoked: {})",
            tenant_id, old_key_id
        );

        Ok(())
    }

    /// Cancel an ongoing rotation (revert to old key)
    pub async fn cancel_rotation(&self, tenant_id: &str) -> Result<()> {
        warn!("Canceling key rotation for tenant: {}", tenant_id);

        let mut rotations = self.active_rotations.write();

        if let Some(rotation) = rotations.remove(tenant_id) {
            // Remove old key from deprecated list
            let mut deprecated = self.deprecated_keys.write();
            deprecated.remove(&rotation.old_key_id);

            info!(
                "Key rotation canceled for tenant {} (restored old key: {})",
                tenant_id, rotation.old_key_id
            );

            Ok(())
        } else {
            Err(VectorizerError::NotFound(
                "No active rotation found".to_string(),
            ))
        }
    }

    /// Get rotation status for a tenant
    pub fn get_rotation_status(&self, tenant_id: &str) -> Option<KeyRotation> {
        let rotations = self.active_rotations.read();
        rotations.get(tenant_id).cloned()
    }

    /// Check if a key is deprecated but still valid
    pub fn is_key_deprecated(&self, key_hash: &str) -> bool {
        let deprecated = self.deprecated_keys.read();

        if let Some(expiry) = deprecated.get(key_hash) {
            // Check if not expired
            SystemTime::now() < *expiry
        } else {
            false
        }
    }

    /// Check if a key is revoked
    pub fn is_key_revoked(&self, key_hash: &str) -> bool {
        let rotations = self.active_rotations.read();

        for rotation in rotations.values() {
            if rotation.old_key_id == key_hash && rotation.old_key_revoked {
                return true;
            }
        }

        false
    }

    /// Clean up expired rotations and deprecated keys
    pub fn cleanup_expired(&self) {
        let now = SystemTime::now();

        // Clean up expired deprecated keys
        {
            let mut deprecated = self.deprecated_keys.write();
            deprecated.retain(|key_hash, expiry| {
                if now >= *expiry {
                    debug!("Removing expired deprecated key: {}", &key_hash[..8]);
                    false
                } else {
                    true
                }
            });
        }

        // Clean up completed rotations (optional - keep for audit trail)
        {
            let mut rotations = self.active_rotations.write();
            let expired: Vec<String> = rotations
                .iter()
                .filter(|(_, rotation)| rotation.is_complete())
                .map(|(tenant_id, _)| tenant_id.clone())
                .collect();

            for tenant_id in expired {
                if let Some(rotation) = rotations.remove(&tenant_id) {
                    debug!(
                        "Cleaning up completed rotation for tenant: {} (old_key: {})",
                        tenant_id, rotation.old_key_id
                    );
                }
            }
        }
    }

    /// Get all active rotations
    pub fn get_all_rotations(&self) -> Vec<KeyRotation> {
        let rotations = self.active_rotations.read();
        rotations.values().cloned().collect()
    }

    /// Get rotations that are nearing expiry (within 24 hours)
    pub fn get_expiring_rotations(&self) -> Vec<KeyRotation> {
        let rotations = self.active_rotations.read();
        let threshold = Utc::now() + chrono::Duration::hours(24);

        rotations
            .values()
            .filter(|r| !r.old_key_revoked && r.grace_period_end <= threshold)
            .cloned()
            .collect()
    }

    /// Validate a key considering rotation status
    pub fn validate_with_rotation(&self, key_hash: &str) -> KeyStatus {
        // Check if revoked
        if self.is_key_revoked(key_hash) {
            return KeyStatus::Revoked;
        }

        // Check if deprecated but still valid
        if self.is_key_deprecated(key_hash) {
            return KeyStatus::Deprecated;
        }

        // Check if expired in deprecated list
        let deprecated = self.deprecated_keys.read();
        if let Some(expiry) = deprecated.get(key_hash) {
            if SystemTime::now() >= *expiry {
                return KeyStatus::Expired;
            }
        }

        KeyStatus::Active
    }

    /// Invalidate cache for a specific key
    /// This should be called after key operations to ensure fresh validation
    pub fn invalidate_key_cache(&self, _key_hash: &str) {
        // Implementation would integrate with HubAuth cache
        debug!("Key cache invalidation requested");
    }

    /// Get grace period duration
    pub fn grace_period(&self) -> Duration {
        self.grace_period
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use super::*;

    fn create_test_manager() -> KeyRotationManager {
        let client = Arc::new(HubClient::new_mock());
        KeyRotationManager::new(client, Some(60)) // 60 seconds for testing
    }

    #[tokio::test]
    async fn test_initiate_rotation() {
        let manager = create_test_manager();
        let tenant_id = "test_tenant";
        let old_key = "old_key_hash_12345678";

        let result = manager.initiate_rotation(tenant_id, old_key).await;
        assert!(result.is_ok());

        let status = manager.get_rotation_status(tenant_id);
        assert!(status.is_some());
        assert_eq!(status.unwrap().status, KeyStatus::Rotating);
    }

    #[tokio::test]
    async fn test_deprecated_key_validation() {
        let manager = create_test_manager();
        let tenant_id = "test_tenant";
        let old_key = "old_key_hash_87654321";

        // Initiate rotation
        manager.initiate_rotation(tenant_id, old_key).await.unwrap();

        // Old key should still be valid
        assert!(manager.is_key_deprecated(old_key));
        assert_eq!(
            manager.validate_with_rotation(old_key),
            KeyStatus::Deprecated
        );
    }

    #[tokio::test]
    async fn test_complete_rotation() {
        let manager = create_test_manager();
        let tenant_id = "test_tenant";
        let old_key = "old_key_hash_complete";

        // Initiate and complete rotation
        manager.initiate_rotation(tenant_id, old_key).await.unwrap();
        manager.complete_rotation(tenant_id).await.unwrap();

        // Old key should be revoked
        assert!(manager.is_key_revoked(old_key));
        assert_eq!(manager.validate_with_rotation(old_key), KeyStatus::Revoked);
    }

    #[tokio::test]
    async fn test_cancel_rotation() {
        let manager = create_test_manager();
        let tenant_id = "test_tenant";
        let old_key = "old_key_hash_cancel";

        // Initiate and cancel rotation
        manager.initiate_rotation(tenant_id, old_key).await.unwrap();
        manager.cancel_rotation(tenant_id).await.unwrap();

        // No rotation should exist
        assert!(manager.get_rotation_status(tenant_id).is_none());
        assert!(!manager.is_key_deprecated(old_key));
    }
}
