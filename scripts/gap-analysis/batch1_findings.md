# Batch 1 — API Surface (REST/MCP/Auth)

Total: 334 fns | DOC: 203 (61%) | INTERNAL: 113 (34%) | **USER_FACING_GAP: 13 reais (4%)** | UNCERTAIN: 0

(Agente reportou 18 mas 5 são test helpers — `advanced_api.rs:533/1003/1022` — corrigi para 13 reais.)

## USER_FACING gaps reais

### Setup wizard
- `server/setup_handlers.rs:542` `display_first_start_guidance()` — sem doc
- `server/setup_handlers.rs:589` `needs_setup()` — sem doc
- `server/setup_handlers.rs:639` `browse_directory()` — sem doc
- → Action: expandir `docs/users/getting-started/SETUP_WIZARD.md`

### Dashboard
- `server/embedded_assets.rs:82` `dashboard_handler()` — endpoint sem doc HTTP
- `server/embedded_assets.rs:89` `dashboard_root_handler()` — routing sem doc
- → Action: criar/expandir `docs/specs/DASHBOARD.md`

### Métricas / observabilidade
- `server/rest_handlers/meta.rs:204` `get_prometheus_metrics()` — endpoint `/metrics` sem doc semântica (já flagged em iter 1, confirmado)
- `server/core/helpers.rs:165` `get_file_watcher_metrics()` — métricas de file watcher sem doc
- → Action: criar `docs/runbooks/MONITORING_SETUP.md` ou `docs/prometheus/METRICS.md`

### Cluster API
- `api/cluster.rs:102` `create_cluster_router()` — factory sem doc
- `api/cluster.rs:396` `get_cluster_leader()` — endpoint sem ref
- `api/cluster.rs:409` `get_cluster_role()` — endpoint sem ref
- → Action: expandir `docs/users/configuration/CLUSTER.md` com endpoint reference

### File operations
- `server/files/validation.rs:141` `get_language_from_extension()` — mapping sem tabela
- `server/files/validation.rs:218` `language()` — sem doc
- → Action: tabela de file-type support em `FILE_OPERATIONS.md`

### Graph API
- `api/graph.rs:48` `GraphApiState::new()` — sem doc state init
- `api/graph.rs:57` `create_graph_router()` — sem doc factory

## Coverage forte (não-gaps)
Discovery, GRAPH (31 fns documentadas), REPLICATION (4), AUTHENTICATION (13),
ADMIN (15), BACKUP_RESTORE (14), FILE_OPERATIONS (18), API_REFERENCE (24),
QDRANT_COMPATIBILITY_INDEX (51) — todos OK.

## INTERNAL counts
auth: 51 | error_middleware: 8 | auth_handlers: 16 | files/validation: 7 | graphql/types: 6 | mcp/connection_manager: 7 | capabilities: 2 | core: 3 | graphql/schema: 3 | advanced_api test: 7 | graph borderline: 2
