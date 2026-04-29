# gRPC — .proto vs GRPC.md

| Item | Count |
|---|---|
| Total RPCs em proto | 30 (Vectorizer 14 + Cluster 16) |
| Documented em GRPC.md | 26 |
| **Proto-only (undocumented)** | **4** |
| Doc-only (fantasma) | 0 |
| Services não wired | 0 |
| **Schema mismatches** | **4** ⚠️ |

## Proto-only (4 RPCs Raft+shard internas)
`cluster.proto:32-37`:
- `GetShardVectors` — shard data migration
- `RaftVote` — consensus voting
- `RaftAppendEntries` — log replication
- `RaftSnapshot` — Raft snapshotting

→ Action: documentar em GRPC.md como "Cluster Internal RPCs" — mesmo
sendo internas, operadores depurando cluster precisam saber que existem.

## 🚨 Schema mismatches em SearchResult/HybridSearchResult
**Breaking change em v3.0.0 não documentada**:
- proto v3.0.0 mudou `score`/`hybrid_score`/`dense_score`/`sparse_score`
  de `double` (f64) para `float` (f32)
- comentário no `vectorizer.proto:184-188` confirma: "downgraded from
  double→float to match canonical crate::models::SearchResult"
- **GRPC.md ainda lista `double`** em linhas 179, 238-240

→ Clientes regenerando stubs receberão `float32`; doc promete `double`.
**Action P0**: corrigir GRPC.md + adicionar nota de migração.

## Services wiring (✅ todos OK)
`crates/vectorizer-server/src/server/core/grpc.rs`:
- VectorizerService (line 28, 33)
- ClusterService (lines 36-44, condicional em cluster_manager)
- Qdrant CollectionsServer/PointsServer/SnapshotsServer (lines 50-65)

## Atenção: Qdrant PointsService
GRPC.md lista subset de PointsService (Upsert/Delete/Get/etc.). Mas
`points_service.proto:98-135` mostra mais ~8 RPCs (Discover, DiscoverBatch,
UpdateBatch, QueryBatch, QueryGroups, Facet, SearchMatrixPairs,
SearchMatrixOffsets) — verificar se estão implementados ou se são stubs.

## Recomendações
1. **P0**: corrigir tipos `double`→`float` em GRPC.md (breaking v3.0.0)
2. **P1**: documentar 4 Raft RPCs como "Cluster Internal"
3. **P1**: validar Qdrant PointsService completude (~8 RPCs adicionais)
4. **P2**: adicionar seção "Migration v3.0.0" sobre regeneração de stubs
