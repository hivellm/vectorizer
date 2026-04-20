//! Alias management — create, rename, delete, list, and resolve
//! alias chains against the primary collection table.
//!
//! Aliases are stored in a `DashMap<String, String>` where the value
//! is the canonical collection name (or another alias, resolved
//! recursively with loop detection). Every lookup path in
//! `VectorStore` (search, insert, metadata) eventually calls
//! [`VectorStore::resolve_alias_target`] before touching
//! `collections`.

use std::collections::HashSet;

use tracing::info;

use super::VectorStore;
use crate::error::{Result, VectorizerError};

impl VectorStore {
    /// Resolve alias chain to a canonical collection name
    pub(super) fn resolve_alias_target(&self, name: &str) -> Result<String> {
        let mut current = name.to_string();
        let mut visited = HashSet::new();

        loop {
            if !visited.insert(current.clone()) {
                return Err(VectorizerError::ConfigurationError(format!(
                    "Alias resolution loop detected for '{}'; visited: {:?}",
                    name, visited
                )));
            }

            match self.aliases.get(&current) {
                Some(target) => {
                    current = target.clone();
                }
                None => break,
            }
        }

        Ok(current)
    }

    /// Remove all aliases pointing to the specified collection
    pub(super) fn remove_aliases_for_collection(&self, collection_name: &str) {
        let canonical = collection_name.to_string();
        self.aliases
            .retain(|_, target| target.as_str() != canonical.as_str());
    }

    /// List all aliases as `(alias, target)` pairs
    pub fn list_aliases(&self) -> Vec<(String, String)> {
        self.aliases
            .iter()
            .map(|entry| (entry.key().clone(), entry.value().clone()))
            .collect()
    }

    /// List aliases pointing to the given collection (accepts canonical name or alias)
    pub fn list_aliases_for_collection(&self, name: &str) -> Result<Vec<String>> {
        let canonical = self.resolve_alias_target(name)?;
        let aliases: Vec<String> = self
            .aliases
            .iter()
            .filter_map(|entry| {
                if entry.value().as_str() == canonical {
                    Some(entry.key().clone())
                } else {
                    None
                }
            })
            .collect();
        Ok(aliases)
    }

    /// Create a new alias pointing to an existing collection
    pub fn create_alias(&self, alias: &str, target: &str) -> Result<()> {
        let alias = alias.trim();
        let target = target.trim();

        if alias.is_empty() {
            return Err(VectorizerError::InvalidConfiguration {
                message: "Alias name cannot be empty".to_string(),
            });
        }

        if target.is_empty() {
            return Err(VectorizerError::InvalidConfiguration {
                message: "Collection name cannot be empty".to_string(),
            });
        }

        if alias == target {
            return Err(VectorizerError::InvalidConfiguration {
                message: "Alias name must differ from collection name".to_string(),
            });
        }

        if self.collections.contains_key(alias) {
            return Err(VectorizerError::CollectionAlreadyExists(alias.to_string()));
        }

        if self.aliases.contains_key(alias) {
            return Err(VectorizerError::CollectionAlreadyExists(alias.to_string()));
        }

        let canonical_target = self.resolve_alias_target(target)?;

        // Ensure target exists (will lazy-load if needed)
        self.get_collection(canonical_target.as_str())?;

        self.aliases
            .insert(alias.to_string(), canonical_target.clone());

        info!(
            "Alias '{}' created for collection '{}' (requested target '{}')",
            alias, canonical_target, target
        );

        Ok(())
    }

    /// Delete an alias by name
    pub fn delete_alias(&self, alias: &str) -> Result<()> {
        if self.aliases.remove(alias).is_some() {
            info!("Alias '{}' deleted", alias);
            Ok(())
        } else {
            Err(VectorizerError::NotFound(format!(
                "Alias '{}' not found",
                alias
            )))
        }
    }

    /// Rename an existing alias
    pub fn rename_alias(&self, old_alias: &str, new_alias: &str) -> Result<()> {
        let new_alias = new_alias.trim();

        if new_alias.is_empty() {
            return Err(VectorizerError::InvalidConfiguration {
                message: "Alias name cannot be empty".to_string(),
            });
        }

        if old_alias == new_alias {
            return Ok(());
        }

        let alias_entry = self
            .aliases
            .remove(old_alias)
            .ok_or_else(|| VectorizerError::NotFound(format!("Alias '{}' not found", old_alias)))?;

        let target_name = alias_entry.1;

        if self.collections.contains_key(new_alias) || self.aliases.contains_key(new_alias) {
            // Re-insert the old alias before returning error
            self.aliases.insert(old_alias.to_string(), target_name);
            return Err(VectorizerError::CollectionAlreadyExists(
                new_alias.to_string(),
            ));
        }

        self.aliases
            .insert(new_alias.to_string(), target_name.clone());
        info!(
            "Alias '{}' renamed to '{}' for collection '{}'",
            old_alias, new_alias, target_name
        );
        Ok(())
    }
}
