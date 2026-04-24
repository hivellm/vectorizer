//! Master Node - Accepts writes and replicates to replica nodes
//!
//! Features:
//! - Maintains replication log
//! - Sends operations to all connected replicas
//! - Monitors replica lag
//! - Handles full and partial sync
//! - Heartbeat mechanism

use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use parking_lot::RwLock;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::{Notify, mpsc};
use tracing::{debug, error, info, warn};
use uuid::Uuid;

use super::config::ReplicationConfig;
use super::durable_log::DurableReplicationLog;
use super::types::{
    ReplicaInfo, ReplicationCommand, ReplicationError, ReplicationOperation, ReplicationResult,
    ReplicationStats, VectorOperation, WriteConcern,
};
use crate::db::VectorStore;

/// Master Node - Accepts writes and replicates to replica nodes
pub struct MasterNode {
    config: ReplicationConfig,
    replication_log: Arc<DurableReplicationLog>,
    vector_store: Arc<VectorStore>,

    /// Connected replicas
    replicas: Arc<RwLock<HashMap<String, ReplicaConnection>>>,

    /// Channel to send operations to replication task
    replication_tx: mpsc::UnboundedSender<ReplicationMessage>,

    /// Per-replica confirmed offsets, updated when ACKs arrive
    confirmed_offsets: Arc<RwLock<HashMap<String, u64>>>,

    /// Notified whenever any ACK arrives so `wait_for_replicas` can wake up
    ack_notify: Arc<Notify>,
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
        let wal_path = if config.wal_enabled {
            let dir = config.wal_dir.as_deref().unwrap_or("data/replication-wal");
            Some(std::path::PathBuf::from(dir).join("replication.wal"))
        } else {
            None
        };

        let replication_log = Arc::new(DurableReplicationLog::new(config.log_size, wal_path)?);
        let (replication_tx, replication_rx) = mpsc::unbounded_channel();

        let replicas = Arc::new(RwLock::new(HashMap::new()));
        let confirmed_offsets = Arc::new(RwLock::new(HashMap::new()));
        let ack_notify = Arc::new(Notify::new());

        let node = Self {
            config,
            replication_log,
            vector_store,
            replicas,
            replication_tx,
            confirmed_offsets,
            ack_notify,
        };

        // Start replication task
        node.start_replication_task(replication_rx);

        Ok(node)
    }

    /// Start listening for replica connections
    pub async fn start(&self) -> ReplicationResult<()> {
        let bind_addr = self.config.bind_address.ok_or_else(|| {
            ReplicationError::Connection("No bind address configured".to_string())
        })?;

        info!("Master node starting on {}", bind_addr);

        // Use SO_REUSEADDR so that rapid Leader→Follower→Leader transitions
        // (common during Raft elections) don't fail with "Address already in use"
        // when the previous MasterNode's socket is still in TIME_WAIT.
        let socket = tokio::net::TcpSocket::new_v4().map_err(|e| ReplicationError::Io(e))?;
        socket
            .set_reuseaddr(true)
            .map_err(|e| ReplicationError::Io(e))?;
        socket
            .bind(bind_addr)
            .map_err(|e| ReplicationError::Io(e))?;
        let listener = socket.listen(128).map_err(|e| ReplicationError::Io(e))?;

        let replicas = Arc::clone(&self.replicas);
        let replication_log = Arc::clone(&self.replication_log);
        let vector_store = Arc::clone(&self.vector_store);
        let confirmed_offsets = Arc::clone(&self.confirmed_offsets);
        let ack_notify = Arc::clone(&self.ack_notify);

        tokio::spawn(async move {
            loop {
                match listener.accept().await {
                    Ok((stream, addr)) => {
                        info!("New replica connection from {}", addr);

                        let replicas = Arc::clone(&replicas);
                        let replication_log = Arc::clone(&replication_log);
                        let vector_store = Arc::clone(&vector_store);
                        let confirmed_offsets = Arc::clone(&confirmed_offsets);
                        let ack_notify = Arc::clone(&ack_notify);

                        tokio::spawn(async move {
                            if let Err(e) = Self::handle_replica(
                                stream,
                                addr,
                                replicas,
                                replication_log,
                                vector_store,
                                confirmed_offsets,
                                ack_notify,
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

    /// Handle a replica connection.
    ///
    /// The TcpStream is split into read and write halves so that ACKs arriving
    /// from the replica can be processed concurrently with commands sent to it.
    /// ACKs update `confirmed_offsets` and wake any caller blocked in
    /// `wait_for_replicas` via `ack_notify`.
    async fn handle_replica(
        mut stream: TcpStream,
        addr: SocketAddr,
        replicas: Arc<RwLock<HashMap<String, ReplicaConnection>>>,
        replication_log: Arc<DurableReplicationLog>,
        vector_store: Arc<VectorStore>,
        confirmed_offsets: Arc<RwLock<HashMap<String, u64>>>,
        ack_notify: Arc<Notify>,
    ) -> ReplicationResult<()> {
        let replica_id = Uuid::new_v4().to_string();

        // Read replica's request (contains last known offset)
        let mut len_buf = [0u8; 4];
        stream.read_exact(&mut len_buf).await?;
        let len = u32::from_be_bytes(len_buf) as usize;

        let mut data_buf = vec![0u8; len];
        stream.read_exact(&mut data_buf).await?;

        let replica_offset: u64 = crate::codec::deserialize(&data_buf)?;

        info!(
            "Replica {} connected from {} with offset {}",
            replica_id, addr, replica_offset
        );

        // Create channel for this replica
        let (tx, mut rx) = mpsc::unbounded_channel();

        // Register replica and initialise its confirmed offset
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
        {
            confirmed_offsets
                .write()
                .insert(replica_id.clone(), replica_offset);
        }

        // Determine sync strategy
        let current_offset = replication_log.current_offset();
        let need_full_sync = if replica_offset == 0 {
            true
        } else {
            replication_log.get_operations(replica_offset).is_none()
        };

        if need_full_sync {
            info!("Performing full sync for replica {}", replica_id);

            let snapshot = super::sync::create_snapshot(&vector_store, current_offset)
                .await
                .map_err(|e| ReplicationError::Sync(e))?;

            let cmd = ReplicationCommand::FullSync {
                snapshot_data: snapshot,
                offset: current_offset,
            };

            Self::send_command(&mut stream, &cmd).await?;

            {
                let mut replicas = replicas.write();
                if let Some(replica) = replicas.get_mut(&replica_id) {
                    replica.offset = current_offset;
                }
            }
        } else {
            info!("Performing partial sync for replica {}", replica_id);

            if let Some(operations) = replication_log.get_operations(replica_offset) {
                let cmd = ReplicationCommand::PartialSync {
                    from_offset: replica_offset,
                    operations,
                };

                Self::send_command(&mut stream, &cmd).await?;

                {
                    let mut replicas = replicas.write();
                    if let Some(replica) = replicas.get_mut(&replica_id) {
                        replica.offset = current_offset;
                    }
                }
            }
        }

        // Split the stream so we can send commands and receive ACKs concurrently.
        let (mut read_half, mut write_half) = stream.into_split();

        // Spawn a task to read ACKs from the replica on the read half.
        let ack_replica_id = replica_id.clone();
        let ack_confirmed_offsets = Arc::clone(&confirmed_offsets);
        let ack_notify_clone = Arc::clone(&ack_notify);

        tokio::spawn(async move {
            let mut len_buf = [0u8; 4];
            loop {
                match read_half.read_exact(&mut len_buf).await {
                    Ok(_) => {}
                    Err(e) => {
                        debug!("ACK reader for replica {} closed: {}", ack_replica_id, e);
                        break;
                    }
                }

                let len = u32::from_be_bytes(len_buf) as usize;
                let mut data_buf = vec![0u8; len];
                if let Err(e) = read_half.read_exact(&mut data_buf).await {
                    debug!(
                        "ACK reader for replica {} read error: {}",
                        ack_replica_id, e
                    );
                    break;
                }

                match crate::codec::deserialize::<ReplicationCommand>(&data_buf) {
                    Ok(ReplicationCommand::Ack { replica_id, offset }) => {
                        debug!(
                            "Received ACK from replica {} for offset {}",
                            replica_id, offset
                        );
                        {
                            let mut map = ack_confirmed_offsets.write();
                            let entry = map.entry(replica_id.clone()).or_insert(0);
                            if offset > *entry {
                                *entry = offset;
                            }
                        }
                        // Wake any tasks waiting in wait_for_replicas
                        ack_notify_clone.notify_waiters();
                    }
                    Ok(other) => {
                        warn!(
                            "Unexpected command from replica {}: {:?}",
                            ack_replica_id, other
                        );
                    }
                    Err(e) => {
                        warn!(
                            "Failed to deserialise ACK from replica {}: {}",
                            ack_replica_id, e
                        );
                    }
                }
            }
        });

        // Ops with offset <= `sync_offset` are already covered by the
        // FullSync/PartialSync sent above. The replica is inserted into the
        // `replicas` map before the snapshot is captured (so `replicate()`
        // callers never drop writes targeted at it), which means the fan-out
        // task can enqueue those same ops into our per-replica `rx` in the
        // window between registration and snapshot capture. Drop them here
        // instead of double-applying on the replica — the replica's
        // `apply_operation` treats `InsertVector` as an idempotent upsert by
        // id, but `vector_count` used to diverge under replay before the
        // matching `insert_batch` fix in `collection/data.rs`.
        let sync_offset = current_offset;

        // Send commands to the replica on the write half.
        loop {
            tokio::select! {
                Some(cmd) = rx.recv() => {
                    if let ReplicationCommand::Operation(ref op) = cmd {
                        if op.offset <= sync_offset {
                            debug!(
                                "Replica {}: skipping op at offset {} (<= sync_offset {})",
                                replica_id, op.offset, sync_offset
                            );
                            continue;
                        }
                    }

                    if let Err(e) = Self::send_command_half(&mut write_half, &cmd).await {
                        error!("Failed to send to replica {}: {}", replica_id, e);
                        break;
                    }

                    // Update the tracked send offset after successful delivery
                    if let ReplicationCommand::Operation(ref op) = cmd {
                        let mut replicas = replicas.write();
                        if let Some(replica) = replicas.get_mut(&replica_id) {
                            replica.offset = op.offset;
                        }
                    }
                }
            }
        }

        // Cleanup on disconnect
        replicas.write().remove(&replica_id);
        confirmed_offsets.write().remove(&replica_id);
        info!("Replica {} disconnected", replica_id);

        Ok(())
    }

    /// Send a command to a replica using a full TcpStream (used during initial sync phase).
    async fn send_command(
        stream: &mut TcpStream,
        cmd: &ReplicationCommand,
    ) -> ReplicationResult<()> {
        use crate::monitoring::metrics::METRICS;

        let data = crate::codec::serialize(cmd)?;
        let len = (data.len() as u32).to_be_bytes();

        let total_bytes = 4 + data.len();
        METRICS
            .replication_bytes_sent_total
            .inc_by(total_bytes as f64);

        stream.write_all(&len).await?;
        stream.write_all(&data).await?;
        stream.flush().await?;

        Ok(())
    }

    /// Send a command to a replica using an owned write half (used after stream split).
    async fn send_command_half(
        write_half: &mut tokio::net::tcp::OwnedWriteHalf,
        cmd: &ReplicationCommand,
    ) -> ReplicationResult<()> {
        use crate::monitoring::metrics::METRICS;

        let data = crate::codec::serialize(cmd)?;
        let len = (data.len() as u32).to_be_bytes();

        let total_bytes = 4 + data.len();
        METRICS
            .replication_bytes_sent_total
            .inc_by(total_bytes as f64);

        write_half.write_all(&len).await?;
        write_half.write_all(&data).await?;
        write_half.flush().await?;

        Ok(())
    }

    /// Replicate an operation to all replicas.
    ///
    /// When WAL is enabled the write is fsynced to disk before returning the
    /// offset, ensuring durability across master crashes.
    pub fn replicate(&self, operation: VectorOperation) -> u64 {
        use crate::monitoring::metrics::METRICS;

        let offset = match self.replication_log.append(operation.clone()) {
            Ok(off) => off,
            Err(e) => {
                // WAL write failed — log the error and fall back to the
                // in-memory offset so the caller is not blocked.
                tracing::error!("WAL append failed (durability compromised): {}", e);
                self.replication_log.current_offset()
            }
        };

        // Update operations pending metric
        METRICS.replication_operations_pending.inc();

        // Send to replication task
        let _ = self
            .replication_tx
            .send(ReplicationMessage::Operation(operation));

        offset
    }

    /// Wait until at least `num_replicas` have confirmed `target_offset`.
    ///
    /// Returns the number of replicas whose confirmed offset is >= `target_offset`
    /// at the time the function returns (either enough confirmed, or timeout).
    pub async fn wait_for_replicas(
        &self,
        target_offset: u64,
        num_replicas: usize,
        timeout: Duration,
    ) -> usize {
        let deadline = tokio::time::Instant::now() + timeout;

        loop {
            // Count replicas that have confirmed at least target_offset
            let confirmed_count = {
                let map = self.confirmed_offsets.read();
                map.values()
                    .filter(|&&offset| offset >= target_offset)
                    .count()
            };

            if confirmed_count >= num_replicas {
                return confirmed_count;
            }

            // Wait for the next ACK or for the deadline to expire
            let notified = self.ack_notify.notified();
            tokio::select! {
                _ = notified => {
                    // An ACK arrived; loop back and recount
                }
                _ = tokio::time::sleep_until(deadline) => {
                    // Timeout: return however many confirmed so far
                    let map = self.confirmed_offsets.read();
                    return map
                        .values()
                        .filter(|&&offset| offset >= target_offset)
                        .count();
                }
            }
        }
    }

    /// Replicate an operation and optionally wait for replica acknowledgements.
    ///
    /// The `concern` parameter controls how many replicas must confirm before
    /// the method returns successfully.  Use `WriteConcern::None` (the default)
    /// for the original fire-and-forget behaviour.
    pub async fn replicate_with_concern(
        &self,
        operation: VectorOperation,
        concern: WriteConcern,
        timeout: Duration,
    ) -> ReplicationResult<u64> {
        let offset = self.replicate(operation);

        match concern {
            WriteConcern::None => Ok(offset),
            WriteConcern::Count(n) => {
                let confirmed = self.wait_for_replicas(offset, n, timeout).await;
                if confirmed >= n {
                    Ok(offset)
                } else {
                    Err(ReplicationError::WriteConcernTimeout {
                        required: n,
                        confirmed,
                        offset,
                    })
                }
            }
            WriteConcern::All => {
                let total = self.replicas.read().len();
                let confirmed = self.wait_for_replicas(offset, total, timeout).await;
                if confirmed >= total {
                    Ok(offset)
                } else {
                    Err(ReplicationError::WriteConcernTimeout {
                        required: total,
                        confirmed,
                        offset,
                    })
                }
            }
        }
    }

    /// Recover the replication log from the WAL on master startup.
    ///
    /// Call this once **before** `start()` to pre-load the in-memory ring
    /// buffer with any operations that were written to the WAL but not yet
    /// confirmed by all replicas at the time of the last crash.
    ///
    /// Taking `&mut self` here is intentional: it provides a compile-time
    /// guarantee that no concurrent readers or writers exist during recovery,
    /// which matches the intended usage (call during startup, before spawning
    /// any tasks).
    pub fn recover_from_wal(&mut self) -> ReplicationResult<u64> {
        // Arc::get_mut succeeds only when this is the sole strong reference,
        // i.e. before any tasks have cloned the Arc.
        match Arc::get_mut(&mut self.replication_log) {
            Some(log) => {
                let last_offset = log.recover()?;
                info!("WAL recovery complete: last_offset={}", last_offset);
                Ok(last_offset)
            }
            None => {
                // Concurrent references exist — recovery cannot proceed safely.
                // This should not happen in practice (caller violated the
                // startup contract).
                warn!("recover_from_wal called with shared Arc references; skipping WAL replay");
                Ok(self.replication_log.current_offset())
            }
        }
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
        use crate::monitoring::metrics::METRICS;

        let replicas = self.replicas.read();
        let master_offset = self.replication_log.current_offset();

        // Calculate aggregate stats
        let mut total_replicated = 0;
        let mut total_lag = 0;

        for replica in replicas.values() {
            total_replicated += replica.offset;
            total_lag += master_offset.saturating_sub(replica.offset);
        }

        let num_replicas = replicas.len();

        // Update replication lag metric (average lag in operations, convert to approximate ms)
        // Assuming ~1ms per operation as rough estimate
        let avg_lag_ops = if num_replicas > 0 {
            total_lag / num_replicas as u64
        } else {
            0
        };
        METRICS.replication_lag_ms.set(avg_lag_ops as f64);

        ReplicationStats {
            role: crate::replication::NodeRole::Master,
            lag_ms: 0,                          // Master doesn't lag
            bytes_sent: total_replicated * 100, // Approximate
            bytes_received: 0,                  // Master doesn't receive
            last_sync: SystemTime::now(),
            operations_pending: 0, // Master doesn't have pending ops
            snapshot_size: 0,      // Would need to track
            connected_replicas: Some(num_replicas),
            // Legacy fields
            master_offset,
            replica_offset: 0, // Not applicable for master
            lag_operations: total_lag,
            total_replicated,
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
                    host: r.address.ip().to_string(),
                    port: r.address.port(),
                    status: crate::replication::ReplicaStatus::Connected,
                    lag_ms,
                    last_heartbeat: UNIX_EPOCH + std::time::Duration::from_millis(r.last_heartbeat),
                    operations_synced: r.offset,
                    address: Some(r.address),
                    offset: r.offset,
                    lag_operations: lag_ops,
                }
            })
            .collect()
    }
}

fn current_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use std::sync::Arc;

    use super::*;
    use crate::db::VectorStore;
    use crate::replication::{CollectionConfigData, NodeRole, ReplicationConfig, VectorOperation};

    fn test_config(log_size: usize) -> ReplicationConfig {
        ReplicationConfig {
            role: NodeRole::Master,
            bind_address: Some("127.0.0.1:0".parse().unwrap()),
            master_address: None,
            master_address_raw: None,
            heartbeat_interval: 5,
            replica_timeout: 30,
            log_size,
            reconnect_interval: 5,
            // Disable WAL in unit tests to avoid touching the filesystem
            wal_enabled: false,
            wal_dir: None,
        }
    }

    #[tokio::test]
    async fn test_master_creation_and_initial_state() {
        let store = Arc::new(VectorStore::new());
        let result = MasterNode::new(test_config(1000), store);
        assert!(result.is_ok());

        let master = result.unwrap();

        // Test initial stats
        let stats = master.get_stats();
        assert_eq!(stats.master_offset, 0);
        assert_eq!(stats.connected_replicas, Some(0)); // No replicas connected

        // Test initial replicas
        let replicas = master.get_replicas();
        assert_eq!(replicas.len(), 0);
    }

    #[tokio::test]
    async fn test_master_replicate_increments_offset() {
        let store = Arc::new(VectorStore::new());
        let master = MasterNode::new(test_config(1000), store).unwrap();

        // Replicate operations
        for i in 0..10 {
            let op = VectorOperation::InsertVector {
                collection: "test".to_string(),
                id: format!("vec_{}", i),
                vector: vec![i as f32; 64],
                payload: None,
                owner_id: None, // No tenant in test
            };
            let offset = master.replicate(op);
            assert_eq!(offset, i + 1);
        }

        // Check final stats
        let stats = master.get_stats();
        assert_eq!(stats.master_offset, 10);
    }

    #[tokio::test]
    async fn test_wait_for_replicas_no_replicas_returns_zero() {
        let store = Arc::new(VectorStore::new());
        let master = MasterNode::new(test_config(1000), store).unwrap();

        // With no replicas connected, wait_for_replicas should time out immediately
        // and return 0.
        let confirmed = master
            .wait_for_replicas(1, 1, Duration::from_millis(50))
            .await;
        assert_eq!(confirmed, 0);
    }

    #[tokio::test]
    async fn test_replicate_with_concern_none_succeeds() {
        let store = Arc::new(VectorStore::new());
        let master = MasterNode::new(test_config(1000), store).unwrap();

        let op = VectorOperation::InsertVector {
            collection: "test".to_string(),
            id: "v1".to_string(),
            vector: vec![1.0; 4],
            payload: None,
            owner_id: None,
        };

        let result = master
            .replicate_with_concern(op, WriteConcern::None, Duration::from_millis(100))
            .await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 1);
    }

    #[tokio::test]
    async fn test_replicate_with_concern_count_times_out_no_replicas() {
        let store = Arc::new(VectorStore::new());
        let master = MasterNode::new(test_config(1000), store).unwrap();

        let op = VectorOperation::InsertVector {
            collection: "test".to_string(),
            id: "v1".to_string(),
            vector: vec![1.0; 4],
            payload: None,
            owner_id: None,
        };

        let result = master
            .replicate_with_concern(op, WriteConcern::Count(1), Duration::from_millis(50))
            .await;
        assert!(matches!(
            result,
            Err(ReplicationError::WriteConcernTimeout {
                required: 1,
                confirmed: 0,
                offset: 1
            })
        ));
    }
}
