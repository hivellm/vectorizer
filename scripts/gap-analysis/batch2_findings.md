# Batch 2 — Core Store (db/, models/)

Total: 461 fns | DOC: 45 (10%) | INTERNAL: 290 (63%) | **USER_FACING_GAP: 110 (24%)** | UNCERTAIN: 16

⚠️ **MAIOR descoberta da iter 2** — quase 1 em cada 4 funções públicas do
core store é user-facing mas sem doc. Concentra-se em features avançadas
(graph, sharding, GPU, payload index, multi-tenancy).

## USER_FACING gaps por área

### 1. Graph API completa (30 fns) — **ZERO doc**
- `db/graph.rs:57-468` — `add_node, remove_node, add_edge, get_neighbors, find_related, find_path, get_connected_components`
- `db/graph_relationship_discovery.rs:14-301` (8 fns) — auto-discovery similaridade, referência, contains, derived_from
- `db/collection/graph.rs:21-152` (4 fns) — get_graph, set_graph, populate_graph_if_empty, enable_graph
- → **Action: criar `docs/specs/GRAPH_RELATIONSHIPS.md`** + seção Graph em `API_REFERENCE.md`

### 2. Distributed Sharding (107 fns total) — **ZERO doc**
- `db/distributed_sharded_collection.rs:49-951` (47 fns) — insert/search/hybrid_search/delete/update distribuídos
- `db/sharded_collection.rs:42-414` (30 fns) — shard mgmt local, rebalancing, routing
- `db/sharding.rs:24-302` (28 fns) — ShardRouter, ShardRebalancer, calculate_moves_for_add/remove
- `db/vector_store/collection_type.rs:51-482` (32 fns) — CollectionType enum dispatch (heterogêneo: GPU/sharded/quantized)
- → **Action: criar `docs/specs/SHARDING.md`** com routing, rebalancing, distribuição

### 3. GPU backend (50 fns) — só mencionado no README
- `db/hive_gpu_collection.rs:47-719` — HiveGpuCollection: add_vectors, search_batch, update_vectors_batch, remove_vectors_batch
- → **Action: criar `docs/deployment/GPU_API.md`** ou expandir GPU_SETUP.md com API spec

### 4. Payload Index / metadata filtering (30 fns) — **ZERO doc**
- `db/payload_index.rs:43-712` — keyword/range/geo/text filtering
- `add_index_config, get_ids_for_keyword, get_ids_in_range, get_ids_in_geo_radius, search_text`
- → **Action: criar `docs/specs/PAYLOAD_FILTERING.md`** — feature crítica de produto, invisível hoje

### 5. Multi-tenancy (28 fns) — HiveHub-specific, sparse
- `db/multi_tenancy.rs:94-407` — TenantMetadata, TenantManager, quota enforcement, usage tracking
- → **Action: criar `docs/deployment/MULTI_TENANCY.md`** ou expandir HUB_INTEGRATION

### 6. WAL + auto-save (21 fns)
- `db/wal_integration.rs:29-179` (12) — log_insert/update/delete, recover_from_wal, checkpoint
- `db/auto_save.rs:56-322` (9) — start, force_save, mark_changed, cleanup_old_snapshots
- → **Action: criar `docs/deployment/DURABILITY.md`** ou seção em REPLICATION.md

### 7. Hybrid search + sparse vectors (16 fns)
- `db/hybrid_search.rs:94-110` (2) — HybridSearchConfig + hybrid_search
- `models/sparse_vector.rs:23-265` (14) — SparseVector: dot_product, cosine_similarity, norm, index ops
- → docs hoje só falam alto-nível; faltam exemplos da API

### 8. Quantized collection (6 fns)
- `db/quantized_collection.rs:71-267` — new, new_with_quantization, add_vectors, search, metadata
- → linkar de API_REFERENCE para PQ_IMPLEMENTATION + guia de seleção

### 9. Outros
- `db/collection/index.rs:19-302` — fast_load_vectors, dump/load_hnsw_index_from_dump (3 fns)
- `db/collection/quantization.rs:16-169` — requantize, train_pq_if_needed (3)
- `db/vector_store/collections.rs:26-875` — create_collection_with_owner/quantization, enable_graph_for_collection (25 fns)
- `models/mod.rs:59-821` — normalize_vector, distance_to_similarity (15)

## UNCERTAIN (16 fns que precisam revisão humana)
- `db/optimized_hnsw.rs:82-448` — provável INTERNAL (HNSW primitives)
- `db/async_indexing.rs:71-490` — IndexBuildManager (start_rebuild, swap_index)
- `db/vector_store/mod.rs:85-143` — VectorStore::new* factories (parte de SDK init?)
- `db/vector_store/metadata.rs:29-68` — stats, get/set_metadata
- `models/vector_utils_simd.rs:14-28` — SIMD distance metrics
- `db/raft.rs:114-516` — RaftStateMachine, RaftNode

## INTERNAL counts (não-gaps)
collection: 16 | persistence: 8 | sparse_vector: 8 | qdrant compat: 45 | optimized_hnsw: 22 | async_indexing: 17 | raft: 12 | sharded_distributed: 40 | multi_tenancy: 10 | payload_index: 30 | wal: 8 | demais: ~74. Total: 290.
