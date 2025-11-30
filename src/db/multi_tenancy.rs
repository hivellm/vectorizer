//! Multi-tenancy support for Vectorizer
//!
//! This module provides tenant isolation, resource quotas, and access control.

use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};

use dashmap::DashMap;
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use tracing::{debug, error, info, warn};

use crate::error::{Result, VectorizerError};

/// Tenant ID type
pub type TenantId = String;

/// Resource quotas for a tenant
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TenantQuotas {
    /// Maximum memory usage in bytes
    pub max_memory_bytes: Option<u64>,
    /// Maximum number of collections
    pub max_collections: Option<usize>,
    /// Maximum vectors per collection
    pub max_vectors_per_collection: Option<usize>,
    /// Maximum queries per second (QPS)
    pub max_qps: Option<u32>,
    /// Maximum storage in bytes
    pub max_storage_bytes: Option<u64>,
}

impl Default for TenantQuotas {
    fn default() -> Self {
        Self {
            max_memory_bytes: Some(1_073_741_824), // 1 GB default
            max_collections: Some(10),
            max_vectors_per_collection: Some(1_000_000),
            max_qps: Some(100),
            max_storage_bytes: Some(10_737_418_240), // 10 GB default
        }
    }
}

/// Current resource usage for a tenant
#[derive(Debug, Clone)]
pub struct TenantUsage {
    /// Current memory usage in bytes
    pub memory_bytes: u64,
    /// Current number of collections
    pub collection_count: usize,
    /// Current total vectors across all collections
    pub total_vectors: usize,
    /// Current QPS (queries per second)
    pub current_qps: u32,
    /// Current storage usage in bytes
    pub storage_bytes: u64,
    /// Timestamp of last QPS calculation
    pub last_qps_calc: Instant,
    /// Query count for QPS calculation
    pub query_count: u32,
}

impl Default for TenantUsage {
    fn default() -> Self {
        Self {
            memory_bytes: 0,
            collection_count: 0,
            total_vectors: 0,
            current_qps: 0,
            storage_bytes: 0,
            last_qps_calc: Instant::now(),
            query_count: 0,
        }
    }
}

/// Tenant metadata
#[derive(Debug, Clone)]
pub struct TenantMetadata {
    /// Tenant ID
    pub tenant_id: TenantId,
    /// Resource quotas
    pub quotas: TenantQuotas,
    /// Current usage
    pub usage: Arc<RwLock<TenantUsage>>,
    /// Created timestamp
    pub created_at: chrono::DateTime<chrono::Utc>,
}

impl TenantMetadata {
    /// Create a new tenant with default quotas
    pub fn new(tenant_id: TenantId) -> Self {
        Self {
            tenant_id: tenant_id.clone(),
            quotas: TenantQuotas::default(),
            usage: Arc::new(RwLock::new(TenantUsage::default())),
            created_at: chrono::Utc::now(),
        }
    }

    /// Create a new tenant with custom quotas
    pub fn with_quotas(tenant_id: TenantId, quotas: TenantQuotas) -> Self {
        Self {
            tenant_id: tenant_id.clone(),
            quotas,
            usage: Arc::new(RwLock::new(TenantUsage::default())),
            created_at: chrono::Utc::now(),
        }
    }

    /// Check if tenant can create a new collection
    pub fn can_create_collection(&self) -> Result<()> {
        let usage = self.usage.read();

        if let Some(max) = self.quotas.max_collections {
            if usage.collection_count >= max {
                return Err(VectorizerError::InvalidConfiguration {
                    message: format!(
                        "Tenant {} has reached maximum collections limit ({})",
                        self.tenant_id, max
                    ),
                });
            }
        }

        Ok(())
    }

    /// Check if tenant can add vectors to a collection
    pub fn can_add_vectors(&self, count: usize) -> Result<()> {
        let usage = self.usage.read();

        if let Some(max) = self.quotas.max_vectors_per_collection {
            if usage.total_vectors + count > max {
                return Err(VectorizerError::InvalidConfiguration {
                    message: format!(
                        "Tenant {} would exceed maximum vectors per collection limit ({})",
                        self.tenant_id, max
                    ),
                });
            }
        }

        Ok(())
    }

    /// Check if tenant can use memory
    pub fn can_use_memory(&self, bytes: u64) -> Result<()> {
        let usage = self.usage.read();

        if let Some(max) = self.quotas.max_memory_bytes {
            if usage.memory_bytes + bytes > max {
                return Err(VectorizerError::InvalidConfiguration {
                    message: format!(
                        "Tenant {} would exceed maximum memory limit ({} bytes)",
                        self.tenant_id, max
                    ),
                });
            }
        }

        Ok(())
    }

    /// Check if tenant can use storage
    pub fn can_use_storage(&self, bytes: u64) -> Result<()> {
        let usage = self.usage.read();

        if let Some(max) = self.quotas.max_storage_bytes {
            if usage.storage_bytes + bytes > max {
                return Err(VectorizerError::InvalidConfiguration {
                    message: format!(
                        "Tenant {} would exceed maximum storage limit ({} bytes)",
                        self.tenant_id, max
                    ),
                });
            }
        }

        Ok(())
    }

    /// Check if tenant can perform query (QPS check)
    pub fn can_perform_query(&self) -> Result<()> {
        let mut usage = self.usage.write();

        // Update QPS calculation
        let now = Instant::now();
        if now.duration_since(usage.last_qps_calc) > Duration::from_secs(1) {
            usage.current_qps = usage.query_count;
            usage.query_count = 0;
            usage.last_qps_calc = now;
        }

        usage.query_count += 1;

        if let Some(max) = self.quotas.max_qps {
            if usage.current_qps >= max {
                return Err(VectorizerError::InvalidConfiguration {
                    message: format!(
                        "Tenant {} has reached maximum QPS limit ({})",
                        self.tenant_id, max
                    ),
                });
            }
        }

        Ok(())
    }

    /// Update memory usage
    pub fn update_memory_usage(&self, delta: i64) {
        let mut usage = self.usage.write();
        if delta > 0 {
            usage.memory_bytes += delta as u64;
        } else {
            usage.memory_bytes = usage.memory_bytes.saturating_sub((-delta) as u64);
        }
    }

    /// Update collection count
    pub fn update_collection_count(&self, delta: i32) {
        let mut usage = self.usage.write();
        if delta > 0 {
            usage.collection_count += delta as usize;
        } else {
            usage.collection_count = usage.collection_count.saturating_sub((-delta) as usize);
        }
    }

    /// Update vector count
    pub fn update_vector_count(&self, delta: i64) {
        let mut usage = self.usage.write();
        if delta > 0 {
            usage.total_vectors += delta as usize;
        } else {
            usage.total_vectors = usage.total_vectors.saturating_sub((-delta) as usize);
        }
    }

    /// Update storage usage
    pub fn update_storage_usage(&self, delta: i64) {
        let mut usage = self.usage.write();
        if delta > 0 {
            usage.storage_bytes += delta as u64;
        } else {
            usage.storage_bytes = usage.storage_bytes.saturating_sub((-delta) as u64);
        }
    }

    /// Get current usage
    pub fn get_usage(&self) -> TenantUsage {
        self.usage.read().clone()
    }
}

/// Multi-tenancy manager
#[derive(Debug, Clone)]
pub struct MultiTenancyManager {
    /// Tenant metadata by tenant ID
    tenants: Arc<DashMap<TenantId, TenantMetadata>>,
    /// Collection to tenant mapping
    collection_tenants: Arc<DashMap<String, TenantId>>,
}

impl MultiTenancyManager {
    /// Create a new multi-tenancy manager
    pub fn new() -> Self {
        Self {
            tenants: Arc::new(DashMap::new()),
            collection_tenants: Arc::new(DashMap::new()),
        }
    }

    /// Register a new tenant
    pub fn register_tenant(&self, tenant_id: TenantId, quotas: Option<TenantQuotas>) {
        let metadata = if let Some(q) = quotas {
            TenantMetadata::with_quotas(tenant_id.clone(), q)
        } else {
            TenantMetadata::new(tenant_id.clone())
        };

        self.tenants.insert(tenant_id.clone(), metadata);
        info!("Registered tenant: {}", tenant_id);
    }

    /// Get tenant metadata
    pub fn get_tenant(&self, tenant_id: &TenantId) -> Option<TenantMetadata> {
        self.tenants.get(tenant_id).map(|t| t.value().clone())
    }

    /// Associate a collection with a tenant
    pub fn associate_collection(&self, collection_name: &str, tenant_id: &TenantId) -> Result<()> {
        // Verify tenant exists
        if !self.tenants.contains_key(tenant_id) {
            return Err(VectorizerError::InvalidConfiguration {
                message: format!("Tenant {} does not exist", tenant_id),
            });
        }

        // Check if collection already associated with different tenant
        if let Some(existing_tenant) = self.collection_tenants.get(collection_name) {
            if existing_tenant.value() != tenant_id {
                return Err(VectorizerError::InvalidConfiguration {
                    message: format!(
                        "Collection {} already associated with tenant {}",
                        collection_name,
                        existing_tenant.value()
                    ),
                });
            }
            return Ok(()); // Already associated with same tenant
        }

        // Associate collection
        self.collection_tenants
            .insert(collection_name.to_string(), tenant_id.clone());

        // Update tenant collection count
        if let Some(tenant) = self.tenants.get(tenant_id) {
            tenant.can_create_collection()?;
            tenant.update_collection_count(1);
        }

        debug!(
            "Associated collection '{}' with tenant '{}'",
            collection_name, tenant_id
        );
        Ok(())
    }

    /// Get tenant for a collection
    pub fn get_collection_tenant(&self, collection_name: &str) -> Option<TenantId> {
        self.collection_tenants
            .get(collection_name)
            .map(|t| t.value().clone())
    }

    /// Check if tenant can access collection
    pub fn can_access_collection(&self, tenant_id: &TenantId, collection_name: &str) -> Result<()> {
        if let Some(collection_tenant) = self.get_collection_tenant(collection_name) {
            if collection_tenant != *tenant_id {
                return Err(VectorizerError::InvalidConfiguration {
                    message: format!(
                        "Tenant {} does not have access to collection {}",
                        tenant_id, collection_name
                    ),
                });
            }
        }
        Ok(())
    }

    /// Check if tenant can perform operation
    pub fn check_tenant_quota(
        &self,
        tenant_id: &TenantId,
        operation: &TenantOperation,
    ) -> Result<()> {
        let tenant =
            self.tenants
                .get(tenant_id)
                .ok_or_else(|| VectorizerError::InvalidConfiguration {
                    message: format!("Tenant {} does not exist", tenant_id),
                })?;

        match operation {
            TenantOperation::CreateCollection => tenant.can_create_collection()?,
            TenantOperation::AddVectors(count) => tenant.can_add_vectors(*count)?,
            TenantOperation::UseMemory(bytes) => tenant.can_use_memory(*bytes)?,
            TenantOperation::UseStorage(bytes) => tenant.can_use_storage(*bytes)?,
            TenantOperation::PerformQuery => tenant.can_perform_query()?,
        }

        Ok(())
    }

    /// Update tenant usage
    pub fn update_usage(&self, tenant_id: &TenantId, update: TenantUsageUpdate) {
        if let Some(tenant) = self.tenants.get(tenant_id) {
            match update {
                TenantUsageUpdate::Memory(delta) => tenant.update_memory_usage(delta),
                TenantUsageUpdate::Storage(delta) => tenant.update_storage_usage(delta),
                TenantUsageUpdate::CollectionCount(delta) => tenant.update_collection_count(delta),
                TenantUsageUpdate::VectorCount(delta) => tenant.update_vector_count(delta),
            }
        }
    }

    /// Remove collection association
    pub fn remove_collection(&self, collection_name: &str) {
        if let Some((_, tenant_id)) = self.collection_tenants.remove(collection_name) {
            if let Some(tenant) = self.tenants.get(&tenant_id) {
                tenant.update_collection_count(-1);
            }
        }
    }

    /// List all tenants
    pub fn list_tenants(&self) -> Vec<TenantId> {
        self.tenants.iter().map(|t| t.key().clone()).collect()
    }

    /// Get tenant usage statistics
    pub fn get_tenant_stats(&self, tenant_id: &TenantId) -> Option<TenantUsage> {
        self.tenants.get(tenant_id).map(|t| t.get_usage())
    }
}

impl Default for MultiTenancyManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Tenant operations for quota checking
#[derive(Debug, Clone)]
pub enum TenantOperation {
    /// Create a new collection
    CreateCollection,
    /// Add vectors to a collection
    AddVectors(usize),
    /// Use memory
    UseMemory(u64),
    /// Use storage
    UseStorage(u64),
    /// Perform a query
    PerformQuery,
}

/// Tenant usage updates
#[derive(Debug, Clone)]
pub enum TenantUsageUpdate {
    /// Memory usage delta (bytes)
    Memory(i64),
    /// Storage usage delta (bytes)
    Storage(i64),
    /// Collection count delta
    CollectionCount(i32),
    /// Vector count delta
    VectorCount(i64),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tenant_creation() {
        let manager = MultiTenancyManager::new();
        manager.register_tenant("tenant1".to_string(), None);

        assert!(manager.get_tenant(&"tenant1".to_string()).is_some());
    }

    #[test]
    fn test_collection_association() {
        let manager = MultiTenancyManager::new();
        manager.register_tenant("tenant1".to_string(), None);

        manager
            .associate_collection("collection1", &"tenant1".to_string())
            .unwrap();

        let tenant = manager.get_collection_tenant("collection1");
        assert_eq!(tenant, Some("tenant1".to_string()));
    }

    #[test]
    fn test_tenant_quota_checking() {
        let manager = MultiTenancyManager::new();
        let quotas = TenantQuotas {
            max_collections: Some(2),
            ..Default::default()
        };
        manager.register_tenant("tenant1".to_string(), Some(quotas));

        // Should be able to create first collection
        manager
            .check_tenant_quota(&"tenant1".to_string(), &TenantOperation::CreateCollection)
            .unwrap();

        manager
            .associate_collection("collection1", &"tenant1".to_string())
            .unwrap();

        // Should be able to create second collection
        manager
            .check_tenant_quota(&"tenant1".to_string(), &TenantOperation::CreateCollection)
            .unwrap();
        manager
            .associate_collection("collection2", &"tenant1".to_string())
            .unwrap();

        // Should fail on third collection
        assert!(
            manager
                .check_tenant_quota(&"tenant1".to_string(), &TenantOperation::CreateCollection)
                .is_err()
        );
    }

    #[test]
    fn test_tenant_access_control() {
        let manager = MultiTenancyManager::new();
        manager.register_tenant("tenant1".to_string(), None);
        manager.register_tenant("tenant2".to_string(), None);

        manager
            .associate_collection("collection1", &"tenant1".to_string())
            .unwrap();

        // Tenant1 should have access
        manager
            .can_access_collection(&"tenant1".to_string(), "collection1")
            .unwrap();

        // Tenant2 should not have access
        assert!(
            manager
                .can_access_collection(&"tenant2".to_string(), "collection1")
                .is_err()
        );
    }
}
