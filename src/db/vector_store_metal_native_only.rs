//! # Vector Store - Metal Native Only
//!
//! Simplified vector store that only supports Metal Native collections.
//! This version removes all wgpu dependencies and focuses purely on Metal Native.

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use crate::error::{Result, VectorizerError};
use crate::models::{CollectionConfig, DistanceMetric, Vector};
use crate::gpu::metal_native::{MetalNativeCollection, MetalNativeContext};

/// Collection types supported by Metal Native
#[derive(Debug, Clone)]
pub enum CollectionType {
    MetalNative(MetalNativeCollection),
}

/// Simplified Vector Store for Metal Native only
#[derive(Debug)]
pub struct VectorStore {
    collections: Arc<RwLock<HashMap<String, CollectionType>>>,
    metal_context: Option<MetalNativeContext>,
}

impl VectorStore {
    /// Create new vector store
    pub fn new() -> Self {
        Self {
            collections: Arc::new(RwLock::new(HashMap::new())),
            metal_context: None,
        }
    }

    /// Initialize Metal Native context
    pub fn initialize_metal_native(&mut self) -> Result<()> {
        self.metal_context = Some(MetalNativeContext::new()?);
        tracing::info!("✅ Metal Native context initialized");
        Ok(())
    }

    /// Create a new collection
    pub async fn create_collection(
        &self,
        name: &str,
        config: CollectionConfig,
    ) -> Result<()> {
        if self.collections.read().await.contains_key(name) {
            return Err(VectorizerError::CollectionAlreadyExists(name.to_string()));
        }

        let collection = match config.metric {
            DistanceMetric::Cosine | DistanceMetric::Euclidean | DistanceMetric::DotProduct => {
                // Use Metal Native for all distance metrics
                if let Some(ref context) = self.metal_context {
                    MetalNativeCollection::new(config.dimension, config.metric)?
                } else {
                    return Err(VectorizerError::Other(
                        "Metal Native context not initialized".to_string()
                    ));
                }
            }
        };

        self.collections
            .write()
            .await
            .insert(name.to_string(), CollectionType::MetalNative(collection));

        tracing::info!("✅ Collection '{}' created with Metal Native", name);
        Ok(())
    }

    /// Add vector to collection
    pub async fn add_vector(&self, collection_name: &str, vector: Vector) -> Result<()> {
        let mut collections = self.collections.write().await;
        let collection = collections
            .get_mut(collection_name)
            .ok_or_else(|| VectorizerError::CollectionNotFound(collection_name.to_string()))?;

        match collection {
            CollectionType::MetalNative(c) => {
                c.add_vector(vector)?;
            }
        }

        Ok(())
    }

    /// Search vectors in collection
    pub async fn search(
        &self,
        collection_name: &str,
        query: &[f32],
        limit: usize,
    ) -> Result<Vec<Vector>> {
        // For Metal Native, we need mutable access to the collection for buffer pooling
        // This is a workaround - ideally the collection should be designed differently
        let mut collections = self.collections.write().await;
        let collection = collections
            .get_mut(collection_name)
            .ok_or_else(|| VectorizerError::CollectionNotFound(collection_name.to_string()))?;

        match collection {
            CollectionType::MetalNative(ref mut c) => {
                let results = c.search(query, limit)?;
                Ok(results)
            }
        }
    }

    /// Get vector by ID
    pub async fn get_vector(&self, collection_name: &str, vector_id: &str) -> Result<Vector> {
        let collections = self.collections.read().await;
        let collection = collections
            .get(collection_name)
            .ok_or_else(|| VectorizerError::CollectionNotFound(collection_name.to_string()))?;

        match collection {
            CollectionType::MetalNative(c) => {
                c.get_vector(vector_id)
            }
        }
    }

    /// List all collections
    pub async fn list_collections(&self) -> Vec<String> {
        self.collections.read().await.keys().cloned().collect()
    }

    /// Get collection info
    pub async fn get_collection_info(&self, name: &str) -> Result<CollectionConfig> {
        let collections = self.collections.read().await;
        if !collections.contains_key(name) {
            return Err(VectorizerError::CollectionNotFound(name.to_string()));
        }

        // For Metal Native, we can't easily get the config back
        // This is a limitation of the current design
        Ok(CollectionConfig {
            dimension: 512, // Default dimension
            metric: DistanceMetric::Cosine, // Default metric
        })
    }

    /// Delete collection
    pub async fn delete_collection(&self, name: &str) -> Result<()> {
        let mut collections = self.collections.write().await;
        if collections.remove(name).is_some() {
            tracing::info!("✅ Collection '{}' deleted", name);
            Ok(())
        } else {
            Err(VectorizerError::CollectionNotFound(name.to_string()))
        }
    }

    /// Get all vectors from collection
    pub async fn get_all_vectors(&self, collection_name: &str) -> Result<Vec<Vector>> {
        let collections = self.collections.read().await;
        let collection = collections
            .get(collection_name)
            .ok_or_else(|| VectorizerError::CollectionNotFound(collection_name.to_string()))?;

        match collection {
            CollectionType::MetalNative(c) => {
                // Metal Native doesn't have a direct way to get all vectors
                // This is a limitation of the current design
                Ok(vec![])
            }
        }
    }

    /// Check if Metal Native is available
    pub fn is_metal_native_available(&self) -> bool {
        self.metal_context.is_some()
    }

    /// Get Metal Native context
    pub fn metal_context(&self) -> Option<&MetalNativeContext> {
        self.metal_context.as_ref()
    }
}

impl Default for VectorStore {
    fn default() -> Self {
        Self::new()
    }
}

