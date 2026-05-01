# Proposal: phase9_bulk-upsert-backpressure

Source: https://github.com/hivellm/vectorizer/issues/263

## Why

Under sustained bulk-upsert load (BM25 vocabulary build over many concurrent
batches into many collections), the server consumes unbounded CPU + memory
until Docker health-checks fail and the container is killed/restarted. Observed
in `hivehub/vectorizer:3.0.0` (server crate v3.0.2) on Windows / Docker Desktop / WSL2.

Concrete repro: ~21 000 small text payloads fanned out by 6 parallel embedder
workers (Cortex `cortex-embedder-worker`, batch=64) into ~75 BM25 collections
(one per `repo×kind` pair). Effects:

- `RestartCount` climbs (12 in our run), `OOMKilled=false` (Docker health
  probe failed, not kernel OOM); `Memory limit=0` so the host saturates.
- Server log floods with `WARN BM25 vocabulary is empty for text '...', using
  hash-based fallback` — thousands per second, one per upsert.
- `GET /collections` and `POST /auth/login` start timing out (>30 s) under
  the burst.
- After ~3–5 min the daemon health-checks the container down → restart →
  vocabulary rebuild starts over → next batch arrives → loop.

Downstream impact: Cortex's `cortex-embedder-worker` happily fires
6 × 64 = 384 in-flight upsert RPCs against a single Vectorizer container.
Without server-side backpressure the only mitigation is to throttle every
consumer (`CORTEX_EMBEDDER_WORKERS=1`), which leaves Vectorizer's actual
capacity unused and slows the entire pipeline.

## What Changes

### 1. Bounded concurrency for vocabulary builds

Introduce a `tokio::sync::Semaphore` guarding the BM25 vocabulary-build path,
sized by a new config knob (default = `num_cpus::get()`). Excess upserts
queue on the semaphore instead of all running in parallel.

### 2. Queue-depth metrics

Expose two gauges on `/metrics`:

- `vectorizer_upsert_queue_depth{collection="..."}` — current waiters.
- `vectorizer_upsert_in_flight{collection="..."}` — current permit holders.

Plus a counter `vectorizer_upsert_rejected_total{reason="queue_full"}`.

### 3. HTTP 429 with `Retry-After`

Once queue depth crosses a configurable high-water mark, return
`429 Too Many Requests` with a `Retry-After` header. Existing SDK clients
(Rust `vectorizer-sdk`, Python SDK) already retry with backoff.

### 4. Read-path isolation

Move the read path (`GET /collections`, `/auth/*`, `/health`,
`/metrics`) to a separate Tokio runtime (or at minimum a dedicated
priority queue) so write-side saturation doesn't starve health probes
and discovery endpoints.

### 5. Log rate-limiting

Replace the per-upsert `WARN BM25 vocabulary is empty …` with a rate-limited
emitter (e.g. one warn per collection per 5 s, plus a counter).
The current spam is itself a CPU/disk-IO sink during bursts.

## Impact

- Affected code:
  - `src/api/` (extractor + 429 layer, Axum router split)
  - `src/db/` (semaphore around vocabulary build, queue-depth instrumentation)
  - `src/embedding/` (BM25 build path)
  - `src/server/` (runtime split, MCP path mirrors REST)
  - `config.example.yml`, `src/config.rs` (new knobs)
- Affected specs: new `specs/backpressure/spec.md` under this task.
- Breaking change: **NO** — defaults preserve current behavior unless
  `backpressure.enabled = true` (or env override).
- User benefit:
  - Container survives bursty bulk upserts; no more restart loop.
  - `/health` and `/collections` stay responsive under load.
  - SDK clients receive standards-compliant 429 + `Retry-After` instead of
    socket timeouts.
  - Operators get queue visibility on `/metrics`.

## Configuration (new keys)

```yaml
backpressure:
  enabled: true                          # default true once shipped
  max_concurrent_vocab_builds: 0         # 0 = num_cpus
  upsert_queue_high_water: 256           # per-collection
  upsert_queue_hard_limit: 1024          # 429 above this
  retry_after_seconds: 2
  read_path_isolated_runtime: true
  log_rate_limit_per_5s: 1
```

Env overrides: `CORTEX_VECTORIZER_MAX_CONCURRENT_BUILDS`,
`CORTEX_VECTORIZER_UPSERT_HIGH_WATER`, `CORTEX_VECTORIZER_UPSERT_HARD_LIMIT`.
