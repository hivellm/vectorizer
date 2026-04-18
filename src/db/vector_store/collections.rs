//! Collection lifecycle — create, delete, lookup (with lazy loading
//! from disk), ownership, graph enablement, and empty-collection
//! cleanup.
//!
//! `get_collection` / `get_collection_mut` are the workhorses: they
//! resolve alias chains, try the in-memory `DashMap` first, and fall
//! back to `.vecdb` (compact) and legacy `.bin` (raw) on disk.
//! Everything else in this file composes those two.

use std::sync::Arc;

use tracing::{debug, info, warn};

use super::{CollectionType, VectorStore};
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

    /// Get a reference to a collection by name.
    ///
    /// Lazy-loads from `.vecdb` (compact) or `.bin` (legacy) on disk
    /// if the collection isn't already in memory.
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

    /// Get a mutable reference to a collection by name
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
        if owner_id.is_none() {
            return Ok(collection);
        }

        let owner = owner_id.unwrap();

        // Check ownership - allow access if collection has no owner or matches
        if collection.owner_id().is_none() || collection.belongs_to(owner) {
            Ok(collection)
        } else {
            Err(VectorizerError::CollectionNotFound(name.to_string()))
        }
    }
}
