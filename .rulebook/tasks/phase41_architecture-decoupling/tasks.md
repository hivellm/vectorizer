## 1. MetricsSink inversion

- [ ] 1.1 Define a `MetricsSink` trait in a low-level module (or `vectorizer-core`); `monitoring` provides the production impl
- [ ] 1.2 Inject `MetricsSink` into `db/ttl_reaper.rs`, `cache/query_cache.rs`, `hub/quota.rs`, `auth/mod.rs` — removing their direct `monitoring::METRICS` imports

## 2. Config ownership inversion

- [ ] 2.1 Move `AuthConfig`, `HubConfig`, `ClusterConfig` ownership out of `config/vectorizer.rs`: config holds generic serde sub-structs; auth/hub/cluster own their typed configs

## 3. Cluster decoupling

- [ ] 3.1 Add a `ShardRouter` trait in `vectorizer-core`; make `db/distributed_sharded_collection.rs` depend on the trait instead of concrete cluster types
- [ ] 3.2 Break `cluster → hub` (`server_client.rs:14,460`): move `TenantContext` to a shared crate or abstract it behind a trait

## 4. Cargo + file hygiene

- [ ] 4.1 Delete dead commented `[[bin]]` stanzas and non-existent feature references (`Cargo.toml:261-372,450-463`); audit external consumers, then remove deprecated `metal-native`/`gpu-accel` aliases
- [ ] 4.2 Split `db/vector_store/collections.rs` (1050 ln) along lifecycle / disk-load / tenancy seams
- [ ] 4.3 Split `db/vector_store/persistence.rs` (884 ln) along compact / legacy-load / snapshot seams

## 5. Concurrency hygiene

- [ ] 5.1 Resolve the DashMap re-entrancy trap in `collections.rs` (`get_collection` Ref vs `get_collection_mut` RefMut deadlock): route mutations through `alter`/`entry` or add documented debug-assert guard
- [ ] 5.2 Standardize lock usage across `db/` (parking_lot for short sections; tokio locks only across await), documenting the rule in the module docs

## 6. Tail (mandatory — enforced by rulebook v5.3.0)

- [ ] 6.1 Update or create documentation covering the implementation
- [ ] 6.2 Write tests covering the new behavior
- [ ] 6.3 Run tests and confirm they pass
