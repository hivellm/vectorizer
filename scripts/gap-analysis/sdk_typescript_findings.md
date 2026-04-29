# SDK TypeScript

Total exported: 85 (REST via VectorizerClient + RpcClient separado)
README cobre: ~45 nomes distintos

| Categoria | Count |
|---|---|
| Doc-only (README mas não no código) | **2** |
| Code-only (código sem README) | **38** |
| SDK→Server orphans | 0 |
| Server→SDK gaps | 2 |

## Doc-only (2 — README promete sem implementar)
- `summarizeText(...)` — README:660-668
- `summarizeContext(...)` — README:671-679
- → Decisão: ou implementa wrapper (depende de wire do `/summarize/text` server-side, já flagged), ou remove do README

## Server→SDK gaps (2)
- `POST /embedding/providers` + `GET /embedding/providers/{id}` — mas estes próprios endpoints são fantasma server-side (iter 1). Resolvendo o lado do server, o gap some.

## Code-only — 38 métodos exportados sem doc no README

### Admin (6)
- `admin.ts:25` `getLogs`
- `admin.ts:31` `forceSaveCollection`
- `admin.ts:73` `getServerConfig`
- `admin.ts:79` `updateConfig`
- `admin.ts:87` `restartServer`
- `admin.ts:123` `getBackupDirectory`

### Auth/Discovery/Files/Graph (6)
- `core.ts:55` `login`
- `discovery.ts:36` `scoreCollections`
- `files.ts:37` `getFileSummary`
- `files.ts:83` `searchByFileType`
- `files.ts:152` `uploadFileContent`
- `graph.ts:64` `deleteGraphEdge`
- `graph.ts:70` `listGraphEdges`

### Qdrant compat (30+)
Toda a layer `qdrant*` (Snapshots, Sharding, Cluster, Query, Matrix APIs) sem nenhum exemplo no README

## Recomendações
1. Adicionar seção "Admin & Server Management" no README
2. Adicionar seção "Qdrant Compatibility" no README
3. Clarificar RPC vs REST cedo no README (`searchBasic`/`hello` são RPC-only)
4. Marcar summarization como future ou implementar
