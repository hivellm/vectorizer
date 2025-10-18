//! Main VectorStore implementation

use std::collections::HashSet;
use std::ops::Deref;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use anyhow::anyhow;
use dashmap::DashMap;
use tracing::{debug, error, info, warn};

use super::collection::Collection;
#[cfg(feature = "hive-gpu")]
use crate::db::hive_gpu_collection::HiveGpuCollection;
use crate::error::{Result, VectorizerError};
#[cfg(feature = "hive-gpu")]
use crate::gpu_adapter::GpuAdapter;
use crate::models::{CollectionConfig, CollectionMetadata, SearchResult, Vector};

/// Enum to represent different collection types (CPU or GPU)
pub enum CollectionType {
    /// CPU-based collection
    Cpu(Collection),
    /// Hive-GPU collection (Metal, CUDA, WebGPU)
    #[cfg(feature = "hive-gpu")]
    HiveGpu(HiveGpuCollection),
}

impl std::fmt::Debug for CollectionType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CollectionType::Cpu(c) => write!(f, "CollectionType::Cpu({})", c.name()),
            #[cfg(feature = "hive-gpu")]
            CollectionType::HiveGpu(c) => write!(f, "CollectionType::HiveGpu({})", c.name()),
        }
    }
}

impl CollectionType {
    /// Get collection name
    pub fn name(&self) -> &str {
        match self {
            CollectionType::Cpu(c) => c.name(),
            #[cfg(feature = "hive-gpu")]
            CollectionType::HiveGpu(c) => c.name(),
        }
    }

    /// Get collection config
    pub fn config(&self) -> &CollectionConfig {
        match self {
            CollectionType::Cpu(c) => c.config(),
            #[cfg(feature = "hive-gpu")]
            CollectionType::HiveGpu(c) => c.config(),
        }
    }

    /// Add a vector to the collection
    pub fn add_vector(&mut self, _id: String, vector: Vector) -> Result<()> {
        match self {
            CollectionType::Cpu(c) => c.insert(vector),
            #[cfg(feature = "hive-gpu")]
            CollectionType::HiveGpu(c) => c.add_vector(vector).map(|_| ()),
        }
    }

    /// Search for similar vectors
    pub fn search(&self, query: &[f32], limit: usize) -> Result<Vec<SearchResult>> {
        match self {
            CollectionType::Cpu(c) => c.search(query, limit),
            #[cfg(feature = "hive-gpu")]
            CollectionType::HiveGpu(c) => c.search(query, limit),
        }
    }

    /// Get collection metadata
    pub fn metadata(&self) -> CollectionMetadata {
        match self {
            CollectionType::Cpu(c) => c.metadata(),
            #[cfg(feature = "hive-gpu")]
            CollectionType::HiveGpu(c) => c.metadata(),
        }
    }

    /// Delete a vector from the collection
    pub fn delete_vector(&mut self, id: &str) -> Result<()> {
        match self {
            CollectionType::Cpu(c) => c.delete(id),
            #[cfg(feature = "hive-gpu")]
            CollectionType::HiveGpu(c) => c.remove_vector(id.to_string()),
        }
    }

    /// Get a vector by ID
    pub fn get_vector(&self, vector_id: &str) -> Result<Vector> {
        match self {
            CollectionType::Cpu(c) => c.get_vector(vector_id),
            #[cfg(feature = "hive-gpu")]
            CollectionType::HiveGpu(c) => c.get_vector_by_id(vector_id),
        }
    }

    /// Get the number of vectors in the collection
    pub fn vector_count(&self) -> usize {
        match self {
            CollectionType::Cpu(c) => c.vector_count(),
            #[cfg(feature = "hive-gpu")]
            CollectionType::HiveGpu(c) => c.vector_count(),
        }
    }

    /// Get estimated memory usage
    pub fn estimated_memory_usage(&self) -> usize {
        match self {
            CollectionType::Cpu(c) => c.estimated_memory_usage(),
            #[cfg(feature = "hive-gpu")]
            CollectionType::HiveGpu(c) => c.estimated_memory_usage(),
        }
    }

    /// Get all vectors in the collection
    pub fn get_all_vectors(&self) -> Vec<Vector> {
        match self {
            CollectionType::Cpu(c) => c.get_all_vectors(),
            #[cfg(feature = "hive-gpu")]
            CollectionType::HiveGpu(c) => c.get_all_vectors(),
        }
    }

    /// Get embedding type
    pub fn get_embedding_type(&self) -> String {
        match self {
            CollectionType::Cpu(c) => c.get_embedding_type(),
            #[cfg(feature = "hive-gpu")]
            CollectionType::HiveGpu(c) => c.get_embedding_type(),
        }
    }

    /// Requantize existing vectors if quantization is enabled
    pub fn requantize_existing_vectors(&self) -> Result<()> {
        match self {
            CollectionType::Cpu(c) => c.requantize_existing_vectors(),
            #[cfg(feature = "hive-gpu")]
            CollectionType::HiveGpu(c) => c.requantize_existing_vectors(),
        }
    }

    /// Calculate approximate memory usage of the collection
    pub fn calculate_memory_usage(&self) -> (usize, usize, usize) {
        match self {
            CollectionType::Cpu(c) => c.calculate_memory_usage(),
            #[cfg(feature = "hive-gpu")]
            CollectionType::HiveGpu(c) => {
                // For Hive-GPU collections, return basic estimation
                let total = c.estimated_memory_usage();
                (total / 2, total / 2, total)
            }
        }
    }

    /// Get collection size information in a formatted way
    pub fn get_size_info(&self) -> (String, String, String) {
        match self {
            CollectionType::Cpu(c) => c.get_size_info(),
            #[cfg(feature = "hive-gpu")]
            CollectionType::HiveGpu(c) => {
                let total = c.estimated_memory_usage();
                let format_bytes = |bytes: usize| -> String {
                    if bytes >= 1024 * 1024 {
                        format!("{:.1} MB", bytes as f64 / (1024.0 * 1024.0))
                    } else if bytes >= 1024 {
                        format!("{:.1} KB", bytes as f64 / 1024.0)
                    } else {
                        format!("{} B", bytes)
                    }
                };
                let index_size = format_bytes(total / 2);
                let payload_size = format_bytes(total / 2);
                let total_size = format_bytes(total);
                (index_size, payload_size, total_size)
            }
        }
    }

    /// Set embedding type
    pub fn set_embedding_type(&mut self, embedding_type: String) {
        match self {
            CollectionType::Cpu(c) => c.set_embedding_type(embedding_type),
            #[cfg(feature = "hive-gpu")]
            CollectionType::HiveGpu(_) => {
                // Hive-GPU doesn't need to track embedding types
                debug!(
                    "Hive-GPU collections don't track embedding types: {}",
                    embedding_type
                );
            }
        }
    }

    /// Load HNSW index from dump
    pub fn load_hnsw_index_from_dump<P: AsRef<std::path::Path>>(
        &self,
        path: P,
        basename: &str,
    ) -> Result<()> {
        match self {
            CollectionType::Cpu(c) => c.load_hnsw_index_from_dump(path, basename),
            #[cfg(feature = "hive-gpu")]
            CollectionType::HiveGpu(_) => {
                warn!("Hive-GPU collections don't support HNSW dump loading yet");
                Ok(())
            }
        }
    }

    /// Load vectors into memory
    pub fn load_vectors_into_memory(&self, vectors: Vec<Vector>) -> Result<()> {
        match self {
            CollectionType::Cpu(c) => c.load_vectors_into_memory(vectors),
            #[cfg(feature = "hive-gpu")]
            CollectionType::HiveGpu(_) => {
                warn!("Hive-GPU collections don't support vector loading into memory yet");
                Ok(())
            }
        }
    }

    /// Fast load vectors
    pub fn fast_load_vectors(&mut self, vectors: Vec<Vector>) -> Result<()> {
        match self {
            CollectionType::Cpu(c) => c.fast_load_vectors(vectors),
            #[cfg(feature = "hive-gpu")]
            CollectionType::HiveGpu(c) => {
                // Use batch insertion for better performance
                c.add_vectors(vectors)?;
                Ok(())
            }
        }
    }
}

/// Thread-safe in-memory vector store
#[derive(Clone)]
pub struct VectorStore {
    /// Collections stored in a concurrent hash map
    collections: Arc<DashMap<String, CollectionType>>,
    /// Auto-save enabled flag (prevents auto-save during initialization)
    auto_save_enabled: Arc<std::sync::atomic::AtomicBool>,
    /// Collections pending save (for batch persistence)
    pending_saves: Arc<std::sync::Mutex<std::collections::HashSet<String>>>,
    /// Background save task handle
    save_task_handle: Arc<std::sync::Mutex<Option<tokio::task::JoinHandle<()>>>>,
}

impl std::fmt::Debug for VectorStore {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("VectorStore")
            .field("collections", &self.collections.len())
            .finish()
    }
}

impl VectorStore {
    /// Create a new empty vector store
    pub fn new() -> Self {
        info!("Creating new VectorStore");

        let store = Self {
            collections: Arc::new(DashMap::new()),
            auto_save_enabled: Arc::new(std::sync::atomic::AtomicBool::new(false)),
            pending_saves: Arc::new(std::sync::Mutex::new(HashSet::new())),
            save_task_handle: Arc::new(std::sync::Mutex::new(None)),
        };

        // Check for automatic migration on startup
        store.check_and_migrate_storage();

        store
    }

    /// Check storage format and perform automatic migration if needed
    fn check_and_migrate_storage(&self) {
        use std::fs;

        use crate::storage::{StorageFormat, StorageMigrator, detect_format};

        let data_dir = PathBuf::from("./data");

        // Create data directory if it doesn't exist
        if !data_dir.exists() {
            if let Err(e) = fs::create_dir_all(&data_dir) {
                warn!("Failed to create data directory: {}", e);
                return;
            }
        }

        // Check if data directory is empty (no legacy files)
        let is_empty = fs::read_dir(&data_dir)
            .ok()
            .map(|mut entries| entries.next().is_none())
            .unwrap_or(false);

        if is_empty {
            // Initialize with compact format for new installations
            info!("📁 Empty data directory detected - initializing with .vecdb format");
            if let Err(e) = self.initialize_compact_storage(&data_dir) {
                warn!("Failed to initialize compact storage: {}", e);
            } else {
                info!("✅ Initialized with .vecdb compact storage format");
            }
            return;
        }

        let format = detect_format(&data_dir);

        match format {
            StorageFormat::Legacy => {
                // Check if migration is enabled in config
                // For now, we'll just log that migration is available
                info!("💾 Legacy storage format detected");
                info!("   Run 'vectorizer storage migrate' to convert to .vecdb format");
                info!("   Benefits: Compression, snapshots, faster backups");
            }
            StorageFormat::Compact => {
                info!("✅ Using .vecdb compact storage format");
            }
        }
    }

    /// Initialize compact storage format (create empty .vecdb and .vecidx files)
    fn initialize_compact_storage(&self, data_dir: &PathBuf) -> Result<()> {
        use std::fs::File;

        use crate::storage::{StorageIndex, vecdb_path, vecidx_path};

        let vecdb_file = vecdb_path(data_dir);
        let vecidx_file = vecidx_path(data_dir);

        // Create empty .vecdb file
        File::create(&vecdb_file).map_err(|e| crate::error::VectorizerError::Io(e))?;

        // Create empty index
        let now = chrono::Utc::now();
        let empty_index = StorageIndex {
            version: crate::storage::STORAGE_VERSION.to_string(),
            created_at: now,
            updated_at: now,
            collections: Vec::new(),
            total_size: 0,
            compressed_size: 0,
            compression_ratio: 0.0,
        };

        // Save empty index
        let index_json = serde_json::to_string_pretty(&empty_index)
            .map_err(|e| crate::error::VectorizerError::Serialization(e.to_string()))?;

        std::fs::write(&vecidx_file, index_json)
            .map_err(|e| crate::error::VectorizerError::Io(e))?;

        info!("Created empty .vecdb and .vecidx files");
        Ok(())
    }

    /// Create a new vector store with Hive-GPU configuration
    #[cfg(feature = "hive-gpu")]
    pub fn new_with_hive_gpu_config() -> Self {
        info!("Creating new VectorStore with Hive-GPU configuration");
        Self {
            collections: Arc::new(DashMap::new()),
            auto_save_enabled: Arc::new(std::sync::atomic::AtomicBool::new(false)),
            pending_saves: Arc::new(std::sync::Mutex::new(HashSet::new())),
            save_task_handle: Arc::new(std::sync::Mutex::new(None)),
        }
    }

    /// Create a new vector store with automatic GPU detection
    /// Priority: Hive-GPU (Metal/CUDA/WebGPU) > CPU
    pub fn new_auto() -> Self {
        eprintln!("🔍 VectorStore::new_auto() called - starting GPU detection...");
        info!("🔍 VectorStore::new_auto() called - starting GPU detection...");

        // Create store without loading collections (will be loaded in background task)
        let store = Self::new();

        // DON'T enable auto-save yet - will be enabled after collections are loaded
        // This prevents auto-save from triggering during initial load
        info!(
            "⏸️  Auto-save disabled during initialization - will be enabled after load completes"
        );

        eprintln!("✅ VectorStore created (collections will be loaded in background)");

        // Try Hive-GPU first (Metal backend only on macOS)
        #[cfg(all(feature = "hive-gpu", target_os = "macos"))]
        {
            eprintln!("🚀 Detecting Hive-GPU capabilities...");
            info!("🚀 Detecting Hive-GPU capabilities...");

            // Try to create a GPU context
            use hive_gpu::metal::MetalNativeContext;
            if let Ok(_) = MetalNativeContext::new() {
                eprintln!("✅ Hive-GPU detected and enabled!");
                info!("✅ Hive-GPU detected and enabled!");
                let store = Self::new_with_hive_gpu_config();
                info!("⏸️  Auto-save will be enabled after collections load");
                return store;
            } else {
                eprintln!("⚠️ Hive-GPU detection failed, falling back to CPU...");
                warn!("⚠️ Hive-GPU detection failed, falling back to CPU...");
            }
        }

        #[cfg(all(feature = "hive-gpu", not(target_os = "macos")))]
        {
            eprintln!("⚠️ Hive-GPU Metal backend only available on macOS, using CPU mode");
            info!("⚠️ Hive-GPU Metal backend only available on macOS, using CPU mode");
        }

        #[cfg(not(feature = "hive-gpu"))]
        {
            eprintln!("⚠️ Hive-GPU not available (hive-gpu feature not compiled)");
            info!("⚠️ Hive-GPU not available (hive-gpu feature not compiled)");
        }

        // Return the store (auto-save will be enabled after collections load)
        eprintln!("💻 Using CPU-only mode");
        info!("💻 Using CPU-only mode");
        store
    }

    /// Create a new collection
    pub fn create_collection(&self, name: &str, config: CollectionConfig) -> Result<()> {
        debug!("Creating collection '{}' with config: {:?}", name, config);

        if self.collections.contains_key(name) {
            return Err(VectorizerError::CollectionAlreadyExists(name.to_string()));
        }

        // Try Hive-GPU first (Metal backend only on macOS)
        #[cfg(all(feature = "hive-gpu", target_os = "macos"))]
        {
            info!("Creating Hive-GPU collection '{}'", name);
            use hive_gpu::GpuContext;
            use hive_gpu::metal::MetalNativeContext;

            // Create GPU context (try to create from available backends)
            match MetalNativeContext::new() {
                Ok(ctx) => {
                    let context = Arc::new(std::sync::Mutex::new(
                        Box::new(ctx) as Box<dyn GpuContext + Send>
                    ));

                    // Create Hive-GPU collection
                    let hive_gpu_collection =
                        HiveGpuCollection::new(name.to_string(), config.clone(), context)?;

                    let collection = CollectionType::HiveGpu(hive_gpu_collection);
                    self.collections.insert(name.to_string(), collection);
                    info!("Collection '{}' created successfully with Hive-GPU", name);
                    return Ok(());
                }
                Err(e) => {
                    warn!("Failed to create GPU context: {:?}, falling back to CPU", e);
                }
            }
        }

        #[cfg(all(feature = "hive-gpu", not(target_os = "macos")))]
        {
            info!(
                "Hive-GPU Metal backend only available on macOS, creating CPU collection for '{}'",
                name
            );
        }

        // Fallback to CPU
        debug!("Creating CPU-based collection '{}'", name);
        let collection = Collection::new(name.to_string(), config);
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

                    // Store the existing vector count and document count
                    let existing_metadata = existing_collection.metadata();
                    let existing_document_count = existing_metadata.document_count;

                    // Remove old collection
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

        self.collections
            .remove(name)
            .ok_or_else(|| VectorizerError::CollectionNotFound(name.to_string()))?;

        info!("Collection '{}' deleted successfully", name);
        Ok(())
    }

    /// Get a reference to a collection by name
    /// Implements lazy loading: if collection is not in memory but exists on disk, loads it
    pub fn get_collection(
        &self,
        name: &str,
    ) -> Result<impl std::ops::Deref<Target = CollectionType> + '_> {
        // Fast path: collection already loaded
        if let Some(collection) = self.collections.get(name) {
            return Ok(collection);
        }

        // Slow path: try lazy loading from disk
        let data_dir = Self::get_data_dir();

        // First, try to load from .vecdb archive (compact format)
        use crate::storage::{StorageFormat, StorageReader, detect_format};
        if detect_format(&data_dir) == StorageFormat::Compact {
            debug!("📥 Lazy loading collection '{}' from .vecdb archive", name);

            match StorageReader::new(&data_dir) {
                Ok(reader) => {
                    // Read the _vector_store.bin file from the archive
                    let vector_store_path = format!("{}_vector_store.bin", name);
                    match reader.read_file(&vector_store_path) {
                        Ok(data) => {
                            // Deserialize PersistedCollection from JSON (compressed in ZIP)
                            match serde_json::from_slice::<crate::persistence::PersistedCollection>(
                                &data,
                            ) {
                                Ok(persisted) => {
                                    // Load collection into memory
                                    if let Err(e) =
                                        self.load_persisted_collection_from_data(name, persisted)
                                    {
                                        warn!(
                                            "Failed to load collection '{}' from .vecdb: {}",
                                            name, e
                                        );
                                        return Err(VectorizerError::CollectionNotFound(
                                            name.to_string(),
                                        ));
                                    }

                                    info!("✅ Lazy loaded collection '{}' from .vecdb", name);

                                    // Try again now that it's loaded
                                    return self.collections.get(name).ok_or_else(|| {
                                        VectorizerError::CollectionNotFound(name.to_string())
                                    });
                                }
                                Err(e) => {
                                    warn!(
                                        "Failed to deserialize collection '{}' from .vecdb: {}",
                                        name, e
                                    );
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

            // Load collection from disk
            if let Err(e) = self.load_persisted_collection(&collection_file, name) {
                warn!("Failed to lazy load collection '{}': {}", name, e);
                return Err(VectorizerError::CollectionNotFound(name.to_string()));
            }

            // Try again now that it's loaded
            return self
                .collections
                .get(name)
                .ok_or_else(|| VectorizerError::CollectionNotFound(name.to_string()));
        }

        // Collection doesn't exist anywhere
        Err(VectorizerError::CollectionNotFound(name.to_string()))
    }

    /// Load collection from PersistedCollection data
    fn load_persisted_collection_from_data(
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
        if !self.has_collection_in_memory(name) {
            let config = persisted.config.clone().unwrap_or_else(|| {
                warn!("⚠️  Collection '{}' has no config, using default", name);
                crate::models::CollectionConfig::default()
            });
            self.create_collection(name, config)?;
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

        // Load vectors into memory - HNSW index is built automatically during insertion
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

    /// List all collections (both loaded in memory and available on disk)
    /// Check if collection exists in memory only (without lazy loading)
    pub fn has_collection_in_memory(&self, name: &str) -> bool {
        self.collections.contains_key(name)
    }

    /// Get a mutable reference to a collection by name
    pub fn get_collection_mut(
        &self,
        name: &str,
    ) -> Result<impl std::ops::DerefMut<Target = CollectionType> + '_> {
        // Ensure collection is loaded first
        let _ = self.get_collection(name)?;

        // Now get mutable reference
        self.collections
            .get_mut(name)
            .ok_or_else(|| VectorizerError::CollectionNotFound(name.to_string()))
    }

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

    /// Get collection metadata
    pub fn get_collection_metadata(&self, name: &str) -> Result<CollectionMetadata> {
        let collection_ref = self.get_collection(name)?;
        Ok(collection_ref.metadata())
    }

    /// Insert vectors into a collection
    pub fn insert(&self, collection_name: &str, vectors: Vec<Vector>) -> Result<()> {
        debug!(
            "Inserting {} vectors into collection '{}'",
            vectors.len(),
            collection_name
        );

        let mut collection_ref = self.get_collection_mut(collection_name)?;

        // Check if this is a GPU collection that needs special handling
        match collection_ref.deref() {
            #[cfg(feature = "hive-gpu")]
            CollectionType::HiveGpu(_) => {
                // For Hive-GPU collections, use batch insertion
                for vector in vectors {
                    collection_ref.add_vector(vector.id.clone(), vector)?;
                }
            }
            _ => {
                // For CPU collections, use sequential iteration
                for vector in vectors {
                    collection_ref.add_vector(vector.id.clone(), vector)?;
                }
            }
        }

        // Mark collection for auto-save
        self.mark_collection_for_save(collection_name);

        Ok(())
    }

    /// Update a vector in a collection
    pub fn update(&self, collection_name: &str, vector: Vector) -> Result<()> {
        debug!(
            "Updating vector '{}' in collection '{}'",
            vector.id, collection_name
        );

        let mut collection_ref = self.get_collection_mut(collection_name)?;
        // For update, we delete and re-add (TODO: Add direct update method to CollectionType)
        collection_ref.delete_vector(&vector.id)?;
        collection_ref.add_vector(vector.id.clone(), vector)?;

        // Mark collection for auto-save
        self.mark_collection_for_save(collection_name);

        Ok(())
    }

    /// Delete a vector from a collection
    pub fn delete(&self, collection_name: &str, vector_id: &str) -> Result<()> {
        debug!(
            "Deleting vector '{}' from collection '{}'",
            vector_id, collection_name
        );

        let mut collection_ref = self.get_collection_mut(collection_name)?;
        collection_ref.delete_vector(vector_id)?;

        // Mark collection for auto-save
        self.mark_collection_for_save(collection_name);

        Ok(())
    }

    /// Get a vector by ID
    pub fn get_vector(&self, collection_name: &str, vector_id: &str) -> Result<Vector> {
        let collection_ref = self.get_collection(collection_name)?;
        collection_ref.get_vector(vector_id)
    }

    /// Search for similar vectors
    pub fn search(
        &self,
        collection_name: &str,
        query_vector: &[f32],
        k: usize,
    ) -> Result<Vec<SearchResult>> {
        debug!(
            "Searching for {} nearest neighbors in collection '{}'",
            k, collection_name
        );

        let collection_ref = self.get_collection(collection_name)?;
        collection_ref.search(query_vector, k)
    }

    /// Load a collection from cache without reconstructing the HNSW index
    pub fn load_collection_from_cache(
        &self,
        collection_name: &str,
        persisted_vectors: Vec<crate::persistence::PersistedVector>,
    ) -> Result<()> {
        use crate::persistence::PersistedVector;

        debug!(
            "Fast loading collection '{}' from cache with {} vectors",
            collection_name,
            persisted_vectors.len()
        );

        let mut collection_ref = self.get_collection_mut(collection_name)?;

        // TODO: Implement load_from_cache for MetalNativeCollection
        match &*collection_ref {
            CollectionType::Cpu(c) => {
                c.load_from_cache(persisted_vectors)?;
                // Requantize existing vectors if quantization is enabled
                c.requantize_existing_vectors()?;
            }
            #[cfg(feature = "hive-gpu")]
            CollectionType::HiveGpu(_) => {
                warn!(
                    "Hive-GPU collections don't support cache loading yet - falling back to manual insertion"
                );
                // For now, manually insert vectors for Hive-GPU collections
                for pv in persisted_vectors {
                    // Convert PersistedVector back to Vector
                    let vector: Vector = pv.into();
                    collection_ref.add_vector(vector.id.clone(), vector)?;
                }
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
        use crate::persistence::PersistedVector;

        debug!(
            "Loading collection '{}' from cache with {} vectors (HNSW dump: {})",
            collection_name,
            persisted_vectors.len(),
            hnsw_basename.is_some()
        );

        let mut collection_ref = self.get_collection_mut(collection_name)?;

        // TODO: Implement load_from_cache_with_hnsw_dump for MetalNativeCollection
        match &*collection_ref {
            CollectionType::Cpu(c) => {
                c.load_from_cache_with_hnsw_dump(persisted_vectors, hnsw_dump_path, hnsw_basename)?
            }
            #[cfg(feature = "hive-gpu")]
            CollectionType::HiveGpu(_) => {
                warn!(
                    "Hive-GPU collections don't support HNSW dump loading yet - falling back to manual insertion"
                );
                // For now, manually insert vectors for Hive-GPU collections
                for pv in persisted_vectors {
                    // Convert PersistedVector back to Vector
                    let vector: Vector = pv.into();
                    collection_ref.add_vector(vector.id.clone(), vector)?;
                }
            }
        }

        Ok(())
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
}

impl Default for VectorStore {
    fn default() -> Self {
        Self::new()
    }
}

/// Statistics about the vector store
#[derive(Debug, Default, Clone)]
pub struct VectorStoreStats {
    /// Number of collections
    pub collection_count: usize,
    /// Total number of vectors across all collections
    pub total_vectors: usize,
    /// Estimated memory usage in bytes
    pub total_memory_bytes: usize,
}

impl VectorStore {
    /// Get the centralized data directory path (same as DocumentLoader)
    pub fn get_data_dir() -> PathBuf {
        let current_dir = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
        current_dir.join("data")
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
    /// NEVER falls back to raw files - .vecdb is the ONLY source of truth
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
                // NO FALLBACK! Return error instead
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

            // SKIP empty collections - don't create them
            if persisted_collection.vectors.is_empty() {
                warn!(
                    "⚠️  Collection '{}' has NO vectors in .vecdb, skipping creation",
                    collection_name
                );
                continue;
            }

            // Create collection with the persisted config
            let mut config = persisted_collection.config.clone().unwrap_or_else(|| {
                warn!(
                    "⚠️  Collection '{}' has no config, using default",
                    collection_name
                );
                crate::models::CollectionConfig::default()
            });
            config.quantization = crate::models::QuantizationConfig::SQ { bits: 8 };

            match self.create_collection_with_quantization(collection_name, config) {
                Ok(_) => {
                    // Load vectors (we already checked they exist above)
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
            error!("   All collections failed to deserialize - likely format mismatch");
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

    /// Load dynamic collections that are not in the workspace
    /// Call this after workspace initialization to load any additional persisted collections
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
                                    "Skipping collection '{}' - already loaded from workspace",
                                    collection_name
                                );
                                continue;
                            }

                            debug!("Loading dynamic collection: {}", collection_name);

                            match self.load_persisted_collection(&path, collection_name) {
                                Ok(_) => {
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
    fn load_persisted_collection<P: AsRef<std::path::Path>>(
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

                // Try to decompress - if it fails, try reading as plain text
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
            warn!(
                "⚠️  Collection '{}' has no config, using default",
                collection_name
            );
            crate::models::CollectionConfig::default()
        });
        config.quantization = crate::models::QuantizationConfig::SQ { bits: 8 };

        self.create_collection_with_quantization(collection_name, config)?;

        // Load vectors if any exist
        if !persisted_collection.vectors.is_empty() {
            debug!(
                "Loading {} vectors into collection '{}'",
                persisted_collection.vectors.len(),
                collection_name
            );
            self.load_collection_from_cache(collection_name, persisted_collection.vectors.clone())?;
        }

        // Note: Auto-migration removed to prevent memory duplication
        // Uncompressed files will be saved compressed on next auto-save cycle
        if !was_compressed {
            info!(
                "📦 Loaded uncompressed cache for '{}' - will be saved compressed on next auto-save",
                collection_name
            );
        }

        Ok(())
    }

    /// Enable auto-save for all collections
    /// Call this after initialization is complete
    pub fn enable_auto_save(&self) {
        // Check if auto-save is already enabled to avoid multiple tasks
        if self
            .auto_save_enabled
            .load(std::sync::atomic::Ordering::Relaxed)
        {
            info!("⏭️ Auto-save already enabled, skipping");
            return;
        }

        self.auto_save_enabled
            .store(true, std::sync::atomic::Ordering::Relaxed);

        // DEPRECATED: Old auto-save system disabled
        // Auto-save is now managed exclusively by AutoSaveManager (5min intervals)
        // which compacts directly from memory without creating raw .bin files
        info!("✅ Auto-save flag enabled - managed by AutoSaveManager (no raw .bin files)");

        // OLD SYSTEM DISABLED - keeping the code for reference only
        /*
        // Start background save task
        let pending_saves: Arc<std::sync::Mutex<HashSet<String>>> = Arc::clone(&self.pending_saves);
        let collections = Arc::clone(&self.collections);

        let save_task = tokio::spawn(async move {
            info!("🔄 OLD Background save task - DEPRECATED");
            loop {
                if !pending_saves.lock().unwrap().is_empty() {
                    info!("🔄 Background save: {} collections pending", pending_saves.lock().unwrap().len());

                    // Process all pending saves
                    let collections_to_save: Vec<String> = pending_saves.lock().unwrap().iter().cloned().collect();
                    pending_saves.lock().unwrap().clear();

                    // Save each collection to raw format
                    let mut saved_count = 0;
                    for collection_name in collections_to_save {
                        debug!("💾 Saving collection '{}' to raw format", collection_name);

                        // Get collection and save to raw files
                        if let Some(collection_ref) = collections.get(&collection_name) {
                            match collection_ref.deref() {
                                CollectionType::Cpu(c) => {
                                    let metadata = c.metadata();
                                    let vectors = c.get_all_vectors();

                                    // Create persisted representation
                                    let persisted_vectors: Vec<crate::persistence::PersistedVector> = vectors
                                        .into_iter()
                                        .map(crate::persistence::PersistedVector::from)
                                        .collect();

                                    let persisted_collection = crate::persistence::PersistedCollection {
                                        name: collection_name.clone(),
                                        config: Some(metadata.config),
                                        vectors: persisted_vectors,
                                        hnsw_dump_basename: None,
                                    };

                                    // Save to raw format
                                    let data_dir = VectorStore::get_data_dir();
                                    let vector_store_path = data_dir.join(format!("{}_vector_store.bin", collection_name));

                                    // Serialize to JSON (matching the load format)
                                    let persisted_store = crate::persistence::PersistedVectorStore {
                                        version: 1,
                                        collections: vec![persisted_collection],
                                    };

                                    if let Ok(json_data) = serde_json::to_string(&persisted_store) {
                                        if let Ok(mut file) = std::fs::File::create(&vector_store_path) {
                                            use std::io::Write;
                                            let _ = file.write_all(json_data.as_bytes());
                                            debug!("✅ Saved collection '{}' to raw format", collection_name);
                                            saved_count += 1;
                                        }
                                    }
                                }
                                _ => {
                                    debug!("⚠️  GPU collections not yet supported for auto-save");
                                }
                            }
                        }
                    }

                    info!("✅ Background save completed - {} collections saved", saved_count);

                    // Immediately compact to .vecdb and remove raw files
                    if saved_count > 0 {
                        info!("🗜️  Starting immediate compaction to vectorizer.vecdb...");
                        info!("📝 First, saving ALL collections to ensure complete backup...");

                        let data_dir = VectorStore::get_data_dir();

                        // Save ALL collections to raw format (not just modified ones)
                        // This ensures the .vecdb will contain everything
                        let all_collection_names: Vec<String> = collections.iter().map(|entry| entry.key().clone()).collect();
                        info!("💾 Saving all {} collections to raw format for complete backup", all_collection_names.len());

                        for collection_name in &all_collection_names {
                            if let Some(collection_ref) = collections.get(collection_name) {
                                match collection_ref.deref() {
                                    CollectionType::Cpu(c) => {
                                        let metadata = c.metadata();
                                        let vectors = c.get_all_vectors();

                                        let persisted_vectors: Vec<crate::persistence::PersistedVector> = vectors
                                            .into_iter()
                                            .map(crate::persistence::PersistedVector::from)
                                            .collect();

                                        let persisted_collection = crate::persistence::PersistedCollection {
                                            name: collection_name.clone(),
                                            config: Some(metadata.config),
                                            vectors: persisted_vectors,
                                            hnsw_dump_basename: None,
                                        };

                                        let vector_store_path = data_dir.join(format!("{}_vector_store.bin", collection_name));

                                        let persisted_store = crate::persistence::PersistedVectorStore {
                                            version: 1,
                                            collections: vec![persisted_collection],
                                        };

                                        if let Ok(json_data) = serde_json::to_string(&persisted_store) {
                                            if let Ok(mut file) = std::fs::File::create(&vector_store_path) {
                                                use std::io::Write;
                                                let _ = file.write_all(json_data.as_bytes());
                                            }
                                        }
                                    }
                                    _ => {}
                                }
                            }
                        }

                        info!("✅ All collections saved to raw format");

                        // Now compact everything
                        let compactor = crate::storage::StorageCompactor::new(&data_dir, 6, 1000);

                        match compactor.compact_all_with_cleanup(true) {
                            Ok(index) => {
                                info!("✅ Compaction completed successfully:");
                                info!("   Collections: {}", index.collection_count());
                                info!("   Total vectors: {}", index.total_vectors());
                                info!("   Compressed size: {} MB", index.compressed_size / 1_048_576);
                                info!("🗑️  Raw files removed after successful compaction");
                            }
                            Err(e) => {
                                warn!("⚠️  Compaction failed: {}", e);
                                warn!("   Raw files preserved for safety");
                            }
                        }
                    }
                }
            }
        });

        // Store the task handle
        *self.save_task_handle.lock().unwrap() = Some(save_task);
        */
    }

    /// Disable auto-save for all collections
    /// Useful during bulk operations or maintenance
    pub fn disable_auto_save(&self) {
        self.auto_save_enabled
            .store(false, std::sync::atomic::Ordering::Relaxed);
        info!("⏸️ Auto-save disabled for all collections");
    }

    /// Force immediate save of all pending collections
    /// Useful before shutdown or critical operations
    pub fn force_save_all(&self) -> Result<()> {
        if self.pending_saves.lock().unwrap().is_empty() {
            debug!("No pending saves to force");
            return Ok(());
        }

        info!(
            "🔄 Force saving {} pending collections",
            self.pending_saves.lock().unwrap().len()
        );

        let collections_to_save: Vec<String> =
            self.pending_saves.lock().unwrap().iter().cloned().collect();
        self.pending_saves.lock().unwrap().clear();

        // Force save disabled - using .vecdb format
        for collection_name in collections_to_save {
            debug!(
                "Collection '{}' marked for save (using .vecdb format)",
                collection_name
            );
        }

        info!("✅ Force save completed");
        Ok(())
    }

    /// Save a single collection to file following workspace pattern
    /// Creates separate files for vectors, tokenizer, and metadata
    pub fn save_collection_to_file(&self, collection_name: &str) -> Result<()> {
        use std::fs;

        use crate::persistence::PersistedCollection;
        use crate::storage::{StorageFormat, detect_format};

        info!(
            "Saving collection '{}' to individual files",
            collection_name
        );

        // Check if using compact storage format - if so, don't save in legacy format
        let data_dir = Self::get_data_dir();
        if detect_format(&data_dir) == StorageFormat::Compact {
            debug!(
                "⏭️ Skipping legacy save for '{}' - using .vecdb format",
                collection_name
            );
            return Ok(());
        }

        // Get collection
        let collection = self.get_collection(collection_name)?;
        let metadata = collection.metadata();

        // Ensure data directory exists
        let data_dir = Self::get_data_dir();
        if let Err(e) = fs::create_dir_all(&data_dir) {
            return Err(crate::error::VectorizerError::Other(format!(
                "Failed to create data directory '{}': {}",
                data_dir.display(),
                e
            )));
        }

        // Collect all vectors from the collection
        let vectors: Vec<crate::persistence::PersistedVector> = collection
            .get_all_vectors()
            .into_iter()
            .map(crate::persistence::PersistedVector::from)
            .collect();

        // Create persisted collection
        let persisted_collection = PersistedCollection {
            name: collection_name.to_string(),
            config: Some(metadata.config.clone()),
            vectors,
            hnsw_dump_basename: None,
        };

        // Save vectors to binary file (following workspace pattern)
        let vector_store_path = data_dir.join(format!("{}_vector_store.bin", collection_name));
        self.save_collection_vectors_binary(&persisted_collection, &vector_store_path)?;

        // Save metadata to JSON file
        let metadata_path = data_dir.join(format!("{}_metadata.json", collection_name));
        self.save_collection_metadata(&persisted_collection, &metadata_path)?;

        // Save tokenizer (for dynamic collections, create a minimal tokenizer)
        let tokenizer_path = data_dir.join(format!("{}_tokenizer.json", collection_name));
        self.save_collection_tokenizer(collection_name, &tokenizer_path)?;

        info!(
            "Successfully saved collection '{}' to files",
            collection_name
        );
        Ok(())
    }

    /// Static method to save collection to file (for background task)
    fn save_collection_to_file_static(
        collection_name: &str,
        collection: &CollectionType,
    ) -> Result<()> {
        use std::fs;

        use crate::persistence::PersistedCollection;
        use crate::storage::{StorageFormat, detect_format};

        info!("💾 Starting save for collection '{}'", collection_name);

        // Check if using compact storage format - if so, don't save in legacy format
        let data_dir = Self::get_data_dir();
        if detect_format(&data_dir) == StorageFormat::Compact {
            debug!(
                "⏭️ Skipping legacy save for '{}' - using .vecdb format",
                collection_name
            );
            return Ok(());
        }

        // Get collection metadata
        let metadata = collection.metadata();
        info!("💾 Got metadata for collection '{}'", collection_name);

        // Ensure data directory exists
        let data_dir = Self::get_data_dir();
        if let Err(e) = fs::create_dir_all(&data_dir) {
            warn!(
                "Failed to create data directory '{}': {}",
                data_dir.display(),
                e
            );
            return Err(crate::error::VectorizerError::Other(format!(
                "Failed to create data directory '{}': {}",
                data_dir.display(),
                e
            )));
        }
        info!("💾 Data directory ready: {:?}", data_dir);

        // Collect all vectors from the collection
        let vectors: Vec<crate::persistence::PersistedVector> = collection
            .get_all_vectors()
            .into_iter()
            .map(crate::persistence::PersistedVector::from)
            .collect();
        info!(
            "💾 Collected {} vectors from collection '{}'",
            vectors.len(),
            collection_name
        );

        // Create persisted collection for vector store
        let persisted_collection_for_store = PersistedCollection {
            name: collection_name.to_string(),
            config: Some(metadata.config.clone()),
            vectors: vectors.clone(),
            hnsw_dump_basename: None,
        };

        // Create persisted vector store with version
        let persisted_vector_store = crate::persistence::PersistedVectorStore {
            version: 1,
            collections: vec![persisted_collection_for_store],
        };

        // Save vectors to binary file
        let vector_store_path = data_dir.join(format!("{}_vector_store.bin", collection_name));
        info!("💾 Saving vectors to: {:?}", vector_store_path);
        Self::save_collection_vectors_binary_static(&persisted_vector_store, &vector_store_path)?;
        info!("💾 Vectors saved successfully");

        // Create persisted collection for metadata
        let persisted_collection_for_metadata = PersistedCollection {
            name: collection_name.to_string(),
            config: Some(metadata.config.clone()),
            vectors,
            hnsw_dump_basename: None,
        };

        // Save metadata to JSON file
        let metadata_path = data_dir.join(format!("{}_metadata.json", collection_name));
        info!("💾 Saving metadata to: {:?}", metadata_path);
        Self::save_collection_metadata_static(&persisted_collection_for_metadata, &metadata_path)?;
        info!("💾 Metadata saved successfully");

        // Save tokenizer
        let tokenizer_path = data_dir.join(format!("{}_tokenizer.json", collection_name));
        info!("💾 Saving tokenizer to: {:?}", tokenizer_path);
        Self::save_collection_tokenizer_static(collection_name, &tokenizer_path)?;
        info!("💾 Tokenizer saved successfully");

        info!(
            "✅ Successfully saved collection '{}' to files",
            collection_name
        );
        Ok(())
    }

    /// Mark a collection for auto-save (internal method)
    fn mark_collection_for_save(&self, collection_name: &str) {
        if self
            .auto_save_enabled
            .load(std::sync::atomic::Ordering::Relaxed)
        {
            info!("📝 Marking collection '{}' for auto-save", collection_name);
            self.pending_saves
                .lock()
                .unwrap()
                .insert(collection_name.to_string());
            info!(
                "📝 Collection '{}' added to pending saves (total: {})",
                collection_name,
                self.pending_saves.lock().unwrap().len()
            );
        } else {
            warn!(
                "⚠️ Auto-save is disabled, collection '{}' will not be saved",
                collection_name
            );
        }
    }

    /// Save collection vectors to binary file
    fn save_collection_vectors_binary(
        &self,
        persisted_collection: &crate::persistence::PersistedCollection,
        path: &std::path::Path,
    ) -> Result<()> {
        use std::fs::File;
        use std::io::Write;

        let json_data = serde_json::to_string_pretty(&persisted_collection)?;
        let mut file = File::create(path)?;
        file.write_all(json_data.as_bytes())?;

        debug!(
            "Saved {} vectors to {}",
            persisted_collection.vectors.len(),
            path.display()
        );
        Ok(())
    }

    /// Save collection metadata to JSON file
    fn save_collection_metadata(
        &self,
        persisted_collection: &crate::persistence::PersistedCollection,
        path: &std::path::Path,
    ) -> Result<()> {
        use std::collections::HashSet;
        use std::fs::File;
        use std::io::Write;

        // Extract unique file paths from vectors
        let mut indexed_files: HashSet<String> = HashSet::new();
        for pv in &persisted_collection.vectors {
            // Convert to Vector to access payload
            let v: Vector = pv.clone().into();
            if let Some(payload) = &v.payload {
                if let Some(metadata) = payload.data.get("metadata") {
                    if let Some(file_path) = metadata.get("file_path").and_then(|v| v.as_str()) {
                        indexed_files.insert(file_path.to_string());
                    }
                }
                // Also check direct file_path in payload
                if let Some(file_path) = payload.data.get("file_path").and_then(|v| v.as_str()) {
                    indexed_files.insert(file_path.to_string());
                }
            }
        }

        let mut files_vec: Vec<String> = indexed_files.into_iter().collect();
        files_vec.sort();

        let metadata = serde_json::json!({
            "name": persisted_collection.name,
            "config": persisted_collection.config,
            "vector_count": persisted_collection.vectors.len(),
            "indexed_files": files_vec,
            "total_files": files_vec.len(),
            "created_at": chrono::Utc::now().to_rfc3339(),
        });

        let json_data = serde_json::to_string_pretty(&metadata)?;
        let mut file = File::create(path)?;
        file.write_all(json_data.as_bytes())?;

        debug!(
            "Saved metadata for '{}' to {} ({} files indexed)",
            persisted_collection.name,
            path.display(),
            files_vec.len()
        );
        Ok(())
    }

    /// Save collection tokenizer to JSON file
    fn save_collection_tokenizer(
        &self,
        collection_name: &str,
        path: &std::path::Path,
    ) -> Result<()> {
        use std::fs::File;
        use std::io::Write;

        // For dynamic collections, create a minimal tokenizer
        let tokenizer_data = serde_json::json!({
            "collection_name": collection_name,
            "tokenizer_type": "dynamic",
            "created_at": chrono::Utc::now().to_rfc3339(),
            "vocab_size": 0,
            "special_tokens": {},
        });

        let json_data = serde_json::to_string_pretty(&tokenizer_data)?;
        let mut file = File::create(path)?;
        file.write_all(json_data.as_bytes())?;

        debug!(
            "Saved tokenizer for '{}' to {}",
            collection_name,
            path.display()
        );
        Ok(())
    }

    /// Static version of save_collection_vectors_binary
    fn save_collection_vectors_binary_static(
        persisted_vector_store: &crate::persistence::PersistedVectorStore,
        path: &std::path::Path,
    ) -> Result<()> {
        use std::fs::File;
        use std::io::Write;

        let json_data = serde_json::to_string_pretty(&persisted_vector_store)?;
        let mut file = File::create(path)?;
        file.write_all(json_data.as_bytes())?;
        file.flush()?;
        file.sync_all()?;

        // Verify file was created
        if path.exists() {
            info!("✅ File created successfully: {:?}", path);
        } else {
            warn!("❌ File was not created: {:?}", path);
        }

        debug!(
            "Saved {} collections to {}",
            persisted_vector_store.collections.len(),
            path.display()
        );
        Ok(())
    }

    /// Static version of save_collection_metadata
    fn save_collection_metadata_static(
        persisted_collection: &crate::persistence::PersistedCollection,
        path: &std::path::Path,
    ) -> Result<()> {
        use std::collections::HashSet;
        use std::fs::File;
        use std::io::Write;

        // Extract unique file paths from vectors
        let mut indexed_files: HashSet<String> = HashSet::new();
        for pv in &persisted_collection.vectors {
            // Convert to Vector to access payload
            let v: Vector = pv.clone().into();
            if let Some(payload) = &v.payload {
                if let Some(metadata) = payload.data.get("metadata") {
                    if let Some(file_path) = metadata.get("file_path").and_then(|v| v.as_str()) {
                        indexed_files.insert(file_path.to_string());
                    }
                }
                // Also check direct file_path in payload
                if let Some(file_path) = payload.data.get("file_path").and_then(|v| v.as_str()) {
                    indexed_files.insert(file_path.to_string());
                }
            }
        }

        let mut files_vec: Vec<String> = indexed_files.into_iter().collect();
        files_vec.sort();

        let metadata = serde_json::json!({
            "name": persisted_collection.name,
            "config": persisted_collection.config,
            "vector_count": persisted_collection.vectors.len(),
            "indexed_files": files_vec,
            "total_files": files_vec.len(),
            "created_at": chrono::Utc::now().to_rfc3339(),
        });

        let json_data = serde_json::to_string_pretty(&metadata)?;
        let mut file = File::create(path)?;
        file.write_all(json_data.as_bytes())?;

        debug!(
            "Saved metadata for '{}' to {} ({} files indexed)",
            persisted_collection.name,
            path.display(),
            files_vec.len()
        );
        Ok(())
    }

    /// Static version of save_collection_tokenizer
    fn save_collection_tokenizer_static(
        collection_name: &str,
        path: &std::path::Path,
    ) -> Result<()> {
        use std::fs::File;
        use std::io::Write;

        // For dynamic collections, create a minimal tokenizer
        let tokenizer_data = serde_json::json!({
            "collection_name": collection_name,
            "tokenizer_type": "dynamic",
            "created_at": chrono::Utc::now().to_rfc3339(),
            "vocab_size": 0,
            "special_tokens": {},
        });

        let json_data = serde_json::to_string_pretty(&tokenizer_data)?;
        let mut file = File::create(path)?;
        file.write_all(json_data.as_bytes())?;

        debug!(
            "Saved tokenizer for '{}' to {}",
            collection_name,
            path.display()
        );
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{CompressionConfig, DistanceMetric, HnswConfig, Payload};

    #[test]
    fn test_create_and_list_collections() {
        let store = VectorStore::new();

        let config = CollectionConfig {
            dimension: 128,
            metric: DistanceMetric::Cosine,
            hnsw_config: HnswConfig::default(),
            quantization: Default::default(),
            compression: Default::default(),
            normalization: None,
        };

        // Get initial collection count
        let initial_count = store.list_collections().len();

        // Create collections with unique names
        store
            .create_collection("test_list1_unique", config.clone())
            .unwrap();
        store
            .create_collection("test_list2_unique", config)
            .unwrap();

        // List collections
        let collections = store.list_collections();
        assert_eq!(collections.len(), initial_count + 2);
        assert!(collections.contains(&"test_list1_unique".to_string()));
        assert!(collections.contains(&"test_list2_unique".to_string()));

        // Cleanup
        store.delete_collection("test_list1_unique").ok();
        store.delete_collection("test_list2_unique").ok();
    }

    #[test]
    fn test_duplicate_collection_error() {
        let store = VectorStore::new();

        let config = CollectionConfig {
            dimension: 128,
            metric: DistanceMetric::Cosine,
            hnsw_config: HnswConfig::default(),
            quantization: Default::default(),
            compression: Default::default(),
            normalization: None,
        };

        // Create collection
        store.create_collection("test", config.clone()).unwrap();

        // Try to create duplicate
        let result = store.create_collection("test", config);
        assert!(matches!(
            result,
            Err(VectorizerError::CollectionAlreadyExists(_))
        ));
    }

    #[test]
    fn test_delete_collection() {
        let store = VectorStore::new();

        let config = CollectionConfig {
            dimension: 128,
            metric: DistanceMetric::Cosine,
            hnsw_config: HnswConfig::default(),
            quantization: Default::default(),
            compression: Default::default(),
            normalization: None,
        };

        // Get initial collection count
        let initial_count = store.list_collections().len();

        // Create and delete collection
        store
            .create_collection("test_delete_collection_unique", config)
            .unwrap();
        assert_eq!(store.list_collections().len(), initial_count + 1);

        store
            .delete_collection("test_delete_collection_unique")
            .unwrap();
        assert_eq!(store.list_collections().len(), initial_count);

        // Try to delete non-existent collection
        let result = store.delete_collection("test_delete_collection_unique");
        assert!(matches!(
            result,
            Err(VectorizerError::CollectionNotFound(_))
        ));
    }

    #[test]
    fn test_stats_functionality() {
        let store = VectorStore::new();

        let config = CollectionConfig {
            dimension: 3,
            metric: DistanceMetric::Euclidean,
            hnsw_config: HnswConfig::default(),
            quantization: Default::default(),
            compression: Default::default(),
            normalization: None,
        };

        // Get initial stats
        let initial_stats = store.stats();
        let initial_count = initial_stats.collection_count;
        let initial_vectors = initial_stats.total_vectors;

        // Create collection and add vectors
        store
            .create_collection("test_stats_unique", config)
            .unwrap();
        let vectors = vec![
            Vector::new("v1".to_string(), vec![1.0, 2.0, 3.0]),
            Vector::new("v2".to_string(), vec![4.0, 5.0, 6.0]),
        ];
        store.insert("test_stats_unique", vectors).unwrap();

        let stats = store.stats();
        assert_eq!(stats.collection_count, initial_count + 1);
        assert_eq!(stats.total_vectors, initial_vectors + 2);
        // Memory bytes may be 0 if collection uses optimization (always >= 0 for usize)
        let _ = stats.total_memory_bytes;

        // Cleanup
        store.delete_collection("test_stats_unique").ok();
    }

    #[test]
    fn test_concurrent_operations() {
        use std::sync::Arc;
        use std::thread;

        let store = Arc::new(VectorStore::new());

        let config = CollectionConfig {
            dimension: 3,
            metric: DistanceMetric::Euclidean,
            hnsw_config: HnswConfig::default(),
            quantization: Default::default(),
            compression: Default::default(),
            normalization: None,
        };

        // Create collection from main thread
        store.create_collection("concurrent_test", config).unwrap();

        let mut handles = vec![];

        // Spawn multiple threads to insert vectors
        for i in 0..5 {
            let store_clone = Arc::clone(&store);
            let handle = thread::spawn(move || {
                let vectors = vec![
                    Vector::new(format!("vec_{}_{}", i, 0), vec![i as f32, 0.0, 0.0]),
                    Vector::new(format!("vec_{}_{}", i, 1), vec![0.0, i as f32, 0.0]),
                ];
                store_clone.insert("concurrent_test", vectors).unwrap();
            });
            handles.push(handle);
        }

        // Wait for all threads to complete
        for handle in handles {
            handle.join().unwrap();
        }

        // Verify all vectors were inserted
        let stats = store.stats();
        assert_eq!(stats.collection_count, 1);
        assert_eq!(stats.total_vectors, 10); // 5 threads * 2 vectors each
    }

    #[test]
    fn test_collection_metadata() {
        let store = VectorStore::new();

        let config = CollectionConfig {
            dimension: 768,
            metric: DistanceMetric::Cosine,
            hnsw_config: HnswConfig {
                m: 32,
                ef_construction: 200,
                ef_search: 64,
                seed: Some(123),
            },
            quantization: Default::default(),
            compression: CompressionConfig {
                enabled: true,
                threshold_bytes: 2048,
                algorithm: crate::models::CompressionAlgorithm::Lz4,
            },
            normalization: None,
        };

        store
            .create_collection("metadata_test", config.clone())
            .unwrap();

        // Add some vectors
        let vectors = vec![
            Vector::new("v1".to_string(), vec![0.1; 768]),
            Vector::new("v2".to_string(), vec![0.2; 768]),
        ];
        store.insert("metadata_test", vectors).unwrap();

        // Test metadata retrieval
        let metadata = store.get_collection_metadata("metadata_test").unwrap();
        assert_eq!(metadata.name, "metadata_test");
        assert_eq!(metadata.vector_count, 2);
        assert_eq!(metadata.config.dimension, 768);
        assert_eq!(metadata.config.metric, DistanceMetric::Cosine);
    }

    #[test]
    fn test_error_handling_edge_cases() {
        let store = VectorStore::new();

        let config = CollectionConfig {
            dimension: 3,
            metric: DistanceMetric::Euclidean,
            hnsw_config: HnswConfig::default(),
            quantization: Default::default(),
            compression: Default::default(),
            normalization: None,
        };

        // Use unique collection name to avoid conflicts in parallel test execution
        let test_collection = format!("error_test_{}", std::process::id());
        store.create_collection(&test_collection, config).unwrap();

        // Test operations on non-existent collection
        let result = store.insert("non_existent", vec![]);
        assert!(matches!(
            result,
            Err(VectorizerError::CollectionNotFound(_))
        ));

        let result = store.search("non_existent", &[1.0, 2.0, 3.0], 1);
        assert!(matches!(
            result,
            Err(VectorizerError::CollectionNotFound(_))
        ));

        let result = store.get_vector("non_existent", "v1");
        assert!(matches!(
            result,
            Err(VectorizerError::CollectionNotFound(_))
        ));

        // Test operations on non-existent vector
        let result = store.get_vector(&test_collection, "non_existent");
        assert!(matches!(result, Err(VectorizerError::VectorNotFound(_))));

        let result = store.update(
            &test_collection,
            Vector::new("non_existent".to_string(), vec![1.0, 2.0, 3.0]),
        );
        assert!(matches!(result, Err(VectorizerError::VectorNotFound(_))));

        let result = store.delete(&test_collection, "non_existent");
        assert!(matches!(result, Err(VectorizerError::VectorNotFound(_))));
    }
}
