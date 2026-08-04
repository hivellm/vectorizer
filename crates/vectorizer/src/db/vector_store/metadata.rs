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

    /// Set (or clear, with `None`) the collection-level TTL in seconds.
    ///
    /// The TTL is the rule "vectors inserted into this collection expire
    /// `ttl_secs` seconds after they arrive". `VectorStore::insert` and
    /// `VectorStore::update` read it and stamp `__expires_at` on the
    /// vector payload, which is what the [`TtlReaper`] then sweeps.
    ///
    /// The rule is durable: it is written to the `.vecdb` archive as
    /// `PersistedCollection::ttl_secs` and restored on load, so it survives
    /// a restart. It is keyed by the collection's canonical name, so an
    /// alias resolves to the same rule as its target.
    ///
    /// [`TtlReaper`]: crate::db::TtlReaper
    pub fn set_collection_ttl(&self, collection: &str, ttl_secs: Option<u64>) {
        let key = self.collection_ttl_key(collection);
        match ttl_secs {
            Some(secs) => {
                self.metadata.insert(key, secs.to_string());
            }
            None => {
                self.metadata.remove(&key);
            }
        }
    }

    /// Collection-level TTL in seconds, or `None` when no TTL is configured.
    ///
    /// A value that does not parse as a `u64` is treated as absent rather
    /// than as an error: the metadata map is free-form and a corrupt entry
    /// must not make every insert fail.
    pub fn collection_ttl(&self, collection: &str) -> Option<u64> {
        self.metadata
            .get(&self.collection_ttl_key(collection))
            .and_then(|entry| entry.value().parse::<u64>().ok())
    }

    /// Every configured collection TTL as `(collection, ttl_secs)` pairs.
    pub fn collection_ttls(&self) -> Vec<(String, u64)> {
        self.metadata
            .iter()
            .filter_map(|entry| {
                let collection = entry.key().strip_prefix(COLLECTION_TTL_PREFIX)?;
                let secs = entry.value().parse::<u64>().ok()?;
                Some((collection.to_string(), secs))
            })
            .collect()
    }

    /// The metadata key holding `collection`'s TTL, resolved through the
    /// alias table so a write or an insert addressed to an alias — including
    /// the grace-window alias `rename_collection` leaves behind — sees the
    /// target's rule rather than a key nobody reads.
    ///
    /// Falls back to the raw name when resolution fails (an alias loop),
    /// because a corrupt alias table must not make every insert error.
    fn collection_ttl_key(&self, collection: &str) -> String {
        let canonical = self
            .resolve_alias_target(collection)
            .unwrap_or_else(|_| collection.to_string());
        format!("{}{}", COLLECTION_TTL_PREFIX, canonical)
    }
}

/// Metadata-key prefix under which collection-level TTLs are stored
/// (`ttl:<collection>` → seconds, as a decimal string).
pub const COLLECTION_TTL_PREFIX: &str = "ttl:";

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;

    #[test]
    fn collection_ttl_round_trips() {
        let store = VectorStore::new();
        assert_eq!(store.collection_ttl("docs"), None);

        store.set_collection_ttl("docs", Some(3600));
        assert_eq!(store.collection_ttl("docs"), Some(3600));
        assert_eq!(
            store.get_metadata("ttl:docs"),
            Some("3600".to_string()),
            "the on-the-wire key shape is part of the contract"
        );

        store.set_collection_ttl("docs", None);
        assert_eq!(store.collection_ttl("docs"), None);
        assert_eq!(store.get_metadata("ttl:docs"), None);
    }

    #[test]
    fn collection_ttl_is_per_collection() {
        let store = VectorStore::new();
        store.set_collection_ttl("a", Some(60));
        store.set_collection_ttl("b", Some(120));

        assert_eq!(store.collection_ttl("a"), Some(60));
        assert_eq!(store.collection_ttl("b"), Some(120));
        assert_eq!(store.collection_ttl("c"), None);

        let mut ttls = store.collection_ttls();
        ttls.sort();
        assert_eq!(ttls, vec![("a".to_string(), 60), ("b".to_string(), 120)]);
    }

    #[test]
    fn unparsable_ttl_entry_reads_as_absent() {
        let store = VectorStore::new();
        store.set_metadata("ttl:docs", "not-a-number".to_string());

        assert_eq!(store.collection_ttl("docs"), None);
        assert!(store.collection_ttls().is_empty());
    }

    #[test]
    fn collection_ttls_ignores_unrelated_metadata() {
        let store = VectorStore::new();
        store.set_metadata("replication_role", "Master".to_string());
        store.set_collection_ttl("docs", Some(30));

        assert_eq!(
            store.collection_ttls(),
            vec![("docs".to_string(), 30)],
            "only ttl:* keys are TTL configuration"
        );
    }

    #[test]
    fn an_alias_reads_and_writes_its_targets_ttl() {
        let store = VectorStore::new();
        store
            .create_collection("docs", collection_config())
            .expect("create");
        store.create_alias("docs_v1", "docs").expect("alias");

        store.set_collection_ttl("docs", Some(60));
        assert_eq!(
            store.collection_ttl("docs_v1"),
            Some(60),
            "an insert addressed to the alias must see the target's rule"
        );

        // Writing through the alias must not mint a second, unread key.
        store.set_collection_ttl("docs_v1", Some(120));
        assert_eq!(store.collection_ttl("docs"), Some(120));
        assert_eq!(
            store.collection_ttls(),
            vec![("docs".to_string(), 120)],
            "the rule is keyed canonically"
        );
    }

    fn collection_config() -> crate::models::CollectionConfig {
        crate::models::CollectionConfig {
            dimension: 4,
            metric: crate::models::DistanceMetric::Cosine,
            ..Default::default()
        }
    }
}
