//! Persistence — loading collections from disk (compact `.vecdb`
//! archive or legacy per-collection `.bin` files), and cache-path
//! helpers for the HNSW dump-assisted fast load.
//!
//! The save side lives in [`super::autosave`].

use std::path::PathBuf;

use tracing::{debug, error, info, warn};

use super::{CollectionType, VectorStore};
use crate::error::{Result, VectorizerError};

impl VectorStore {
    /// Get the centralized data directory path (same as DocumentLoader)
    pub fn get_data_dir() -> PathBuf {
        let current_dir = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
        current_dir.join("data")
    }

    /// Load a collection from cache without reconstructing the HNSW index
    pub fn load_collection_from_cache(
        &self,
        collection_name: &str,
        persisted_vectors: Vec<crate::persistence::PersistedVector>,
    ) -> Result<()> {
        debug!(
            "Fast loading collection '{}' from cache with {} vectors",
            collection_name,
            persisted_vectors.len()
        );

        let mut collection_ref = self.get_collection_mut(collection_name)?;

        match &mut *collection_ref {
            CollectionType::Cpu(c) => {
                c.load_from_cache(persisted_vectors)?;
                // Requantize existing vectors if quantization is enabled
                c.requantize_existing_vectors()?;
            }
            #[cfg(feature = "hive-gpu")]
            CollectionType::HiveGpu(c) => {
                c.load_from_cache(persisted_vectors)?;
            }
            CollectionType::Sharded(_) => {
                warn!("Sharded collections don't support load_from_cache yet");
            }
            CollectionType::DistributedSharded(_) => {
                warn!("Distributed collections don't support load_from_cache yet");
            }
        }

        Ok(())
    }

    /// Load a collection from cache with optional HNSW dump for instant loading
    pub fn load_collection_from_cache_with_hnsw_dump(
        &self,
        collection_name: &str,
        persisted_vectors: Vec<crate::persistence::PersistedVector>,
        hnsw_dump_path: Option<&std::path::Path>,
        hnsw_basename: Option<&str>,
    ) -> Result<()> {
        debug!(
            "Loading collection '{}' from cache with {} vectors (HNSW dump: {})",
            collection_name,
            persisted_vectors.len(),
            hnsw_basename.is_some()
        );

        let mut collection_ref = self.get_collection_mut(collection_name)?;

        match &mut *collection_ref {
            CollectionType::Cpu(c) => {
                c.load_from_cache_with_hnsw_dump(persisted_vectors, hnsw_dump_path, hnsw_basename)?
            }
            #[cfg(feature = "hive-gpu")]
            CollectionType::HiveGpu(c) => {
                c.load_from_cache_with_hnsw_dump(persisted_vectors, hnsw_dump_path, hnsw_basename)?;
            }
            CollectionType::Sharded(_) => {
                warn!("Sharded collections don't support load_from_cache_with_hnsw_dump yet");
            }
            CollectionType::DistributedSharded(_) => {
                warn!(
                    "Distributed collections don't support \
                     load_from_cache_with_hnsw_dump yet"
                );
            }
        }

        Ok(())
    }

    /// Load all persisted collections from the data directory
    pub fn load_all_persisted_collections(&self) -> Result<usize> {
        let data_dir = Self::get_data_dir();
        if !data_dir.exists() {
            debug!("Data directory does not exist: {:?}", data_dir);
            return Ok(0);
        }

        info!("🔍 Detecting storage format...");

        // Detect storage format
        let format = crate::storage::detect_format(&data_dir);

        match format {
            crate::storage::StorageFormat::Compact => {
                info!("📦 Found vectorizer.vecdb - loading from compressed archive");
                self.load_from_vecdb()
            }
            crate::storage::StorageFormat::Legacy => {
                info!("📁 Using legacy format - loading from raw files");
                self.load_from_raw_files()
            }
        }
    }

    /// Load collections from vectorizer.vecdb (compressed archive)
    /// NEVER falls back to raw files — .vecdb is the ONLY source of truth
    fn load_from_vecdb(&self) -> Result<usize> {
        use crate::storage::StorageReader;

        let data_dir = Self::get_data_dir();
        let reader = match StorageReader::new(&data_dir) {
            Ok(r) => r,
            Err(e) => {
                error!("❌ CRITICAL: Failed to create StorageReader: {}", e);
                error!("   vectorizer.vecdb exists but cannot be read!");
                error!("   This usually indicates .vecdb corruption.");
                error!("   RESTORE FROM SNAPSHOT in data/snapshots/ if available.");
                // NO FALLBACK! Return error instead
                return Err(VectorizerError::Storage(format!(
                    "Failed to read vectorizer.vecdb: {}",
                    e
                )));
            }
        };

        // Extract all collections in memory
        let persisted_collections = match reader.extract_all_collections() {
            Ok(collections) => collections,
            Err(e) => {
                error!(
                    "❌ CRITICAL: Failed to extract collections from .vecdb: {}",
                    e
                );
                error!("   This usually indicates .vecdb corruption or format mismatch");
                error!("   RESTORE FROM SNAPSHOT in data/snapshots/ if available.");
                return Err(VectorizerError::Storage(format!(
                    "Failed to extract from vectorizer.vecdb: {}",
                    e
                )));
            }
        };

        info!(
            "📦 Loading {} collections from archive...",
            persisted_collections.len()
        );

        let mut collections_loaded = 0;

        for (i, persisted_collection) in persisted_collections.iter().enumerate() {
            let collection_name = &persisted_collection.name;
            info!(
                "⏳ Loading collection {}/{}: '{}'",
                i + 1,
                persisted_collections.len(),
                collection_name
            );

            // Create collection with the persisted config
            // NOTE: We now preserve empty collections (they have valid metadata/config)
            // Previously we skipped empty collections, causing metadata loss on restart
            let mut config = persisted_collection.config.clone().unwrap_or_else(|| {
                debug!(
                    "⚠️  Collection '{}' has no config, using default",
                    collection_name
                );
                crate::models::CollectionConfig::default()
            });
            config.quantization = crate::models::QuantizationConfig::SQ { bits: 8 };

            match self.create_collection_with_quantization(collection_name, config.clone()) {
                Ok(_) => {
                    // Enable graph BEFORE loading vectors if graph is enabled in config
                    if config.graph.as_ref().map(|g| g.enabled).unwrap_or(false) {
                        if let Err(e) = self.enable_graph_for_collection(collection_name) {
                            warn!(
                                "⚠️  Failed to enable graph for collection '{}' before loading vectors: {} (continuing anyway)",
                                collection_name, e
                            );
                        } else {
                            info!(
                                "✅ Graph enabled for collection '{}' before loading vectors",
                                collection_name
                            );
                        }
                    }

                    // Load vectors if they exist
                    if persisted_collection.vectors.is_empty() {
                        // Empty collection — just count it as loaded (metadata preserved)
                        collections_loaded += 1;
                        info!(
                            "✅ Restored empty collection '{}' (metadata only) ({}/{})",
                            collection_name,
                            i + 1,
                            persisted_collections.len()
                        );
                        continue;
                    }

                    debug!(
                        "Loading {} vectors into collection '{}'",
                        persisted_collection.vectors.len(),
                        collection_name
                    );

                    match self.load_collection_from_cache(
                        collection_name,
                        persisted_collection.vectors.clone(),
                    ) {
                        Ok(_) => {
                            // If graph wasn't enabled before (config didn't have it), enable it now
                            // This handles collections that don't have graph in config but should have it enabled
                            if config.graph.as_ref().map(|g| g.enabled).unwrap_or(false) {
                                // Graph already enabled, nodes should be created
                            } else {
                                // Enable graph for all collections from workspace automatically
                                if let Err(e) = self.enable_graph_for_collection(collection_name) {
                                    warn!(
                                        "⚠️  Failed to enable graph for collection '{}': {} (continuing anyway)",
                                        collection_name, e
                                    );
                                } else {
                                    info!(
                                        "✅ Graph enabled for collection '{}' (auto-enabled for workspace)",
                                        collection_name
                                    );
                                }
                            }

                            collections_loaded += 1;
                            info!(
                                "✅ Successfully loaded collection '{}' with {} vectors ({}/{})",
                                collection_name,
                                persisted_collection.vectors.len(),
                                i + 1,
                                persisted_collections.len()
                            );
                        }
                        Err(e) => {
                            error!(
                                "❌ CRITICAL: Failed to load vectors for collection '{}': {}",
                                collection_name, e
                            );
                            // Remove the empty collection
                            let _ = self.delete_collection(collection_name);
                        }
                    }
                }
                Err(e) => {
                    error!(
                        "❌ CRITICAL: Failed to create collection '{}': {}",
                        collection_name, e
                    );
                }
            }
        }

        info!(
            "✅ Loaded {} collections from memory (no temp files)",
            collections_loaded
        );

        // SAFETY CHECK: If no collections loaded but .vecdb exists, something is wrong
        if collections_loaded == 0 && persisted_collections.len() > 0 {
            error!(
                "❌ CRITICAL: Failed to load any collections despite {} in archive!",
                persisted_collections.len()
            );
            error!("   All collections failed to deserialize — likely format mismatch");
            warn!("🔄 Attempting fallback to raw files...");
            return self.load_from_raw_files();
        }

        // Clean up any legacy raw files after successful load from .vecdb
        if collections_loaded > 0 {
            info!("🧹 Cleaning up legacy raw files...");
            match Self::cleanup_raw_files(&data_dir) {
                Ok(removed) => {
                    if removed > 0 {
                        info!("🗑️  Removed {} legacy raw files", removed);
                    } else {
                        debug!("✅ No legacy raw files to clean up");
                    }
                }
                Err(e) => {
                    warn!("⚠️  Failed to clean up raw files: {}", e);
                }
            }
        }

        Ok(collections_loaded)
    }

    /// Clean up raw collection files from data directory
    fn cleanup_raw_files(data_dir: &std::path::Path) -> Result<usize> {
        use std::fs;

        let mut removed_count = 0;

        for entry in fs::read_dir(data_dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_file() {
                if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                    // Skip .vecdb and .vecidx files
                    if name == "vectorizer.vecdb" || name == "vectorizer.vecidx" {
                        continue;
                    }

                    // Remove legacy collection files
                    if name.ends_with("_vector_store.bin")
                        || name.ends_with("_tokenizer.json")
                        || name.ends_with("_metadata.json")
                        || name.ends_with("_checksums.json")
                    {
                        match fs::remove_file(&path) {
                            Ok(_) => {
                                debug!("   Removed: {}", name);
                                removed_count += 1;
                            }
                            Err(e) => {
                                warn!("   Failed to remove {}: {}", name, e);
                            }
                        }
                    }
                }
            }
        }

        Ok(removed_count)
    }

    /// Load collections from raw files (legacy format)
    fn load_from_raw_files(&self) -> Result<usize> {
        let data_dir = Self::get_data_dir();

        // Collect all collection files first
        let mut collection_files = Vec::new();
        for entry in std::fs::read_dir(&data_dir)? {
            let entry = entry?;
            let path = entry.path();

            if let Some(extension) = path.extension() {
                if extension == "bin" {
                    // Extract collection name from filename (remove _vector_store.bin suffix)
                    if let Some(filename) = path.file_name().and_then(|n| n.to_str()) {
                        if let Some(collection_name) = filename.strip_suffix("_vector_store.bin") {
                            debug!("Found persisted collection: {}", collection_name);
                            collection_files.push((path.clone(), collection_name.to_string()));
                        }
                    }
                }
            }
        }

        info!(
            "📦 Found {} persisted collections to load",
            collection_files.len()
        );

        // Load collections sequentially but with better progress reporting
        let mut collections_loaded = 0;
        for (i, (path, collection_name)) in collection_files.iter().enumerate() {
            info!(
                "⏳ Loading collection {}/{}: '{}'",
                i + 1,
                collection_files.len(),
                collection_name
            );

            match self.load_persisted_collection(path, collection_name) {
                Ok(_) => {
                    // Enable graph for this collection automatically
                    if let Err(e) = self.enable_graph_for_collection(collection_name) {
                        warn!(
                            "⚠️  Failed to enable graph for collection '{}': {} (continuing anyway)",
                            collection_name, e
                        );
                    } else {
                        info!("✅ Graph enabled for collection '{}'", collection_name);
                    }

                    collections_loaded += 1;
                    info!(
                        "✅ Successfully loaded collection '{}' from persistence ({}/{})",
                        collection_name,
                        i + 1,
                        collection_files.len()
                    );
                }
                Err(e) => {
                    warn!(
                        "❌ Failed to load collection '{}' from {:?}: {}",
                        collection_name, path, e
                    );
                }
            }
        }

        info!(
            "📊 Loaded {} collections from raw files",
            collections_loaded
        );

        // After loading raw files, compact them to vecdb
        if collections_loaded > 0 {
            info!("💾 Compacting raw files to vectorizer.vecdb...");
            match self.compact_to_vecdb() {
                Ok(_) => info!("✅ Successfully created vectorizer.vecdb"),
                Err(e) => warn!("⚠️  Failed to create vectorizer.vecdb: {}", e),
            }
        }

        Ok(collections_loaded)
    }

    /// Compact raw files to vectorizer.vecdb
    fn compact_to_vecdb(&self) -> Result<()> {
        use crate::storage::StorageCompactor;

        let data_dir = Self::get_data_dir();
        let compactor = StorageCompactor::new(&data_dir, 6, 1000);

        info!("🗜️  Starting compaction of raw files...");

        // Compact with cleanup (remove raw files after successful compaction)
        match compactor.compact_all_with_cleanup(true) {
            Ok(index) => {
                info!("✅ Compaction completed successfully:");
                info!("   Collections: {}", index.collection_count());
                info!("   Total vectors: {}", index.total_vectors());
                info!(
                    "   Compressed size: {} MB",
                    index.compressed_size / 1_048_576
                );
                Ok(())
            }
            Err(e) => {
                error!("❌ Compaction failed: {}", e);
                error!("   Raw files have been preserved");
                Err(e)
            }
        }
    }

    /// Load dynamic collections that are not in the workspace.
    /// Call this after workspace initialization to load any additional persisted collections.
    pub fn load_dynamic_collections(&mut self) -> Result<usize> {
        let data_dir = Self::get_data_dir();
        if !data_dir.exists() {
            debug!("Data directory does not exist: {:?}", data_dir);
            return Ok(0);
        }

        let mut dynamic_collections_loaded = 0;
        let existing_collections: std::collections::HashSet<String> =
            self.list_collections().into_iter().collect();

        // Find all .bin files in the data directory that are not already loaded
        for entry in std::fs::read_dir(&data_dir)? {
            let entry = entry?;
            let path = entry.path();

            if let Some(extension) = path.extension() {
                if extension == "bin" {
                    // Extract collection name from filename (remove _vector_store.bin suffix)
                    if let Some(filename) = path.file_name().and_then(|n| n.to_str()) {
                        if let Some(collection_name) = filename.strip_suffix("_vector_store.bin") {
                            // Skip if this collection is already loaded (from workspace)
                            if existing_collections.contains(collection_name) {
                                debug!(
                                    "Skipping collection '{}' — already loaded from workspace",
                                    collection_name
                                );
                                continue;
                            }

                            debug!("Loading dynamic collection: {}", collection_name);

                            match self.load_persisted_collection(&path, collection_name) {
                                Ok(_) => {
                                    // Enable graph for this collection automatically
                                    if let Err(e) =
                                        self.enable_graph_for_collection(collection_name)
                                    {
                                        warn!(
                                            "⚠️  Failed to enable graph for collection '{}': {} (continuing anyway)",
                                            collection_name, e
                                        );
                                    } else {
                                        info!(
                                            "✅ Graph enabled for collection '{}'",
                                            collection_name
                                        );
                                    }

                                    dynamic_collections_loaded += 1;
                                    info!(
                                        "✅ Loaded dynamic collection '{}' from persistence",
                                        collection_name
                                    );
                                }
                                Err(e) => {
                                    warn!(
                                        "❌ Failed to load dynamic collection '{}' from {:?}: {}",
                                        collection_name, path, e
                                    );
                                }
                            }
                        }
                    }
                }
            }
        }

        if dynamic_collections_loaded > 0 {
            info!(
                "📊 Loaded {} additional dynamic collections from persistence",
                dynamic_collections_loaded
            );
        }

        Ok(dynamic_collections_loaded)
    }

    /// Load a single persisted collection from file
    pub(super) fn load_persisted_collection<P: AsRef<std::path::Path>>(
        &self,
        path: P,
        collection_name: &str,
    ) -> Result<()> {
        use std::io::Read;

        use flate2::read::GzDecoder;

        use crate::persistence::PersistedVectorStore;

        let path = path.as_ref();
        debug!(
            "Loading persisted collection '{}' from {:?}",
            collection_name, path
        );

        // Read and parse the JSON file with compression support
        let (json_data, was_compressed) = match std::fs::File::open(path) {
            Ok(file) => {
                let mut decoder = GzDecoder::new(file);
                let mut json_string = String::new();

                // Try to decompress — if it fails, try reading as plain text
                match decoder.read_to_string(&mut json_string) {
                    Ok(_) => {
                        debug!("📦 Loaded compressed collection cache");
                        (json_string, true)
                    }
                    Err(_) => {
                        // Not a gzip file, try reading as plain text (backward compatibility)
                        debug!("📦 Loaded uncompressed collection cache");
                        (std::fs::read_to_string(path)?, false)
                    }
                }
            }
            Err(e) => {
                return Err(crate::error::VectorizerError::Other(format!(
                    "Failed to open file: {}",
                    e
                )));
            }
        };

        let persisted: PersistedVectorStore = serde_json::from_str(&json_data)?;

        // Check version
        if persisted.version != 1 {
            return Err(crate::error::VectorizerError::Other(format!(
                "Unsupported persisted collection version: {}",
                persisted.version
            )));
        }

        // Find the collection in the persisted data
        let persisted_collection = persisted
            .collections
            .iter()
            .find(|c| c.name == collection_name)
            .ok_or_else(|| {
                crate::error::VectorizerError::Other(format!(
                    "Collection '{}' not found in persisted data",
                    collection_name
                ))
            })?;

        // Create collection with the persisted config
        let mut config = persisted_collection.config.clone().unwrap_or_else(|| {
            debug!(
                "⚠️  Collection '{}' has no config, using default",
                collection_name
            );
            crate::models::CollectionConfig::default()
        });
        config.quantization = crate::models::QuantizationConfig::SQ { bits: 8 };

        self.create_collection_with_quantization(collection_name, config.clone())?;

        // Enable graph BEFORE loading vectors if graph is enabled in config
        if config.graph.as_ref().map(|g| g.enabled).unwrap_or(false) {
            if let Err(e) = self.enable_graph_for_collection(collection_name) {
                warn!(
                    "⚠️  Failed to enable graph for collection '{}' before loading vectors: {} (continuing anyway)",
                    collection_name, e
                );
            } else {
                info!(
                    "✅ Graph enabled for collection '{}' before loading vectors",
                    collection_name
                );
            }
        }

        // Load vectors if any exist
        if !persisted_collection.vectors.is_empty() {
            debug!(
                "Loading {} vectors into collection '{}'",
                persisted_collection.vectors.len(),
                collection_name
            );
            self.load_collection_from_cache(collection_name, persisted_collection.vectors.clone())?;
        }

        // If graph wasn't enabled before (config didn't have it), enable it now
        if !config.graph.as_ref().map(|g| g.enabled).unwrap_or(false) {
            if let Err(e) = self.enable_graph_for_collection(collection_name) {
                warn!(
                    "⚠️  Failed to enable graph for collection '{}': {} (continuing anyway)",
                    collection_name, e
                );
            } else {
                info!(
                    "✅ Graph enabled for collection '{}' (auto-enabled for workspace)",
                    collection_name
                );
            }
        }

        // Note: Auto-migration removed to prevent memory duplication
        // Uncompressed files will be saved compressed on next auto-save cycle
        if !was_compressed {
            info!(
                "📦 Loaded uncompressed cache for '{}' — will be saved compressed on next auto-save",
                collection_name
            );
        }

        Ok(())
    }
}
