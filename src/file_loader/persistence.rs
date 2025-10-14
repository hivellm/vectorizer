//! Thin wrapper for .vecdb persistence using existing storage module

use crate::{
    VectorStore,
    error::Result,
    storage::{StorageReader, StorageCompactor},
};
use tracing::{info, warn};

/// Thin persistence wrapper - uses existing storage module
pub struct Persistence {
    data_dir: std::path::PathBuf,
}

impl Persistence {
    pub fn new() -> Self {
        Self {
            data_dir: std::path::PathBuf::from("./data"),
        }
    }

    /// Check if a collection exists in .vecdb archive
    pub fn collection_exists_in_vecdb(&self, collection_name: &str) -> bool {
        let vecdb_path = self.data_dir.join("vectorizer.vecdb");
        
        if !vecdb_path.exists() {
            return false;
        }

        match StorageReader::new(&self.data_dir) {
            Ok(reader) => {
                match reader.get_collection(collection_name) {
                    Ok(Some(_)) => true,
                    _ => false,
                }
            }
            Err(_) => false,
        }
    }

    /// Save collection using existing persistence module
    pub fn save_collection_legacy_temp(&self, store: &VectorStore, collection_name: &str) -> Result<()> {
        // Use existing VectorStore save functionality
        let collection = store.get_collection(collection_name)?;
        let vectors = collection.get_all_vectors();
        
        if vectors.is_empty() {
            warn!("Collection '{}' has no vectors, skipping", collection_name);
            return Ok(());
        }

        // Save using existing persistence (creates temporary _vector_store.bin)
        let temp_path = self.data_dir.join(format!("{}_vector_store.bin", collection_name));
        
        let sub_store = VectorStore::new();
        let meta = collection.metadata();
        sub_store.create_collection(collection_name, meta.config.clone())?;
        sub_store.insert(collection_name, vectors)?;
        sub_store.save(&temp_path)?;

        Ok(())
    }

    /// Compact all using existing StorageCompactor
    pub fn compact_and_cleanup(&self) -> Result<usize> {
        info!("Compacting to .vecdb using existing StorageCompactor...");

        let compactor = StorageCompactor::new(&self.data_dir, 6, 1000);
        let index = compactor.compact_all()?;

        info!("Compacted {} collections to .vecdb", index.collection_count());

        // Cleanup temporary files
        self.cleanup_temp_files()?;

        Ok(index.collection_count())
    }

    fn cleanup_temp_files(&self) -> Result<()> {
        if let Ok(entries) = std::fs::read_dir(&self.data_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_file() {
                    if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                        if name.ends_with("_vector_store.bin")
                            || name.ends_with("_tokenizer.json")
                            || name.ends_with("_metadata.json")
                        {
                            let _ = std::fs::remove_file(&path);
                        }
                    }
                }
            }
        }

        Ok(())
    }
}

