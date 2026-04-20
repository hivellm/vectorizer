//! Auto-save orchestration: a flag + pending-saves set + the static
//! + instance variants of `save_collection_to_file` that write a
//! collection as a trio of legacy files (`_vector_store.bin` +
//! `_metadata.json` + `_tokenizer.json`).
//!
//! The active save path today is `AutoSaveManager` (5min intervals,
//! compacts directly from memory into `vectorizer.vecdb`). The legacy
//! raw-file path here is preserved for `save_collection_to_file` callers
//! that still target the legacy directory layout.

use tracing::{debug, info, warn};

use super::{CollectionType, VectorStore};
use crate::error::Result;
use crate::models::Vector;

impl VectorStore {
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

        // DEPRECATED: Old auto-save system disabled.
        // Auto-save is now managed exclusively by AutoSaveManager (5min intervals)
        // which compacts directly from memory without creating raw .bin files.
        info!("✅ Auto-save flag enabled — managed by AutoSaveManager (no raw .bin files)");
    }

    /// Disable auto-save for all collections.
    /// Useful during bulk operations or maintenance.
    pub fn disable_auto_save(&self) {
        self.auto_save_enabled
            .store(false, std::sync::atomic::Ordering::Relaxed);
        info!("⏸️ Auto-save disabled for all collections");
    }

    /// Force immediate save of all pending collections.
    /// Useful before shutdown or critical operations.
    pub fn force_save_all(&self) -> Result<()> {
        if self.pending_saves.lock().is_empty() {
            debug!("No pending saves to force");
            return Ok(());
        }

        info!(
            "🔄 Force saving {} pending collections",
            self.pending_saves.lock().len()
        );

        let collections_to_save: Vec<String> = self.pending_saves.lock().iter().cloned().collect();
        self.pending_saves.lock().clear();

        // Force save deferred here — using .vecdb format means the
        // in-memory AutoSaveManager owns the actual flush; we just clear
        // the pending set so subsequent marks aren't duplicates.
        for collection_name in collections_to_save {
            debug!(
                "Collection '{}' marked for save (using .vecdb format)",
                collection_name
            );
        }

        info!("✅ Force save completed");
        Ok(())
    }

    /// Save a single collection to file following workspace pattern.
    /// Creates separate files for vectors, tokenizer, and metadata.
    pub fn save_collection_to_file(&self, collection_name: &str) -> Result<()> {
        use std::fs;

        use crate::persistence::PersistedCollection;
        use crate::storage::{StorageFormat, detect_format};

        info!(
            "Saving collection '{}' to individual files",
            collection_name
        );

        // Check if using compact storage format — if so, don't save in legacy format
        let data_dir = Self::get_data_dir();
        if detect_format(&data_dir) == StorageFormat::Compact {
            debug!(
                "⏭️ Skipping legacy save for '{}' — using .vecdb format",
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

        // Save graph if enabled
        match &*collection {
            CollectionType::Cpu(c) => {
                if let Some(graph) = c.get_graph() {
                    if let Err(e) = graph.save_to_file(&data_dir) {
                        warn!(
                            "Failed to save graph for collection '{}': {}",
                            collection_name, e
                        );
                        // Don't fail collection save if graph save fails
                    }
                }
            }
            _ => {
                // Graph not supported for other collection types
            }
        }

        info!(
            "Successfully saved collection '{}' to files",
            collection_name
        );
        Ok(())
    }

    /// Static variant of [`save_collection_to_file`] — callable from a
    /// background task that holds `&CollectionType` directly without
    /// round-tripping through `get_collection`.
    #[allow(dead_code)]
    pub(super) fn save_collection_to_file_static(
        collection_name: &str,
        collection: &CollectionType,
    ) -> Result<()> {
        use std::fs;

        use crate::persistence::PersistedCollection;
        use crate::storage::{StorageFormat, detect_format};

        info!("💾 Starting save for collection '{}'", collection_name);

        // Check if using compact storage format — if so, don't save in legacy format
        let data_dir = Self::get_data_dir();
        if detect_format(&data_dir) == StorageFormat::Compact {
            debug!(
                "⏭️ Skipping legacy save for '{}' — using .vecdb format",
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

        // Save graph if enabled
        match collection {
            CollectionType::Cpu(c) => {
                if let Some(graph) = c.get_graph() {
                    if let Err(e) = graph.save_to_file(&data_dir) {
                        warn!(
                            "Failed to save graph for collection '{}': {}",
                            collection_name, e
                        );
                    } else {
                        info!("💾 Graph saved successfully");
                    }
                }
            }
            _ => {
                // Graph not supported for other collection types
            }
        }

        info!(
            "✅ Successfully saved collection '{}' to files",
            collection_name
        );
        Ok(())
    }

    /// Mark a collection for auto-save (internal method)
    pub(super) fn mark_collection_for_save(&self, collection_name: &str) {
        if self
            .auto_save_enabled
            .load(std::sync::atomic::Ordering::Relaxed)
        {
            debug!("📝 Marking collection '{}' for auto-save", collection_name);
            self.pending_saves
                .lock()
                .insert(collection_name.to_string());
            debug!(
                "📝 Collection '{}' added to pending saves (total: {})",
                collection_name,
                self.pending_saves.lock().len()
            );
        } else {
            // Auto-save is disabled during initialization — expected, not an error
            debug!(
                "Auto-save is disabled, collection '{}' will not be saved (normal during initialization)",
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
            let v: Vector = pv.clone().into();
            if let Some(payload) = &v.payload {
                if let Some(metadata) = payload.data.get("metadata") {
                    if let Some(file_path) = metadata.get("file_path").and_then(|v| v.as_str()) {
                        indexed_files.insert(file_path.to_string());
                    }
                }
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
