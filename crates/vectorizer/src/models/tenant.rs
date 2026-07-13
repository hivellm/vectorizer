//! Tenant identity + permission model.
//!
//! Moved here from `hub/auth.rs` in phase41: `cluster/server_client.rs`
//! needed `TenantContext`, and a `cluster → hub` import was one of the
//! nine upward back-references blocking the workspace split (analysis
//! 2026-07-11 §1.1). `models/` is the foundation data layer, so both
//! `hub` and `cluster` may depend on it. `hub/auth.rs` re-exports these
//! types at their old paths, and the SDK-specific conversion
//! (`TenantPermission::from_sdk_permission`) stays there because it
//! depends on the HiveHub SDK types.

use serde::{Deserialize, Serialize};

/// Permission levels for API keys
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum TenantPermission {
    /// Full administrative access
    Admin,
    /// Read and write data operations
    ReadWrite,
    /// Read-only access
    ReadOnly,
    /// MCP protocol access (limited)
    Mcp,
}

impl TenantPermission {
    /// Check if this permission allows the specified operation
    pub fn allows(&self, operation: &str) -> bool {
        match self {
            TenantPermission::Admin => true,
            TenantPermission::ReadWrite => {
                matches!(
                    operation,
                    "create_collection"
                        | "delete_collection"
                        | "list_collections"
                        | "insert_vectors"
                        | "update_vectors"
                        | "delete_vectors"
                        | "search"
                        | "get_collection"
                        | "get_vector"
                )
            }
            TenantPermission::ReadOnly => {
                matches!(
                    operation,
                    "list_collections" | "search" | "get_collection" | "get_vector"
                )
            }
            TenantPermission::Mcp => {
                matches!(
                    operation,
                    "list_collections" | "insert_vectors" | "update_vectors" | "search"
                )
            }
        }
    }

    /// Check if this permission level is admin
    pub fn is_admin(&self) -> bool {
        matches!(self, TenantPermission::Admin)
    }

    /// Check if this permission allows write operations
    pub fn can_write(&self) -> bool {
        matches!(
            self,
            TenantPermission::Admin | TenantPermission::ReadWrite | TenantPermission::Mcp
        )
    }
}

impl std::fmt::Display for TenantPermission {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TenantPermission::Admin => write!(f, "ADMIN"),
            TenantPermission::ReadWrite => write!(f, "READ_WRITE"),
            TenantPermission::ReadOnly => write!(f, "READ_ONLY"),
            TenantPermission::Mcp => write!(f, "MCP"),
        }
    }
}

/// Tenant context extracted from a validated API key
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TenantContext {
    /// Unique tenant identifier
    pub tenant_id: String,
    /// Tenant display name
    pub tenant_name: String,
    /// API key identifier (for logging, not the actual key)
    pub api_key_id: String,
    /// Permissions granted to this API key
    pub permissions: Vec<TenantPermission>,
    /// Rate limit overrides (if any)
    pub rate_limits: Option<TenantRateLimits>,
    /// Validation timestamp
    pub validated_at: chrono::DateTime<chrono::Utc>,
    /// Whether this is a test key
    pub is_test: bool,
}

impl TenantContext {
    /// Get the tenant ID
    pub fn tenant_id(&self) -> &str {
        &self.tenant_id
    }

    /// Check if tenant has a specific permission
    pub fn has_permission(&self, permission: TenantPermission) -> bool {
        self.permissions.contains(&permission)
            || self.permissions.contains(&TenantPermission::Admin)
    }

    /// Check if tenant is allowed to perform an operation
    pub fn can_perform(&self, operation: &str) -> bool {
        self.permissions.iter().any(|p| p.allows(operation))
    }

    /// Check if this is an admin tenant
    pub fn is_admin(&self) -> bool {
        self.permissions.contains(&TenantPermission::Admin)
    }

    /// Check if tenant has admin-level access
    pub fn can_admin(&self) -> bool {
        self.permissions.contains(&TenantPermission::Admin)
    }

    /// Check if tenant can perform write operations
    pub fn can_write(&self) -> bool {
        self.permissions.iter().any(|p| p.can_write())
    }

    /// Get the highest permission level
    pub fn highest_permission(&self) -> TenantPermission {
        if self.permissions.contains(&TenantPermission::Admin) {
            TenantPermission::Admin
        } else if self.permissions.contains(&TenantPermission::ReadWrite) {
            TenantPermission::ReadWrite
        } else if self.permissions.contains(&TenantPermission::Mcp) {
            TenantPermission::Mcp
        } else {
            TenantPermission::ReadOnly
        }
    }
}

/// Rate limit configuration for a tenant
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TenantRateLimits {
    /// Requests per minute
    pub requests_per_minute: Option<u32>,
    /// Requests per hour
    pub requests_per_hour: Option<u32>,
    /// Requests per day
    pub requests_per_day: Option<u32>,
}
