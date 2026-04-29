# Batch 5 — Hub/Migration/Workspace/Batch/Discovery

Total: 330 fns | DOC: 210 (64%) | INTERNAL: 96 (29%) | **USER_FACING_GAP: 18 (5%)** | UNCERTAIN: 6

## USER_FACING gaps reais

### Batch ops (14 gaps) — **maior buraco do batch 5**
- `batch/processor.rs:67/125/163/201/575` — batch_insert/update/delete/search/execute_operation: documentados em API_REFERENCE mas SEM guia dedicado
- `batch/config.rs:73-187` (8 fns) — limites de memória/tamanho sem doc
- `batch/error.rs:105-359` (10 fns) — `BatchError`, `is_retryable()`, `should_retry()`, `success_rate()` — sem error code registry
- `batch/parallel.rs:31-109` — chunk/task processing sem guidance de tuning
- `batch/progress.rs:26-168` (8 fns) — progress reporting sem exemplos
- → **Action: criar `docs/users/api/BATCH.md`** com error registry, semântica atomic vs partial failure, memory limits, progress callbacks

### Migration (3 gaps)
- `hub_migration.rs:271` `execute()` — sem error recovery guide
- `hub_migration.rs:404` `rollback()` — sem RTO/RPO documentados
- `qdrant/data_migration.rs:27` `export_collection()` — sem semântica de export (point-in-time? incremental?)

### Discovery (1 gap)
- `discovery/hybrid.rs:37` `search_with_text()` — variant não documentado separadamente

## UNCERTAIN — features completas SEM doc
- **IP Whitelist** (`hub/ip_whitelist.rs:240-413`, 8 fns) — feature completa de tenant-scoped IP allow/blocklists, **ZERO doc**
  - → Action: nova seção em `HUB_INTEGRATION.md` "IP Access Control"
- **Request Signing** (`hub/request_signing.rs:180`) — `SigningValidator`, sem doc
  - → Action: documentar ou clarificar que é interno

## Cobertura forte
| Módulo | Doc% |
|---|---|
| Hub | 100% (106/161 documentadas, 55 internas corretamente separadas) |
| Workspace | 100% (45/51) |
| Discovery | 100% (20/22) |
| Migration | 100% (24/27) — mas confirmação de iter 1: tenant handlers ainda disabled |
| **Batch** | **22%** ← buraco grande |

## Confirmações iter 1
- ✅ Tenant migration handlers permanecem disabled (`hub_handlers/mod.rs:16`)
- ✅ Discovery 9-step pipeline 100% documentado
- ✅ HUB_INTEGRATION.md cobre auth/quota/usage/backups (linhas 88-602)
