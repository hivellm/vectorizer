# Distributed Sharding & Clustering Specification

**Status**: Research & Specification  
**Priority**: P3 (Future/Experimental)  
**Complexity**: Very High  
**Created**: October 1, 2025  
**Target Version**: v1.5.0+

## Problem Statement

Current Vectorizer is single-node:
- ❌ **Limited by single machine resources** (CPU, memory, disk)
- ❌ **No horizontal scaling** for increased load
- ❌ **No high availability** (single point of failure)
- ❌ **No geographic distribution** for latency optimization
- ❌ **Manual sharding** if needed

**Need**: Distributed architecture supporting:
- ✅ Automatic sharding across multiple nodes
- ✅ High availability with replication
- ✅ Load balancing
- ✅ Fault tolerance
- ✅ Transparent to clients

## Proposed Architecture

### Cluster Topology

```
                    ┌─────────────────────────────────┐
                    │      Load Balancer / Gateway     │
                    │    (Consistent Hash Ring)        │
                    └──────────┬──────────────────────┘
                               │
         ┌─────────────────────┼─────────────────────┐
         │                     │                     │
    ┌────▼────┐           ┌────▼────┐           ┌────▼────┐
    │ Node 1  │           │ Node 2  │           │ Node 3  │
    │ Primary │◄─────────►│ Primary │◄─────────►│ Primary │
    │         │   SWIM    │         │   SWIM    │         │
    │ Shards: │  Gossip   │ Shards: │  Gossip   │ Shards: │
    │  0-99   │           │ 100-199 │           │ 200-255 │
    └────┬────┘           └────┬────┘           └────┬────┘
         │                     │                     │
    ┌────▼────┐           ┌────▼────┐           ┌────▼────┐
    │ Replica │           │ Replica │           │ Replica │
    │ (Node 2)│           │ (Node 3)│           │ (Node 1)│
    └─────────┘           └─────────┘           └─────────┘
```

## Technical Design

### 1. Membership Management (SWIM Protocol)

```rust
use std::collections::HashMap;
use std::net::SocketAddr;
use tokio::time::{interval, Duration};

pub struct SwimMembership {
    local_node: NodeInfo,
    members: Arc<RwLock<HashMap<String, MemberState>>>,
    config: SwimConfig,
}

#[derive(Clone, Debug)]
pub struct NodeInfo {
    pub id: String,
    pub addr: SocketAddr,
    pub metadata: NodeMetadata,
}

#[derive(Clone, Debug)]
pub struct NodeMetadata {
    pub shards: Vec<ShardRange>,
    pub capacity_cpu: f32,
    pub capacity_memory_gb: f32,
    pub region: String,
}

#[derive(Clone, Debug)]
pub enum MemberState {
    Alive {
        last_seen: Instant,
        incarnation: u64,
    },
    Suspect {
        suspected_at: Instant,
        incarnation: u64,
    },
    Dead {
        confirmed_at: Instant,
    },
}

pub struct SwimConfig {
    pub probe_interval: Duration,        // 1 second
    pub probe_timeout: Duration,         // 500ms
    pub suspect_timeout: Duration,       // 5 seconds
    pub gossip_fanout: usize,           // 3 nodes
    pub max_message_size: usize,        // 1KB
}

impl SwimMembership {
    pub async fn start(&self) {
        // Spawn background tasks
        tokio::spawn(self.clone().probe_loop());
        tokio::spawn(self.clone().gossip_loop());
        tokio::spawn(self.clone().suspect_timeout_loop());
    }
    
    async fn probe_loop(self) {
        let mut interval = interval(self.config.probe_interval);
        
        loop {
            interval.tick().await;
            
            // Select random member to probe
            let target = self.select_random_member().await;
            
            // Send probe (ping)
            match self.send_probe(&target).await {
                Ok(_) => {
                    // Mark as alive
                    self.mark_alive(&target.id).await;
                },
                Err(_) => {
                    // Indirect probe through other members
                    if !self.indirect_probe(&target).await {
                        // Mark as suspect
                        self.mark_suspect(&target.id).await;
                    }
                }
            }
        }
    }
    
    async fn gossip_loop(self) {
        let mut interval = interval(Duration::from_millis(100));
        
        loop {
            interval.tick().await;
            
            // Select random members for gossip
            let targets = self.select_gossip_targets(self.config.gossip_fanout).await;
            
            // Prepare gossip message
            let message = self.prepare_gossip_message().await;
            
            // Send to targets
            for target in targets {
                let _ = self.send_gossip(&target, &message).await;
            }
        }
    }
    
    async fn mark_suspect(&self, node_id: &str) {
        let mut members = self.members.write().await;
        
        if let Some(state) = members.get_mut(node_id) {
            *state = MemberState::Suspect {
                suspected_at: Instant::now(),
                incarnation: match state {
                    MemberState::Alive { incarnation, .. } => *incarnation,
                    _ => 0,
                },
            };
            
            info!("Node {} marked as SUSPECT", node_id);
        }
    }
}
```

### 2. Sharding Strategy (Consistent Hashing)

```rust
use std::collections::BTreeMap;
use sha2::{Sha256, Digest};

pub struct ConsistentHashRing {
    ring: BTreeMap<u64, String>,        // hash -> node_id
    virtual_nodes: usize,               // 150 per node
    nodes: HashMap<String, NodeInfo>,
}

impl ConsistentHashRing {
    pub fn new(virtual_nodes: usize) -> Self {
        Self {
            ring: BTreeMap::new(),
            virtual_nodes,
            nodes: HashMap::new(),
        }
    }
    
    pub fn add_node(&mut self, node: NodeInfo) {
        // Add virtual nodes for better distribution
        for i in 0..self.virtual_nodes {
            let key = format!("{}:{}", node.id, i);
            let hash = self.hash(&key);
            self.ring.insert(hash, node.id.clone());
        }
        
        self.nodes.insert(node.id.clone(), node);
        
        info!("Node {} added to ring with {} virtual nodes", 
            node.id, self.virtual_nodes);
    }
    
    pub fn remove_node(&mut self, node_id: &str) {
        // Remove all virtual nodes
        self.ring.retain(|_, id| id != node_id);
        self.nodes.remove(node_id);
        
        info!("Node {} removed from ring", node_id);
    }
    
    pub fn get_node(&self, key: &str) -> Option<&NodeInfo> {
        let hash = self.hash(key);
        
        // Find first node with hash >= key hash (clockwise on ring)
        let node_id = self.ring
            .range(hash..)
            .next()
            .or_else(|| self.ring.iter().next())
            .map(|(_, id)| id)?;
        
        self.nodes.get(node_id)
    }
    
    pub fn get_replicas(&self, key: &str, count: usize) -> Vec<&NodeInfo> {
        let hash = self.hash(key);
        let mut replicas = Vec::new();
        let mut seen = HashSet::new();
        
        // Walk clockwise around ring
        for (_, node_id) in self.ring.range(hash..).chain(self.ring.iter()) {
            if !seen.contains(node_id) {
                if let Some(node) = self.nodes.get(node_id) {
                    replicas.push(node);
                    seen.insert(node_id.clone());
                    
                    if replicas.len() == count {
                        break;
                    }
                }
            }
        }
        
        replicas
    }
    
    fn hash(&self, key: &str) -> u64 {
        let mut hasher = Sha256::new();
        hasher.update(key.as_bytes());
        let result = hasher.finalize();
        
        // Use first 8 bytes as u64
        u64::from_be_bytes(result[..8].try_into().unwrap())
    }
}
```

### 3. Data Sharding

```rust
pub struct ShardedVectorStore {
    ring: Arc<RwLock<ConsistentHashRing>>,
    local_shards: Arc<RwLock<HashMap<ShardId, VectorStore>>>,
    replication_factor: usize,
    sharding_strategy: ShardingStrategy,
}

#[derive(Clone, Debug)]
pub enum ShardingStrategy {
    // Shard by collection name
    ByCollection,
    
    // Shard by collection + vector ID
    ByVectorId { shards_per_collection: usize },
    
    // Shard by hash of vector content
    ByContent,
    
    // Custom sharding function
    Custom { func: Arc<dyn Fn(&str, &Vector) -> ShardId + Send + Sync> },
}

impl ShardedVectorStore {
    pub async fn insert_vector(
        &self,
        collection: &str,
        vector: Vector,
    ) -> Result<()> {
        // 1. Determine shard
        let shard_id = self.calculate_shard(collection, &vector)?;
        
        // 2. Get responsible nodes
        let shard_key = format!("shard:{}", shard_id);
        let nodes = self.ring.read().await.get_replicas(&shard_key, self.replication_factor);
        
        // 3. Write to primary
        let primary = &nodes[0];
        if self.is_local_node(primary) {
            // Local write
            self.write_local(shard_id, collection, vector.clone()).await?;
        } else {
            // Remote write
            self.write_remote(primary, shard_id, collection, vector.clone()).await?;
        }
        
        // 4. Replicate to replicas (async)
        for replica in &nodes[1..] {
            let replica = (*replica).clone();
            let vector = vector.clone();
            let collection = collection.to_string();
            
            tokio::spawn(async move {
                if let Err(e) = self.replicate_write(&replica, shard_id, &collection, &vector).await {
                    error!("Replication failed to {}: {}", replica.id, e);
                }
            });
        }
        
        Ok(())
    }
    
    pub async fn search_vector(
        &self,
        collection: &str,
        query: &str,
        limit: usize,
    ) -> Result<Vec<SearchResult>> {
        // 1. Determine which shards contain this collection
        let shard_ids = self.get_collection_shards(collection).await?;
        
        // 2. Query all shards in parallel
        let mut handles = vec![];
        
        for shard_id in shard_ids {
            let shard_key = format!("shard:{}", shard_id);
            let node = self.ring.read().await.get_node(&shard_key);
            
            if let Some(node) = node {
                let handle = self.query_shard(
                    node.clone(),
                    shard_id,
                    collection.to_string(),
                    query.to_string(),
                    limit,
                );
                
                handles.push(handle);
            }
        }
        
        // 3. Collect results from all shards
        let results = futures::future::join_all(handles).await;
        
        // 4. Merge and re-rank
        let merged = self.merge_results(results, limit)?;
        
        Ok(merged)
    }
    
    fn calculate_shard(&self, collection: &str, vector: &Vector) -> Result<ShardId> {
        match &self.sharding_strategy {
            ShardingStrategy::ByCollection => {
                let hash = self.hash(collection);
                Ok(hash % self.total_shards() as u64)
            },
            
            ShardingStrategy::ByVectorId { shards_per_collection } => {
                let key = format!("{}:{}", collection, vector.id);
                let hash = self.hash(&key);
                Ok(hash % shards_per_collection)
            },
            
            ShardingStrategy::ByContent => {
                // Hash vector content for content-based sharding
                let content_hash = self.hash_vector_content(&vector.data);
                Ok(content_hash % self.total_shards() as u64)
            },
            
            ShardingStrategy::Custom { func } => {
                Ok(func(collection, vector))
            },
        }
    }
}
```

### 4. Consensus (Raft Protocol)

```rust
pub struct RaftConsensus {
    node_id: String,
    state: Arc<RwLock<RaftState>>,
    log: Arc<RwLock<ReplicatedLog>>,
    cluster: Arc<RwLock<ClusterState>>,
}

#[derive(Clone, Debug)]
pub enum RaftState {
    Follower { leader: Option<String> },
    Candidate { votes_received: HashSet<String> },
    Leader { next_index: HashMap<String, u64> },
}

pub struct ReplicatedLog {
    entries: Vec<LogEntry>,
    commit_index: u64,
    last_applied: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LogEntry {
    pub term: u64,
    pub index: u64,
    pub command: ClusterCommand,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum ClusterCommand {
    // Shard operations
    CreateShard { shard_id: ShardId, replicas: Vec<String> },
    DeleteShard { shard_id: ShardId },
    RebalanceShards { plan: RebalancePlan },
    
    // Collection operations
    CreateCollection { name: String, config: CollectionConfig },
    DeleteCollection { name: String },
    
    // Node operations
    AddNode { node: NodeInfo },
    RemoveNode { node_id: String },
    
    // Configuration
    UpdateConfig { key: String, value: String },
}

impl RaftConsensus {
    pub async fn propose_command(&self, command: ClusterCommand) -> Result<u64> {
        // 1. Check if leader
        let state = self.state.read().await;
        if !matches!(*state, RaftState::Leader { .. }) {
            return Err(Error::NotLeader);
        }
        drop(state);
        
        // 2. Append to local log
        let entry = self.append_log_entry(command).await?;
        
        // 3. Replicate to followers
        self.replicate_entry(&entry).await?;
        
        // 4. Wait for majority
        self.wait_for_commit(entry.index).await?;
        
        // 5. Apply to state machine
        self.apply_committed_entries().await?;
        
        Ok(entry.index)
    }
    
    async fn replicate_entry(&self, entry: &LogEntry) -> Result<()> {
        let cluster = self.cluster.read().await;
        let followers: Vec<_> = cluster.members.iter()
            .filter(|(id, _)| *id != &self.node_id)
            .collect();
        
        let mut handles = vec![];
        
        for (follower_id, follower) in followers {
            let entry = entry.clone();
            let follower = follower.clone();
            
            let handle = tokio::spawn(async move {
                // Send AppendEntries RPC
                let response = send_append_entries(
                    &follower.addr,
                    AppendEntriesRequest {
                        term: entry.term,
                        leader_id: self.node_id.clone(),
                        prev_log_index: entry.index - 1,
                        entries: vec![entry],
                        leader_commit: self.log.read().await.commit_index,
                    }
                ).await?;
                
                Ok::<_, Error>((follower_id.clone(), response))
            });
            
            handles.push(handle);
        }
        
        // Wait for majority
        let results = futures::future::join_all(handles).await;
        let successful = results.iter().filter(|r| r.is_ok()).count();
        
        if successful < (cluster.members.len() / 2) {
            return Err(Error::ReplicationFailed);
        }
        
        Ok(())
    }
}
```

### 5. Shard Migration

```rust
pub struct ShardMigrator {
    source_node: NodeInfo,
    target_node: NodeInfo,
    progress: Arc<RwLock<MigrationProgress>>,
}

pub struct MigrationProgress {
    pub shard_id: ShardId,
    pub total_vectors: usize,
    pub migrated_vectors: usize,
    pub status: MigrationStatus,
    pub started_at: DateTime<Utc>,
    pub estimated_completion: Option<DateTime<Utc>>,
}

#[derive(Clone, Debug)]
pub enum MigrationStatus {
    Preparing,
    Copying,
    Verifying,
    Switching,
    Complete,
    Failed { error: String },
}

impl ShardMigrator {
    pub async fn migrate_shard(&self, shard_id: ShardId) -> Result<()> {
        // Phase 1: Prepare
        self.update_status(MigrationStatus::Preparing).await;
        self.create_target_shard(shard_id).await?;
        
        // Phase 2: Copy data (with write forwarding)
        self.update_status(MigrationStatus::Copying).await;
        self.copy_shard_data(shard_id).await?;
        
        // Phase 3: Catch up with ongoing writes
        self.sync_pending_writes(shard_id).await?;
        
        // Phase 4: Verify data integrity
        self.update_status(MigrationStatus::Verifying).await;
        self.verify_shard_data(shard_id).await?;
        
        // Phase 5: Atomic switch
        self.update_status(MigrationStatus::Switching).await;
        self.switch_shard_primary(shard_id).await?;
        
        // Phase 6: Cleanup old data
        self.cleanup_old_shard(shard_id).await?;
        
        self.update_status(MigrationStatus::Complete).await;
        
        Ok(())
    }
    
    async fn copy_shard_data(&self, shard_id: ShardId) -> Result<()> {
        let batch_size = 1000;
        let mut offset = 0;
        
        loop {
            // Read batch from source
            let batch = self.read_source_batch(shard_id, offset, batch_size).await?;
            
            if batch.is_empty() {
                break;
            }
            
            // Write to target
            self.write_target_batch(shard_id, &batch).await?;
            
            // Update progress
            offset += batch.len();
            self.update_migrated_count(offset).await;
            
            // Rate limit to avoid overload
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
        
        Ok(())
    }
}
```

### 6. Gossip-based Replication

```rust
pub struct GossipReplicator {
    local_node: NodeInfo,
    peers: Arc<RwLock<Vec<NodeInfo>>>,
    pending_updates: Arc<Mutex<Vec<Update>>>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Update {
    pub id: String,
    pub shard_id: ShardId,
    pub collection: String,
    pub operation: Operation,
    pub timestamp: DateTime<Utc>,
    pub origin_node: String,
    pub version_vector: VersionVector,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct VersionVector {
    // node_id -> sequence_number
    pub versions: HashMap<String, u64>,
}

impl GossipReplicator {
    pub async fn replicate_update(&self, update: Update) {
        // 1. Add to pending queue
        self.pending_updates.lock().await.push(update.clone());
        
        // 2. Gossip to random peers
        let peers = self.select_random_peers(3).await;
        
        for peer in peers {
            let update = update.clone();
            
            tokio::spawn(async move {
                if let Err(e) = send_gossip(&peer.addr, &update).await {
                    error!("Failed to gossip to {}: {}", peer.id, e);
                }
            });
        }
    }
    
    pub async fn handle_gossip(&self, update: Update) -> Result<()> {
        // 1. Check if already applied using version vector
        if self.already_applied(&update).await {
            return Ok(());
        }
        
        // 2. Apply update locally
        self.apply_update(&update).await?;
        
        // 3. Forward to other peers (anti-entropy)
        self.forward_gossip(&update).await;
        
        Ok(())
    }
    
    async fn already_applied(&self, update: &Update) -> bool {
        let local_vv = self.get_local_version_vector().await;
        
        // Check if we've seen a higher version from the origin node
        local_vv.versions.get(&update.origin_node)
            .map(|&v| v >= update.version_vector.versions[&update.origin_node])
            .unwrap_or(false)
    }
}
```

### 7. Query Routing

```rust
pub struct QueryRouter {
    ring: Arc<RwLock<ConsistentHashRing>>,
    local_node_id: String,
}

impl QueryRouter {
    pub async fn route_query(
        &self,
        request: SearchRequest,
    ) -> Result<SearchResponse> {
        // 1. Determine which shards need to be queried
        let shards = self.determine_query_shards(&request).await?;
        
        // 2. Group shards by node
        let queries_by_node = self.group_by_node(shards).await;
        
        // 3. Execute queries in parallel
        let mut handles = vec![];
        
        for (node, shard_ids) in queries_by_node {
            let request = request.clone();
            
            let handle = if node.id == self.local_node_id {
                // Local query
                self.query_local_shards(shard_ids, request)
            } else {
                // Remote query
                self.query_remote_shards(node, shard_ids, request)
            };
            
            handles.push(handle);
        }
        
        // 4. Collect results
        let results = futures::future::join_all(handles).await;
        
        // 5. Merge and rank
        let merged = self.merge_and_rank(results, request.limit)?;
        
        Ok(SearchResponse {
            results: merged,
            total_shards_queried: shards.len(),
            query_time_ms: start.elapsed().as_millis() as f64,
        })
    }
    
    async fn determine_query_shards(&self, request: &SearchRequest) -> Result<Vec<ShardId>> {
        match &request.scope {
            QueryScope::AllCollections => {
                // Query all shards
                Ok((0..self.total_shards()).collect())
            },
            
            QueryScope::SingleCollection(name) => {
                // Query only shards containing this collection
                self.get_collection_shards(name).await
            },
            
            QueryScope::Collections(names) => {
                // Query union of shard sets
                let mut all_shards = HashSet::new();
                for name in names {
                    let shards = self.get_collection_shards(name).await?;
                    all_shards.extend(shards);
                }
                Ok(all_shards.into_iter().collect())
            },
        }
    }
}
```

## Alternative Approaches

### Option A: SWIM + Consistent Hashing (Recommended)

**Pros**:
- ✅ Proven protocol (used by Consul, Memberlist)
- ✅ Efficient failure detection
- ✅ Minimal network overhead
- ✅ Easy to implement

**Cons**:
- ❌ Eventually consistent
- ❌ No strong consistency guarantees

**Best for**: High availability, large clusters (10+ nodes)

### Option B: Raft Consensus

**Pros**:
- ✅ Strong consistency
- ✅ Simple to understand
- ✅ Well-tested (etcd, Consul)

**Cons**:
- ❌ Higher latency (requires quorum)
- ❌ Limited scalability (typically 3-7 nodes)
- ❌ More complex

**Best for**: Small clusters (3-5 nodes), critical data

### Option C: Hybrid (SWIM + Raft)

**Approach**:
- Use **SWIM** for membership and failure detection
- Use **Raft** for metadata consensus (shard assignments, config)
- Use **Gossip** for data replication

**Pros**:
- ✅ Best of both worlds
- ✅ Fast failure detection (SWIM)
- ✅ Strong consistency for metadata (Raft)
- ✅ High availability for data (Gossip)

**Cons**:
- ❌ More complex to implement
- ❌ Two protocols to maintain

**Best for**: Production deployments (recommended)

### Option D: CRDTs (Conflict-free Replicated Data Types)

**Approach**:
- Use **CRDTs** for automatic conflict resolution
- No consensus needed
- Eventually consistent

**Pros**:
- ✅ No coordination overhead
- ✅ Always available
- ✅ Partition tolerant

**Cons**:
- ❌ Complex for vector operations
- ❌ No total ordering
- ❌ May not fit vector database semantics

**Best for**: Geo-distributed deployments with high latency

## Recommended Architecture (Hybrid)

```rust
pub struct DistributedVectorizer {
    // Membership (SWIM)
    membership: SwimMembership,
    
    // Metadata consensus (Raft)
    consensus: RaftConsensus,
    
    // Data sharding (Consistent Hash)
    shard_router: ConsistentHashRing,
    
    // Data replication (Gossip)
    replicator: GossipReplicator,
    
    // Local storage
    local_store: VectorStore,
}

impl DistributedVectorizer {
    pub async fn start_cluster(&mut self) -> Result<()> {
        // 1. Join cluster (SWIM)
        self.membership.join_cluster(self.config.seed_nodes.clone()).await?;
        
        // 2. Start Raft consensus
        self.consensus.start().await?;
        
        // 3. Wait for leader election
        self.consensus.wait_for_leader().await?;
        
        // 4. Discover shard assignments
        let assignments = self.consensus.get_shard_assignments().await?;
        
        // 5. Load assigned shards
        for shard_id in assignments.primary_shards {
            self.load_shard(shard_id).await?;
        }
        
        // 6. Start replication
        self.replicator.start().await?;
        
        // 7. Participate in gossip
        self.membership.start_gossip().await?;
        
        info!("Node {} joined cluster successfully", self.membership.local_node.id);
        
        Ok(())
    }
}
```

## Sharding Strategies Comparison

### Strategy 1: Collection-Based Sharding

```rust
// Each collection lives entirely on one set of nodes
// Shard key: hash(collection_name) % num_shards

Pros:
  ✅ Simple queries (no scatter-gather within collection)
  ✅ Locality preserved
  ✅ Easy to reason about
  
Cons:
  ❌ Unbalanced if collections vary in size
  ❌ Hot collections create hotspots
  ❌ Limited parallelism for single collection
```

### Strategy 2: Vector-Based Sharding

```rust
// Vectors distributed across shards
// Shard key: hash(collection_name + vector_id) % num_shards

Pros:
  ✅ Perfect load balancing
  ✅ Parallelism even for single collection
  ✅ No hotspots
  
Cons:
  ❌ All queries are scatter-gather
  ❌ Higher latency
  ❌ More network traffic
```

### Strategy 3: Hybrid Sharding (Recommended)

```rust
// Small collections: collection-based
// Large collections: vector-based with multiple sub-shards

Pros:
  ✅ Balances locality and distribution
  ✅ Optimizes for common case
  ✅ Adaptable to workload
  
Cons:
  ❌ More complex routing logic
  ❌ Requires monitoring and rebalancing
```

## Configuration

```yaml
# config.yml
cluster:
  # Basic settings
  enabled: true
  node_id: "node-1"
  region: "us-east-1"
  
  # Membership (SWIM)
  swim:
    probe_interval: 1s
    probe_timeout: 500ms
    suspect_timeout: 5s
    gossip_fanout: 3
    port: 7946
  
  # Consensus (Raft)
  raft:
    election_timeout: 150-300ms
    heartbeat_interval: 50ms
    snapshot_interval: 10000   # entries
    port: 7947
  
  # Sharding
  sharding:
    strategy: "hybrid"
    total_shards: 256
    replication_factor: 3
    virtual_nodes: 150         # per physical node
  
  # Replication
  replication:
    method: "gossip"
    sync_interval: 100ms
    batch_size: 1000
    compression: true
  
  # Seed nodes for bootstrapping
  seed_nodes:
    - "node-1.cluster.local:7946"
    - "node-2.cluster.local:7946"
    - "node-3.cluster.local:7946"
  
  # Rebalancing
  rebalancing:
    auto_enabled: true
    trigger_threshold: 0.20    # 20% imbalance
    max_concurrent_migrations: 2
    throttle_mb_per_sec: 100
```

## Monitoring & Observability

```rust
pub struct ClusterMetrics {
    // Membership metrics
    pub active_nodes: usize,
    pub suspected_nodes: usize,
    pub dead_nodes: usize,
    
    // Shard metrics
    pub total_shards: usize,
    pub healthy_shards: usize,
    pub under_replicated_shards: usize,
    pub over_replicated_shards: usize,
    
    // Replication metrics
    pub replication_lag_ms: HashMap<ShardId, f64>,
    pub pending_replications: usize,
    
    // Query metrics
    pub local_queries_per_sec: f64,
    pub remote_queries_per_sec: f64,
    pub avg_scatter_fanout: f32,
    
    // Migration metrics
    pub active_migrations: usize,
    pub completed_migrations: usize,
    pub failed_migrations: usize,
}

// Expose via API
GET /api/cluster/metrics
GET /api/cluster/health
GET /api/cluster/topology
```

## Implementation Phases

### Phase 1: Foundation (4 weeks)
- [ ] SWIM membership implementation
- [ ] Consistent hash ring
- [ ] Basic node communication
- [ ] Cluster formation

### Phase 2: Sharding (6 weeks)
- [ ] Shard assignment logic
- [ ] Shard migration
- [ ] Query routing
- [ ] Load balancing

### Phase 3: Replication (4 weeks)
- [ ] Gossip protocol
- [ ] Anti-entropy
- [ ] Conflict resolution
- [ ] Verification

### Phase 4: Consensus (6 weeks)
- [ ] Raft implementation
- [ ] Leader election
- [ ] Log replication
- [ ] State machine

### Phase 5: Operations (4 weeks)
- [ ] Auto-rebalancing
- [ ] Rolling upgrades
- [ ] Monitoring dashboard
- [ ] Alerting system

**Total**: 24 weeks (~6 months) with 2-3 developers

## Testing Strategy

### Chaos Engineering

```rust
#[cfg(test)]
mod chaos_tests {
    #[tokio::test]
    async fn test_node_failure_during_write() {
        let cluster = setup_test_cluster(5).await;
        
        // Start write load
        let write_handle = tokio::spawn(async move {
            for i in 0..10000 {
                cluster.insert_vector("test", generate_vector(i)).await?;
            }
        });
        
        // Kill random node during writes
        tokio::time::sleep(Duration::from_secs(2)).await;
        cluster.kill_random_node().await;
        
        // Wait for writes to complete
        write_handle.await??;
        
        // Verify all data present
        assert_eq!(cluster.count_vectors("test").await?, 10000);
    }
    
    #[tokio::test]
    async fn test_network_partition() {
        let cluster = setup_test_cluster(5).await;
        
        // Create partition: [1,2] <-X-> [3,4,5]
        cluster.partition_network(vec![1, 2], vec![3, 4, 5]).await;
        
        // Both sides should elect leaders
        tokio::time::sleep(Duration::from_secs(1)).await;
        
        // Writes to majority partition should succeed
        let result = cluster.node(3).insert_vector("test", vec![1.0; 384]).await;
        assert!(result.is_ok());
        
        // Writes to minority should fail
        let result = cluster.node(1).insert_vector("test", vec![1.0; 384]).await;
        assert!(result.is_err());
        
        // Heal partition
        cluster.heal_network().await;
        
        // Eventually consistent
        tokio::time::sleep(Duration::from_secs(5)).await;
        assert_eq!(
            cluster.node(1).count_vectors("test").await?,
            cluster.node(3).count_vectors("test").await?
        );
    }
}
```

## Performance Targets

| Metric | Single Node | 3-Node Cluster | 10-Node Cluster |
|--------|-------------|----------------|-----------------|
| Write Latency | 10µs | 15µs | 20µs |
| Read Latency | 0.8ms | 1.2ms | 1.5ms |
| Throughput (writes) | 100K/s | 250K/s | 800K/s |
| Throughput (reads) | 1000 QPS | 2500 QPS | 8000 QPS |
| Max Collections | 1K | 5K | 20K |
| Max Vectors | 10M | 50M | 200M |

## Deployment Example

```yaml
# Kubernetes StatefulSet
apiVersion: apps/v1
kind: StatefulSet
metadata:
  name: vectorizer-cluster
spec:
  serviceName: vectorizer
  replicas: 5
  selector:
    matchLabels:
      app: vectorizer
  template:
    metadata:
      labels:
        app: vectorizer
    spec:
      containers:
      - name: vectorizer
        image: hivellm/vectorizer:0.25.0-cluster
        env:
        - name: CLUSTER_ENABLED
          value: "true"
        - name: NODE_ID
          valueFrom:
            fieldRef:
              fieldPath: metadata.name
        - name: SEED_NODES
          value: "vectorizer-0.vectorizer:7946,vectorizer-1.vectorizer:7946,vectorizer-2.vectorizer:7946"
        ports:
        - containerPort: 15001  # REST API
        - containerPort: 15002  # MCP
        - containerPort: 15003  # gRPC
        - containerPort: 7946   # SWIM
        - containerPort: 7947   # Raft
        volumeMounts:
        - name: data
          mountPath: /data
  volumeClaimTemplates:
  - metadata:
      name: data
    spec:
      accessModes: ["ReadWriteOnce"]
      resources:
        requests:
          storage: 100Gi
```

## Migration Path

### From Single Node to Cluster

```rust
pub async fn migrate_to_cluster(
    single_node: &VectorStore,
    cluster_config: ClusterConfig,
) -> Result<()> {
    // 1. Create cluster with single node
    let cluster = DistributedVectorizer::new_single_node(cluster_config)?;
    
    // 2. Add collections to cluster
    for collection in single_node.list_collections().await? {
        cluster.import_collection(collection).await?;
    }
    
    // 3. Add additional nodes
    for node in cluster_config.additional_nodes {
        cluster.add_node(node).await?;
    }
    
    // 4. Trigger rebalancing
    cluster.rebalance().await?;
    
    // 5. Wait for data distribution
    cluster.wait_for_balance().await?;
    
    Ok(())
}
```

## Success Criteria

- ✅ Cluster formation working
- ✅ Automatic failure detection (< 5s)
- ✅ Shard migration without downtime
- ✅ Query scatter-gather working
- ✅ 3x throughput with 3 nodes
- ✅ Linear scalability up to 10 nodes
- ✅ Survive node failures gracefully
- ✅ Network partition tolerance

## Future Enhancements

### Multi-Datacenter

```rust
pub struct MultiDatacenterCluster {
    regions: Vec<Region>,
    cross_region_replication: bool,
    locality_aware_routing: bool,
}

// Read from nearest datacenter
// Write to local, async replicate to other regions
```

### Auto-Scaling

```rust
pub struct AutoScaler {
    min_nodes: usize,
    max_nodes: usize,
    scale_up_threshold: f32,   // CPU/memory %
    scale_down_threshold: f32,
}

// Automatically add/remove nodes based on load
```

### Geo-Replication

```rust
pub struct GeoReplicatedCluster {
    regions: HashMap<String, Cluster>,
    replication_strategy: GeoReplicationStrategy,
}

pub enum GeoReplicationStrategy {
    ActivePassive { primary_region: String },
    ActiveActive { conflict_resolution: CRDTs },
    Regional { local_read_write: bool },
}
```

## Libraries & Crates

### Recommended Crates

```toml
[dependencies]
# Membership & Gossip
swim = "0.4"               # SWIM protocol
memberlist = "0.1"         # Alternative SWIM

# Consensus
raft = "0.7"               # Raft implementation
tikv-raft = "0.1"         # Production-grade Raft (from TiKV)

# Networking
tonic = "0.11"            # Already have
quinn = "0.10"            # QUIC for low-latency

# Serialization
prost = "0.12"            # Already have
bincode = "1.3"           # Already have

# Async primitives
tokio = "1.40"            # Already have
futures = "0.3"           # Already have
```

## Comparison with Existing Solutions

| Feature | Vectorizer Cluster | Milvus | Qdrant | Weaviate |
|---------|-------------------|--------|--------|----------|
| Sharding | ✅ Planned | ✅ | ✅ | ✅ |
| Replication | ✅ Planned | ✅ | ✅ | ✅ |
| Auto-rebalancing | ✅ Planned | ✅ | ⚠️ Manual | ✅ |
| Multi-DC | 🔮 Future | ✅ | ❌ | ✅ |
| Embedded Mode | ✅ Current | ❌ | ✅ | ❌ |
| Rust Native | ✅ | ❌ (Go) | ✅ | ❌ (Go) |

## Risk Assessment

### Technical Risks

**High Risks**:
- Split-brain scenarios during network partitions
- Data consistency during migrations
- Performance degradation with network latency

**Mitigations**:
- Use proven protocols (Raft for consensus)
- Comprehensive testing with chaos engineering
- Gradual rollout with feature flags

### Operational Risks

**Medium Risks**:
- Increased operational complexity
- More failure modes
- Higher resource requirements

**Mitigations**:
- Excellent monitoring and alerting
- Automated recovery procedures
- Clear runbooks and documentation

## Estimated Effort

### Development
- **Phase 1-2**: 10 weeks (membership + sharding)
- **Phase 3-4**: 10 weeks (replication + consensus)
- **Phase 5**: 4 weeks (operations)
- **Total**: 24 weeks (~6 months)

### Team
- 2 Senior Rust developers (distributed systems experience)
- 1 DevOps engineer (Kubernetes, monitoring)

### Testing
- Unit tests: 2 weeks
- Integration tests: 3 weeks
- Chaos tests: 2 weeks
- Load tests: 1 week
- **Total**: 8 weeks (parallel with development)

---

## 📚 References

### SWIM Protocol
- Paper: [SWIM: Scalable Weakly-consistent Infection-style Process Group Membership Protocol](https://www.cs.cornell.edu/projects/Quicksilver/public_pdfs/SWIM.pdf)
- Implementation: [HashiCorp Memberlist](https://github.com/hashicorp/memberlist)

### Raft Consensus
- Paper: [In Search of an Understandable Consensus Algorithm](https://raft.github.io/raft.pdf)
- Visualization: [The Secret Lives of Data - Raft](http://thesecretlivesofdata.com/raft/)

### Consistent Hashing
- Paper: [Consistent Hashing and Random Trees](https://www.akamai.com/us/en/multimedia/documents/technical-publication/consistent-hashing-and-random-trees-distributed-caching-protocols-for-relieving-hot-spots-on-the-world-wide-web-technical-publication.pdf)

### Distributed Systems
- Book: "Designing Data-Intensive Applications" by Martin Kleppmann
- Book: "Database Internals" by Alex Petrov

---

## Decision: When to Implement?

### Implement Clustering If:
- ✅ Single node capacity exceeded (>10M vectors)
- ✅ Need high availability (99.9%+ uptime)
- ✅ Geographic distribution required
- ✅ Team has distributed systems expertise

### Stay Single Node If:
- ✅ < 10M vectors (fits on one machine)
- ✅ Downtime acceptable
- ✅ Single region deployment
- ✅ Team prefers simplicity

### Our Recommendation:
**Implement after v1.0.0** when:
1. Core features proven stable
2. User demand exists
3. Single-node limits reached
4. Team has bandwidth

**Priority**: P3 (Future enhancement, not v1.0 requirement)

---

**Estimated Effort**: 6 months (2-3 developers)  
**Dependencies**: v1.0.0 complete, proven at scale  
**Risk**: High (complex distributed systems)  
**Reward**: Unlimited scalability, high availability

