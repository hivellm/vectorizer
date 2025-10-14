//! Storage writer for creating and updating .vecdb archives

use crate::error::{Result, VectorizerError};
use crate::storage::{CollectionIndex, StorageIndex};
use crate::storage::index::{FileEntry, FileType, detect_file_type};
use sha2::{Sha256, Digest};
use std::fs::{self, File};
use std::io::{self, Write, Read};
use std::path::{Path, PathBuf};
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
    
    /// Discover collections from flat file structure
    fn discover_collections(&self, source_dir: &Path) -> Result<std::collections::HashMap<String, Vec<PathBuf>>> {
        use std::collections::HashMap;
        
        let mut collections: HashMap<String, Vec<PathBuf>> = HashMap::new();
        
        for entry in fs::read_dir(source_dir).map_err(|e| VectorizerError::Io(e))? {
            let entry = entry.map_err(|e| VectorizerError::Io(e))?;
            let path = entry.path();
            
            if path.is_file() {
                if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                    // Extract collection name (everything before the last underscore)
                    if let Some(pos) = name.rfind('_') {
                        let collection_name = &name[..pos];
                        collections.entry(collection_name.to_string())
                            .or_insert_with(Vec::new)
                            .push(path.clone());
                    }
                }
            }
        }
        
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
            
            // Archive path: data/collection_name/filename
            let archive_path = PathBuf::from("data")
                .join(collection_name)
                .join(file_name);
            
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
        
        // Create test collection
        let collection_dir = collections_dir.join("test_collection");
        fs::create_dir_all(&collection_dir).unwrap();
        
        let test_file = collection_dir.join("test.bin");
        let mut file = File::create(&test_file).unwrap();
        file.write_all(b"test vector data").unwrap();
        
        let writer = StorageWriter::new(&data_dir, 3);
        let result = writer.write_archive(&collections_dir);
        
        assert!(result.is_ok());
        let index = result.unwrap();
        assert!(index.collections.len() > 0);
    }
}

