# SDK Python

Total: 78 métodos públicos | README cobre: 64

| Categoria | Count |
|---|---|
| Doc-only (README sem código) | **12** |
| Code-only (código sem README) | **15** |
| SDK→Server orphans | 0 |
| Server→SDK gaps | 4 |
| **🚨 Bugs reais** | **1** |

## 🚨 BUG CRÍTICO em runtime
`vectorizer/graph.py:159` `delete_graph_edge(edge_id)`:
```python
data = await self._transport.delete(f"/graph/collections/{collection}/edges")
#                                                          ^^^^^^^^^^
# NameError: collection não definido neste método
```
Provavelmente deveria ser `f"/graph/edges/{edge_id}"`. **Método dispara NameError em qualquer chamada**. Fix imediato necessário, não é só doc gap.

## Doc-only (12 — README promete sem implementar)

### Nomes errados (singular vs plural)
- `delete_vector` (linha 164) — código tem `delete_vectors` (plural)
- `update_vector` (linha 177) — não existe

### RPC-only mascarado como REST
- `search_basic` (linha 308) — RPC-only
- `hello` (linha 23) — RPC-only

### Doc-only confirmados (igual iter 1)
- `summarize_text` (linha 433)
- `summarize_context` (linha 449)

### Workspace/Backup — server tem, SDK não expõe explicitamente
- `add_workspace` / `list_workspaces` / `remove_workspace` (linhas 465-485)
- `create_backup` / `list_backups` / `restore_backup` (linhas 494-511)
- → Existem no server mas SDK Python não tem método explícito (só dynamic via `__getattr__` — undiscoverable)

## Server→SDK gaps (4)
- `cleanup_empty_collections` (MCP tool) — não em collections.py
- `list_empty_collections` (MCP tool) — não em collections.py
- `search_by_file` (REST handler) — não em search.py

## Code-only — 15 métodos sem doc no README
- Toda a layer Qdrant compat (`qdrant_list_collections`, `qdrant_get_collection`, `qdrant_create_collection`, `qdrant_upsert_points`, `qdrant_delete_points`, `qdrant_retrieve_points`, `qdrant_count_points`)
- `admin.py:247` `upload_file()`
- `auth.py:52` `login()`

## Recomendações
1. **P0**: fix bug em `graph.py:159` (NameError em runtime)
2. **P0**: expor workspace/backup como métodos explícitos
3. **P1**: corrigir nomes singular/plural no README (delete_vector→delete_vectors)
4. **P1**: adicionar `list_empty_collections`/`cleanup_empty_collections`
5. **P2**: documentar Qdrant compat layer com API reference
