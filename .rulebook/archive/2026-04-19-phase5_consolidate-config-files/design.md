# Design: phase5_consolidate-config-files

## Decision summary

Five top-level YAMLs collapse into **one canonical base** + **four
mode override fragments**:

```
config.example.yml          # canonical base — every key, sane defaults, rich comments
config/modes/dev.yml        # delta-only: developer-friendly defaults
config/modes/production.yml # delta-only: hardening + perf tuning
config/modes/cluster.yml    # delta-only: distributed sharding + replication
config/modes/hub.yml        # delta-only: HiveHub multi-tenant
```

Layered loader order (highest precedence last):

1. **Base** — `config.example.yml` (or whatever path is passed to the
   loader; in production the operator typically renames the example to
   `config.yml`).
2. **Mode override** — selected via env (`VECTORIZER_MODE=production`)
   or CLI flag (`--mode production`); applied as a deep YAML merge on
   top of base.
3. **Environment variables** — `VECTORIZER_<SECTION>_<KEY>` overrides
   (existing pattern; not added here).
4. **CLI flags** — explicit `--host`, `--port` overrides (existing
   pattern; not added here).

A missing mode override is not an error — the loader logs that no mode
was selected and uses the base alone.

## Per-section base defaults + mode deltas

The base values are picked to be **safe-by-default** for a single-node
local development setup. Mode overrides scale up.

### `rpc` (added in v3.x)
- **Base**: `enabled: true`, `host: "0.0.0.0"`, `port: 15503`. RPC is
  the recommended primary transport per
  `phase6_make-rpc-default-transport`.
- **Mode deltas**: none. All modes inherit RPC enabled. Operators on
  restricted networks set `rpc.enabled: false` directly in their
  config copy.

### `server`
- **Base**: `host: "127.0.0.1"` (loopback, dev-safe), `port: 15002`,
  `mcp_port: 15002`.
- **production**: `host: "0.0.0.0"`, `data_dir: "./data"`.
- **hub**: `host: "0.0.0.0"`.
- **cluster**: inherits base (loopback is fine since cluster nodes
  bind to specific addresses via `cluster.servers[]`).
- **dev**: inherits base.

### `logging`
- **Base**: `level: "info"`, `format: "json"`, `log_requests: true`,
  `log_responses: false`, `log_errors: true`,
  `correlation_id_enabled: true`.
- **production**: `level: "warn"` (the only override; production
  hardening).
- **dev**: `level: "debug"`.

### `gpu`
- **Base**: `enabled: true`, `batch_size: 1000`,
  `fallback_to_cpu: true`, `preferred_backend: "auto"`.
- **production**: `batch_size: 2000` (production tuning).
- **hub**: `enabled: false` (cloud worker constraint).

### `monitoring`
- **Base**: prometheus + system metrics + telemetry sections, all
  enabled by default with sensible thresholds.
- **Mode deltas**: none. The matrix shows all five existing files
  agree on monitoring values.

### `cluster`
- **Base**: `enabled: false`, `node_id: null`, `discovery: "static"`,
  `timeout_ms: 5000`, `retry_count: 3`. The `cluster.memory.*`
  subsection lives in base too with conservative caps.
- **cluster**: `enabled: true`, `node_id: "node-1"`, plus
  `cluster.servers: [...]` array. Memory subsection inherits from
  base.

### `collections.defaults`
- **Base**: `dimension: 512`, `metric: "cosine"`,
  `quantization.type: "sq" / bits: 8`, `embedding.model: "bm25"`,
  `index.type: "hnsw" / m: 16, ef_construction: 200, ef_search: 64`.
- **Mode deltas**: none. The matrix confirms all five existing files
  agree.

### `transmutation`
- **Base**: `enabled: true`, `max_file_size_mb: 50`,
  `conversion_timeout_secs: 300`, `preserve_images: false`.
- **production**: `max_file_size_mb: 100`, `conversion_timeout_secs:
  600` (production large-file support).

### `normalization`
- **Base**: `enabled: true`, `level: "conservative"`, line-endings +
  content-detection blocks, `cache.max_entries: 10000`,
  `cache.ttl_seconds: 3600`.
- **production**: `cache.max_entries: 50000`, `cache.ttl_seconds:
  7200` (production caching).

### `performance`
- **Base**: `cpu.max_threads: 8`, `cpu.enable_simd: true`,
  `cpu.memory_pool_size_mb: 1024`, `simd.enabled: true`,
  `batch.default_size: 100`, `batch.max_size: 1000`,
  `batch.parallel_processing: true`, `query_cache.enabled: true`,
  `query_cache.max_size: 1000`, `query_cache.ttl_seconds: 300`.
- **production**: `cpu.max_threads: 16`, `cpu.memory_pool_size_mb:
  4096`, `batch.default_size: 500`, `batch.max_size: 2000`.
- **cluster**: `cpu.memory_pool_size_mb: 512` (per-node tighter cap).

### `workspace`
- **Base**: `enabled: true`, `default_workspace_file:
  "./workspace.yml"`.
- **production**: `auto_load_collections: true`.
- **cluster**: `enabled: false` (cluster mode loads collections from
  shards, not a workspace manifest).
- **hub**: `enabled: false` (multi-tenant; collections come from hub
  registration).

### `api`
- **Base**: `rest.enabled: true`, `rest.cors_enabled: true`,
  `rest.max_request_size_mb: 10`, `rest.timeout_seconds: 30`,
  `mcp.enabled: true`, `mcp.port: 15002`, `mcp.max_connections: 100`,
  `grpc.enabled: true`, `grpc.port: 15003`,
  `grpc.max_concurrent_streams: 100`, `grpc.max_message_size_mb: 10`.
- **production**: `rest.max_request_size_mb: 50`,
  `rest.timeout_seconds: 60`, `mcp.max_connections: 500`. Production
  also opts out of gRPC by setting `grpc.enabled: false` (REST + MCP
  are sufficient for production).

### `auth`
- **Base**: `enabled: false` (dev-friendly), `jwt_secret: ""`,
  `jwt_expiration: 3600`, `api_key_length: 32`,
  `rate_limit_per_minute: 100`, `rate_limit_per_hour: 1000`. Note:
  `AuthManager::new` already refuses to boot if `enabled: true` and
  `jwt_secret` is empty / shorter than 32 chars / equal to the
  historical insecure default — see CHANGELOG entry from
  phase4_jwt-secret-validation.
- **hub**: `enabled: true`. The hub mode requires JWT/API key auth
  for tenant isolation; operators must set `jwt_secret` separately.
- **production**: inherits base `auth.enabled: false` only when
  authentication is handled by an upstream gateway. When the server
  itself terminates auth, operators flip to `enabled: true` in their
  production config copy and provide a strong secret.

### `security`
- **Base**: `rate_limiting.enabled: true / requests_per_second: 100 /
  burst_size: 200`, `tls.enabled: false` (dev), `audit.enabled: true /
  max_entries: 10000`, `rbac.enabled: false / default_role: "Viewer"`.
- **Mode deltas**: none in the existing files. Operators flip TLS on
  per-deployment based on their cert provisioning.

### `storage`
- **Base**: `mmap.enabled: false`, `mmap.default_for_new_collections:
  false`, `wal.enabled: true / checkpoint_interval: 1000 /
  checkpoint_interval_secs: 300 / max_wal_size_mb: 100 / wal_dir:
  "./data/wal"`, `quantization.pq.enabled: false`,
  `sharding.enabled: false`.
- **production**: adds `storage.compression: { enabled: true, format:
  "zstd", level: 6 }`, `storage.compaction: { auto_compact: true,
  batch_size: 2000 }`, `storage.snapshots: { enabled: true,
  interval_hours: 2, max_snapshots: 24, retention_days: 7, path:
  "./data/snapshots" }`. These are net-new keys that base does NOT
  carry by default.
- **cluster**: `mmap.enabled: true`,
  `mmap.default_for_new_collections: true`,
  `quantization.pq.enabled: true`, `sharding.enabled: true`.
- **hub**: `mmap.enabled: true`,
  `mmap.default_for_new_collections: true`.

### `replication`
- **Base**: `enabled: false`, `role: "standalone"`,
  `heartbeat_interval_secs: 5`, `replica_timeout_secs: 30`,
  `log_size: 1000000`, `reconnect_interval_secs: 5`.
- **Mode deltas**: none. The matrix confirms replication is OFF in
  every existing file. Operators flip on by editing their config
  directly when they bring up a master/replica pair.

### `hub`
- **Base**: `enabled: false`, `api_url:
  "https://api.hivehub.cloud"`, `service_api_key: ""`,
  `timeout_seconds: 30`, `retries: 3`, `usage_report_interval: 300`,
  `tenant_isolation: "collection"`, plus `cache` + `connection_pool`
  subsections.
- **hub**: `enabled: true`. Operators provide their `service_api_key`
  separately. The cluster mode also flips `hub.enabled: true` because
  the existing `config.cluster.yml` ran the cluster against a local
  hub instance for testing.

### `file_watcher`
- **Base**: `enabled: true`, `debounce_delay_ms: 1000`,
  `min_file_size_bytes: 1`, `max_file_size_bytes: 10485760`,
  `hash_validation_enabled: true`, `collection_name:
  "workspace-files"`. Watcher is on by default for the
  developer-friendly local setup.
- **production**: `enabled: false`. Production environments serve
  pre-indexed data; the file watcher's per-FS-event indexing path
  isn't a fit.
- **cluster**: `enabled: false` for the same reason; cluster nodes
  also force-disable the watcher when `cluster.memory.disable_file_watcher:
  true` regardless of this setting.
- **hub**: `enabled: false` for the same reason.

## Loader contract

```rust
// src/config/layered.rs
pub fn load_layered(
    base_path: &Path,
    mode: Option<&str>,
    modes_dir: Option<&Path>,
) -> Result<VectorizerConfig, ConfigError>;
```

- `base_path` is the canonical YAML (typically `config.yml` or
  `config.example.yml`).
- `mode` is read from `VECTORIZER_MODE` env var or the
  `--mode <name>` CLI flag. `None` → no override applied.
- `modes_dir` defaults to `./config/modes/` relative to `base_path`'s
  directory; tests can override it.
- Mode override application is a **deep YAML merge**: scalar values
  in the override replace base; map values are recursively merged;
  array values fully replace base (no element-level merge — replacing
  the whole `cluster.servers` list is the natural shape).
- Unknown keys in override files are warnings, not errors. The
  serde-side `VectorizerConfig` deserialization performs the strict
  validation; the merge layer is purely textual.
- Loader errors are typed (`ConfigError::BaseNotFound`,
  `ConfigError::ModeNotFound`, `ConfigError::ParseError`,
  `ConfigError::MergeError`) so callers can distinguish "you forgot to
  copy `config.yml`" from "your override has a typo".

The existing single-file loader path (`std::fs::read_to_string` +
`serde_yaml::from_str`) stays as the no-mode fallback — adding the
layered loader is additive, not a rewrite.

## What ships in this task vs follow-ups

This task ships:

- This `design.md` (the matrix + canonical-default decisions).
- The layered loader at `src/config/layered.rs`.
- An audited `config.example.yml` matching the canonical-defaults
  table above (largely already true; spot fixes only).
- `config/modes/{dev,production}.yml` as the **proof entries** —
  enough to verify the merge produces the right shape end-to-end.
- `docs/deployment/configuration.md` describing the layered model
  and migration path.
- Tests for the merge + each shipped mode override.

Follow-up slots (each named with rationale in tasks.md so a future
contributor can pick them up):

- `config/modes/cluster.yml` and `hub.yml` deltas. The matrix above
  has the values; the work is mechanical YAML composition + a test
  per mode. Carved out so this task ships the loader pattern in a
  reviewable size.
- Deletion of the legacy `config.production.yml`, `config.cluster.yml`,
  `config.hub.yml`, `config.yml`. Not done in this task because
  operators have copies of these checked into their own deployment
  repos; deleting them without a migration window would break
  upgrades. The migration script (next bullet) lands first.
- `scripts/migrate_config.sh` taking an old mode file and producing
  the new base + override split.
- Updates to `docker-compose.*.yml`, `helm/`, `k8s/` references that
  pin the old filenames. These all currently mount or reference
  `config.yml` only; the layered loader's default `base_path` is
  `./config.yml`, so those manifests need no change unless they
  switch to the override pattern explicitly.
