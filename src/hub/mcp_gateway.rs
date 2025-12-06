//! MCP Gateway for HiveHub Cloud integration
//!
//! This module provides the bridge between MCP (Model Context Protocol) operations
//! and HiveHub Cloud multi-tenant features including:
//! - User authentication for MCP requests
//! - Tenant-scoped collection filtering
//! - Operation logging and auditing
//! - Access key validation

use std::sync::Arc;

use serde::{Deserialize, Serialize};
use tracing::{debug, error, info, warn};
use uuid::Uuid;

use super::{HubManager, QuotaType, TenantContext, UsageMetrics};
use crate::error::{Result, VectorizerError};

/// MCP operation types for logging and quota tracking
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum McpOperationType {
    /// List collections
    ListCollections,
    /// Create a new collection
    CreateCollection,
    /// Delete a collection
    DeleteCollection,
    /// Get collection info
    GetCollectionInfo,
    /// Insert vector/text
    Insert,
    /// Search vectors
    Search,
    /// Get vector by ID
    GetVector,
    /// Update vector
    UpdateVector,
    /// Delete vector
    DeleteVector,
    /// Graph operation
    GraphOperation,
    /// Cluster operation
    ClusterOperation,
    /// File operation
    FileOperation,
    /// Discovery operation
    Discovery,
    /// Unknown operation
    Unknown,
}

impl McpOperationType {
    /// Parse operation type from MCP tool name
    pub fn from_tool_name(name: &str) -> Self {
        match name {
            "list_collections" => Self::ListCollections,
            "create_collection" => Self::CreateCollection,
            "delete_collection" => Self::DeleteCollection,
            "get_collection_info" => Self::GetCollectionInfo,
            "insert_text" | "insert_vector" | "insert_vectors" => Self::Insert,
            "search"
            | "search_vectors"
            | "search_intelligent"
            | "search_semantic"
            | "search_hybrid"
            | "search_extra"
            | "multi_collection_search" => Self::Search,
            "get_vector" => Self::GetVector,
            "update_vector" => Self::UpdateVector,
            "delete_vector" | "delete_vectors" => Self::DeleteVector,
            name if name.starts_with("graph_") => Self::GraphOperation,
            name if name.starts_with("cluster_") => Self::ClusterOperation,
            "get_file_content"
            | "list_files"
            | "get_file_chunks"
            | "get_project_outline"
            | "get_related_files" => Self::FileOperation,
            "filter_collections" | "expand_queries" => Self::Discovery,
            _ => Self::Unknown,
        }
    }

    /// Check if this operation requires write access
    pub fn requires_write(&self) -> bool {
        matches!(
            self,
            Self::CreateCollection
                | Self::DeleteCollection
                | Self::Insert
                | Self::UpdateVector
                | Self::DeleteVector
        )
    }

    /// Check if this is a read-only operation
    pub fn is_read_only(&self) -> bool {
        !self.requires_write()
    }
}

/// MCP operation log entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpOperationLog {
    /// Operation ID (UUID)
    pub operation_id: Uuid,
    /// Tenant ID (user_id from HiveHub)
    pub tenant_id: String,
    /// MCP tool name
    pub tool_name: String,
    /// Operation type
    pub operation_type: McpOperationType,
    /// Collection name (if applicable)
    pub collection: Option<String>,
    /// Timestamp (Unix epoch milliseconds)
    pub timestamp: u64,
    /// Duration in milliseconds
    pub duration_ms: u64,
    /// Success status
    pub success: bool,
    /// Error message (if failed)
    pub error: Option<String>,
    /// Request metadata
    pub metadata: serde_json::Value,
}

/// MCP Gateway for HiveHub integration
///
/// Provides authentication, authorization, and logging for MCP operations
/// in a multi-tenant HiveHub Cloud environment.
pub struct McpHubGateway {
    /// HiveHub manager
    hub_manager: Arc<HubManager>,
    /// Operation logs buffer
    operation_logs: parking_lot::RwLock<Vec<McpOperationLog>>,
    /// Maximum logs to keep in memory before flushing
    max_logs_buffer: usize,
}

impl McpHubGateway {
    /// Create a new MCP Hub Gateway
    pub fn new(hub_manager: Arc<HubManager>) -> Self {
        Self {
            hub_manager,
            operation_logs: parking_lot::RwLock::new(Vec::new()),
            max_logs_buffer: 1000,
        }
    }

    /// Create gateway with custom buffer size
    pub fn with_buffer_size(hub_manager: Arc<HubManager>, max_logs_buffer: usize) -> Self {
        Self {
            hub_manager,
            operation_logs: parking_lot::RwLock::new(Vec::new()),
            max_logs_buffer,
        }
    }

    /// Check if HiveHub integration is enabled
    pub fn is_enabled(&self) -> bool {
        self.hub_manager.is_enabled()
    }

    /// Validate an MCP access key and return tenant context
    ///
    /// This validates the access key against HiveHub and returns
    /// the tenant context if valid.
    pub async fn validate_access_key(&self, access_key: &str) -> Result<TenantContext> {
        if !self.is_enabled() {
            return Err(VectorizerError::ConfigurationError(
                "HiveHub integration is not enabled".to_string(),
            ));
        }

        debug!("Validating MCP access key");
        self.hub_manager.validate_api_key(access_key).await
    }

    /// Check if a tenant can perform an operation
    ///
    /// Validates both permissions and quotas.
    pub async fn authorize_operation(
        &self,
        tenant: &TenantContext,
        operation: McpOperationType,
        collection: Option<&str>,
    ) -> Result<bool> {
        if !self.is_enabled() {
            return Ok(true); // No authorization in standalone mode
        }

        // Check write permission for write operations
        if operation.requires_write() && !tenant.can_write() {
            warn!(
                "Tenant {} denied write access for operation {:?}",
                tenant.tenant_id(),
                operation
            );
            return Err(VectorizerError::AuthorizationError(
                "Write access not allowed".to_string(),
            ));
        }

        // Check collection ownership if specified
        if let Some(collection_name) = collection {
            // Parse tenant prefix from collection name
            if let Some((owner_id, _)) = self.hub_manager.parse_tenant_collection(collection_name) {
                if owner_id != tenant.tenant_id() && !tenant.can_admin() {
                    warn!(
                        "Tenant {} attempted to access collection owned by {}",
                        tenant.tenant_id(),
                        owner_id
                    );
                    return Err(VectorizerError::AuthorizationError(
                        "Cannot access collections owned by other tenants".to_string(),
                    ));
                }
            }
        }

        // Check quota for create operations
        if matches!(operation, McpOperationType::CreateCollection) {
            let allowed = self
                .hub_manager
                .check_quota(tenant.tenant_id(), QuotaType::CollectionCount, 1)
                .await?;
            if !allowed {
                return Err(VectorizerError::RateLimitExceeded {
                    limit_type: "collection_count".to_string(),
                    limit: 0,
                });
            }
        }

        // Check quota for insert operations
        if matches!(operation, McpOperationType::Insert) {
            let allowed = self
                .hub_manager
                .check_quota(tenant.tenant_id(), QuotaType::VectorCount, 1)
                .await?;
            if !allowed {
                return Err(VectorizerError::RateLimitExceeded {
                    limit_type: "vector_count".to_string(),
                    limit: 0,
                });
            }
        }

        Ok(true)
    }

    /// Get tenant-scoped collection name
    ///
    /// Prefixes the collection name with the tenant ID for isolation.
    pub fn tenant_collection_name(&self, tenant: &TenantContext, collection_name: &str) -> String {
        self.hub_manager
            .tenant_collection_name(tenant.tenant_id(), collection_name)
    }

    /// Filter collections by tenant ownership
    ///
    /// Returns only collections that belong to the tenant.
    pub fn filter_collections_for_tenant(
        &self,
        tenant: &TenantContext,
        all_collections: &[String],
    ) -> Vec<String> {
        let tenant_prefix = format!("{}:", tenant.tenant_id());

        all_collections
            .iter()
            .filter(|name| {
                // Include if:
                // 1. Has tenant prefix matching this tenant
                // 2. Admin users can see all collections
                name.starts_with(&tenant_prefix) || tenant.can_admin()
            })
            .cloned()
            .collect()
    }

    /// Strip tenant prefix from collection name for display
    ///
    /// Returns the user-facing collection name without the tenant prefix.
    pub fn display_collection_name(&self, tenant: &TenantContext, full_name: &str) -> String {
        if let Some((owner_id, collection_name)) =
            self.hub_manager.parse_tenant_collection(full_name)
        {
            if owner_id == tenant.tenant_id() {
                return collection_name;
            }
            // For admin viewing other tenant's collections, show full name
            return full_name.to_string();
        }
        full_name.to_string()
    }

    /// Log an MCP operation
    pub fn log_operation(&self, log: McpOperationLog) {
        let mut logs = self.operation_logs.write();
        logs.push(log);

        // Flush if buffer is full
        if logs.len() >= self.max_logs_buffer {
            // In a production system, this would send to HiveHub or a logging service
            info!(
                "MCP operation logs buffer full ({} entries), would flush to logging service",
                logs.len()
            );
            logs.clear();
        }
    }

    /// Create an operation log entry
    pub fn create_log_entry(
        &self,
        tenant: &TenantContext,
        tool_name: &str,
        collection: Option<&str>,
        duration_ms: u64,
        success: bool,
        error: Option<String>,
        metadata: serde_json::Value,
    ) -> McpOperationLog {
        McpOperationLog {
            operation_id: Uuid::new_v4(),
            tenant_id: tenant.tenant_id().to_string(),
            tool_name: tool_name.to_string(),
            operation_type: McpOperationType::from_tool_name(tool_name),
            collection: collection.map(String::from),
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_millis() as u64,
            duration_ms,
            success,
            error,
            metadata,
        }
    }

    /// Get pending operation logs count
    pub fn pending_logs_count(&self) -> usize {
        self.operation_logs.read().len()
    }

    /// Flush operation logs
    ///
    /// In production, this would send logs to HiveHub Cloud.
    pub async fn flush_logs(&self) -> Result<usize> {
        let mut logs = self.operation_logs.write();
        let count = logs.len();

        if count > 0 {
            debug!("Flushing {} MCP operation logs", count);
            // TODO: Send to HiveHub Cloud logging endpoint
            logs.clear();
        }

        Ok(count)
    }

    /// Record usage metrics for an operation
    pub async fn record_operation_usage(
        &self,
        tenant: &TenantContext,
        operation: McpOperationType,
        collection_id: Option<Uuid>,
    ) -> Result<()> {
        if !self.is_enabled() {
            return Ok(());
        }

        // Only track usage for operations that consume resources
        let metrics = match operation {
            McpOperationType::Insert => UsageMetrics {
                vectors_inserted: 1,
                ..Default::default()
            },
            McpOperationType::Search => UsageMetrics {
                search_count: 1,
                ..Default::default()
            },
            _ => return Ok(()), // No usage tracking for other operations
        };

        if let Some(coll_id) = collection_id {
            self.hub_manager.record_usage(coll_id, metrics).await?;
        }

        Ok(())
    }

    /// Get the underlying HubManager
    pub fn hub_manager(&self) -> &Arc<HubManager> {
        &self.hub_manager
    }
}

/// MCP request context with tenant information
#[derive(Debug, Clone)]
pub struct McpRequestContext {
    /// Tenant context from authentication
    pub tenant: TenantContext,
    /// Request ID for tracing
    pub request_id: Uuid,
    /// Request start time
    pub start_time: std::time::Instant,
}

impl McpRequestContext {
    /// Create a new request context
    pub fn new(tenant: TenantContext) -> Self {
        Self {
            tenant,
            request_id: Uuid::new_v4(),
            start_time: std::time::Instant::now(),
        }
    }

    /// Get elapsed time since request start in milliseconds
    pub fn elapsed_ms(&self) -> u64 {
        self.start_time.elapsed().as_millis() as u64
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_operation_type_from_tool_name() {
        assert_eq!(
            McpOperationType::from_tool_name("list_collections"),
            McpOperationType::ListCollections
        );
        assert_eq!(
            McpOperationType::from_tool_name("search"),
            McpOperationType::Search
        );
        assert_eq!(
            McpOperationType::from_tool_name("insert_text"),
            McpOperationType::Insert
        );
        assert_eq!(
            McpOperationType::from_tool_name("graph_list_nodes"),
            McpOperationType::GraphOperation
        );
        assert_eq!(
            McpOperationType::from_tool_name("cluster_add_node"),
            McpOperationType::ClusterOperation
        );
        assert_eq!(
            McpOperationType::from_tool_name("unknown_tool"),
            McpOperationType::Unknown
        );
    }

    #[test]
    fn test_operation_requires_write() {
        assert!(McpOperationType::CreateCollection.requires_write());
        assert!(McpOperationType::Insert.requires_write());
        assert!(McpOperationType::DeleteVector.requires_write());
        assert!(!McpOperationType::Search.requires_write());
        assert!(!McpOperationType::ListCollections.requires_write());
    }

    #[test]
    fn test_operation_is_read_only() {
        assert!(!McpOperationType::CreateCollection.is_read_only());
        assert!(McpOperationType::Search.is_read_only());
        assert!(McpOperationType::GetVector.is_read_only());
    }
}
