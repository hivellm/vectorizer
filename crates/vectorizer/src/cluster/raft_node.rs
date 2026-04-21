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

/// Snapshot stored in memory.
#[derive(Debug)]
pub struct ClusterSnapshot {
    pub meta: SnapshotMetaOf<TypeConfig>,
    pub data: Vec<u8>,
}

/// The Raft state machine for cluster metadata.
pub struct ClusterStateMachine {
    sm: RwLock<StateMachineData>,
    snapshot_idx: parking_lot::Mutex<u64>,
    current_snapshot: RwLock<Option<ClusterSnapshot>>,
}

impl ClusterStateMachine {
    pub fn new() -> Self {
        Self {
            sm: RwLock::new(StateMachineData::default()),
            snapshot_idx: parking_lot::Mutex::new(0),
            current_snapshot: RwLock::new(None),
        }
    }

    /// Read current state (for external queries).
    pub async fn state(&self) -> StateMachineData {
        self.sm.read().await.clone()
    }
}

impl RaftSnapshotBuilder<TypeConfig> for Arc<ClusterStateMachine> {
    async fn build_snapshot(&mut self) -> Result<SnapshotOf<TypeConfig>, io::Error> {
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

        *self.current_snapshot.write().await = Some(snapshot);

        info!(snapshot_size = data.len(), "Raft snapshot built");

        Ok(SnapshotOf::<TypeConfig> {
            meta,
            snapshot: Cursor::new(data),
        })
    }
}

impl RaftStateMachine<TypeConfig> for Arc<ClusterStateMachine> {
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

    async fn begin_receiving_snapshot(&mut self) -> Result<SnapshotDataOf<TypeConfig>, io::Error> {
        Ok(Cursor::new(Vec::new()))
    }

    async fn install_snapshot(
        &mut self,
        meta: &SnapshotMetaOf<TypeConfig>,
        snapshot: SnapshotDataOf<TypeConfig>,
    ) -> Result<(), io::Error> {
        let new_sm: StateMachineData = serde_json::from_slice(snapshot.get_ref())
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e.to_string()))?;

        *self.sm.write().await = new_sm;

        let snap = ClusterSnapshot {
            meta: meta.clone(),
            data: snapshot.into_inner(),
        };
        *self.current_snapshot.write().await = Some(snap);

        info!("Raft snapshot installed");
        Ok(())
    }

    async fn get_current_snapshot(&mut self) -> Result<Option<SnapshotOf<TypeConfig>>, io::Error> {
        match &*self.current_snapshot.read().await {
            Some(snap) => Ok(Some(SnapshotOf::<TypeConfig> {
                meta: snap.meta.clone(),
                snapshot: Cursor::new(snap.data.clone()),
            })),
            None => Ok(None),
        }
    }
}

// ---------------------------------------------------------------------------
// Log storage (in-memory, based on openraft-memstore)
// ---------------------------------------------------------------------------

/// In-memory Raft log storage.
pub struct ClusterLogStore {
    last_purged_log_id: RwLock<Option<LogIdOf<TypeConfig>>>,
    log: RwLock<BTreeMap<u64, String>>,
    vote: RwLock<Option<Vote<leader_id_mode::LeaderId<u64, u64>>>>,
}

impl ClusterLogStore {
    pub fn new() -> Self {
        Self {
            last_purged_log_id: RwLock::new(None),
            log: RwLock::new(BTreeMap::new()),
            vote: RwLock::new(None),
        }
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

    async fn read_vote(
        &mut self,
    ) -> Result<Option<Vote<leader_id_mode::LeaderId<u64, u64>>>, io::Error> {
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
        Ok(())
    }

    async fn append<I>(
        &mut self,
        entries: I,
        callback: IOFlushed<TypeConfig>,
    ) -> Result<(), io::Error>
    where
        I: IntoIterator<Item = EntryOf<TypeConfig>> + OptionalSend,
    {
        let mut log = self.log.write().await;
        for entry in entries {
            let s = serde_json::to_string(&entry)
                .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e.to_string()))?;
            log.insert(entry.index(), s);
        }
        callback.io_completed(Ok(()));
        Ok(())
    }

    async fn truncate_after(
        &mut self,
        last_log_id: Option<LogIdOf<TypeConfig>>,
    ) -> Result<(), io::Error> {
        let start = match last_log_id {
            Some(id) => id.index() + 1,
            None => 0,
        };
        let mut log = self.log.write().await;
        let keys: Vec<u64> = log.range(start..).map(|(k, _)| *k).collect();
        for k in keys {
            log.remove(&k);
        }
        Ok(())
    }

    async fn purge(&mut self, log_id: LogIdOf<TypeConfig>) -> Result<(), io::Error> {
        *self.last_purged_log_id.write().await = Some(log_id);
        let mut log = self.log.write().await;
        let keys: Vec<u64> = log.range(..=log_id.index()).map(|(k, _)| *k).collect();
        for k in keys {
            log.remove(&k);
        }
        Ok(())
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
            vectorizer_protocol::grpc_gen::cluster::cluster_service_client::ClusterServiceClient::new(channel);

        let response = client
            .raft_vote(tonic::Request::new(
                vectorizer_protocol::grpc_gen::cluster::RaftVoteRequest { data },
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
            vectorizer_protocol::grpc_gen::cluster::cluster_service_client::ClusterServiceClient::new(channel);

        let response = client
            .raft_append_entries(tonic::Request::new(
                vectorizer_protocol::grpc_gen::cluster::RaftAppendEntriesRequest { data },
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
        snapshot: SnapshotOf<TypeConfig>,
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
            vectorizer_protocol::grpc_gen::cluster::cluster_service_client::ClusterServiceClient::new(channel);

        let response = client
            .raft_snapshot(tonic::Request::new(
                vectorizer_protocol::grpc_gen::cluster::RaftSnapshotRequest {
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
    /// Create a new Raft manager. Does NOT start the node — call `initialize()` for bootstrap.
    pub async fn new(node_id: u64) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let config = Arc::new(
            Config {
                heartbeat_interval: 500,
                election_timeout_min: 1500,
                election_timeout_max: 3000,
                ..Default::default()
            }
            .validate()?,
        );

        let log_store = Arc::new(ClusterLogStore::new());
        let state_machine = Arc::new(ClusterStateMachine::new());
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
            Err(e) => {
                // "NotAllowed" means already initialized — safe to ignore
                let err_str = format!("{}", e);
                if err_str.contains("NotAllowed") || err_str.contains("already initialized") {
                    debug!(
                        node_id = self.node_id,
                        "Raft already initialized, skipping bootstrap"
                    );
                    Ok(())
                } else {
                    Err(e.into())
                }
            }
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
