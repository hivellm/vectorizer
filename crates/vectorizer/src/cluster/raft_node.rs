//! Raft consensus layer for Vectorizer cluster coordination.
//!
//! Uses `openraft` for leader election and metadata consensus.
//! Vector data replication uses separate TCP streaming (hybrid approach).

// Internal data-layout file: public fields are self-documenting; the
// blanket allow keeps `cargo doc -W missing-docs` clean without padding
// every field with a tautological `///` comment. See
// phase4_enforce-public-api-docs.
#![allow(missing_docs)]

use std::collections::BTreeMap;
use std::fmt::Debug;
use std::io;
use std::io::Cursor;
use std::ops::RangeBounds;
use std::sync::Arc;

use futures::Stream;
use openraft::alias::{
    EntryOf, LogIdOf, SnapshotDataOf, SnapshotMetaOf, SnapshotOf, StoredMembershipOf,
};
use openraft::entry::RaftEntry;
use openraft::raft::StreamAppendResult;
use openraft::storage::{
    EntryResponder, IOFlushed, LogState, RaftLogReader, RaftLogStorage, RaftSnapshotBuilder,
    RaftStateMachine,
};
use openraft::{Config, EntryPayload, OptionalSend, Vote};
use parking_lot;
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

// ---------------------------------------------------------------------------
// Type configuration
// ---------------------------------------------------------------------------

/// Choose the default LeaderId implementation (advanced mode: allows multiple leaders per term)
mod leader_id_mode {
    pub use openraft::impls::leader_id_adv::LeaderId;
}

openraft::declare_raft_types!(
    /// Raft type configuration for Vectorizer cluster consensus.
    pub TypeConfig:
        D = ClusterCommand,
        R = ClusterResponse,
        Node = RaftNodeInfo,
        LeaderId = leader_id_mode::LeaderId<Self::Term, Self::NodeId>,
);

/// Snapshot payload type for cluster consensus: the whole state machine
/// serialized as JSON, carried in an in-memory cursor.
///
/// openraft 0.10.0-alpha.29 moved `SnapshotData` off `RaftTypeConfig` and onto
/// the components that produce and consume snapshot bytes, so the state
/// machine, its snapshot builder and the network all name this type
/// independently. Keeping one alias here means they cannot drift apart —
/// `RaftStateMachine::SnapshotBuilder` is bound to
/// `RaftSnapshotBuilder<C, SnapshotData = Self::SnapshotData>`, so a mismatch
/// would be a compile error anyway.
pub type ClusterSnapshotData = Cursor<Vec<u8>>;

// ---------------------------------------------------------------------------
// Application data types
// ---------------------------------------------------------------------------

/// Commands that go through Raft consensus (metadata operations only).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ClusterCommand {
    /// Record which node is the current leader.
    SetLeader { node_id: u64 },
    /// Create a collection across the cluster.
    CreateCollection {
        name: String,
        dimension: usize,
        metric: String,
    },
    /// Delete a collection across the cluster.
    DeleteCollection { name: String },
    /// Assign a shard to a node with an epoch for conflict resolution.
    AssignShard {
        shard_id: u32,
        node_id: u64,
        epoch: u64,
    },
    /// Register a new node in the cluster.
    AddNode {
        node_id: u64,
        address: String,
        grpc_port: u16,
    },
    /// Remove a node from the cluster.
    RemoveNode { node_id: u64 },
}

impl std::fmt::Display for ClusterCommand {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::SetLeader { node_id } => write!(f, "SetLeader({})", node_id),
            Self::CreateCollection { name, .. } => write!(f, "CreateCollection({})", name),
            Self::DeleteCollection { name } => write!(f, "DeleteCollection({})", name),
            Self::AssignShard {
                shard_id, node_id, ..
            } => write!(f, "AssignShard({} → {})", shard_id, node_id),
            Self::AddNode { node_id, .. } => write!(f, "AddNode({})", node_id),
            Self::RemoveNode { node_id } => write!(f, "RemoveNode({})", node_id),
        }
    }
}

/// Response returned after applying a [`ClusterCommand`].
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ClusterResponse {
    pub success: bool,
    pub message: String,
}

/// Node address information stored in Raft membership.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct RaftNodeInfo {
    pub address: String,
    pub grpc_port: u16,
}

impl std::fmt::Display for RaftNodeInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}", self.address, self.grpc_port)
    }
}

// ---------------------------------------------------------------------------
// State machine
// ---------------------------------------------------------------------------

/// Serializable state machine data (for snapshots).
#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct StateMachineData {
    pub last_applied_log: Option<LogIdOf<TypeConfig>>,
    pub last_membership: StoredMembershipOf<TypeConfig>,
    pub leader_id: Option<u64>,
    pub collections: BTreeMap<String, (usize, String)>,
    pub shard_assignments: BTreeMap<u32, (u64, u64)>,
    pub nodes: BTreeMap<u64, (String, u16)>,
}

/// Latest state-machine snapshot. Persisted alongside the log so a restart
/// does not lose state whose log entries were already purged.
#[derive(Debug, Serialize, Deserialize)]
pub struct ClusterSnapshot {
    pub meta: SnapshotMetaOf<TypeConfig>,
    pub data: Vec<u8>,
}

/// The Raft state machine for cluster metadata.
pub struct ClusterStateMachine {
    sm: RwLock<StateMachineData>,
    snapshot_idx: parking_lot::Mutex<u64>,
    current_snapshot: RwLock<Option<ClusterSnapshot>>,
    /// Where the latest snapshot is persisted; `None` keeps it in memory only.
    snapshot_path: Option<std::path::PathBuf>,
}

impl ClusterStateMachine {
    pub fn new() -> Self {
        Self {
            sm: RwLock::new(StateMachineData::default()),
            snapshot_idx: parking_lot::Mutex::new(0),
            current_snapshot: RwLock::new(None),
            snapshot_path: None,
        }
    }

    /// Open a state machine whose snapshots are persisted under `dir`,
    /// restoring the last one if present. Entries logged after it are
    /// re-applied by openraft from the persisted log.
    pub fn open(dir: &std::path::Path) -> io::Result<Self> {
        std::fs::create_dir_all(dir)?;
        let snapshot_path = dir.join(RAFT_SNAPSHOT_FILE);
        let snapshot = match std::fs::read(&snapshot_path) {
            Ok(bytes) => Some(
                serde_json::from_slice::<ClusterSnapshot>(&bytes)
                    .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e.to_string()))?,
            ),
            Err(e) if e.kind() == io::ErrorKind::NotFound => None,
            Err(e) => return Err(e),
        };
        let sm = match &snapshot {
            Some(snap) => serde_json::from_slice::<StateMachineData>(&snap.data)
                .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e.to_string()))?,
            None => StateMachineData::default(),
        };
        Ok(Self {
            sm: RwLock::new(sm),
            snapshot_idx: parking_lot::Mutex::new(0),
            current_snapshot: RwLock::new(snapshot),
            snapshot_path: Some(snapshot_path),
        })
    }

    /// Write `snapshot` to disk. No-op for an in-memory state machine.
    fn persist_snapshot(&self, snapshot: &ClusterSnapshot) -> io::Result<()> {
        let Some(path) = &self.snapshot_path else {
            return Ok(());
        };
        let bytes = serde_json::to_vec(snapshot)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e.to_string()))?;
        write_atomic(path, &bytes)
    }

    /// Read current state (for external queries).
    pub async fn state(&self) -> StateMachineData {
        self.sm.read().await.clone()
    }
}

impl RaftSnapshotBuilder<TypeConfig> for Arc<ClusterStateMachine> {
    type SnapshotData = ClusterSnapshotData;

    async fn build_snapshot(
        &mut self,
    ) -> Result<SnapshotOf<TypeConfig, ClusterSnapshotData>, io::Error> {
        let sm = self.sm.read().await;
        let data = serde_json::to_vec(&*sm)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e.to_string()))?;

        let snapshot_idx = {
            let mut idx = self.snapshot_idx.lock();
            *idx += 1;
            *idx
        };

        let snapshot_id = if let Some(last) = sm.last_applied_log {
            format!(
                "{}-{}-{}",
                last.committed_leader_id(),
                last.index(),
                snapshot_idx
            )
        } else {
            format!("--{}", snapshot_idx)
        };

        let meta = SnapshotMetaOf::<TypeConfig> {
            last_log_id: sm.last_applied_log,
            last_membership: sm.last_membership.clone(),
            snapshot_id,
        };

        let snapshot = ClusterSnapshot {
            meta: meta.clone(),
            data: data.clone(),
        };

        self.persist_snapshot(&snapshot)?;
        *self.current_snapshot.write().await = Some(snapshot);

        info!(snapshot_size = data.len(), "Raft snapshot built");

        Ok(SnapshotOf::<TypeConfig, ClusterSnapshotData> {
            meta,
            snapshot: Cursor::new(data),
        })
    }
}

impl RaftStateMachine<TypeConfig> for Arc<ClusterStateMachine> {
    type SnapshotData = ClusterSnapshotData;
    type SnapshotBuilder = Self;

    async fn applied_state(
        &mut self,
    ) -> Result<(Option<LogIdOf<TypeConfig>>, StoredMembershipOf<TypeConfig>), io::Error> {
        let sm = self.sm.read().await;
        Ok((sm.last_applied_log, sm.last_membership.clone()))
    }

    async fn apply<Strm>(&mut self, mut entries: Strm) -> Result<(), io::Error>
    where
        Strm: Stream<Item = Result<EntryResponder<TypeConfig>, io::Error>> + Unpin + OptionalSend,
    {
        use futures::TryStreamExt;

        let mut sm = self.sm.write().await;

        while let Some((entry, responder)) = entries.try_next().await? {
            debug!(%entry.log_id, "applying cluster command");

            sm.last_applied_log = Some(entry.log_id);

            let response = match entry.payload {
                EntryPayload::Blank => ClusterResponse {
                    success: true,
                    message: "blank".into(),
                },
                EntryPayload::Normal(ref cmd) => match cmd {
                    ClusterCommand::SetLeader { node_id } => {
                        sm.leader_id = Some(*node_id);
                        ClusterResponse {
                            success: true,
                            message: format!("leader set to {}", node_id),
                        }
                    }
                    ClusterCommand::CreateCollection {
                        name,
                        dimension,
                        metric,
                    } => {
                        sm.collections
                            .insert(name.clone(), (*dimension, metric.clone()));
                        info!(
                            "Raft: collection '{}' created (dim={}, metric={})",
                            name, dimension, metric
                        );
                        ClusterResponse {
                            success: true,
                            message: format!("collection '{}' created", name),
                        }
                    }
                    ClusterCommand::DeleteCollection { name } => {
                        sm.collections.remove(name);
                        ClusterResponse {
                            success: true,
                            message: format!("collection '{}' deleted", name),
                        }
                    }
                    ClusterCommand::AssignShard {
                        shard_id,
                        node_id,
                        epoch,
                    } => {
                        sm.shard_assignments.insert(*shard_id, (*node_id, *epoch));
                        ClusterResponse {
                            success: true,
                            message: format!(
                                "shard {} → node {} (epoch {})",
                                shard_id, node_id, epoch
                            ),
                        }
                    }
                    ClusterCommand::AddNode {
                        node_id,
                        address,
                        grpc_port,
                    } => {
                        sm.nodes.insert(*node_id, (address.clone(), *grpc_port));
                        info!("Raft: node {} added ({}:{})", node_id, address, grpc_port);
                        ClusterResponse {
                            success: true,
                            message: format!("node {} added", node_id),
                        }
                    }
                    ClusterCommand::RemoveNode { node_id } => {
                        sm.nodes.remove(node_id);
                        ClusterResponse {
                            success: true,
                            message: format!("node {} removed", node_id),
                        }
                    }
                },
                EntryPayload::Membership(ref mem) => {
                    sm.last_membership =
                        StoredMembershipOf::<TypeConfig>::new(Some(entry.log_id), mem.clone());
                    ClusterResponse {
                        success: true,
                        message: "membership updated".into(),
                    }
                }
            };

            if let Some(responder) = responder {
                responder.send(response);
            }
        }
        Ok(())
    }

    async fn get_snapshot_builder(&mut self) -> Self::SnapshotBuilder {
        self.clone()
    }

    async fn begin_receiving_snapshot(&mut self) -> Result<Self::SnapshotData, io::Error> {
        Ok(Cursor::new(Vec::new()))
    }

    async fn install_snapshot(
        &mut self,
        meta: &SnapshotMetaOf<TypeConfig>,
        snapshot: Self::SnapshotData,
    ) -> Result<(), io::Error> {
        let new_sm: StateMachineData = serde_json::from_slice(snapshot.get_ref())
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e.to_string()))?;

        let snap = ClusterSnapshot {
            meta: meta.clone(),
            data: snapshot.into_inner(),
        };
        self.persist_snapshot(&snap)?;

        *self.sm.write().await = new_sm;
        *self.current_snapshot.write().await = Some(snap);

        info!("Raft snapshot installed");
        Ok(())
    }

    async fn get_current_snapshot(
        &mut self,
    ) -> Result<Option<SnapshotOf<TypeConfig, Self::SnapshotData>>, io::Error> {
        match &*self.current_snapshot.read().await {
            Some(snap) => Ok(Some(SnapshotOf::<TypeConfig, ClusterSnapshotData> {
                meta: snap.meta.clone(),
                snapshot: Cursor::new(snap.data.clone()),
            })),
            None => Ok(None),
        }
    }
}

// ---------------------------------------------------------------------------
// Log storage (based on openraft-memstore, optionally persisted to disk)
// ---------------------------------------------------------------------------

type ClusterVote = Vote<leader_id_mode::LeaderId<u64, u64>>;

/// How long a node resuming persisted Raft state holds its own elections —
/// see [`RaftManager::open_with_rejoin_grace`]. Covers the few seconds a
/// recreated pod's DNS record takes to resolve for its peers.
pub const REJOIN_ELECTION_GRACE: std::time::Duration = std::time::Duration::from_secs(10);

/// File holding the persisted [`ClusterLogStore`] inside the Raft directory.
const RAFT_LOG_FILE: &str = "raft-log.json";
/// File holding the persisted state-machine snapshot inside the Raft directory.
const RAFT_SNAPSHOT_FILE: &str = "raft-snapshot.json";

/// On-disk image of a [`ClusterLogStore`].
#[derive(Serialize, Deserialize, Default)]
struct PersistedLog {
    vote: Option<ClusterVote>,
    last_purged_log_id: Option<LogIdOf<TypeConfig>>,
    log: BTreeMap<u64, String>,
}

/// Replace `path` with `bytes` atomically: write a sibling temp file, fsync
/// it, then rename over the target. The Raft metadata log is a handful of
/// small entries, so this runs synchronously on the storage task.
fn write_atomic(path: &std::path::Path, bytes: &[u8]) -> io::Result<()> {
    use std::io::Write;

    let tmp = path.with_extension("json.tmp");
    {
        let mut file = std::fs::File::create(&tmp)?;
        file.write_all(bytes)?;
        file.sync_all()?;
    }
    std::fs::rename(&tmp, path)?;
    if let Some(dir) = path.parent() {
        // Persist the rename itself; not every platform can fsync a directory.
        if let Ok(dir) = std::fs::File::open(dir) {
            let _ = dir.sync_all();
        }
    }
    Ok(())
}

/// Raft log storage: vote, log entries and purge marker.
///
/// Raft requires these to survive a restart. When they are lost, a follower
/// restarted on its own comes back with an empty log and no membership: it
/// stays a Learner forever, because the leader believes it already holds the
/// log and only heartbeats it. [`ClusterLogStore::open`] persists them; the
/// in-memory [`ClusterLogStore::new`] is for tests and single-process use.
pub struct ClusterLogStore {
    last_purged_log_id: RwLock<Option<LogIdOf<TypeConfig>>>,
    log: RwLock<BTreeMap<u64, String>>,
    vote: RwLock<Option<ClusterVote>>,
    /// Where the log is persisted; `None` keeps it in memory only.
    path: Option<std::path::PathBuf>,
}

impl ClusterLogStore {
    pub fn new() -> Self {
        Self {
            last_purged_log_id: RwLock::new(None),
            log: RwLock::new(BTreeMap::new()),
            vote: RwLock::new(None),
            path: None,
        }
    }

    /// Open (or create) a log store persisted under `dir`.
    pub fn open(dir: &std::path::Path) -> io::Result<Self> {
        std::fs::create_dir_all(dir)?;
        let path = dir.join(RAFT_LOG_FILE);
        let persisted = match std::fs::read(&path) {
            Ok(bytes) => serde_json::from_slice::<PersistedLog>(&bytes)
                .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e.to_string()))?,
            Err(e) if e.kind() == io::ErrorKind::NotFound => PersistedLog::default(),
            Err(e) => return Err(e),
        };
        info!(
            path = %path.display(),
            entries = persisted.log.len(),
            has_vote = persisted.vote.is_some(),
            "Raft log store opened"
        );
        Ok(Self {
            last_purged_log_id: RwLock::new(persisted.last_purged_log_id),
            log: RwLock::new(persisted.log),
            vote: RwLock::new(persisted.vote),
            path: Some(path),
        })
    }

    /// Whether this store holds Raft state from an earlier run.
    async fn holds_state(&self) -> bool {
        self.vote.read().await.is_some() || !self.log.read().await.is_empty()
    }

    /// Write the current state to disk. No-op for an in-memory store.
    ///
    /// openraft drives storage from a single task, so mutations never race
    /// with each other; callers release their write lock before persisting.
    async fn persist(&self) -> io::Result<()> {
        let Some(path) = &self.path else {
            return Ok(());
        };
        let image = PersistedLog {
            vote: *self.vote.read().await,
            last_purged_log_id: *self.last_purged_log_id.read().await,
            log: self.log.read().await.clone(),
        };
        let bytes = serde_json::to_vec(&image)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e.to_string()))?;
        write_atomic(path, &bytes)
    }
}

impl RaftLogReader<TypeConfig> for Arc<ClusterLogStore> {
    async fn try_get_log_entries<RB: RangeBounds<u64> + Clone + Debug + OptionalSend>(
        &mut self,
        range: RB,
    ) -> Result<Vec<EntryOf<TypeConfig>>, io::Error> {
        let log = self.log.read().await;
        let mut entries = Vec::new();
        for (_, serialized) in log.range(range) {
            let ent: EntryOf<TypeConfig> = serde_json::from_str(serialized)
                .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e.to_string()))?;
            entries.push(ent);
        }
        Ok(entries)
    }

    async fn read_vote(&mut self) -> Result<Option<ClusterVote>, io::Error> {
        Ok(*self.vote.read().await)
    }
}

impl RaftLogStorage<TypeConfig> for Arc<ClusterLogStore> {
    type LogReader = Self;

    async fn get_log_state(&mut self) -> Result<LogState<TypeConfig>, io::Error> {
        let log = self.log.read().await;
        let last = match log.iter().next_back() {
            None => None,
            Some((_, s)) => {
                let ent: EntryOf<TypeConfig> = serde_json::from_str(s)
                    .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e.to_string()))?;
                Some(ent.log_id())
            }
        };
        let last_purged = *self.last_purged_log_id.read().await;
        Ok(LogState {
            last_purged_log_id: last_purged,
            last_log_id: last.or(last_purged),
        })
    }

    async fn get_log_reader(&mut self) -> Self::LogReader {
        self.clone()
    }

    async fn save_vote(
        &mut self,
        vote: &Vote<leader_id_mode::LeaderId<u64, u64>>,
    ) -> Result<(), io::Error> {
        *self.vote.write().await = Some(*vote);
        self.persist().await
    }

    async fn append<I>(
        &mut self,
        entries: I,
        callback: IOFlushed<TypeConfig>,
    ) -> Result<(), io::Error>
    where
        I: IntoIterator<Item = EntryOf<TypeConfig>> + OptionalSend,
    {
        {
            let mut log = self.log.write().await;
            for entry in entries {
                let s = serde_json::to_string(&entry)
                    .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e.to_string()))?;
                log.insert(entry.index(), s);
            }
        }
        // Entries count as flushed only once they are on disk.
        let result = self.persist().await;
        let flushed = result
            .as_ref()
            .map(|_| ())
            .map_err(|e| io::Error::new(e.kind(), e.to_string()));
        callback.io_completed(flushed);
        result
    }

    async fn truncate_after(
        &mut self,
        last_log_id: Option<LogIdOf<TypeConfig>>,
    ) -> Result<(), io::Error> {
        let start = match last_log_id {
            Some(id) => id.index() + 1,
            None => 0,
        };
        {
            let mut log = self.log.write().await;
            let keys: Vec<u64> = log.range(start..).map(|(k, _)| *k).collect();
            for k in keys {
                log.remove(&k);
            }
        }
        self.persist().await
    }

    async fn purge(&mut self, log_id: LogIdOf<TypeConfig>) -> Result<(), io::Error> {
        *self.last_purged_log_id.write().await = Some(log_id);
        {
            let mut log = self.log.write().await;
            let keys: Vec<u64> = log.range(..=log_id.index()).map(|(k, _)| *k).collect();
            for k in keys {
                log.remove(&k);
            }
        }
        self.persist().await
    }
}

// ---------------------------------------------------------------------------
// Network (gRPC-backed)
// ---------------------------------------------------------------------------

/// Network factory that creates per-target connections using gRPC.
pub struct ClusterRaftNetwork {
    /// Known node addresses: node_id -> "http://host:grpc_port"
    pub targets: Arc<parking_lot::RwLock<std::collections::BTreeMap<u64, String>>>,
}

impl ClusterRaftNetwork {
    /// Create a new network factory with an empty address table.
    pub fn new() -> Self {
        Self {
            targets: Arc::new(parking_lot::RwLock::new(std::collections::BTreeMap::new())),
        }
    }
}

/// A single gRPC connection to a remote Raft node.
///
/// Creates a fresh tonic Channel for each RPC to handle DNS resolution
/// changes (pods restart with new IPs in Kubernetes). The `connect_timeout`
/// prevents hanging when the peer isn't ready yet.
pub struct ClusterRaftConnection {
    /// Full gRPC endpoint URL, e.g. "http://host:15003".
    target_addr: String,
}

impl openraft::network::RaftNetworkFactory<TypeConfig> for ClusterRaftNetwork {
    type Network = ClusterRaftConnection;

    async fn new_client(&mut self, target: u64, node: &RaftNodeInfo) -> Self::Network {
        let addr = format!("http://{}:{}", node.address, node.grpc_port);
        info!(
            target_node = target,
            target_addr = %addr,
            "Raft: creating gRPC connection to peer"
        );
        ClusterRaftConnection { target_addr: addr }
    }
}

impl openraft::network::v2::RaftNetworkV2<TypeConfig> for ClusterRaftConnection {
    /// Snapshot payload this connection transfers. Must agree with the state
    /// machine's `SnapshotData`, which is where `Raft` enforces
    /// snapshot-type compatibility since alpha.29.
    type SnapshotData = ClusterSnapshotData;

    /// Send a vote request to the remote node via gRPC.
    async fn vote(
        &mut self,
        rpc: openraft::raft::VoteRequest<TypeConfig>,
        _option: openraft::network::RPCOption,
    ) -> Result<openraft::raft::VoteResponse<TypeConfig>, openraft::error::RPCError<TypeConfig>>
    {
        let data = crate::codec::serialize(&rpc).map_err(|e| {
            openraft::error::RPCError::Network(openraft::error::NetworkError::new(&e))
        })?;

        let channel = tonic::transport::Channel::from_shared(self.target_addr.clone())
            .map_err(|e| {
                openraft::error::RPCError::Network(openraft::error::NetworkError::new(&e))
            })?
            .connect_timeout(std::time::Duration::from_secs(3))
            .connect()
            .await
            .map_err(|e| {
                warn!("Raft vote: connect to {} failed: {:?}", self.target_addr, e);
                openraft::error::RPCError::Network(openraft::error::NetworkError::new(&e))
            })?;

        let mut client =
            vectorizer_grpc::grpc_gen::cluster::cluster_service_client::ClusterServiceClient::new(
                channel,
            );

        let response = client
            .raft_vote(tonic::Request::new(
                vectorizer_grpc::grpc_gen::cluster::RaftVoteRequest { data },
            ))
            .await
            .map_err(|e| {
                warn!("Raft vote RPC to {} failed: {:?}", self.target_addr, e);
                openraft::error::RPCError::Network(openraft::error::NetworkError::new(&e))
            })?;

        let resp: openraft::raft::VoteResponse<TypeConfig> =
            crate::codec::deserialize(&response.into_inner().data).map_err(|e| {
                openraft::error::RPCError::Network(openraft::error::NetworkError::new(&e))
            })?;

        Ok(resp)
    }

    /// Send an append-entries request to the remote node via gRPC.
    async fn append_entries(
        &mut self,
        rpc: openraft::raft::AppendEntriesRequest<TypeConfig>,
        _option: openraft::network::RPCOption,
    ) -> Result<
        openraft::raft::AppendEntriesResponse<TypeConfig>,
        openraft::error::RPCError<TypeConfig>,
    > {
        let data = crate::codec::serialize(&rpc).map_err(|e| {
            openraft::error::RPCError::Network(openraft::error::NetworkError::new(&e))
        })?;

        let channel = tonic::transport::Channel::from_shared(self.target_addr.clone())
            .map_err(|e| {
                openraft::error::RPCError::Network(openraft::error::NetworkError::new(&e))
            })?
            .connect_timeout(std::time::Duration::from_secs(3))
            .connect()
            .await
            .map_err(|e| {
                openraft::error::RPCError::Network(openraft::error::NetworkError::new(&e))
            })?;

        let mut client =
            vectorizer_grpc::grpc_gen::cluster::cluster_service_client::ClusterServiceClient::new(
                channel,
            );

        let response = client
            .raft_append_entries(tonic::Request::new(
                vectorizer_grpc::grpc_gen::cluster::RaftAppendEntriesRequest { data },
            ))
            .await
            .map_err(|e| {
                warn!(
                    "Raft append_entries RPC to {} failed: {:?}",
                    self.target_addr, e
                );
                openraft::error::RPCError::Network(openraft::error::NetworkError::new(&e))
            })?;

        let resp: openraft::raft::AppendEntriesResponse<TypeConfig> =
            crate::codec::deserialize(&response.into_inner().data).map_err(|e| {
                openraft::error::RPCError::Network(openraft::error::NetworkError::new(&e))
            })?;

        Ok(resp)
    }

    /// Install a full snapshot on the remote node via gRPC.
    async fn full_snapshot(
        &mut self,
        vote: Vote<leader_id_mode::LeaderId<u64, u64>>,
        snapshot: SnapshotOf<TypeConfig, Self::SnapshotData>,
        _cancel: impl futures::Future<Output = openraft::error::ReplicationClosed>
        + OptionalSend
        + 'static,
        _option: openraft::network::RPCOption,
    ) -> Result<
        openraft::raft::SnapshotResponse<TypeConfig>,
        openraft::error::StreamingError<TypeConfig>,
    > {
        let vote_data = crate::codec::serialize(&vote).map_err(|e| {
            openraft::error::StreamingError::Network(openraft::error::NetworkError::new(&e))
        })?;

        let snapshot_meta = crate::codec::serialize(&snapshot.meta).map_err(|e| {
            openraft::error::StreamingError::Network(openraft::error::NetworkError::new(&e))
        })?;

        // Consume the cursor to get the raw snapshot bytes.
        let snapshot_data = snapshot.snapshot.into_inner();

        let channel = tonic::transport::Channel::from_shared(self.target_addr.clone())
            .map_err(|e| {
                openraft::error::StreamingError::Network(openraft::error::NetworkError::new(&e))
            })?
            .connect_timeout(std::time::Duration::from_secs(5))
            .connect()
            .await
            .map_err(|e| {
                openraft::error::StreamingError::Network(openraft::error::NetworkError::new(&e))
            })?;

        let mut client =
            vectorizer_grpc::grpc_gen::cluster::cluster_service_client::ClusterServiceClient::new(
                channel,
            );

        let response = client
            .raft_snapshot(tonic::Request::new(
                vectorizer_grpc::grpc_gen::cluster::RaftSnapshotRequest {
                    vote_data,
                    snapshot_meta,
                    snapshot_data,
                },
            ))
            .await
            .map_err(|e| {
                openraft::error::StreamingError::Network(openraft::error::NetworkError::new(&e))
            })?;

        let resp: openraft::raft::SnapshotResponse<TypeConfig> =
            crate::codec::deserialize(&response.into_inner().data).map_err(|e| {
                openraft::error::StreamingError::Network(openraft::error::NetworkError::new(&e))
            })?;

        Ok(resp)
    }

    /// Stream append-entries sequentially using the default openraft helper.
    fn stream_append<'s, S>(
        &'s mut self,
        input: S,
        option: openraft::network::RPCOption,
    ) -> futures::future::BoxFuture<
        's,
        Result<
            futures::stream::BoxStream<
                's,
                Result<StreamAppendResult<TypeConfig>, openraft::error::RPCError<TypeConfig>>,
            >,
            openraft::error::RPCError<TypeConfig>,
        >,
    >
    where
        S: Stream<Item = openraft::raft::AppendEntriesRequest<TypeConfig>>
            + OptionalSend
            + Unpin
            + 'static,
    {
        openraft::network::stream_append_sequential(self, input, option)
    }
}

// ---------------------------------------------------------------------------
// Raft manager (public API)
// ---------------------------------------------------------------------------

/// The Raft type alias for Vectorizer.
pub type VectorizerRaft = openraft::Raft<TypeConfig, Arc<ClusterStateMachine>>;

/// Manages the Raft consensus node lifecycle.
pub struct RaftManager {
    pub raft: VectorizerRaft,
    pub state_machine: Arc<ClusterStateMachine>,
    pub log_store: Arc<ClusterLogStore>,
    pub node_id: u64,
}

impl RaftManager {
    /// Create a Raft manager whose log, vote and snapshot live in memory only.
    /// Does NOT start the node — call `initialize()` for bootstrap.
    ///
    /// A restart loses all Raft state; use [`RaftManager::open`] for a node
    /// that has to rejoin its cluster after restarting.
    pub async fn new(node_id: u64) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        Self::with_storage(
            node_id,
            Arc::new(ClusterLogStore::new()),
            Arc::new(ClusterStateMachine::new()),
        )
        .await
    }

    /// Create a Raft manager that persists its log, vote and snapshot under
    /// `dir` and resumes from them after a restart. A node reopened this way
    /// keeps its membership, so it rejoins the cluster as a follower instead
    /// of staying a learner; `initialize_cluster` becomes a no-op.
    pub async fn open(
        node_id: u64,
        dir: &std::path::Path,
    ) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        Self::open_with_rejoin_grace(node_id, dir, REJOIN_ELECTION_GRACE).await
    }

    /// [`RaftManager::open`] with an explicit rejoin grace period.
    ///
    /// A node that resumes from persisted state comes back as a voter. In
    /// Kubernetes its peers cannot resolve the recreated pod's DNS name for
    /// a few seconds, so the leader's heartbeats do not reach it; campaigning
    /// meanwhile makes a healthy leader step down. For `grace` after resuming,
    /// this node does not start elections — it still votes and follows.
    pub async fn open_with_rejoin_grace(
        node_id: u64,
        dir: &std::path::Path,
        grace: std::time::Duration,
    ) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let log_store = Arc::new(ClusterLogStore::open(dir)?);
        let resumed = log_store.holds_state().await;
        let manager = Self::with_storage(
            node_id,
            log_store,
            Arc::new(ClusterStateMachine::open(dir)?),
        )
        .await?;
        if resumed && !grace.is_zero() {
            manager.raft.runtime_config().elect(false);
            info!(
                node_id,
                grace_secs = grace.as_secs(),
                "Resumed persisted Raft state; holding elections while peers re-resolve this node"
            );
            let raft = manager.raft.clone();
            tokio::spawn(async move {
                tokio::time::sleep(grace).await;
                raft.runtime_config().elect(true);
            });
        }
        Ok(manager)
    }

    async fn with_storage(
        node_id: u64,
        log_store: Arc<ClusterLogStore>,
        state_machine: Arc<ClusterStateMachine>,
    ) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let config = Arc::new(
            Config {
                heartbeat_interval: 500,
                election_timeout_min: 1500,
                election_timeout_max: 3000,
                ..Default::default()
            }
            .validate()?,
        );

        let network = ClusterRaftNetwork::new();

        let raft = openraft::Raft::new(
            node_id,
            config,
            network,
            log_store.clone(),
            state_machine.clone(),
        )
        .await?;

        info!(node_id, "Raft node created");

        Ok(Self {
            raft,
            state_machine,
            log_store,
            node_id,
        })
    }

    /// Bootstrap a single-node cluster (for initial leader).
    pub async fn initialize_single(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let mut members = BTreeMap::new();
        members.insert(self.node_id, RaftNodeInfo::default());
        self.raft.initialize(members).await?;
        info!(
            node_id = self.node_id,
            "Raft single-node cluster initialized"
        );
        Ok(())
    }

    /// Bootstrap a multi-node cluster with all members.
    ///
    /// Only the **first node** (lowest node_id) should call this. The other
    /// nodes are included in the initial membership and will participate in
    /// the first election once they can reach each other via gRPC.
    ///
    /// If the Raft state is already initialized (e.g. after a restart),
    /// this is a no-op.
    pub async fn initialize_cluster(
        &self,
        members: BTreeMap<u64, RaftNodeInfo>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        match self.raft.initialize(members.clone()).await {
            Ok(_) => {
                info!(
                    node_id = self.node_id,
                    member_count = members.len(),
                    "Raft multi-node cluster initialized"
                );
                Ok(())
            }
            // NotAllowed means this node already holds Raft state (it was
            // reopened from disk) — the cluster exists, nothing to bootstrap.
            // Matched on the type: the error's Display text does not name it.
            Err(e)
                if matches!(
                    e.api_error(),
                    Some(openraft::error::InitializeError::NotAllowed(_))
                ) =>
            {
                info!(
                    node_id = self.node_id,
                    "Raft already initialized, skipping bootstrap"
                );
                Ok(())
            }
            Err(e) => Err(e.into()),
        }
    }

    /// Propose a command to the Raft cluster. Must be called on the leader.
    pub async fn propose(
        &self,
        cmd: ClusterCommand,
    ) -> Result<ClusterResponse, Box<dyn std::error::Error + Send + Sync>> {
        let resp = self.raft.client_write(cmd).await?;
        Ok(resp.data)
    }

    /// Get current state machine data.
    pub async fn state(&self) -> StateMachineData {
        self.state_machine.state().await
    }

    /// Check if this node believes it is the leader.
    pub async fn is_leader(&self) -> bool {
        self.raft
            .ensure_linearizable(openraft::raft::ReadPolicy::LeaseRead)
            .await
            .is_ok()
    }

    /// Access the underlying Raft instance for advanced operations.
    pub fn raft(&self) -> &VectorizerRaft {
        &self.raft
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_raft_manager_creation() {
        let mgr = RaftManager::new(1).await.unwrap();
        assert_eq!(mgr.node_id, 1);

        let state = mgr.state().await;
        assert!(state.collections.is_empty());
        assert!(state.nodes.is_empty());
        assert!(state.leader_id.is_none());
    }

    #[tokio::test]
    async fn test_state_machine_data_serialization() {
        let data = StateMachineData {
            last_applied_log: None,
            last_membership: StoredMembershipOf::<TypeConfig>::default(),
            leader_id: Some(1),
            collections: BTreeMap::from([("test".into(), (128, "cosine".into()))]),
            shard_assignments: BTreeMap::from([(0, (1, 5))]),
            nodes: BTreeMap::from([(1, ("localhost".into(), 15003))]),
        };

        let json = serde_json::to_string(&data).unwrap();
        let recovered: StateMachineData = serde_json::from_str(&json).unwrap();

        assert_eq!(recovered.leader_id, Some(1));
        assert_eq!(recovered.collections.len(), 1);
        assert_eq!(recovered.nodes.len(), 1);
    }

    async fn wait_for_leader(mgr: &RaftManager) -> bool {
        let deadline = tokio::time::Instant::now() + std::time::Duration::from_secs(15);
        while tokio::time::Instant::now() < deadline {
            if mgr.is_leader().await {
                return true;
            }
            tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        }
        false
    }

    /// A node reopened from its Raft directory must come back with its
    /// membership and applied commands. With the in-memory store, a restarted
    /// follower lost its membership and stayed a learner forever.
    #[tokio::test]
    async fn test_persistent_raft_state_survives_restart() {
        let dir = tempfile::tempdir().unwrap();

        let first = RaftManager::open(7, dir.path()).await.unwrap();
        first.initialize_single().await.unwrap();
        assert!(
            wait_for_leader(&first).await,
            "first run never elected itself"
        );
        first
            .propose(ClusterCommand::AddNode {
                node_id: 7,
                address: "node-7".into(),
                grpc_port: 15003,
            })
            .await
            .unwrap();
        first.raft().shutdown().await.unwrap();
        drop(first);

        let reopened =
            RaftManager::open_with_rejoin_grace(7, dir.path(), std::time::Duration::ZERO)
                .await
                .unwrap();
        // Bootstrapping again must be refused as already-initialized, not
        // start a second cluster.
        reopened
            .initialize_cluster(BTreeMap::from([(7, RaftNodeInfo::default())]))
            .await
            .unwrap();
        assert!(
            wait_for_leader(&reopened).await,
            "reopened node lost its membership"
        );
        let deadline = tokio::time::Instant::now() + std::time::Duration::from_secs(10);
        while !reopened.state().await.nodes.contains_key(&7) {
            assert!(
                tokio::time::Instant::now() < deadline,
                "command committed before the restart was not re-applied"
            );
            tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        }
        reopened.raft().shutdown().await.unwrap();
    }

    #[tokio::test]
    async fn test_in_memory_log_store_does_not_touch_disk() {
        let store = ClusterLogStore::new();
        assert!(store.path.is_none());
        store.persist().await.unwrap();
    }

    #[tokio::test]
    async fn test_cluster_command_display() {
        let cmd = ClusterCommand::CreateCollection {
            name: "test".into(),
            dimension: 128,
            metric: "cosine".into(),
        };
        assert!(format!("{}", cmd).contains("CreateCollection"));
    }
}
