# Batch 4 — Distributed/Persistence

Total: 312 fns | DOC: 84 (27%) | INTERNAL: 162 (52%) | **USER_FACING_GAP: 48 (15%)** | UNCERTAIN: 18

## Critical USER_FACING gaps

### Replication
- `replication/types.rs:280` `WriteConcern` enum — operators precisam decidir consistency level, sem doc
- `replication/master.rs:501` `replicate_with_concern()` — **API de sync replication funciona**, mas REPLICATION.md:255 diz "Future"
  - **Action: atualizar REPLICATION.md, sync replication NÃO é future, é real**

### Cluster / HA / Raft
- `cluster/mod.rs:91,95,99` `max_cache_memory_bytes`, `enforce_mmap_storage`, `disable_file_watcher` — knobs documentados em specs internos, validators públicos sem doc
- `cluster/collection_sync.rs:259` `QuorumResult { quorum_met, ... }` — usuários precisam interpretar quórum, sem doc
- `cluster/validator.rs:228` `with_limits()` — limites de cache silenciosos
- `cluster/raft_node.rs:770-800` + `cluster/leader_router.rs` — eleição Raft completa, ZERO doc operacional
- `cluster/ha_manager.rs:56,97` `on_become_leader/follower` callbacks — sem doc do state machine
- **CRITICAL: REPLICATION.md:20 promete "Manual Failover" mas NÃO há API promote/demote exposta** — operadores não têm procedimento

### Persistence (.vecdb format)
- `persistence/mod.rs` — `PersistedVector`, `PersistedVectorStore` sem doc de formato user-facing
- STORAGE.md:37 menciona "ZIP archive" mas formato real é `gzip + bincode v1.0 + CRC32`
- **Action: criar `docs/users/operations/VECDB_FORMAT.md`** com layout binário completo, versionamento, recovery

### Storage
- `storage/compact.rs:337` `compact_all_with_cleanup(remove_source_files: bool)` — operação destrutiva sem doc

## Coverage por feature

| Feature | Code | Doc | Gap |
|---|---|---|---|
| Quorum consensus | ✅ `collection_sync.rs:43-160` | ❌ | CRITICAL |
| Manual failover (REPLICATION.md:20) | ❌ API ausente | ✅ promete | **DOC mente** |
| Raft leader election | ✅ `raft_node.rs:770-800` | ❌ | CRITICAL |
| HA state callbacks | ✅ `ha_manager.rs` | ❌ | MODERATE |
| Sync replication (`WriteConcern::All`) | ✅ `master.rs:501-545` | ❌ "Future" | **DOC desatualizada** |
| .vecdb gzip+bincode+CRC32 layout | ✅ código | ⚠️ "ZIP" só | CRITICAL para backups |

## INTERNAL counts
cluster: 78 | replication: 19 | persistence: 38 | storage: 27
