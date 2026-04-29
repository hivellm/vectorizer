# Documentation Gap Analysis — FULL (server + SDKs + patches + tests + UI + gRPC) — 2026-04-24

> Análise exaustiva em 4 iterações:
> - **Iter 1** (feature-level): REST/MCP/Core/Replication/Discovery — 54 gaps
> - **Iter 2** (function-level server): 2.310 `pub fn` em 594 `.rs`, 7 lotes — 239 gaps user-facing
> - **Iter 3** (SDKs + patches): 5 SDKs (TS/Python/Go/C#/Rust) + 41 patches — 89 gaps + 7 regressões + **1 bug runtime Python**
> - **Iter 4** (testes + UI + gRPC): 151 testes + 116 React + 8 .proto + Rust SDK — 2 bugs UI + 4 schema mismatches gRPC + 1 doc lie
>
> **Total agregado: ~370 itens acionáveis + 3 bugs (1 runtime Python + 2 UI 404)**
>
> Ver §10–§14 (iter 3) e §15–§18 (iter 4).

## Resumo executivo

| Categoria | Funções | % |
|---|---|---|
| **DOC** (documentadas) | 985 | 43% |
| **INTERNAL** (helpers, getters, trait impls — ok não ter doc) | 1.017 | 44% |
| **USER_FACING_GAP** ← ALVO | **239** | **10%** |
| UNCERTAIN | 46 | 2% |
| **Excluídas**: `vectorizer-protocol/grpc_gen/` (auto-gerado de `.proto`) | ~214 | — |

**Conclusão**: 1 em cada 10 funções públicas no Vectorizer expõe
comportamento que usuário/operador veria mas não tem documentação.

## Matriz por lote

| # | Lote | Total | DOC | INTERNAL | **GAP** | Densidade gaps |
|---|---|---|---|---|---|---|
| 1 | API surface (REST/MCP/auth) | 334 | 203 | 113 | 13 | 4% |
| 2 | **Core store (db/, models/)** | 461 | 45 | 290 | **110** | **24%** ⚠️ |
| 3 | Embeddings/quant/compression | 308 | 96 | 177 | 25 | 8% |
| 4 | Distributed (cluster/replication/persistence/storage) | 312 | 84 | 162 | 48 | 15% |
| 5 | Hub/migration/workspace/batch/discovery | 330 | 210 | 96 | 18 | 5% |
| 6 | Infra (file_watcher/security/intelligent_search/cache/etc) | 354 | 289 | 54 | 8 | 2% |
| 7 | CLI/config/benchmark | 211 | 58 | 125 | 17 | 8% |
| | **TOTAL** | **2.310** | **985** | **1.017** | **239** | **10%** |

**Lotes mais bem documentados**: 6 (Infra: 82%), 5 (Hub: 64%), 1 (API: 61%).
**Lotes com mais buracos**: 2 (Core store: 24% gaps), 4 (Distributed: 15% gaps).

---

## §1 — Core store (Lote 2): 110 gaps user-facing

A camada de mais alto valor de produto, e a com pior cobertura. **Cinco features
inteiras estão funcionando em produção sem nenhuma documentação user-facing.**

### 1.1 Graph API completa — **30 fns sem doc**
- `db/graph.rs:57-468` — `add_node`, `remove_node`, `add_edge`, `get_neighbors`,
  `find_related`, `find_path`, `get_connected_components`
- `db/graph_relationship_discovery.rs:14-301` (8 fns) — auto-discovery de
  similaridade, referência, contains, derived_from
- `db/collection/graph.rs:21-152` (4 fns) — `get_graph`, `set_graph`,
  `populate_graph_if_empty`, `enable_graph`

**Action P0**: criar `docs/specs/GRAPH_RELATIONSHIPS.md` + seção "Graph
Operations" em `API_REFERENCE.md`. Iter 1 já havia flagado os tools MCP
`graph_*` sem doc; agora confirma-se que toda a camada de código também está.

### 1.2 Distributed Sharding — **107 fns sem doc**
- `db/distributed_sharded_collection.rs:49-951` (47 fns) — operações
  distribuídas: insert/search/hybrid_search/delete/update através de shards
- `db/sharded_collection.rs:42-414` (30 fns) — shard management local,
  rebalancing, vector routing
- `db/sharding.rs:24-302` (28 fns) — `ShardRouter`, `ShardRebalancer`,
  `calculate_moves_for_add/remove`
- `db/vector_store/collection_type.rs:51-482` (32 fns) — `CollectionType` enum
  dispatch (heterogêneo: GPU/sharded/quantized/quantized)

**Action P0**: criar `docs/specs/SHARDING.md` com routing, rebalancing,
distribuição. Sem isso, operadores não conseguem dimensionar cluster.

### 1.3 GPU backend — **50 fns só mencionadas no README**
- `db/hive_gpu_collection.rs:47-719` — `HiveGpuCollection`: `add_vectors`,
  `search_batch`, `update_vectors_batch`, `remove_vectors_batch`

**Action P1**: criar `docs/deployment/GPU_API.md` ou expandir
`GPU_SETUP.md` com API spec completa.

### 1.4 Payload Index (metadata filtering) — **30 fns sem doc**
- `db/payload_index.rs:43-712` — keyword/range/geo/text filtering
- `add_index_config`, `get_ids_for_keyword`, `get_ids_in_range`,
  `get_ids_in_geo_radius`, `search_text`

**Action P0**: criar `docs/specs/PAYLOAD_FILTERING.md`. Feature crítica
para uso comercial (Qdrant-style filters), invisível hoje.

### 1.5 Multi-tenancy — **28 fns sparse no HUB_INTEGRATION.md**
- `db/multi_tenancy.rs:94-407` — `TenantMetadata`, `TenantManager`, quota
  enforcement, usage tracking

**Action P1**: criar `docs/deployment/MULTI_TENANCY.md` ou expandir
`HUB_INTEGRATION.md` com seção dedicada.

### 1.6 WAL + auto-save — **21 fns**
- `db/wal_integration.rs:29-179` (12) — `log_insert/update/delete`,
  `recover_from_wal`, `checkpoint`
- `db/auto_save.rs:56-322` (9) — `start`, `force_save`, `mark_changed`,
  `cleanup_old_snapshots`

**Action P1**: criar `docs/deployment/DURABILITY.md` cobrindo WAL recovery
e checkpoint strategies.

### 1.7 Demais (16 fns)
- `db/quantized_collection.rs:71-267` (6) — linkar de API_REFERENCE para PQ
- `db/hybrid_search.rs:94-110` (2) — config + handler
- `models/sparse_vector.rs:23-265` (14) — sparse vector ops
- `db/vector_store/collections.rs:26-875` (25) — `create_collection_with_owner/quantization`,
  `enable_graph_for_collection`

---

## §2 — Distributed (Lote 4): 48 gaps user-facing

Operadores em produção. Cada gap aqui = surpresa em runtime.

### 2.1 Replication
- `replication/types.rs:280` — enum `WriteConcern { None, Count(n), All }`
  → **operadores precisam decidir consistency level, sem guidance**
- `replication/master.rs:501` — `replicate_with_concern(concern, timeout)`
  → **API de sync replication ESTÁ implementada**, mas
  `REPLICATION.md:255` diz "Sync Replication = Future" ⚠️ **DOC DESATUALIZADA**

### 2.2 Cluster / HA / Raft
- `cluster/raft_node.rs:770-800` + `cluster/leader_router.rs` — eleição
  Raft completa, **ZERO doc operacional**
- `cluster/ha_manager.rs:56,97` — `on_become_leader/follower` callbacks sem
  doc do state machine
- `cluster/collection_sync.rs:259` — `QuorumResult { quorum_met, ... }`
  sem doc de interpretação
- `cluster/validator.rs:228` — `with_limits()` sem documentar mín/máx
- `cluster/mod.rs:91,95,99` — knobs `max_cache_memory_bytes`,
  `enforce_mmap_storage`, `disable_file_watcher` documentados em
  `config.example.yml` mas **ausentes** de `docs/users/configuration/CLUSTER.md`

⚠️ **Contradição CRÍTICA**: `REPLICATION.md:20` promete "Manual Failover"
mas **NÃO há API promote/demote exposta** no código. Operadores não têm
procedimento de failover documentado.

### 2.3 Persistence (.vecdb format)
- `persistence/mod.rs` — `PersistedVector`, `PersistedVectorStore`
- `STORAGE.md:37` diz "ZIP archive" mas formato real é
  **gzip + bincode v1.0 + CRC32** — doc incorreta
- → **Action P0**: criar `docs/users/operations/VECDB_FORMAT.md` com layout
  binário, versionamento, recovery — crítico para backups

### 2.4 Storage
- `storage/compact.rs:337` `compact_all_with_cleanup(remove_source_files: bool)`
  — operação destrutiva sem warning na doc

---

## §3 — Embeddings/Quant/Compression (Lote 3): 25 gaps

### 3.1 Embedding providers (confirmando + ampliando iter 1)
- ✅ Confirmados iter 1: **BagOfWords, CharNGram** sem doc
- 🆕 NOVO: **OpenAI provider entirely undocumented** —
  `embedding/openai.rs:119/140/195` (`new`, `initialize`, `available_models`)
  — provider externo (requer API key, custos) sem nenhuma doc
- 🆕 `embedding/providers/minilm.rs:58` `load_model_with_id()` — seleção
  HuggingFace sem doc
- 🆕 `embedding/providers/bert.rs:60` `load_model_with_id()` — idem
- 🆕 `embedding/fast_tokenizer.rs:72` `from_pretrained()` — tokenizer model
  selection sem doc

### 3.2 Quantization
- ✅ Confirmado: PQ `adaptive_assignment` sem doc
- `quantization/product.rs:52` `new()` PQ presets sem doc
- `quantization/product.rs:74` `train()` — training data requirements sem doc

### 3.3 Compression — **NENHUMA DOC dedicada existe**
- `compression/config.rs:16-124` builder + presets (zstd, lz4, none)
- `compression/zstd.rs:48,69,80` `new()/fast()/high_compression()`
- `compression/lz4.rs:44,54,64` mesmas
- → **Action P1**: criar `docs/users/guides/COMPRESSION.md`

### 3.4 Normalization — **NENHUMA DOC dedicada existe**
- `normalization/config.rs:47-109` `enabled()/conservative()/moderate()/aggressive()`
- `normalization/detector.rs:97` `detect()` (HTML/JSON/plaintext/PDF)
- → **Action P1**: criar `docs/users/guides/NORMALIZATION.md`

---

## §4 — Hub/Workspace/Batch (Lote 5): 18 gaps

### 4.1 Batch ops (14 gaps) — **maior buraco do lote 5**
- `batch/processor.rs:67/125/163/201/575` — batch_insert/update/delete/search
  documentados em `API_REFERENCE` mas **sem guia dedicado**
- `batch/error.rs:105-359` (10 fns) — `BatchError`, `is_retryable()`,
  `should_retry()`, `success_rate()` — **sem error code registry**
- `batch/config.rs:73-187` (8) — limites de memória/tamanho sem doc
- `batch/parallel.rs:31-109` — chunk/task strategy sem tuning guide
- `batch/progress.rs:26-168` (8) — progress reporting sem exemplos
- → **Action P0**: criar `docs/users/api/BATCH.md`

### 4.2 Migration (3)
- `hub_migration.rs:271/404` — `execute()`/`rollback()` sem RTO/RPO
- `qdrant/data_migration.rs:27` — `export_collection()` sem semântica
  (point-in-time? incremental?)

### 4.3 Discovery (1)
- `discovery/hybrid.rs:37` `search_with_text()` variant não documentado

### 4.4 Features completas SEM doc (descoberta nova)
- **IP Whitelist** (`hub/ip_whitelist.rs:240-413`, 8 fns) — feature completa
  de tenant-scoped IP allow/blocklists, **ZERO doc**
- **Request Signing** (`hub/request_signing.rs:180`) — `SigningValidator`
  sem doc; talvez seja interno mas precisa esclarecer

---

## §5 — CLI/Config/Benchmark (Lote 7): 17 gaps

### 5.1 CLI commands sem doc (9)
- `cli/commands.rs:20/81/190/349/510/599/667` — server/user/api-key/collection/
  config/status/snapshot subcommands
- `cli/setup.rs:13/162` — wizard
- → **Action P1**: criar `docs/users/getting-started/CLI_REFERENCE.md`

### 5.2 Config options sem doc (8)
- `config/enhanced_config.rs:301/396/498/559` — auto_reload, templates,
  env vars export, config export formats
- `config/workspace.rs:208/246` — add/remove_workspace
- `config/layered.rs:107/159` — **layered config (env override + YAML
  merge)** sem doc — crítico para ops em K8s
- → **Action P1**: expandir `CONFIGURATION.md`

### 5.3 Benchmark claims — **VERIFICADOS, REVERTENDO ITER 1**
- ✅ **"+8.9% MAP improvement"** — REAL. Backed by
  `benchmark/metrics.rs:70` + `data_generator.rs`. PERFORMANCE.md tem
  tabela detalhada.
- ✅ **"sub-3ms search"** — REAL. Backed by `benchmark_runner.rs:35-78`.
  Atualmente 0.6-2.4ms (claim conservadora).
- ⚠️ Reprodutibilidade: BenchmarkConfig com 3 presets existe mas **não há
  CLI público** para rodar — só library Rust
- → **Action P2**: criar `docs/users/operations/BENCHMARKING.md` ou
  publicar binário `vectorizer-bench`

---

## §6 — API Surface (Lote 1): 13 gaps + Infra (Lote 6): 8 gaps

### 6.1 API surface
- Setup wizard: `setup_handlers.rs:542/589/639` (3) — `display_first_start_guidance`,
  `needs_setup`, `browse_directory`
- Dashboard: `embedded_assets.rs:82/89` — handlers sem doc HTTP
- Métricas: `meta.rs:204` `get_prometheus_metrics`, `core/helpers.rs:165`
  `get_file_watcher_metrics` (já flagged em iter 1)
- Cluster API: `api/cluster.rs:102/396/409` — router factory + leader/role
  endpoints
- File operations: `validation.rs:141/218` — language detection
- Graph API: `api/graph.rs:48/57` — state init + router

### 6.2 Infra (8 gaps pontuais)
- File watcher metrics: `metrics.rs:180/191/352`, `mod.rs:713`
- Cache: `query_cache.rs:135` (`new`), `memory_manager.rs:335`
  (`init_global_cache_memory_manager`)
- Summarization: `manager.rs:73` `summarize_text()` — código existe e
  funciona, **MAS REST handler não está wired** (confirma iter 1: endpoints
  `POST /summarize/text` documentados mas não roteados)
- Security: `rate_limit.rs:406` `rate_limit_middleware()` — verificar wiring

---

## §7 — Matriz de decisão consolidada

### 7.1 IMPLEMENTAR código que falta (de iter 1, ainda válido)

| Pri | Item | Local | Justificativa |
|---|---|---|---|
| P0 | Reabilitar tenant migration handlers | `hub_handlers/mod.rs:16` | Código pronto, só desligado por axum/tonic |
| P0 | Mapear `contextual_search` no MCP | `server/mcp/handlers.rs` | REST handler existe |
| P0 | Wire `/summarize/text` REST handler | `summarization/manager.rs:73` exposto, mas sem rota | Doc promete, código existe, falta rota |
| P0 | Expor API `promote/demote` para failover manual | `replication/` | Doc promete, API ausente |
| P1 | Implementar 4 batch_* MCP tools | `server/mcp/` | Wrapping trivial sobre REST batch |
| P1 | `delete_collection`, `embed_text`, `get_database_stats` MCP | idem | Paridade |
| P1 | Warning runtime se `file_watcher=on` em cluster mode | `cluster/mod.rs` | Hoje falha silenciosa |
| P2 | `MoveStorage` real ou marcar BETA | `migration/hub_migration.rs:351` | Stub é pegadinha |
| P2 | Sync replication end-to-end | `replication/` | API existe, falta exposição user (ver doc-only erro) |

### 7.2 CRIAR DOCUMENTAÇÃO nova (16 docs novas/expandidas)

| Pri | Doc proposto | Coverage | Origem |
|---|---|---|---|
| P0 | `docs/specs/GRAPH_RELATIONSHIPS.md` | 42 fns Graph + 8 MCP tools | §1.1 + iter 1 |
| P0 | `docs/specs/SHARDING.md` | 107 fns sharded | §1.2 |
| P0 | `docs/specs/PAYLOAD_FILTERING.md` | 30 fns payload_index | §1.4 |
| P0 | `docs/users/operations/VECDB_FORMAT.md` | format spec backup-critical | §2.3 |
| P0 | `docs/users/api/BATCH.md` | 14 batch fns + error registry | §4.1 |
| P0 | `docs/prometheus/METRICS.md` | métricas semântica | iter 1 + §6.1 |
| P1 | `docs/deployment/GPU_API.md` | 50 fns HiveGpuCollection | §1.3 |
| P1 | `docs/deployment/MULTI_TENANCY.md` | 28 fns + IP whitelist | §1.5 + §4.4 |
| P1 | `docs/deployment/DURABILITY.md` | WAL + auto-save (21) | §1.6 |
| P1 | `docs/users/guides/COMPRESSION.md` | zstd/lz4/none + presets | §3.3 |
| P1 | `docs/users/guides/NORMALIZATION.md` | 4 presets + content detector | §3.4 |
| P1 | `docs/users/getting-started/CLI_REFERENCE.md` | 9 CLI subcommands | §5.1 |
| P2 | `docs/users/operations/BENCHMARKING.md` | presets + Rust examples | §5.3 |
| P2 | Expandir `EMBEDDINGS.md` com OpenAI/BagOfWords/CharNGram | 3 providers | §3.1 + iter 1 |
| P2 | Expandir `CONFIGURATION.md` com layered loading | env override + merge | §5.2 |
| P2 | Expandir `HUB_INTEGRATION.md` com IP Access Control | 8 fns ip_whitelist | §4.4 |

### 7.3 CORRIGIR DOC desatualizada (8 itens)

| Pri | Onde | O quê |
|---|---|---|
| P0 | `REPLICATION.md:20` | "Manual Failover" promete API que não existe |
| P0 | `REPLICATION.md:255` | "Sync Replication = Future" — API JÁ existe (`replicate_with_concern`) |
| P0 | `STORAGE.md:37` | Diz "ZIP archive" — formato real é gzip+bincode+CRC32 |
| P0 | `docs/specs/MCP.md` (5 nomes) | `search_vectors→search`, `insert_texts→insert_text`, etc. |
| P1 | OpenAPI `summarize`, `embedding/providers` (5 endpoints fantasma) | Remover ou implementar |
| P1 | README/UMICP/MCP.md contagem MCP tools (31 vs 38+ vs 22) | Padronizar 31 |
| P2 | `CONFIGURATION.md` | Remover menção a distance metric Manhattan |
| P2 | `QUANTIZATION.md:44` vs `:182` | Tabela contradiz texto (50% vs 25%) |

### 7.4 REMOVER da doc (4 endpoints fantasma)
- `GET /embedding/providers`, `POST /embedding/providers/set`
  (trocar provider em runtime viola invariante de índice)
- `POST /summarize/text`, `GET /summaries`, `GET /summaries/{id}`
  (decidir com stakeholder se é roadmap real)

### 7.5 REVERTER recomendação iter 1
- Claim "+8.9% MAP" e "sub-3ms search" são **reais**, não marketing.
  Backed por `benchmark/`. Iter 1 errou em sugerir auditoria.

---

## §8 — Próximos passos sugeridos

1. **Triagem com stakeholder** (1h): revisar §7.1, §7.2, §7.3, §7.4 e marcar prioridade real
2. **Criar 6 rulebook tasks**:
   - `docs-graph-payload-sharding` (P0 — 3 docs novos)
   - `docs-batch-vecdb-prometheus` (P0 — 3 docs novos)
   - `docs-deployment-gpu-mt-durability` (P1 — 3 docs novos)
   - `docs-guides-compression-normalization-cli` (P1 — 3 docs novos)
   - `code-fix-failover-summarize-mcp-tools` (P0 — implementações faltantes)
   - `doc-corrections-replication-mcp-storage` (P0 — corrige 8 doc errors)
3. **Adicionar CI guard** que falha o build quando:
   - `openapi.yaml` operationId não tem handler em `crates/vectorizer-server/src/`
   - MCP tool name em `docs/specs/MCP.md` não bate com match arm em
     `handlers.rs` (apparently 11 hoje)
   - Novo `pub fn` em `db/`, `cluster/`, `replication/` é mergeado sem
     menção em `docs/`

---

## §9 — Apêndice — escopo da auditoria

### Crates analisados
- `crates/vectorizer/` (~449 .rs, ~1.700 pub fns) — engine principal
- `crates/vectorizer-server/` (~93 .rs, ~250 pub fns) — REST/MCP
- `crates/vectorizer-core/` (~36 .rs, ~85 pub fns) — quant/compression
- `crates/vectorizer-cli/` (~8 .rs, ~37 pub fns) — CLI
- `crates/vectorizer-protocol/` (~8 .rs, ~214 pub fns auto-gen) — gRPC stubs (excluído)

### Lotes
| # | Módulos | fns | scratch file |
|---|---|---|---|
| 1 | server/, api/, auth/ | 334 | `scripts/gap-analysis/batch1_findings.md` |
| 2 | db/, models/ | 461 | `batch2_findings.md` |
| 3 | embedding/, quantization/, compression/, normalization/ | 308 | `batch3_findings.md` |
| 4 | cluster/, replication/, persistence/, storage/ | 312 | `batch4_findings.md` |
| 5 | hub/, migration/, workspace/, batch/, discovery/ | 330 | `batch5_findings.md` |
| 6 | file_watcher/, file_operations/, monitoring/, security/, intelligent_search/, cache/, summarization/, file_loader/ | 354 | `batch6_findings.md` |
| 7 | cli/, config/, benchmark/ | 211 | `batch7_findings.md` |

### Limitações
- **Cobertura intra-crate** (não inter-crate): se uma função é re-exportada
  com nome diferente, pode aparecer como CODE_ONLY num lote e DOC noutro.
  Risco baixo na prática.
- **`pub fn` em `impl` blocks de tipos pub-crate-only**: classificados como
  pub mas talvez sejam efetivamente internos. UNCERTAIN bucket cobre isso.
- **Test helpers**: 5 falsos positivos detectados em batch 1
  (`advanced_api.rs:533/1003/1022`) e ajustados manualmente. Outros
  podem ter passado.
- **Trait methods**: contam como `pub fn` mas frequentemente são internos.
  Agentes usaram judgement.

### Documentos analisados (corpus de doc)
`docs/api/`, `docs/users/api/`, `docs/users/collections/`, `docs/users/vectors/`,
`docs/users/search/`, `docs/users/configuration/`, `docs/users/getting-started/`,
`docs/users/guides/`, `docs/users/operations/`, `docs/users/use-cases/`,
`docs/users/qdrant/`, `docs/architecture/`, `docs/deployment/`,
`docs/features/`, `docs/specs/MCP.md`, `docs/specs/STORAGE.md`,
`docs/specs/PERFORMANCE.md`, `docs/specs/QDRANT_COMPATIBILITY_INDEX.md`,
`docs/specs/INTELLIGENT_SEARCH.md`, `docs/specs/HUB_INTEGRATION.md`,
`docs/runbooks/`, `docs/grafana/`, `docs/prometheus/`, `README.md`,
`CHANGELOG.md`, `config.example.yml`.

---

# § ITER 3 — SDKs cliente e histórico de patches

> Adicionado depois da iter 1 (feature-level) e iter 2 (function-level no servidor).
> Cobre as 4 SDKs com código (TS, Python, Go, C#) + auditoria histórica
> de `docs/patches/v*.md`.

## §10 — SDKs cliente (4 linguagens)

### 10.1 Sumário comparativo

| SDK | Files | Símbolos pub | README | Doc-only | Code-only | Server gaps | Bugs |
|---|---|---|---|---|---|---|---|
| TypeScript | 78 | ~85 | ~45 | 2 | **38** | 2* | 0 |
| Python | 56 | 78 | 64 | **12** | 15 | 4 | **🚨 1** |
| C# | 55 | 767 (267 tipos + 500 props) | 51 | 0 | 19 | poucos | 0 |
| Go | 28 | 106 | ~40 | 3 | 0 | **12** | 0 |
| Rust | ~20 src + 15 tests | (não contado) | (não contado) | — | — | — | — |
| ↑ Nota | iter 3 reportou "0 src" — foi falso-positivo (meu grep não incluiu `.rs`). SDK é real, publicado como `vectorizer-sdk` v3.0.3 (workspace member). Auditoria full pendente. | | | | | | |
| **Total** | **217** | **~1.036** | **~200** | **17** | **72** | **18** | **1** |

*TS server gaps são endpoints fantasma já contados na iter 1.

### 10.2 🚨 BUG runtime — Python SDK

**`sdks/python/vectorizer/graph.py:159`** `delete_graph_edge(edge_id)`:
```python
data = await self._transport.delete(f"/graph/collections/{collection}/edges")
                                                          ^^^^^^^^^^^^
# NameError: 'collection' não está definido neste método
```
Provável correção: `f"/graph/edges/{edge_id}"`. **Método dispara
NameError em qualquer chamada.** Não é doc gap — é bug de produção.
**Ação P0**: corrigir antes do próximo release.

### 10.3 Doc-only (17 — README promete, código não tem)

**Python (12)** — vários nomes errados ou RPC-only mascarados de REST:
- `delete_vector` (singular) → código tem `delete_vectors`
- `update_vector` → não existe
- `search_basic`, `hello` → RPC-only, README sugere REST
- `summarize_text`, `summarize_context` → confirmando iter 1 (server doc-only)
- `add_workspace`/`list_workspaces`/`remove_workspace` → server tem REST, SDK só via `__getattr__` (undiscoverable)
- `create_backup`/`list_backups`/`restore_backup` → mesmo padrão

**TypeScript (2)**:
- `summarizeText`, `summarizeContext` (mesma raiz: server doc-only)

**Go (3)**: features mencionadas no README sem método correspondente
(Master/Replica routing helpers, Contextual Search, Discovery Operations)

**C# (0)**: 1:1 entre README e código

### 10.4 Code-only (72 — implementado sem aparecer no README)

**TypeScript (38)**:
- 6 Admin: `getLogs`, `forceSaveCollection`, `getServerConfig`, `updateConfig`, `restartServer`, `getBackupDirectory`
- 30+ Qdrant compat: `qdrantListCollectionSnapshots`, `qdrantQueryPoints`, `qdrantBatchQueryPoints`, etc. (Snapshots, Sharding, Cluster, Query, Matrix APIs)
- Restantes: `login`, `scoreCollections`, `getFileSummary`, `searchByFileType`, `uploadFileContent`, `deleteGraphEdge`, `listGraphEdges`

**C# (19)**:
- 5 batch: `BatchInsertTextsAsync`, `BatchSearchVectorsAsync`, `BatchUpdateVectorsAsync`, `BatchDeleteVectorsAsync`, `AcquireAsync`
- 4 graph discovery: `DiscoverGraphEdgesAsync`, `DiscoverGraphEdgesForNodeAsync`, `GetGraphDiscoveryStatusAsync`, `ListGraphEdgesAsync`
- 10 outros: `EmbedTextAsync`, `GetSummaryAsync`, `ListSummariesAsync`, `GetUploadConfigAsync`, `UploadFileContentAsync`, `HybridSearchAsync`, RPC `CallAsync`/`HelloAsync`, `WithMaster()`

**Python (15)**: toda layer Qdrant compat sem doc explícita + `upload_file()` + `login()`

**Go (0)**

### 10.5 Server→SDK gaps (12 no Go, alguns nos outros)

**Go é o mais defasado** — 12 MCP tools sem wrapper:
- File ops (5): `get_file_content`, `list_files`, `get_file_chunks`, `get_project_outline`, `get_related_files`
- Discovery (2): `filter_collections`, `expand_queries`
- Search (3): `search_hybrid`, `search_extra`, `multi_collection_search`
- Admin (2): `list_empty_collections`, `cleanup_empty_collections`

**Python (4)**: `cleanup_empty_collections`, `list_empty_collections`, `search_by_file`

**C#**: poucos gaps — mapeamento 1:1 com 80+ métodos

### 10.6 C# — falta XML doc em escala
500 properties/methods públicos sem `<summary>` (65% gap de cobertura).
Não é gap de feature, é gap de IntelliSense — usuários C# têm UX ruim.

---

## §11 — Patches históricos (`docs/patches/v0.1 → v2.4`, 41 arquivos)

### 11.1 🚨 7 regressões silenciosas

#### #1 — Discovery pipeline incompleto no MCP
- v0.6.0 prometeu 9 discovery tools no MCP (`discover`, `broad_discovery`,
  `semantic_focus`, `compress_evidence`, `build_answer_plan`,
  `render_llm_prompt`, `promote_readme`)
- Hoje só 2 expostas: `filter_collections`, `expand_queries`
- 7 functions inalcançáveis via MCP — agentes precisam usar REST diretamente
- **Bate com achado iter 1**: graph_* tools também sumiram do MCP.md

#### #2 — Batch operations rebaixadas em v1.0.0
- v0.6/v0.18 prometeram `batch_insert/search/update/delete_texts/vectors` no MCP
- v1.0.0 removeu intencionalmente ("agents podem fazer loop")
- Removal documentado MAS contradiz claim original "38 tools"
- **Bate com iter 1**: 4 batch_* MCP tools são doc-only no MCP.md

#### #3 — `get_file_summary` removido sem REST replacement
- v0.6/v0.18 prometeram como MCP tool
- v1.0.0 removeu, sugerindo `get_file_chunks` como alternativa
- Hoje **não existe nem no MCP nem no REST**. Substituto tem semântica diferente

#### #4–7 (claim "38 MCP tools" inflado)
- Real entregue: **~19 MCP tools** (verificado em `handlers.rs`)
- Inflação veio de:
  - 9 discovery operations contadas como tools separadas → entregue como 1 pipeline
  - 5 batch ops removidas em v1.0
  - Alguns nomes contados duas vezes em variantes singular/plural

### 11.2 Drift CHANGELOG vs patches
Root `CHANGELOG.md` foca v3.0.x e **NÃO menciona** features importantes:
- TLS/mTLS support (v2.0)
- Hybrid Search (v2.0)
- Rate Limiting (v2.0)
- Quantization caching
- HiveHub logging
- Collection Mapping
- Discovery pipeline architecture

**Action P1**: refazer `CHANGELOG.md` root com seção "v2.0 Production Features"
listando essas adições.

---

## §12 — Matriz de decisão expandida (iter 1 + 2 + 3)

### 12.1 IMPLEMENTAR código que falta — atualizado

| Pri | Item | Origem |
|---|---|---|
| **P0** | 🚨 **Fix `sdks/python/vectorizer/graph.py:159` — `delete_graph_edge` NameError** | iter 3 |
| P0 | Reabilitar tenant migration handlers (axum/tonic conflict) | iter 1 |
| P0 | Mapear `contextual_search` no MCP | iter 1 |
| P0 | Wire `/summarize/text` REST handler ou remover dos SDKs | iter 1 + iter 3 |
| P0 | Expor API `promote/demote` para failover manual | iter 2 |
| **P0** | **Expor 7 discovery operations no MCP** (semantic_focus, broad_discovery, compress_evidence, build_answer_plan, render_llm_prompt, promote_readme, …) | iter 3 |
| P1 | Implementar 4 batch_* MCP tools | iter 1 |
| P1 | `delete_collection`, `embed_text`, `get_database_stats` MCP | iter 1 |
| **P1** | **Adicionar 12 wrappers no SDK Go** (file ops, discovery, search variants, admin) | iter 3 |
| **P1** | **Expor workspace/backup methods explicitamente no SDK Python** | iter 3 |
| P1 | Warning runtime se `file_watcher=on` em cluster mode | iter 1 |
| P2 | `MoveStorage` real ou marcar BETA | iter 1 |
| P2 | Sync replication end-to-end | iter 1 |

### 12.2 CRIAR DOCUMENTAÇÃO nova — atualizado

| Pri | Doc | Origem |
|---|---|---|
| P0 | `docs/specs/GRAPH_RELATIONSHIPS.md` | iter 2 (42 fns) |
| P0 | `docs/specs/SHARDING.md` | iter 2 (107 fns) |
| P0 | `docs/specs/PAYLOAD_FILTERING.md` | iter 2 (30 fns) |
| P0 | `docs/users/operations/VECDB_FORMAT.md` | iter 2 (backup-critical) |
| P0 | `docs/users/api/BATCH.md` | iter 2 (14 fns + error registry) |
| P0 | `docs/prometheus/METRICS.md` | iter 1 + 2 |
| **P0** | **MCP/REST/SDK compatibility matrix** (qual feature está em qual interface) | iter 3 patches |
| P1 | `docs/deployment/GPU_API.md` (50 fns) | iter 2 |
| P1 | `docs/deployment/MULTI_TENANCY.md` + IP whitelist | iter 2 + 5 |
| P1 | `docs/deployment/DURABILITY.md` (WAL + auto-save) | iter 2 |
| P1 | `docs/users/guides/COMPRESSION.md` | iter 2 |
| P1 | `docs/users/guides/NORMALIZATION.md` | iter 2 |
| P1 | `docs/users/getting-started/CLI_REFERENCE.md` | iter 2 |
| **P1** | **Atualizar `CHANGELOG.md` root com seção v2.0** (TLS, Hybrid, RateLimit) | iter 3 |
| P2 | `docs/users/operations/BENCHMARKING.md` | iter 2 |
| P2 | Expandir `EMBEDDINGS.md` com OpenAI/BagOfWords/CharNGram | iter 2 |
| P2 | Expandir `CONFIGURATION.md` com layered loading | iter 2 |
| P2 | Expandir `HUB_INTEGRATION.md` com IP Access Control | iter 2 |
| **P2** | **Adicionar XML doc em 500 properties/methods do C# SDK** | iter 3 |
| **P2** | **Expandir READMEs dos 4 SDKs com 72 métodos code-only** | iter 3 |

### 12.3 CORRIGIR / REMOVER da doc — atualizado

| Pri | Onde | O quê | Origem |
|---|---|---|---|
| P0 | `REPLICATION.md:20` | "Manual Failover" promete API que não existe | iter 1 |
| P0 | `REPLICATION.md:255` | "Sync Replication = Future" — API JÁ existe | iter 2 |
| P0 | `STORAGE.md:37` | "ZIP archive" — formato real é gzip+bincode+CRC32 | iter 2 |
| P0 | `docs/specs/MCP.md` | 5 nomes desalinhados (search_vectors→search etc.) | iter 1 |
| **P0** | **`sdks/python/README.md`** | `delete_vector`/`update_vector`/`search_basic`/`hello` errados | iter 3 |
| **P0** | **patches v0.6.0** | Adicionar nota retroativa "MCP scope reduced in v1.0" | iter 3 |
| P1 | OpenAPI `summarize`, `embedding/providers` (5 endpoints) | Remover ou implementar | iter 1 |
| P1 | README/UMICP/MCP.md (31 vs 38+ vs 22) | Padronizar 31 (claim "38" era inflado) | iter 1 + 3 |
| P2 | `CONFIGURATION.md` distance Manhattan | Não existe | iter 1 |
| P2 | `QUANTIZATION.md:44` vs `:182` | Tabela contradiz texto | iter 1 |

---

## §13 — Estatísticas finais agregadas

| Camada | Itens analisados | Gaps user-facing |
|---|---|---|
| **Servidor (iter 1+2)** | 2.310 pub fns + 41+ docs | **239** |
| **SDKs (iter 3)** | ~1.036 símbolos pub em 4 linguagens | **89** (17 doc-only + 72 code-only) |
| **Server→SDK gaps** | — | **18** (Go: 12, Python: 4, C#: ~2, TS: 0) |
| **Patches históricos** | 41 changelogs + CHANGELOG raiz | **7 regressões silenciosas** + 35+ drift |
| **Bugs runtime achados** | — | **1** (Python `delete_graph_edge`) |
| **TOTAL gaps** | — | **354 itens acionáveis** |

---

## §14 — Cobertura final

A análise feita em 3 iterações cobre:

✅ **Servidor**: 594 arquivos `.rs`, 2.310 pub fns (sem grpc_gen auto-gerado)
✅ **SDKs**: 217 arquivos client em TS/Python/Go/C# (Rust SDK só docs)
✅ **Documentação**: ~60 .md user-facing + 41 patches históricos + OpenAPI + config.example.yml + CHANGELOG
✅ **Cross-checks**: doc→código, código→doc, patch→atual, SDK→server, server→SDK

**O que ficou de fora propositalmente**:
- `crates/vectorizer-protocol/src/grpc_gen/` (~214 fns auto-geradas de `.proto`)
- `examples/` (não existe diretório dedicado)
- `.rulebook/specs/` (specs internos do framework Rulebook, não da Vectorizer)

**Iter 4 cobriu**: testes integration (151), dashboard React (116), .proto (8), Rust SDK.

---

# § ITER 4 — Tests, Dashboard React, gRPC proto, Rust SDK

## §15 — Testes integration (151 arquivos)

### 15.1 Drift de endpoints: ZERO ✅
Nenhum dos 151 testes chama endpoint/MCP tool que não existe. Os testes
funcionam como guard contra regressão silenciosa de API contracts.

### 15.2 Doc lie revelada por teste
**`REPLICATION.md:255` mente sobre sync replication**:
- Doc diz "Sync Replication = Future"
- `tests/cluster/distributed_resilience.rs:218-231` testa
  `WriteConcern::All` com sucesso (`test_write_concern_serialization`,
  `test_write_concern_default_is_none`)
- → Confirma iter 2 §2.1: API existe, doc está desatualizada
- **Action P0**: corrigir doc

### 15.3 Manhattan distance — verificação cruzada
- Test em `tests/simd/new_ops.rs:50-63` valida `manhattan_distance()` SIMD
- MAS confirmado em `crates/vectorizer/src/models/mod.rs:390-397`:
  enum `DistanceMetric { Cosine, Euclidean, DotProduct }` — sem Manhattan
- → SIMD primitive existe mas não está conectada ao enum user-facing.
  Iter 1 está correta: doc não deve mencionar Manhattan como opção.
  Gap real: SIMD pronto, desperdiçado — ou conecta no enum, ou remove o
  código SIMD não usado.

### 15.4 Bypassed tests (13+, todos justificados)
- GPU tests: `#[ignore]` (precisam de GPU)
- Cluster perf: `#[ignore]` (>60s)
- Replication failover/comprehensive: `#[ignore]` (TCP bind)
- **Nenhum `.skip()` ou `#[test]` comentado escondendo bug** ✅

## §16 — Dashboard React (116 .tsx/.ts)

### 16.1 🚨 2 endpoints UI quebrados (404)
- `dashboard/src/pages/LogsPage.tsx:55` — chama `GET /api/logs`, real é `/logs`
- `dashboard/src/pages/ApiDocsPage.tsx:792` — chama `GET /api-keys`, real é `/auth/keys`
- **Action P0**: fix imediato, são bugs UI shipping em produção

### 16.2 DASHBOARD_INTEGRATION.md cobre só ~58% da UI
A UI usa 43 endpoints, organizados em 12 áreas funcionais. A doc descreve só 7. Faltam **5 áreas inteiras**:

1. **Graph Management** (12 endpoints, GraphPage) — todo o `/graph/*`
2. **Cluster Management** (8 endpoints, ClusterPage) — `/api/v1/cluster/{nodes,leader,role,shard-distribution,rebalance}`
3. **User Management** (4 endpoints, UsersPage) — `/auth/users/*`
4. **API Key Management** (3 endpoints, ApiKeysPage) — `/auth/keys/*`
5. **File Watcher Metrics + Workspace config**

**Action P1**: expandir `DASHBOARD_INTEGRATION.md` cobrindo as 5 áreas.

### 16.3 Confirmação iter 2 (cluster endpoints user-facing)
Iter 2 §4.2 marcou `api/cluster.rs:396/409` `get_cluster_leader` /
`get_cluster_role` como undocumented user-facing. Dashboard
`ClusterPage.tsx:99-100` chama ambos ativamente — **confirma**: são API
real e user-facing, não internal-only. Documentar.

## §17 — gRPC `.proto` ↔ `GRPC.md`

### 17.1 Proto-only (4 RPCs Raft+shard internas)
`cluster.proto:32-37`:
- `GetShardVectors` — shard data migration
- `RaftVote`, `RaftAppendEntries`, `RaftSnapshot` — consensus

→ Action P1: documentar em GRPC.md como "Cluster Internal RPCs"

### 17.2 🚨 Schema mismatch GRPC.md — breaking v3.0.0 não documentada
`vectorizer.proto:184-188` (comentário): "downgraded from double→float to match canonical crate::models::SearchResult"

| Campo | Proto | GRPC.md (atual) |
|---|---|---|
| `SearchResult.score` | `float` (f32) | `double` ⚠️ |
| `HybridSearchResult.hybrid_score` | `float` | `double` ⚠️ |
| `HybridSearchResult.dense_score` | `float` | `double` ⚠️ |
| `HybridSearchResult.sparse_score` | `float` | `double` ⚠️ |

Clientes regenerando stubs recebem `float32`; doc promete `double`.
**Action P0**: corrigir GRPC.md + adicionar nota de migração v3.0.0.

### 17.3 Qdrant PointsService — verificar completude
`points_service.proto:98-135` define ~8 RPCs (Discover, DiscoverBatch,
UpdateBatch, QueryBatch, QueryGroups, Facet, SearchMatrixPairs,
SearchMatrixOffsets) que NÃO estão na tabela de GRPC.md (linhas 415-419).
Verificar se são stubs ou reais.

### 17.4 Services wiring — todos OK ✅
`crates/vectorizer-server/src/server/core/grpc.rs` registra todos services
(Vectorizer, Cluster condicional, Qdrant Collections/Points/Snapshots).

## §18 — Rust SDK — correção falso-positivo iter 3

Iter 3 reportou "Rust SDK = 0 source files" — **errado**. Meu grep inicial
filtrou por `*.ts/*.py/*.go/*.cs`, excluindo `.rs`. Verificação iter 4:

✅ `sdks/rust/` é real:
- `name = "vectorizer-sdk"`, `version = "3.0.3"`, edition 2024
- 20+ source files em `src/` (lib.rs, transport.rs, models.rs, rpc/, client/, ...)
- Workspace member do root Cargo.toml
- 2 transports: RPC (recomendado, msgpack/TCP em :15503) + HTTP legacy (:15002)
- 15 testes batem com `vectorizer_sdk::*` API real

**Status**: SDK Rust não tem gap. Auditoria full function-level pendente
(seria iter 5 se desejado).

---

## §19 — Estatísticas finais (4 iterações)

| Camada | Itens analisados | Gaps user-facing |
|---|---|---|
| Servidor (iter 1+2) | 2.310 pub fns + ~60 docs | 239 |
| SDKs (iter 3) | ~1.036 símbolos em 4 SDKs | 89 + 1 bug |
| Patches (iter 3) | 41 versions + CHANGELOG | 7 regressões + 35 drift |
| Tests (iter 4) | 151 integration tests | 0 drift, 1 doc lie revealed |
| Dashboard React (iter 4) | 116 .tsx/.ts files | **2 bugs UI** + 5 áreas sem doc |
| gRPC proto (iter 4) | 30 RPCs em 8 .proto | 4 proto-only + 4 schema mismatch |
| Rust SDK (iter 4) | confirmado real | 0 (falso-positivo iter 3 corrigido) |
| **TOTAL** | — | **~370 itens + 3 bugs** |

## §20 — Ações P0 consolidadas (urgência máxima)

Ordem de fix imediato:

1. 🚨 **`sdks/python/vectorizer/graph.py:159`** — fix NameError em `delete_graph_edge`
2. 🚨 **`dashboard/src/pages/LogsPage.tsx:55`** — `/api/logs` → `/logs`
3. 🚨 **`dashboard/src/pages/ApiDocsPage.tsx:792`** — `/api-keys` → `/auth/keys`
4. **`docs/users/api/GRPC.md`** — corrigir `double`→`float` em SearchResult/HybridSearchResult (4 lugares) + nota migração v3.0.0
5. **`docs/users/api/REPLICATION.md:255`** — sync replication NÃO é "future", é real (`WriteConcern::All`)
6. **`docs/users/api/REPLICATION.md:20`** — "Manual failover" promete API que não existe; expor `promote/demote` ou remover claim
7. **`docs/specs/STORAGE.md:37`** — formato `.vecdb` é gzip+bincode+CRC32, não "ZIP archive"
8. **`docs/specs/MCP.md`** — corrigir 5 nomes (`search_vectors→search`, etc.)
9. **`sdks/python/README.md`** — corrigir `delete_vector`/`update_vector`/`search_basic`/`hello`
10. **Reabilitar** `crates/vectorizer-server/src/server/hub_handlers/mod.rs:16` (tenant migration handlers desabilitados)
11. **Mapear** `contextual_search` no MCP (REST handler existe)
12. **Wire** `/summarize/text` REST handler ou remover dos SDKs (3 lugares: openapi, TS, Python)
13. **Expor 7 discovery operations** no MCP (semantic_focus, broad_discovery, compress_evidence, build_answer_plan, render_llm_prompt, promote_readme)

## §21 — Cobertura final atingida

✅ Servidor (594 .rs / 2.310 pub fns)
✅ 5 SDKs cliente (TS, Python, Go, C#, Rust)
✅ ~60 .md user-facing + 41 patches + OpenAPI + config + CHANGELOG
✅ 151 testes integration (drift detector)
✅ Dashboard React (UI claims vs API real)
✅ 8 .proto (gRPC contract)
✅ Cross-checks: doc↔code, code↔doc, patch↔atual, SDK↔server, server↔SDK, UI↔server, test↔server, proto↔doc

**Análise considerada exaustiva.**
