## 1. MetricsSink inversion

- [x] 1.1 `MetricsSink` trait in `vectorizer-core/src/metrics_sink.rs` (with `NoopMetricsSink` default); production `PrometheusMetricsSink` in `monitoring/metrics_sink.rs` (optionally wrapping `ApiKeyUsageRecorder`)
- [x] 1.2 Injected into `db/ttl_reaper.rs`, `cache/query_cache.rs`, `hub/quota.rs`, `auth/mod.rs` (AuthManager::set_metrics) — the four `use crate::monitoring` back-references are gone (grep-clean); AuthHandlerState owns the usage recorder and hands the manager a wrapping sink

## 2. Config ownership inversion

- [x] 2.1 `AuthConfig`/`HubConfig`/`ClusterConfig` moved into `config/sections/{auth,hub,cluster}.rs` as plain serde types; `auth/`, `hub/`, `cluster/` re-export them at their old paths — `config/` has zero service-module imports (grep-clean)

## 3. Cluster decoupling

- [x] 3.1 `ShardTopology` trait in `db/shard_topology.rs` (String node ids, db-side `ShardId`); `cluster::ClusterShardTopology` implements it over `DistributedShardRouter` + `ClusterManager`; `DistributedShardedCollection` now holds `Arc<dyn ShardTopology>` — `ClusterManager`/`DistributedShardRouter`/`NodeId` imports removed from db. **Documented residual**: `ClusterClientPool` stays concrete (it IS the gRPC transport; abstracting it means abstracting the wire client — out of scope for a topology seam). db addresses it via the new string-id `get_client_by_id`, so no `NodeId` import remains. Stub-topology unit test proves db sharded collections construct + route with zero real cluster types (spec scenario)
- [x] 3.2 `TenantContext`/`TenantPermission`/`TenantRateLimits` moved to `models/tenant.rs`; `hub/auth.rs` re-exports at old paths and keeps the SDK-specific `from_sdk_permission` impl (inherent impls may live in any module of the defining crate); `cluster/server_client.rs` imports from models — cluster→hub edge gone

## 4. Cargo + file hygiene

- [x] 4.1 Dead commented `[[bin]]` stanzas + non-existent feature references removed from Cargo.toml (0 remain); deprecated `metal-native`/`gpu-accel` aliases audited per agent report
- [x] 4.2 `collections.rs` (1050 ln) split into `collections/{lifecycle,disk_load,tenancy}.rs`
- [x] 4.3 `vector_store/persistence.rs` (884 ln) split into `persistence/{snapshots,loading}.rs` with a `mod.rs` re-exporting `NativeSnapshotInfo`

## 5. Concurrency hygiene

- [x] 5.1 DashMap re-entrancy invariants documented on `get_collection{,_mut}`; `VectorStore::update` refactored to the shared-Ref pattern `delete` uses (removes the deadlock CLASS that bit bulk_update_metadata in phase39, not just the one instance)
- [x] 5.2 Lock convention documented in `db/mod.rs` (parking_lot for non-await sections; tokio::sync only across await — auto_save.rs + wal_integration.rs are the sanctioned users)

## 6. Tail (mandatory — enforced by rulebook v5.3.0)

- [x] 6.1 Update or create documentation covering the implementation — CHANGELOG [3.5.0] + module docs at every seam (shard_topology, metrics_sink, tenant, persistence/mod, collections split, lock convention)
- [x] 6.2 Write tests covering the new behavior — stub-topology construction test (db without cluster types), recording-MetricsSink test on query_cache, existing suites re-verified after every move
- [x] 6.3 Run tests and confirm they pass — vectorizer lib 1036, server lib 218, cluster_integration + distributed_sharding 11, clippy -D warnings 0
