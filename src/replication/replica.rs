//! Replica Node - Read-only node that receives updates from master
//!
//! Features:
//! - Connects to master node
//! - Receives full/partial sync
//! - Applies operations to local store
//! - Auto-reconnect on disconnect
//! - Read-only enforcement

use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use parking_lot::RwLock;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::time::sleep;
use tracing::{debug, error, info, warn};
use uuid::Uuid;

use super::config::ReplicationConfig;
use super::types::{
    ReplicationCommand, ReplicationError, ReplicationOperation, ReplicationResult,
    ReplicationStats, VectorOperation,
};
use crate::db::VectorStore;

/// Replica Node - Read-only node that receives from master
pub struct ReplicaNode {
    config: ReplicationConfig,
    vector_store: Arc<VectorStore>,

    /// Stable identifier for this replica instance across reconnects
    replica_id: String,

    /// Current replication state
    state: Arc<RwLock<ReplicaState>>,
}

#[derive(Debug, Clone)]
struct ReplicaState {
    /// Last replicated offset
    offset: u64,
    /// Last heartbeat timestamp
    last_heartbeat: u64,
    /// Connected to master
    connected: bool,
    /// Total operations replicated
    total_replicated: u64,
    /// Total bytes received
    total_bytes: u64,
}

impl Default for ReplicaState {
    fn default() -> Self {
        Self {
            offset: 0,
            last_heartbeat: 0,
            connected: false,
            total_replicated: 0,
            total_bytes: 0,
        }
    }
}

impl ReplicaNode {
    /// Create a new replica node
    pub fn new(config: ReplicationConfig, vector_store: Arc<VectorStore>) -> Self {
        Self {
            config,
            vector_store,
            replica_id: Uuid::new_v4().to_string(),
            state: Arc::new(RwLock::new(ReplicaState::default())),
        }
    }

    /// Start the replica node (connects to master and processes updates)
    pub async fn start(&self) -> ReplicationResult<()> {
        // Validate that we have a master address configured (either raw or resolved)
        if self.config.master_address.is_none() && self.config.master_address_raw.is_none() {
            return Err(ReplicationError::Connection(
                "No master address configured".to_string(),
            ));
        }

        let display_addr = self
            .config
            .master_address_raw
            .clone()
            .or_else(|| self.config.master_address.map(|a| a.to_string()))
            .unwrap_or_else(|| "unknown".to_string());
        info!("Replica node starting, master address: {}", display_addr);

        let base_interval = self.config.reconnect_duration();
        let max_interval = Duration::from_secs(60); // Cap at 60 seconds
        let mut current_interval = base_interval;
        let mut consecutive_failures: u32 = 0;

        loop {
            // Re-resolve DNS on each reconnect attempt so we follow the
            // master to a new IP after a pod restart in Kubernetes.
            let master_addr = match self.config.resolve_master_address().await {
                Some(addr) => addr,
                None => {
                    error!("Failed to resolve master address, will retry...");
                    sleep(current_interval).await;
                    current_interval = (current_interval * 2).min(max_interval);
                    consecutive_failures = consecutive_failures.saturating_add(1);
                    continue;
                }
            };

            match self.connect_and_sync(master_addr).await {
                Ok(_) => {
                    info!("Disconnected from master, will reconnect...");
                    // Reset backoff after a successful connection.
                    current_interval = base_interval;
                    consecutive_failures = 0;
                }
                Err(e) => {
                    consecutive_failures = consecutive_failures.saturating_add(1);
                    if consecutive_failures <= 3 {
                        error!(
                            "Replication error: {}, will retry in {:?}...",
                            e, current_interval
                        );
                    } else if consecutive_failures % 12 == 0 {
                        // Log every ~12 failures to reduce spam (roughly once per minute at max backoff).
                        warn!(
                            "Replication still failing after {} attempts: {} (retrying every {:?})",
                            consecutive_failures, e, current_interval
                        );
                    }
                }
            }

            // Update state
            {
                let mut state = self.state.write();
                state.connected = false;
            }

            // Wait with exponential backoff (base * 2^min(failures-1, 4), capped at max_interval).
            sleep(current_interval).await;
            current_interval = (current_interval * 2).min(max_interval);
        }
    }

    /// Connect to master and sync
    async fn connect_and_sync(&self, master_addr: std::net::SocketAddr) -> ReplicationResult<()> {
        info!("Connecting to master at {}", master_addr);

        // Use a 5-second connection timeout to fail fast when the master is
        // unreachable (e.g., pod not ready in Kubernetes), instead of waiting
        // for the OS TCP timeout which can be 30-120 seconds.
        let mut stream =
            tokio::time::timeout(Duration::from_secs(5), TcpStream::connect(master_addr))
                .await
                .map_err(|_| {
                    ReplicationError::Connection(format!(
                        "Connection to master at {} timed out after 5s",
                        master_addr
                    ))
                })??;

        // Update state
        {
            let mut state = self.state.write();
            state.connected = true;
            state.last_heartbeat = current_timestamp();
        }

        // Send current offset to master
        let current_offset = self.state.read().offset;
        let data = crate::codec::serialize(&current_offset)?;
        let len = (data.len() as u32).to_be_bytes();

        stream.write_all(&len).await?;
        stream.write_all(&data).await?;
        stream.flush().await?;

        info!("Sent offset {} to master", current_offset);

        // Process commands from master
        loop {
            let cmd = self.receive_command(&mut stream).await?;

            match cmd {
                ReplicationCommand::FullSync {
                    snapshot_data,
                    offset,
                } => {
                    info!("Receiving full sync (offset: {})", offset);

                    // Apply snapshot
                    super::sync::apply_snapshot(&self.vector_store, &snapshot_data)
                        .await
                        .map_err(|e| ReplicationError::Sync(e))?;

                    // Update state
                    {
                        let mut state = self.state.write();
                        state.offset = offset;
                        state.total_bytes += snapshot_data.len() as u64;
                    }

                    info!("Full sync completed at offset {}", offset);
                }
                ReplicationCommand::PartialSync {
                    from_offset,
                    operations,
                } => {
                    info!(
                        "Receiving partial sync from offset {} ({} operations)",
                        from_offset,
                        operations.len()
                    );

                    // Apply operations
                    for op in operations {
                        self.apply_operation(&op.operation).await?;

                        // Update state
                        let new_offset = op.offset;
                        {
                            let mut state = self.state.write();
                            state.offset = new_offset;
                            state.total_replicated += 1;
                        }
                    }

                    info!("Partial sync completed");
                }
                ReplicationCommand::Operation(op) => {
                    debug!("Receiving operation at offset {}", op.offset);

                    // Apply operation
                    self.apply_operation(&op.operation).await?;

                    // Update state and capture offset for ACK
                    let confirmed_offset = op.offset;
                    {
                        let mut state = self.state.write();
                        state.offset = confirmed_offset;
                        state.total_replicated += 1;
                    }

                    // Send ACK back to master on the same stream
                    if let Err(e) =
                        Self::send_ack(&mut stream, &self.replica_id, confirmed_offset).await
                    {
                        warn!(
                            "Failed to send ACK to master for offset {}: {}",
                            confirmed_offset, e
                        );
                    } else {
                        debug!("Sent ACK to master for offset {}", confirmed_offset);
                    }
                }
                ReplicationCommand::Heartbeat {
                    master_offset,
                    timestamp,
                } => {
                    debug!(
                        "Received heartbeat: master_offset={}, timestamp={}",
                        master_offset, timestamp
                    );

                    // Update heartbeat
                    {
                        let mut state = self.state.write();
                        state.last_heartbeat = current_timestamp();
                    }
                }
                ReplicationCommand::Ack { .. } => {
                    // Replicas don't process ACKs
                    warn!("Received unexpected ACK command");
                }
            }
        }
    }

    /// Send an ACK frame back to master on the shared TCP stream.
    ///
    /// Called after each `Operation` is successfully applied so the master can
    /// track which offset each replica has confirmed.
    async fn send_ack(
        stream: &mut TcpStream,
        replica_id: &str,
        offset: u64,
    ) -> ReplicationResult<()> {
        let ack = ReplicationCommand::Ack {
            replica_id: replica_id.to_string(),
            offset,
        };
        let data = crate::codec::serialize(&ack)?;
        let len = (data.len() as u32).to_be_bytes();
        stream.write_all(&len).await?;
        stream.write_all(&data).await?;
        stream.flush().await?;
        Ok(())
    }

    /// Receive a command from master
    async fn receive_command(
        &self,
        stream: &mut TcpStream,
    ) -> ReplicationResult<ReplicationCommand> {
        use crate::monitoring::metrics::METRICS;

        let mut len_buf = [0u8; 4];
        stream.read_exact(&mut len_buf).await?;
        let len = u32::from_be_bytes(len_buf) as usize;

        let mut data_buf = vec![0u8; len];
        stream.read_exact(&mut data_buf).await?;

        // Track bytes received (4 bytes for length + data)
        let total_bytes = 4 + len;
        METRICS
            .replication_bytes_received_total
            .inc_by(total_bytes as f64);

        // Update state
        let mut state = self.state.write();
        state.total_bytes += total_bytes as u64;
        drop(state);

        let cmd: ReplicationCommand = crate::codec::deserialize(&data_buf)?;

        Ok(cmd)
    }

    /// Apply a vector operation
    ///
    /// For multi-tenant mode (HiveHub integration), the owner_id is preserved
    /// during replication to maintain tenant ownership on replica nodes.
    async fn apply_operation(&self, operation: &VectorOperation) -> ReplicationResult<()> {
        match operation {
            VectorOperation::CreateCollection {
                name,
                config,
                owner_id,
            } => {
                let collection_config = crate::models::CollectionConfig {
                    dimension: config.dimension,
                    metric: parse_distance_metric(&config.metric),
                    hnsw_config: crate::models::HnswConfig::default(),
                    quantization: crate::models::QuantizationConfig::None,
                    compression: Default::default(),
                    normalization: None,
                    storage_type: Some(crate::models::StorageType::Memory),
                    sharding: None,
                    graph: None,
                    encryption: None,
                };

                // In multi-tenant mode, we use create_collection_with_owner if owner_id is present
                if let Some(owner) = owner_id {
                    if let Ok(uuid) = uuid::Uuid::parse_str(owner) {
                        self.vector_store
                            .create_collection_with_owner(name, collection_config, uuid)
                            .map_err(|e| ReplicationError::InvalidOperation(e.to_string()))?;
                        debug!("Created collection: {} with owner: {}", name, owner);
                    } else {
                        // Fallback to regular creation if UUID is invalid
                        self.vector_store
                            .create_collection(name, collection_config)
                            .map_err(|e| ReplicationError::InvalidOperation(e.to_string()))?;
                        debug!("Created collection: {} (invalid owner_id: {})", name, owner);
                    }
                } else {
                    self.vector_store
                        .create_collection(name, collection_config)
                        .map_err(|e| ReplicationError::InvalidOperation(e.to_string()))?;
                    debug!("Created collection: {}", name);
                }
            }
            VectorOperation::DeleteCollection { name, owner_id: _ } => {
                // owner_id is used for audit/logging, actual deletion uses collection name
                self.vector_store
                    .delete_collection(name)
                    .map_err(|e| ReplicationError::InvalidOperation(e.to_string()))?;

                debug!("Deleted collection: {}", name);
            }
            VectorOperation::InsertVector {
                collection,
                id,
                vector,
                payload,
                owner_id: _,
            } => {
                // owner_id is preserved in the operation for audit purposes
                // The collection already has the owner association
                let payload_obj = payload.as_ref().map(|p| crate::models::Payload {
                    data: serde_json::from_slice(p).unwrap_or_default(),
                });

                let vec = crate::models::Vector {
                    id: id.clone(),
                    data: vector.clone(),
                    sparse: None,
                    payload: payload_obj,
                    document_id: None,
                };

                self.vector_store
                    .insert(collection, vec![vec])
                    .map_err(|e| ReplicationError::InvalidOperation(e.to_string()))?;

                debug!("Inserted vector {} in collection {}", id, collection);
            }
            VectorOperation::UpdateVector {
                collection,
                id,
                vector,
                payload,
                owner_id: _,
            } => {
                // For update, we use upsert semantics
                // owner_id is preserved for audit purposes
                let payload_obj = payload.as_ref().map(|p| crate::models::Payload {
                    data: serde_json::from_slice(p).unwrap_or_default(),
                });

                if let Some(data) = vector {
                    let vec = crate::models::Vector {
                        id: id.clone(),
                        data: data.clone(),
                        sparse: None,
                        payload: payload_obj,
                        document_id: None,
                    };

                    self.vector_store
                        .insert(collection, vec![vec])
                        .map_err(|e| ReplicationError::InvalidOperation(e.to_string()))?;
                }

                debug!("Updated vector {} in collection {}", id, collection);
            }
            VectorOperation::DeleteVector {
                collection,
                id,
                owner_id: _,
            } => {
                // owner_id is preserved for audit purposes
                info!(
                    "Replica applying delete operation: {} from {}",
                    id, collection
                );
                self.vector_store.delete(collection, id).map_err(|e| {
                    error!("Failed to delete vector {} from {}: {}", id, collection, e);
                    ReplicationError::InvalidOperation(e.to_string())
                })?;

                info!("Deleted vector {} from collection {}", id, collection);
            }
        }

        Ok(())
    }

    /// Get replication statistics
    pub fn get_stats(&self) -> ReplicationStats {
        let state = self.state.read();

        ReplicationStats {
            role: crate::replication::NodeRole::Replica,
            lag_ms: current_timestamp().saturating_sub(state.last_heartbeat),
            bytes_sent: 0, // Replicas don't send data
            bytes_received: state.total_bytes,
            last_sync: UNIX_EPOCH + Duration::from_millis(state.last_heartbeat),
            operations_pending: 0,    // Would need master offset to calculate
            snapshot_size: 0,         // Not tracked by replica
            connected_replicas: None, // Only master has replicas
            // Legacy fields
            master_offset: 0, // Not tracked by replica
            replica_offset: state.offset,
            lag_operations: 0, // Would need master offset to calculate
            total_replicated: state.total_replicated,
        }
    }

    /// Check if connected to master
    pub fn is_connected(&self) -> bool {
        self.state.read().connected
    }

    /// Get current offset
    pub fn get_offset(&self) -> u64 {
        self.state.read().offset
    }
}

fn parse_distance_metric(metric: &str) -> crate::models::DistanceMetric {
    match metric.to_lowercase().as_str() {
        "euclidean" => crate::models::DistanceMetric::Euclidean,
        "cosine" => crate::models::DistanceMetric::Cosine,
        "dotproduct" | "dot_product" => crate::models::DistanceMetric::DotProduct,
        _ => crate::models::DistanceMetric::Cosine,
    }
}

fn current_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis() as u64
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use super::*;
    use crate::db::VectorStore;
    use crate::replication::{NodeRole, ReplicationConfig};

    #[tokio::test]
    async fn test_replica_creation_and_initial_state() {
        let store = Arc::new(VectorStore::new());
        let config = ReplicationConfig {
            role: NodeRole::Replica,
            bind_address: None,
            master_address: Some("127.0.0.1:7000".parse().unwrap()),
            master_address_raw: None,
            heartbeat_interval: 5,
            replica_timeout: 30,
            log_size: 1000,
            reconnect_interval: 5,
            wal_enabled: false,
            wal_dir: None,
        };

        let replica = ReplicaNode::new(config, store);

        // Test initial state
        assert_eq!(replica.get_offset(), 0);
        assert!(!replica.is_connected());

        // Test initial stats
        let stats = replica.get_stats();
        assert_eq!(stats.replica_offset, 0);
        assert_eq!(stats.total_replicated, 0);
        assert_eq!(stats.bytes_received, 0); // New field for bytes received
        assert_eq!(stats.role, crate::replication::NodeRole::Replica);
        assert_eq!(stats.master_offset, 0);
    }

    #[test]
    fn test_parse_distance_metric_variants() {
        // Test all variants
        assert_eq!(
            parse_distance_metric("euclidean"),
            crate::models::DistanceMetric::Euclidean
        );
        assert_eq!(
            parse_distance_metric("EUCLIDEAN"),
            crate::models::DistanceMetric::Euclidean
        );
        assert_eq!(
            parse_distance_metric("cosine"),
            crate::models::DistanceMetric::Cosine
        );
        assert_eq!(
            parse_distance_metric("COSINE"),
            crate::models::DistanceMetric::Cosine
        );
        assert_eq!(
            parse_distance_metric("dotproduct"),
            crate::models::DistanceMetric::DotProduct
        );
        assert_eq!(
            parse_distance_metric("dot_product"),
            crate::models::DistanceMetric::DotProduct
        );
        assert_eq!(
            parse_distance_metric("DOT_PRODUCT"),
            crate::models::DistanceMetric::DotProduct
        );

        // Test default (unknown)
        assert_eq!(
            parse_distance_metric("unknown"),
            crate::models::DistanceMetric::Cosine
        );
    }
}
