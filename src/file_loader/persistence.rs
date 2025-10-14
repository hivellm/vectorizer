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
        // Use existing VectorStore save functionality directly
        let collection = store.get_collection(collection_name)?;
        let vectors = collection.get_all_vectors();
        
        if vectors.is_empty() {
            warn!("Collection '{}' has no vectors, skipping temp save", collection_name);
            return Ok(());
        }

        // Build PersistedCollection manually (NO new VectorStore - prevents memory loop)
        use crate::persistence::{PersistedCollection, PersistedVector};
        use std::fs::File;
        use std::io::BufWriter;
        
        let temp_path = self.data_dir.join(format!("{}_vector_store.bin", collection_name));
        let meta = collection.metadata();
        
        // Convert vectors to PersistedVector format using From trait
        let persisted_vectors: Vec<PersistedVector> = vectors.into_iter()
            .map(|v| PersistedVector::from(v))
            .collect();
        
        let persisted = PersistedCollection {
            name: collection_name.to_string(),
            config: meta.config.clone(),
            vectors: persisted_vectors,
            hnsw_dump_basename: None,
        };
        
        let file = File::create(&temp_path)
            .map_err(|e| crate::error::VectorizerError::Io(e))?;
        let writer = BufWriter::new(file);
        
        bincode::serialize_into(writer, &persisted)
            .map_err(|e| crate::error::VectorizerError::Serialization(e.to_string()))?;

        info!("Saved temp collection '{}' to {}", collection_name, temp_path.display());
        
        // Save complete metadata with file list and checksums
        self.save_complete_metadata(collection_name, &meta, &persisted)?;
        self.save_checksums(collection_name, &persisted)?;
        
        Ok(())
    }
    
    /// Save complete metadata with file list and checksums
    fn save_complete_metadata(
        &self, 
        collection_name: &str, 
        metadata: &crate::models::CollectionMetadata,
        persisted: &crate::persistence::PersistedCollection,
    ) -> Result<()> {
        use serde_json;
        use std::fs::File;
        use std::io::BufWriter;
        use std::collections::HashSet;
        
        let metadata_path = self.data_dir.join(format!("{}_metadata.json", collection_name));
        
        // Extract unique file paths from vectors
        let mut indexed_files: HashSet<String> = HashSet::new();
        
        for vector in &persisted.vectors {
            if let Ok(runtime_vec) = crate::persistence::PersistedVector::into_runtime(vector.clone()) {
                if let Some(payload) = &runtime_vec.payload {
                    if let Some(file_path_val) = payload.data.get("file_path") {
                        if let Some(file_path) = file_path_val.as_str() {
                            indexed_files.insert(file_path.to_string());
                        }
                    }
                }
            }
        }
        
        #[derive(serde::Serialize)]
        struct MetadataJson {
            collection_name: String,
            dimension: usize,
            vector_count: usize,
            distance_metric: String,
            created_at: String,
            updated_at: String,
            indexed_files: Vec<String>,
            file_count: usize,
        }
        
        let mut files_vec: Vec<String> = indexed_files.into_iter().collect();
        files_vec.sort();
        
        let metadata_json = MetadataJson {
            collection_name: collection_name.to_string(),
            dimension: metadata.config.dimension,
            vector_count: metadata.vector_count,
            distance_metric: format!("{:?}", metadata.config.metric),
            created_at: metadata.created_at.to_rfc3339(),
            updated_at: metadata.updated_at.to_rfc3339(),
            indexed_files: files_vec.clone(),
            file_count: files_vec.len(),
        };
        
        let file = File::create(&metadata_path)
            .map_err(|e| crate::error::VectorizerError::Io(e))?;
        let writer = BufWriter::new(file);
        
        serde_json::to_writer_pretty(writer, &metadata_json)
            .map_err(|e| crate::error::VectorizerError::Serialization(e.to_string()))?;
        
        info!("Saved metadata for collection '{}': {} vectors from {} files", 
            collection_name, metadata.vector_count, files_vec.len());
        Ok(())
    }
    
    /// Save file checksums for file watcher
    fn save_checksums(&self, collection_name: &str, persisted: &crate::persistence::PersistedCollection) -> Result<()> {
        use serde_json;
        use std::fs::File;
        use std::io::BufWriter;
        use std::collections::HashMap;
        use sha2::{Sha256, Digest};
        
        let checksums_path = self.data_dir.join(format!("{}_checksums.json", collection_name));
        
        // Extract file paths from vector payloads and compute checksums
        let mut file_checksums: HashMap<String, String> = HashMap::new();
        
        for vector in &persisted.vectors {
            // Convert PersistedVector to Vector to access payload
            if let Ok(runtime_vec) = crate::persistence::PersistedVector::into_runtime(vector.clone()) {
                if let Some(payload) = &runtime_vec.payload {
                    if let Some(file_path_val) = payload.data.get("file_path") {
                        if let Some(file_path) = file_path_val.as_str() {
                            // Compute checksum if we haven't already
                            if !file_checksums.contains_key(file_path) {
                                if let Ok(content) = std::fs::read(file_path) {
                                    let mut hasher = Sha256::new();
                                    hasher.update(&content);
                                    let hash = format!("{:x}", hasher.finalize());
                                    file_checksums.insert(file_path.to_string(), hash);
                                }
                            }
                        }
                    }
                }
            }
        }
        
        let file = File::create(&checksums_path)
            .map_err(|e| crate::error::VectorizerError::Io(e))?;
        let writer = BufWriter::new(file);
        
        serde_json::to_writer_pretty(writer, &file_checksums)
            .map_err(|e| crate::error::VectorizerError::Serialization(e.to_string()))?;
        
        info!("Saved checksums for {} files in collection '{}' to {}", 
            file_checksums.len(), collection_name, checksums_path.display());
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
                        // Only delete vector_store.bin (temp binary)
                        // KEEP metadata.json, tokenizer.json, and checksums.json for file watcher
                        if name.ends_with("_vector_store.bin") {
                            let _ = std::fs::remove_file(&path);
                            info!("Cleaned up temp file: {}", name);
                        }
                    }
                }
            }
        }

        Ok(())
    }
}


