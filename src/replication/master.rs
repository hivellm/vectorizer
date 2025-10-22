//! Master Node - Accepts writes and replicates to replica nodes
//!
//! Features:
//! - Maintains replication log
//! - Sends operations to all connected replicas
//! - Monitors replica lag
//! - Handles full and partial sync
//! - Heartbeat mechanism

use super::config::ReplicationConfig;
use super::replication_log::ReplicationLog;
use super::types::{
    ReplicaInfo, ReplicationCommand, ReplicationError, ReplicationOperation, ReplicationResult,
    ReplicationStats, VectorOperation,
};
use crate::db::VectorStore;
use parking_lot::RwLock;
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::mpsc;
use tracing::{debug, error, info, warn};
use uuid::Uuid;

/// Master Node - Accepts writes and replicates to replica nodes
pub struct MasterNode {
    config: ReplicationConfig,
    replication_log: Arc<ReplicationLog>,
    vector_store: Arc<VectorStore>,

    /// Connected replicas
    replicas: Arc<RwLock<HashMap<String, ReplicaConnection>>>,

    /// Channel to send operations to replication task
    replication_tx: mpsc::UnboundedSender<ReplicationMessage>,
}

struct ReplicaConnection {
    id: String,
    address: SocketAddr,
    offset: u64,
    connected_at: u64,
    last_heartbeat: u64,
    sender: mpsc::UnboundedSender<ReplicationCommand>,
}

enum ReplicationMessage {
    Operation(VectorOperation),
    Heartbeat,
}

impl MasterNode {
    /// Create a new master node
    pub fn new(
        config: ReplicationConfig,
        vector_store: Arc<VectorStore>,
    ) -> ReplicationResult<Self> {
        let replication_log = Arc::new(ReplicationLog::new(config.log_size));
        let (replication_tx, replication_rx) = mpsc::unbounded_channel();

        let replicas = Arc::new(RwLock::new(HashMap::new()));

        let node = Self {
            config,
            replication_log,
            vector_store,
            replicas,
            replication_tx,
        };

        // Start replication task
        node.start_replication_task(replication_rx);

        Ok(node)
    }

    /// Start listening for replica connections
    pub async fn start(&self) -> ReplicationResult<()> {
        let bind_addr = self
            .config
            .bind_address
            .ok_or_else(|| ReplicationError::Connection("No bind address configured".to_string()))?;

        info!("Master node starting on {}", bind_addr);

        let listener = TcpListener::bind(bind_addr).await?;

        let replicas = Arc::clone(&self.replicas);
        let replication_log = Arc::clone(&self.replication_log);
        let vector_store = Arc::clone(&self.vector_store);

        tokio::spawn(async move {
            loop {
                match listener.accept().await {
                    Ok((stream, addr)) => {
                        info!("New replica connection from {}", addr);

                        let replicas = Arc::clone(&replicas);
                        let replication_log = Arc::clone(&replication_log);
                        let vector_store = Arc::clone(&vector_store);

                        tokio::spawn(async move {
                            if let Err(e) = Self::handle_replica(
                                stream,
                                addr,
                                replicas,
                                replication_log,
                                vector_store,
                            )
                            .await
                            {
                                error!("Replica handler error: {}", e);
                            }
                        });
                    }
                    Err(e) => {
                        error!("Accept error: {}", e);
                    }
                }
            }
        });

        // Start heartbeat task
        self.start_heartbeat_task();

        Ok(())
    }

    /// Handle a replica connection
    async fn handle_replica(
        mut stream: TcpStream,
        addr: SocketAddr,
        replicas: Arc<RwLock<HashMap<String, ReplicaConnection>>>,
        replication_log: Arc<ReplicationLog>,
        vector_store: Arc<VectorStore>,
    ) -> ReplicationResult<()> {
        let replica_id = Uuid::new_v4().to_string();

        // Read replica's request (contains last known offset)
        let mut len_buf = [0u8; 4];
        stream.read_exact(&mut len_buf).await?;
        let len = u32::from_be_bytes(len_buf) as usize;

        let mut data_buf = vec![0u8; len];
        stream.read_exact(&mut data_buf).await?;

        let replica_offset: u64 = bincode::deserialize(&data_buf)?;

        info!(
            "Replica {} connected from {} with offset {}",
            replica_id, addr, replica_offset
        );

        // Create channel for this replica
        let (tx, mut rx) = mpsc::unbounded_channel();

        // Register replica
        {
            let mut replicas = replicas.write();
            replicas.insert(
                replica_id.clone(),
                ReplicaConnection {
                    id: replica_id.clone(),
                    address: addr,
                    offset: replica_offset,
                    connected_at: current_timestamp(),
                    last_heartbeat: current_timestamp(),
                    sender: tx,
                },
            );
        }

        // Determine sync strategy
        let current_offset = replication_log.current_offset();
        let need_full_sync = if replica_offset == 0 {
            true
        } else {
            replication_log
                .get_operations(replica_offset)
                .is_none()
        };

        if need_full_sync {
            info!("Performing full sync for replica {}", replica_id);

            // Create and send snapshot
            let snapshot = super::sync::create_snapshot(&vector_store, current_offset)
                .await
                .map_err(|e| ReplicationError::Sync(e))?;

            let cmd = ReplicationCommand::FullSync {
                snapshot_data: snapshot,
                offset: current_offset,
            };

            Self::send_command(&mut stream, &cmd).await?;

            // Update replica offset
            {
                let mut replicas = replicas.write();
                if let Some(replica) = replicas.get_mut(&replica_id) {
                    replica.offset = current_offset;
                }
            }
        } else {
            info!("Performing partial sync for replica {}", replica_id);

            // Get operations since replica's offset
            if let Some(operations) = replication_log.get_operations(replica_offset) {
                let cmd = ReplicationCommand::PartialSync {
                    from_offset: replica_offset,
                    operations,
                };

                Self::send_command(&mut stream, &cmd).await?;

                // Update replica offset
                {
                    let mut replicas = replicas.write();
                    if let Some(replica) = replicas.get_mut(&replica_id) {
                        replica.offset = current_offset;
                    }
                }
            }
        }

        // Start sending operations to replica
        loop {
            tokio::select! {
                Some(cmd) = rx.recv() => {
                    if let Err(e) = Self::send_command(&mut stream, &cmd).await {
                        error!("Failed to send to replica {}: {}", replica_id, e);
                        break;
                    }
                }
            }
        }

        // Cleanup on disconnect
        replicas.write().remove(&replica_id);
        info!("Replica {} disconnected", replica_id);

        Ok(())
    }

    /// Send a command to replica
    async fn send_command(
        stream: &mut TcpStream,
        cmd: &ReplicationCommand,
    ) -> ReplicationResult<()> {
        let data = bincode::serialize(cmd)?;
        let len = (data.len() as u32).to_be_bytes();

        stream.write_all(&len).await?;
        stream.write_all(&data).await?;
        stream.flush().await?;

        Ok(())
    }

    /// Replicate an operation to all replicas
    pub fn replicate(&self, operation: VectorOperation) -> u64 {
        let offset = self.replication_log.append(operation.clone());

        // Send to replication task
        let _ = self.replication_tx.send(ReplicationMessage::Operation(operation));

        offset
    }

    /// Start replication task (sends operations to replicas)
    fn start_replication_task(&self, mut rx: mpsc::UnboundedReceiver<ReplicationMessage>) {
        let replicas = Arc::clone(&self.replicas);
        let replication_log = Arc::clone(&self.replication_log);

        tokio::spawn(async move {
            while let Some(msg) = rx.recv().await {
                match msg {
                    ReplicationMessage::Operation(operation) => {
                        let offset = replication_log.current_offset();
                        let timestamp = current_timestamp();

                        let repl_op = ReplicationOperation {
                            offset,
                            timestamp,
                            operation,
                        };

                        let cmd = ReplicationCommand::Operation(repl_op);

                        // Send to all replicas
                        let replicas = replicas.read();
                        for (id, replica) in replicas.iter() {
                            if let Err(e) = replica.sender.send(cmd.clone()) {
                                warn!("Failed to queue operation for replica {}: {}", id, e);
                            }
                        }
                    }
                    ReplicationMessage::Heartbeat => {
                        let offset = replication_log.current_offset();
                        let timestamp = current_timestamp();

                        let cmd = ReplicationCommand::Heartbeat {
                            master_offset: offset,
                            timestamp,
                        };

                        let replicas = replicas.read();
                        for (id, replica) in replicas.iter() {
                            if let Err(e) = replica.sender.send(cmd.clone()) {
                                warn!("Failed to send heartbeat to replica {}: {}", id, e);
                            }
                        }
                    }
                }
            }
        });
    }

    /// Start heartbeat task
    fn start_heartbeat_task(&self) {
        let interval = self.config.heartbeat_duration();
        let tx = self.replication_tx.clone();

        tokio::spawn(async move {
            let mut interval = tokio::time::interval(interval);
            loop {
                interval.tick().await;
                let _ = tx.send(ReplicationMessage::Heartbeat);
            }
        });
    }

    /// Get replication statistics
    pub fn get_stats(&self) -> ReplicationStats {
        let replicas = self.replicas.read();
        let master_offset = self.replication_log.current_offset();

        // Calculate aggregate stats
        let mut total_replicated = 0;
        let mut total_lag = 0;

        for replica in replicas.values() {
            total_replicated += replica.offset;
            total_lag += master_offset.saturating_sub(replica.offset);
        }

        ReplicationStats {
            master_offset,
            replica_offset: 0, // Not applicable for master
            lag_operations: total_lag,
            lag_ms: 0,
            total_replicated,
            total_bytes: 0,
            last_heartbeat: current_timestamp(),
            connected: !replicas.is_empty(),
        }
    }

    /// Get information about all replicas
    pub fn get_replicas(&self) -> Vec<ReplicaInfo> {
        let replicas = self.replicas.read();
        let master_offset = self.replication_log.current_offset();

        replicas
            .values()
            .map(|r| {
                let lag_ops = master_offset.saturating_sub(r.offset);
                let lag_ms = current_timestamp().saturating_sub(r.last_heartbeat);

                ReplicaInfo {
                    id: r.id.clone(),
                    address: r.address,
                    offset: r.offset,
                    lag_operations: lag_ops,
                    lag_ms,
                    connected: true,
                    last_heartbeat: r.last_heartbeat,
                }
            })
            .collect()
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
    use super::*;
    use crate::db::VectorStore;
    use crate::replication::{NodeRole, ReplicationConfig, VectorOperation, CollectionConfigData};
    use std::sync::Arc;

    #[tokio::test]
    async fn test_master_creation_and_initial_state() {
        let store = Arc::new(VectorStore::new());
        let config = ReplicationConfig {
            role: NodeRole::Master,
            bind_address: Some("127.0.0.1:0".parse().unwrap()),
            master_address: None,
            heartbeat_interval: 5,
            replica_timeout: 30,
            log_size: 1000,
            reconnect_interval: 5,
        };

        let result = MasterNode::new(config, store);
        assert!(result.is_ok());
        
        let master = result.unwrap();
        
        // Test initial stats
        let stats = master.get_stats();
        assert_eq!(stats.master_offset, 0);
        assert!(!stats.connected);
        
        // Test initial replicas
        let replicas = master.get_replicas();
        assert_eq!(replicas.len(), 0);
    }

    #[tokio::test]
    async fn test_master_replicate_increments_offset() {
        let store = Arc::new(VectorStore::new());
        let config = ReplicationConfig {
            role: NodeRole::Master,
            bind_address: Some("127.0.0.1:0".parse().unwrap()),
            master_address: None,
            heartbeat_interval: 5,
            replica_timeout: 30,
            log_size: 1000,
            reconnect_interval: 5,
        };

        let master = MasterNode::new(config, store).unwrap();

        // Replicate operations
        for i in 0..10 {
            let op = VectorOperation::InsertVector {
                collection: "test".to_string(),
                id: format!("vec_{}", i),
                vector: vec![i as f32; 64],
                payload: None,
            };
            let offset = master.replicate(op);
            assert_eq!(offset, i + 1);
        }

        // Check final stats
        let stats = master.get_stats();
        assert_eq!(stats.master_offset, 10);
    }
}


