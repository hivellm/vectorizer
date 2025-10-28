use std::collections::HashMap;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::models::{CollectionConfig, DistanceMetric};

/// Collection types for persistence system
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum CollectionType {
    /// From workspace configuration (read-only)
    Workspace,
    /// Created at runtime via API/MCP (read-write)
    Dynamic,
}

/// Source information for collections
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CollectionSource {
    Workspace {
        project_name: String,
        config_path: String,
    },
    Dynamic {
        created_by: Option<String>,
        api_endpoint: String,
    },
}

/// Enhanced collection metadata with persistence information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnhancedCollectionMetadata {
    /// Collection identifier
    pub id: String,
    /// Collection name
    pub name: String,
    /// Collection type (workspace or dynamic)
    pub collection_type: CollectionType,
    /// Vector dimension
    pub dimension: usize,
    /// Distance metric
    pub metric: DistanceMetric,
    /// Creation timestamp
    pub created_at: DateTime<Utc>,
    /// Last update timestamp
    pub updated_at: DateTime<Utc>,
    /// Number of vectors in collection
    pub vector_count: usize,
    /// Number of documents in collection
    pub document_count: usize,
    /// Whether collection is read-only
    pub is_read_only: bool,
    /// Source information
    pub source: CollectionSource,
    /// Collection configuration
    pub config: CollectionConfig,
    /// Data integrity checksum
    pub data_checksum: Option<String>,
    /// Index integrity checksum
    pub index_checksum: Option<String>,
    /// Last integrity validation timestamp
    pub last_validation: Option<DateTime<Utc>>,
    /// Index version for compatibility
    pub index_version: u32,
    /// Compression ratio achieved
    pub compression_ratio: Option<f32>,
    /// Memory usage in MB
    pub memory_usage_mb: Option<f32>,
    /// Last transaction ID processed
    pub last_transaction_id: Option<u64>,
    /// Number of pending operations
    pub pending_operations: usize,
}

impl EnhancedCollectionMetadata {
    /// Create new workspace collection metadata
    pub fn new_workspace(
        name: String,
        project_name: String,
        config_path: String,
        config: CollectionConfig,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: format!("workspace-{}", name),
            name: name.clone(),
            collection_type: CollectionType::Workspace,
            dimension: config.dimension,
            metric: config.metric.clone(),
            created_at: now,
            updated_at: now,
            vector_count: 0,
            document_count: 0,
            is_read_only: true,
            source: CollectionSource::Workspace {
                project_name,
                config_path,
            },
            config,
            data_checksum: None,
            index_checksum: None,
            last_validation: None,
            index_version: 1,
            compression_ratio: None,
            memory_usage_mb: None,
            last_transaction_id: None,
            pending_operations: 0,
        }
    }

    /// Create new dynamic collection metadata
    pub fn new_dynamic(
        name: String,
        created_by: Option<String>,
        api_endpoint: String,
        config: CollectionConfig,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: format!("dynamic-{}", name),
            name: name.clone(),
            collection_type: CollectionType::Dynamic,
            dimension: config.dimension,
            metric: config.metric.clone(),
            created_at: now,
            updated_at: now,
            vector_count: 0,
            document_count: 0,
            is_read_only: false,
            source: CollectionSource::Dynamic {
                created_by,
                api_endpoint,
            },
            config,
            data_checksum: None,
            index_checksum: None,
            last_validation: None,
            index_version: 1,
            compression_ratio: None,
            memory_usage_mb: None,
            last_transaction_id: None,
            pending_operations: 0,
        }
    }

    /// Update metadata after operations
    pub fn update_after_operation(&mut self, vector_count: usize, document_count: usize) {
        self.vector_count = vector_count;
        self.document_count = document_count;
        self.updated_at = Utc::now();
        self.pending_operations = self.pending_operations.saturating_sub(1);
    }

    /// Check if collection is workspace collection
    pub fn is_workspace(&self) -> bool {
        matches!(self.collection_type, CollectionType::Workspace)
    }

    /// Check if collection is dynamic collection
    pub fn is_dynamic(&self) -> bool {
        matches!(self.collection_type, CollectionType::Dynamic)
    }

    /// Generate data checksum
    pub fn calculate_data_checksum(&self) -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        self.id.hash(&mut hasher);
        self.name.hash(&mut hasher);
        self.vector_count.hash(&mut hasher);
        self.document_count.hash(&mut hasher);
        self.dimension.hash(&mut hasher);
        self.updated_at.timestamp().hash(&mut hasher);

        format!("{:x}", hasher.finish())
    }

    /// Generate index checksum
    pub fn calculate_index_checksum(&self) -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        self.id.hash(&mut hasher);
        self.index_version.hash(&mut hasher);
        self.vector_count.hash(&mut hasher);
        self.compression_ratio
            .unwrap_or(1.0)
            .to_bits()
            .hash(&mut hasher);

        format!("{:x}", hasher.finish())
    }

    /// Update checksums
    pub fn update_checksums(&mut self) {
        self.data_checksum = Some(self.calculate_data_checksum());
        self.index_checksum = Some(self.calculate_index_checksum());
        self.last_validation = Some(Utc::now());
    }
}

/// WAL (Write-Ahead Log) entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WALEntry {
    /// Sequence number for ordering
    pub sequence: u64,
    /// Timestamp of operation
    pub timestamp: DateTime<Utc>,
    /// Operation type
    pub operation: Operation,
    /// Collection ID
    pub collection_id: String,
    /// Transaction ID (if part of transaction)
    pub transaction_id: Option<u64>,
}

/// Operations that can be logged in WAL
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Operation {
    /// Insert vector(s) into collection
    InsertVector {
        vector_id: String,
        data: Vec<f32>,
        metadata: HashMap<String, String>,
    },
    /// Update existing vector
    UpdateVector {
        vector_id: String,
        data: Option<Vec<f32>>,
        metadata: Option<HashMap<String, String>>,
    },
    /// Delete vector(s) from collection
    DeleteVector { vector_id: String },
    /// Create new collection
    CreateCollection { config: CollectionConfig },
    /// Delete collection
    DeleteCollection,
    /// Checkpoint marker
    Checkpoint {
        vector_count: usize,
        document_count: usize,
        checksum: String,
    },
}

impl Operation {
    /// Get operation type name for logging
    pub fn operation_type(&self) -> &'static str {
        match self {
            Operation::InsertVector { .. } => "insert_vector",
            Operation::UpdateVector { .. } => "update_vector",
            Operation::DeleteVector { .. } => "delete_vector",
            Operation::CreateCollection { .. } => "create_collection",
            Operation::DeleteCollection => "delete_collection",
            Operation::Checkpoint { .. } => "checkpoint",
        }
    }

    /// Check if operation modifies data
    pub fn is_data_modifying(&self) -> bool {
        matches!(
            self,
            Operation::InsertVector { .. }
                | Operation::UpdateVector { .. }
                | Operation::DeleteVector { .. }
                | Operation::CreateCollection { .. }
                | Operation::DeleteCollection
        )
    }
}

/// Transaction information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transaction {
    /// Unique transaction ID
    pub id: u64,
    /// Collection ID
    pub collection_id: String,
    /// List of operations in transaction
    pub operations: Vec<Operation>,
    /// Transaction status
    pub status: TransactionStatus,
    /// Transaction start time
    pub started_at: DateTime<Utc>,
    /// Transaction end time (if completed)
    pub completed_at: Option<DateTime<Utc>>,
    /// Data checksum for validation
    pub checksum: Option<String>,
}

/// Transaction status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TransactionStatus {
    /// Transaction is in progress
    InProgress,
    /// Transaction completed successfully
    Committed,
    /// Transaction was rolled back
    RolledBack,
    /// Transaction failed
    Failed,
}

impl Transaction {
    /// Create new transaction
    pub fn new(id: u64, collection_id: String) -> Self {
        Self {
            id,
            collection_id,
            operations: Vec::new(),
            status: TransactionStatus::InProgress,
            started_at: Utc::now(),
            completed_at: None,
            checksum: None,
        }
    }

    /// Add operation to transaction
    pub fn add_operation(&mut self, operation: Operation) {
        self.operations.push(operation);
    }

    /// Commit transaction
    pub fn commit(&mut self) {
        self.status = TransactionStatus::Committed;
        self.completed_at = Some(Utc::now());
    }

    /// Rollback transaction
    pub fn rollback(&mut self) {
        self.status = TransactionStatus::RolledBack;
        self.completed_at = Some(Utc::now());
    }

    /// Mark transaction as failed
    pub fn fail(&mut self) {
        self.status = TransactionStatus::Failed;
        self.completed_at = Some(Utc::now());
    }

    /// Check if transaction is completed
    pub fn is_completed(&self) -> bool {
        matches!(
            self.status,
            TransactionStatus::Committed
                | TransactionStatus::RolledBack
                | TransactionStatus::Failed
        )
    }

    /// Calculate transaction checksum
    pub fn calculate_checksum(&self) -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        self.id.hash(&mut hasher);
        self.collection_id.hash(&mut hasher);
        self.operations.len().hash(&mut hasher);
        self.started_at.timestamp().hash(&mut hasher);

        format!("{:x}", hasher.finish())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::CollectionConfig;

    #[test]
    fn test_workspace_metadata_creation() {
        let config = CollectionConfig {
            dimension: 384,
            metric: DistanceMetric::Cosine,
            quantization: crate::models::QuantizationConfig::default(),
            hnsw_config: crate::models::HnswConfig::default(),
            compression: crate::models::CompressionConfig::default(),
            normalization: None,
        };

        let metadata = EnhancedCollectionMetadata::new_workspace(
            "test-collection".to_string(),
            "test-project".to_string(),
            "/path/to/config.yml".to_string(),
            config.clone(),
        );

        assert_eq!(metadata.name, "test-collection");
        assert_eq!(metadata.collection_type, CollectionType::Workspace);
        assert!(metadata.is_read_only);
        assert!(metadata.is_workspace());
        assert!(!metadata.is_dynamic());
    }

    #[test]
    fn test_dynamic_metadata_creation() {
        let config = CollectionConfig {
            dimension: 384,
            metric: DistanceMetric::Cosine,
            quantization: crate::models::QuantizationConfig::default(),
            hnsw_config: crate::models::HnswConfig::default(),
            compression: crate::models::CompressionConfig::default(),
            normalization: None,
        };

        let metadata = EnhancedCollectionMetadata::new_dynamic(
            "dynamic-collection".to_string(),
            Some("user123".to_string()),
            "/api/v1/collections".to_string(),
            config.clone(),
        );

        assert_eq!(metadata.name, "dynamic-collection");
        assert_eq!(metadata.collection_type, CollectionType::Dynamic);
        assert!(!metadata.is_read_only);
        assert!(!metadata.is_workspace());
        assert!(metadata.is_dynamic());
    }

    #[test]
    fn test_metadata_update_after_operation() {
        let config = CollectionConfig {
            dimension: 384,
            metric: DistanceMetric::Cosine,
            quantization: crate::models::QuantizationConfig::default(),
            hnsw_config: crate::models::HnswConfig::default(),
            compression: crate::models::CompressionConfig::default(),
            normalization: None,
        };

        let mut metadata = EnhancedCollectionMetadata::new_dynamic(
            "test".to_string(),
            None,
            "/api".to_string(),
            config,
        );

        metadata.pending_operations = 5;
        metadata.update_after_operation(100, 50);

        assert_eq!(metadata.vector_count, 100);
        assert_eq!(metadata.document_count, 50);
        assert_eq!(metadata.pending_operations, 4);
    }

    #[test]
    fn test_transaction_lifecycle() {
        let mut transaction = Transaction::new(1, "collection1".to_string());

        assert_eq!(transaction.status, TransactionStatus::InProgress);
        assert!(!transaction.is_completed());

        transaction.add_operation(Operation::InsertVector {
            vector_id: "vec1".to_string(),
            data: vec![1.0, 2.0, 3.0],
            metadata: HashMap::new(),
        });

        assert_eq!(transaction.operations.len(), 1);

        transaction.commit();
        assert_eq!(transaction.status, TransactionStatus::Committed);
        assert!(transaction.is_completed());
        assert!(transaction.completed_at.is_some());
    }

    #[test]
    fn test_operation_types() {
        let insert_op = Operation::InsertVector {
            vector_id: "vec1".to_string(),
            data: vec![1.0, 2.0],
            metadata: HashMap::new(),
        };

        assert_eq!(insert_op.operation_type(), "insert_vector");
        assert!(insert_op.is_data_modifying());

        let checkpoint_op = Operation::Checkpoint {
            vector_count: 100,
            document_count: 50,
            checksum: "abc123".to_string(),
        };

        assert_eq!(checkpoint_op.operation_type(), "checkpoint");
        assert!(!checkpoint_op.is_data_modifying());
    }
}
