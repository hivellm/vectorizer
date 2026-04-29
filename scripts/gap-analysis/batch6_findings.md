# Batch 6 — Infra (file_watcher, security, intelligent_search, cache, summarization, monitoring)

Total: 354 fns | DOC: 289 (82%) | INTERNAL: 54 (15%) | **USER_FACING_GAP: 8 (2%)** | UNCERTAIN: 0

✅ Camada mais bem documentada (82% cobertura) — gaps são pontuais.

## USER_FACING gaps reais

### File watcher (3)
- `file_watcher/metrics.rs:180` `FileWatcherMetrics::new()`
- `file_watcher/metrics.rs:191` `get_metrics()`
- `file_watcher/metrics.rs:352` `get_summary()`
- `file_watcher/mod.rs:713` `FileWatcherManager::get_metrics()`
- → Action: criar/expandir `docs/features/FILE_WATCHER.md` com seção "Metrics endpoints" (já flagged em batch 1)

### Cache (2)
- `cache/query_cache.rs:135` `QueryCache::new()` — config user-selecionável
- `cache/memory_manager.rs:335` `init_global_cache_memory_manager()` — entry point public
- → Action: documentar em `docs/users/configuration/CACHE.md`

### Summarization (1)
- `summarization/manager.rs:73` `SummarizationManager::summarize_text()` — código existe e funciona, MAS REST handler não está wired (confirma achado iter 1: endpoints `POST /summarize/text` documentados mas não roteados)
- → Action: ou wire o handler, ou marcar SUMMARIZATION.md como "programmatic only"

### Security (2)
- `security/rate_limit.rs:406` `rate_limit_middleware()` — verificar se está montado
- → Já mencionado em STUBS_ANALYSIS.md mas confirmar wiring

## Cobertura por módulo (forte!)
| Módulo | DOC% | Status |
|---|---|---|
| intelligent_search | 96% | ✅ |
| security | 95% | ✅ |
| file_operations | 93% | ✅ |
| monitoring | 89% | ✅ |
| file_loader | 84% | ✅ |
| file_watcher | 84% | ⚠️ metrics gaps |
| cache | 55% | ⚠️ config knobs hidden |
| summarization | 17% | ⚠️ doc é farta, mas só 4 fns são pub |

## Confirmações de iter 1
- ✅ Summarization code EXISTE (não é stub) — extractive/keyword/sentence/abstractive(OpenAI) implementados
- ✅ REST handler `/summarize/text` não está wired — gap real
- ✅ Encryption (p256+aes_gcm) confirmado
- ✅ Rate limiting documentado em API_REFERENCE.md
