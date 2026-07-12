# Proposal: phase41_architecture-decoupling

Source: docs/analysis/2026-07-11-improvement-analysis/ (§1.1-1.3, §1.5)

## Why

The 2026-07-11 improvement analysis mapped why the umbrella-crate
split (started in phase4) stalled: nine upward back-references where
low-level modules reach into service modules:

| Consumer → Provider | Location |
|---|---|
| db → cluster | `db/distributed_sharded_collection.rs:16` |
| db → monitoring | `db/ttl_reaper.rs:27` |
| cache → monitoring | `cache/query_cache.rs:157` |
| config → auth | `config/vectorizer.rs:7` |
| config → hub | `config/vectorizer.rs:9` |
| config → cluster | `config/vectorizer.rs:39,663` |
| cluster → hub | `cluster/server_client.rs:14,460` |
| hub → monitoring | `hub/quota.rs:17` |
| auth → monitoring | `auth/mod.rs:298,329,337` |

Four of nine are the global `monitoring::METRICS`; three are config
owning service-specific structs. Additional structural debt: ~20 dead
commented `[[bin]]` stanzas referencing non-existent features
(`Cargo.toml:261-372,450-463`), oversized multi-concern files
(`collections.rs` 1050 lines, `persistence.rs` 884), a documented
DashMap re-entrancy deadlock trap (`get_collection` Ref +
`get_collection_mut` on the same shard), and inconsistent lock
libraries (`tokio::sync::RwLock` in `auto_save.rs`/`wal_integration.rs`
vs `parking_lot` elsewhere).

## What Changes

- Introduce a `MetricsSink` trait (defined low, implemented by
  `monitoring`), injected into db/cache/hub/auth — removes 4 of 9
  back-references.
- Move service-specific config structs (`AuthConfig`, `HubConfig`,
  `ClusterConfig`) out of `config/vectorizer.rs` ownership: config
  holds serde-generic sub-structs owned by each service module —
  breaks the config→{auth,hub,cluster} triangle.
- Add a `ShardRouter` abstraction in `vectorizer-core` so
  `db/distributed_sharded_collection.rs` depends on a trait instead
  of concrete cluster types; decouple `cluster → hub` via a
  `TenantContext` trait or move of the type to a shared crate.
- Delete the dead `[[bin]]` stanzas and non-existent feature
  references from `crates/vectorizer/Cargo.toml`; remove the
  deprecated `metal-native`/`gpu-accel` aliases after confirming no
  external consumer.
- Split `db/vector_store/collections.rs` and
  `db/vector_store/persistence.rs` along their existing doc-comment
  seams (lifecycle / disk-load / tenancy; compact / legacy-load /
  snapshots).
- Resolve the DashMap re-entrancy trap: route mutations through
  `alter`/`entry` or document + debug-assert the no-Ref-held rule.
- Standardize lock usage: parking_lot for short critical sections,
  tokio locks only where held across await, documented per module.

## Impact

- Affected specs: `specs/decoupling/spec.md` (new, in this task)
- Affected code: `crates/vectorizer/src/{monitoring,db,cache,config,auth,hub,cluster}/`,
  `crates/vectorizer-core/src/`, `crates/vectorizer/Cargo.toml`
- Breaking change: NO for external APIs (internal module boundaries
  only; feature-alias removal gated on external-consumer audit)
- User benefit: unblocks the crate split (faster builds, clearer
  ownership); removes a real deadlock hazard
