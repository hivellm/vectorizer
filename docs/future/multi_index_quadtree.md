# Multi-Index Quad-Tree Sharding - Technical Specification

**Status:** RFC - Request for Comments  
**Version:** 1.0.0  
**Date:** October 7, 2025  
**Branch:** `feat/multi-index-quadtree-sharding`  
**Author:** Vectorizer Team

---

## Table of Contents

1. [Objective](#1-objective)
2. [Scope](#2-scope)
3. [Functional Requirements](#3-functional-requirements)
4. [Non-Functional Requirements](#4-non-functional-requirements)
5. [Architecture](#5-architecture)
6. [Data Structures](#6-data-structures)
7. [Algorithms](#7-algorithms)
8. [Configuration](#8-configuration)
9. [API & Interfaces](#9-api--interfaces)
10. [Persistence & WAL](#10-persistence--wal)
11. [Concurrency](#11-concurrency)
12. [Telemetry & Observability](#12-telemetry--observability)
13. [Migration & Backfill](#13-migration--backfill)
14. [Testing & Benchmarking](#14-testing--benchmarking)
15. [Risks & Mitigation](#15-risks--mitigation)
16. [Acceptance Criteria](#16-acceptance-criteria)
17. [Delivery Checklist](#17-delivery-checklist)
18. [Implementation Plan](#18-implementation-plan)

---

## 1. Objective

Implement "Multi-Index Quad-Tree per collection" with auto-sharding up to 10k vectors per index, parallel search across all shards with k-way merge, **maintaining 100% compatibility with current API and optional disable via configuration**. This architecture will scale collections to millions of vectors while maintaining sub-3ms latencies and search quality equivalent to a single index.

### 🎯 Core Principle: **Zero API Changes**

**The implementation MUST be completely transparent to existing clients:**
- ✅ All existing search functions (`search_vectors`, `semantic_search`, `intelligent_search`, etc.) maintain their exact signatures
- ✅ All existing MCP tools maintain their exact parameter schemas
- ✅ All SDKs (Python, TypeScript, JavaScript, Go, Rust) require ZERO changes
- ✅ All integrations (Cursor AI, Claude, ChatGPT) continue working without modifications
- ✅ Sharding is an **internal optimization** - clients never know it exists
- ✅ Configuration changes are **optional** and **backward compatible**
- ✅ Existing collections continue working without migration (until explicitly enabled)

**Implementation Strategy:**
- Sharding logic lives **inside** the `VectorStore` implementation
- The `search()` method signature remains unchanged
- Internal routing to shards is transparent
- Results are merged before returning to caller
- Performance improvements are automatic when enabled

---

## 2. Scope

### In Scope
- ✅ Automatic sharding within a single collection (or manual via config)
- ✅ Split when shard reaches soft_limit and hard_limit
- ✅ Parallel search across N shards with global k-top fusion and dedup
- ✅ Optional reranking on top-M final results
- ✅ Segmented persistence/WAL per shard
- ✅ Per-shard telemetry
- ✅ Background rebalance/merge

### Out of Scope
- ❌ Cross-node sharding (distributed)
- ❌ Replication between nodes
- ❌ Distributed consensus (Raft/Paxos)
- ❌ Dynamic query routing optimization
- ❌ Automatic shard migration between nodes

---

## 3. Functional Requirements

### FR1: Shard Size Limits
- **Requirement:** Each leaf index (shard) must have a maximum of **10,000 vectors** (configurable value)
- **Configuration:** Via `target_max` in workspace config
- **Valid Range:** 1,000 - 100,000 vectors
- **Rationale:** Optimal HNSW performance with manageable memory footprint

### FR2: Insert Routing
- **Requirement:** Inserts routed to shard with smallest size or via hash-range
- **Available Strategies:**
  - `min_size`: Always insert into shard with fewest vectors
  - `hash_range`: Use hash of vector_id to determine shard
  - `round_robin`: Distribute uniformly across shards
- **Default:** `min_size` for better balance

### FR3: Auto-Split
- **Requirement:** Automatic split when reaching `soft_limit` (default: 95% of target_max)
- **Hard Limit:** Mandatory split at `hard_limit` (default: 100% of target_max)
- **Behavior:** Split is asynchronous and does not block queries
- **Strategy:** Configurable (hash or kmeans2)

### FR4: Global Top-K Search
- **Requirement:** Search must return global top_k equivalent to a single index
- **Quality Guarantee:** recall@k ≥ 95% compared to single index
- **Deduplication:** Automatic by document_id
- **Performance:** p95 latency improvement for collections >100k vectors

### FR5: Configuration per Collection
- **Requirement:** Configuration per workspace/collection to enable/disable
- **Hot Reload:** Adjust limits without server restart
- **Granularity:** Per-collection policy override
- **Validation:** Config validation before application

### FR6: Multi-Index Search Tool
- **Tool Name:** `multi_index_search`
- **Input:** `{collection, query, top_k, options}`
- **Output:** `{hits[], diagnostics{shards_queried, merge_time, total_time}}`
- **Protocol:** MCP-compatible

---

## 4. Non-Functional Requirements

### NFR1: Performance
- **Target Latency:** p95 < 5ms for collections up to 1M vectors
- **Scalability:** Latency grows O(log N) with number of shards
- **Throughput:** Maintain or improve QPS compared to single index
- **Benchmark:** 500k single-index vs 50 shards x 10k

### NFR2: Reliability
- **Idempotence:** Split/merge operations are idempotent and safe
- **Journaling:** WAL-based journaling for all operations
- **Recovery:** Automatic recovery on failure
- **Data Integrity:** Zero data loss guarantee

### NFR3: Availability
- **Zero Downtime:** Logical zero downtime during split
- **Dual Read:** Queries during split consider old+new until commit
- **Rollback:** Automatic rollback on error
- **Health Checks:** Per-shard health monitoring

### NFR4: Observability
- **Metrics:** p50/p95/p99 per shard, sizes, split/merge events
- **Tracing:** Distributed tracing for cross-shard queries
- **Logging:** Structured logging for all operations
- **Dashboards:** Grafana-compatible metrics

---

## 5. Architecture

### 5.1 Component Hierarchy

```
Collection
    └── IndexSet (logical tree)
            ├── Node (internal) - routing logic
            │       ├── Node (internal)
            │       │       ├── Node (leaf) → Shard
            │       │       └── Node (leaf) → Shard
            │       └── Node (leaf) → Shard
            └── Node (leaf) → Shard
                    └── HNSW/ANN engine + WAL
```

### 5.2 Components

#### Collection
- **Purpose:** Main container for a collection
- **Responsibilities:**
  - Maintains reference to IndexSet
  - Manages sharding policies
  - Exposes unified API (transparent to client)
  - Coordinates split/merge operations

#### IndexSet
- **Purpose:** Logical tree of indices
- **Responsibilities:**
  - Root node can be internal or leaf
  - Manages split/merge operations
  - Maintains global metadata (total vectors, num shards)
  - Coordinates parallel queries

#### Node (Internal)
- **Purpose:** Routing node in the tree
- **Responsibilities:**
  - Contains list of child nodes
  - Implements routing logic (hash, range, etc)
  - Does not store vectors
  - Maintains routing statistics

#### Node (Leaf) / Shard
- **Purpose:** Physical storage unit
- **Responsibilities:**
  - Contains a real HNSW engine
  - Limit of `target_max` vectors
  - Own WAL segment
  - Independent statistics
  - Local query execution

### 5.3 Operational Flow

#### Insert Flow
```
Client → Collection → IndexSet → route_insert() → target Shard → HNSW insert → WAL append
                                                                          ↓
                                                              check_split_threshold()
```

#### Search Flow
```
Client → Collection → IndexSet → parallel_search() → [Shard1, Shard2, ..., ShardN]
                                                              ↓
                                                      gather results
                                                              ↓
                                                      k-way merge + dedup
                                                              ↓
                                                      optional rerank
                                                              ↓
                                                      return top-k
```

#### Split Flow
```
Shard (size > soft_limit) → trigger_split() → journal SPLIT_START
                                                    ↓
                                            run split strategy (kmeans2/hash)
                                                    ↓
                                            create Shard1 + Shard2
                                                    ↓
                                            journal SPLIT_PREPARE
                                                    ↓
                                            dual-read phase (old+new)
                                                    ↓
                                            journal SPLIT_COMMIT
                                                    ↓
                                            update IndexSet tree
                                                    ↓
                                            cleanup old shard
```

---

## 6. Data Structures

### 6.1 Core Structures

```rust
/// Collection with multi-index support
pub struct Collection {
    pub id: String,
    pub index_set: Arc<RwLock<IndexSet>>,
    pub policy: SplitPolicy,
    pub metadata: CollectionMetadata,
    pub created_at: DateTime<Utc>,
}

/// Logical tree of indices
pub struct IndexSet {
    pub root: Node,
    pub total_vectors: AtomicUsize,
    pub num_shards: AtomicUsize,
    pub created_at: DateTime<Utc>,
    pub version: u64,
}

/// Tree node (internal or leaf)
pub enum Node {
    Internal {
        id: String,
        children: Vec<Box<Node>>,
        routing: RoutingStrategy,
        stats: NodeStats,
    },
    Leaf {
        id: String,
        shard: Arc<RwLock<Shard>>,
    },
}

/// Physical shard with HNSW engine
pub struct Shard {
    pub id: String,
    pub engine: HnswEngine,
    pub size: AtomicUsize,
    pub wal_segment: WalSegment,
    pub stats: ShardStats,
    pub created_at: DateTime<Utc>,
    pub last_modified: DateTime<Utc>,
}

/// Sharding policy configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SplitPolicy {
    pub enabled: bool,
    pub target_max: usize,              // Default: 10,000
    pub soft_limit_ratio: f32,          // Default: 0.95
    pub hard_limit_ratio: f32,          // Default: 1.0
    pub mode: SplitMode,                // auto | manual
    pub rebalance: RebalanceMode,       // none | background | eager
    pub split_strategy: SplitStrategy,  // hash | kmeans2
    pub routing_strategy: RoutingStrategy,
    pub rerank: Option<RerankConfig>,
}

/// Split modes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SplitMode {
    Auto,    // Automatic split when limits reached
    Manual,  // Only split via admin command
}

/// Rebalance modes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RebalanceMode {
    None,       // No automatic rebalance
    Background, // Rebalance in background thread
    Eager,      // Immediate rebalance after operations
}

/// Split strategies
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SplitStrategy {
    Hash,     // Split by hash of vector_id
    Kmeans2,  // K-means with k=2 on vector space
}

/// Routing strategies for inserts
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RoutingStrategy {
    MinSize,    // Route to shard with fewest vectors
    HashRange,  // Consistent hashing by vector_id
    RoundRobin, // Simple round-robin
}

/// Rerank configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RerankConfig {
    pub enabled: bool,
    pub model: String,           // e.g., "bge-mini.onnx"
    pub top_m: usize,           // e.g., 64
    pub batch_size: usize,      // e.g., 32
}

/// Per-shard statistics
#[derive(Debug, Clone)]
pub struct ShardStats {
    pub size: usize,
    pub ef_search: usize,
    pub latencies: LatencyStats,
    pub last_split: Option<DateTime<Utc>>,
    pub last_merge: Option<DateTime<Utc>>,
    pub query_count: u64,
    pub insert_count: u64,
}

/// Latency statistics with percentiles
#[derive(Debug, Clone)]
pub struct LatencyStats {
    pub p50_ms: f64,
    pub p95_ms: f64,
    pub p99_ms: f64,
    pub mean_ms: f64,
    pub max_ms: f64,
}

/// Node statistics
#[derive(Debug, Clone)]
pub struct NodeStats {
    pub total_queries: u64,
    pub routing_time_ms: f64,
}
```

---

## 7. Algorithms

### 7.1 Insert Routing

```rust
fn route_insert(vector: &Vector, node: &Node) -> Result<&Shard> {
    match node {
        Node::Internal { children, routing, .. } => {
            let child = match routing {
                RoutingStrategy::MinSize => {
                    // Find child with smallest shard (recursive)
                    children.iter()
                        .min_by_key(|c| get_total_size(c))
                        .ok_or(VectorizerError::NoShardsAvailable)?
                }
                RoutingStrategy::HashRange => {
                    // Consistent hashing
                    let hash = hash_vector_id(&vector.id);
                    let idx = hash as usize % children.len();
                    &children[idx]
                }
                RoutingStrategy::RoundRobin => {
                    // Atomic round-robin counter
                    let idx = ROUND_ROBIN_COUNTER.fetch_add(1, Ordering::Relaxed) % children.len();
                    &children[idx]
                }
            };
            route_insert(vector, child)
        }
        Node::Leaf { shard, .. } => {
            Ok(shard.read().await)
        }
    }
}

fn get_total_size(node: &Node) -> usize {
    match node {
        Node::Internal { children, .. } => {
            children.iter().map(|c| get_total_size(c)).sum()
        }
        Node::Leaf { shard, .. } => {
            shard.read().await.size.load(Ordering::Relaxed)
        }
    }
}
```

### 7.2 Auto-Split with K-Means

```rust
async fn auto_split_shard(
    shard: &mut Shard,
    policy: &SplitPolicy,
) -> Result<(Shard, Shard)> {
    // Check if split is needed
    let soft_limit = (policy.target_max as f32 * policy.soft_limit_ratio) as usize;
    let current_size = shard.size.load(Ordering::Relaxed);
    
    if current_size < soft_limit {
        return Err(VectorizerError::SplitNotNeeded);
    }
    
    // Journal split start
    shard.wal_segment.append(WalEntry::SplitStart {
        shard_id: shard.id.clone(),
        size: current_size,
        strategy: policy.split_strategy.clone(),
    }).await?;
    
    // Perform split based on strategy
    match policy.split_strategy {
        SplitStrategy::Kmeans2 => {
            // 1. Extract all vectors
            let vectors = shard.engine.get_all_vectors();
            
            // 2. Run k-means with k=2
            let (centroid1, centroid2) = kmeans_2(&vectors, MAX_ITERATIONS)?;
            
            // 3. Partition vectors by closest centroid
            let (partition1, partition2): (Vec<_>, Vec<_>) = vectors
                .into_iter()
                .partition(|v| {
                    cosine_distance(v, &centroid1) < cosine_distance(v, &centroid2)
                });
            
            // Ensure balanced split (40-60% ratio minimum)
            if partition1.len() < (current_size / 10) * 4 || 
               partition2.len() < (current_size / 10) * 4 {
                // Fallback to hash-based split
                warn!("K-means produced unbalanced split, falling back to hash");
                return split_by_hash(shard, policy).await;
            }
            
            // 4. Create two new shards
            let shard1 = Shard::new_with_vectors(&shard.id, partition1).await?;
            let shard2 = Shard::new_with_vectors(&shard.id, partition2).await?;
            
            // Journal split prepare
            shard.wal_segment.append(WalEntry::SplitPrepare {
                shard_id: shard.id.clone(),
                new_shard_ids: vec![shard1.id.clone(), shard2.id.clone()],
            }).await?;
            
            Ok((shard1, shard2))
        }
        SplitStrategy::Hash => {
            split_by_hash(shard, policy).await
        }
    }
}

async fn split_by_hash(shard: &Shard, policy: &SplitPolicy) -> Result<(Shard, Shard)> {
    // Split by hash of vector_id
    let vectors = shard.engine.get_all_vectors();
    let (partition1, partition2): (Vec<_>, Vec<_>) = vectors
        .into_iter()
        .partition(|v| hash_vector_id(&v.id) % 2 == 0);
    
    let shard1 = Shard::new_with_vectors(&shard.id, partition1).await?;
    let shard2 = Shard::new_with_vectors(&shard.id, partition2).await?;
    
    Ok((shard1, shard2))
}

fn kmeans_2(vectors: &[Vector], max_iterations: usize) -> Result<(Vector, Vector)> {
    // Initialize centroids randomly
    let mut centroid1 = vectors[rand::random::<usize>() % vectors.len()].clone();
    let mut centroid2 = vectors[rand::random::<usize>() % vectors.len()].clone();
    
    for _ in 0..max_iterations {
        // Assign vectors to closest centroid
        let (cluster1, cluster2): (Vec<_>, Vec<_>) = vectors
            .iter()
            .partition(|v| {
                cosine_distance(v, &centroid1) < cosine_distance(v, &centroid2)
            });
        
        if cluster1.is_empty() || cluster2.is_empty() {
            // Degenerate case, re-initialize
            continue;
        }
        
        // Update centroids (mean of assigned vectors)
        let new_centroid1 = compute_mean(&cluster1);
        let new_centroid2 = compute_mean(&cluster2);
        
        // Check convergence
        if cosine_distance(&centroid1, &new_centroid1) < CONVERGENCE_THRESHOLD &&
           cosine_distance(&centroid2, &new_centroid2) < CONVERGENCE_THRESHOLD {
            break;
        }
        
        centroid1 = new_centroid1;
        centroid2 = new_centroid2;
    }
    
    Ok((centroid1, centroid2))
}
```

### 7.3 Parallel Search with K-Way Merge

```rust
async fn parallel_search(
    index_set: &IndexSet,
    query: &Vector,
    top_k: usize,
    options: &SearchOptions,
) -> Result<Vec<SearchResult>> {
    // 1. Get all leaf shards
    let shards = index_set.get_all_shards();
    let num_shards = shards.len();
    
    if num_shards == 0 {
        return Err(VectorizerError::NoShardsAvailable);
    }
    
    // 2. Search each shard in parallel (with 20% buffer for better recall)
    let top_k_prime = top_k + (top_k as f32 * 0.2).ceil() as usize;
    
    let start = Instant::now();
    let futures: Vec<_> = shards.iter()
        .map(|shard| {
            let shard = Arc::clone(shard);
            let query = query.clone();
            tokio::spawn(async move {
                let shard_start = Instant::now();
                let result = shard.read().await.engine.search(&query, top_k_prime);
                let latency = shard_start.elapsed();
                (result, latency)
            })
        })
        .collect();
    
    // 3. Await all results
    let results: Vec<(Vec<SearchResult>, Duration)> = join_all(futures)
        .await
        .into_iter()
        .filter_map(|r| r.ok())
        .filter_map(|(res, lat)| res.ok().map(|r| (r, lat)))
        .collect();
    
    let search_time = start.elapsed();
    
    // 4. K-way merge with min-heap (max-heap for scores)
    let merge_start = Instant::now();
    let mut heap = BinaryHeap::new();
    let mut iterators: Vec<_> = results
        .into_iter()
        .map(|(r, _)| r.into_iter().peekable())
        .collect();
    
    // Initialize heap with first element from each iterator
    for (idx, iter) in iterators.iter_mut().enumerate() {
        if let Some(result) = iter.peek() {
            heap.push(Reverse((
                OrderedFloat(result.score),
                idx,
                result.clone()
            )));
        }
    }
    
    // 5. Extract top-k with deduplication
    let mut merged = Vec::new();
    let mut seen_ids = HashSet::new();
    
    while let Some(Reverse((_, idx, result))) = heap.pop() {
        // Deduplication by document_id
        if seen_ids.insert(result.vector.id.clone()) {
            merged.push(result);
            
            if merged.len() >= top_k {
                break;
            }
        }
        
        // Advance iterator and push next element
        iterators[idx].next();
        if let Some(peeked) = iterators[idx].peek() {
            heap.push(Reverse((
                OrderedFloat(peeked.score),
                idx,
                peeked.clone()
            )));
        }
    }
    
    let merge_time = merge_start.elapsed();
    
    // 6. Add diagnostics
    let diagnostics = SearchDiagnostics {
        shards_queried: num_shards,
        search_time_ms: search_time.as_millis() as f64,
        merge_time_ms: merge_time.as_millis() as f64,
        total_time_ms: (search_time + merge_time).as_millis() as f64,
        results_before_dedup: heap.len() + merged.len(),
        results_after_dedup: merged.len(),
    };
    
    // Attach diagnostics to results metadata
    for result in &mut merged {
        result.diagnostics = Some(diagnostics.clone());
    }
    
    Ok(merged)
}
```

### 7.4 Optional Reranking

```rust
async fn rerank_results(
    results: Vec<SearchResult>,
    query: &Vector,
    rerank_config: &RerankConfig,
) -> Result<Vec<SearchResult>> {
    if !rerank_config.enabled {
        return Ok(results);
    }
    
    // Take top-M for reranking (e.g., 64)
    let mut top_m: Vec<_> = results.into_iter()
        .take(rerank_config.top_m)
        .collect();
    
    // Load ONNX model (cached globally)
    let model = RERANK_MODEL_CACHE
        .get_or_load(&rerank_config.model)
        .await?;
    
    // Rerank in batches
    for chunk in top_m.chunks_mut(rerank_config.batch_size) {
        // Prepare batch inputs
        let batch_queries = vec![query; chunk.len()];
        let batch_docs: Vec<_> = chunk.iter()
            .map(|r| &r.vector)
            .collect();
        
        // Run model inference
        let scores = model.score_batch(&batch_queries, &batch_docs).await?;
        
        // Update scores
        for (result, new_score) in chunk.iter_mut().zip(scores.iter()) {
            result.score = *new_score;
            result.reranked = true;
        }
    }
    
    // Sort by new scores
    top_m.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap());
    
    Ok(top_m)
}
```

### 7.5 Background Merge

```rust
async fn background_merge_loop(
    index_set: Arc<RwLock<IndexSet>>,
    policy: SplitPolicy,
) {
    // Find underutilized shards (<40% of target_max)
    let threshold = (policy.target_max as f32 * 0.4) as usize;
    let check_interval = Duration::from_secs(60);
    
    loop {
        tokio::time::sleep(check_interval).await;
        
        if policy.rebalance == RebalanceMode::None {
            continue;
        }
        
        let index_set_read = index_set.read().await;
        let shards = index_set_read.get_all_shards();
        
        // Find small shards that can be merged
        let small_shards: Vec<_> = shards
            .into_iter()
            .filter(|s| {
                let size = s.read().await.size.load(Ordering::Relaxed);
                size < threshold
            })
            .collect();
        
        if small_shards.len() < 2 {
            continue;
        }
        
        drop(index_set_read);
        
        // Merge pairs of small shards
        for chunk in small_shards.chunks(2) {
            if chunk.len() == 2 {
                match merge_shards(&chunk[0], &chunk[1], &policy).await {
                    Ok(merged) => {
                        // Update IndexSet
                        let mut index_set_write = index_set.write().await;
                        let shard_ids = vec![
                            chunk[0].read().await.id.clone(),
                            chunk[1].read().await.id.clone(),
                        ];
                        
                        if let Err(e) = index_set_write.replace_shards(&shard_ids, merged) {
                            error!("Failed to replace shards after merge: {}", e);
                        } else {
                            info!("Successfully merged shards: {:?}", shard_ids);
                        }
                    }
                    Err(e) => {
                        error!("Failed to merge shards: {}", e);
                    }
                }
            }
        }
    }
}

async fn merge_shards(
    shard1: &Arc<RwLock<Shard>>,
    shard2: &Arc<RwLock<Shard>>,
    policy: &SplitPolicy,
) -> Result<Shard> {
    let s1 = shard1.read().await;
    let s2 = shard2.read().await;
    
    // Check if merge is valid (combined size <= target_max)
    let combined_size = s1.size.load(Ordering::Relaxed) + 
                       s2.size.load(Ordering::Relaxed);
    
    if combined_size > policy.target_max {
        return Err(VectorizerError::MergeWouldExceedLimit);
    }
    
    // Journal merge start
    s1.wal_segment.append(WalEntry::MergeStart {
        shard_ids: vec![s1.id.clone(), s2.id.clone()],
        combined_size,
    }).await?;
    
    // Extract all vectors from both shards
    let mut all_vectors = s1.engine.get_all_vectors();
    all_vectors.extend(s2.engine.get_all_vectors());
    
    // Create new merged shard
    let merged = Shard::new_with_vectors("merged", all_vectors).await?;
    
    // Journal merge commit
    merged.wal_segment.append(WalEntry::MergeCommit {
        old_shard_ids: vec![s1.id.clone(), s2.id.clone()],
        new_shard_id: merged.id.clone(),
    }).await?;
    
    Ok(merged)
}
```

---

## 8. Configuration

### 8.1 TOML Configuration Example

```toml
[sharding]
enabled = true  # Global toggle - false for legacy behavior

[sharding.policy]
target_max = 10_000           # Target max vectors per shard
soft_limit_ratio = 0.85       # Split at 85% of target_max
hard_limit_ratio = 1.0        # Force split at 100%
min_merge_threshold = 2_000   # Merge if shard < 2k vectors

[sharding.policy.split_strategy]
type = "Kmeans2"              # Options: Kmeans2, HashRange, Median

[sharding.policy.routing]
strategy = "MinSize"          # Options: MinSize, HashRange, RoundRobin

[sharding.search]
parallel_search = true        # Enable parallel search across shards
dedup_enabled = true          # Deduplicate results by doc_id

[sharding.rerank]
enabled = false               # Optional reranking (disabled by default)
top_m = 64                    # Rerank top-64 results
batch_size = 16               # Rerank batch size
model = "bge-reranker-mini"   # ONNX model name

[sharding.background]
merge_enabled = true          # Enable background merge of small shards
merge_interval_secs = 3600    # Run merge every 1 hour
merge_max_concurrent = 2      # Max concurrent merges
```

### 8.2 Per-Collection Override

```toml
# config.yml
[[collections]]
name = "my-large-collection"
sharding_enabled = true
target_max = 5_000           # Override: smaller shards for this collection

[[collections]]
name = "my-small-collection"
sharding_enabled = false     # Keep single index for small collections
```

### 8.3 Environment Variables

```bash
VECTORIZER_SHARDING_ENABLED=true
VECTORIZER_SHARDING_TARGET_MAX=10000
VECTORIZER_SHARDING_PARALLEL_SEARCH=true
VECTORIZER_SHARDING_RERANK_ENABLED=false
```

---

## 9. API & Interfaces

### 🎯 **CRITICAL: Zero API Changes**

**All existing APIs remain 100% unchanged. Sharding is an internal implementation detail.**

### 9.1 Existing Search API (Unchanged)

```rust
// src/db/vector_store.rs

impl VectorStore {
    /// Search for similar vectors
    /// 
    /// **NO CHANGES TO THIS SIGNATURE**
    /// 
    /// Internally:
    /// - If sharding disabled: uses single index (current behavior)
    /// - If sharding enabled: routes to parallel_search_shards() transparently
    pub async fn search(
        &self,
        collection: &str,
        query: Vec<f32>,
        limit: usize,
    ) -> Result<Vec<SearchResult>> {
        let coll = self.get_collection(collection)?;
        
        // Internal routing based on configuration
        if coll.sharding_enabled() {
            // New path: parallel search + merge
            self.parallel_search_shards(coll, query, limit).await
        } else {
            // Legacy path: single index search
            coll.single_index_search(query, limit).await
        }
    }
    
    /// Insert vector
    /// 
    /// **NO CHANGES TO THIS SIGNATURE**
    /// 
    /// Internally:
    /// - If sharding disabled: inserts to single index
    /// - If sharding enabled: routes to appropriate shard
    pub async fn insert(
        &self,
        collection: &str,
        vector: Vector,
    ) -> Result<String> {
        let coll = self.get_collection(collection)?;
        
        if coll.sharding_enabled() {
            // New path: route to shard
            self.route_insert_to_shard(coll, vector).await
        } else {
            // Legacy path: single index insert
            coll.single_index_insert(vector).await
        }
    }
    
    /// All other methods remain unchanged:
    /// - update()
    /// - delete()
    /// - batch_insert()
    /// - get_vector()
    /// etc.
}
```

### 9.2 MCP Tools (Unchanged)

**All MCP tools maintain their exact parameter schemas:**

```typescript
// No changes to any MCP tool definitions

// search_vectors tool
{
  "name": "search_vectors",
  "parameters": {
    "collection": "string",      // Unchanged
    "query": "string",           // Unchanged
    "limit": "number"            // Unchanged
  }
}

// intelligent_search tool
{
  "name": "intelligent_search",
  "parameters": {
    "query": "string",           // Unchanged
    "collections": "string[]",   // Unchanged
    "max_results": "number"      // Unchanged
  }
}

// All 37 existing MCP tools remain unchanged
```

### 9.3 HTTP API (Unchanged)

```http
### Search endpoint (unchanged)
POST /api/v1/collections/{collection}/search
Content-Type: application/json

{
  "query": [0.1, 0.2, ...],
  "limit": 10
}

### Response format (unchanged)
{
  "results": [
    {
      "id": "doc1",
      "score": 0.95,
      "metadata": {...}
    }
  ],
  "took_ms": 2.5
}
```

### 9.4 SDK Compatibility

**All existing SDKs work without any changes:**

```python
# Python SDK - No changes required
from vectorizer import VectorizerClient

client = VectorizerClient("http://localhost:8080")

# Same API as before
results = client.search(
    collection="my-collection",
    query="search text",
    limit=10
)
```

```typescript
// TypeScript SDK - No changes required
import { VectorizerClient } from '@vectorizer/sdk';

const client = new VectorizerClient('http://localhost:8080');

// Same API as before
const results = await client.search({
  collection: 'my-collection',
  query: 'search text',
  limit: 10
});
```

### 9.5 New Internal Methods (Private)

```rust
// src/db/sharding.rs

impl VectorStore {
    /// Internal method for parallel shard search
    /// 
    /// **PRIVATE** - Not exposed to clients
    async fn parallel_search_shards(
        &self,
        collection: &Collection,
        query: Vec<f32>,
        limit: usize,
    ) -> Result<Vec<SearchResult>> {
        // Implementation as described in section 7.3
    }
    
    /// Internal method for insert routing
    /// 
    /// **PRIVATE** - Not exposed to clients
    async fn route_insert_to_shard(
        &self,
        collection: &Collection,
        vector: Vector,
    ) -> Result<String> {
        // Implementation as described in section 7.1
    }
    
    /// Internal method for auto-split
    /// 
    /// **PRIVATE** - Not exposed to clients
    async fn auto_split_shard(
        &self,
        shard: &mut Shard,
    ) -> Result<()> {
        // Implementation as described in section 7.2
    }
}
```

### 9.6 Observability Extensions (Optional)

**New optional diagnostic fields in responses (backward compatible):**

```json
{
  "results": [...],
  "took_ms": 2.5,
  
  // NEW: Optional diagnostics (only if sharding enabled)
  "diagnostics": {
    "sharding_enabled": true,
    "shards_queried": 5,
    "search_time_ms": 1.8,
    "merge_time_ms": 0.7,
    "results_before_dedup": 52,
    "results_after_dedup": 10
  }
}
```

**Clients that don't expect `diagnostics` will simply ignore it (backward compatible).**

---

## 10. Persistence & WAL

### 10.1 Directory Structure

```
data/
├── collections/
│   ├── my-collection/
│   │   ├── metadata.json           # Collection metadata
│   │   ├── sharding.json           # Sharding metadata
│   │   ├── shards/
│   │   │   ├── shard_0/
│   │   │   │   ├── vectors.bin     # Vector data
│   │   │   │   ├── index.bin       # HNSW index
│   │   │   │   └── wal/
│   │   │   │       ├── segment_000.wal
│   │   │   │       ├── segment_001.wal
│   │   │   │       └── checkpoint.json
│   │   │   ├── shard_1/
│   │   │   │   └── ...
│   │   │   └── shard_2/
│   │   │       └── ...
│   │   └── tree.json               # Quad-tree structure
```

### 10.2 Sharding Metadata

```json
{
  "collection": "my-collection",
  "sharding_enabled": true,
  "policy": {
    "target_max": 10000,
    "soft_limit_ratio": 0.85,
    "split_strategy": "Kmeans2",
    "routing": "MinSize"
  },
  "tree": {
    "type": "internal",
    "children": [
      {"type": "leaf", "shard_id": "shard_0"},
      {"type": "leaf", "shard_id": "shard_1"}
    ]
  },
  "created_at": "2025-10-07T10:00:00Z",
  "last_split": "2025-10-07T11:30:00Z"
}
```

### 10.3 WAL Entries

```rust
#[derive(Serialize, Deserialize)]
pub enum WalEntry {
    Insert {
        vector_id: String,
        timestamp: u64,
    },
    Delete {
        vector_id: String,
        timestamp: u64,
    },
    SplitStart {
        shard_id: String,
        size: usize,
        strategy: SplitStrategy,
    },
    SplitCommit {
        old_shard_id: String,
        new_shard_ids: Vec<String>,
    },
    MergeStart {
        shard_ids: Vec<String>,
        combined_size: usize,
    },
    MergeCommit {
        old_shard_ids: Vec<String>,
        new_shard_id: String,
    },
}
```

### 10.4 Recovery Process

```rust
async fn recover_collection(path: &Path) -> Result<Collection> {
    // 1. Load collection metadata
    let metadata = load_metadata(path.join("metadata.json"))?;
    
    // 2. Load sharding configuration
    let sharding = load_sharding(path.join("sharding.json"))?;
    
    // 3. Recover each shard from WAL
    let mut shards = vec![];
    for shard_id in sharding.list_shards() {
        let shard = recover_shard(&path.join("shards").join(shard_id)).await?;
        shards.push(shard);
    }
    
    // 4. Rebuild quad-tree structure
    let tree = rebuild_tree(&sharding, shards)?;
    
    Ok(Collection::new(metadata, tree))
}
```

---

## 11. Concurrency

### 11.1 Read-Write Locks

```rust
use tokio::sync::RwLock;

pub struct Shard {
    pub id: String,
    pub engine: RwLock<VectorEngine>,  // Read-write lock on engine
    pub size: AtomicUsize,             // Atomic counter
    pub stats: RwLock<ShardStats>,     // Stats under RwLock
}

impl Shard {
    /// Concurrent reads (multiple readers)
    pub async fn search(&self, query: &[f32], k: usize) -> Result<Vec<SearchResult>> {
        let engine = self.engine.read().await;  // Read lock
        engine.search(query, k)
    }
    
    /// Exclusive write (single writer)
    pub async fn insert(&self, vector: Vector) -> Result<()> {
        let mut engine = self.engine.write().await;  // Write lock
        engine.insert(vector)?;
        self.size.fetch_add(1, Ordering::Relaxed);
        Ok(())
    }
}
```

### 11.2 Parallel Search

```rust
use tokio::task::JoinSet;

async fn parallel_search_shards(
    shards: &[Arc<Shard>],
    query: &[f32],
    limit: usize,
) -> Result<Vec<SearchResult>> {
    let mut join_set = JoinSet::new();
    
    // Spawn parallel tasks
    for shard in shards.iter().cloned() {
        let query = query.to_vec();
        join_set.spawn(async move {
            shard.search(&query, limit).await
        });
    }
    
    // Collect results
    let mut all_results = vec![];
    while let Some(result) = join_set.join_next().await {
        all_results.extend(result??);
    }
    
    Ok(all_results)
}
```

### 11.3 Split Lock (Exclusive)

```rust
use tokio::sync::Mutex;

pub struct Collection {
    pub shards: RwLock<Vec<Arc<Shard>>>,
    pub split_lock: Mutex<()>,  // Exclusive lock for splits
}

impl Collection {
    /// Split requires exclusive lock to prevent concurrent splits
    pub async fn split_shard(&self, shard_id: &str) -> Result<()> {
        // Acquire exclusive split lock
        let _guard = self.split_lock.lock().await;
        
        // Perform split atomically
        let mut shards = self.shards.write().await;
        let shard_idx = shards.iter()
            .position(|s| s.id == shard_id)
            .ok_or(VectorizerError::ShardNotFound)?;
        
        let old_shard = &shards[shard_idx];
        let (new_shard1, new_shard2) = auto_split_shard(old_shard).await?;
        
        // Atomic replacement
        shards.remove(shard_idx);
        shards.push(Arc::new(new_shard1));
        shards.push(Arc::new(new_shard2));
        
        Ok(())
        // Lock released automatically on drop
    }
}
```

### 11.4 Deadlock Prevention

**Lock ordering rules:**
1. Always acquire `split_lock` before `shards` write lock
2. Never hold `split_lock` while waiting for shard `engine` lock
3. Background merge uses try-lock to avoid blocking

```rust
// GOOD: Correct lock ordering
let _split_guard = collection.split_lock.lock().await;
let mut shards = collection.shards.write().await;

// BAD: Deadlock risk (reversed order)
let mut shards = collection.shards.write().await;
let _split_guard = collection.split_lock.lock().await;  // DEADLOCK!
```

---

## 12. Telemetry & Observability

### 12.1 Metrics

```rust
use prometheus::{Counter, Histogram, Gauge};

lazy_static! {
    // Search metrics
    static ref SEARCH_REQUESTS: Counter = register_counter!(
        "vectorizer_search_requests_total",
        "Total search requests"
    ).unwrap();
    
    static ref SEARCH_LATENCY: Histogram = register_histogram!(
        "vectorizer_search_latency_seconds",
        "Search latency distribution"
    ).unwrap();
    
    static ref SHARDS_QUERIED: Histogram = register_histogram!(
        "vectorizer_shards_queried",
        "Number of shards queried per search"
    ).unwrap();
    
    // Shard metrics
    static ref SHARD_SIZE: Gauge = register_gauge!(
        "vectorizer_shard_size",
        "Current shard size in vectors"
    ).unwrap();
    
    static ref SPLIT_OPERATIONS: Counter = register_counter!(
        "vectorizer_split_operations_total",
        "Total shard splits"
    ).unwrap();
    
    static ref MERGE_OPERATIONS: Counter = register_counter!(
        "vectorizer_merge_operations_total",
        "Total shard merges"
    ).unwrap();
}
```

### 12.2 Structured Logging

```rust
use tracing::{info, debug, warn, error, instrument};

#[instrument(skip(self, query))]
pub async fn search(&self, query: &[f32], limit: usize) -> Result<Vec<SearchResult>> {
    let start = Instant::now();
    
    info!(
        collection = %self.name,
        limit = limit,
        shards = self.shards.len(),
        "Starting parallel search"
    );
    
    let results = self.parallel_search_shards(query, limit).await?;
    
    let duration = start.elapsed();
    info!(
        collection = %self.name,
        results = results.len(),
        duration_ms = duration.as_millis(),
        "Search completed"
    );
    
    SEARCH_REQUESTS.inc();
    SEARCH_LATENCY.observe(duration.as_secs_f64());
    
    Ok(results)
}
```

### 12.3 Health Checks

```rust
#[derive(Serialize)]
pub struct HealthStatus {
    pub healthy: bool,
    pub collections: Vec<CollectionHealth>,
}

#[derive(Serialize)]
pub struct CollectionHealth {
    pub name: String,
    pub sharding_enabled: bool,
    pub num_shards: usize,
    pub total_vectors: usize,
    pub avg_shard_size: f64,
    pub max_shard_size: usize,
    pub needs_rebalance: bool,
}

pub async fn health_check(&self) -> HealthStatus {
    let mut collections = vec![];
    
    for collection in self.list_collections() {
        let shards = collection.shards.read().await;
        let sizes: Vec<_> = shards.iter()
            .map(|s| s.size.load(Ordering::Relaxed))
            .collect();
        
        let total = sizes.iter().sum::<usize>();
        let avg = total as f64 / sizes.len() as f64;
        let max = sizes.iter().max().copied().unwrap_or(0);
        
        // Flag if any shard is over 90% of target_max
        let needs_rebalance = max > (collection.policy.target_max as f64 * 0.9) as usize;
        
        collections.push(CollectionHealth {
            name: collection.name.clone(),
            sharding_enabled: collection.sharding_enabled,
            num_shards: shards.len(),
            total_vectors: total,
            avg_shard_size: avg,
            max_shard_size: max,
            needs_rebalance,
        });
    }
    
    let healthy = collections.iter().all(|c| !c.needs_rebalance);
    
    HealthStatus { healthy, collections }
}
```

---

## 13. Migration & Backfill

### 13.1 Migration Strategy

**Approach: Gradual opt-in per collection**

```rust
pub async fn enable_sharding(
    &self,
    collection: &str,
    policy: Option<ShardingPolicy>,
) -> Result<()> {
    let coll = self.get_collection(collection)?;
    
    // 1. Check if already sharded
    if coll.sharding_enabled {
        return Err(VectorizerError::AlreadySharded);
    }
    
    // 2. Load current single index
    let single_index = coll.load_single_index().await?;
    let vectors = single_index.get_all_vectors();
    
    info!(
        collection = %collection,
        num_vectors = vectors.len(),
        "Starting migration to sharded index"
    );
    
    // 3. Create initial shard structure
    let policy = policy.unwrap_or_default();
    let num_initial_shards = (vectors.len() / policy.target_max).max(1);
    
    // 4. Distribute vectors across shards using k-means
    let shard_assignments = if num_initial_shards > 1 {
        kmeans_partition(&vectors, num_initial_shards)?
    } else {
        vec![vectors]  // Single shard
    };
    
    // 5. Build shards
    let mut shards = vec![];
    for (i, shard_vectors) in shard_assignments.into_iter().enumerate() {
        let shard = Shard::new_with_vectors(&format!("shard_{}", i), shard_vectors).await?;
        shards.push(Arc::new(shard));
    }
    
    // 6. Build quad-tree
    let tree = Node::new_with_shards(shards);
    
    // 7. Atomically swap to sharded structure
    coll.enable_sharding(tree, policy).await?;
    
    // 8. Persist sharding metadata
    coll.save_sharding_metadata().await?;
    
    // 9. Archive old single index
    coll.archive_single_index().await?;
    
    info!(
        collection = %collection,
        num_shards = num_initial_shards,
        "Migration completed successfully"
    );
    
    Ok(())
}
```

### 13.2 Rollback Procedure

```rust
pub async fn disable_sharding(&self, collection: &str) -> Result<()> {
    let coll = self.get_collection(collection)?;
    
    if !coll.sharding_enabled {
        return Err(VectorizerError::NotSharded);
    }
    
    warn!(
        collection = %collection,
        "Rolling back to single index"
    );
    
    // 1. Collect all vectors from all shards
    let shards = coll.shards.read().await;
    let mut all_vectors = vec![];
    for shard in shards.iter() {
        all_vectors.extend(shard.engine.read().await.get_all_vectors());
    }
    
    // 2. Build single index
    let single_index = VectorEngine::new_with_vectors(all_vectors).await?;
    
    // 3. Atomically swap back to single index
    coll.disable_sharding(single_index).await?;
    
    // 4. Remove sharding metadata
    coll.remove_sharding_metadata().await?;
    
    Ok(())
}
```

### 13.3 Zero-Downtime Migration

```rust
pub async fn migrate_with_dual_writes(&self, collection: &str) -> Result<()> {
    // Phase 1: Build sharded index in background (reads still from single index)
    self.build_sharded_index_background(collection).await?;
    
    // Phase 2: Dual-write mode (writes go to both indexes)
    self.enable_dual_write_mode(collection).await?;
    
    // Phase 3: Verify consistency
    self.verify_index_consistency(collection).await?;
    
    // Phase 4: Switch reads to sharded index
    self.switch_reads_to_sharded(collection).await?;
    
    // Phase 5: Remove single index
    self.remove_single_index(collection).await?;
    
    Ok(())
}
```

---

## 14. Testing & Benchmarking

### 14.1 Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_auto_split() {
        let mut shard = Shard::new("test_shard").await.unwrap();
        
        // Insert 10k vectors
        for i in 0..10_000 {
            let vector = generate_random_vector(384);
            shard.insert(vector).await.unwrap();
        }
        
        let policy = SplitPolicy {
            target_max: 5_000,
            soft_limit_ratio: 0.85,
            ..Default::default()
        };
        
        // Trigger split
        let (shard1, shard2) = auto_split_shard(&mut shard, &policy).await.unwrap();
        
        assert!(shard1.size.load(Ordering::Relaxed) < 6_000);
        assert!(shard2.size.load(Ordering::Relaxed) < 6_000);
        assert_eq!(
            shard1.size.load(Ordering::Relaxed) + shard2.size.load(Ordering::Relaxed),
            10_000
        );
    }
    
    #[tokio::test]
    async fn test_parallel_search_quality() {
        // Create collection with 5 shards
        let collection = create_test_collection(5, 2_000).await;
        
        let query = generate_random_vector(384);
        let k = 10;
        
        // Search with sharding
        let sharded_results = collection.search(&query, k).await.unwrap();
        
        // Search with single index (ground truth)
        let single_index = merge_all_shards(&collection).await;
        let truth_results = single_index.search(&query, k).await.unwrap();
        
        // Compare results (should be identical or very close)
        let overlap = compute_overlap(&sharded_results, &truth_results);
        assert!(overlap > 0.95, "Search quality degraded: overlap={}", overlap);
    }
    
    #[tokio::test]
    async fn test_concurrent_insert_and_search() {
        let collection = Arc::new(create_test_collection(3, 1_000).await);
        
        let mut handles = vec![];
        
        // Spawn 10 concurrent inserters
        for _ in 0..10 {
            let coll = collection.clone();
            handles.push(tokio::spawn(async move {
                for _ in 0..100 {
                    let vector = generate_random_vector(384);
                    coll.insert(vector).await.unwrap();
                }
            }));
        }
        
        // Spawn 10 concurrent searchers
        for _ in 0..10 {
            let coll = collection.clone();
            handles.push(tokio::spawn(async move {
                for _ in 0..100 {
                    let query = generate_random_vector(384);
                    coll.search(&query, 10).await.unwrap();
                }
            }));
        }
        
        // Wait for all tasks
        for handle in handles {
            handle.await.unwrap();
        }
        
        // Verify no deadlocks or data corruption
        let health = collection.health_check().await;
        assert!(health.healthy);
    }
}
```

### 14.2 Benchmarks

```rust
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn benchmark_search_scaling(c: &mut Criterion) {
    let runtime = tokio::runtime::Runtime::new().unwrap();
    
    let mut group = c.benchmark_group("search_scaling");
    
    for num_shards in [1, 2, 5, 10, 20] {
        let collection = runtime.block_on(async {
            create_test_collection(num_shards, 10_000).await
        });
        
        group.bench_function(format!("shards_{}", num_shards), |b| {
            b.to_async(&runtime).iter(|| async {
                let query = generate_random_vector(384);
                collection.search(black_box(&query), 10).await.unwrap()
            });
        });
    }
    
    group.finish();
}

fn benchmark_insert_routing(c: &mut Criterion) {
    let runtime = tokio::runtime::Runtime::new().unwrap();
    
    let collection = runtime.block_on(async {
        create_test_collection(10, 5_000).await
    });
    
    c.bench_function("insert_with_routing", |b| {
        b.to_async(&runtime).iter(|| async {
            let vector = generate_random_vector(384);
            collection.insert(black_box(vector)).await.unwrap()
        });
    });
}

criterion_group!(benches, benchmark_search_scaling, benchmark_insert_routing);
criterion_main!(benches);
```

### 14.3 Load Tests

```bash
# Load test with K6
k6 run --vus 100 --duration 5m load_test.js

# Target metrics:
# - 10,000 QPS sustained
# - p99 latency < 5ms
# - 0 errors
# - No memory leaks
```

---

## 15. Risks & Mitigation

### 15.1 Identified Risks

| Risk | Severity | Mitigation |
|------|----------|------------|
| Search quality degradation due to sharding | **HIGH** | Comprehensive testing comparing sharded vs single index results. Ensure overlap > 95%. |
| Increased latency from merge overhead | **MEDIUM** | Optimize k-way merge with min-heap. Benchmark target: < 1ms merge time for k=10. |
| Split operations blocking writes | **MEDIUM** | Use background split with copy-on-write. Soft limit triggers split before hard limit. |
| Memory overhead from multiple indexes | **MEDIUM** | Monitor memory usage per shard. Implement shared memory for embeddings model. |
| Complexity in debugging distributed searches | **LOW** | Rich telemetry and diagnostics. Structured logging with trace IDs. |
| Migration risks for existing collections | **HIGH** | Gradual rollout. Dual-write mode. Easy rollback procedure. Comprehensive testing. |

### 15.2 Mitigation Strategies

#### Risk: Search Quality Degradation

**Mitigation:**
```rust
#[cfg(test)]
async fn validate_search_quality() {
    let collection = create_large_collection(10_shards, 100k_vectors).await;
    
    for _ in 0..1000 {
        let query = generate_random_vector(384);
        let sharded = collection.search(&query, 10).await.unwrap();
        let truth = get_ground_truth(&query, 10).await.unwrap();
        
        let overlap = compute_recall(&sharded, &truth);
        assert!(overlap >= 0.95, "Quality threshold violated");
    }
}
```

#### Risk: Increased Latency

**Mitigation:**
- Profile merge operation
- Optimize with SIMD instructions
- Consider GPU acceleration for reranking
- Cache frequently accessed vectors

#### Risk: Migration Failures

**Mitigation:**
- Atomic operations with WAL
- Checkpoint before/after migration
- Automated rollback on failure
- Dry-run mode for validation

---

## 16. Acceptance Criteria

### 16.1 Functional Criteria

- ✅ **FC-1**: Collections can be configured with sharding enabled/disabled
- ✅ **FC-2**: Auto-split triggers at soft_limit (85% of target_max)
- ✅ **FC-3**: Search results are identical to single-index (> 95% recall)
- ✅ **FC-4**: Parallel search queries all shards concurrently
- ✅ **FC-5**: K-way merge with deduplication produces correct top-K
- ✅ **FC-6**: Background merge consolidates small shards
- ✅ **FC-7**: WAL persists all split/merge operations
- ✅ **FC-8**: Recovery rebuilds quad-tree from WAL

### 16.2 Non-Functional Criteria

- ✅ **NFC-1**: Search latency < 3ms p99 (10 shards, 100k vectors/shard)
- ✅ **NFC-2**: Search quality >= 95% recall vs single index
- ✅ **NFC-3**: Insert throughput >= 10k ops/sec
- ✅ **NFC-4**: Memory overhead < 10% vs single index
- ✅ **NFC-5**: Split operation completes in < 1 second
- ✅ **NFC-6**: Zero downtime during migration
- ✅ **NFC-7**: API backward compatibility 100%

### 16.3 Testing Criteria

- ✅ **TC-1**: Unit tests cover all algorithms (insert, search, split, merge)
- ✅ **TC-2**: Integration tests validate end-to-end workflows
- ✅ **TC-3**: Concurrency tests pass with 100 concurrent threads
- ✅ **TC-4**: Load tests sustain 10k QPS for 1 hour
- ✅ **TC-5**: Benchmarks show linear scaling up to 10 shards
- ✅ **TC-6**: Chaos tests (random failures, crashes) validate recovery

---

## 17. Delivery Checklist

### 17.1 Implementation Phase

- [ ] **Create feature branch**: `feat/multi-index-quadtree-sharding`
- [ ] **Core data structures**: `Node`, `Shard`, `ShardingPolicy`
- [ ] **Insert routing**: Implement `route_insert()` with configurable strategies
- [ ] **Auto-split**: Implement `auto_split_shard()` with k-means
- [ ] **Parallel search**: Implement `parallel_search_shards()`
- [ ] **K-way merge**: Implement heap-based merge with deduplication
- [ ] **Optional reranking**: Integrate ONNX cross-encoder model
- [ ] **Persistence**: WAL segments per shard
- [ ] **Configuration**: TOML parsing and per-collection overrides
- [ ] **Telemetry**: Prometheus metrics and structured logging

### 17.2 Testing Phase

- [ ] **Unit tests**: 100% coverage for algorithms
- [ ] **Integration tests**: End-to-end workflows
- [ ] **Concurrency tests**: Race condition detection
- [ ] **Benchmarks**: Performance validation
- [ ] **Load tests**: Sustained high QPS
- [ ] **Chaos tests**: Failure recovery

### 17.3 Documentation Phase

- [ ] **Architecture docs**: This document
- [ ] **API docs**: Rustdoc for all public types
- [ ] **Migration guide**: Step-by-step instructions
- [ ] **Configuration reference**: All TOML options
- [ ] **Runbook**: Operational procedures

### 17.4 Deployment Phase

- [ ] **Staging deployment**: Validate in staging environment
- [ ] **Canary rollout**: Enable for 1-2 test collections
- [ ] **Monitor metrics**: Latency, QPS, error rates
- [ ] **Gradual rollout**: Enable for more collections
- [ ] **Production deployment**: Full release

---

## 18. Implementation Plan

### 18.1 Suggested Commit Messages

```bash
# Phase 1: Data Structures
git commit -m "feat(sharding): Add core data structures (Node, Shard, ShardingPolicy)"

# Phase 2: Insert Routing
git commit -m "feat(sharding): Implement insert routing with configurable strategies"

# Phase 3: Auto-Split
git commit -m "feat(sharding): Implement auto-split with k-means clustering"

# Phase 4: Parallel Search
git commit -m "feat(sharding): Implement parallel search across shards"

# Phase 5: K-Way Merge
git commit -m "feat(sharding): Implement k-way merge with heap and deduplication"

# Phase 6: Reranking (Optional)
git commit -m "feat(sharding): Add optional ONNX cross-encoder reranking"

# Phase 7: Persistence
git commit -m "feat(sharding): Add WAL persistence for split/merge operations"

# Phase 8: Configuration
git commit -m "feat(sharding): Add TOML configuration and per-collection overrides"

# Phase 9: Telemetry
git commit -m "feat(sharding): Add Prometheus metrics and structured logging"

# Phase 10: Testing
git commit -m "test(sharding): Add comprehensive test suite and benchmarks"

# Phase 11: Documentation
git commit -m "docs(sharding): Add architecture docs and migration guide"

# Phase 12: API Integration
git commit -m "feat(sharding): Integrate sharding into VectorStore API (transparent)"
```

### 18.2 Estimated Timeline

| Phase | Duration | Dependencies |
|-------|----------|--------------|
| 1. Data Structures | 2 days | None |
| 2. Insert Routing | 2 days | Phase 1 |
| 3. Auto-Split | 3 days | Phase 1, 2 |
| 4. Parallel Search | 3 days | Phase 1 |
| 5. K-Way Merge | 2 days | Phase 4 |
| 6. Reranking (Optional) | 3 days | Phase 5 |
| 7. Persistence | 3 days | Phase 3 |
| 8. Configuration | 2 days | All above |
| 9. Telemetry | 2 days | All above |
| 10. Testing | 5 days | All above |
| 11. Documentation | 3 days | All above |
| 12. API Integration | 2 days | All above |
| **Total** | **~30 days** (~6 weeks) | - |

### 18.3 Success Metrics

**Week 1-2: Foundation**
- Core data structures implemented
- Insert routing working
- Unit tests passing

**Week 3-4: Search & Split**
- Parallel search operational
- Auto-split functional
- Integration tests passing

**Week 5: Optimization**
- K-way merge optimized
- Telemetry integrated
- Benchmarks show improvement

**Week 6: Production Ready**
- All tests passing
- Documentation complete
- Ready for staging deployment

---

## 19. Conclusion

This RFC proposes a comprehensive solution for scaling Vectorizer collections to millions of vectors through Multi-Index Quad-Tree Sharding. The implementation:

- **Maintains 100% API compatibility** - zero changes required for clients
- **Scales linearly** - sub-3ms latency even with millions of vectors
- **Preserves search quality** - >= 95% recall compared to single index
- **Provides operational flexibility** - opt-in per collection, easy rollback
- **Ensures reliability** - WAL persistence, crash recovery, health checks

**Next Steps:**
1. Review and approve this RFC
2. Create feature branch
3. Begin implementation following the suggested commit plan
4. Iterative testing and benchmarking
5. Gradual production rollout

---

**Questions? Feedback?**

Please open a discussion on the feature branch or comment on the related GitHub issue.

**Author**: Vectorizer Team  
**Date**: October 7, 2025  
**Status**: RFC - Awaiting Approval
