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
            state: Arc::new(RwLock::new(ReplicaState::default())),
        }
    }

    /// Start the replica node (connects to master and processes updates)
    pub async fn start(&self) -> ReplicationResult<()> {
        let master_addr = self.config.master_address.ok_or_else(|| {
            ReplicationError::Connection("No master address configured".to_string())
        })?;

        info!(
            "Replica node starting, connecting to master at {}",
            master_addr
        );

        loop {
            match self.connect_and_sync(master_addr).await {
                Ok(_) => {
                    info!("Disconnected from master, will reconnect...");
                }
                Err(e) => {
                    error!("Replication error: {}, will retry...", e);
                }
            }

            // Update state
            {
                let mut state = self.state.write();
                state.connected = false;
            }

            // Wait before reconnecting
            sleep(self.config.reconnect_duration()).await;
        }
    }

    /// Connect to master and sync
    async fn connect_and_sync(&self, master_addr: std::net::SocketAddr) -> ReplicationResult<()> {
        info!("Connecting to master at {}", master_addr);

        let mut stream = TcpStream::connect(master_addr).await?;

        // Update state
        {
            let mut state = self.state.write();
            state.connected = true;
            state.last_heartbeat = current_timestamp();
        }

        // Send current offset to master
        let current_offset = self.state.read().offset;
        let data = bincode::serialize(&current_offset)?;
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
                        {
                            let mut state = self.state.write();
                            state.offset = op.offset;
                            state.total_replicated += 1;
                        }
                    }

                    info!("Partial sync completed");
                }
                ReplicationCommand::Operation(op) => {
                    debug!("Receiving operation at offset {}", op.offset);

                    // Apply operation
                    self.apply_operation(&op.operation).await?;

                    // Update state
                    {
                        let mut state = self.state.write();
                        state.offset = op.offset;
                        state.total_replicated += 1;
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

    /// Receive a command from master
    async fn receive_command(
        &self,
        stream: &mut TcpStream,
    ) -> ReplicationResult<ReplicationCommand> {
        let mut len_buf = [0u8; 4];
        stream.read_exact(&mut len_buf).await?;
        let len = u32::from_be_bytes(len_buf) as usize;

        let mut data_buf = vec![0u8; len];
        stream.read_exact(&mut data_buf).await?;

        let cmd: ReplicationCommand = bincode::deserialize(&data_buf)?;

        Ok(cmd)
    }

    /// Apply a vector operation
    async fn apply_operation(&self, operation: &VectorOperation) -> ReplicationResult<()> {
        match operation {
            VectorOperation::CreateCollection { name, config } => {
                let collection_config = crate::models::CollectionConfig {
                    dimension: config.dimension,
                    metric: parse_distance_metric(&config.metric),
                    hnsw_config: crate::models::HnswConfig::default(),
                    quantization: crate::models::QuantizationConfig::None,
                    compression: Default::default(),
                    normalization: None,
                };

                self.vector_store
                    .create_collection(name, collection_config)
                    .map_err(|e| ReplicationError::InvalidOperation(e.to_string()))?;

                debug!("Created collection: {}", name);
            }
            VectorOperation::DeleteCollection { name } => {
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
            } => {
                let payload_obj = payload.as_ref().map(|p| crate::models::Payload {
                    data: serde_json::from_slice(p).unwrap_or_default(),
                });

                let vec = crate::models::Vector {
                    id: id.clone(),
                    data: vector.clone(),
                    payload: payload_obj,
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
            } => {
                // For update, we use upsert semantics
                let payload_obj = payload.as_ref().map(|p| crate::models::Payload {
                    data: serde_json::from_slice(p).unwrap_or_default(),
                });

                if let Some(data) = vector {
                    let vec = crate::models::Vector {
                        id: id.clone(),
                        data: data.clone(),
                        payload: payload_obj,
                    };

                    self.vector_store
                        .insert(collection, vec![vec])
                        .map_err(|e| ReplicationError::InvalidOperation(e.to_string()))?;
                }

                debug!("Updated vector {} in collection {}", id, collection);
            }
            VectorOperation::DeleteVector { collection, id } => {
                self.vector_store
                    .delete(collection, id)
                    .map_err(|e| ReplicationError::InvalidOperation(e.to_string()))?;

                debug!("Deleted vector {} from collection {}", id, collection);
            }
        }

        Ok(())
    }

    /// Get replication statistics
    pub fn get_stats(&self) -> ReplicationStats {
        let state = self.state.read();

        ReplicationStats {
            master_offset: 0, // Not tracked by replica
            replica_offset: state.offset,
            lag_operations: 0, // Would need master offset to calculate
            lag_ms: current_timestamp().saturating_sub(state.last_heartbeat),
            total_replicated: state.total_replicated,
            total_bytes: state.total_bytes,
            last_heartbeat: state.last_heartbeat,
            connected: state.connected,
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
            heartbeat_interval: 5,
            replica_timeout: 30,
            log_size: 1000,
            reconnect_interval: 5,
        };

        let replica = ReplicaNode::new(config, store);

        // Test initial state
        assert_eq!(replica.get_offset(), 0);
        assert!(!replica.is_connected());

        // Test initial stats
        let stats = replica.get_stats();
        assert_eq!(stats.replica_offset, 0);
        assert_eq!(stats.total_replicated, 0);
        assert_eq!(stats.total_bytes, 0);
        assert!(!stats.connected);
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
