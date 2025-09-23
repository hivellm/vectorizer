//! Role-based access control (RBAC) system
//! 
//! Defines roles, permissions, and access control for the vector database

use serde::{Deserialize, Serialize};
use std::collections::HashSet;

/// User roles in the system
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Role {
    /// System administrator with full access
    Admin,
    /// Regular user with standard access
    User,
    /// API user with limited access
    ApiUser,
    /// Read-only user
    ReadOnly,
    /// Service account for automated systems
    Service,
}

impl Role {
    /// Get all permissions for a role
    pub fn permissions(&self) -> Vec<Permission> {
        match self {
            Role::Admin => vec![
                Permission::Read,
                Permission::Write,
                Permission::Delete,
                Permission::CreateCollection,
                Permission::DeleteCollection,
                Permission::ManageUsers,
                Permission::ManageApiKeys,
                Permission::ViewLogs,
                Permission::SystemConfig,
            ],
            Role::User => vec![
                Permission::Read,
                Permission::Write,
                Permission::CreateCollection,
            ],
            Role::ApiUser => vec![
                Permission::Read,
                Permission::Write,
            ],
            Role::ReadOnly => vec![
                Permission::Read,
            ],
            Role::Service => vec![
                Permission::Read,
                Permission::Write,
                Permission::CreateCollection,
            ],
        }
    }

    /// Check if a role has a specific permission
    pub fn has_permission(&self, permission: &Permission) -> bool {
        self.permissions().contains(permission)
    }

    /// Get role hierarchy level (higher number = more privileges)
    pub fn hierarchy_level(&self) -> u8 {
        match self {
            Role::Admin => 5,
            Role::User => 3,
            Role::Service => 3,
            Role::ApiUser => 2,
            Role::ReadOnly => 1,
        }
    }

    /// Check if this role can manage another role
    pub fn can_manage_role(&self, other_role: &Role) -> bool {
        self.hierarchy_level() > other_role.hierarchy_level()
    }
}

/// System permissions
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Permission {
    /// Read vectors and collections
    Read,
    /// Write/update vectors
    Write,
    /// Delete vectors
    Delete,
    /// Create new collections
    CreateCollection,
    /// Delete collections
    DeleteCollection,
    /// Manage user accounts
    ManageUsers,
    /// Manage API keys
    ManageApiKeys,
    /// View system logs
    ViewLogs,
    /// Modify system configuration
    SystemConfig,
}

impl Permission {
    /// Get a human-readable description of the permission
    pub fn description(&self) -> &'static str {
        match self {
            Permission::Read => "Read vectors and collections",
            Permission::Write => "Write/update vectors",
            Permission::Delete => "Delete vectors",
            Permission::CreateCollection => "Create new collections",
            Permission::DeleteCollection => "Delete collections",
            Permission::ManageUsers => "Manage user accounts",
            Permission::ManageApiKeys => "Manage API keys",
            Permission::ViewLogs => "View system logs",
            Permission::SystemConfig => "Modify system configuration",
        }
    }

    /// Get the resource type this permission applies to
    pub fn resource_type(&self) -> &'static str {
        match self {
            Permission::Read | Permission::Write | Permission::Delete => "vector",
            Permission::CreateCollection | Permission::DeleteCollection => "collection",
            Permission::ManageUsers => "user",
            Permission::ManageApiKeys => "api_key",
            Permission::ViewLogs | Permission::SystemConfig => "system",
        }
    }
}

/// Access control context for checking permissions
#[derive(Debug, Clone)]
pub struct AccessContext {
    /// User ID
    pub user_id: String,
    /// User roles
    pub roles: Vec<Role>,
    /// Collection being accessed (if applicable)
    pub collection: Option<String>,
    /// Resource being accessed (if applicable)
    pub resource: Option<String>,
}

impl AccessContext {
    /// Create a new access context
    pub fn new(user_id: String, roles: Vec<Role>) -> Self {
        Self {
            user_id,
            roles,
            collection: None,
            resource: None,
        }
    }

    /// Set the collection context
    pub fn with_collection(mut self, collection: String) -> Self {
        self.collection = Some(collection);
        self
    }

    /// Set the resource context
    pub fn with_resource(mut self, resource: String) -> Self {
        self.resource = Some(resource);
        self
    }

    /// Check if the user has a specific permission
    pub fn has_permission(&self, permission: &Permission) -> bool {
        self.roles.iter().any(|role| role.has_permission(permission))
    }

    /// Check if the user has any of the specified permissions
    pub fn has_any_permission(&self, permissions: &[Permission]) -> bool {
        permissions.iter().any(|perm| self.has_permission(perm))
    }

    /// Check if the user has all of the specified permissions
    pub fn has_all_permissions(&self, permissions: &[Permission]) -> bool {
        permissions.iter().all(|perm| self.has_permission(perm))
    }

    /// Get all permissions the user has
    pub fn all_permissions(&self) -> HashSet<Permission> {
        let mut permissions = HashSet::new();
        for role in &self.roles {
            for permission in role.permissions() {
                permissions.insert(permission);
            }
        }
        permissions
    }

    /// Check if the user is an admin
    pub fn is_admin(&self) -> bool {
        self.roles.contains(&Role::Admin)
    }

    /// Check if the user can perform admin operations
    pub fn can_admin(&self) -> bool {
        self.has_permission(&Permission::ManageUsers) || self.is_admin()
    }
}

/// Permission checker utility
#[derive(Debug)]
pub struct PermissionChecker;

impl PermissionChecker {
    /// Check if a user can access a collection
    pub fn can_access_collection(context: &AccessContext, _collection_name: &str) -> bool {
        context.has_permission(&Permission::Read)
    }

    /// Check if a user can create a collection
    pub fn can_create_collection(context: &AccessContext) -> bool {
        context.has_permission(&Permission::CreateCollection)
    }

    /// Check if a user can delete a collection
    pub fn can_delete_collection(context: &AccessContext) -> bool {
        context.has_permission(&Permission::DeleteCollection)
    }

    /// Check if a user can read vectors
    pub fn can_read_vectors(context: &AccessContext) -> bool {
        context.has_permission(&Permission::Read)
    }

    /// Check if a user can write vectors
    pub fn can_write_vectors(context: &AccessContext) -> bool {
        context.has_permission(&Permission::Write)
    }

    /// Check if a user can delete vectors
    pub fn can_delete_vectors(context: &AccessContext) -> bool {
        context.has_permission(&Permission::Delete)
    }

    /// Check if a user can manage API keys
    pub fn can_manage_api_keys(context: &AccessContext) -> bool {
        context.has_permission(&Permission::ManageApiKeys)
    }

    /// Check if a user can view logs
    pub fn can_view_logs(context: &AccessContext) -> bool {
        context.has_permission(&Permission::ViewLogs)
    }

    /// Check if a user can modify system configuration
    pub fn can_modify_config(context: &AccessContext) -> bool {
        context.has_permission(&Permission::SystemConfig)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_role_permissions() {
        let admin_permissions = Role::Admin.permissions();
        assert!(admin_permissions.contains(&Permission::Read));
        assert!(admin_permissions.contains(&Permission::Write));
        assert!(admin_permissions.contains(&Permission::ManageUsers));
        assert!(admin_permissions.contains(&Permission::SystemConfig));

        let user_permissions = Role::User.permissions();
        assert!(user_permissions.contains(&Permission::Read));
        assert!(user_permissions.contains(&Permission::Write));
        assert!(!user_permissions.contains(&Permission::ManageUsers));

        let readonly_permissions = Role::ReadOnly.permissions();
        assert!(readonly_permissions.contains(&Permission::Read));
        assert!(!readonly_permissions.contains(&Permission::Write));
    }

    #[test]
    fn test_role_hierarchy() {
        assert!(Role::Admin.can_manage_role(&Role::User));
        assert!(Role::Admin.can_manage_role(&Role::ReadOnly));
        assert!(Role::User.can_manage_role(&Role::ReadOnly));
        assert!(!Role::User.can_manage_role(&Role::Admin));
        assert!(!Role::ReadOnly.can_manage_role(&Role::User));
    }

    #[test]
    fn test_permission_descriptions() {
        assert_eq!(Permission::Read.description(), "Read vectors and collections");
        assert_eq!(Permission::Write.description(), "Write/update vectors");
        assert_eq!(Permission::ManageUsers.description(), "Manage user accounts");
    }

    #[test]
    fn test_permission_resource_types() {
        assert_eq!(Permission::Read.resource_type(), "vector");
        assert_eq!(Permission::CreateCollection.resource_type(), "collection");
        assert_eq!(Permission::ManageUsers.resource_type(), "user");
        assert_eq!(Permission::SystemConfig.resource_type(), "system");
    }

    #[test]
    fn test_access_context() {
        let context = AccessContext::new("user123".to_string(), vec![Role::User]);
        
        assert!(context.has_permission(&Permission::Read));
        assert!(context.has_permission(&Permission::Write));
        assert!(!context.has_permission(&Permission::ManageUsers));
        assert!(!context.is_admin());
        assert!(!context.can_admin());
    }

    #[test]
    fn test_access_context_with_collection() {
        let context = AccessContext::new("user123".to_string(), vec![Role::User])
            .with_collection("test_collection".to_string());
        
        assert_eq!(context.collection, Some("test_collection".to_string()));
    }

    #[test]
    fn test_permission_checker() {
        let admin_context = AccessContext::new("admin".to_string(), vec![Role::Admin]);
        let user_context = AccessContext::new("user".to_string(), vec![Role::User]);
        let readonly_context = AccessContext::new("readonly".to_string(), vec![Role::ReadOnly]);

        // Admin can do everything
        assert!(PermissionChecker::can_create_collection(&admin_context));
        assert!(PermissionChecker::can_delete_collection(&admin_context));
        assert!(PermissionChecker::can_manage_api_keys(&admin_context));

        // User can read/write but not delete collections
        assert!(PermissionChecker::can_read_vectors(&user_context));
        assert!(PermissionChecker::can_write_vectors(&user_context));
        assert!(PermissionChecker::can_create_collection(&user_context));
        assert!(!PermissionChecker::can_delete_collection(&user_context));
        assert!(!PermissionChecker::can_manage_api_keys(&user_context));

        // ReadOnly can only read
        assert!(PermissionChecker::can_read_vectors(&readonly_context));
        assert!(!PermissionChecker::can_write_vectors(&readonly_context));
        assert!(!PermissionChecker::can_create_collection(&readonly_context));
    }

    #[test]
    fn test_all_permissions() {
        let context = AccessContext::new("user123".to_string(), vec![Role::User, Role::ReadOnly]);
        let permissions = context.all_permissions();
        
        assert!(permissions.contains(&Permission::Read));
        assert!(permissions.contains(&Permission::Write));
        assert!(permissions.contains(&Permission::CreateCollection));
        assert!(!permissions.contains(&Permission::ManageUsers));
    }
}
