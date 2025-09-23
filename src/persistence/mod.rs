//! Persistence module for saving and loading vector stores

use crate::{
    db::VectorStore,
    error::{Result, VectorizerError},
    models::{CollectionConfig, Payload, Vector},
};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use tracing::{debug, info};

/// Persisted representation of a vector store
#[derive(Serialize, Deserialize)]
pub struct PersistedVectorStore {
    /// Version for backward compatibility
    version: u32,
    /// Collections
    collections: Vec<PersistedCollection>,
}

/// Persisted representation of a collection
#[derive(Serialize, Deserialize)]
pub struct PersistedCollection {
    /// Collection name
    name: String,
    /// Collection configuration
    config: CollectionConfig,
    /// Vectors in the collection
    vectors: Vec<PersistedVector>,
}

/// Persisted representation of a vector with payload serialized as JSON bytes
#[derive(Serialize, Deserialize)]
pub struct PersistedVector {
    id: String,
    data: Vec<f32>,
    /// Payload serialized as compact JSON bytes to satisfy bincode length requirements
    payload_json: Option<Vec<u8>>,
}

impl From<Vector> for PersistedVector {
    fn from(v: Vector) -> Self {
        let payload_json = v
            .payload
            .as_ref()
            .map(|p| serde_json::to_vec(&p.data))
            .transpose()
            .ok()
            .flatten();

        PersistedVector {
            id: v.id,
            data: v.data,
            payload_json,
        }
    }
}

impl PersistedVector {
    fn into_runtime(self) -> Result<Vector> {
        let payload = match self.payload_json {
            Some(bytes) => {
                let value: serde_json::Value = serde_json::from_slice(&bytes)?;
                Some(Payload::new(value))
            }
            None => None,
        };

        Ok(Vector {
            id: self.id,
            data: self.data,
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

            collections.push(PersistedCollection {
                name: collection_name,
                config: metadata.config,
                vectors,
            });
        }

        // Preserve original collection order for consistency

        let persisted = PersistedVectorStore {
            version: 1,
            collections,
        };

        // Serialize with bincode
        let data = bincode::serialize(&persisted)?;

        // Write to file
        fs::write(path, data)?;

        info!("Vector store saved successfully");
        Ok(())
    }

    /// Load a vector store from a file
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path = path.as_ref();
        info!("Loading vector store from {:?}", path);

        // Read file
        let data = fs::read(path)?;

        // Deserialize with bincode
        let persisted: PersistedVectorStore = bincode::deserialize(&data)?;

        // Check version
        if persisted.version != 1 {
            return Err(VectorizerError::Other(format!(
                "Unsupported vector store version: {}",
                persisted.version
            )));
        }

        // Create new vector store
        let store = VectorStore::new();

        // Restore collections
        for collection in persisted.collections {
            store.create_collection(&collection.name, collection.config)?;

            if !collection.vectors.is_empty() {
                // Reconstruct runtime vectors
                let mut runtime_vectors = Vec::with_capacity(collection.vectors.len());
                for pv in collection.vectors {
                    runtime_vectors.push(pv.into_runtime()?);
                }
                store.insert(&collection.name, runtime_vectors)?;
            }
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
            quantization: None,
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
