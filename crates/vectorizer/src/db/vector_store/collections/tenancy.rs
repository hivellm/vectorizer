//! Ownership / multi-tenancy queries over the collection map.
//!
//! Builds on [`super::disk_load::VectorStore::get_collection`] and
//! [`super::lifecycle::VectorStore::delete_collection`]; owns no state
//! of its own.

use tracing::{debug, info, warn};

use super::super::{CollectionType, VectorStore};
use crate::error::{Result, VectorizerError};

impl VectorStore {
    /// List collections owned by a specific user (for multi-tenancy)
    pub fn list_collections_for_owner(&self, owner_id: &uuid::Uuid) -> Vec<String> {
        self.collections
            .iter()
            .filter(|entry| entry.value().belongs_to(owner_id))
            .map(|entry| entry.key().clone())
            .collect()
    }

    /// Delete all collections owned by a specific tenant (for tenant cleanup on deletion)
    ///
    /// Returns the number of collections deleted.
    pub fn cleanup_tenant_data(&self, owner_id: &uuid::Uuid) -> Result<usize> {
        let collections_to_delete = self.list_collections_for_owner(owner_id);
        let count = collections_to_delete.len();

        for collection_name in collections_to_delete {
            if let Err(e) = self.delete_collection(&collection_name) {
                warn!(
                    "Failed to delete collection '{}' for tenant {}: {}",
                    collection_name, owner_id, e
                );
                // Continue deleting other collections even if one fails
            } else {
                info!(
                    "Deleted collection '{}' for tenant {} during cleanup",
                    collection_name, owner_id
                );
            }
        }

        info!(
            "Tenant cleanup complete: deleted {} collections for owner {}",
            count, owner_id
        );
        Ok(count)
    }

    /// Get collection metadata for a specific owner (returns None if not owned by that user)
    pub fn get_collection_for_owner(
        &self,
        name: &str,
        owner_id: &uuid::Uuid,
    ) -> Option<crate::models::CollectionMetadata> {
        // Previously: `.ok()?` silently conflated "alias does not exist" with
        // "alias table corrupted / lock poisoned". Caller still gets None for
        // both, but the log now distinguishes them so operational issues are
        // visible.
        let canonical = match self.resolve_alias_target(name) {
            Ok(c) => c,
            Err(e) => {
                debug!(
                    "get_collection_for_owner({}): alias resolution failed: {}",
                    name, e
                );
                return None;
            }
        };
        self.collections.get(&canonical).and_then(|collection| {
            if collection.belongs_to(owner_id) {
                Some(collection.metadata())
            } else {
                None
            }
        })
    }

    /// Check if a collection is owned by the given user
    pub fn is_collection_owned_by(&self, name: &str, owner_id: &uuid::Uuid) -> bool {
        let canonical = match self.resolve_alias_target(name) {
            Ok(name) => name,
            Err(_) => return false,
        };
        self.collections
            .get(&canonical)
            .map(|c| c.belongs_to(owner_id))
            .unwrap_or(false)
    }

    /// Get a reference to a collection by name, with ownership validation
    ///
    /// Returns the collection only if:
    /// 1. The collection exists
    /// 2. Either the collection has no owner, or the owner matches the given owner_id
    pub fn get_collection_with_owner(
        &self,
        name: &str,
        owner_id: Option<&uuid::Uuid>,
    ) -> Result<impl std::ops::Deref<Target = CollectionType> + '_> {
        // First get the collection normally
        let collection = self.get_collection(name)?;

        // If no owner_id is provided, allow access (non-tenant mode)
        let Some(owner) = owner_id else {
            return Ok(collection);
        };

        // Check ownership - allow access if collection has no owner or matches
        if collection.owner_id().is_none() || collection.belongs_to(owner) {
            Ok(collection)
        } else {
            Err(VectorizerError::CollectionNotFound(name.to_string()))
        }
    }
}
