//! Workspace + file-watcher config loaders used during bootstrap.
//!
//! - [`load_file_watcher_config`] — reads `workspace.yml` into a
//!   [`crate::file_watcher::FileWatcherConfig`], falling back to
//!   defaults on any error.
//! - [`load_workspace_collections`] — walks the workspace tree,
//!   reconciles against the on-disk `.vecdb` archive (if any), and
//!   either reloads pre-indexed collections into memory or kicks off a
//!   fresh [`FileLoader`] pass against the project files.
//!
//! This module has no knowledge of the server struct — it operates
//! purely on `VectorStore` + `EmbeddingManager` references so it can
//! also be called from tests / tools.

use std::sync::Arc;

use tracing::{debug, info, warn};

use crate::VectorStore;
use crate::embedding::EmbeddingManager;

/// Load file watcher configuration from workspace.yml
pub(super) async fn load_file_watcher_config()
-> anyhow::Result<crate::file_watcher::FileWatcherConfig> {
    match crate::file_watcher::FileWatcherConfig::from_yaml_file("workspace.yml") {
        Ok(config) => {
            info!(
                "Loaded file watcher configuration from workspace: watch_paths={:?}, exclude_patterns={:?}",
                config.watch_paths, config.exclude_patterns
            );
            Ok(config)
        }
        Err(e) => {
            info!(
                "Failed to load workspace configuration: {}, using default file watcher config",
                e
            );
            Ok(crate::file_watcher::FileWatcherConfig::default())
        }
    }
}

/// Load workspace collections using the file_loader module.
///
/// Returns the number of collections indexed / loaded. Collections
/// that already exist in memory are skipped; collections present in
/// the `.vecdb` archive are force-loaded with their HNSW index;
/// everything else is indexed from project files via
/// [`crate::file_loader::FileLoader`].
pub(super) async fn load_workspace_collections(
    store: &Arc<VectorStore>,
    embedding_manager: &Arc<EmbeddingManager>,
    mut cancel_rx: tokio::sync::watch::Receiver<bool>,
) -> anyhow::Result<usize> {
    use std::path::Path;

    use crate::file_loader::{FileLoader, LoaderConfig};
    use crate::workspace::manager::WorkspaceManager;

    // Look for workspace configuration file
    let workspace_file = Path::new("workspace.yml");
    info!(
        "Checking for workspace file at: {}",
        workspace_file.display()
    );
    if !workspace_file.exists() {
        info!("No workspace configuration file found at workspace.yml");
        return Ok(0);
    }

    info!("Found workspace configuration file, loading...");

    // Load workspace configuration
    let workspace_manager = match WorkspaceManager::load_from_file(workspace_file) {
        Ok(manager) => manager,
        Err(e) => {
            warn!("Failed to load workspace configuration: {}", e);
            return Ok(0);
        }
    };

    info!(
        "Workspace loaded with {} projects",
        workspace_manager.config().projects.len()
    );

    let mut indexed_count = 0;

    // Check if using .vecdb format - if so, verify collections in archive first
    let data_dir = std::path::PathBuf::from("./data");
    let vecdb_path = data_dir.join("vectorizer.vecdb");
    let using_vecdb = vecdb_path.exists();

    let mut existing_in_vecdb = std::collections::HashSet::new();
    if using_vecdb {
        use crate::storage::StorageReader;
        info!(
            "🔍 .vecdb file exists at {}, attempting to read...",
            vecdb_path.display()
        );
        match StorageReader::new(&data_dir) {
            Ok(reader) => match reader.list_collections() {
                Ok(collections) => {
                    existing_in_vecdb = collections.iter().cloned().collect();
                    info!(
                        "📦 Found {} collections in .vecdb archive: {:?}",
                        existing_in_vecdb.len(),
                        existing_in_vecdb
                    );
                }
                Err(e) => {
                    warn!("Failed to list collections from .vecdb: {}", e);
                }
            },
            Err(e) => {
                warn!("Failed to create StorageReader for .vecdb: {}", e);
            }
        }
    } else {
        info!("📦 .vecdb file does not exist, will create after indexing");
    }

    // Process each enabled project
    for project in workspace_manager.enabled_projects() {
        // Check for cancellation
        if *cancel_rx.borrow() {
            info!("🛑 Workspace loading cancelled by user");
            break;
        }

        info!("Processing project: {}", project.name);

        for collection in &project.collections {
            // Check for cancellation
            if *cancel_rx.borrow() {
                info!("🛑 Workspace loading cancelled by user");
                break;
            }

            info!("Processing collection: {}", collection.name);

            // Check if collection already exists in .vecdb archive
            if using_vecdb && existing_in_vecdb.contains(&collection.name) {
                // Collection exists in .vecdb - FORCE LOAD it into memory (same as when no cache)
                if !store.has_collection_in_memory(&collection.name) {
                    info!(
                        "📥 FORCE LOADING collection '{}' from .vecdb into memory...",
                        collection.name
                    );

                    // Use the SAME method as when .vecdb doesn't exist - load directly from .vecdb
                    use crate::storage::StorageReader;
                    match StorageReader::new(&std::path::PathBuf::from("./data")) {
                        Ok(reader) => {
                            let vector_store_path = format!("{}_vector_store.bin", collection.name);
                            match reader.read_file(&vector_store_path) {
                                Ok(data) => {
                                    // Try to deserialize as PersistedVectorStore first (correct format)
                                    let persisted = match serde_json::from_slice::<
                                        crate::persistence::PersistedVectorStore,
                                    >(
                                        &data
                                    ) {
                                        Ok(persisted_store) => {
                                            // Extract the first collection from the store
                                            persisted_store.collections.into_iter().next()
                                        }
                                        Err(_) => {
                                            // Fallback: try deserializing as PersistedCollection directly (legacy format)
                                            serde_json::from_slice::<
                                                crate::persistence::PersistedCollection,
                                            >(&data)
                                            .ok()
                                        }
                                    };

                                    if let Some(mut persisted) = persisted {
                                        // BACKWARD COMPATIBILITY: If name is empty, infer from filename
                                        if persisted.name.is_empty() {
                                            persisted.name = collection.name.clone();
                                        }

                                        // Use EXACT config from .vecdb (not workspace config!)
                                        let config = persisted.config.clone().unwrap_or_else(|| {
                                                warn!("⚠️  Collection '{}' has no config in .vecdb, using default", collection.name);
                                                crate::models::CollectionConfig::default()
                                            });
                                        let vector_count = persisted.vectors.len();

                                        info!(
                                            "📥 Loading collection '{}' from .vecdb with {} vectors...",
                                            collection.name, vector_count
                                        );

                                        // Convert vectors FIRST (before creating collection)
                                        info!(
                                            "🔄 Converting {} persisted vectors to runtime format...",
                                            persisted.vectors.len()
                                        );
                                        let vectors: Vec<crate::models::Vector> = persisted
                                            .vectors
                                            .into_iter()
                                            .filter_map(|pv| match pv.into_runtime() {
                                                Ok(v) => Some(v),
                                                Err(e) => {
                                                    warn!(
                                                        "Failed to convert persisted vector: {}",
                                                        e
                                                    );
                                                    None
                                                }
                                            })
                                            .collect();

                                        info!(
                                            "🔄 Converted {} vectors successfully",
                                            vectors.len()
                                        );

                                        // Create collection with config FROM .vecdb
                                        if let Err(e) =
                                            store.create_collection(&collection.name, config)
                                        {
                                            // Collection might already exist from lazy loading - just load vectors with HNSW
                                            warn!(
                                                "Collection '{}' already exists (maybe from lazy loading), loading vectors with HNSW anyway: {}",
                                                collection.name, e
                                            );
                                            if let Ok(mut collection_ref) =
                                                store.get_collection_mut(&collection.name)
                                            {
                                                info!(
                                                    "🔄 Loading {} vectors with HNSW index into existing collection '{}'...",
                                                    vectors.len(),
                                                    collection.name
                                                );
                                                // Use fast_load_vectors() to build HNSW index properly
                                                if let Err(e) =
                                                    collection_ref.fast_load_vectors(vectors)
                                                {
                                                    warn!(
                                                        "❌ FAILED to load vectors with HNSW into collection '{}': {}",
                                                        collection.name, e
                                                    );
                                                } else {
                                                    // Enable graph for this collection automatically
                                                    if let Err(e) = store
                                                        .enable_graph_for_collection(
                                                            &collection.name,
                                                        )
                                                    {
                                                        warn!(
                                                            "⚠️  Failed to enable graph for collection '{}': {} (continuing anyway)",
                                                            collection.name, e
                                                        );
                                                    } else {
                                                        info!(
                                                            "✅ Graph enabled for collection '{}'",
                                                            collection.name
                                                        );
                                                    }

                                                    info!(
                                                        "✅ Collection '{}' loaded from .vecdb with {} vectors + HNSW index",
                                                        collection.name, vector_count
                                                    );
                                                    indexed_count += 1;
                                                }
                                            }
                                            continue;
                                        }

                                        // Collection created successfully - now load vectors with HNSW index
                                        if let Ok(mut collection_ref) =
                                            store.get_collection_mut(&collection.name)
                                        {
                                            info!(
                                                "🔄 Loading {} vectors with HNSW index into collection '{}'...",
                                                vectors.len(),
                                                collection.name
                                            );
                                            // Use fast_load_vectors() to build HNSW index properly
                                            if let Err(e) =
                                                collection_ref.fast_load_vectors(vectors)
                                            {
                                                warn!(
                                                    "❌ FAILED to load vectors with HNSW into collection '{}': {}",
                                                    collection.name, e
                                                );
                                            } else {
                                                // Enable graph for this collection automatically
                                                if let Err(e) = store
                                                    .enable_graph_for_collection(&collection.name)
                                                {
                                                    warn!(
                                                        "⚠️  Failed to enable graph for collection '{}': {} (continuing anyway)",
                                                        collection.name, e
                                                    );
                                                } else {
                                                    info!(
                                                        "✅ Graph enabled for collection '{}'",
                                                        collection.name
                                                    );
                                                }

                                                info!(
                                                    "✅ Collection '{}' loaded from .vecdb with {} vectors + HNSW index",
                                                    collection.name, vector_count
                                                );
                                                indexed_count += 1;
                                            }
                                        } else {
                                            warn!(
                                                "❌ FAILED to get collection '{}' after creation!",
                                                collection.name
                                            );
                                        }
                                    } else {
                                        debug!(
                                            "Failed to deserialize collection '{}' from .vecdb: no collection found",
                                            collection.name
                                        );
                                    }
                                }
                                Err(e) => {
                                    warn!(
                                        "Failed to read collection '{}' from .vecdb: {}",
                                        collection.name, e
                                    );
                                }
                            }
                        }
                        Err(e) => {
                            warn!(
                                "Failed to create StorageReader for collection '{}': {}",
                                collection.name, e
                            );
                        }
                    }
                } else {
                    info!(
                        "✅ Collection '{}' already in memory, skipping",
                        collection.name
                    );
                }
                continue;
            }

            // Check if collection already exists in store memory (WITHOUT lazy loading)
            if store.has_collection_in_memory(&collection.name) {
                info!(
                    "✅ Collection '{}' already exists in memory, skipping",
                    collection.name
                );
                continue;
            }

            // Get project path
            let project_path = match workspace_manager.get_project_path(&project.name) {
                Ok(path) => path,
                Err(e) => {
                    warn!("Failed to get project path for '{}': {}", project.name, e);
                    continue;
                }
            };

            // Use FileLoader to index files
            let mut loader_config = LoaderConfig {
                max_chunk_size: 2048,
                chunk_overlap: 256,
                include_patterns: collection.processing.include_patterns.clone(),
                exclude_patterns: collection.processing.exclude_patterns.clone(),
                embedding_dimension: collection.embedding.dimension,
                embedding_type: "bm25".to_string(),
                collection_name: collection.name.clone(),
                max_file_size: 1024 * 1024, // 1MB
            };

            // CRITICAL: Always enforce hardcoded exclusions (Python cache, binaries, etc.)
            loader_config.ensure_hardcoded_excludes();

            // Create embedding manager for this collection
            let mut coll_embedding_manager = crate::embedding::EmbeddingManager::new();
            let bm25 = crate::embedding::Bm25Embedding::new(collection.embedding.dimension);
            coll_embedding_manager.register_provider("bm25".to_string(), Box::new(bm25));
            coll_embedding_manager.set_default_provider("bm25")?;

            let mut loader =
                FileLoader::with_embedding_manager(loader_config, coll_embedding_manager);

            match loader
                .load_and_index_project(&project_path.to_string_lossy(), store)
                .await
            {
                Ok(file_count) => {
                    if file_count > 0 {
                        info!(
                            "Indexed {} vectors for collection '{}'",
                            file_count, collection.name
                        );
                        indexed_count += 1;
                    } else {
                        info!(
                            "Collection '{}' already exists in .vecdb, no indexing needed",
                            collection.name
                        );
                    }
                }
                Err(e) => {
                    warn!("Failed to index collection '{}': {}", collection.name, e);
                }
            }
        }
    }

    // Silence unused warning when EmbeddingManager isn't currently consulted
    // by the per-collection path above — the parameter stays in the signature
    // because callers in `bootstrap` already own it.
    let _ = embedding_manager;

    Ok(indexed_count)
}
