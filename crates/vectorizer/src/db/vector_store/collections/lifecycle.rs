//! Collection lifecycle — create (CPU / owned / quantized), rename,
//! delete, graph enablement, and empty-collection cleanup.
//!
//! Lookup with lazy disk-loading lives in [`super::disk_load`];
//! ownership / multi-tenancy queries live in [`super::tenancy`].

use std::sync::Arc;

use tracing::{debug, error, info, warn};

use super::super::{CollectionType, VectorStore};
use crate::db::collection::Collection;
#[cfg(feature = "hive-gpu")]
use crate::db::hive_gpu_collection::HiveGpuCollection;
use crate::db::sharded_collection::ShardedCollection;
use crate::error::{Result, VectorizerError};
#[cfg(feature = "hive-gpu")]
use crate::gpu_adapter::GpuAdapter;
use crate::models::CollectionConfig;

impl VectorStore {
    /// Create a new collection
    pub fn create_collection(&self, name: &str, config: CollectionConfig) -> Result<()> {
        // Check GPU config - respect gpu.enabled setting
        let allow_gpu = {
            // Try to load config from config.yml
            let config_path = std::path::PathBuf::from("config.yml");
            if config_path.exists() {
                if let Ok(config_content) = std::fs::read_to_string(&config_path) {
                    if let Ok(vectorizer_config) =
                        serde_yaml::from_str::<crate::config::VectorizerConfig>(&config_content)
                    {
                        vectorizer_config.gpu.enabled
                    } else {
                        // If parsing fails, default to false (safer for persistence)
                        false
                    }
                } else {
                    false
                }
            } else {
                // No config file, default to false (safer for persistence)
                false
            }
        };

        self.create_collection_internal(name, config, allow_gpu, None)
    }

    /// Create a new collection with an owner (for multi-tenant mode)
    ///
    /// In HiveHub cluster mode, each collection is owned by a specific user/tenant.
    /// This method creates the collection and associates it with the given owner_id.
    ///
    /// Note: Respects GPU config from config.yml (same as create_collection) to ensure
    /// collections can be persisted. GPU collections are not yet supported for persistence.
    pub fn create_collection_with_owner(
        &self,
        name: &str,
        config: CollectionConfig,
        owner_id: uuid::Uuid,
    ) -> Result<()> {
        // Check GPU config - respect gpu.enabled setting (same logic as create_collection)
        let allow_gpu = {
            let config_path = std::path::PathBuf::from("config.yml");
            if config_path.exists() {
                if let Ok(config_content) = std::fs::read_to_string(&config_path) {
                    if let Ok(vectorizer_config) =
                        serde_yaml::from_str::<crate::config::VectorizerConfig>(&config_content)
                    {
                        vectorizer_config.gpu.enabled
                    } else {
                        false
                    }
                } else {
                    false
                }
            } else {
                false
            }
        };

        self.create_collection_internal(name, config, allow_gpu, Some(owner_id))
    }

    /// Create a collection with GPU disabled (for testing)
    pub fn create_collection_cpu_only(&self, name: &str, config: CollectionConfig) -> Result<()> {
        self.create_collection_internal(name, config, false, None)
    }

    /// Internal collection creation with GPU control and owner support
    fn create_collection_internal(
        &self,
        name: &str,
        config: CollectionConfig,
        allow_gpu: bool,
        owner_id: Option<uuid::Uuid>,
    ) -> Result<()> {
        // Reject empty / whitespace-only names at the chokepoint so every
        // caller (REST, gRPC, MCP, internal utilities) sees the same 400
        // instead of silently producing a collection whose name is unusable
        // as a REST path segment.
        if name.trim().is_empty() {
            return Err(VectorizerError::InvalidConfiguration {
                message: "collection name cannot be empty".to_string(),
            });
        }

        debug!("Creating collection '{}' with config: {:?}", name, config);

        if self.collections.contains_key(name) {
            return Err(VectorizerError::CollectionAlreadyExists(name.to_string()));
        }

        if self.aliases.contains_key(name) {
            return Err(VectorizerError::CollectionAlreadyExists(name.to_string()));
        }

        // Try Hive-GPU if allowed (multi-backend support)
        #[cfg(feature = "hive-gpu")]
        if allow_gpu {
            use crate::db::gpu_detection::{GpuBackendType, GpuDetector};

            info!("Detecting GPU backend for collection '{}'", name);
            let backend = GpuDetector::detect_best_backend();

            if backend != GpuBackendType::None {
                info!("Creating {} GPU collection '{}'", backend.name(), name);

                // Create GPU context for detected backend
                match GpuAdapter::create_context(backend) {
                    Ok(context) => {
                        let context = Arc::new(parking_lot::Mutex::new(context));

                        // Create Hive-GPU collection
                        let mut hive_gpu_collection = HiveGpuCollection::new(
                            name.to_string(),
                            config.clone(),
                            context,
                            backend,
                        )?;

                        // Set owner_id for multi-tenancy support
                        if let Some(id) = owner_id {
                            hive_gpu_collection.set_owner_id(Some(id));
                            debug!("GPU collection '{}' assigned to owner {}", name, id);
                        }

                        let collection = CollectionType::HiveGpu(hive_gpu_collection);
                        self.collections.insert(name.to_string(), collection);
                        info!(
                            "Collection '{}' created successfully with {} GPU",
                            name,
                            backend.name()
                        );
                        return Ok(());
                    }
                    Err(e) => {
                        warn!(
                            "Failed to create {} GPU context: {:?}, falling back to CPU",
                            backend.name(),
                            e
                        );
                    }
                }
            } else {
                info!("No GPU available, creating CPU collection for '{}'", name);
            }
        }
        // Silence unused variable warning when hive-gpu feature is disabled
        let _ = allow_gpu;

        // Check if sharding is enabled
        if config.sharding.is_some() {
            info!("Creating sharded collection '{}'", name);
            let mut sharded_collection = ShardedCollection::new(name.to_string(), config)?;

            // Set owner if provided (multi-tenant mode)
            if let Some(owner) = owner_id {
                sharded_collection.set_owner_id(Some(owner));
                debug!("Set owner_id {} for sharded collection '{}'", owner, name);
            }

            self.collections.insert(
                name.to_string(),
                CollectionType::Sharded(sharded_collection),
            );
            info!("Sharded collection '{}' created successfully", name);
            return Ok(());
        }

        // Fallback to CPU
        debug!("Creating CPU-based collection '{}'", name);
        let mut collection = Collection::new(name.to_string(), config);

        // Set owner if provided (multi-tenant mode)
        if let Some(owner) = owner_id {
            collection.set_owner_id(Some(owner));
            debug!("Set owner_id {} for CPU collection '{}'", owner, name);
        }

        self.collections
            .insert(name.to_string(), CollectionType::Cpu(collection));

        info!("Collection '{}' created successfully", name);
        Ok(())
    }

    /// Create or update collection with automatic quantization
    pub fn create_collection_with_quantization(
        &self,
        name: &str,
        config: CollectionConfig,
    ) -> Result<()> {
        debug!(
            "Creating/updating collection '{}' with automatic quantization",
            name
        );

        // Check if collection already exists
        if let Some(existing_collection) = self.collections.get(name) {
            // Check if quantization is enabled in the new config
            let quantization_enabled = matches!(
                config.quantization,
                crate::models::QuantizationConfig::SQ { bits: 8 }
            );

            // Check if existing collection has quantization
            let existing_quantization_enabled = matches!(
                existing_collection.config().quantization,
                crate::models::QuantizationConfig::SQ { bits: 8 }
            );

            if quantization_enabled && !existing_quantization_enabled {
                info!(
                    "🔄 Collection '{}' needs quantization upgrade - applying automatically",
                    name
                );

                // Store existing vectors
                let existing_vectors = existing_collection.get_all_vectors();
                let vector_count = existing_vectors.len();

                if vector_count > 0 {
                    info!(
                        "📦 Storing {} existing vectors for quantization upgrade",
                        vector_count
                    );

                    // Remove old collection
                    drop(existing_collection);
                    self.collections.remove(name);

                    // Create new collection with quantization
                    self.create_collection(name, config)?;

                    // Get the new collection
                    let mut new_collection = self.get_collection_mut(name)?;

                    // Apply quantization to existing vectors
                    for vector in existing_vectors {
                        let vector_id = vector.id.clone();
                        if let Err(e) = new_collection.add_vector(vector_id.clone(), vector) {
                            warn!(
                                "Failed to add vector {} to quantized collection: {}",
                                vector_id, e
                            );
                        }
                    }

                    info!(
                        "✅ Successfully upgraded collection '{}' with quantization for {} vectors",
                        name, vector_count
                    );
                } else {
                    // Collection is empty, just recreate with new config
                    drop(existing_collection);
                    self.collections.remove(name);
                    self.create_collection(name, config)?;
                    info!("✅ Recreated empty collection '{}' with quantization", name);
                }
            } else {
                debug!(
                    "Collection '{}' already has correct quantization configuration",
                    name
                );
            }
        } else {
            // Collection doesn't exist, create it normally with quantization
            self.create_collection(name, config)?;
        }

        Ok(())
    }

    /// Atomically rename a collection.
    ///
    /// ## What this does
    ///
    /// 1. Validates `old_name` / `new_name` (non-empty, different, no `/`).
    /// 2. Resolves `old_name` through the alias table to find the canonical key.
    /// 3. Removes the entry under the canonical key and reinserts it under
    ///    `new_name`, then calls `CollectionType::set_name` so the value's
    ///    own name field stays in sync with the map key.
    /// 4. Registers the old canonical name as a grace-window alias pointing to
    ///    `new_name` so existing callers keep working without reconfiguration.
    /// 5. Appends a `RenameCollection` op to the replication WAL so that
    ///    replicas observe the rename on their next partial-sync.
    ///
    /// ## Persistence
    ///
    /// The on-disk `.vecdb` archive is written by `AutoSaveManager::force_save`
    /// (5-minute timer or explicit REST call). The caller (`rename_collection`
    /// REST handler) always invokes `auto_save_manager.mark_changed()` after
    /// this method returns, guaranteeing that the next compaction writes the
    /// new name as the `PersistedCollection.name` field.  The collection is
    /// therefore durable after the next compaction cycle or explicit flush.
    ///
    /// ## No-op cause (fixed here)
    ///
    /// Previously the method only swapped the HashMap key but did NOT update
    /// the `Collection.name` (or equivalent) field stored inside the value.
    /// As a result:
    /// - `collection.name()` returned the stale old name, so `GET /collections`
    ///   reported the old name even after a successful 200 rename response.
    /// - The replication WAL had no `RenameCollection` variant, so replicas
    ///   never applied the rename.
    pub fn rename_collection(&self, old_name: &str, new_name: &str) -> Result<()> {
        let old_name = old_name.trim();
        let new_name = new_name.trim();

        if old_name.is_empty() {
            return Err(VectorizerError::InvalidConfiguration {
                message: "old collection name cannot be empty".to_string(),
            });
        }
        if new_name.is_empty() {
            return Err(VectorizerError::InvalidConfiguration {
                message: "new collection name cannot be empty".to_string(),
            });
        }
        // Mirror create_collection validation: reject names containing '/'.
        if new_name.contains('/') {
            return Err(VectorizerError::InvalidConfiguration {
                message: "collection name must not contain '/'".to_string(),
            });
        }
        if old_name == new_name {
            return Err(VectorizerError::InvalidConfiguration {
                message: "rename source equals destination".to_string(),
            });
        }

        // Resolve in case old_name is itself an alias.
        let canonical_old = self.resolve_alias_target(old_name)?;

        // Ensure the source exists (will lazy-load from disk if needed).
        let _ = self.get_collection(canonical_old.as_str())?;

        // Destination must not collide with an existing collection or alias.
        if self.collections.contains_key(new_name) {
            return Err(VectorizerError::CollectionAlreadyExists(
                new_name.to_string(),
            ));
        }
        if self.aliases.contains_key(new_name) {
            return Err(VectorizerError::CollectionAlreadyExists(
                new_name.to_string(),
            ));
        }

        // Atomic swap: remove under old key, reinsert under new key.
        let (_old_key, mut collection) = self
            .collections
            .remove(canonical_old.as_str())
            .ok_or_else(|| VectorizerError::CollectionNotFound(old_name.to_string()))?;

        // Synchronise the value's own name field with the new map key so that
        // any call to `collection.name()` (e.g. in `list_collections` metadata
        // building or `compact_from_memory` serialisation) returns the correct
        // name and does not carry the stale old name into the .vecdb archive.
        collection.set_name(new_name.to_string());

        self.collections.insert(new_name.to_string(), collection);

        // Register old canonical name as a grace-window alias → new name.
        // Any existing aliases that pointed to canonical_old are re-targeted.
        self.aliases
            .iter_mut()
            .filter(|e| e.value().as_str() == canonical_old.as_str())
            .for_each(|mut e| {
                *e.value_mut() = new_name.to_string();
            });

        // Also keep the old canonical name itself as an alias so callers
        // using the old name continue to resolve it transparently.
        self.aliases
            .insert(canonical_old.to_string(), new_name.to_string());

        // Append a RenameCollection op to the replication WAL so replicas can
        // apply the rename during their next partial-sync round. Fire-and-
        // forget — the existing log_wal_insert / update / delete helpers use
        // the same pattern (best-effort; never block the request).
        {
            let wal_guard = self.wal.lock();
            if let Some(wal) = wal_guard.as_ref() {
                let wal_clone = wal.clone();
                let old = canonical_old.to_string();
                let new = new_name.to_string();
                if tokio::runtime::Handle::try_current().is_ok() {
                    tokio::spawn(async move {
                        if let Err(e) = wal_clone.log_rename_collection(&old, &new).await {
                            error!("Failed to log rename to WAL: {}", e);
                        }
                    });
                } else if let Ok(rt) = tokio::runtime::Runtime::new() {
                    if let Err(e) =
                        rt.block_on(async { wal_clone.log_rename_collection(&old, &new).await })
                    {
                        error!("Failed to log rename to WAL: {}", e);
                    }
                }
            }
        }

        info!(
            "Collection '{}' renamed to '{}'; '{}' kept as grace-window alias",
            canonical_old, new_name, canonical_old
        );
        Ok(())
    }

    /// Remove the grace-window alias created by `rename_collection`.
    ///
    /// Operators call this once client migration to the new name is
    /// complete (typically one minor version after the rename).
    pub fn drop_rename_alias(&self, old_name: &str) -> Result<()> {
        self.delete_alias(old_name)
    }

    /// Delete a collection
    pub fn delete_collection(&self, name: &str) -> Result<()> {
        debug!("Deleting collection '{}'", name);

        let canonical = self.resolve_alias_target(name)?;

        self.collections
            .remove(canonical.as_str())
            .ok_or_else(|| VectorizerError::CollectionNotFound(name.to_string()))?;

        // Remove any aliases pointing to this collection
        self.remove_aliases_for_collection(canonical.as_str());

        info!(
            "Collection '{}' (canonical '{}') deleted successfully",
            name, canonical
        );
        Ok(())
    }

    /// Enable graph for an existing collection and populate with existing vectors
    pub fn enable_graph_for_collection(&self, collection_name: &str) -> Result<()> {
        let canonical = self.resolve_alias_target(collection_name)?;
        let canonical_ref = canonical.as_str();

        // Ensure collection is loaded first
        let _ = self.get_collection(canonical_ref)?;

        // Get mutable reference to collection
        let mut collection_ref = self.get_collection_mut(canonical_ref)?;

        match &mut *collection_ref {
            CollectionType::Cpu(collection) => {
                // Check if graph already exists in memory
                if collection.get_graph().is_some() {
                    info!(
                        "Graph already enabled for collection '{}', skipping",
                        canonical_ref
                    );
                    return Ok(());
                }

                // Try to load graph from disk first (only if file actually exists)
                let data_dir = Self::get_data_dir();
                let graph_path = data_dir.join(format!("{}_graph.json", canonical_ref));

                if graph_path.exists() {
                    if let Ok(graph) =
                        crate::db::graph::Graph::load_from_file(canonical_ref, &data_dir)
                    {
                        let node_count = graph.node_count();
                        let edge_count = graph.edge_count();

                        // Only use disk graph if it has nodes
                        if node_count > 0 {
                            collection.set_graph(Arc::new(graph.clone()));
                            info!(
                                "Loaded graph for collection '{}' from disk with {} nodes and {} edges",
                                canonical_ref, node_count, edge_count
                            );

                            // If graph has nodes but no edges, discover edges automatically
                            if edge_count == 0 {
                                info!(
                                    "Graph for '{}' has {} nodes but no edges, discovering edges automatically",
                                    canonical_ref, node_count
                                );

                                let config = crate::models::AutoRelationshipConfig {
                                    similarity_threshold: 0.7,
                                    max_per_node: 10,
                                    enabled_types: vec!["SIMILAR_TO".to_string()],
                                };

                                let nodes = graph.get_all_nodes();
                                let nodes_to_process: Vec<String> =
                                    nodes.iter().take(100).map(|n| n.id.clone()).collect();

                                let mut edges_created = 0;
                                for node_id in &nodes_to_process {
                                    if let Ok(_edges) =
                                        crate::db::graph_relationship_discovery::discover_edges_for_node(
                                            &graph, node_id, collection, &config,
                                        )
                                    {
                                        edges_created += _edges;
                                    }
                                }

                                info!(
                                    "Auto-discovery created {} edges for {} nodes in collection '{}' (use API endpoint /graph/discover/{} for full discovery)",
                                    edges_created,
                                    nodes_to_process.len().min(node_count),
                                    canonical_ref,
                                    canonical_ref
                                );
                            }

                            return Ok(());
                        }
                    }
                }

                // No valid graph on disk, create new graph
                collection.enable_graph()
            }
            #[cfg(feature = "hive-gpu")]
            CollectionType::HiveGpu(_) => Err(VectorizerError::Storage(
                "Graph not yet supported for GPU collections".to_string(),
            )),
            CollectionType::Sharded(_) => Err(VectorizerError::Storage(
                "Graph not yet supported for sharded collections".to_string(),
            )),
            CollectionType::DistributedSharded(_) => Err(VectorizerError::Storage(
                "Graph not yet supported for distributed collections".to_string(),
            )),
        }
    }

    /// Enable graph for all workspace collections
    pub fn enable_graph_for_all_workspace_collections(&self) -> Result<Vec<String>> {
        let collections = self.list_collections();
        let mut enabled = Vec::new();

        for collection_name in collections {
            match self.enable_graph_for_collection(&collection_name) {
                Ok(_) => {
                    info!("✅ Graph enabled for collection '{}'", collection_name);
                    enabled.push(collection_name);
                }
                Err(e) => {
                    warn!(
                        "⚠️ Failed to enable graph for collection '{}': {}",
                        collection_name, e
                    );
                }
            }
        }

        Ok(enabled)
    }

    /// List all collections (both loaded in memory and available on disk)
    pub fn list_collections(&self) -> Vec<String> {
        use std::collections::HashSet;

        let mut collection_names = HashSet::new();

        // Add collections already loaded in memory
        for entry in self.collections.iter() {
            collection_names.insert(entry.key().clone());
        }

        // Add collections available on disk
        let data_dir = Self::get_data_dir();
        if data_dir.exists() {
            if let Ok(entries) = std::fs::read_dir(data_dir) {
                for entry in entries.flatten() {
                    if let Some(filename) = entry.file_name().to_str() {
                        if filename.ends_with("_vector_store.bin") {
                            if let Some(name) = filename.strip_suffix("_vector_store.bin") {
                                collection_names.insert(name.to_string());
                            }
                        }
                    }
                }
            }
        }

        collection_names.into_iter().collect()
    }

    /// Check if a collection is empty (has zero vectors)
    pub fn is_collection_empty(&self, name: &str) -> Result<bool> {
        let collection_ref = self.get_collection(name)?;
        Ok(collection_ref.vector_count() == 0)
    }

    /// List all empty collections
    pub fn list_empty_collections(&self) -> Vec<String> {
        self.list_collections()
            .into_iter()
            .filter(|name| self.is_collection_empty(name).unwrap_or(false))
            .collect()
    }

    /// Cleanup (delete) all empty collections
    pub fn cleanup_empty_collections(&self, dry_run: bool) -> Result<usize> {
        let empty_collections = self.list_empty_collections();
        let count = empty_collections.len();

        if dry_run {
            info!(
                "🧹 Dry run: Would delete {} empty collections: {:?}",
                count, empty_collections
            );
            return Ok(count);
        }

        let mut deleted_count = 0;
        for collection_name in &empty_collections {
            if let Err(e) = self.delete_collection(collection_name) {
                warn!(
                    "Failed to delete empty collection '{}': {}",
                    collection_name, e
                );
            } else {
                info!("Deleted empty collection '{}'", collection_name);
                deleted_count += 1;
            }
        }

        info!(
            "🧹 Cleanup complete: deleted {} empty collections",
            deleted_count
        );
        Ok(deleted_count)
    }
}
