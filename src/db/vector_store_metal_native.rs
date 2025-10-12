//! Vector Store implementation for Metal Native only
//! 
//! This is a simplified version that only supports Metal Native collections
//! when the metal-native feature is enabled.

use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use std::path::PathBuf;
use tracing::{debug, info, warn};

use super::collection::Collection;
use crate::gpu::{MetalNativeCollection, MetalNativeHnswGraph};
use crate::models::{CollectionConfig, DistanceMetric, Vector};
use crate::error::{Result, VectorizerError};

/// Simplified Vector Store for Metal Native only
pub struct VectorStore {
    /// Collections stored in this vector store
    collections: HashMap<String, CollectionType>,
    /// Storage path for persistence
    storage_path: PathBuf,
}

/// Collection types supported by Metal Native
pub enum CollectionType {
    /// CPU-based collection
    Cpu(Collection),
    /// Metal Native collection (Apple Silicon)
    MetalNative(MetalNativeCollection),
}

impl VectorStore {
    /// Create a new vector store
    pub fn new() -> Self {
        Self {
            collections: HashMap::new(),
            storage_path: PathBuf::from("./data"),
        }
    }

    /// Create a new collection
    pub fn create_collection(&mut self, name: &str, config: CollectionConfig) -> Result<()> {
        if self.collections.contains_key(name) {
            return Err(VectorizerError::CollectionAlreadyExists(name.to_string()));
        }

        info!("Creating collection '{}' with Metal Native", name);

        // Create Metal Native collection
        let collection = MetalNativeCollection::new(config.dimension, config.metric)?;
        self.collections.insert(name.to_string(), CollectionType::MetalNative(collection));

        info!("✅ Collection '{}' created successfully with Metal Native", name);
        Ok(())
    }

    /// Add a vector to a collection
    pub fn add_vector(&mut self, collection_name: &str, vector: Vector) -> Result<usize> {
        let collection = self.collections.get_mut(collection_name)
            .ok_or_else(|| VectorizerError::CollectionNotFound(collection_name.to_string()))?;

        match collection {
            CollectionType::Cpu(c) => c.add_vector(vector),
            CollectionType::MetalNative(c) => c.add_vector(vector),
        }
    }

    /// Search for similar vectors
    pub fn search(&self, collection_name: &str, query: &[f32], limit: usize) -> Result<Vec<(usize, f32)>> {
        let collection = self.collections.get_mut(collection_name)
            .ok_or_else(|| VectorizerError::CollectionNotFound(collection_name.to_string()))?;

        match collection {
            CollectionType::Cpu(c) => c.search(query, limit),
            CollectionType::MetalNative(ref mut c) => c.search(query, limit),
        }
    }

    /// Get a vector by ID
    pub fn get_vector(&self, collection_name: &str, vector_id: &str) -> Result<Vector> {
        let collection = self.collections.get(collection_name)
            .ok_or_else(|| VectorizerError::CollectionNotFound(collection_name.to_string()))?;

        match collection {
            CollectionType::Cpu(c) => c.get_vector(vector_id),
            CollectionType::MetalNative(c) => {
                // For Metal Native, we need to find the vector by ID
                // This is a simplified implementation
                Err(VectorizerError::Other("Vector retrieval not implemented for Metal Native".to_string()))
            }
        }
    }

    /// Get all vectors from a collection
    pub fn get_all_vectors(&self, collection_name: &str) -> Result<Vec<Vector>> {
        let collection = self.collections.get(collection_name)
            .ok_or_else(|| VectorizerError::CollectionNotFound(collection_name.to_string()))?;

        match collection {
            CollectionType::Cpu(c) => Ok(c.get_all_vectors()),
            CollectionType::MetalNative(_) => {
                // For Metal Native, this is not implemented yet
                Err(VectorizerError::Other("Get all vectors not implemented for Metal Native".to_string()))
            }
        }
    }

    /// List all collection names
    pub fn list_collections(&self) -> Vec<String> {
        self.collections.keys().cloned().collect()
    }

    /// Delete a collection
    pub fn delete_collection(&mut self, name: &str) -> Result<()> {
        if self.collections.remove(name).is_some() {
            info!("✅ Collection '{}' deleted", name);
            Ok(())
        } else {
            Err(VectorizerError::CollectionNotFound(name.to_string()))
        }
    }

    /// Get collection info
    pub fn get_collection_info(&self, name: &str) -> Result<CollectionInfo> {
        let collection = self.collections.get(name)
            .ok_or_else(|| VectorizerError::CollectionNotFound(name.to_string()))?;

        match collection {
            CollectionType::Cpu(c) => Ok(CollectionInfo {
                name: name.to_string(),
                vector_count: c.vector_count(),
                dimension: c.dimension(),
                metric: c.metric(),
            }),
            CollectionType::MetalNative(c) => Ok(CollectionInfo {
                name: name.to_string(),
                vector_count: c.vector_count(),
                dimension: c.dimension(),
                metric: c.metric(),
            }),
        }
    }
}

/// Collection information
#[derive(Debug, Clone)]
pub struct CollectionInfo {
    pub name: String,
    pub vector_count: usize,
    pub dimension: usize,
    pub metric: DistanceMetric,
}

impl Default for VectorStore {
    fn default() -> Self {
        Self::new()
    }
}

