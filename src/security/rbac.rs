//! Role-Based Access Control (RBAC)
//!
//! This module implements fine-grained permission system for the vectorizer.

use std::collections::HashSet;

use serde::{Deserialize, Serialize};

/// Permission enum - defines all possible actions
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Permission {
    // Collection permissions
    CreateCollection,
    ReadCollection,
    UpdateCollection,
    DeleteCollection,
    ListCollections,

    // Vector permissions
    InsertVector,
    SearchVector,
    UpdateVector,
    DeleteVector,
    GetVector,

    // Batch operations
    BatchInsert,
    BatchSearch,
    BatchUpdate,
    BatchDelete,

    // Admin permissions
    ManageAPIKeys,
    ViewAuditLogs,
    ConfigureServer,
    ManageReplication,
    ViewMetrics,

    // System permissions
    ViewSystemStats,
    RestartServer,
    BackupData,
    RestoreData,
}

/// Role - defines a set of permissions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Role {
    pub name: String,
    pub permissions: HashSet<Permission>,
}

impl Role {
    /// Create a new role
    pub fn new(name: impl Into<String>, permissions: Vec<Permission>) -> Self {
        Self {
            name: name.into(),
            permissions: permissions.into_iter().collect(),
        }
    }

    /// Check if role has a specific permission
    pub fn has_permission(&self, permission: Permission) -> bool {
        self.permissions.contains(&permission)
    }

    /// Add a permission to the role
    pub fn add_permission(&mut self, permission: Permission) {
        self.permissions.insert(permission);
    }

    /// Remove a permission from the role
    pub fn remove_permission(&mut self, permission: Permission) {
        self.permissions.remove(&permission);
    }

    // ═══════════════════════════════════════════════════════════════════════
    // Predefined Roles
    // ═══════════════════════════════════════════════════════════════════════

    /// Viewer role - read-only access
    pub fn viewer() -> Self {
        Self::new(
            "Viewer",
            vec![
                Permission::ReadCollection,
                Permission::ListCollections,
                Permission::SearchVector,
                Permission::GetVector,
                Permission::ViewSystemStats,
            ],
        )
    }

    /// Editor role - read/write access (no admin)
    pub fn editor() -> Self {
        let mut role = Self::viewer();
        role.name = "Editor".to_string();
        role.add_permission(Permission::CreateCollection);
        role.add_permission(Permission::UpdateCollection);
        role.add_permission(Permission::InsertVector);
        role.add_permission(Permission::UpdateVector);
        role.add_permission(Permission::DeleteVector);
        role.add_permission(Permission::BatchInsert);
        role.add_permission(Permission::BatchSearch);
        role.add_permission(Permission::BatchUpdate);
        role.add_permission(Permission::BatchDelete);
        role
    }

    /// Admin role - full access
    pub fn admin() -> Self {
        let mut role = Self::editor();
        role.name = "Admin".to_string();
        role.add_permission(Permission::DeleteCollection);
        role.add_permission(Permission::ManageAPIKeys);
        role.add_permission(Permission::ViewAuditLogs);
        role.add_permission(Permission::ConfigureServer);
        role.add_permission(Permission::ManageReplication);
        role.add_permission(Permission::ViewMetrics);
        role.add_permission(Permission::RestartServer);
        role.add_permission(Permission::BackupData);
        role.add_permission(Permission::RestoreData);
        role
    }
}

/// User - represents a user with roles and permissions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: String,
    pub roles: Vec<Role>,
}

impl User {
    /// Create a new user
    pub fn new(id: impl Into<String>, roles: Vec<Role>) -> Self {
        Self {
            id: id.into(),
            roles,
        }
    }

    /// Check if user has a specific permission
    pub fn has_permission(&self, permission: Permission) -> bool {
        self.roles
            .iter()
            .any(|role| role.has_permission(permission))
    }

    /// Add a role to the user
    pub fn add_role(&mut self, role: Role) {
        self.roles.push(role);
    }

    /// Remove a role from the user
    pub fn remove_role(&mut self, role_name: &str) {
        self.roles.retain(|role| role.name != role_name);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_role_creation() {
        let role = Role::new("CustomRole", vec![Permission::ReadCollection]);
        assert_eq!(role.name, "CustomRole");
        assert_eq!(role.permissions.len(), 1);
        assert!(role.has_permission(Permission::ReadCollection));
    }

    #[test]
    fn test_viewer_role() {
        let viewer = Role::viewer();
        assert_eq!(viewer.name, "Viewer");
        assert!(viewer.has_permission(Permission::ReadCollection));
        assert!(viewer.has_permission(Permission::SearchVector));
        assert!(!viewer.has_permission(Permission::InsertVector));
        assert!(!viewer.has_permission(Permission::DeleteCollection));
    }

    #[test]
    fn test_editor_role() {
        let editor = Role::editor();
        assert_eq!(editor.name, "Editor");
        assert!(editor.has_permission(Permission::ReadCollection));
        assert!(editor.has_permission(Permission::InsertVector));
        assert!(editor.has_permission(Permission::CreateCollection));
        assert!(!editor.has_permission(Permission::DeleteCollection));
        assert!(!editor.has_permission(Permission::ManageAPIKeys));
    }

    #[test]
    fn test_admin_role() {
        let admin = Role::admin();
        assert_eq!(admin.name, "Admin");
        assert!(admin.has_permission(Permission::ReadCollection));
        assert!(admin.has_permission(Permission::InsertVector));
        assert!(admin.has_permission(Permission::DeleteCollection));
        assert!(admin.has_permission(Permission::ManageAPIKeys));
        assert!(admin.has_permission(Permission::ViewAuditLogs));
    }

    #[test]
    fn test_add_remove_permission() {
        let mut role = Role::viewer();
        assert!(!role.has_permission(Permission::InsertVector));

        role.add_permission(Permission::InsertVector);
        assert!(role.has_permission(Permission::InsertVector));

        role.remove_permission(Permission::InsertVector);
        assert!(!role.has_permission(Permission::InsertVector));
    }

    #[test]
    fn test_role_hierarchy() {
        let viewer = Role::viewer();
        let editor = Role::editor();
        let admin = Role::admin();

        // Viewer < Editor < Admin (in terms of permissions)
        assert!(viewer.permissions.len() < editor.permissions.len());
        assert!(editor.permissions.len() < admin.permissions.len());

        // All viewer permissions are in editor
        for perm in &viewer.permissions {
            assert!(editor.has_permission(*perm));
        }

        // All editor permissions are in admin
        for perm in &editor.permissions {
            assert!(admin.has_permission(*perm));
        }
    }

    #[test]
    fn test_permission_equality() {
        let perm1 = Permission::CreateCollection;
        let perm2 = Permission::CreateCollection;
        let perm3 = Permission::ReadCollection;

        assert_eq!(perm1, perm2);
        assert_ne!(perm1, perm3);
    }

    #[test]
    fn test_permission_hash() {
        use std::collections::HashSet;

        let mut permissions = HashSet::new();
        permissions.insert(Permission::CreateCollection);
        permissions.insert(Permission::ReadCollection);
        permissions.insert(Permission::CreateCollection); // Duplicate

        assert_eq!(permissions.len(), 2); // Duplicate should be ignored
    }

    #[test]
    fn test_role_creation_detailed() {
        let mut permissions = HashSet::new();
        permissions.insert(Permission::CreateCollection);
        permissions.insert(Permission::ReadCollection);

        let role = Role {
            name: "test_role".to_string(),
            permissions,
        };

        assert_eq!(role.name, "test_role");
        assert!(role.has_permission(Permission::CreateCollection));
        assert!(role.has_permission(Permission::ReadCollection));
        assert!(!role.has_permission(Permission::DeleteCollection));
    }

    #[test]
    fn test_role_serialization() {
        let mut permissions = HashSet::new();
        permissions.insert(Permission::CreateCollection);
        permissions.insert(Permission::ReadCollection);

        let role = Role {
            name: "test_role".to_string(),
            permissions,
        };

        let serialized = serde_json::to_string(&role).unwrap();
        let deserialized: Role = serde_json::from_str(&serialized).unwrap();

        assert_eq!(role.name, deserialized.name);
        assert_eq!(role.permissions, deserialized.permissions);
    }

    #[test]
    fn test_user_creation() {
        let mut permissions = HashSet::new();
        permissions.insert(Permission::CreateCollection);
        permissions.insert(Permission::ReadCollection);

        let role = Role {
            name: "test_role".to_string(),
            permissions,
        };

        let user = User {
            id: "user1".to_string(),
            roles: vec![role],
        };

        assert_eq!(user.id, "user1");
        assert_eq!(user.roles.len(), 1);
        assert!(user.has_permission(Permission::CreateCollection));
        assert!(user.has_permission(Permission::ReadCollection));
        assert!(!user.has_permission(Permission::DeleteCollection));
    }

    #[test]
    fn test_user_multiple_roles() {
        let mut permissions1 = HashSet::new();
        permissions1.insert(Permission::CreateCollection);
        permissions1.insert(Permission::ReadCollection);

        let mut permissions2 = HashSet::new();
        permissions2.insert(Permission::DeleteCollection);
        permissions2.insert(Permission::UpdateCollection);

        let role1 = Role {
            name: "role1".to_string(),
            permissions: permissions1,
        };

        let role2 = Role {
            name: "role2".to_string(),
            permissions: permissions2,
        };

        let user = User {
            id: "user1".to_string(),
            roles: vec![role1, role2],
        };

        assert_eq!(user.id, "user1");
        assert_eq!(user.roles.len(), 2);
        assert!(user.has_permission(Permission::CreateCollection));
        assert!(user.has_permission(Permission::ReadCollection));
        assert!(user.has_permission(Permission::DeleteCollection));
        assert!(user.has_permission(Permission::UpdateCollection));
    }

    #[test]
    fn test_user_serialization() {
        let mut permissions = HashSet::new();
        permissions.insert(Permission::CreateCollection);
        permissions.insert(Permission::ReadCollection);

        let role = Role {
            name: "test_role".to_string(),
            permissions,
        };

        let user = User {
            id: "user1".to_string(),
            roles: vec![role],
        };

        let serialized = serde_json::to_string(&user).unwrap();
        let deserialized: User = serde_json::from_str(&serialized).unwrap();

        assert_eq!(user.id, deserialized.id);
        assert_eq!(user.roles.len(), deserialized.roles.len());
    }
}
