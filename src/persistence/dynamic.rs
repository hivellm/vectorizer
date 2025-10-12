use crate::persistence::{
    types::{EnhancedCollectionMetadata, CollectionType, CollectionSource, WALEntry, Operation, Transaction, TransactionStatus},
    wal::{WriteAheadLog, WALConfig, WALError},
};
use crate::models::{CollectionConfig, DistanceMetric};
use crate::db::VectorStore;
use std::path::{Path, PathBuf};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tokio::fs;
use tokio::sync::Mutex as AsyncMutex;
use tracing::{info, warn, error, debug};
use thiserror::Error;
use serde_json;

/// Dynamic collection persistence manager
pub struct DynamicCollectionPersistence {
    /// Base directory for dynamic collections
    base_path: PathBuf,
    /// WAL instance
    wal: Arc<WriteAheadLog>,
    /// Active transactions
    active_transactions: Arc<Mutex<HashMap<u64, Transaction>>>,
    /// Checkpoint interval
    checkpoint_interval: std::time::Duration,
    /// Vector store reference
    vector_store: Arc<VectorStore>,
}

/// Persistence errors
#[derive(Debug, Error)]
pub enum PersistenceError {
    #[error("WAL error: {0}")]
    WALError(#[from] WALError),
    
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    
    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),
    
    #[error("Collection '{0}' not found")]
    CollectionNotFound(String),
    
    #[error("Collection '{0}' is read-only")]
    ReadOnlyCollection(String),
    
    #[error("Transaction {0} not found")]
    TransactionNotFound(u64),
    
    #[error("Transaction {0} is not in progress")]
    TransactionNotInProgress(u64),
    
    #[error("Cannot delete workspace collection '{0}'")]
    CannotDeleteWorkspace(String),
    
    #[error("Checkpoint failed: {0}")]
    CheckpointFailed(String),
    
    #[error("Recovery failed: {0}")]
    RecoveryFailed(String),
}

/// Persistence configuration
#[derive(Debug, Clone)]
pub struct PersistenceConfig {
    /// Base directory for dynamic collections
    pub data_dir: PathBuf,
    /// WAL configuration
    pub wal_config: WALConfig,
    /// Checkpoint interval
    pub checkpoint_interval: std::time::Duration,
    /// Auto-recovery enabled
    pub auto_recovery: bool,
    /// Verify integrity on startup
    pub verify_integrity: bool,
}

impl Default for PersistenceConfig {
    fn default() -> Self {
        Self {
            data_dir: PathBuf::from("./data/dynamic"),
            wal_config: WALConfig::default(),
            checkpoint_interval: std::time::Duration::from_secs(300), // 5 minutes
            auto_recovery: true,
            verify_integrity: true,
        }
    }
}

impl DynamicCollectionPersistence {
    /// Create new persistence manager
    pub async fn new(
        config: PersistenceConfig,
        vector_store: Arc<VectorStore>,
    ) -> Result<Self, PersistenceError> {
        // Ensure data directory exists
        fs::create_dir_all(&config.data_dir).await.map_err(PersistenceError::IoError)?;
        
        // Create WAL
        let wal_path = config.data_dir.join("wal.log");
        let wal = Arc::new(
            WriteAheadLog::new(wal_path, config.wal_config.clone()).await?
        );

        let persistence = Self {
            base_path: config.data_dir,
            wal,
            active_transactions: Arc::new(Mutex::new(HashMap::new())),
            checkpoint_interval: config.checkpoint_interval,
            vector_store,
        };

        // Auto-recovery if enabled
        if config.auto_recovery {
            persistence.auto_recover().await?;
        }

        // Verify integrity if enabled
        if config.verify_integrity {
            persistence.verify_all_integrity().await?;
        }

        info!("Dynamic collection persistence initialized at {}", persistence.base_path.display());
        Ok(persistence)
    }

    /// Get collection directory path
    fn collection_path(&self, collection_id: &str) -> PathBuf {
        self.base_path.join(collection_id)
    }

    /// Get metadata file path
    fn metadata_path(&self, collection_id: &str) -> PathBuf {
        self.collection_path(collection_id).join("metadata.json")
    }

    /// Get vectors file path
    fn vectors_path(&self, collection_id: &str) -> PathBuf {
        self.collection_path(collection_id).join("vectors.bin")
    }

    /// Get index file path
    fn index_path(&self, collection_id: &str) -> PathBuf {
        self.collection_path(collection_id).join("index.hnsw")
    }

    /// Create new dynamic collection
    pub async fn create_collection(
        &self,
        name: String,
        config: CollectionConfig,
        created_by: Option<String>,
    ) -> Result<EnhancedCollectionMetadata, PersistenceError> {
        // Check if collection already exists
        if self.collection_exists(&name).await {
            return Err(PersistenceError::CollectionNotFound(name));
        }

        let metadata = EnhancedCollectionMetadata::new_dynamic(
            name.clone(),
            created_by,
            "/api/v1/collections".to_string(),
            config,
        );

        // Create collection directory first
        let collection_dir = self.collection_path(&metadata.id);
        fs::create_dir_all(&collection_dir).await.map_err(PersistenceError::IoError)?;

        // Log creation to WAL
        let operation = Operation::CreateCollection {
            config: metadata.config.clone(),
        };
        self.wal.append(&metadata.id, operation).await?;

        // Save metadata
        self.save_metadata(&metadata).await?;

        info!("Dynamic collection '{}' created with ID '{}'", name, metadata.id);
        Ok(metadata)
    }

    /// Save collection metadata
    async fn save_metadata(&self, metadata: &EnhancedCollectionMetadata) -> Result<(), PersistenceError> {
        let metadata_path = self.metadata_path(&metadata.id);
        let json = serde_json::to_string_pretty(metadata).map_err(PersistenceError::SerializationError)?;
        
        fs::write(&metadata_path, json).await.map_err(PersistenceError::IoError)?;
        debug!("Metadata saved for collection '{}'", metadata.id);
        Ok(())
    }

    /// Load collection metadata
    async fn load_metadata(&self, collection_id: &str) -> Result<EnhancedCollectionMetadata, PersistenceError> {
        let metadata_path = self.metadata_path(collection_id);
        
        if !metadata_path.exists() {
            return Err(PersistenceError::CollectionNotFound(collection_id.to_string()));
        }

        let content = fs::read_to_string(&metadata_path).await.map_err(PersistenceError::IoError)?;
        let metadata: EnhancedCollectionMetadata = serde_json::from_str(&content)
            .map_err(PersistenceError::SerializationError)?;

        Ok(metadata)
    }

    /// Check if collection exists
    pub async fn collection_exists(&self, collection_name: &str) -> bool {
        // Try to find by name (check all dynamic collections)
        let entries = fs::read_dir(&self.base_path).await;
        if let Ok(mut entries) = entries {
            while let Ok(Some(entry)) = entries.next_entry().await {
                if let Ok(metadata) = self.load_metadata(&entry.file_name().to_string_lossy()).await {
                    if metadata.name == collection_name {
                        return true;
                    }
                }
            }
        }
        false
    }

    /// Get collection by name
    pub async fn get_collection_by_name(&self, name: &str) -> Result<EnhancedCollectionMetadata, PersistenceError> {
        let entries = fs::read_dir(&self.base_path).await.map_err(PersistenceError::IoError)?;
        
        let mut entries = entries;
        while let Ok(Some(entry)) = entries.next_entry().await {
            if let Ok(metadata) = self.load_metadata(&entry.file_name().to_string_lossy()).await {
                if metadata.name == name {
                    return Ok(metadata);
                }
            }
        }
        
        Err(PersistenceError::CollectionNotFound(name.to_string()))
    }

    /// List all dynamic collections
    pub async fn list_collections(&self) -> Result<Vec<EnhancedCollectionMetadata>, PersistenceError> {
        let mut collections = Vec::new();
        
        // Check if base_path exists
        if !self.base_path.exists() {
            return Ok(collections);
        }
        
        let mut entries = fs::read_dir(&self.base_path).await.map_err(PersistenceError::IoError)?;
        while let Ok(Some(entry)) = entries.next_entry().await {
            if entry.file_type().await.map_err(PersistenceError::IoError)?.is_dir() {
                let collection_id = entry.file_name().to_string_lossy().to_string();
                if let Ok(metadata) = self.load_metadata(&collection_id).await {
                    collections.push(metadata);
                }
            }
        }
        
        Ok(collections)
    }

    /// Begin transaction
    pub async fn begin_transaction(&self, collection_id: &str) -> Result<u64, PersistenceError> {
        let transaction_id = self.wal.current_sequence();
        let transaction = Transaction::new(transaction_id, collection_id.to_string());
        
        let mut active_transactions = self.active_transactions.lock().unwrap();
        active_transactions.insert(transaction_id, transaction);
        
        debug!("Transaction {} started for collection {}", transaction_id, collection_id);
        Ok(transaction_id)
    }

    /// Add operation to transaction
    pub async fn add_to_transaction(
        &self,
        transaction_id: u64,
        operation: Operation,
    ) -> Result<(), PersistenceError> {
        let mut active_transactions = self.active_transactions.lock().unwrap();
        
        if let Some(transaction) = active_transactions.get_mut(&transaction_id) {
            if transaction.status != TransactionStatus::InProgress {
                return Err(PersistenceError::TransactionNotInProgress(transaction_id));
            }
            
            transaction.add_operation(operation);
            debug!("Operation added to transaction {}", transaction_id);
            Ok(())
        } else {
            Err(PersistenceError::TransactionNotFound(transaction_id))
        }
    }

    /// Commit transaction
    pub async fn commit_transaction(&self, transaction_id: u64) -> Result<(), PersistenceError> {
        let mut active_transactions = self.active_transactions.lock().unwrap();
        
        if let Some(mut transaction) = active_transactions.remove(&transaction_id) {
            if transaction.status != TransactionStatus::InProgress {
                return Err(PersistenceError::TransactionNotInProgress(transaction_id));
            }

            // Append to WAL
            self.wal.append_transaction(&transaction).await?;
            
            // Apply operations to collection
            self.apply_transaction(&transaction).await?;
            
            transaction.commit();
            info!("Transaction {} committed with {} operations", 
                  transaction_id, transaction.operations.len());
            
            Ok(())
        } else {
            Err(PersistenceError::TransactionNotFound(transaction_id))
        }
    }

    /// Rollback transaction
    pub async fn rollback_transaction(&self, transaction_id: u64) -> Result<(), PersistenceError> {
        let mut active_transactions = self.active_transactions.lock().unwrap();
        
        if let Some(mut transaction) = active_transactions.remove(&transaction_id) {
            transaction.rollback();
            info!("Transaction {} rolled back", transaction_id);
            Ok(())
        } else {
            Err(PersistenceError::TransactionNotFound(transaction_id))
        }
    }

    /// Apply transaction operations to collection
    async fn apply_transaction(&self, transaction: &Transaction) -> Result<(), PersistenceError> {
        let mut metadata = self.load_metadata(&transaction.collection_id).await?;
        
        for operation in &transaction.operations {
            match operation {
                Operation::InsertVector { vector_id, data, metadata: meta } => {
                    // Insert vector (this would integrate with VectorStore)
                    // For now, just update counts
                    metadata.vector_count += 1;
                    metadata.document_count += 1;
                },
                Operation::UpdateVector { vector_id, .. } => {
                    // Update vector
                    // No count change for updates
                },
                Operation::DeleteVector { vector_id } => {
                    // Delete vector
                    metadata.vector_count = metadata.vector_count.saturating_sub(1);
                    metadata.document_count = metadata.document_count.saturating_sub(1);
                },
                Operation::CreateCollection { .. } => {
                    // Already handled in create_collection
                },
                Operation::DeleteCollection => {
                    // Delete collection
                    self.delete_collection_files(&transaction.collection_id).await?;
                    return Ok(());
                },
                Operation::Checkpoint { vector_count, document_count, checksum } => {
                    // Update metadata with checkpoint info
                    metadata.vector_count = *vector_count;
                    metadata.document_count = *document_count;
                    metadata.data_checksum = Some(checksum.clone());
                },
            }
        }
        
        metadata.update_checksums();
        metadata.last_transaction_id = Some(transaction.id);
        self.save_metadata(&metadata).await?;
        
        Ok(())
    }

    /// Delete collection files
    async fn delete_collection_files(&self, collection_id: &str) -> Result<(), PersistenceError> {
        let collection_path = self.collection_path(collection_id);
        
        if collection_path.exists() {
            fs::remove_dir_all(&collection_path).await.map_err(PersistenceError::IoError)?;
            info!("Collection files deleted for '{}'", collection_id);
        }
        
        Ok(())
    }

    /// Delete collection
    pub async fn delete_collection(&self, collection_name: &str) -> Result<(), PersistenceError> {
        let metadata = self.get_collection_by_name(collection_name).await?;
        
        if metadata.is_workspace() {
            return Err(PersistenceError::CannotDeleteWorkspace(collection_name.to_string()));
        }

        // Log deletion to WAL
        let operation = Operation::DeleteCollection;
        self.wal.append(&metadata.id, operation).await?;

        // Delete files
        self.delete_collection_files(&metadata.id).await?;
        
        info!("Dynamic collection '{}' deleted", collection_name);
        Ok(())
    }

    /// Create checkpoint for collection
    pub async fn checkpoint_collection(&self, collection_id: &str) -> Result<(), PersistenceError> {
        let metadata = self.load_metadata(collection_id).await?;
        
        let operation = Operation::Checkpoint {
            vector_count: metadata.vector_count,
            document_count: metadata.document_count,
            checksum: metadata.calculate_data_checksum(),
        };
        
        self.wal.append(collection_id, operation).await?;
        
        // Update metadata
        let mut updated_metadata = metadata;
        updated_metadata.update_checksums();
        self.save_metadata(&updated_metadata).await?;
        
        debug!("Checkpoint created for collection '{}'", collection_id);
        Ok(())
    }

    /// Auto-recovery from WAL
    pub async fn auto_recover(&self) -> Result<(), PersistenceError> {
        info!("Starting auto-recovery from WAL");
        
        // Get all collection directories
        let mut entries = fs::read_dir(&self.base_path).await.map_err(PersistenceError::IoError)?;
        while let Ok(Some(entry)) = entries.next_entry().await {
            if entry.file_type().await.map_err(PersistenceError::IoError)?.is_dir() {
                let collection_id = entry.file_name().to_string_lossy().to_string();
                
                // Skip WAL file
                if collection_id == "wal.log" {
                    continue;
                }
                
                if let Err(e) = self.recover_collection(&collection_id).await {
                    warn!("Failed to recover collection '{}': {}", collection_id, e);
                }
            }
        }
        
        info!("Auto-recovery completed");
        Ok(())
    }

    /// Recover specific collection
    async fn recover_collection(&self, collection_id: &str) -> Result<(), PersistenceError> {
        debug!("Recovering collection '{}'", collection_id);
        
        // Load current metadata
        let mut metadata = match self.load_metadata(collection_id).await {
            Ok(meta) => meta,
            Err(PersistenceError::CollectionNotFound(_)) => {
                // Collection doesn't exist, skip recovery
                return Ok(());
            },
            Err(e) => return Err(e),
        };
        
        // Get last transaction ID from metadata
        let last_transaction_id = metadata.last_transaction_id.unwrap_or(0);
        
        // Recover from WAL
        let wal_entries = self.wal.recover(collection_id).await?;
        
        // Apply only new entries
        let new_entries: Vec<_> = wal_entries
            .into_iter()
            .filter(|entry| entry.transaction_id.map_or(false, |id| id > last_transaction_id))
            .collect();
        
        if !new_entries.is_empty() {
            debug!("Recovering {} WAL entries for collection '{}'", 
                   new_entries.len(), collection_id);
            
            // Apply recovery operations
            for entry in new_entries {
                self.apply_operation(&mut metadata, &entry.operation).await?;
            }
            
            metadata.update_checksums();
            self.save_metadata(&metadata).await?;
        }
        
        Ok(())
    }

    /// Apply single operation to metadata
    async fn apply_operation(
        &self,
        metadata: &mut EnhancedCollectionMetadata,
        operation: &Operation,
    ) -> Result<(), PersistenceError> {
        match operation {
            Operation::InsertVector { .. } => {
                metadata.vector_count += 1;
                metadata.document_count += 1;
            },
            Operation::UpdateVector { .. } => {
                // No count change for updates
            },
            Operation::DeleteVector { .. } => {
                metadata.vector_count = metadata.vector_count.saturating_sub(1);
                metadata.document_count = metadata.document_count.saturating_sub(1);
            },
            Operation::Checkpoint { vector_count, document_count, checksum } => {
                metadata.vector_count = *vector_count;
                metadata.document_count = *document_count;
                metadata.data_checksum = Some(checksum.clone());
            },
            _ => {
                // Other operations don't affect metadata counts
            },
        }
        
        metadata.updated_at = chrono::Utc::now();
        Ok(())
    }

    /// Verify integrity of all collections
    pub async fn verify_all_integrity(&self) -> Result<(), PersistenceError> {
        info!("Verifying integrity of all dynamic collections");
        
        let collections = self.list_collections().await?;
        for metadata in collections {
            if let Err(e) = self.verify_collection_integrity(&metadata).await {
                warn!("Integrity check failed for collection '{}': {}", metadata.name, e);
            }
        }
        
        info!("Integrity verification completed");
        Ok(())
    }

    /// Verify integrity of specific collection
    async fn verify_collection_integrity(&self, metadata: &EnhancedCollectionMetadata) -> Result<(), PersistenceError> {
        let calculated_checksum = metadata.calculate_data_checksum();
        
        if let Some(stored_checksum) = &metadata.data_checksum {
            if calculated_checksum != *stored_checksum {
                warn!("Checksum mismatch for collection '{}': calculated={}, stored={}", 
                      metadata.name, calculated_checksum, stored_checksum);
            }
        }
        
        Ok(())
    }

    /// Get persistence statistics
    pub async fn get_stats(&self) -> Result<PersistenceStats, PersistenceError> {
        let collections = self.list_collections().await?;
        let wal_stats = self.wal.get_stats().await?;
        
        let total_collections = collections.len();
        let total_vectors: usize = collections.iter().map(|c| c.vector_count).sum();
        let total_documents: usize = collections.iter().map(|c| c.document_count).sum();
        
        Ok(PersistenceStats {
            total_collections,
            total_vectors,
            total_documents,
            wal_entries: wal_stats.entry_count,
            wal_size_bytes: wal_stats.file_size_bytes,
            active_transactions: self.active_transactions.lock().unwrap().len(),
        })
    }
}

/// Persistence statistics
#[derive(Debug, Clone)]
pub struct PersistenceStats {
    pub total_collections: usize,
    pub total_vectors: usize,
    pub total_documents: usize,
    pub wal_entries: usize,
    pub wal_size_bytes: u64,
    pub active_transactions: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    use crate::models::QuantizationConfig;

    async fn create_test_persistence() -> DynamicCollectionPersistence {
        let temp_dir = tempdir().unwrap();
        let data_dir = temp_dir.path().join("data");
        std::fs::create_dir_all(&data_dir).unwrap(); // Create directory
        
        let config = PersistenceConfig {
            data_dir,
            ..Default::default()
        };
        
        // Create mock vector store
        let vector_store = Arc::new(VectorStore::new());
        
        DynamicCollectionPersistence::new(config, vector_store).await.unwrap()
    }

    #[tokio::test]
    async fn test_create_dynamic_collection() {
        let persistence = create_test_persistence().await;
        
        let config = CollectionConfig {
            dimension: 384,
            metric: DistanceMetric::Cosine,
            quantization: QuantizationConfig::default(),
            hnsw_config: crate::models::HnswConfig::default(),
            compression: crate::models::CompressionConfig::default(),
            normalization: None,
        };
        
        let metadata = persistence.create_collection(
            "test-collection".to_string(),
            config,
            Some("user123".to_string()),
        ).await.unwrap();
        
        assert_eq!(metadata.name, "test-collection");
        assert_eq!(metadata.collection_type, CollectionType::Dynamic);
        assert!(!metadata.is_read_only);
        assert!(metadata.is_dynamic());
    }

    #[tokio::test]
    async fn test_collection_exists() {
        let persistence = create_test_persistence().await;
        
        let config = CollectionConfig {
            dimension: 384,
            metric: DistanceMetric::Cosine,
            quantization: QuantizationConfig::default(),
            hnsw_config: crate::models::HnswConfig::default(),
            compression: crate::models::CompressionConfig::default(),
            normalization: None,
        };
        
        // Collection doesn't exist yet
        assert!(!persistence.collection_exists("test-collection").await);
        
        // Create collection
        persistence.create_collection(
            "test-collection".to_string(),
            config,
            None,
        ).await.unwrap();
        
        // Collection should exist now
        assert!(persistence.collection_exists("test-collection").await);
    }

    #[tokio::test]
    async fn test_list_collections() {
        let persistence = create_test_persistence().await;
        
        let config = CollectionConfig {
            dimension: 384,
            metric: DistanceMetric::Cosine,
            quantization: QuantizationConfig::default(),
            hnsw_config: crate::models::HnswConfig::default(),
            compression: crate::models::CompressionConfig::default(),
            normalization: None,
        };
        
        // Initially empty
        let collections = persistence.list_collections().await.unwrap();
        assert_eq!(collections.len(), 0);
        
        // Create collections
        persistence.create_collection("collection1".to_string(), config.clone(), None).await.unwrap();
        persistence.create_collection("collection2".to_string(), config, None).await.unwrap();
        
        // Should have 2 collections
        let collections = persistence.list_collections().await.unwrap();
        assert_eq!(collections.len(), 2);
    }

    #[tokio::test]
    async fn test_transaction_lifecycle() {
        let persistence = create_test_persistence().await;
        
        let config = CollectionConfig {
            dimension: 384,
            metric: DistanceMetric::Cosine,
            quantization: QuantizationConfig::default(),
            hnsw_config: crate::models::HnswConfig::default(),
            compression: crate::models::CompressionConfig::default(),
            normalization: None,
        };
        
        let metadata = persistence.create_collection(
            "test-collection".to_string(),
            config,
            None,
        ).await.unwrap();
        
        // Begin transaction
        let transaction_id = persistence.begin_transaction(&metadata.id).await.unwrap();
        
        // Add operation
        let operation = Operation::InsertVector {
            vector_id: "vec1".to_string(),
            data: vec![1.0, 2.0, 3.0],
            metadata: std::collections::HashMap::new(),
        };
        persistence.add_to_transaction(transaction_id, operation).await.unwrap();
        
        // Commit transaction
        persistence.commit_transaction(transaction_id).await.unwrap();
    }

    #[tokio::test]
    async fn test_delete_collection() {
        let persistence = create_test_persistence().await;
        
        let config = CollectionConfig {
            dimension: 384,
            metric: DistanceMetric::Cosine,
            quantization: QuantizationConfig::default(),
            hnsw_config: crate::models::HnswConfig::default(),
            compression: crate::models::CompressionConfig::default(),
            normalization: None,
        };
        
        // Create collection
        let metadata = persistence.create_collection(
            "test-collection".to_string(),
            config,
            None,
        ).await.unwrap();
        
        // Verify it exists
        assert!(persistence.collection_exists("test-collection").await);
        
        // Delete collection
        persistence.delete_collection("test-collection").await.unwrap();
        
        // Verify it's gone
        assert!(!persistence.collection_exists("test-collection").await);
    }

    #[tokio::test]
    async fn test_get_stats() {
        let persistence = create_test_persistence().await;
        
        let config = CollectionConfig {
            dimension: 384,
            metric: DistanceMetric::Cosine,
            quantization: QuantizationConfig::default(),
            hnsw_config: crate::models::HnswConfig::default(),
            compression: crate::models::CompressionConfig::default(),
            normalization: None,
        };
        
        // Create some collections
        persistence.create_collection("collection1".to_string(), config.clone(), None).await.unwrap();
        persistence.create_collection("collection2".to_string(), config, None).await.unwrap();
        
        let stats = persistence.get_stats().await.unwrap();
        assert_eq!(stats.total_collections, 2);
        assert_eq!(stats.total_vectors, 0); // No vectors inserted yet
        assert_eq!(stats.total_documents, 0);
    }
}
