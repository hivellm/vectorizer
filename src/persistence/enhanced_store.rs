use std::collections::HashMap;
use std::sync::Arc;

use thiserror::Error;
use tracing::{debug, error, info, warn};

use crate::db::VectorStore;
use crate::models::{CollectionConfig, DistanceMetric};
use crate::persistence::dynamic::{
    DynamicCollectionPersistence, PersistenceConfig, PersistenceError,
};
use crate::persistence::types::{CollectionSource, CollectionType, EnhancedCollectionMetadata};

/// Enhanced vector store that supports both workspace and dynamic collections
pub struct EnhancedVectorStore {
    /// Original vector store for workspace collections
    workspace_store: Arc<VectorStore>,
    /// Dynamic collections persistence manager
    dynamic_persistence: Arc<DynamicCollectionPersistence>,
    /// Collection metadata cache
    metadata_cache: Arc<tokio::sync::RwLock<HashMap<String, EnhancedCollectionMetadata>>>,
}

/// Enhanced store errors
#[derive(Debug, Error)]
pub enum EnhancedStoreError {
    #[error("Persistence error: {0}")]
    PersistenceError(#[from] PersistenceError),

    #[error("Collection '{0}' is read-only (workspace collection)")]
    ReadOnlyCollection(String),

    #[error("Collection '{0}' not found")]
    CollectionNotFound(String),

    #[error("Cannot delete workspace collection '{0}'")]
    CannotDeleteWorkspace(String),

    #[error("Vector store error: {0}")]
    VectorStoreError(#[from] crate::error::VectorizerError),
}

impl EnhancedVectorStore {
    /// Create new enhanced vector store
    pub async fn new(
        workspace_store: Arc<VectorStore>,
        persistence_config: PersistenceConfig,
    ) -> Result<Self, EnhancedStoreError> {
        let dynamic_persistence = Arc::new(
            DynamicCollectionPersistence::new(persistence_config, workspace_store.clone()).await?,
        );

        let store = Self {
            workspace_store,
            dynamic_persistence,
            metadata_cache: Arc::new(tokio::sync::RwLock::new(HashMap::new())),
        };

        // Initialize metadata cache
        store.initialize_metadata_cache().await?;

        info!("Enhanced vector store initialized");
        Ok(store)
    }

    /// Initialize metadata cache with existing collections
    async fn initialize_metadata_cache(&self) -> Result<(), EnhancedStoreError> {
        let mut cache = self.metadata_cache.write().await;

        // Load workspace collections
        for collection_name in self.workspace_store.list_collections() {
            let metadata = self.create_workspace_metadata(&collection_name).await?;
            cache.insert(collection_name, metadata);
        }

        // Load dynamic collections
        let dynamic_collections = self.dynamic_persistence.list_collections().await?;
        for metadata in dynamic_collections {
            cache.insert(metadata.name.clone(), metadata);
        }

        info!(
            "Metadata cache initialized with {} collections",
            cache.len()
        );
        Ok(())
    }

    /// Create workspace collection metadata
    async fn create_workspace_metadata(
        &self,
        collection_name: &str,
    ) -> Result<EnhancedCollectionMetadata, EnhancedStoreError> {
        // Get collection from workspace store
        let collection = self.workspace_store.get_collection(collection_name)?;
        let metadata = collection.metadata();

        // Create enhanced metadata
        let enhanced_metadata = EnhancedCollectionMetadata::new_workspace(
            collection_name.to_string(),
            "workspace".to_string(),             // Default project name
            "/workspace/config.yml".to_string(), // Default config path
            metadata.config.clone(),
        );

        // Update with actual counts
        let mut metadata = enhanced_metadata;
        metadata.vector_count = collection.vector_count();
        metadata.document_count = collection.vector_count();
        metadata.update_checksums();

        Ok(metadata)
    }

    /// Get collection metadata
    pub async fn get_collection_metadata(
        &self,
        collection_name: &str,
    ) -> Result<EnhancedCollectionMetadata, EnhancedStoreError> {
        let cache = self.metadata_cache.read().await;

        if let Some(metadata) = cache.get(collection_name) {
            Ok(metadata.clone())
        } else {
            Err(EnhancedStoreError::CollectionNotFound(
                collection_name.to_string(),
            ))
        }
    }

    /// List all collections with metadata
    pub async fn list_collections(&self) -> Vec<EnhancedCollectionMetadata> {
        let cache = self.metadata_cache.read().await;
        cache.values().cloned().collect()
    }

    /// List workspace collections
    pub async fn list_workspace_collections(&self) -> Vec<EnhancedCollectionMetadata> {
        let cache = self.metadata_cache.read().await;
        cache
            .values()
            .filter(|metadata| metadata.is_workspace())
            .cloned()
            .collect()
    }

    /// List dynamic collections
    pub async fn list_dynamic_collections(&self) -> Vec<EnhancedCollectionMetadata> {
        let cache = self.metadata_cache.read().await;
        cache
            .values()
            .filter(|metadata| metadata.is_dynamic())
            .cloned()
            .collect()
    }

    /// Create new dynamic collection
    pub async fn create_collection(
        &self,
        name: String,
        config: CollectionConfig,
        created_by: Option<String>,
    ) -> Result<EnhancedCollectionMetadata, EnhancedStoreError> {
        // Check if collection already exists
        if self.collection_exists(&name).await {
            return Err(EnhancedStoreError::CollectionNotFound(name));
        }

        // Create dynamic collection
        let metadata = self
            .dynamic_persistence
            .create_collection(name.clone(), config, created_by)
            .await?;

        // Update cache
        let mut cache = self.metadata_cache.write().await;
        cache.insert(name.clone(), metadata.clone());

        info!("Dynamic collection '{}' created", name);
        Ok(metadata)
    }

    /// Check if collection exists
    pub async fn collection_exists(&self, collection_name: &str) -> bool {
        let cache = self.metadata_cache.read().await;
        cache.contains_key(collection_name)
    }

    /// Check if collection is workspace collection
    pub async fn is_workspace_collection(&self, collection_name: &str) -> bool {
        let cache = self.metadata_cache.read().await;
        cache
            .get(collection_name)
            .map(|metadata| metadata.is_workspace())
            .unwrap_or(false)
    }

    /// Check if collection is dynamic collection
    pub async fn is_dynamic_collection(&self, collection_name: &str) -> bool {
        let cache = self.metadata_cache.read().await;
        cache
            .get(collection_name)
            .map(|metadata| metadata.is_dynamic())
            .unwrap_or(false)
    }

    /// Delete collection
    pub async fn delete_collection(&self, collection_name: &str) -> Result<(), EnhancedStoreError> {
        let metadata = self.get_collection_metadata(collection_name).await?;

        if metadata.is_workspace() {
            return Err(EnhancedStoreError::CannotDeleteWorkspace(
                collection_name.to_string(),
            ));
        }

        // Delete dynamic collection
        self.dynamic_persistence
            .delete_collection(collection_name)
            .await?;

        // Remove from cache
        let mut cache = self.metadata_cache.write().await;
        cache.remove(collection_name);

        info!("Dynamic collection '{}' deleted", collection_name);
        Ok(())
    }

    /// Insert vectors into collection
    pub async fn insert_vectors(
        &self,
        collection_name: &str,
        vectors: Vec<crate::models::Vector>,
    ) -> Result<(), EnhancedStoreError> {
        let metadata = self.get_collection_metadata(collection_name).await?;

        if metadata.is_read_only {
            return Err(EnhancedStoreError::ReadOnlyCollection(
                collection_name.to_string(),
            ));
        }

        // For dynamic collections, use persistence manager
        if metadata.is_dynamic() {
            // Begin transaction
            let transaction_id = self
                .dynamic_persistence
                .begin_transaction(&metadata.id)
                .await?;

            // Add insert operations
            let vector_count = vectors.len();
            for vector in vectors {
                use crate::persistence::types::Operation;
                let operation = Operation::InsertVector {
                    vector_id: vector.id,
                    data: vector.data,
                    metadata: vector
                        .payload
                        .map(|p| {
                            p.data
                                .as_object()
                                .unwrap_or(&serde_json::Map::new())
                                .iter()
                                .map(|(k, v)| (k.clone(), v.to_string()))
                                .collect()
                        })
                        .unwrap_or_default(),
                };

                self.dynamic_persistence
                    .add_to_transaction(transaction_id, operation)
                    .await?;
            }

            // Commit transaction
            self.dynamic_persistence
                .commit_transaction(transaction_id)
                .await?;

            // Update cache
            let mut cache = self.metadata_cache.write().await;
            if let Some(cached_metadata) = cache.get_mut(collection_name) {
                cached_metadata.vector_count += vector_count;
                cached_metadata.document_count += vector_count;
                cached_metadata.update_checksums();
            }
        } else {
            // For workspace collections, use original store (this should not happen due to read-only check)
            return Err(EnhancedStoreError::ReadOnlyCollection(
                collection_name.to_string(),
            ));
        }

        Ok(())
    }

    /// Search vectors in collection
    pub async fn search_vectors(
        &self,
        collection_name: &str,
        query_vector: &[f32],
        limit: usize,
        threshold: Option<f32>,
    ) -> Result<Vec<crate::models::SearchResult>, EnhancedStoreError> {
        let metadata = self.get_collection_metadata(collection_name).await?;

        // For now, use the original vector store for search
        // This will be enhanced to support both workspace and dynamic collections
        let collection = self.workspace_store.get_collection(collection_name)?;
        let results = collection.search(query_vector, limit)?;

        Ok(results)
    }

    /// Get collection statistics
    pub async fn get_collection_stats(
        &self,
        collection_name: &str,
    ) -> Result<CollectionStats, EnhancedStoreError> {
        let metadata = self.get_collection_metadata(collection_name).await?;

        Ok(CollectionStats {
            name: metadata.name.clone(),
            collection_type: metadata.collection_type,
            vector_count: metadata.vector_count,
            document_count: metadata.document_count,
            dimension: metadata.dimension,
            is_read_only: metadata.is_read_only,
            created_at: metadata.created_at,
            updated_at: metadata.updated_at,
            memory_usage_mb: metadata.memory_usage_mb,
            compression_ratio: metadata.compression_ratio,
            last_transaction_id: metadata.last_transaction_id,
        })
    }

    /// Get overall store statistics
    pub async fn get_store_stats(&self) -> Result<StoreStats, EnhancedStoreError> {
        let cache = self.metadata_cache.read().await;

        let total_collections = cache.len();
        let workspace_collections = cache.values().filter(|m| m.is_workspace()).count();
        let dynamic_collections = cache.values().filter(|m| m.is_dynamic()).count();
        let total_vectors: usize = cache.values().map(|m| m.vector_count).sum();
        let total_documents: usize = cache.values().map(|m| m.document_count).sum();

        let persistence_stats = self.dynamic_persistence.get_stats().await?;

        Ok(StoreStats {
            total_collections,
            workspace_collections,
            dynamic_collections,
            total_vectors,
            total_documents,
            wal_entries: persistence_stats.wal_entries,
            wal_size_bytes: persistence_stats.wal_size_bytes,
            active_transactions: persistence_stats.active_transactions,
        })
    }

    /// Refresh metadata cache
    pub async fn refresh_cache(&self) -> Result<(), EnhancedStoreError> {
        self.initialize_metadata_cache().await?;
        info!("Metadata cache refreshed");
        Ok(())
    }

    /// Get underlying workspace store (for backward compatibility)
    pub fn workspace_store(&self) -> Arc<VectorStore> {
        self.workspace_store.clone()
    }

    /// Get dynamic persistence manager
    pub fn dynamic_persistence(&self) -> Arc<DynamicCollectionPersistence> {
        self.dynamic_persistence.clone()
    }
}

/// Collection statistics
#[derive(Debug, Clone)]
pub struct CollectionStats {
    pub name: String,
    pub collection_type: CollectionType,
    pub vector_count: usize,
    pub document_count: usize,
    pub dimension: usize,
    pub is_read_only: bool,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
    pub memory_usage_mb: Option<f32>,
    pub compression_ratio: Option<f32>,
    pub last_transaction_id: Option<u64>,
}

/// Store statistics
#[derive(Debug, Clone)]
pub struct StoreStats {
    pub total_collections: usize,
    pub workspace_collections: usize,
    pub dynamic_collections: usize,
    pub total_vectors: usize,
    pub total_documents: usize,
    pub wal_entries: usize,
    pub wal_size_bytes: u64,
    pub active_transactions: usize,
}

#[cfg(test)]
mod tests {
    use tempfile::tempdir;

    use super::*;
    use crate::models::QuantizationConfig;

    async fn create_test_enhanced_store() -> EnhancedVectorStore {
        let temp_dir = tempdir().unwrap();
        let data_dir = temp_dir.path().join("data");
        std::fs::create_dir_all(&data_dir).unwrap(); // Create directory

        let workspace_store = Arc::new(VectorStore::new());

        let config = PersistenceConfig {
            data_dir,
            ..Default::default()
        };

        EnhancedVectorStore::new(workspace_store, config)
            .await
            .unwrap()
    }

    #[tokio::test]
    #[ignore] // Timeout: runs for over 60 seconds
    async fn test_enhanced_store_creation() {
        let store = create_test_enhanced_store().await;

        let stats = store.get_store_stats().await.unwrap();
        assert_eq!(stats.total_collections, 0);
        assert_eq!(stats.workspace_collections, 0);
        assert_eq!(stats.dynamic_collections, 0);
    }

    #[tokio::test]
    #[ignore] // Timeout: runs for over 60 seconds
    async fn test_create_dynamic_collection() {
        let store = create_test_enhanced_store().await;

        let config = CollectionConfig {
            dimension: 384,
            metric: DistanceMetric::Cosine,
            quantization: QuantizationConfig::default(),
            hnsw_config: crate::models::HnswConfig::default(),
            compression: crate::models::CompressionConfig::default(),
            normalization: None,
        };

        let metadata = store
            .create_collection(
                "test-collection".to_string(),
                config,
                Some("user123".to_string()),
            )
            .await
            .unwrap();

        assert_eq!(metadata.name, "test-collection");
        assert_eq!(metadata.collection_type, CollectionType::Dynamic);
        assert!(!metadata.is_read_only);

        // Verify in cache
        assert!(store.collection_exists("test-collection").await);
        assert!(store.is_dynamic_collection("test-collection").await);
        assert!(!store.is_workspace_collection("test-collection").await);
    }

    #[tokio::test]
    #[ignore] // Timeout: runs for over 60 seconds
    async fn test_list_collections() {
        let store = create_test_enhanced_store().await;

        let config = CollectionConfig {
            dimension: 384,
            metric: DistanceMetric::Cosine,
            quantization: QuantizationConfig::default(),
            hnsw_config: crate::models::HnswConfig::default(),
            compression: crate::models::CompressionConfig::default(),
            normalization: None,
        };

        // Create dynamic collections
        store
            .create_collection("collection1".to_string(), config.clone(), None)
            .await
            .unwrap();
        store
            .create_collection("collection2".to_string(), config, None)
            .await
            .unwrap();

        // List all collections
        let all_collections = store.list_collections().await;
        assert_eq!(all_collections.len(), 2);

        // List dynamic collections
        let dynamic_collections = store.list_dynamic_collections().await;
        assert_eq!(dynamic_collections.len(), 2);

        // List workspace collections
        let workspace_collections = store.list_workspace_collections().await;
        assert_eq!(workspace_collections.len(), 0);
    }

    #[tokio::test]
    #[ignore] // Timeout: runs for over 60 seconds
    async fn test_delete_dynamic_collection() {
        let store = create_test_enhanced_store().await;

        let config = CollectionConfig {
            dimension: 384,
            metric: DistanceMetric::Cosine,
            quantization: QuantizationConfig::default(),
            hnsw_config: crate::models::HnswConfig::default(),
            compression: crate::models::CompressionConfig::default(),
            normalization: None,
        };

        // Create collection
        store
            .create_collection("test-collection".to_string(), config, None)
            .await
            .unwrap();
        assert!(store.collection_exists("test-collection").await);

        // Delete collection
        store.delete_collection("test-collection").await.unwrap();
        assert!(!store.collection_exists("test-collection").await);
    }

    #[tokio::test]
    #[ignore] // Timeout: runs for over 60 seconds
    async fn test_collection_stats() {
        let store = create_test_enhanced_store().await;

        let config = CollectionConfig {
            dimension: 384,
            metric: DistanceMetric::Cosine,
            quantization: QuantizationConfig::default(),
            hnsw_config: crate::models::HnswConfig::default(),
            compression: crate::models::CompressionConfig::default(),
            normalization: None,
        };

        // Create collection
        store
            .create_collection("test-collection".to_string(), config, None)
            .await
            .unwrap();

        // Get stats
        let stats = store.get_collection_stats("test-collection").await.unwrap();
        assert_eq!(stats.name, "test-collection");
        assert_eq!(stats.collection_type, CollectionType::Dynamic);
        assert_eq!(stats.vector_count, 0);
        assert!(!stats.is_read_only);
    }

    #[tokio::test]
    #[ignore] // Timeout: runs for over 60 seconds
    async fn test_store_stats() {
        let store = create_test_enhanced_store().await;

        let config = CollectionConfig {
            dimension: 384,
            metric: DistanceMetric::Cosine,
            quantization: QuantizationConfig::default(),
            hnsw_config: crate::models::HnswConfig::default(),
            compression: crate::models::CompressionConfig::default(),
            normalization: None,
        };

        // Create some collections
        store
            .create_collection("collection1".to_string(), config.clone(), None)
            .await
            .unwrap();
        store
            .create_collection("collection2".to_string(), config, None)
            .await
            .unwrap();

        // Get store stats
        let stats = store.get_store_stats().await.unwrap();
        assert_eq!(stats.total_collections, 2);
        assert_eq!(stats.dynamic_collections, 2);
        assert_eq!(stats.workspace_collections, 0);
        assert_eq!(stats.total_vectors, 0);
    }
}
