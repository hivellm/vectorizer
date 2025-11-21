//! WAL (Write-Ahead Log) integration for Collection operations
//!
//! This module provides integration between the WAL system and Collection operations,
//! ensuring all mutations are logged before being applied to ensure crash recovery.

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;

use tokio::sync::Mutex as AsyncMutex;
use tracing::{debug, error, warn};

use crate::error::Result;
use crate::models::Vector;
use crate::persistence::types::Operation;
use crate::persistence::wal::{WALConfig, WALError, WriteAheadLog};

/// WAL integration manager for VectorStore
#[derive(Clone)]
pub struct WalIntegration {
    /// WAL instance (None if WAL is disabled)
    wal: Option<Arc<WriteAheadLog>>,
    /// Data directory for WAL files
    data_dir: PathBuf,
}

impl WalIntegration {
    /// Create new WAL integration (disabled by default)
    pub fn new_disabled() -> Self {
        Self {
            wal: None,
            data_dir: PathBuf::from("data"),
        }
    }

    /// Create new WAL integration with WAL enabled
    pub async fn new(
        data_dir: PathBuf,
        config: Option<WALConfig>,
    ) -> std::result::Result<Self, WALError> {
        let config = config.unwrap_or_else(|| {
            WALConfig {
                checkpoint_threshold: 1000,
                max_wal_size_mb: 100,
                checkpoint_interval: std::time::Duration::from_secs(300), // 5 minutes
                compression: false,
            }
        });

        let wal_path = data_dir.join("vectorizer.wal");
        let wal = WriteAheadLog::new(&wal_path, config).await?;

        info!("WAL integration enabled at {}", wal_path.display());

        Ok(Self {
            wal: Some(Arc::new(wal)),
            data_dir,
        })
    }

    /// Check if WAL is enabled
    pub fn is_enabled(&self) -> bool {
        self.wal.is_some()
    }

    /// Log insert operation to WAL
    pub async fn log_insert(
        &self,
        collection_name: &str,
        vector: &Vector,
    ) -> std::result::Result<(), WALError> {
        if let Some(wal) = &self.wal {
            // Convert payload to metadata HashMap
            let metadata = if let Some(payload) = &vector.payload {
                if let Some(obj) = payload.data.as_object() {
                    obj.iter()
                        .filter_map(|(k, v)| v.as_str().map(|s| (k.clone(), s.to_string())))
                        .collect()
                } else {
                    HashMap::new()
                }
            } else {
                HashMap::new()
            };

            let operation = Operation::InsertVector {
                vector_id: vector.id.clone(),
                data: vector.data.clone(),
                metadata,
            };

            wal.append(collection_name, operation).await?;
            debug!("Logged insert operation for vector '{}'", vector.id);
        }
        Ok(())
    }

    /// Log update operation to WAL
    pub async fn log_update(
        &self,
        collection_name: &str,
        vector: &Vector,
    ) -> std::result::Result<(), WALError> {
        if let Some(wal) = &self.wal {
            // Convert payload to metadata HashMap
            let metadata = if let Some(payload) = &vector.payload {
                if let Some(obj) = payload.data.as_object() {
                    Some(
                        obj.iter()
                            .filter_map(|(k, v)| v.as_str().map(|s| (k.clone(), s.to_string())))
                            .collect(),
                    )
                } else {
                    None
                }
            } else {
                None
            };

            let operation = Operation::UpdateVector {
                vector_id: vector.id.clone(),
                data: Some(vector.data.clone()),
                metadata,
            };

            wal.append(collection_name, operation).await?;
            debug!("Logged update operation for vector '{}'", vector.id);
        }
        Ok(())
    }

    /// Log delete operation to WAL
    pub async fn log_delete(
        &self,
        collection_name: &str,
        vector_id: &str,
    ) -> std::result::Result<(), WALError> {
        if let Some(wal) = &self.wal {
            let operation = Operation::DeleteVector {
                vector_id: vector_id.to_string(),
            };

            wal.append(collection_name, operation).await?;
            debug!("Logged delete operation for vector '{}'", vector_id);
        }
        Ok(())
    }

    /// Recover operations from WAL for a collection
    pub async fn recover_collection(
        &self,
        collection_name: &str,
    ) -> std::result::Result<Vec<crate::persistence::types::WALEntry>, WALError> {
        if let Some(wal) = &self.wal {
            wal.recover(collection_name).await
        } else {
            Ok(Vec::new())
        }
    }

    /// Get WAL instance (for advanced operations)
    pub fn wal(&self) -> Option<&Arc<WriteAheadLog>> {
        self.wal.as_ref()
    }

    /// Create checkpoint
    pub async fn checkpoint(&self) -> std::result::Result<u64, WALError> {
        if let Some(wal) = &self.wal {
            wal.checkpoint().await
        } else {
            Ok(0)
        }
    }

    /// Check if checkpoint is needed
    pub async fn should_checkpoint(&self) -> std::result::Result<bool, WALError> {
        if let Some(wal) = &self.wal {
            wal.should_checkpoint().await
        } else {
            Ok(false)
        }
    }
}

use tracing::info;

#[cfg(test)]
mod tests {
    use serde_json::json;
    use tempfile::tempdir;

    use super::*;
    use crate::models::Payload;

    #[tokio::test]
    async fn test_wal_integration_disabled() {
        let integration = WalIntegration::new_disabled();
        assert!(!integration.is_enabled());

        let vector = Vector {
            id: "test".to_string(),
            data: vec![1.0, 2.0, 3.0],
            payload: None,
            sparse: None,
        };

        // Should not error even when disabled
        assert!(
            integration
                .log_insert("test_collection", &vector)
                .await
                .is_ok()
        );
    }

    #[tokio::test]
    async fn test_wal_integration_enabled() {
        let temp_dir = tempdir().unwrap();
        let integration = WalIntegration::new(temp_dir.path().to_path_buf(), None)
            .await
            .unwrap();

        assert!(integration.is_enabled());

        let vector = Vector {
            id: "test".to_string(),
            data: vec![1.0, 2.0, 3.0],
            payload: None,
            sparse: None,
        };

        // Log insert
        assert!(
            integration
                .log_insert("test_collection", &vector)
                .await
                .is_ok()
        );

        // Log update
        assert!(
            integration
                .log_update("test_collection", &vector)
                .await
                .is_ok()
        );

        // Log delete
        assert!(
            integration
                .log_delete("test_collection", "test")
                .await
                .is_ok()
        );
    }

    #[tokio::test]
    async fn test_wal_integration_with_payload() {
        let temp_dir = tempdir().unwrap();
        let integration = WalIntegration::new(temp_dir.path().to_path_buf(), None)
            .await
            .unwrap();

        let payload = Payload {
            data: json!({
                "file_path": "/path/to/file.txt",
                "title": "Test Document"
            }),
        };

        let vector = Vector {
            id: "test".to_string(),
            data: vec![1.0, 2.0, 3.0],
            payload: Some(payload),
            sparse: None,
        };

        assert!(
            integration
                .log_insert("test_collection", &vector)
                .await
                .is_ok()
        );
    }
}
