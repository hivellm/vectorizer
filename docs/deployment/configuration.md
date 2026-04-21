# Configuration — layered model

Vectorizer's runtime config is **layered**: a single canonical base
file plus an optional mode override. The merge happens at boot. A
deployment that doesn't pick a mode runs against the base alone.

```
base (config.yml or config/config.example.yml)
   ↓ deep YAML merge
mode override (config/modes/<mode>.yml)
   ↓ env var overrides (existing pattern, e.g. VECTORIZER_HOST)
   ↓ CLI flag overrides (existing pattern, e.g. --host)
   = effective VectorizerConfig
```

## Why this exists

The repo previously carried five near-identical config files
(`config.yml`, `config/config.example.yml`, `config/presets/production.yml`,
`config/presets/cluster.yml`, `config/presets/hub.yml`) — 2076 lines total, with real
drift between them (`logging.level` was `debug` / `debug` / `warn` /
`warn` / `info` across the five). The drift made every change risky:
any new operator flag had to be propagated to every file and could
silently miss one. The layered model collapses to **one base** + small
**delta-only** mode files.

## Files

| File | Role |
|---|---|
| `config.yml` (operator-supplied) or `config/config.example.yml` (template) | **Base layer** — every key with sane single-node-dev defaults + rich comments. |
| `config/modes/dev.yml` | Developer-loop deltas (verbose logging, file watcher on). |
| `config/modes/production.yml` | Production deltas (warn logging, larger threads / batch / cache, +compression, +snapshots, file watcher off). |
| `config/modes/cluster.yml` | (next slot) cluster deltas (cluster.enabled=true, mmap on, sharding on, PQ on). |
| `config/modes/hub.yml` | (next slot) HiveHub deltas (hub.enabled=true, gpu.enabled=false, server bind 0.0.0.0). |

## How the merge works

The loader is implemented in [`src/config/layered.rs`](../../src/config/layered.rs). Per-type semantics:

- **Scalar** values in the override replace the base scalar.
- **Map** values are merged key-by-key, recursively.
- **Array** values fully replace the base array. There is no
  element-level merge — the natural shape (e.g. replacing the whole
  `cluster.servers: [...]` list) is what operators expect.
- **Null** in the override clears the base value (rarely needed; the
  only way to "unset" a field that has a non-null base default).

Missing keys in the override stay at their base value. Unknown keys
in the override are tolerated by the merge layer; the strict serde
deserialization on the merged document is what catches typos.

## Selecting a mode

Two ways to pick a mode:

```bash
# Environment variable (recommended for systemd / Docker / k8s)
export VECTORIZER_MODE=production
./target/release/vectorizer

# Per-process via the existing CLI plumbing (passes through to
# `VECTORIZER_MODE` internally)
VECTORIZER_MODE=dev cargo run
```

If `VECTORIZER_MODE` is unset, the loader uses the base alone and
logs `no mode override requested; using base config alone`.

If `VECTORIZER_MODE` names a mode that doesn't exist as
`config/modes/<mode>.yml`, the bootstrap fails with a typed
`ConfigError::ModeNotFound { mode, path }` — the error message names
both the mode you asked for and the path the loader looked at, so
you can fix a typo or a missing fragment without re-deriving the
loader's path logic.

A typo in the override (e.g. `serer:` instead of `server:`) will not
fail the merge but WILL fail the strict deserialization with a clear
`Schema(...)` error naming the unknown field.

## Examples

### Single-node dev workstation

```bash
# config.yml = a copy of config/config.example.yml
# no mode set → base alone
./vectorizer
# logs: 📑 no mode override requested; using base config alone
```

### Production deployment

```bash
cp config/config.example.yml config.yml
export VECTORIZER_MODE=production
./vectorizer
# logs: 📑 VECTORIZER_MODE=production — config validated through the layered loader
```

The effective config is `config.yml` + `config/modes/production.yml`
merged in order. Production-only knobs (zstd compression, scheduled
snapshots, larger memory pool, warn logging) are added on top of
whatever the operator put in `config.yml`.

### Cluster deployment

```bash
export VECTORIZER_MODE=cluster
# `cluster.servers[]` MUST be set in your config.yml — the cluster
# mode override only flips `cluster.enabled: true` and the storage
# knobs (mmap, sharding, PQ); it does NOT supply server addresses.
```

### Verifying the merged config

```bash
# Quick check from a Rust shell:
cargo test --test all_tests --all-features config::layered_real_files
```

The integration tests in
[`tests/config/layered_real_files.rs`](../../tests/config/layered_real_files.rs)
load the real `config/config.example.yml` + every shipped mode override
and assert the merged result deserializes into the strict
`VectorizerConfig`. They run on every PR.

## Migration from the legacy 5-file layout

The legacy files (`config/presets/production.yml`, `config/presets/cluster.yml`,
`config/presets/hub.yml`, the multi-tenant `config.yml`) are still in the
repo for the v3.x.0 release window. They will be deleted in a future
release once a `scripts/migrate_config.sh` helper lands that
ingests one of those files and produces the new base + override
split. Until then:

- Operators using the layered loader (`VECTORIZER_MODE` set) read
  from `config.yml` (base) + `config/modes/<mode>.yml`. The legacy
  files are not consulted.
- Operators not using the layered loader continue to point
  `--config <path>` at one of the legacy files; the existing
  single-file load path still works.

The deletion of the legacy files is tracked as the next slot of
`phase5_consolidate-config-files`. The migration script + delete
together require a release-window cadence; the layered loader and
its mode overrides do not, so they ship first.
