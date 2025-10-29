//! Thin wrapper for .vecdb persistence using existing storage module

use tracing::{error, info, warn};

use crate::VectorStore;
use crate::error::Result;
use crate::storage::{StorageCompactor, StorageReader};

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
            Ok(reader) => match reader.get_collection(collection_name) {
                Ok(Some(_)) => true,
                _ => false,
            },
            Err(_) => false,
        }
    }

    /// Save collection using existing persistence module
    pub fn save_collection_legacy_temp(
        &self,
        store: &VectorStore,
        collection_name: &str,
    ) -> Result<()> {
        // Use existing VectorStore save functionality directly
        let collection = store.get_collection(collection_name)?;
        let vectors = collection.get_all_vectors();

        if vectors.is_empty() {
            warn!(
                "Collection '{}' has no vectors, skipping temp save",
                collection_name
            );
            return Ok(());
        }

        // Build PersistedCollection manually (NO new VectorStore - prevents memory loop)
        use std::fs::File;
        use std::io::BufWriter;

        use crate::persistence::{PersistedCollection, PersistedVector};

        // Ensure data directory exists
        if !self.data_dir.exists() {
            std::fs::create_dir_all(&self.data_dir)
                .map_err(|e| crate::error::VectorizerError::Io(e))?;
            info!("Created data directory: {}", self.data_dir.display());
        }

        let temp_path = self
            .data_dir
            .join(format!("{}_vector_store.bin", collection_name));
        info!(
            "💾 Saving collection '{}' to: {}",
            collection_name,
            temp_path.display()
        );

        let meta = collection.metadata();

        // Convert vectors to PersistedVector format using From trait
        let persisted_vectors: Vec<PersistedVector> = vectors
            .into_iter()
            .map(|v| PersistedVector::from(v))
            .collect();

        let persisted = PersistedCollection {
            name: collection_name.to_string(),
            config: Some(meta.config.clone()),
            vectors: persisted_vectors,
            hnsw_dump_basename: None,
            tokenizer: None,
            checksums: None,
        };

        let file = File::create(&temp_path).map_err(|e| crate::error::VectorizerError::Io(e))?;
        let writer = BufWriter::new(file);

        // Use JSON (without pretty formatting - will be compressed in ZIP anyway)
        serde_json::to_writer(writer, &persisted).map_err(|e| {
            crate::error::VectorizerError::Serialization(format!("JSON serialize error: {}", e))
        })?;

        // Verify file was created
        if temp_path.exists() {
            let file_size = std::fs::metadata(&temp_path).map(|m| m.len()).unwrap_or(0);
            info!(
                "✅ Saved temp collection '{}' to {} ({} MB, {} vectors)",
                collection_name,
                temp_path.display(),
                file_size / 1_048_576,
                persisted.vectors.len()
            );
        } else {
            warn!("⚠️  File not found after save: {}", temp_path.display());
        }

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
        use std::collections::HashSet;
        use std::fs::File;
        use std::io::BufWriter;

        use serde_json;

        let metadata_path = self
            .data_dir
            .join(format!("{}_metadata.json", collection_name));

        // Extract unique file paths from vectors
        let mut indexed_files: HashSet<String> = HashSet::new();

        for vector in &persisted.vectors {
            if let Ok(runtime_vec) =
                crate::persistence::PersistedVector::into_runtime(vector.clone())
            {
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

        let file =
            File::create(&metadata_path).map_err(|e| crate::error::VectorizerError::Io(e))?;
        let writer = BufWriter::new(file);

        serde_json::to_writer_pretty(writer, &metadata_json)
            .map_err(|e| crate::error::VectorizerError::Serialization(e.to_string()))?;

        info!(
            "Saved metadata for collection '{}': {} vectors from {} files",
            collection_name,
            metadata.vector_count,
            files_vec.len()
        );
        Ok(())
    }

    /// Save file checksums for file watcher
    fn save_checksums(
        &self,
        collection_name: &str,
        persisted: &crate::persistence::PersistedCollection,
    ) -> Result<()> {
        use std::collections::HashMap;
        use std::fs::File;
        use std::io::BufWriter;

        use serde_json;
        use sha2::{Digest, Sha256};

        info!("🔍 save_checksums called for collection '{}'", collection_name);

        let checksums_path = self
            .data_dir
            .join(format!("{}_checksums.json", collection_name));

        info!("📁 Checksums path: {}", checksums_path.display());

        // Extract file paths from vector payloads and compute checksums
        let mut file_checksums: HashMap<String, String> = HashMap::new();

        for vector in &persisted.vectors {
            // Convert PersistedVector to Vector to access payload
            if let Ok(runtime_vec) =
                crate::persistence::PersistedVector::into_runtime(vector.clone())
            {
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

        info!("📊 Computed checksums for {} files", file_checksums.len());

        let file =
            File::create(&checksums_path).map_err(|e| {
                error!("❌ Failed to create checksums file: {}", e);
                crate::error::VectorizerError::Io(e)
            })?;
        let writer = BufWriter::new(file);

        serde_json::to_writer_pretty(writer, &file_checksums)
            .map_err(|e| {
                error!("❌ Failed to write checksums: {}", e);
                crate::error::VectorizerError::Serialization(e.to_string())
            })?;

        info!(
            "✅ Saved checksums for {} files in collection '{}' to {}",
            file_checksums.len(),
            collection_name,
            checksums_path.display()
        );
        
        // Verify file was created
        if checksums_path.exists() {
            let metadata = std::fs::metadata(&checksums_path)?;
            info!("✅ Checksums file verified: {} bytes", metadata.len());
        } else {
            error!("❌ CRITICAL: Checksums file was NOT created!");
        }
        
        Ok(())
    }

    /// Compact all using existing StorageCompactor
    pub fn compact_and_cleanup(&self) -> Result<usize> {
        info!("🗜️  Starting compaction to .vecdb using StorageCompactor...");
        info!("   Data directory: {}", self.data_dir.display());

        // Verify data directory exists
        if !self.data_dir.exists() {
            warn!("Data directory does not exist: {}", self.data_dir.display());
            std::fs::create_dir_all(&self.data_dir)
                .map_err(|e| crate::error::VectorizerError::Io(e))?;
            info!("Created data directory");
        }

        // CRITICAL: List ALL files BEFORE compaction to verify tokenizer/checksums exist
        info!("📋 BEFORE COMPACTION - Files in data directory:");
        if let Ok(entries) = std::fs::read_dir(&self.data_dir) {
            let mut tokenizer_count = 0;
            let mut checksum_count = 0;
            let mut vector_count = 0;
            let mut metadata_count = 0;
            
            for entry in entries {
                if let Ok(entry) = entry {
                    let path = entry.path();
                    if path.is_file() {
                        if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                            if name.ends_with("_tokenizer.json") {
                                tokenizer_count += 1;
                                info!("   ✅ TOKENIZER: {}", name);
                            } else if name.ends_with("_checksums.json") {
                                checksum_count += 1;
                                info!("   ✅ CHECKSUM: {}", name);
                            } else if name.ends_with("_vector_store.bin") {
                                vector_count += 1;
                                info!("   ✅ VECTOR: {}", name);
                            } else if name.ends_with("_metadata.json") {
                                metadata_count += 1;
                                info!("   ✅ METADATA: {}", name);
                            }
                        }
                    }
                }
            }
            
            info!("📊 File counts:");
            info!("   Tokenizers: {}", tokenizer_count);
            info!("   Checksums: {}", checksum_count);
            info!("   Vector stores: {}", vector_count);
            info!("   Metadata: {}", metadata_count);
            
            if tokenizer_count == 0 {
                error!("❌ CRITICAL: NO TOKENIZER FILES FOUND BEFORE COMPACTION!");
            }
            if checksum_count == 0 {
                error!("❌ CRITICAL: NO CHECKSUM FILES FOUND BEFORE COMPACTION!");
            }
        }

        // CRITICAL: Check if .vecdb already exists before compaction
        let vecdb_path = self.data_dir.join("vectorizer.vecdb");
        let vecdb_exists = vecdb_path.exists();

        let compactor = StorageCompactor::new(&self.data_dir, 6, 1000);

        info!("🔄 Calling compactor.compact_all()...");
        let index = compactor.compact_all()?;

        // CRITICAL PROTECTION: NEVER overwrite existing .vecdb with empty data!
        if index.collection_count() == 0 && vecdb_exists {
            warn!("⚠️  PROTECTION: Refusing to overwrite existing .vecdb with 0 collections!");
            warn!("⚠️  This would destroy the database!");
            return Err(crate::error::VectorizerError::Other(
                "Refusing to overwrite .vecdb with empty data - collections not loaded correctly"
                    .to_string(),
            ));
        }

        let vecidx_path = self.data_dir.join("vectorizer.vecidx");

        info!(
            "✅ Compacted {} collections to .vecdb",
            index.collection_count()
        );
        info!(
            "   .vecdb file: {} (exists: {})",
            vecdb_path.display(),
            vecdb_path.exists()
        );
        info!(
            "   .vecidx file: {} (exists: {})",
            vecidx_path.display(),
            vecidx_path.exists()
        );

        if vecdb_path.exists() {
            let size = std::fs::metadata(&vecdb_path).map(|m| m.len()).unwrap_or(0);
            info!("   .vecdb size: {} MB", size / 1_048_576);
        }

        // Cleanup temporary files
        self.cleanup_temp_files()?;

        Ok(index.collection_count())
    }

    fn cleanup_temp_files(&self) -> Result<()> {
        // DO NOT DELETE FILES!
        // The .bin, _metadata.json, _tokenizer.json files are needed for loading from .vecdb
        // They should ONLY be deleted when explicitly requested by user, not automatically
        info!("✅ Skipping cleanup - files kept for .vecdb loading");
        Ok(())
    }
}
