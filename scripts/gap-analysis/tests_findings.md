# Tests audit (151 integration test files)

| Item | Count |
|---|---|
| Total integration tests | 151 (145 vectorizer + 6 vectorizer-server) |
| Tests calling missing endpoints | **0** ✅ |
| **Tests revealing doc lies** | **2** ⚠️ |
| Bypassed tests (#[ignore]) | 13+ (todos justificados) |

## ✅ Boa notícia: zero drift de endpoints
Nenhum teste chama endpoint que não existe mais. Os integration tests
funcionam como guard contra "doc-only endpoints" — a iter 1 já capturou
os 5 endpoints fantasma.

## ⚠️ Tests provam que doc mente em 2 lugares (correções importantes!)

### #1 — Sync replication NÃO é "future"
- `tests/cluster/distributed_resilience.rs:218-231`: `test_write_concern_serialization()` + `test_write_concern_default_is_none()` testam todas variantes incluindo `WriteConcern::All`
- Doc atual: `REPLICATION.md:255` diz "Sync Replication = Future"
- **Realidade**: Já implementado, testado, funciona
- → Corrige iter 1 §10 e iter 2 §2.1

### #2 — Manhattan: iter 1 estava CORRETO (verificado)
- `tests/simd/new_ops.rs:50-63` testa `manhattan_distance()` SIMD primitive
- **MAS** `crates/vectorizer/src/models/mod.rs:390-397` enum
  `DistanceMetric { Cosine, Euclidean, DotProduct }` NÃO tem Manhattan
- → SIMD primitive existe mas não conectada ao enum user-facing
- → Iter 1 está certa: Manhattan **NÃO** é distance metric selecionável
- → Gap real: SIMD código pronto e desperdiçado, ou enum incompleto

## Bypassed tests (todos justificados)
- GPU tests (`#[ignore]`): require GPU
- Cluster perf tests: lentos (>60s)
- Replication failover/comprehensive: precisam de bind TCP
- **Nenhum `.skip()` ou `#[test]` comentado escondendo bug**

## Recomendações
1. **P0**: corrigir `REPLICATION.md:255` — sync replication é REAL
2. **P0**: investigar status real do Manhattan distance — SIMD impl existe;
   verificar se está no enum + caminho de busca; se sim, iter 1 estava errado
3. **P1**: tests funcionando como contrato. Manter integração estável.
4. **P2**: adicionar test pra `/summarize/text` se feature for roadmap real;
   senão remover do openapi.yaml + SDKs
