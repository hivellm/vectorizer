//! Write-ahead log integration.
//!
//! - [`log_wal_insert`] / [`log_wal_update`] / [`log_wal_delete`] are
//!   the synchronous wrappers called from the CRUD path in
//!   [`super::vectors`]. They dispatch into the async WAL write either
//!   via the current tokio runtime (fire-and-forget) or a
//!   newly-constructed one (for rare non-async callers). WAL failures
//!   are logged and swallowed — the user-facing operation still
//!   succeeds — because WAL is best-effort.
//! - [`enable_wal`] wires a real `WalIntegration` into the store;
//!   before this runs the store holds a `new_disabled()` instance so
//!   the `is_enabled()` branches naturally short-circuit.
//! - [`recover_from_wal`], [`recover_and_replay_wal`],
//!   [`recover_all_from_wal`] implement crash recovery at startup.
//! - [`log_wal_insert`]: [`VectorStore::log_wal_insert`]
//! - [`log_wal_update`]: [`VectorStore::log_wal_update`]
//! - [`log_wal_delete`]: [`VectorStore::log_wal_delete`]
//! - [`enable_wal`]: [`VectorStore::enable_wal`]
//! - [`recover_from_wal`]: [`VectorStore::recover_from_wal`]
//! - [`recover_and_replay_wal`]: [`VectorStore::recover_and_replay_wal`]
//! - [`recover_all_from_wal`]: [`VectorStore::recover_all_from_wal`]

use std::path::PathBuf;

use tracing::{debug, error, info, warn};

use super::VectorStore;
use crate::db::wal_integration::WalIntegration;
use crate::error::{Result, VectorizerError};
use crate::models::Vector;

impl VectorStore {
    /// Log insert operation to WAL (synchronous wrapper)
    /// Note: This is fire-and-forget to avoid blocking. WAL errors are logged but don't fail the operation.
    pub(super) fn log_wal_insert(&self, collection_name: &str, vectors: &[Vector]) -> Result<()> {
        let wal_guard = self.wal.lock();
        if let Some(wal) = wal_guard.as_ref() {
            if wal.is_enabled() {
                // Try to get current runtime handle
                if let Ok(_handle) = tokio::runtime::Handle::try_current() {
                    // We're in an async context, spawn task for logging (fire-and-forget)
                    // Note: In production, this is acceptable as WAL is best-effort
                    // For tests, we'll add a small delay to allow writes to complete
                    let wal_clone = wal.clone();
                    let collection_name = collection_name.to_string();
                    let vectors_clone: Vec<Vector> = vectors.iter().cloned().collect();

                    tokio::spawn(async move {
                        for vector in vectors_clone {
                            if let Err(e) = wal_clone.log_insert(&collection_name, &vector).await {
                                error!("Failed to log insert to WAL: {}", e);
                            }
                        }
                    });
                } else {
                    // No runtime exists, try to create a temporary one
                    // WAL logging is best-effort and shouldn't block operations
                    match tokio::runtime::Runtime::new() {
                        Ok(rt) => {
                            // Log each vector to WAL
                            for vector in vectors {
                                if let Err(e) = rt.block_on(async {
                                    wal.log_insert(collection_name, vector).await
                                }) {
                                    error!("Failed to log insert to WAL: {}", e);
                                    // Don't fail the operation, just log the error
                                }
                            }
                        }
                        Err(e) => {
                            debug!(
                                "Could not create tokio runtime for WAL insert (non-async context): {}. WAL logging skipped.",
                                e
                            );
                            // Don't fail the operation if WAL logging fails
                        }
                    }
                }
            }
        }
        Ok(())
    }

    /// Log update operation to WAL (synchronous wrapper)
    /// Note: This is fire-and-forget to avoid blocking. WAL errors are logged but don't fail the operation.
    pub(super) fn log_wal_update(&self, collection_name: &str, vector: &Vector) -> Result<()> {
        let wal_guard = self.wal.lock();
        if let Some(wal) = wal_guard.as_ref() {
            if wal.is_enabled() {
                if let Ok(_handle) = tokio::runtime::Handle::try_current() {
                    let wal_clone = wal.clone();
                    let collection_name = collection_name.to_string();
                    let vector_clone = vector.clone();

                    tokio::spawn(async move {
                        if let Err(e) = wal_clone.log_update(&collection_name, &vector_clone).await
                        {
                            error!("Failed to log update to WAL: {}", e);
                        }
                    });
                } else {
                    // In non-async contexts, try to create a runtime, but don't fail if it doesn't work
                    // WAL logging is best-effort and shouldn't block operations
                    match tokio::runtime::Runtime::new() {
                        Ok(rt) => {
                            if let Err(e) =
                                rt.block_on(async { wal.log_update(collection_name, vector).await })
                            {
                                error!("Failed to log update to WAL: {}", e);
                            }
                        }
                        Err(e) => {
                            debug!(
                                "Could not create tokio runtime for WAL update (non-async context): {}. WAL logging skipped.",
                                e
                            );
                            // Don't fail the operation if WAL logging fails
                        }
                    }
                }
            }
        }
        // Always return Ok - WAL logging is best-effort and shouldn't fail operations
        Ok(())
    }

    /// Log delete operation to WAL (synchronous wrapper)
    /// Note: This is fire-and-forget to avoid blocking. WAL errors are logged but don't fail the operation.
    /// If no tokio runtime is available, WAL logging is skipped to avoid deadlocks.
    pub(super) fn log_wal_delete(&self, collection_name: &str, vector_id: &str) -> Result<()> {
        let wal_guard = self.wal.lock();
        if let Some(wal) = wal_guard.as_ref() {
            if wal.is_enabled() {
                if let Ok(_handle) = tokio::runtime::Handle::try_current() {
                    let wal_clone = wal.clone();
                    let collection_name = collection_name.to_string();
                    let vector_id = vector_id.to_string();

                    tokio::spawn(async move {
                        if let Err(e) = wal_clone.log_delete(&collection_name, &vector_id).await {
                            error!("Failed to log delete to WAL: {}", e);
                        }
                    });
                } else {
                    // Skip WAL logging when no tokio runtime is available
                    // Creating a new runtime here would cause deadlocks when called from async context
                    debug!(
                        "Skipping WAL delete log for {}/{} - no tokio runtime available",
                        collection_name, vector_id
                    );
                }
            }
        }
        Ok(())
    }

    /// Enable WAL for this vector store
    pub async fn enable_wal(
        &self,
        data_dir: PathBuf,
        config: Option<crate::persistence::wal::WALConfig>,
    ) -> Result<()> {
        let wal = WalIntegration::new(data_dir, config)
            .await
            .map_err(|e| VectorizerError::Storage(format!("Failed to enable WAL: {}", e)))?;

        let mut wal_guard = self.wal.lock();
        *wal_guard = Some(wal);
        info!("WAL enabled for VectorStore");
        Ok(())
    }

    /// Recover collection from WAL after crash
    pub async fn recover_from_wal(
        &self,
        collection_name: &str,
    ) -> Result<Vec<crate::persistence::types::WALEntry>> {
        let wal_guard = self.wal.lock();
        if let Some(wal) = wal_guard.as_ref() {
            wal.recover_collection(collection_name)
                .await
                .map_err(|e| VectorizerError::Storage(format!("WAL recovery failed: {}", e)))
        } else {
            Ok(Vec::new())
        }
    }

    /// Recover and replay WAL entries for a collection
    pub async fn recover_and_replay_wal(&self, collection_name: &str) -> Result<usize> {
        use crate::persistence::types::Operation;

        let entries = self.recover_from_wal(collection_name).await?;
        if entries.is_empty() {
            debug!(
                "No WAL entries to recover for collection '{}'",
                collection_name
            );
            return Ok(0);
        }

        info!(
            "Recovering {} WAL entries for collection '{}'",
            entries.len(),
            collection_name
        );

        let mut replayed = 0;

        for entry in entries {
            match &entry.operation {
                Operation::InsertVector {
                    vector_id,
                    data,
                    metadata,
                    collection_name: _,
                } => {
                    // Reconstruct payload from metadata
                    let payload = if !metadata.is_empty() {
                        use serde_json::json;

                        use crate::models::Payload;
                        let mut payload_data = serde_json::Map::new();
                        for (k, v) in metadata {
                            payload_data.insert(k.clone(), json!(v));
                        }
                        Some(Payload {
                            data: json!(payload_data),
                        })
                    } else {
                        None
                    };

                    let vector = Vector {
                        id: vector_id.clone(),
                        data: data.clone(),
                        payload,
                        sparse: None,
                        document_id: None,
                    };

                    // Try to insert (may fail if already exists, which is OK)
                    if self.insert(collection_name, vec![vector]).is_ok() {
                        replayed += 1;
                    }
                }
                Operation::UpdateVector {
                    vector_id,
                    data,
                    metadata,
                    collection_name: _,
                } => {
                    if let Some(data) = data {
                        // Reconstruct payload from metadata
                        let payload = if let Some(metadata) = metadata {
                            if !metadata.is_empty() {
                                use serde_json::json;

                                use crate::models::Payload;
                                let mut payload_data = serde_json::Map::new();
                                for (k, v) in metadata {
                                    payload_data.insert(k.clone(), json!(v));
                                }
                                Some(Payload {
                                    data: json!(payload_data),
                                })
                            } else {
                                None
                            }
                        } else {
                            None
                        };

                        let vector = Vector {
                            id: vector_id.clone(),
                            data: data.clone(),
                            payload,
                            sparse: None,
                            document_id: None,
                        };

                        // Try to update (may fail if doesn't exist, which is OK)
                        if self.update(collection_name, vector).is_ok() {
                            replayed += 1;
                        }
                    }
                }
                Operation::DeleteVector {
                    vector_id,
                    collection_name: _,
                } => {
                    // Try to delete (may fail if doesn't exist, which is OK)
                    if self.delete(collection_name, vector_id).is_ok() {
                        replayed += 1;
                    }
                }
                Operation::Checkpoint { .. } => {
                    // Checkpoint entries are informational, skip
                    debug!("Skipping checkpoint entry in recovery");
                }
                Operation::CreateCollection { .. } | Operation::DeleteCollection { .. } => {
                    // Collection operations are handled separately
                    debug!("Skipping collection operation in recovery");
                }
            }
        }

        info!(
            "Recovered {} operations from WAL for collection '{}'",
            replayed, collection_name
        );

        Ok(replayed)
    }

    /// Recover all collections from WAL (call on startup)
    pub async fn recover_all_from_wal(&self) -> Result<usize> {
        let wal_guard = self.wal.lock();
        if let Some(wal) = wal_guard.as_ref() {
            if !wal.is_enabled() {
                debug!("WAL is disabled, skipping recovery");
                return Ok(0);
            }
        } else {
            return Ok(0);
        }

        // Drop the guard before calling other methods that may need it
        drop(wal_guard);

        // Get all collection names
        let collection_names: Vec<String> = self.list_collections();

        let mut total_recovered = 0;
        for collection_name in collection_names {
            match self.recover_and_replay_wal(&collection_name).await {
                Ok(count) => {
                    total_recovered += count;
                }
                Err(e) => {
                    warn!(
                        "Failed to recover WAL for collection '{}': {}",
                        collection_name, e
                    );
                }
            }
        }

        if total_recovered > 0 {
            info!("Recovered {} total operations from WAL", total_recovered);
        }

        Ok(total_recovered)
    }
}
