# SDK Go

Total: 106 símbolos exportados (56 métodos + 1 fn + 49 tipos)
README cobre: ~40 métodos | godoc: 98% (55/56)

| Categoria | Count |
|---|---|
| Doc-only | 3 (features sem método dedicado) |
| Code-only | 0 |
| SDK→Server orphans | 0 |
| **Server→SDK gaps** | **12 MCP tools sem wrapper Go** |
| Sem godoc | 1 (`VectorizerError.Error()`) |

## Doc-only — features mencionadas mas sem método
- README cita Master/Replica routing mas sem `GetReadClient`/`GetWriteClient` expostos
- README cita Contextual Search, Multi-Collection Search, Discovery — sem método correspondente

## Server→SDK gaps — 12 MCP tools faltando

### File operations (5) — alta prioridade
- `get_file_content`, `list_files`, `get_file_chunks`, `get_project_outline`, `get_related_files`

### Discovery/filtering (2)
- `filter_collections`, `expand_queries`

### Search variants (3)
- `search_hybrid`, `search_extra`, `multi_collection_search`

### Admin (2)
- `list_empty_collections`, `cleanup_empty_collections`

## Conclusão
Go SDK é **wrapper fino e bem documentado** (98% godoc). MAS ~20% da
superfície MCP do server não está exposta, especialmente file operations.
Production-ready para core vector ops; falta tudo que envolve indexação
de codebase/documentos.
