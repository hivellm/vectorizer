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
}
