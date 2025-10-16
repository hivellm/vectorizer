//! Storage writer for creating and updating .vecdb archives

use crate::error::{Result, VectorizerError};
use crate::storage::{CollectionIndex, StorageIndex};
use crate::storage::index::{FileEntry, FileType, detect_file_type};
use sha2::{Sha256, Digest};
use std::fs::{self, File};
use std::io::{self, Write, Read};
use std::path::{Path, PathBuf};
use tracing::{info, warn};
use walkdir::WalkDir;
use zip::ZipWriter;
use zip::write::FileOptions;

/// Writer for creating and updating .vecdb archives
pub struct StorageWriter {
    /// Compression level (1-22 for zstd)
    compression_level: i32,
    
    /// Data directory
    data_dir: PathBuf,
}

impl StorageWriter {
    /// Create a new storage writer
    pub fn new(data_dir: impl AsRef<Path>, compression_level: i32) -> Self {
        Self {
            compression_level: compression_level.clamp(1, 22),
            data_dir: data_dir.as_ref().to_path_buf(),
        }
    }
    
    /// Write all collections to .vecdb archive atomically
    pub fn write_archive(&self, collections_dir: &Path) -> Result<StorageIndex> {
        let vecdb_path = self.data_dir.join(crate::storage::VECDB_FILE);
        let vecidx_path = self.data_dir.join(crate::storage::VECIDX_FILE);
        let temp_vecdb = vecdb_path.with_extension(format!("vecdb{}", crate::storage::TEMP_SUFFIX));
        let temp_vecidx = vecidx_path.with_extension(format!("vecidx{}", crate::storage::TEMP_SUFFIX));
        
        // Create temporary archive
        let mut index = self.create_archive(&temp_vecdb, collections_dir)?;
        
        // Save index to temporary file
        index.save(&temp_vecidx)?;
        
        // Atomic rename - only if everything succeeded
        fs::rename(&temp_vecdb, &vecdb_path)
            .map_err(|e| VectorizerError::Io(e))?;
        fs::rename(&temp_vecidx, &vecidx_path)
            .map_err(|e| VectorizerError::Io(e))?;
        
        Ok(index)
    }
    
    /// Write collections from memory to .vecdb archive atomically (no raw files created)
    pub fn write_from_memory(&self, collections: Vec<crate::persistence::PersistedCollection>) -> Result<StorageIndex> {
        let vecdb_path = self.data_dir.join(crate::storage::VECDB_FILE);
        let vecidx_path = self.data_dir.join(crate::storage::VECIDX_FILE);
        let temp_vecdb = vecdb_path.with_extension(format!("vecdb{}", crate::storage::TEMP_SUFFIX));
        let temp_vecidx = vecidx_path.with_extension(format!("vecidx{}", crate::storage::TEMP_SUFFIX));
        
        info!("🗜️  Writing {} collections from memory to {}", collections.len(), temp_vecdb.display());
        
        // Create temporary archive from memory
        let mut index = self.create_archive_from_memory(&temp_vecdb, collections)?;
        
        // Save index to temporary file
        index.save(&temp_vecidx)?;
        
        // Atomic rename - only if everything succeeded
        fs::rename(&temp_vecdb, &vecdb_path)
            .map_err(|e| VectorizerError::Io(e))?;
        fs::rename(&temp_vecidx, &vecidx_path)
            .map_err(|e| VectorizerError::Io(e))?;
        
        info!("✅ Successfully wrote vectorizer.vecdb from memory");
        
        Ok(index)
    }
    
    /// Create the archive file
    fn create_archive(&self, archive_path: &Path, source_dir: &Path) -> Result<StorageIndex> {
        let file = File::create(archive_path)
            .map_err(|e| VectorizerError::Io(e))?;
        
        let mut zip = ZipWriter::new(file);
        let mut index = StorageIndex::new();
        
        // Group files by collection name (pattern: collection-name_type.ext)
        let collections = self.discover_collections(source_dir)?;
        
        for (collection_name, files) in collections {
            let collection_index = self.add_flat_collection_to_archive(
                &mut zip,
                source_dir,
                &collection_name,
                &files
            )?;
            
            index.add_collection(collection_index);
        }
        
        zip.finish().map_err(|e| VectorizerError::Storage(e.to_string()))?;
        
        Ok(index)
    }
    
    /// Create the archive file from memory collections
    fn create_archive_from_memory(&self, archive_path: &Path, collections: Vec<crate::persistence::PersistedCollection>) -> Result<StorageIndex> {
        let file = File::create(archive_path)
            .map_err(|e| VectorizerError::Io(e))?;
        
        let mut zip = ZipWriter::new(file);
        let mut index = StorageIndex::new();
        
        let options = FileOptions::<()>::default()
            .compression_method(zip::CompressionMethod::Deflated)
            .unix_permissions(0o644);
        
        let collections_count = collections.len();
        
        for persisted_collection in collections {
            let collection_name = persisted_collection.name.clone();
            let vector_count = persisted_collection.vectors.len();
            let dimension = persisted_collection.config.as_ref().map(|c| c.dimension).unwrap_or(512);
            let config_clone = persisted_collection.config.clone();
            
            // Create collection index entry
            let mut collection_index = CollectionIndex {
                name: collection_name.clone(),
                files: Vec::new(),
                vector_count,
                dimension,
                metadata: std::collections::HashMap::new(),
            };
            
            // Serialize collection wrapped in PersistedVectorStore to JSON
            let vector_store_name = format!("{}_vector_store.bin", collection_name);
            let wrapped_store = crate::persistence::PersistedVectorStore {
                version: 1,
                collections: vec![persisted_collection],
            };
            let json_data = serde_json::to_vec(&wrapped_store)
                .map_err(|e| VectorizerError::Serialization(format!("Failed to serialize collection: {}", e)))?;
            
            let original_size = json_data.len() as u64;
            
            // Write to ZIP
            zip.start_file(&vector_store_name, options)
                .map_err(|e| VectorizerError::Storage(format!("Failed to start ZIP file: {}", e)))?;
            zip.write_all(&json_data)
                .map_err(|e| VectorizerError::Io(e))?;
            
            let compressed_size = json_data.len() as u64; // ZIP will compress it
            
            collection_index.files.push(crate::storage::index::FileEntry {
                path: vector_store_name.clone(),
                file_type: crate::storage::index::FileType::Vectors,
                size: original_size,
                compressed_size,
                checksum: String::new(), // Will be calculated by StorageIndex
            });
            
            // Write metadata if config exists
            if let Some(config) = config_clone {
                // Create a simple metadata structure for serialization
                #[derive(serde::Serialize)]
                struct CollectionMetadataForStorage {
                    name: String,
                    config: crate::models::CollectionConfig,
                    created_at: chrono::DateTime<chrono::Utc>,
                    modified_at: chrono::DateTime<chrono::Utc>,
                    vector_count: usize,
                }
                
                let metadata = CollectionMetadataForStorage {
                    name: collection_name.clone(),
                    config: config.clone(),
                    created_at: chrono::Utc::now(),
                    modified_at: chrono::Utc::now(),
                    vector_count,
                };
                
                let metadata_name = format!("{}_metadata.json", collection_name);
                let metadata_json = serde_json::to_vec_pretty(&metadata)
                    .map_err(|e| VectorizerError::Serialization(format!("Failed to serialize metadata: {}", e)))?;
                
                zip.start_file(&metadata_name, options)
                    .map_err(|e| VectorizerError::Storage(format!("Failed to start metadata file: {}", e)))?;
                zip.write_all(&metadata_json)
                    .map_err(|e| VectorizerError::Io(e))?;
                
                collection_index.files.push(crate::storage::index::FileEntry {
                    path: metadata_name,
                    file_type: crate::storage::index::FileType::Metadata,
                    size: metadata_json.len() as u64,
                    compressed_size: metadata_json.len() as u64,
                    checksum: String::new(),
                });
            }
            
            info!("   Added collection '{}' with {} vectors", collection_name, vector_count);
            index.add_collection(collection_index);
        }
        
        zip.finish().map_err(|e| VectorizerError::Storage(e.to_string()))?;
        
        info!("✅ Created archive with {} collections", collections_count);
        Ok(index)
    }
    
    /// Discover collections from flat file structure
    fn discover_collections(&self, source_dir: &Path) -> Result<std::collections::HashMap<String, Vec<PathBuf>>> {
        use std::collections::HashMap;
        
        let mut collections: HashMap<String, Vec<PathBuf>> = HashMap::new();
        
        info!("🔍 Discovering collections in: {}", source_dir.display());
        
        let mut total_files = 0;
        let mut bin_files = 0;
        
        for entry in fs::read_dir(source_dir).map_err(|e| VectorizerError::Io(e))? {
            let entry = entry.map_err(|e| VectorizerError::Io(e))?;
            let path = entry.path();
            
            if path.is_file() {
                total_files += 1;
                if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                    if name.ends_with("_vector_store.bin") {
                        bin_files += 1;
                        info!("   Found .bin file: {}", name);
                    }
                    
                    // Extract collection name - remove known suffixes
                    let collection_name = if let Some(stripped) = name.strip_suffix("_vector_store.bin") {
                        stripped.to_string()
                    } else if let Some(stripped) = name.strip_suffix("_metadata.json") {
                        stripped.to_string()
                    } else if let Some(stripped) = name.strip_suffix("_tokenizer.json") {
                        stripped.to_string()
                    } else if let Some(stripped) = name.strip_suffix("_checksums.json") {
                        stripped.to_string()
                    } else {
                        // Fallback: use everything before last underscore
                        if let Some(pos) = name.rfind('_') {
                            name[..pos].to_string()
                        } else {
                            continue; // Skip files that don't match pattern
                        }
                    };
                    
                    collections.entry(collection_name)
                        .or_insert_with(Vec::new)
                        .push(path.clone());
                }
            }
        }
        
        info!("🔍 Discovery complete: {} total files, {} .bin files, {} collections", 
            total_files, bin_files, collections.len());
        
        Ok(collections)
    }
    
    /// Add collection from flat file structure
    fn add_flat_collection_to_archive(
        &self,
        zip: &mut ZipWriter<File>,
        source_dir: &Path,
        collection_name: &str,
        files: &[PathBuf]
    ) -> Result<CollectionIndex> {
        let mut collection_index = CollectionIndex::new(collection_name.to_string());
        
        for file_path in files {
            let file_name = file_path.file_name()
                .and_then(|n| n.to_str())
                .ok_or_else(|| VectorizerError::Storage("Invalid file name".to_string()))?;
            
            // Archive path: just the filename (no subdirectories)
            let archive_path = PathBuf::from(file_name);
            
            let file_entry = self.add_file_to_archive(zip, file_path, &archive_path)?;
            collection_index.add_file(file_entry);
            
            // Extract metadata
            if file_name.contains("metadata.json") {
                if let Ok(metadata) = self.read_collection_metadata(file_path) {
                    collection_index.vector_count = metadata.vector_count;
                    collection_index.dimension = metadata.dimension;
                }
            }
        }
        
        Ok(collection_index)
    }
    
    /// Add a collection directory to the archive
    fn add_collection_to_archive(
        &self,
        zip: &mut ZipWriter<File>,
        collection_dir: &Path,
        collection_name: &str
    ) -> Result<CollectionIndex> {
        let mut collection_index = CollectionIndex::new(collection_name.to_string());
        
        // Walk through all files in the collection directory
        for entry in WalkDir::new(collection_dir) {
            let entry = entry.map_err(|e| VectorizerError::Io(e.into()))?;
            let path = entry.path();
            
            if path.is_file() {
                let relative_path = path.strip_prefix(collection_dir.parent().unwrap_or(collection_dir))
                    .map_err(|e| VectorizerError::Storage(e.to_string()))?;
                
                let file_entry = self.add_file_to_archive(zip, path, relative_path)?;
                collection_index.add_file(file_entry);
                
                // Extract vector count and dimension from metadata if available
                if let Some(filename) = path.file_name().and_then(|n| n.to_str()) {
                    if filename.contains("metadata.json") {
                        if let Ok(metadata) = self.read_collection_metadata(path) {
                            collection_index.vector_count = metadata.vector_count;
                            collection_index.dimension = metadata.dimension;
                        }
                    }
                }
            }
        }
        
        Ok(collection_index)
    }
    
    /// Add a single file to the archive
    fn add_file_to_archive(
        &self,
        zip: &mut ZipWriter<File>,
        file_path: &Path,
        archive_path: &Path
    ) -> Result<FileEntry> {
        let mut file = File::open(file_path)
            .map_err(|e| VectorizerError::Io(e))?;
        
        let mut content = Vec::new();
        file.read_to_end(&mut content)
            .map_err(|e| VectorizerError::Io(e))?;
        
        let original_size = content.len() as u64;
        let checksum = self.calculate_checksum(&content);
        
        // Decompress .gz files before adding to archive (avoid double compression)
        let content_to_compress = if file_path.extension().and_then(|e| e.to_str()) == Some("gz") {
            self.decompress_gzip(&content)?
        } else {
            content
        };
        
        // Determine file type
        let file_type = detect_file_type(archive_path.to_str().unwrap_or(""));
        
        // Add to ZIP with default compression
        let archive_path_str = archive_path.to_str()
            .ok_or_else(|| VectorizerError::Storage("Invalid archive path".to_string()))?
            .replace('\\', "/"); // Use forward slashes in ZIP
        
        // Use SimpleFileOptions for compatibility
        let options = zip::write::SimpleFileOptions::default()
            .compression_method(zip::CompressionMethod::Deflated);
        
        zip.start_file(&archive_path_str, options)
            .map_err(|e| VectorizerError::Storage(e.to_string()))?;
        zip.write_all(&content_to_compress)
            .map_err(|e| VectorizerError::Io(e))?;
        
        // Get compressed size (approximate)
        let compressed_size = (content_to_compress.len() as f64 * 0.5) as u64; // Estimate
        
        Ok(FileEntry::new(
            archive_path_str,
            original_size,
            compressed_size,
            checksum,
            file_type,
        ))
    }
    
    /// Calculate SHA-256 checksum
    fn calculate_checksum(&self, data: &[u8]) -> String {
        let mut hasher = Sha256::new();
        hasher.update(data);
        format!("{:x}", hasher.finalize())
    }
    
    /// Decompress gzip data
    fn decompress_gzip(&self, data: &[u8]) -> Result<Vec<u8>> {
        use flate2::read::GzDecoder;
        
        let mut decoder = GzDecoder::new(data);
        let mut decompressed = Vec::new();
        decoder.read_to_end(&mut decompressed)
            .map_err(|e| VectorizerError::Storage(format!("Gzip decompression failed: {}", e)))?;
        
        Ok(decompressed)
    }
    
    /// Read collection metadata
    fn read_collection_metadata(&self, path: &Path) -> Result<CollectionMetadata> {
        let content = fs::read_to_string(path)
            .map_err(|e| VectorizerError::Io(e))?;
        
        let metadata: CollectionMetadata = serde_json::from_str(&content)
            .map_err(|e| VectorizerError::Deserialization(e.to_string()))?;
        
        Ok(metadata)
    }
}

/// Simplified collection metadata for extraction
#[derive(Debug, serde::Deserialize)]
struct CollectionMetadata {
    #[serde(default)]
    vector_count: usize,
    #[serde(default)]
    dimension: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use std::io::Write;

    #[test]
    fn test_writer_creation() {
        let temp_dir = TempDir::new().unwrap();
        let writer = StorageWriter::new(temp_dir.path(), 3);
        assert_eq!(writer.compression_level, 3);
    }

    #[test]
    fn test_calculate_checksum() {
        let temp_dir = TempDir::new().unwrap();
        let writer = StorageWriter::new(temp_dir.path(), 3);
        
        let data = b"test data";
        let checksum = writer.calculate_checksum(data);
        assert!(!checksum.is_empty());
        assert_eq!(checksum.len(), 64); // SHA-256 produces 64 hex chars
    }

    #[test]
    fn test_write_archive() {
        let temp_dir = TempDir::new().unwrap();
        let data_dir = temp_dir.path().join("data");
        let collections_dir = data_dir.join("collections");
        fs::create_dir_all(&collections_dir).unwrap();
        
        // Create test collection file in the correct format
        let test_file = collections_dir.join("test_collection_vector_store.bin");
        let mut file = File::create(&test_file).unwrap();
        file.write_all(b"test vector data").unwrap();
        
        let writer = StorageWriter::new(&data_dir, 3);
        let result = writer.write_archive(&collections_dir);
        
        assert!(result.is_ok());
        let index = result.unwrap();
        assert!(index.collections.len() > 0);
    }
}

