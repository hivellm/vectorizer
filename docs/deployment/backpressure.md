# Bulk-upsert backpressure runbook

Tracks: [hivellm/vectorizer#263](https://github.com/hivellm/vectorizer/issues/263).

## Problem this solves

Under sustained bulk-upsert load — typically a fan-out producer
(Cortex `cortex-embedder-worker`, Synap stream consumers, ...) firing
hundreds of in-flight upsert RPCs against many fresh BM25 collections
in parallel — the server used to consume unbounded CPU + memory until
Docker health checks failed and the container was killed/restarted.
The restart loop persisted because each fresh restart had to rebuild
the same vocabularies, while the next batch of upserts arrived before
those builds finished.

`v3.2.0+` introduces three layers of bounded-resource enforcement,
plus log-rate-limiting and SDK alignment, so the original failure
mode cannot reproduce with the default config.

## What's enforced

### 1. Bounded BM25 vocabulary-build concurrency

A single shared `tokio::sync::Semaphore` gates the CPU-heavy section
of every BM25 vocabulary build. Default capacity = `num_cpus::get()`,
configurable via `backpressure.max_concurrent_vocab_builds`.

Excess builds queue on the semaphore instead of running in parallel
and saturating every core.

### 2. Per-collection upsert admission

Each upsert (REST `/insert*`, gRPC `InsertVector*` / Qdrant `Upsert`,
MCP `insert_text`) acquires a per-collection in-flight ticket from
`UpsertQueue` before touching the embedding pipeline. Two thresholds:

| Threshold | Default | Behavior |
| --- | --- | --- |
| `upsert_queue_high_water` | 256 | Accept the upsert; emit a structured warn; bump `vectorizer_upsert_rejected_total{reason="queue_high_water_warn"}`. |
| `upsert_queue_hard_limit` | 1024 | **Refuse** — REST `429 Too Many Requests` with `Retry-After`; gRPC `RESOURCE_EXHAUSTED` with `retry-after` metadata; MCP structured error `{ code: "queue_full", retryAfterSeconds: N }`. |

The first-party SDKs (Rust `vectorizer-sdk`, Python `sdks/python/`)
honor `Retry-After` automatically.

### 3. Read-path responsiveness

Phases 1+2 above bound the actual contended resource (CPU). Reads
(`/health`, `GET /collections`, `/auth/*`, `/prometheus/metrics`)
share the same Tokio runtime as writes but always have CPU headroom
because writers can't monopolize cores.

A literal split-runtime mode is reserved behind
`backpressure.read_path_isolated_runtime` for future work; current
benchmarks show the CPU cap alone is sufficient.

### 4. Log rate-limiting

The `WARN BM25 vocabulary is empty for text '...' using hash-based
fallback` line is rate-limited to **1 emission per collection per
5 s window** by default. The full count is preserved in the
`vectorizer_bm25_empty_vocab_fallback_total{collection}` counter so
operators don't lose the volume signal.

## Configuration

```yaml
backpressure:
  enabled: true
  max_concurrent_vocab_builds: 0   # 0 = auto = num_cpus::get()
  upsert_queue_high_water: 256     # per-collection; warn above
  upsert_queue_hard_limit: 1024    # per-collection; 429 above
  retry_after_seconds: 2           # value used in 429 Retry-After
  read_path_isolated_runtime: true # reserved for future split-runtime
  log_rate_limit_per_5s: 1         # max "BM25 vocab empty" warns per coll/5s
```

### Env var overrides

Highest precedence; useful for ops-time tuning without a config-file
redeploy.

| Variable | Effect |
| --- | --- |
| `CORTEX_VECTORIZER_BACKPRESSURE_ENABLED` | `true` / `false` to toggle enforcement entirely. |
| `CORTEX_VECTORIZER_MAX_CONCURRENT_BUILDS` | Override `max_concurrent_vocab_builds`. |
| `CORTEX_VECTORIZER_UPSERT_HIGH_WATER` | Override `upsert_queue_high_water`. |
| `CORTEX_VECTORIZER_UPSERT_HARD_LIMIT` | Override `upsert_queue_hard_limit`. |

### Tuning notes

- **Per-collection** caps are intentional: a single noisy collection
  can't starve other collections under the same server.
- The hard limit is not a quota — it's an admission gate. SDKs retry
  past it. Set it high enough that healthy producers aren't bouncing
  off it.
- High-water alerts are leading indicators; tune `upsert_queue_high_water`
  so the warn fires _before_ producers start saturating.

## Metrics

All metrics are registered automatically and surface on
`GET /prometheus/metrics`.

| Metric | Type | Labels | What it tells you |
| --- | --- | --- | --- |
| `vectorizer_upsert_queue_depth` | gauge | `collection` | Current per-collection in-flight upsert count. Reflects live atomic counter at scrape time. |
| `vectorizer_upsert_in_flight` | gauge | `collection` | Mirror of `_queue_depth`; kept under a distinct name for dashboards that distinguish "currently doing work" vs "currently waiting". |
| `vectorizer_vocab_build_permits_available` | gauge | _(none)_ | Permits free on the global vocab-build semaphore. `available == 0` means writers are queued. |
| `vectorizer_upsert_rejected_total` | counter | `reason` | `queue_full` = HTTP 429 / gRPC `RESOURCE_EXHAUSTED`. `queue_high_water_warn` = depth crossed the high-water mark but admit succeeded. |
| `vectorizer_bm25_empty_vocab_fallback_total` | counter | `collection` | Bumped on every empty-vocabulary fallback regardless of warn rate-limit. |

### Interpreting the dashboard

- **Sudden spike on `upsert_queue_depth`** + steady growth → producer
  is faster than the writer can drain. Either raise the hard limit
  (if CPU has headroom) or throttle the producer.
- **`vocab_build_permits_available` floored at 0** for sustained
  windows → vocab-build is the bottleneck. Raise
  `max_concurrent_vocab_builds` if `num_cpus` is already maxed and
  you've moved to a bigger host.
- **`upsert_rejected_total{reason="queue_full"}` rising** → either
  raise `upsert_queue_hard_limit` (if SLO allows tolerating more
  in-flight work per collection) or fix the producer's concurrency.
- **`bm25_empty_vocab_fallback_total` >> 0 for a steady-state
  collection** → vocabulary never built; check that the file_loader
  pipeline finished step 3 (`build_vocabulary_gated`).

### Alert suggestions (Prometheus)

```yaml
- alert: VectorizerUpsertQueueFullSustained
  expr: rate(vectorizer_upsert_rejected_total{reason="queue_full"}[5m]) > 0
  for: 10m
  labels:
    severity: warning
  annotations:
    summary: "Vectorizer is rejecting upserts (queue full) on {{ $labels.instance }}"
    description: "Producer is exceeding `upsert_queue_hard_limit` for >10m. Check producer concurrency or raise the cap."

- alert: VectorizerVocabBuildSaturated
  expr: vectorizer_vocab_build_permits_available == 0
  for: 5m
  labels:
    severity: warning
  annotations:
    summary: "Vectorizer vocab-build semaphore has been wedged for >5m on {{ $labels.instance }}"
    description: "All vocab-build permits held continuously. Either the host is undersized or a build is stuck."
```

## Validating the fix locally

The original repro from issue #263:

> ~21 000 small text payloads published over Synap stream, fanned out
> by 6 parallel embedder workers (Cortex `cortex-embedder-worker`,
> batch=64) into ~75 BM25 collections.

To validate locally without standing up Cortex:

```bash
# 1. Run server with default backpressure config
docker run --rm -it \
  -p 15002:15002 \
  -e CORTEX_VECTORIZER_MAX_CONCURRENT_BUILDS=4 \
  -e CORTEX_VECTORIZER_UPSERT_HARD_LIMIT=512 \
  hivehub/vectorizer:3.2.0

# 2. From another shell, flood with concurrent inserts
# (vectorizer-cli's bench subcommand or a custom load generator).
# Confirm:
#   - 429s appear once depth crosses 512 per collection
#   - Retry-After is set to 2 (default)
#   - /health stays < 500ms p99 even under sustained load
#   - Container does NOT restart

curl -sf http://localhost:15002/health
curl -s http://localhost:15002/prometheus/metrics | grep -E '^vectorizer_(upsert|vocab)'
```

## Grafana panels

A starter panel JSON for the four backpressure gauges + the rejected
counter is shipped at [docs/grafana/backpressure-panels.json](../grafana/backpressure-panels.json).
Import it into the existing `vectorizer-dashboard.json` (also under
`docs/grafana/`) or use it as a standalone row.
