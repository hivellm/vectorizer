//! `VectorStore` — the top-level in-memory collection registry.
//!
//! This module only holds the struct, constructors, storage-format
//! probe, and submodule declarations. Everything else is split by
//! concern:
//!
//! - [`collection_type`] — the `CollectionType` dispatch enum
//!   (CPU / GPU / Sharded / DistributedSharded) and its delegated
//!   methods
//! - [`collections`]     — collection CRUD, ownership, lazy loading,
//!   empty-collection cleanup, graph enablement
//! - [`aliases`]         — alias resolution + CRUD
//! - [`vectors`]         — insert / update / delete / get-vector
//! - [`search`]          — search + hybrid_search dispatch
//! - [`metadata`]        — stats + the `metadata` DashMap accessor
//!   + `VectorStoreStats`
//! - [`wal`]             — write-ahead log writers + recovery + replay
//! - [`persistence`]     — lazy `.vecdb` / legacy `.bin` loading
//!   (the save half lives in [`autosave`])
//! - [`autosave`]        — auto-save flag + pending-saves set + legacy
//!   `save_collection_*` writers
//!
//! The `Collection`, `Sharded`, `DistributedSharded`, and
//! `HiveGpuCollection` concrete types all live in sibling `db/`
//! modules — this directory only unifies them behind `CollectionType`.

use std::collections::HashSet;
use std::sync::Arc;

use dashmap::DashMap;
use tracing::info;

use crate::db::wal_integration::WalIntegration;
// Names the tests module at `src/db/vector_store_tests.rs` picks up via
// `use super::*;`. Kept in sync with the pre-split surface so the test
// file doesn't need touching.
#[allow(unused_imports)]
use crate::error::{Result, VectorizerError};
#[allow(unused_imports)]
use crate::models::{CollectionConfig, Vector};

mod aliases;
mod autosave;
mod collection_type;
mod collections;
mod metadata;
mod persistence;
mod search;
mod vectors;
mod wal;

pub use collection_type::CollectionType;
pub use metadata::VectorStoreStats;

/// Thread-safe in-memory vector store
#[derive(Clone)]
pub struct VectorStore {
    /// Collections stored in a concurrent hash map
    pub(super) collections: Arc<DashMap<String, CollectionType>>,
    /// Collection aliases (alias -> target collection)
    pub(super) aliases: Arc<DashMap<String, String>>,
    /// Auto-save enabled flag (prevents auto-save during initialization)
    pub(super) auto_save_enabled: Arc<std::sync::atomic::AtomicBool>,
    /// Collections pending save (for batch persistence)
    pub(super) pending_saves: Arc<parking_lot::Mutex<std::collections::HashSet<String>>>,
    /// Background save task handle
    #[allow(dead_code)]
    pub(super) save_task_handle: Arc<parking_lot::Mutex<Option<tokio::task::JoinHandle<()>>>>,
    /// Global metadata (for replication config, etc.)
    pub(super) metadata: Arc<DashMap<String, String>>,
    /// WAL integration (optional, for crash recovery)
    pub(super) wal: Arc<parking_lot::Mutex<Option<WalIntegration>>>,
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
            aliases: Arc::new(DashMap::new()),
            auto_save_enabled: Arc::new(std::sync::atomic::AtomicBool::new(false)),
            pending_saves: Arc::new(parking_lot::Mutex::new(HashSet::new())),
            save_task_handle: Arc::new(parking_lot::Mutex::new(None)),
            metadata: Arc::new(DashMap::new()),
            wal: Arc::new(parking_lot::Mutex::new(
                Some(WalIntegration::new_disabled()),
            )),
        };

        // Check for automatic migration on startup
        store.check_and_migrate_storage();

        store
    }

    /// Create a new empty vector store with CPU-only collections (for testing).
    /// Bypasses GPU detection and ensures consistent behavior across platforms.
    pub fn new_cpu_only() -> Self {
        info!("Creating new VectorStore (CPU-only mode for testing)");

        Self {
            collections: Arc::new(DashMap::new()),
            aliases: Arc::new(DashMap::new()),
            auto_save_enabled: Arc::new(std::sync::atomic::AtomicBool::new(false)),
            pending_saves: Arc::new(parking_lot::Mutex::new(HashSet::new())),
            save_task_handle: Arc::new(parking_lot::Mutex::new(None)),
            metadata: Arc::new(DashMap::new()),
            wal: Arc::new(parking_lot::Mutex::new(
                Some(WalIntegration::new_disabled()),
            )),
        }
    }

    /// Create a new vector store with Hive-GPU configuration
    #[cfg(feature = "hive-gpu")]
    pub fn new_with_hive_gpu_config() -> Self {
        info!("Creating new VectorStore with Hive-GPU configuration");
        Self {
            collections: Arc::new(DashMap::new()),
            aliases: Arc::new(DashMap::new()),
            auto_save_enabled: Arc::new(std::sync::atomic::AtomicBool::new(false)),
            pending_saves: Arc::new(parking_lot::Mutex::new(HashSet::new())),
            save_task_handle: Arc::new(parking_lot::Mutex::new(None)),
            metadata: Arc::new(DashMap::new()),
            wal: Arc::new(parking_lot::Mutex::new(
                Some(WalIntegration::new_disabled()),
            )),
        }
    }

    /// Create a new vector store with automatic GPU detection.
    /// Priority: Hive-GPU (Metal/CUDA/WebGPU) > CPU.
    pub fn new_auto() -> Self {
        info!("🔍 VectorStore::new_auto() called - starting GPU detection...");

        // Create store without loading collections (will be loaded in background task)
        let store = Self::new();

        // DON'T enable auto-save yet — will be enabled after collections are loaded.
        // This prevents auto-save from triggering during initial load.
        info!(
            "⏸️  Auto-save disabled during initialization — will be enabled after load completes"
        );

        info!("✅ VectorStore created (collections will be loaded in background)");

        // Detect best available GPU backend
        #[cfg(feature = "hive-gpu")]
        {
            use crate::db::gpu_detection::{GpuBackendType, GpuDetector};

            info!("🚀 Detecting GPU capabilities...");

            let backend = GpuDetector::detect_best_backend();

            match backend {
                GpuBackendType::None => {
                    // CPU mode is the default, no need to log
                }
                _ => {
                    info!("✅ {} GPU detected and enabled!", backend.name());

                    if let Some(gpu_info) = GpuDetector::get_gpu_info(backend) {
                        info!("📊 GPU Info: {}", gpu_info);
                    }

                    let store = Self::new_with_hive_gpu_config();
                    info!("⏸️  Auto-save will be enabled after collections load");
                    return store;
                }
            }
        }

        #[cfg(not(feature = "hive-gpu"))]
        {
            info!("⚠️ Hive-GPU not available (hive-gpu feature not compiled)");
        }

        // Return the store (auto-save will be enabled after collections load)
        info!("💻 Using CPU-only mode");
        store
    }

    /// Check storage format and perform automatic migration if needed
    fn check_and_migrate_storage(&self) {
        use std::fs;

        use crate::storage::{StorageFormat, detect_format};

        let data_dir = std::path::PathBuf::from("./data");

        // Create data directory if it doesn't exist
        if !data_dir.exists() {
            if let Err(e) = fs::create_dir_all(&data_dir) {
                tracing::warn!("Failed to create data directory: {}", e);
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
            info!("📁 Empty data directory detected — initializing with .vecdb format");
            if let Err(e) = self.initialize_compact_storage(&data_dir) {
                tracing::warn!("Failed to initialize compact storage: {}", e);
            } else {
                info!("✅ Initialized with .vecdb compact storage format");
            }
            return;
        }

        let format = detect_format(&data_dir);

        match format {
            StorageFormat::Legacy => {
                // Check if migration is enabled in config
                // For now, just log that migration is available
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
    fn initialize_compact_storage(
        &self,
        data_dir: &std::path::PathBuf,
    ) -> crate::error::Result<()> {
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
}

impl Default for VectorStore {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
#[path = "../vector_store_tests.rs"]
mod tests;
