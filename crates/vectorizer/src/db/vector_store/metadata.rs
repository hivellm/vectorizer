//! Metadata + stats accessors on `VectorStore`.
//!
//! - [`VectorStore::stats`] is a point-in-time snapshot of collection
//!   count + total vectors + estimated memory.
//! - The `metadata` family is a free-form `DashMap<String, String>`
//!   used by replication / cluster features to stash crate-global config
//!   (e.g. the active replication role) without dragging a dedicated
//!   struct through every call site.

// Internal data-layout file: public fields are self-documenting; the
// blanket allow keeps `cargo doc -W missing-docs` clean without padding
// every field with a tautological `///` comment. See
// phase4_enforce-public-api-docs.
#![allow(missing_docs)]

use super::VectorStore;
use crate::error::Result;
use crate::models::CollectionMetadata;

/// Statistics about the vector store
pub struct VectorStoreStats {
    pub collection_count: usize,
    pub total_vectors: usize,
    pub total_memory_bytes: usize,
}

impl VectorStore {
    /// Get collection metadata
    pub fn get_collection_metadata(&self, name: &str) -> Result<CollectionMetadata> {
        let collection_ref = self.get_collection(name)?;
        Ok(collection_ref.metadata())
    }

    /// Get statistics about the vector store
    pub fn stats(&self) -> VectorStoreStats {
        let mut total_vectors = 0;
        let mut total_memory_bytes = 0;

        for entry in self.collections.iter() {
            let collection = entry.value();
            total_vectors += collection.vector_count();
            total_memory_bytes += collection.estimated_memory_usage();
        }

        VectorStoreStats {
            collection_count: self.collections.len(),
            total_vectors,
            total_memory_bytes,
        }
    }

    /// Get metadata value by key
    pub fn get_metadata(&self, key: &str) -> Option<String> {
        self.metadata.get(key).map(|v| v.value().clone())
    }

    /// Set metadata value
    pub fn set_metadata(&self, key: &str, value: String) {
        self.metadata.insert(key.to_string(), value);
    }

    /// Remove metadata value
    pub fn remove_metadata(&self, key: &str) -> Option<String> {
        self.metadata.remove(key).map(|(_, v)| v)
    }

    /// List all metadata keys
    pub fn list_metadata_keys(&self) -> Vec<String> {
        self.metadata
            .iter()
            .map(|entry| entry.key().clone())
            .collect()
    }
}
