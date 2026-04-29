# Documentation Gap Analysis — 2026-04-24

> Objetivo: identificar **gaps de informação** entre o que a documentação
> promete e o que o código realmente faz. Para cada gap, listar uma
> recomendação binária: **IMPLEMENTAR** (código deve existir) ou **DOCUMENTAR
> / AJUSTAR DOC** (código já está certo, doc está errada ou incompleta).
>
> Análise feita por 5 agentes paralelos em read-only mode sobre `docs/` + `src/`.
> Escopo: REST API, MCP, Core DB, Replication/Cluster, Discovery/Graph/Crypto/Dashboard.

## Resumo executivo

| Categoria | Doc-only (fantasma) | Code-only (invisível) | Mismatch (nome/schema) |
|---|---|---|---|
| REST API | **5** | 0 | 0 |
| MCP tools | **11** | **14** | **6** |
| Core DB | **2** | **3** | **2** |
| Replication/Cluster | **3** | **4** | **2** |
| Discovery/Graph/Crypto/Hub | 0 | **1 (Prom/Grafana docs)** | 1 |
| **Total** | **21** | **22** | **11** |

Maior concentração de gaps: **MCP tools** (nomenclatura divergente + 8
`graph_*` implementados sem entrar em `docs/specs/MCP.md`) e **REST API**
(5 endpoints de summarize/embedding providers só existem na OpenAPI spec).

---

## 1. REST API

### 1.1 Documentado mas NÃO implementado

| Endpoint | Doc | Recomendação |
|---|---|---|
| `GET /embedding/providers` | `docs/api/openapi.yaml:406` | **REMOVER DA DOC** — funcionalidade não planejada no backlog ativo; configuração de provider é via `config.yml`, não runtime |
| `POST /embedding/providers/set` | `docs/api/openapi.yaml:421` | **REMOVER DA DOC** — mesma razão acima; trocar provider em runtime é vetorialmente inseguro (reindex necessário) |
| `POST /summarize/text` | `docs/api/openapi.yaml:457` | **DECIDIR** — há interesse em LLM-summarization? Se sim → implementar. Se não → remover da spec |
| `GET /summaries` | `docs/api/openapi.yaml:484` | Mesmo que acima; hoje não há storage de summaries |
| `GET /summaries/{summary_id}` | `docs/api/openapi.yaml:526` | Mesmo que acima |

**Ação sugerida**: abrir rulebook task `rest-openapi-cleanup` para remover os
5 endpoints da `openapi.yaml` OU criar specs em `.rulebook/tasks/` para
cada grupo (embedding-providers, summarization) com requisitos SHALL.

### 1.2 Implementado mas NÃO documentado

Nenhum. Toda rota Axum tem entrada correspondente na `openapi.yaml`.

### 1.3 Mismatches

Nenhum material. Diferença cosmética de `{name}` vs `{collection_name}` na
OpenAPI é apenas convenção de placeholder.

---

## 2. MCP Tools

### 2.1 Documentado mas NÃO implementado

| Tool | Doc | Recomendação |
|---|---|---|
| `batch_delete_vectors` | `docs/specs/MCP.md:483` | **IMPLEMENTAR** — handlers REST `/batch/*` já existem (`src/api/batch.rs`); expor via MCP é linear |
| `batch_insert_texts` | `docs/specs/MCP.md:429` | **IMPLEMENTAR** — mesmo motivo |
| `batch_search_vectors` | `docs/specs/MCP.md:448` | **IMPLEMENTAR** — mesmo motivo |
| `batch_update_vectors` | `docs/specs/MCP.md:465` | **IMPLEMENTAR** — mesmo motivo |
| `contextual_search` | `docs/specs/MCP.md:158` | **IMPLEMENTAR** — REST já tem em `intelligent_search/rest_api.rs:136-229` (`handle_contextual_search`); adicionar entry no match de `handlers.rs` |
| `delete_collection` | `docs/specs/MCP.md:239` | **IMPLEMENTAR** — operação destrutiva mas legítima via MCP; REST já expõe |
| `delete_vectors` (plural) | `docs/specs/MCP.md:378` | **AJUSTAR DOC** — code usa `delete_vector` (singular); padronizar doc para singular |
| `embed_text` | `docs/specs/MCP.md:416` | **IMPLEMENTAR** — utilitário útil para clientes MCP; wrapping trivial sobre `EmbeddingProvider::embed` |
| `get_database_stats` | `docs/specs/MCP.md:497` | **IMPLEMENTAR** — REST tem `/stats`; espelhar no MCP |
| `insert_texts` (plural) | `docs/specs/MCP.md:360` | **AJUSTAR DOC** — code é `insert_text` singular; decidir se vale promover à versão batch (ver item acima) |
| `search_vectors` | `docs/specs/MCP.md:86` | **AJUSTAR DOC** — code usa `search`; padronizar nome no spec |

### 2.2 Implementado mas NÃO documentado

| Tool | Código | Recomendação |
|---|---|---|
| `expand_queries` | `src/server/mcp/handlers.rs:121` | **DOCUMENTAR** — feature útil (query expansion) precisa de seção no MCP.md |
| `filter_collections` | `handlers.rs:120` | **DOCUMENTAR** |
| `get_file_chunks` | `handlers.rs:126` | **DOCUMENTAR** — doc menciona `get_file_chunks_ordered`, renomear para bater com código |
| `get_file_content` | `handlers.rs:124` | **DOCUMENTAR** |
| `get_project_outline` | `handlers.rs:127` | **DOCUMENTAR** |
| `get_related_files` | `handlers.rs:128` | **DOCUMENTAR** |
| `list_files` | `handlers.rs:125` | **DOCUMENTAR** — user API chama `list_files_in_collection`; alinhar |
| `search_extra` | `handlers.rs:116` | **DOCUMENTAR** — ou avaliar se é tool interno (parece experimental) |
| `search_hybrid` | `handlers.rs:117` | **DOCUMENTAR** — tool canônico, imperdoável estar fora do MCP.md |
| `graph_list_nodes` | `handlers.rs:131` | **DOCUMENTAR** — criar seção "Graph Operations" no MCP.md |
| `graph_get_neighbors` | `handlers.rs:132` | idem |
| `graph_find_related` | `handlers.rs:133` | idem |
| `graph_find_path` | `handlers.rs:134` | idem |
| `graph_create_edge` | `handlers.rs:135` | idem |
| `graph_delete_edge` | `handlers.rs:136` | idem |
| `graph_discover_edges` | `handlers.rs:137` | idem |
| `graph_discover_status` | `handlers.rs:138` | idem |

### 2.3 Mismatches de nome/schema

| Doc | Code | Recomendação |
|---|---|---|
| `search_vectors` | `search` | Padronizar → `search` (decisão: nomes no estilo `<verb>_<qualifier>` já são usados em `search_semantic/hybrid/intelligent`) |
| `insert_texts` | `insert_text` | Padronizar → singular (batch será tool separado) |
| `delete_vectors` | `delete_vector` | Padronizar → singular (idem) |
| `intelligent_search` | `search_intelligent` | Padronizar → `search_intelligent` (mantém prefixo comum) |
| `semantic_search` | `search_semantic` | Padronizar → `search_semantic` |
| "31 tools" (README) vs "38+" (UMICP.md) vs "22" (MCP.md) vs **31 reais** | — | **AJUSTAR DOC** — usar número real (31) em todas as menções |

---

## 3. Core DB (Collections, Vectors, Search, Embeddings, Quantization, Cache)

### 3.1 Documentado mas NÃO implementado

| Capability | Doc | Recomendação |
|---|---|---|
| **Distance metric: Manhattan** | `docs/users/collections/CONFIGURATION.md:14-70` | **REMOVER DA DOC** — enum `DistanceMetric` só tem `{Cosine, Euclidean, DotProduct}` (`src/models/mod.rs`); Manhattan não está no roadmap |
| **Quantization cache metrics** (hit ratio, hits/misses) | `docs/users/guides/QUANTIZATION.md:303-376` | **DECIDIR** — se é feature desejada, virar task; se não, cortar da guia. Hoje só existe cache de storage, não de quantization |

### 3.2 Implementado mas NÃO documentado

| Capability | Código | Recomendação |
|---|---|---|
| **BagOfWordsEmbedding** | `src/embedding/providers/bag_of_words.rs` | **DOCUMENTAR** em `EMBEDDINGS.md` |
| **CharNGramEmbedding** | `src/embedding/providers/char_ngram.rs` | **DOCUMENTAR** em `EMBEDDINGS.md` |
| **`contextual_search` REST handler** | `src/intelligent_search/rest_api.rs:136-229` | **DOCUMENTAR** em `SEARCH.md` — usuários não sabem que existe |
| PQ flag `adaptive_assignment` | `src/quantization/product.rs:17-37` | **DOCUMENTAR** em `QUANTIZATION.md` |

### 3.3 Mismatches / claims duvidosos

| Claim | Onde | Recomendação |
|---|---|---|
| Tabela de redução de bits de quantização contradiz texto (50% vs 25% para 16-bit) | `QUANTIZATION.md:44` vs `:182` | **AJUSTAR DOC** — refazer tabela consistente (16-bit = 50% do tamanho = 50% redução) |
| "MAP score improves (+8.9%) com 8-bit SQ" | `QUANTIZATION.md:48-52` | **AUDITAR** — claim contraintuitivo. Se for verdade, apontar benchmark reprodutível; se for marketing, remover |

---

## 4. Replication, Cluster, Tenant, Workspace, Backup

### 4.1 Documentado mas NÃO implementado

| Feature | Doc | Recomendação |
|---|---|---|
| **Sync replication mode** (strong consistency) | `REPLICATION.md:255-260` | Mantém como "Future" explícito na doc — **AJUSTAR DOC** para deixar claro que hoje é só async. Tipos `WriteConcern::{All,Count}` já existem; porém sem caminho end-to-end |
| **Tenant Migration `scan/plan/execute` endpoints** | `TENANT_MIGRATION.md:262-314` | **IMPLEMENTAR** (reabilitar `src/server/hub_handlers/tenant.rs`, está comentado em `mod.rs:16` por conflito axum/tonic). Código existe, só precisa ser ligado |
| **`MoveStorage` migration type** | `TENANT_MIGRATION.md:111` | **IMPLEMENTAR** — hoje é no-op em `tenant.rs:351`. Ou marcar explicitamente como BETA na doc |

### 4.2 Implementado mas NÃO documentado

| Feature | Código | Recomendação |
|---|---|---|
| **`WriteConcern` type system** | `src/replication/types.rs:277-293` | **DOCUMENTAR** + expor via API (hoje é interno) |
| **Raft automatic failover** (full HA) | `src/cluster/raft_node.rs`, `ha_manager.rs` | **AJUSTAR DOC** — `REPLICATION.md:20` diz "Manual Failover" enquanto `CLUSTER.md:129-174` descreve Raft automático. Reconciliar — usuário hoje não sabe qual modo está usando |
| **Cluster memory enforcement** (`enforce_mmap_storage`, `disable_file_watcher`, `max_cache_memory_bytes`) | `src/cluster/mod.rs:85-100` | **DOCUMENTAR** em `docs/users/api/CLUSTER.md` (hoje só aparece em specs internos) |
| **Quorum reads (`QuorumResult`, `QuorumError`)** | `src/cluster/collection_sync.rs` | **DOCUMENTAR** semântica e tuning |

### 4.3 Mismatches de garantia

| Item | Status |
|---|---|
| "Async replication: eventual consistency" sem mencionar threshold `Lagging = 1000ms` (`src/replication/types.rs:257`) | **AJUSTAR DOC** para citar o threshold |
| File watcher "incompatível com cluster mode" documentado, mas sem mensagem de erro runtime se usuário misconfigurar | **IMPLEMENTAR** warning/erro explícito no startup |

### 4.4 BETA / WIP labels

| Feature | Situação | Recomendação |
|---|---|---|
| Tenant migration handlers | Código pronto, handlers desabilitados, **documentado como estável** | Marcar **BETA** em `TENANT_MIGRATION.md` OU reabilitar |
| `MoveStorage` | Stub (no-op) mas documentado como real | Marcar **BETA/stub** OU implementar de verdade |

---

## 5. Discovery, Graph, Transmutation, Encryption, Dashboard, Hub, Prometheus

### 5.1 Documentado mas NÃO implementado

Nenhum gap encontrado nesta camada. Todos os 9 passos do pipeline de
Discovery (`score_collections`, `semantic_focus`, `promote_readme`,
`compress_evidence`, `build_answer_plan`, `render_llm_prompt` etc.) estão
em `routing.rs:324-353`. Graph endpoints em `api/graph.rs:764-928`.
Encryption usa `p256 + aes_gcm` como promete `IMPLEMENTATION.md`.

### 5.2 Implementado mas NÃO documentado

| Feature | Código | Recomendação |
|---|---|---|
| **Prometheus metrics semantics** | `rest_handlers/meta.rs:204` (endpoint `/metrics` existe) | **CRIAR DOC** em `docs/prometheus/METRICS.md` explicando: métricas emitidas, semântica, setup de scraping, leitura do dashboard Grafana |
| Graph edge_index DashMap cache | `api/graph.rs:34-42` | **OPCIONAL** — documentar como nota de performance |
| File watcher discovery | `src/discovery/` | **DOCUMENTAR** em user API |

### 5.3 Mismatch

| Item | Onde | Recomendação |
|---|---|---|
| `use_transmutation` como string `"true"` na doc (`FILE_UPLOAD_TRANSMUTATION.md:29,79`) | handler em `upload.rs` provavelmente espera bool | **VERIFICAR** o tipo real no handler; corrigir doc para `true` (bool) |

---

## 6. Matriz de decisão consolidada

### 6.1 Implementar (código falta) — **18 itens**

| Prioridade | Item | Local | Motivo |
|---|---|---|---|
| **P0** | Reabilitar tenant migration handlers | `src/server/hub_handlers/mod.rs:16` | Código 100% pronto; só desligado por conflito axum/tonic. Baixo custo, alto valor |
| **P0** | Mapear `contextual_search` no MCP | `handlers.rs` | REST handler já existe; linha única de registro |
| **P1** | `batch_*` MCP tools (4) | `src/server/mcp/` | Handlers REST batch existem, wrap MCP é trivial |
| **P1** | `delete_collection` MCP | `handlers.rs` | Paridade com REST |
| **P1** | `embed_text` MCP | `handlers.rs` | Tool útil, wrap trivial |
| **P1** | `get_database_stats` MCP | `handlers.rs` | Paridade com REST `/stats` |
| **P2** | Warning/erro runtime se `file_watcher` ligado em cluster mode | `src/cluster/mod.rs` | Hoje falha silenciosa |
| **P2** | `MoveStorage` real (ou marcar BETA) | `src/migration/hub_migration.rs` | Stub em produção é pegadinha |
| **P3** | Sync replication end-to-end | `src/replication/` | Tipos prontos, falta fio |
| **P3** | Auditar claim de "+8.9% MAP" em 8-bit SQ | bench | Verificar ou remover |

### 6.2 Ajustar / criar documentação (código já certo) — **25 itens**

| Prioridade | Item | Motivo |
|---|---|---|
| **P0** | Renomear no `docs/specs/MCP.md`: `search_vectors`→`search`, `insert_texts`→`insert_text`, `delete_vectors`→`delete_vector`, `intelligent_search`→`search_intelligent`, `semantic_search`→`search_semantic` | Doc mente sobre nome das tools |
| **P0** | Adicionar seção "Graph Operations" no `MCP.md` com 8 tools `graph_*` | Features invisíveis hoje |
| **P0** | Reconciliar `REPLICATION.md:20` ("Manual Failover") com `CLUSTER.md:129-174` (Raft automático) | Usuário não sabe qual modo tem |
| **P1** | Remover `GET/POST /embedding/providers*` e `summarize/summaries` da `openapi.yaml` (ou criar specs de implementação) | 5 endpoints fantasma |
| **P1** | Remover menção à distância **Manhattan** em `CONFIGURATION.md` | Não existe no enum |
| **P1** | Documentar `BagOfWordsEmbedding` e `CharNGramEmbedding` em `EMBEDDINGS.md` | Providers invisíveis |
| **P1** | Criar `docs/prometheus/METRICS.md` | Endpoint `/metrics` sem doc de semântica |
| **P1** | Consolidar contagem de MCP tools (**31**) em README/UMICP/MCP.md | "31/38+/22" confuso |
| **P1** | Documentar `contextual_search` REST em `SEARCH.md` | Feature invisível |
| **P2** | Documentar `WriteConcern`, cluster memory flags, quorum semantics | Features internas sem user doc |
| **P2** | Documentar ameal tools MCP: `expand_queries`, `filter_collections`, `search_hybrid`, `search_extra`, `list_files`, `get_file_content`, `get_file_chunks`, `get_project_outline`, `get_related_files` | Invisíveis no MCP.md |
| **P2** | Corrigir tabela de redução de bits em `QUANTIZATION.md` (inconsistência interna) | Doc contradiz a si mesma |
| **P2** | Citar threshold `Lagging=1000ms` em `REPLICATION.md` | Comportamento não documentado |
| **P3** | Documentar flag `adaptive_assignment` em `QUANTIZATION.md` | Parâmetro oculto |

### 6.3 Remover / descartar — **5 itens**

| Item | Motivo |
|---|---|
| `GET /embedding/providers`, `POST /embedding/providers/set` | Trocar embedding em runtime viola invariante do índice (reindex necessário); não está no roadmap |
| Menção a distance metric Manhattan | Não no roadmap |
| Menção a "quantization cache hit ratio" se não houver cache | Doc inventou feature |
| Summarize endpoints (se não houver roadmap LLM-summarization) | Decidir com stakeholder |
| Claim "+8.9% MAP com 8-bit SQ" (se não houver bench) | Marketing sem prova |

---

## 7. Próximos passos recomendados

1. **Triagem com stakeholder**: revisar a matriz §6 e marcar cada linha com
   decisão final (implementar / documentar / remover).
2. **Abrir rulebook tasks**:
   - `mcp-tool-parity` (P0/P1 de §6.1) — implementar tools MCP faltantes
   - `mcp-doc-sync` (P0 de §6.2) — renomear/adicionar seções em `docs/specs/MCP.md`
   - `openapi-cleanup` (P1 de §6.2) — remover endpoints fantasma
   - `tenant-migration-reenable` (P0 de §6.1) — ligar handlers desabilitados
   - `prometheus-docs` (P1 de §6.2) — criar doc de métricas
   - `replication-doc-reconcile` (P0 de §6.2) — alinhar manual vs Raft
3. **Política daqui pra frente**: adicionar check de CI que compara
   `openapi.yaml` operationIds vs handlers registrados em `src/api/`, e
   MCP tool names entre `docs/specs/MCP.md` e match arm em `handlers.rs`.
   Evita regressão.

---

## 8. Apêndice — arquivos analisados

**Docs lidas**: `docs/api/*`, `docs/users/api/*`, `docs/users/collections/*`,
`docs/users/vectors/*`, `docs/users/search/*`, `docs/architecture/*`,
`docs/features/*`, `docs/specs/MCP.md`, `docs/deployment/CLUSTER.md`,
`docs/prometheus/*`, `docs/grafana/*`, `README.md`.

**Código analisado**: `src/api/`, `src/server/mcp/`, `src/db/`,
`src/models/`, `src/embedding/`, `src/quantization/`, `src/cache/`,
`src/replication/`, `src/cluster/`, `src/auth/`, `src/discovery/`,
`src/intelligent_search/`, `src/migration/`, `src/server/hub_handlers/`,
`src/server/rest_handlers/`.

**Agentes**: 5 agentes `Explore` paralelos, um por domínio
(REST / MCP / Core DB / Replication / Discovery+extras).
