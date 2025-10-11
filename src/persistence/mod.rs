//! Persistence module for saving and loading vector stores

use crate::{
    db::VectorStore,
    error::{Result, VectorizerError},
    models::{CollectionConfig, Payload, Vector},
};
use serde::{Deserialize, Serialize};
use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::Path;
use flate2::Compression;
use flate2::read::GzDecoder;
use flate2::write::GzEncoder;
use tracing::{debug, error, info, warn};

// New persistence system modules
pub mod types;
pub mod wal;
pub mod dynamic;
pub mod enhanced_store;

// Demo tests
#[cfg(test)]
mod demo_test;
#[cfg(test)]
mod debug_test;

/// Persisted representation of a vector store
#[derive(Serialize, Deserialize)]
pub struct PersistedVectorStore {
    /// Version for backward compatibility
    pub version: u32,
    /// Collections
    pub collections: Vec<PersistedCollection>,
}

/// Persisted representation of a collection
#[derive(Serialize, Deserialize)]
pub struct PersistedCollection {
    /// Collection name
    pub name: String,
    /// Collection configuration
    pub config: CollectionConfig,
    /// Vectors in the collection
    pub vectors: Vec<PersistedVector>,
    /// HNSW index dump basename (if available)
    pub hnsw_dump_basename: Option<String>,
}

/// Persisted representation of a vector with payload serialized as JSON string
#[derive(Clone, Serialize, Deserialize)]
pub struct PersistedVector {
    id: String,
    data: Vec<f32>,
    /// Payload serialized as JSON string to satisfy bincode requirements
    payload_json: Option<String>,
    /// Whether the vector data is already normalized for cosine similarity
    normalized: bool,
}

impl From<Vector> for PersistedVector {
    fn from(v: Vector) -> Self {
        let payload_json = v
            .payload
            .as_ref()
            .and_then(|p| serde_json::to_string(&p.data).ok());

        // Check if vector is already normalized (for cosine similarity)
        let norm_squared: f32 = v.data.iter().map(|x| x * x).sum();
        let norm = norm_squared.sqrt();
        let normalized = (norm - 1.0).abs() <= 1e-6;

        PersistedVector {
            id: v.id,
            data: v.data,
            payload_json,
            normalized,
        }
    }
}

impl From<PersistedVector> for Vector {
    fn from(pv: PersistedVector) -> Self {
        // Try to use the existing into_runtime method first
        match pv.into_runtime() {
            Ok(vector) => vector,
            Err(_) => {
                // This should never happen since PersistedVector is created from Vector
                // But provide a fallback just in case
                Vector {
                    id: String::new(), // Can't access pv.id since it's moved
                    data: Vec::new(),   // Can't access pv.data since it's moved
                    payload: None,
                }
            }
        }
    }
}

impl PersistedVector {
    pub fn into_runtime(self) -> Result<Vector> {
        let payload = match self.payload_json {
            Some(json_str) => {
                let value: serde_json::Value = serde_json::from_str(&json_str)?;
                let mut payload = Payload::new(value);
                // Normalize payload content (fix line endings from legacy data)
                payload.normalize();
                Some(payload)
            }
            None => None,
        };

        Ok(Vector {
            id: self.id,
            data: self.data,
            payload,
        })
    }

    /// Convert to runtime vector with payload, skipping normalization if already normalized
    pub fn into_runtime_with_payload(self) -> Result<Vector> {
        let payload = match self.payload_json {
            Some(json_str) => {
                let value: serde_json::Value = serde_json::from_str(&json_str)?;
                let mut payload = Payload::new(value);
                // Normalize payload content (fix line endings from legacy data)
                payload.normalize();
                Some(payload)
            }
            None => None,
        };

        // If already normalized, use data as-is; otherwise normalize
        let data = if self.normalized {
            self.data
        } else {
            // Apply normalization for cosine similarity
            let norm_squared: f32 = self.data.iter().map(|x| x * x).sum();
            let norm = norm_squared.sqrt();
            if norm == 0.0 {
                self.data
            } else {
                self.data.iter().map(|x| x / norm).collect()
            }
        };

        Ok(Vector {
            id: self.id,
            data,
            payload,
        })
    }
}

impl VectorStore {
    /// Save the vector store to a file
    pub fn save<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let path = path.as_ref();
        info!("Saving vector store to {:?}", path);

        // Build persisted representation
        let mut collections = Vec::new();

        for collection_name in self.list_collections() {
            let collection = self.get_collection(&collection_name)?;
            let metadata = collection.metadata();

            // Collect all vectors from the collection and convert to persisted representation
            // Note: We preserve the original insertion order to maintain HNSW index consistency
            let vectors: Vec<PersistedVector> = collection
                .get_all_vectors()
                .into_iter()
                .map(PersistedVector::from)
                .collect();

            // HNSW dump will be done after loading vectors into index, not during save
            let hnsw_dump_basename = None;

            collections.push(PersistedCollection {
                name: collection_name,
                config: metadata.config,
                vectors,
                hnsw_dump_basename,
            });
        }

        // Preserve original collection order for consistency

        let persisted = PersistedVectorStore {
            version: 1,
            collections,
        };

        // Serialize with JSON (for better compatibility with serde_json::Value)
        let json_data = serde_json::to_string(&persisted)?; // Use to_string instead of to_string_pretty for better compression
        
        // Compress with gzip for better storage efficiency
        let file = File::create(path)?;
        let mut encoder = GzEncoder::new(file, Compression::best());
        encoder.write_all(json_data.as_bytes())?;
        encoder.finish()?;

        let original_size = json_data.len();
        let compressed_size = fs::metadata(path)?.len();
        let compression_ratio = (1.0 - (compressed_size as f64 / original_size as f64)) * 100.0;
        
        info!(
            "Vector store saved successfully (Original: {} bytes, Compressed: {} bytes, Ratio: {:.1}%)",
            original_size, compressed_size, compression_ratio
        );
        Ok(())
    }

    /// Load a vector store from a file
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path = path.as_ref();
        info!("Loading vector store from {:?}", path);

        // Try to read as gzip compressed file first
        let (json_data, was_compressed) = match File::open(path) {
            Ok(file) => {
                let mut decoder = GzDecoder::new(file);
                let mut json_string = String::new();
                
                // Try to decompress - if it fails, try reading as plain text
                match decoder.read_to_string(&mut json_string) {
                    Ok(_) => {
                        info!("Loaded compressed vector store");
                        (json_string, true)
                    }
                    Err(_) => {
                        // Not a gzip file, try reading as plain text (backward compatibility)
                        warn!("Loaded uncompressed vector store - will auto-migrate to compressed format");
                        (fs::read_to_string(path)?, false)
                    }
                }
            }
            Err(e) => return Err(VectorizerError::Other(format!("Failed to open file: {}", e))),
        };

        // Deserialize with JSON (for better compatibility with serde_json::Value)
        let persisted: PersistedVectorStore = serde_json::from_str(&json_data)?;

        // Check version
        if persisted.version != 1 {
            return Err(VectorizerError::Other(format!(
                "Unsupported vector store version: {}",
                persisted.version
            )));
        }

        // Create new vector store (CPU-only to avoid recursion)
        let store = VectorStore::new();

        // Restore collections with fast loading and automatic quantization
        for collection in persisted.collections {
            // Create collection config with quantization enabled
            let mut config = collection.config.clone();
            config.quantization = crate::models::QuantizationConfig::SQ { bits: 8 };
            
            store.create_collection_with_quantization(&collection.name, config)?;

            if !collection.vectors.is_empty() {
                // Load vectors from cache (HNSW dump not implemented yet)
                store.load_collection_from_cache(&collection.name, collection.vectors)?;
            }
        }

        // Note: Auto-migration removed to prevent memory duplication
        // Uncompressed files will be saved compressed on next manual save
        if !was_compressed {
            info!("📦 Loaded uncompressed store - will be saved compressed on next save operation");
        }

        info!("Vector store loaded successfully");
        Ok(store)
    }
}

/// Persistence manager for handling compressed saves
pub struct PersistenceManager {
    /// Whether to use compression
    compress: bool,
}

impl PersistenceManager {
    /// Create a new persistence manager
    pub fn new(compress: bool) -> Self {
        Self { compress }
    }

    /// Save data with optional compression
    pub fn save<T: Serialize, P: AsRef<Path>>(&self, data: &T, path: P) -> Result<()> {
        let serialized = bincode::serialize(data)?;

        let final_data = if self.compress {
            debug!("Compressing data before saving");
            lz4_flex::compress_prepend_size(&serialized)
        } else {
            serialized
        };

        fs::write(path, final_data)?;
        Ok(())
    }

    /// Load data with automatic decompression detection
    pub fn load<T: for<'de> Deserialize<'de>, P: AsRef<Path>>(&self, path: P) -> Result<T> {
        let data = fs::read(path)?;

        // Try to decompress first
        let decompressed = if data.len() >= 4 {
            // Check if it looks like LZ4 compressed data
            match lz4_flex::decompress_size_prepended(&data) {
                Ok(decompressed) => {
                    debug!("Data was compressed, decompressed successfully");
                    decompressed
                }
                Err(_) => {
                    debug!("Data was not compressed");
                    data
                }
            }
        } else {
            data
        };

        let deserialized = bincode::deserialize(&decompressed)?;
        Ok(deserialized)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{DistanceMetric, HnswConfig};
    use tempfile::tempdir;

    #[test]
    fn test_save_and_load_empty_store() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("test.vdb");

        // Save empty store
        let store = VectorStore::new();
        store.save(&path).unwrap();

        // Load and verify
        let loaded = VectorStore::load(&path).unwrap();
        assert_eq!(loaded.list_collections().len(), 0);
    }

    #[test]
    fn test_save_and_load_with_collections() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("test.vdb");

        // Create store with collections
        let store = VectorStore::new();
        let config = CollectionConfig {
            dimension: 128,
            metric: DistanceMetric::Cosine,
            hnsw_config: HnswConfig::default(),
            quantization: crate::models::QuantizationConfig::default(),
            compression: Default::default(),
        };

        store.create_collection("test1", config.clone()).unwrap();
        store.create_collection("test2", config).unwrap();

        // Save
        store.save(&path).unwrap();

        // Load and verify
        let loaded = VectorStore::load(&path).unwrap();
        let collections = loaded.list_collections();
        assert_eq!(collections.len(), 2);
        assert!(collections.contains(&"test1".to_string()));
        assert!(collections.contains(&"test2".to_string()));
    }

    #[test]
    fn test_persistence_manager_compression() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("test.bin");

        // Test data
        let data = vec![1u8; 1000];

        // Save with compression
        let manager = PersistenceManager::new(true);
        manager.save(&data, &path).unwrap();

        // Check that file is smaller than original
        let file_size = fs::metadata(&path).unwrap().len();
        assert!(file_size < 1000);

        // Load and verify
        let loaded: Vec<u8> = manager.load(&path).unwrap();
        assert_eq!(loaded, data);
    }

    #[test]
    fn test_persistence_manager_without_compression() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("test_uncompressed.bin");

        // Test data
        let data = vec![42u8; 500];

        // Save without compression
        let manager = PersistenceManager::new(false);
        manager.save(&data, &path).unwrap();

        // File should be same size (plus some overhead for bincode)
        let file_size = fs::metadata(&path).unwrap().len();
        assert!(file_size >= 500);

        // Load and verify
        let loaded: Vec<u8> = manager.load(&path).unwrap();
        assert_eq!(loaded, data);
    }

    #[test]
    fn test_persistence_manager_auto_decompression() {
        let dir = tempdir().unwrap();
        let path_compressed = dir.path().join("compressed.bin");
        let path_uncompressed = dir.path().join("uncompressed.bin");

        let data = vec![7u8; 2000];

        // Save both compressed and uncompressed versions
        let manager_compressed = PersistenceManager::new(true);
        let manager_uncompressed = PersistenceManager::new(false);

        manager_compressed.save(&data, &path_compressed).unwrap();
        manager_uncompressed
            .save(&data, &path_uncompressed)
            .unwrap();

        // Both should load correctly regardless of compression setting
        let loaded_compressed: Vec<u8> = manager_compressed.load(&path_compressed).unwrap();
        let loaded_uncompressed: Vec<u8> = manager_uncompressed.load(&path_uncompressed).unwrap();
        let cross_loaded_compressed: Vec<u8> = manager_uncompressed.load(&path_compressed).unwrap();
        let cross_loaded_uncompressed: Vec<u8> =
            manager_compressed.load(&path_uncompressed).unwrap();

        assert_eq!(loaded_compressed, data);
        assert_eq!(loaded_uncompressed, data);
        assert_eq!(cross_loaded_compressed, data);
        assert_eq!(cross_loaded_uncompressed, data);

        // Compressed file should be smaller
        let compressed_size = fs::metadata(&path_compressed).unwrap().len();
        let uncompressed_size = fs::metadata(&path_uncompressed).unwrap().len();
        assert!(compressed_size < uncompressed_size);
    }
}

// Include comprehensive tests module
#[cfg(test)]
mod persistence_tests {
    include!("tests.rs");
}
