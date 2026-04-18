//! Shard data migration for moving vector data between cluster nodes.
//!
//! [`ShardMigrator`] transfers the actual vector data (not just pointer mappings) from
//! a source node to a target node during rebalancing or planned shard moves.
//!
//! # Transfer flow
//!
//! 1. Fetch vector batches from the source node via `GetShardVectors` gRPC (or the local
//!    `VectorStore` when the source is the current node).
//! 2. Insert each batch into the target node via `RemoteInsertVector` gRPC (or the local
//!    `VectorStore`).
//! 3. Track progress in-memory so callers can observe ongoing migrations.

use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use chrono::{DateTime, Utc};
use parking_lot::RwLock;
use thiserror::Error;
use tracing::{debug, error, info, warn};
use uuid::Uuid;

use super::node::NodeId;
use super::server_client::{ClusterClient, ClusterClientPool};
use crate::db::VectorStore;
use crate::db::sharding::ShardId;
use crate::error::VectorizerError;

// ---------------------------------------------------------------------------
// Error type
// ---------------------------------------------------------------------------

/// Errors that can occur during shard data migration.
#[derive(Debug, Error)]
pub enum MigrationError {
    /// The source collection could not be found or read.
    #[error("Source collection error: {0}")]
    SourceCollection(String),

    /// The target node rejected one or more vector inserts.
    #[error("Target insert error (vector '{id}'): {reason}")]
    TargetInsert { id: String, reason: String },

    /// A gRPC or network error occurred during transfer.
    #[error("Transport error: {0}")]
    Transport(String),

    /// A migration with the given ID was not found.
    #[error("Migration not found: {0}")]
    NotFound(String),

    /// The migration was cancelled before it finished.
    #[error("Migration cancelled: {0}")]
    Cancelled(String),

    /// An underlying VectorizerError.
    #[error("Vectorizer error: {0}")]
    Vectorizer(#[from] VectorizerError),
}

// ---------------------------------------------------------------------------
// Status & progress types
// ---------------------------------------------------------------------------

/// Lifecycle status of a single shard migration.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MigrationStatus {
    /// Migration is queued but has not started transferring data yet.
    Pending,
    /// Migration is actively transferring vectors.
    InProgress,
    /// All vectors have been transferred successfully.
    Completed,
    /// Migration failed with the given error message.
    Failed(String),
    /// Migration was cancelled by the caller.
    Cancelled,
}

/// Live progress snapshot for an ongoing or completed shard migration.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct MigrationProgress {
    /// Unique ID for this migration operation.
    pub migration_id: String,
    /// The shard being migrated.
    pub shard_id: u32,
    /// Node the shard is migrating from.
    pub from_node: String,
    /// Node the shard is migrating to.
    pub to_node: String,
    /// Number of vectors successfully transferred so far.
    pub vectors_transferred: u64,
    /// Total vectors to transfer (populated once the source is queried).
    pub total_vectors: u64,
    /// Current status.
    pub status: MigrationStatus,
    /// When the migration started.
    pub started_at: DateTime<Utc>,
}

// ---------------------------------------------------------------------------
// Migration result
// ---------------------------------------------------------------------------

/// Summary returned after a migration attempt completes (or fails).
#[derive(Debug)]
pub struct MigrationResult {
    /// Unique migration ID.
    pub migration_id: String,
    /// Whether the migration succeeded.
    pub success: bool,
    /// Human-readable message.
    pub message: String,
    /// Number of vectors that were transferred.
    pub vectors_transferred: u64,
    /// Total vectors that were in the source shard.
    pub total_vectors: u64,
}

// ---------------------------------------------------------------------------
// ShardMigrator
// ---------------------------------------------------------------------------

/// Default batch size when transferring vectors between nodes.
const DEFAULT_BATCH_SIZE: u32 = 500;

/// Migrates shard vector data between cluster nodes.
///
/// Call [`ShardMigrator::migrate_shard_data`] to start a migration.  Progress can
/// be observed via [`ShardMigrator::list_migrations`] while the migration is running.
#[derive(Clone)]
pub struct ShardMigrator {
    /// Pool of gRPC clients used to contact remote nodes.
    client_pool: ClusterClientPool,
    /// Local vector store (used when source or target is the current node).
    store: Arc<VectorStore>,
    /// ID of the current node (used to detect local vs. remote transfers).
    local_node_id: NodeId,
    /// Active and recently completed migrations, keyed by migration ID.
    active_migrations: Arc<RwLock<HashMap<String, MigrationProgress>>>,
}

impl ShardMigrator {
    /// Create a new [`ShardMigrator`].
    ///
    /// - `client_pool` – shared pool used to open gRPC connections to remote nodes.
    /// - `store` – the local [`VectorStore`] for reads/writes when operating on the current node.
    /// - `local_node_id` – the node ID of the running process (used to distinguish local vs. remote).
    pub fn new(
        client_pool: ClusterClientPool,
        store: Arc<VectorStore>,
        local_node_id: NodeId,
    ) -> Self {
        Self {
            client_pool,
            store,
            local_node_id,
            active_migrations: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Returns a snapshot of all tracked migrations (active and recent).
    pub fn list_migrations(&self) -> Vec<MigrationProgress> {
        let migrations = self.active_migrations.read();
        migrations.values().cloned().collect()
    }

    /// Returns the progress for a single migration by ID, if it exists.
    pub fn get_migration(&self, migration_id: &str) -> Option<MigrationProgress> {
        let migrations = self.active_migrations.read();
        migrations.get(migration_id).cloned()
    }

    /// Transfer all vector data for `collection_name` from `from_node` to `to_node`.
    ///
    /// The shard mapping in the router is **not** updated here; that is the caller's
    /// responsibility.  This method only moves the underlying vector data.
    ///
    /// # Arguments
    ///
    /// - `shard_id` – the shard being migrated (stored in progress tracking).
    /// - `from_node` – source node ID and its gRPC address (`"host:port"`).
    /// - `to_node` – target node ID and its gRPC address (`"host:port"`).
    /// - `collection_name` – collection whose vectors belong to this shard.
    ///
    /// # Errors
    ///
    /// Returns [`MigrationError`] if the source cannot be read or the target rejects writes.
    pub async fn migrate_shard_data(
        &self,
        shard_id: ShardId,
        from_node: (&NodeId, &str),
        to_node: (&NodeId, &str),
        collection_name: &str,
    ) -> Result<MigrationResult, MigrationError> {
        let migration_id = Uuid::new_v4().to_string();
        let (from_node_id, from_addr) = from_node;
        let (to_node_id, to_addr) = to_node;

        info!(
            migration_id = %migration_id,
            shard_id = shard_id.as_u32(),
            from_node = %from_node_id,
            to_node = %to_node_id,
            collection = %collection_name,
            "Starting shard data migration",
        );

        // Register migration as Pending
        {
            let mut migrations = self.active_migrations.write();
            migrations.insert(
                migration_id.clone(),
                MigrationProgress {
                    migration_id: migration_id.clone(),
                    shard_id: shard_id.as_u32(),
                    from_node: from_node_id.as_str().to_string(),
                    to_node: to_node_id.as_str().to_string(),
                    vectors_transferred: 0,
                    total_vectors: 0,
                    status: MigrationStatus::Pending,
                    started_at: Utc::now(),
                },
            );
        }

        // Transition to InProgress
        self.set_status(&migration_id, MigrationStatus::InProgress);

        let result = self
            .run_migration(
                &migration_id,
                shard_id,
                from_node_id,
                from_addr,
                to_node_id,
                to_addr,
                collection_name,
            )
            .await;

        match &result {
            Ok(res) => {
                info!(
                    migration_id = %migration_id,
                    vectors_transferred = res.vectors_transferred,
                    "Shard migration completed successfully",
                );
                self.set_status(&migration_id, MigrationStatus::Completed);
            }
            Err(e) => {
                error!(
                    migration_id = %migration_id,
                    error = %e,
                    "Shard migration failed",
                );
                self.set_status(&migration_id, MigrationStatus::Failed(e.to_string()));
            }
        }

        result
    }

    // ------------------------------------------------------------------
    // Internal helpers
    // ------------------------------------------------------------------

    /// Execute the actual paginated transfer loop.
    async fn run_migration(
        &self,
        migration_id: &str,
        shard_id: ShardId,
        from_node_id: &NodeId,
        from_addr: &str,
        to_node_id: &NodeId,
        to_addr: &str,
        collection_name: &str,
    ) -> Result<MigrationResult, MigrationError> {
        let is_local_source = from_node_id == &self.local_node_id;
        let is_local_target = to_node_id == &self.local_node_id;

        let mut offset: u32 = 0;
        let mut total_vectors: u64 = 0;
        let mut vectors_transferred: u64 = 0;

        loop {
            // ---- Fetch batch from source ----
            let batch = if is_local_source {
                self.fetch_local_batch(collection_name, offset, DEFAULT_BATCH_SIZE)?
            } else {
                self.fetch_remote_batch(
                    from_node_id,
                    from_addr,
                    collection_name,
                    shard_id.as_u32(),
                    offset,
                    DEFAULT_BATCH_SIZE,
                )
                .await?
            };

            if total_vectors == 0 {
                total_vectors = batch.total_count as u64;
                self.update_total(migration_id, total_vectors);
                debug!(
                    migration_id = %migration_id,
                    total_vectors,
                    "Discovered total vector count for migration",
                );
            }

            if batch.vectors.is_empty() {
                break;
            }

            let batch_len = batch.vectors.len() as u64;

            // ---- Insert batch into target ----
            if is_local_target {
                self.insert_local_batch(collection_name, &batch.vectors)?;
            } else {
                self.insert_remote_batch(to_node_id, to_addr, collection_name, &batch.vectors)
                    .await?;
            }

            vectors_transferred += batch_len;
            offset += batch_len as u32;

            self.update_transferred(migration_id, vectors_transferred);

            debug!(
                migration_id = %migration_id,
                vectors_transferred,
                total_vectors,
                "Migration batch transferred",
            );

            if !batch.has_more {
                break;
            }
        }

        Ok(MigrationResult {
            migration_id: migration_id.to_string(),
            success: true,
            message: format!(
                "Migrated {} vectors from {} to {}",
                vectors_transferred, from_node_id, to_node_id,
            ),
            vectors_transferred,
            total_vectors,
        })
    }

    // ------------------------------------------------------------------
    // Source helpers
    // ------------------------------------------------------------------

    /// Fetch a batch of vectors from the local VectorStore.
    fn fetch_local_batch(
        &self,
        collection_name: &str,
        offset: u32,
        limit: u32,
    ) -> Result<BatchResult, MigrationError> {
        let collection = self.store.get_collection(collection_name).map_err(|e| {
            MigrationError::SourceCollection(format!(
                "Cannot read local collection '{}': {}",
                collection_name, e
            ))
        })?;

        let all_vectors = collection.get_all_vectors();
        let total_count = all_vectors.len() as u32;
        let offset_usize = offset as usize;
        let limit_usize = limit as usize;

        let vectors: Vec<VectorEntry> = all_vectors
            .into_iter()
            .skip(offset_usize)
            .take(limit_usize)
            .map(|v| VectorEntry {
                id: v.id,
                vector: v.data,
                payload_json: v
                    .payload
                    .as_ref()
                    .and_then(|p| serde_json::to_string(p).ok()),
            })
            .collect();

        let fetched = vectors.len() as u32;
        let has_more = (offset + fetched) < total_count;

        Ok(BatchResult {
            vectors,
            total_count,
            has_more,
        })
    }

    /// Fetch a batch of vectors from a remote node via gRPC.
    async fn fetch_remote_batch(
        &self,
        node_id: &NodeId,
        address: &str,
        collection_name: &str,
        shard_id: u32,
        offset: u32,
        limit: u32,
    ) -> Result<BatchResult, MigrationError> {
        let client = self
            .client_pool
            .get_client(node_id, address)
            .await
            .map_err(|e| MigrationError::Transport(e.to_string()))?;

        let (proto_vectors, total_count, has_more) = client
            .get_shard_vectors(collection_name, shard_id, offset, limit, None)
            .await
            .map_err(|e| MigrationError::Transport(e.to_string()))?;

        let vectors: Vec<VectorEntry> = proto_vectors
            .into_iter()
            .map(|v| VectorEntry {
                id: v.id,
                vector: v.vector,
                payload_json: v.payload_json,
            })
            .collect();

        Ok(BatchResult {
            vectors,
            total_count,
            has_more,
        })
    }

    // ------------------------------------------------------------------
    // Target helpers
    // ------------------------------------------------------------------

    /// Insert a batch of vectors into the local VectorStore.
    fn insert_local_batch(
        &self,
        collection_name: &str,
        vectors: &[VectorEntry],
    ) -> Result<(), MigrationError> {
        for entry in vectors {
            let payload = entry
                .payload_json
                .as_deref()
                .map(|json| serde_json::from_str(json))
                .transpose()
                .map_err(|e| MigrationError::TargetInsert {
                    id: entry.id.clone(),
                    reason: format!("Failed to parse payload JSON: {}", e),
                })?;

            let vector_obj = crate::models::Vector {
                id: entry.id.clone(),
                data: entry.vector.clone(),
                sparse: None,
                payload,
                document_id: None,
            };

            let mut collection = self
                .store
                .get_collection_mut(collection_name)
                .map_err(|e| MigrationError::TargetInsert {
                    id: entry.id.clone(),
                    reason: format!("Cannot get mutable collection: {}", e),
                })?;

            collection
                .add_vector(entry.id.clone(), vector_obj)
                .map_err(|e| MigrationError::TargetInsert {
                    id: entry.id.clone(),
                    reason: e.to_string(),
                })?;
        }
        Ok(())
    }

    /// Insert a batch of vectors into a remote node via gRPC.
    async fn insert_remote_batch(
        &self,
        node_id: &NodeId,
        address: &str,
        collection_name: &str,
        vectors: &[VectorEntry],
    ) -> Result<(), MigrationError> {
        let client = self
            .client_pool
            .get_client(node_id, address)
            .await
            .map_err(|e| MigrationError::Transport(e.to_string()))?;

        for entry in vectors {
            let payload: Option<serde_json::Value> = entry
                .payload_json
                .as_deref()
                .map(|json| serde_json::from_str(json))
                .transpose()
                .map_err(|e| MigrationError::TargetInsert {
                    id: entry.id.clone(),
                    reason: format!("Failed to parse payload JSON: {}", e),
                })?;

            client
                .insert_vector(
                    collection_name,
                    &entry.id,
                    &entry.vector,
                    payload.as_ref(),
                    None,
                )
                .await
                .map_err(|e| MigrationError::TargetInsert {
                    id: entry.id.clone(),
                    reason: e.to_string(),
                })?;
        }
        Ok(())
    }

    // ------------------------------------------------------------------
    // Progress tracking helpers
    // ------------------------------------------------------------------

    fn set_status(&self, migration_id: &str, status: MigrationStatus) {
        let mut migrations = self.active_migrations.write();
        if let Some(progress) = migrations.get_mut(migration_id) {
            progress.status = status;
        }
    }

    fn update_total(&self, migration_id: &str, total: u64) {
        let mut migrations = self.active_migrations.write();
        if let Some(progress) = migrations.get_mut(migration_id) {
            progress.total_vectors = total;
        }
    }

    fn update_transferred(&self, migration_id: &str, transferred: u64) {
        let mut migrations = self.active_migrations.write();
        if let Some(progress) = migrations.get_mut(migration_id) {
            progress.vectors_transferred = transferred;
        }
    }
}

// ---------------------------------------------------------------------------
// Internal transfer types (not exposed in the public API)
// ---------------------------------------------------------------------------

/// A normalized vector entry used during transfer regardless of source.
struct VectorEntry {
    id: String,
    vector: Vec<f32>,
    payload_json: Option<String>,
}

/// Result of a single paginated fetch from a source node.
struct BatchResult {
    vectors: Vec<VectorEntry>,
    total_count: u32,
    has_more: bool,
}
