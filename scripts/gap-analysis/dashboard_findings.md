# Dashboard React UI (116 .tsx/.ts)

| Item | Count |
|---|---|
| Endpoints chamados pela UI | 43 únicos |
| **Endpoints quebrados (404)** | **2** ⚠️ |
| Features UI sem doc no DASHBOARD_INTEGRATION.md | 5 áreas |
| Confirmações iter 2 | sim |

## 🚨 2 endpoints quebrados (UI dispara 404)
- `dashboard/src/pages/LogsPage.tsx:55` — chama `GET /api/logs`, real é `/logs`
- `dashboard/src/pages/ApiDocsPage.tsx:792` — chama `GET /api-keys`, real é `/auth/keys`
- → **Fix imediato**, são bugs de UI

## Features UI sem doc (5 áreas — DASHBOARD_INTEGRATION.md cobre só 7 das 12 reais)

### Graph Management (12 endpoints, GraphPage)
- `/graph/nodes/{collection}`, `/graph/nodes/.../neighbors`, `/graph/nodes/.../related`
- `/graph/path` POST, `/graph/edges` POST, `/graph/edges/{edge_id}` DELETE
- `/graph/collections/{collection}/edges` GET
- `/graph/discover/{collection}` POST + `/discover/{collection}/{node_id}` POST
- `/graph/discover/{collection}/status`, `/graph/enable/{collection}` POST
- `/graph/status/{collection}`

### Cluster Management (8 endpoints, ClusterPage)
- `/api/v1/cluster/nodes` GET/POST/{node_id} GET/DELETE
- `/api/v1/cluster/shard-distribution`, `/api/v1/cluster/rebalance` POST
- `/api/v1/cluster/leader`, `/api/v1/cluster/role` ← **confirma iter 2**

### User Management (4 endpoints, UsersPage)
- `/auth/users` GET/POST, `/auth/users/{username}/password` PUT, `/auth/users/{username}` DELETE

### API Key Management (3 endpoints, ApiKeysPage)
- `/auth/keys` GET/POST, `/auth/keys/{id}` DELETE

### File Watcher Metrics
- `/metrics` GET (já era gap conhecido iter 1+2)
- `/workspace/config` GET

## Confirmação iter 2 — cluster endpoints
Iter 2 §4.2 flagged `api/cluster.rs:396` `get_cluster_leader` e `:409` `get_cluster_role` como user-facing sem doc. Dashboard `ClusterPage.tsx:99-100` chama **ativamente** ambos:
```tsx
const leaderResult = await fetchJSON<LeaderInfo>('/api/v1/cluster/leader');
const roleResult = await fetchJSON<ClusterRole>('/api/v1/cluster/role');
```
→ Confirma que SIM são user-facing reais. Dashboard depende deles.

## Recomendações
1. **P0**: corrigir 2 endpoints quebrados na UI (LogsPage, ApiDocsPage)
2. **P0**: expandir DASHBOARD_INTEGRATION.md com 5 áreas: Graph, Cluster, Users, API keys, Metrics
3. **P1**: documentar `cluster/leader` e `cluster/role` em user docs (já confirmados como user-facing)
4. **P1**: adicionar nota de security/middleware sobre quais endpoints requerem admin
