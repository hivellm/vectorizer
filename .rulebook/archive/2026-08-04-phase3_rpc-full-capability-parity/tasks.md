## 1. Durability and replication of RPC writes

- [x] 1.1 Carry `auto_save_manager` and the replication master on `RpcState`
- [x] 1.2 Mark auto-save on every mutating RPC command
- [x] 1.3 Make `collections.force_save` actually persist
- [x] 1.4 Replicate collection and vector mutations made over RPC

## 2. collections.create parity

- [x] 2.1 Honour the `graph` config so a graph-enabled collection is creatable over RPC
- [x] 2.2 Validate the embedding provider exists and its dimension matches
- [x] 2.3 Pick storage type by cluster mode, as REST does

## 3. Graph reachability

- [x] 3.1 Add `graph.enable`
- [x] 3.2 Add `graph.status`

## 4. Stats and providers

- [x] 4.1 Add `embedding.list_providers`
- [x] 4.2 Add `collections.get_stats`
- [x] 4.3 Add `stats.database`

## 5. Search

- [x] 5.1 Add `search.extra`

## 6. User management

- [x] 6.1 Implement `auth.users_create`
- [x] 6.2 Implement `auth.users_list`
- [x] 6.3 Implement `auth.users_delete`
- [x] 6.4 Implement `auth.users_change_password`

## 7. Cluster

- [x] 7.1 Add `cluster.nodes_list`
- [x] 7.2 Add `cluster.node_get`
- [x] 7.3 Add `cluster.node_remove`
- [x] 7.4 Add `cluster.leader`
- [x] 7.5 Add `cluster.role`

## 8. Files

- [x] 8.1 Add `files.config_get`

## 9. Parity enforcement

- [x] 9.1 Advertise every new command in `rpc_capability_names()`
- [x] 9.2 Add an RPC column to the capability registry
- [x] 9.3 Assert RPC parity at boot in `assert_inventory_invariants`

## 10. Tail (docs + tests — check or waive with tailWaiver)

- [x] 10.1 Update or create documentation covering the implementation — rewrote the command catalog in `docs/specs/VECTORIZER_RPC.md`, corrected the registry-naming claim, updated CHANGELOG
- [x] 10.2 Write tests covering the new behavior — durability classification, force_save, graph reachability, create guards, the stats trio, search.extra, files.config_get, cluster inspection, user-management gating
- [x] 10.3 Run tests and confirm they pass — 235 lib tests green, clippy clean, workspace check clean, live container battery green
