# Sharding — Specification

> **BETA status.** Per the project CLAUDE.md, `src/cluster/` and the sharded
> collection paths that depend on it are marked **BETA**. Distributed sharding
> requires cluster mode; interfaces, on-wire formats, and operational defaults
> may change without a deprecation window.

This document describes how Vectorizer distributes a single collection across
multiple shards, how vectors are routed, how rebalancing works, and the public
API surface that implements it. It is the **developer/operator reference** for
sharding; see [`docs/deployment/CLUSTER.md`](../deployment/CLUSTER.md) for the
cluster install guide and [`docs/users/api/REPLICATION.md`](../users/api/REPLICATION.md)
for master/replica replication (a different feature).

---

## Overview

Vectorizer has **two independent mechanisms** for horizontal scale:

| Feature        | Purpose                                                     | Status |
|----------------|-------------------------------------------------------------|--------|
| Replication    | Full copies of the dataset for read fan-out / HA            | GA (leader/follower) |
| Sharding       | Split one collection across N shards (and optionally N nodes)| BETA   |

Sharding is used when a single collection is too large to fit in one
process's RAM, when HNSW build time on one node becomes prohibitive, or when
throughput for writes/searches must be fanned out across shards in parallel.
It is **not** a replacement for replication: a sharded collection on a single
node has zero redundancy against process death; a sharded collection on a
cluster has no redundancy either unless cluster-level replication is enabled
on top.

Two concrete backends exist:

* **`ShardedCollection`** (`src/db/sharded_collection.rs`) — *local* sharding.
  One process, N shards, each shard is a regular `Collection`. Gives
  intra-process parallelism and lets HNSW structures stay small per shard.
* **`DistributedShardedCollection`** (`src/db/distributed_sharded_collection.rs`)
  — *distributed* sharding. Shards are spread across cluster nodes via
  `cluster::DistributedShardRouter`; cross-node operations go over gRPC
  through `ClusterClientPool`.

Both are selected automatically from `CollectionConfig.sharding` by
`VectorStore` and are dispatched through the `CollectionType` enum
(`src/db/vector_store/collection_type.rs`).

---

## Architecture

```
                   CollectionType (enum)
         ┌────────────┬───────────┬────────────────┐
         │            │           │                │
      Cpu          HiveGpu     Sharded       DistributedSharded
                    (GPU)   (local N shards)  (cluster N shards × N nodes)
                             │                   │
                             ▼                   ▼
                       ShardRouter      DistributedShardRouter
                         │                   │
                         │ consistent hash   │ consistent hash
                         │ ring (in-proc)    │ ring (cluster-wide)
                         │                   │
                         ▼                   ▼
                    shard → Collection  shard → (NodeId, local Collection
                                                 or gRPC stub)
```

### Local — `ShardedCollection`

* Holds a `DashMap<ShardId, Collection>` of regular per-shard collections.
* Uses a single `Arc<ShardRouter>` (in-process `ConsistentHashRing`) for
  routing and rebalancing decisions.
* Methods are **synchronous**.
* Per-shard collections are named `"{name}_{shard_id}"` and have their own
  `sharding = None` (shards themselves are never recursively sharded).

### Distributed — `DistributedShardedCollection`

* Holds a `HashMap<ShardId, Collection>` of **local** shards (only those
  assigned to this node by the router) under a `parking_lot::RwLock`.
* Uses `cluster::DistributedShardRouter` whose ring maps each position to a
  `(ShardId, NodeId)` pair.
* Every CRUD method is **async** because a shard may live on a remote node
  and needs gRPC.
* `ClusterClientPool` is consulted for a `ClusterClient` per target node.
* Caches collection-wide `vector_count` for 5 s
  (`vector_count_cache_ttl = Duration::from_secs(5)`) because a cluster-wide
  count requires N gRPC round trips.

### `ShardRouter` — local

Source: `src/db/sharding.rs` (`pub struct ShardRouter`).

* Wraps an `Arc<RwLock<ConsistentHashRing>>` + collection name.
* Creates virtual nodes; default is `100` per shard
  (`ShardRouter::new(..., shard_count)` hard-codes 100 vnodes).
* Exposes `route_vector(&str) -> ShardId` and
  `route_search(Option<&[ShardId]>) -> Vec<ShardId>`.

### `DistributedShardRouter` — cluster-wide

Source: `src/cluster/shard_router.rs`.

* Same conceptual ring but entries are `(ShardId, NodeId)`.
* Tracks an **epoch per assignment** (`shard_epochs: HashMap<ShardId,u64>`)
  and a global `current_epoch`. Every `assign_shard` bumps `current_epoch`
  atomically before mutating the ring, so concurrent assignments get
  strictly increasing epochs. `apply_if_higher_epoch` is used by
  `state_sync` to reject stale updates.
* Supports tenant-aware routing (multi-tenant cluster mode): see
  `get_shard_for_tenant_vector`, `get_node_for_tenant_vector`,
  `get_shard_for_tenant`, `get_shards_for_tenant`.

### `ShardRebalancer`

Source: `src/db/sharding.rs`.

* Wraps an `Arc<ShardRouter>` + `threshold: f32`.
* Exposes `needs_rebalancing`, `calculate_moves_for_add`,
  `calculate_moves_for_remove`, `calculate_balance_moves`.
* **Not wired to an automatic trigger today** — callers decide when to run
  it. See **Rebalancing** below.

---

## Vector routing

### Local (`ConsistentHashRing`)

Implementation: `src/db/sharding.rs`.

* **Algorithm:** consistent hashing with virtual nodes.
* **Hash function:** `std::collections::hash_map::DefaultHasher` (SipHash-1-3
  internally; *not* cross-platform guaranteed, but stable within one
  process lifetime).
* **Vector placement:** `hash_vector_id(vector_id)` → first ring entry with
  hash ≥ vector's hash (wrap around if none).
* **Virtual nodes:** `ShardRouter::new` uses **100** vnodes per shard
  unconditionally. `ShardingConfig.virtual_nodes_per_shard` (see
  [Configuration](#operations)) is plumbed through the config but is
  *only* honored by the distributed router — the local router ignores it
  at construction. **This is a known mismatch.**

### Distributed (`DistributedShardRouter`)

Implementation: `src/cluster/shard_router.rs`.

* **Algorithm:** same consistent-hash-with-vnodes pattern.
* **Hash function:** `xxhash_rust::xxh3::xxh3_64`. Chosen explicitly for
  *cross-platform determinism* so heterogeneous clusters route the same
  vector to the same shard. The comment in `hash_shard_vnode` calls this
  out.
* **Vector → Shard:** same circular lookup rule.
* **Shard → Node:** `shard_to_node: HashMap<ShardId, NodeId>` cached for
  O(1) reads (`get_node_for_shard`), kept consistent with the ring on
  every mutation.
* **Tenant-scoped routing:** the tenant id is mixed into the hash with a
  `0xFF` separator byte (`hash_tenant_vector`) to avoid
  concatenation-collision classes like `("ab","c")` vs `("a","bc")`.
  `get_shards_for_tenant(tenant, n)` deterministically derives `n` distinct
  shards for a given tenant by re-hashing with an index suffix
  (`hash_tenant_shard`).

> **Correctness note.** Because the two routers use different hash
> functions (SipHash vs xxh3), a vector routed locally and a vector
> routed in a cluster will not, in general, map to the same shard index.
> This is only observable if you migrate data from a single-node sharded
> deployment to a cluster — plan a full re-index in that case.

---

## Rebalancing

### Local — `ShardRebalancer`

`needs_rebalancing(counts: &HashMap<ShardId, usize>) -> bool`

* Computes `avg = total / shard_count`.
* Returns `true` if either `(max - avg) / avg > threshold` **or**
  `(avg - min) / avg > threshold`.
* Default threshold is `rebalance_threshold = 0.2` (20%), from
  `ShardingConfig::default()`.
* Empty or all-zero shard maps return `false`.

`calculate_moves_for_add` / `calculate_moves_for_remove`

* Return the list of `(vector_id, data, new_shard)` that need to move after
  a topology change.
* **Note (source-backed):** the current `calculate_moves_for_add`
  implementation computes `old_shard` and `new_shard` by calling
  `router.route_vector` twice on *the same* router — it does not compare
  against a pre-change router. In practice this means the returned move
  list is only meaningful when callers swap the router's ring between the
  two calls (tests today exercise `needs_rebalancing` and
  `calculate_balance_moves` but not this path). TBD: confirm whether this
  is intentional scaffolding for a planned two-phase rebalance or a bug to
  fix before GA.

`calculate_balance_moves(vectors, shard_counts) -> Vec<(id, data, from, to)>`

* Classifies shards as overloaded / underloaded using the same threshold,
  sorts overloaded descending and underloaded ascending by count, and
  walks the input `vectors` emitting `(id, data, src, dst)` tuples.
* The returned list is a **plan**; nothing is moved by the rebalancer
  itself.

### Distributed — `DistributedShardRouter::rebalance` / `calculate_migration_plan`

* `rebalance(shard_ids, node_ids)` removes any shards whose node is no
  longer in `node_ids` and then does **round-robin** assignment
  (`shard[i] → node[i % node_ids.len()]`).
* `calculate_migration_plan(shard_ids, node_ids)` targets
  `shard_ids.len() / node_ids.len()` shards per node and returns a
  `Vec<(ShardId, from_node, to_node)>` migration plan. Data is *not* moved
  by this function — actual movement is performed by
  `cluster::ShardMigrator::migrate_shard_data` over gRPC
  (`GetShardVectors` on source, `RemoteInsertVector` on target), with
  progress surfaced through `MigrationProgress` /
  `ShardMigrator::list_migrations`.

### When does rebalancing run?

* **Automatic trigger:** only during `DistributedShardedCollection::new`,
  which calls `shard_router.rebalance(&shard_ids, &node_ids)` once when
  the collection is created and nodes are known.
* **Manual trigger:** via the cluster REST endpoint
  `POST /api/v1/cluster/rebalance`
  (`crates/vectorizer-server/src/api/cluster.rs`). This re-runs
  `shard_router.rebalance` over all active nodes and returns the new
  distribution.
* **On node add/remove:** *no automatic rebalance today*. Operators must
  call the endpoint after topology changes. TBD: verify whether
  `cluster::state_sync` triggers a rebalance on membership change — not
  observed in the router sources.

### Data movement guarantees during rebalance

* **Writes are NOT blocked.** `DistributedShardRouter::assign_shard`
  increments the epoch and atomically rewrites ring entries under write
  locks, but concurrent `insert`/`update`/`delete` calls use read locks
  and route to whichever shard→node mapping exists at the instant of
  lookup.
* **Migration is read-copy, not move.** `ShardMigrator` reads in 500-vector
  batches (`DEFAULT_BATCH_SIZE`) from source via gRPC `GetShardVectors`,
  inserts into target via `RemoteInsertVector`, and finally the caller is
  expected to update the shard→node mapping. Until the mapping flip,
  reads still go to the source; after the flip, reads go to the target.
* **Consistency window.** Any write that lands on the old node between
  the last batch read and the mapping flip will be silently lost unless
  the caller serializes traffic externally. This is why the current
  migration flow is considered BETA.
* **Failure modes.** `MigrationError` variants: `SourceCollection`,
  `TargetInsert { id, reason }`, `Transport(String)`, `NotFound`,
  `Cancelled`, or a wrapped `VectorizerError`. Partial progress is
  preserved in `MigrationProgress.vectors_transferred` so operators can
  inspect how far it got before retrying.

---

## Operations

### Local sharding

**Enable by setting `sharding` on `CollectionConfig`:**

```yaml
# config.yml, per-collection
sharding:
  shard_count: 4
  virtual_nodes_per_shard: 100   # distributed only; see routing note
  rebalance_threshold: 0.2       # 20% deviation triggers needs_rebalancing
```

Defaults (`models::ShardingConfig::default`):

| Field                     | Default |
|---------------------------|---------|
| `shard_count`             | 4       |
| `virtual_nodes_per_shard` | 100     |
| `rebalance_threshold`     | 0.2     |

**Heuristics for `shard_count` (local):**

* One shard ≈ one HNSW index. Size each shard so that
  `m × vector_count × dim × 4 bytes` stays inside CPU cache-friendly and
  within ~1 GB of working set, which typically means `shard_count ≈ ⌈
  total_vectors / 5M ⌉` for 768-dim dense vectors — TBD, no benchmark
  doc currently validates this number.
* `shard_count` greater than the CPU core count is rarely useful locally
  — shard-parallel search is CPU-bound (see `ShardedCollection::search`,
  which iterates shards serially today; intra-process parallelism lives
  inside each shard's HNSW).
* `rebalance_threshold` below `0.1` will flap on workloads with highly
  skewed ID distributions.

### Distributed sharding

**Requires cluster mode.** See `docs/deployment/CLUSTER.md` for the
cluster bring-up; this section only describes what changes when the
collection is sharded.

* Minimum cluster: the collection's constructor requires
  `cluster_manager.get_active_nodes()` to be non-empty. Any node in the
  cluster can host shards.
* Shard assignment on creation is round-robin over active nodes at that
  moment (`DistributedShardRouter::rebalance` invoked from
  `DistributedShardedCollection::new`). This means creating a sharded
  collection before a node joins yields an uneven layout until the
  operator calls `/api/v1/cluster/rebalance`.
* Once created, the local node materializes a `Collection` for each shard
  whose node equals `cluster_manager.local_node_id()`; all other shards
  are remote.
* The local node retains the same `sharding` settings in
  `CollectionConfig`, but the per-shard `Collection` instances are
  created with `sharding = None` — shards are never recursively sharded.

**Creating a distributed collection over REST:**

```bash
curl -X POST "http://$NODE:15002/collections" \
  -H "Content-Type: application/json" \
  -d '{
    "name": "distributed-collection",
    "dimension": 512,
    "metric": "cosine",
    "sharding": {
      "shard_count": 6,
      "virtual_nodes_per_shard": 100,
      "rebalance_threshold": 0.2
    }
  }'
```

---

## API surface

API is grouped by operation so the 100+ public functions across four
source files remain browsable. Each row names the owning class.

### Administration / lifecycle

| Operation                         | Class                                   | Function |
|-----------------------------------|-----------------------------------------|----------|
| Construct from config             | `ShardedCollection`                     | `new(name, config)` |
| Construct distributed             | `DistributedShardedCollection`          | `new(name, config, cluster_manager, client_pool)` |
| Build router                      | `ShardRouter`                           | `new(collection_name, shard_count)` |
| Build distributed router          | `DistributedShardRouter`                | `new(vnodes)`, `with_epoch(vnodes, initial_epoch)` |
| Build ring directly               | `ConsistentHashRing`                    | `new(shard_count, virtual_nodes_per_shard)` |
| Build rebalancer                  | `ShardRebalancer`                       | `new(router, threshold)` |
| Add a shard                       | `ShardedCollection`, `ShardRouter`, `ConsistentHashRing` | `add_shard(shard_id, weight)` |
| Remove a shard                    | `ShardedCollection`, `ShardRouter`, `ConsistentHashRing`, `DistributedShardRouter` | `remove_shard(shard_id)` |
| Assign shard to node              | `DistributedShardRouter`                | `assign_shard(shard, node) -> epoch` |
| Apply only if newer epoch         | `DistributedShardRouter`                | `apply_if_higher_epoch(...)` |
| Migrate assignment                | `DistributedShardRouter`                | `migrate_shard(shard, from, to)` |
| Compute migration plan            | `DistributedShardRouter`                | `calculate_migration_plan(shards, nodes)` |
| Move bytes between nodes          | `cluster::ShardMigrator`                | `migrate_shard_data(shard, from, to, collection)` |
| List / query migrations           | `cluster::ShardMigrator`                | `list_migrations`, `get_migration` |
| Rebalance (round-robin)           | `DistributedShardRouter`                | `rebalance(shards, nodes)` |
| REST trigger                      | HTTP                                    | `POST /api/v1/cluster/rebalance` |
| REST distribution dump            | HTTP                                    | `GET  /api/v1/cluster/shard-distribution` |
| Qdrant-compatible admin           | HTTP                                    | `create_shard_key`, `delete_shard_key`, `list_shard_keys` (`vectorizer-server/src/server/qdrant/sharding_handlers.rs`) |

### Vector CRUD

| Operation               | Class                          | Function | Notes |
|-------------------------|--------------------------------|----------|-------|
| Insert one              | `ShardedCollection`            | `insert(vector)` | sync |
| Insert one              | `DistributedShardedCollection` | `insert(vector)` | **async**, local or gRPC |
| Insert batch            | `ShardedCollection`            | `insert_batch(vectors)` | groups by shard |
| Insert batch            | `DistributedShardedCollection` | `insert_batch(vectors)` | groups by shard, then by node; remote side chunked into 100-vector sub-batches |
| Update one              | `ShardedCollection`            | `update(vector)` | sync |
| Update one              | `DistributedShardedCollection` | `update(vector)` | async |
| Delete one              | `ShardedCollection`            | `delete(id)` | sync |
| Delete one              | `DistributedShardedCollection` | `delete(id)` | async |
| Get one by id           | `ShardedCollection`            | `get_vector(id)` | sync |
| Get one by id           | `DistributedShardedCollection` | — | **not supported**; `CollectionType::get_vector` returns an error pointing callers to the async cluster router |
| Requantize all shards   | `ShardedCollection`            | `requantize_existing_vectors()` | iterates all shards |
| Requantize local shards | `DistributedShardedCollection` | `requantize_existing_vectors()` | local only; remote shards must be triggered on their nodes |

### Search

| Operation                  | Class                          | Function | Notes |
|----------------------------|--------------------------------|----------|-------|
| Dense search               | `ShardedCollection`            | `search(query, k, shard_keys)` | serial over shards, full sort, truncate k |
| Dense search               | `DistributedShardedCollection` | `search(query, k, threshold, shard_keys)` | async; locals in-process, remotes via `search_vectors` RPC; merge uses `select_nth_unstable_by` for top-k |
| Hybrid (dense + sparse)    | `ShardedCollection`            | `hybrid_search(dense, sparse, config, shard_keys)` | serial over shards |
| Hybrid (dense + sparse)    | `DistributedShardedCollection` | `hybrid_search(dense, sparse, config, shard_keys)` | async; falls back to dense RPC if a remote returns `Unimplemented` |
| Hybrid on local shards only| `DistributedShardedCollection` | `hybrid_search_local_only(...)` | used by cluster gRPC handler to avoid recursion |
| Pick shards for search     | `ShardRouter`, `ConsistentHashRing` | `route_search(shard_keys)` / `get_shards_for_search(shard_keys)` | `None` = all active shards |

### Routing / introspection

| Query                            | Class                          | Function |
|----------------------------------|--------------------------------|----------|
| Vector id → shard                | `ShardRouter`, `DistributedShardRouter`, `ConsistentHashRing` | `route_vector`, `get_shard_for_vector` |
| Vector id → node                 | `DistributedShardRouter`       | `get_node_for_vector` |
| Shard → node                     | `DistributedShardRouter`       | `get_node_for_shard` |
| Node → shard list                | `DistributedShardRouter`       | `get_shards_for_node` |
| All shards                       | `ShardRouter`, `DistributedShardRouter`, `ConsistentHashRing` | `get_shard_ids`, `get_all_shards` |
| All nodes                        | `DistributedShardRouter`       | `get_nodes` |
| Shard metadata (weight, count…)  | `ShardRouter`, `ConsistentHashRing` | `get_shard_metadata`, `update_shard_count`, `shard_count` |
| Shard epoch                      | `DistributedShardRouter`       | `get_shard_epoch`, `get_all_shard_epochs`, `current_epoch` |
| Tenant-scoped routing            | `DistributedShardRouter`       | `get_shard_for_tenant`, `get_shard_for_tenant_vector`, `get_node_for_tenant`, `get_node_for_tenant_vector`, `get_shards_for_tenant` |
| Collection name                  | `ShardRouter`                  | `collection_name` |

### Counts / metadata

| Query                    | Class                          | Function |
|--------------------------|--------------------------------|----------|
| Total vectors (sync)     | `ShardedCollection`            | `vector_count` |
| Total vectors (cluster)  | `DistributedShardedCollection` | `vector_count().await` (5 s TTL cache) |
| Total docs (sync)        | `ShardedCollection`            | `document_count` |
| Local docs (sync)        | `DistributedShardedCollection` | `document_count` |
| Total docs (cluster)     | `DistributedShardedCollection` | `document_count_distributed().await` |
| Per-shard counts         | `ShardedCollection`            | `shard_counts` |
| Needs rebalancing?       | `ShardedCollection`, `ShardRebalancer` | `needs_rebalancing`, `needs_rebalancing(&counts)` |
| Plan moves (add shard)   | `ShardRebalancer`              | `calculate_moves_for_add` |
| Plan moves (remove shard)| `ShardRebalancer`              | `calculate_moves_for_remove` |
| Plan moves (balance)     | `ShardRebalancer`              | `calculate_balance_moves` |
| Owner / tenant (local)   | `ShardedCollection`            | `owner_id`, `set_owner_id`, `belongs_to` |

### `CollectionType` dispatch (integration surface)

Source: `src/db/vector_store/collection_type.rs`. Every `VectorStore`
call goes through `CollectionType`, which selects among `Cpu`, `HiveGpu`,
`Sharded`, `DistributedSharded`. Sharding-specific behavior:

| Method                           | Sharded                              | DistributedSharded                                                                 |
|----------------------------------|--------------------------------------|------------------------------------------------------------------------------------|
| `add_vector`, `insert_batch`     | delegates to `ShardedCollection`     | spins a fresh `tokio::runtime::Runtime` and `block_on`s the async path             |
| `search`, `hybrid_search`        | delegates (serial over shards)       | same runtime-per-call pattern                                                      |
| `delete_vector`, `update_vector`, `get_vector` | supported             | returns `Storage(...)` error — callers must use the async cluster router           |
| `get_all_vectors`                | returns `Vec::new()` (not efficient) | returns `Vec::new()`                                                               |
| `metadata`, `vector_count`, `document_count`, `estimated_memory_usage`, `calculate_memory_usage`, `get_size_info` | aggregates from shards | aggregates from **local** shards only; exact cluster numbers require async API    |
| `get_embedding_type`             | `"sharded"`                          | `"distributed"`                                                                    |
| `get_graph`                      | `None`                               | `None` (graph not supported on sharded/distributed yet)                            |
| `load_hnsw_index_from_dump`      | logs warning, no-op                  | logs warning, no-op                                                                |
| `set_embedding_type`             | logs debug, no-op                    | logs debug, no-op                                                                  |
| `load_vectors_into_memory`       | uses `insert_batch`                  | logs warning, no-op                                                                |
| `fast_load_vectors`              | uses `insert_batch`                  | logs warning, no-op                                                                |
| `owner_id`, `belongs_to`         | supported                            | hard-coded `None` / `false` — multi-tenancy on distributed is TBD                  |
| `requantize_existing_vectors`    | iterates every shard                 | iterates local shards only                                                         |

---

## Monitoring

### What is exposed today

* `GET /api/v1/cluster/shard-distribution` returns per-node shard lists,
  suitable for polling into a dashboard.
* `GET /api/v1/cluster/nodes`, `/leader`, `/role` describe cluster
  membership and role (useful to detect nodes that dropped out and whose
  shards consequently became unreachable).
* `CollectionType::metadata()` for a `Sharded` collection sums
  `vector_count` and `document_count` across every shard. Scraping
  `/collections/{name}` returns these aggregated figures.
* `ShardedCollection::shard_counts()` (programmatic) returns the full
  `HashMap<ShardId, usize>` — expose this through a custom handler if you
  need per-shard Prometheus labels.
* `cluster::ShardMigrator::list_migrations` / `get_migration` expose
  `MigrationProgress { migration_id, shard_id, from_node, to_node,
  vectors_transferred, total_vectors, status, started_at }`.

### What is **not** exposed

* Per-shard Prometheus time series. `docs/specs/METRICS_REFERENCE.md` does
  not define any `vectorizer_shard_*` counter or gauge today. TBD — add
  metrics for shard vector count, search latency per shard, rebalance
  event counters.
* Epoch gauges. `DistributedShardRouter::current_epoch` is programmatic
  only.
* Alerting rules. None shipped; operators should build them off the
  shard-distribution endpoint.

### Detecting unbalanced shards

1. Poll `shard-distribution` and `shard_counts` at a fixed cadence
   (e.g. every 30 s).
2. Compute `max/avg` and `min/avg` and alert if either crosses
   `ShardingConfig.rebalance_threshold`. This is exactly what
   `ShardRebalancer::needs_rebalancing` does on the server side.
3. On alert, call `POST /api/v1/cluster/rebalance`. The endpoint returns
   the new distribution; diff against the previous snapshot to produce
   an audit log.

### Detecting stuck migrations

* Poll `ShardMigrator::list_migrations` (currently no HTTP surface — TBD
  to expose via `/api/v1/cluster/migrations`) and alert if any
  `MigrationStatus::InProgress` has `started_at` older than your SLO, or
  if `vectors_transferred` plateaus.

---

## Limitations & BETA status

The cluster subsystem that sharding depends on is explicitly BETA. On
top of the general BETA caveat, the following **specific limitations**
are visible in the current code:

1. **Distributed writes are not transactional across shards.** A batch
   of N vectors with mixed target nodes can partially succeed — per-node
   failures log a warning and return `Err`, but vectors already written
   to other nodes stay written.
2. **`CollectionType::{get_vector,update_vector,delete_vector}` do not
   work synchronously on distributed collections.** They return a
   `Storage(...)` error and require the async cluster router.
3. **Multi-tenancy (`owner_id`) is not supported on
   `DistributedShardedCollection`.** `CollectionType::owner_id` returns
   `None` and `belongs_to` returns `false` unconditionally.
4. **Graphs are not supported on sharded collections.**
   `CollectionType::get_graph` returns `None` for both `Sharded` and
   `DistributedSharded`.
5. **HNSW index dumps cannot be loaded into sharded collections.**
   `load_hnsw_index_from_dump` logs a warning and no-ops.
6. **`ShardedCollection::search` iterates shards serially.** True
   parallel fan-out across shards is not implemented in the local
   backend; parallelism comes from HNSW inside each shard only.
7. **`calculate_moves_for_add` is effectively dead code.** Its current
   body calls `route_vector` twice on the same router, so it always
   returns an empty move list. Callers needing rebalance on shard-add
   should use `calculate_balance_moves` or drive the plan via
   `DistributedShardRouter::calculate_migration_plan` + `ShardMigrator`.
   TBD: fix or remove.
8. **Two different hash functions.** Local routing uses SipHash;
   distributed routing uses xxh3. Converting a local-sharded collection
   into a distributed one requires a full re-index.
9. **No automatic rebalance on node join/leave.** Operators must call
   `POST /api/v1/cluster/rebalance` manually. TBD: verify whether
   `state_sync` plans to wire this.
10. **Rebalance is round-robin, not load-aware.**
    `DistributedShardRouter::rebalance` ignores existing shard sizes and
    simply assigns by index; use `calculate_migration_plan` if you need
    load-aware placement.
11. **Per-runtime cost on the sync path.** Every sync
    `CollectionType::*` call that hits a `DistributedShardedCollection`
    spins up a brand-new `tokio::runtime::Runtime` and `block_on`s.
    Under fd pressure this can fail; `metadata()` falls back to
    `vector_count = 0` with a warning when the runtime cannot be
    created (see `collection_type.rs` comment).
12. **Cross-node search drops failed shards silently.**
    `DistributedShardedCollection::search` and `hybrid_search` `warn!`
    on per-shard or per-node errors but still return the partial result
    set. Callers needing strict completeness must check cluster health
    separately.

---

## See also

* [`docs/deployment/CLUSTER.md`](../deployment/CLUSTER.md) — cluster
  bring-up, HA mode, Kubernetes/Helm, shard distribution endpoint usage,
  troubleshooting uneven shard distribution.
* [`docs/users/api/REPLICATION.md`](../users/api/REPLICATION.md) — how
  master/replica replication works (complementary to sharding, not a
  substitute).
* [`docs/specs/REPLICATION.md`](REPLICATION.md) — replication internals
  and configuration.
* `src/db/sharding.rs` — `ShardId`, `ShardMetadata`, `ConsistentHashRing`,
  `ShardRouter`, `ShardRebalancer`.
* `src/db/sharded_collection.rs` — local sharded collection backend.
* `src/db/distributed_sharded_collection.rs` — distributed sharded
  collection backend.
* `src/db/vector_store/collection_type.rs` — dispatch enum wrapping all
  four backends.
* `src/cluster/shard_router.rs` — `DistributedShardRouter` with
  epoch-stamped assignments and tenant-aware routing.
* `src/cluster/shard_migrator.rs` — `ShardMigrator`, `MigrationStatus`,
  `MigrationProgress`, `MigrationResult`.
* `crates/vectorizer-server/src/api/cluster.rs` — REST handlers for
  shard distribution and rebalance.
* `crates/vectorizer-server/src/server/qdrant/sharding_handlers.rs` —
  Qdrant-compatible `create_shard_key` / `delete_shard_key` /
  `list_shard_keys`.
