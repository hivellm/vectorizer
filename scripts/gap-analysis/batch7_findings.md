# Batch 7 — CLI/Config/Benchmark

Total: 211 fns | DOC: 58 (28%) | INTERNAL: 125 (59%) | **USER_FACING_GAP: 17 (8%)** | UNCERTAIN: 11

## USER_FACING gaps reais

### CLI commands sem doc (9)
- `cli/commands.rs:20` `handle_server_command` (start/stop/restart)
- `cli/commands.rs:81` `handle_user_command` (RBAC)
- `cli/commands.rs:190` `handle_api_key_command` (lifecycle)
- `cli/commands.rs:349` `handle_collection_command` (CRUD)
- `cli/commands.rs:510` `handle_config_command` (reload/export/import)
- `cli/commands.rs:599` `handle_status_command` (health)
- `cli/commands.rs:667` `handle_snapshot_command` (backup/restore)
- `cli/setup.rs:13` `run` (wizard init)
- `cli/setup.rs:162` `run_wizard` (full flow)
- → **Action: criar `docs/users/getting-started/CLI_REFERENCE.md`** com `--help` de cada subcommand + exemplos

### Config options sem doc (8)
- `config/enhanced_config.rs:301` `enable_auto_reload` — feature undocumented
- `config/enhanced_config.rs:396` `create_from_template` — sem template docs
- `config/enhanced_config.rs:498` `get_config_as_env_vars` — export sem doc
- `config/enhanced_config.rs:559` `export_config` — formatos suportados (JSON/YAML/TOML) sem doc
- `config/workspace.rs:208` `add_workspace` — falta exemplos programáticos
- `config/workspace.rs:246` `remove_workspace` — falta na WORKSPACE.md
- `config/layered.rs:107` `load_layered` — env var override + YAML merge **NÃO documentado** (crítico para ops!)
- `config/layered.rs:159` `merge_yaml` — semântica de merge sem doc
- → **Action: expandir `docs/users/configuration/CONFIGURATION.md`** com layered loading, auto-reload, export

## Benchmark claims audit — RESULTADO IMPORTANTE
Iter 1 suspeitou de marketing fluff. Verificação em batch 7:

- ✅ **"+8.9% MAP improvement"** — VERIFICADO. Backed by `benchmark/metrics.rs:70` (`from_latencies`) + `data_generator.rs`. PERFORMANCE.md tem tabela detalhada.
- ✅ **"sub-3ms search"** — VERIFICADO. Backed by `benchmark_runner.rs:35-78`. Atualmente 0.6-2.4ms. Claim conservadora.

**Reverter recomendação iter 1**: claims são reais, NÃO marketing.

## Reprodutibilidade dos benchmarks ⚠️
- BenchmarkConfig com 3 presets (quick/comprehensive/regression) existe
- MAS: nenhum CLI público para executá-los — só library
- → **Action: criar `docs/users/operations/BENCHMARKING.md`** com exemplos Rust OU publicar `vectorizer-bench` CLI

## gRPC note
`crates/vectorizer-protocol/src/grpc_gen/` (~214 fns auto-geradas) corretamente excluído.
`docs/users/api/GRPC.md` existe e é adequado.

## Cobertura por sub-módulo
| Sub-módulo | Total | DOC% |
|---|---|---|
| benchmark | 141 | 27% (38 doc, mas 103 INTERNAL OK) |
| cli | 37 | 3% (1 doc, 9 GAP, 24 INTERNAL) |
| config | 33 | 18% (6 doc, 8 GAP) |
