//! Collection lookup with lazy disk-loading.
//!
//! `get_collection` / `get_collection_mut` are the workhorses: they
//! resolve alias chains, try the in-memory `DashMap` first, and fall
//! back to `.vecdb` (compact) and legacy `.bin` (raw) on disk.
//! [`super::lifecycle`] and [`super::tenancy`] both build on top of
//! these two accessors.

use tracing::{debug, info, warn};

use super::super::{CollectionType, VectorStore};
use crate::error::{Result, VectorizerError};

impl VectorStore {
    /// Get a reference to a collection by name.
    ///
    /// Lazy-loads from `.vecdb` (compact) or `.bin` (legacy) on disk
    /// if the collection isn't already in memory.
    ///
    /// # Deadlock invariant
    ///
    /// The returned guard is a [`dashmap::mapref::one::Ref`] borrowed from
    /// the same DashMap shard that [`Self::get_collection_mut`] locks
    /// exclusively. **Never call `get_collection_mut` (or any method that
    /// calls it, e.g. `VectorStore::update`/`VectorStore::delete`'s
    /// GPU-only fallback) while still holding a `Ref` returned from this
    /// method on the same collection** — DashMap's shard lock is not
    /// reentrant, so the second call blocks forever waiting on a lock the
    /// current thread already holds. This caused a production deadlock in
    /// `bulk_update_metadata` (fixed in phase39; see
    /// `rest_handlers/vectors.rs`). Drop the `Ref` (end its scope) before
    /// requesting a mutable reference to the same collection. Prefer
    /// matching on `&*collection_ref` and calling the variant's own
    /// `&self` method (as `delete`/`update` below do for `Cpu`/`Sharded`)
    /// over reaching for `get_collection_mut` at all.
    pub fn get_collection(
        &self,
        name: &str,
    ) -> Result<impl std::ops::Deref<Target = CollectionType> + '_> {
        let canonical = self.resolve_alias_target(name)?;
        let canonical_ref = canonical.as_str();

        // Fast path: collection already loaded
        if let Some(collection) = self.collections.get(canonical_ref) {
            return Ok(collection);
        }

        // Slow path: try lazy loading from disk
        let data_dir = Self::get_data_dir();

        // First, try to load from .vecdb archive (compact format)
        use crate::storage::{StorageFormat, StorageReader, detect_format};
        if detect_format(&data_dir) == StorageFormat::Compact {
            debug!(
                "📥 Lazy loading collection '{}' from .vecdb archive",
                canonical_ref
            );

            match StorageReader::new(&data_dir) {
                Ok(reader) => {
                    // Read the _vector_store.bin file from the archive
                    let vector_store_path = format!("{}_vector_store.bin", canonical_ref);
                    match reader.read_file(&vector_store_path) {
                        Ok(data) => {
                            // Try to deserialize as PersistedVectorStore first (correct format)
                            match serde_json::from_slice::<crate::persistence::PersistedVectorStore>(
                                &data,
                            ) {
                                Ok(persisted_store) => {
                                    // Extract the first collection from the store
                                    if let Some(mut persisted) =
                                        persisted_store.collections.into_iter().next()
                                    {
                                        // BACKWARD COMPATIBILITY: If name is empty, infer from filename
                                        if persisted.name.is_empty() {
                                            persisted.name = canonical_ref.to_string();
                                        }

                                        // Load collection into memory
                                        if let Err(e) = self.load_persisted_collection_from_data(
                                            canonical_ref,
                                            persisted,
                                        ) {
                                            warn!(
                                                "Failed to load collection '{}' from .vecdb: {}",
                                                canonical_ref, e
                                            );
                                            return Err(VectorizerError::CollectionNotFound(
                                                name.to_string(),
                                            ));
                                        }

                                        info!(
                                            "✅ Lazy loaded collection '{}' from .vecdb",
                                            canonical_ref
                                        );

                                        // Try again now that it's loaded
                                        return self.collections.get(canonical_ref).ok_or_else(
                                            || {
                                                VectorizerError::CollectionNotFound(
                                                    name.to_string(),
                                                )
                                            },
                                        );
                                    } else {
                                        warn!(
                                            "No collection found in vector store file '{}'",
                                            vector_store_path
                                        );
                                    }
                                }
                                Err(_) => {
                                    // Fallback: try deserializing as PersistedCollection directly (legacy format)
                                    match serde_json::from_slice::<
                                        crate::persistence::PersistedCollection,
                                    >(&data)
                                    {
                                        Ok(mut persisted) => {
                                            if persisted.name.is_empty() {
                                                persisted.name = canonical_ref.to_string();
                                            }

                                            if let Err(e) = self
                                                .load_persisted_collection_from_data(
                                                    canonical_ref,
                                                    persisted,
                                                )
                                            {
                                                warn!(
                                                    "Failed to load collection '{}' from .vecdb: {}",
                                                    canonical_ref, e
                                                );
                                                return Err(VectorizerError::CollectionNotFound(
                                                    name.to_string(),
                                                ));
                                            }

                                            info!(
                                                "✅ Lazy loaded collection '{}' from .vecdb (legacy format)",
                                                canonical_ref
                                            );

                                            return self.collections.get(canonical_ref).ok_or_else(
                                                || {
                                                    VectorizerError::CollectionNotFound(
                                                        name.to_string(),
                                                    )
                                                },
                                            );
                                        }
                                        Err(_) => {
                                            debug!(
                                                "Failed to deserialize collection '{}' from .vecdb (both formats failed)",
                                                canonical_ref
                                            );
                                        }
                                    }
                                }
                            }
                        }
                        Err(e) => {
                            debug!(
                                "Collection file '{}' not found in .vecdb: {}",
                                vector_store_path, e
                            );
                        }
                    }
                }
                Err(e) => {
                    warn!("Failed to create StorageReader: {}", e);
                }
            }
        }

        // Fallback: try loading from legacy _vector_store.bin file
        let collection_file = data_dir.join(format!("{}_vector_store.bin", name));

        if collection_file.exists() {
            debug!(
                "📥 Lazy loading collection '{}' from legacy .bin file",
                name
            );

            if let Err(e) = self.load_persisted_collection(&collection_file, name) {
                debug!(
                    "Failed to lazy load collection '{}' from legacy file: {}",
                    name, e
                );
                return Err(VectorizerError::CollectionNotFound(name.to_string()));
            }

            return self
                .collections
                .get(name)
                .ok_or_else(|| VectorizerError::CollectionNotFound(name.to_string()));
        }

        Err(VectorizerError::CollectionNotFound(name.to_string()))
    }

    /// Load collection from PersistedCollection data (in-memory; no file I/O here)
    pub(super) fn load_persisted_collection_from_data(
        &self,
        name: &str,
        persisted: crate::persistence::PersistedCollection,
    ) -> Result<()> {
        use crate::models::Vector;

        let vector_count = persisted.vectors.len();
        info!(
            "Loading collection '{}' with {} vectors from .vecdb",
            name, vector_count
        );

        // Create collection if it doesn't exist
        let config = if !self.has_collection_in_memory(name) {
            let config = persisted.config.clone().unwrap_or_else(|| {
                debug!("⚠️  Collection '{}' has no config, using default", name);
                crate::models::CollectionConfig::default()
            });
            self.create_collection(name, config.clone())?;
            config
        } else {
            // Get existing config
            let collection = self
                .collections
                .get(name)
                .ok_or_else(|| VectorizerError::CollectionNotFound(name.to_string()))?;
            collection.config().clone()
        };

        // Enable graph BEFORE loading vectors if graph is enabled in config
        if config.graph.as_ref().map(|g| g.enabled).unwrap_or(false) {
            if let Err(e) = self.enable_graph_for_collection(name) {
                warn!(
                    "⚠️  Failed to enable graph for collection '{}' before loading vectors: {} (continuing anyway)",
                    name, e
                );
            } else {
                info!(
                    "✅ Graph enabled for collection '{}' before loading vectors",
                    name
                );
            }
        }

        // Convert persisted vectors to runtime vectors
        let vectors: Vec<Vector> = persisted
            .vectors
            .into_iter()
            .filter_map(|pv| pv.into_runtime().ok())
            .collect();

        info!(
            "Converted {} persisted vectors to runtime format",
            vectors.len()
        );

        // Load vectors into the collection
        let collection = self
            .collections
            .get(name)
            .ok_or_else(|| VectorizerError::CollectionNotFound(name.to_string()))?;

        info!(
            "🔨 Loading {} vectors and building HNSW index for collection '{}'...",
            vectors.len(),
            name
        );
        match collection.load_vectors_into_memory(vectors) {
            Ok(_) => {
                info!(
                    "✅ Collection '{}' loaded from .vecdb with {} vectors and HNSW index built",
                    name, vector_count
                );
            }
            Err(e) => {
                warn!(
                    "❌ Failed to load vectors into collection '{}': {}",
                    name, e
                );
                return Err(e);
            }
        }

        Ok(())
    }

    /// Check if collection exists in memory only (without lazy loading)
    pub fn has_collection_in_memory(&self, name: &str) -> bool {
        match self.resolve_alias_target(name) {
            Ok(canonical) => self.collections.contains_key(canonical.as_str()),
            Err(_) => false,
        }
    }

    /// Get a mutable reference to a collection by name.
    ///
    /// # Deadlock invariant
    ///
    /// Takes an exclusive [`dashmap::mapref::one::RefMut`] on the same
    /// DashMap shard that [`Self::get_collection`] locks with a shared
    /// `Ref`. **Never call this while the current thread already holds a
    /// `Ref`/`RefMut` from `get_collection`/`get_collection_mut` on the
    /// same collection** — the shard lock is not reentrant and the call
    /// deadlocks instead of erroring. This is the exact bug fixed in
    /// `bulk_update_metadata` (phase39; see `rest_handlers/vectors.rs`).
    /// Only reach for this method when the underlying variant genuinely
    /// requires `&mut self` (currently: `HiveGpu`, which mutates a
    /// non-atomic vector-count field) — `Cpu` and `Sharded` expose `&self`
    /// mutation methods and should be called directly through the shared
    /// `Ref` from `get_collection` instead (see `VectorStore::delete` and
    /// `VectorStore::update` in `vector_store/vectors.rs`).
    pub fn get_collection_mut(
        &self,
        name: &str,
    ) -> Result<impl std::ops::DerefMut<Target = CollectionType> + '_> {
        let canonical = self.resolve_alias_target(name)?;
        let canonical_ref = canonical.as_str();

        // Ensure collection is loaded first
        let _ = self.get_collection(canonical_ref)?;

        // Now get mutable reference
        self.collections
            .get_mut(canonical_ref)
            .ok_or_else(|| VectorizerError::CollectionNotFound(name.to_string()))
    }
}
