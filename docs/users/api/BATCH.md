---
title: Batch Operations API
module: api
id: batch-api
order: 8
description: User-facing guide for bulk insert, update, delete, and search operations
tags: [api, batch, bulk, ingestion, performance]
---

# Batch Operations — User API Guide

This page documents the batch endpoints exposed over REST, the request/response
shapes, the semantics for partial failures, the full error-code registry, size
limits, progress/parallelism knobs, and operational guidance. If you hit batch
APIs in production this is the single page you should read.

A short, example-only entry already exists in
[`API_REFERENCE.md`](./API_REFERENCE.md#batch-operations) (around lines
558–660). This guide is the authoritative reference. Where the two disagree,
prefer this document.

---

## Overview

Batch endpoints let a client ship many vector operations in a single HTTP
request. They optimize **throughput over latency**: one batch of 1 000
inserts is dramatically cheaper than 1 000 individual inserts (one TCP
session, one auth check, one cache invalidation), and the server amortizes
embedding-manager locking and HNSW index acquisition across the whole batch.

**Use a batch when** you're ingesting in bulk, re-embedding a collection,
bulk-deleting by id list, or running several searches against the same
collection. **Stick with single-op endpoints when** the write is
user-interactive and latency matters more than throughput, or when you need
strict ordering across heterogeneous operations (batches are type-uniform).

### Two layers, different semantics

- **REST endpoints** (this document). Process arrays item-by-item with
  per-entry error capture. **Partial success is always returned**.
- **In-process Rust API** (`vectorizer::batch::BatchProcessor`). Used by
  embedded / library consumers. Exposes the `atomic` flag, enforces
  `max_batch_size`, memory limits, retry/timeout/concurrency knobs. REST
  endpoints do **not** currently use `BatchProcessor` — they re-implement a
  per-item loop in `crates/vectorizer-server/src/server/rest_handlers/`.

---

## Supported REST endpoints

All four endpoints accept and return `application/json`. Authentication follows
the server-wide rules (JWT / API Key, see [AUTHENTICATION](./AUTHENTICATION.md)).

| Method | Path              | Handler                                        | Purpose                                |
|--------|-------------------|------------------------------------------------|----------------------------------------|
| POST   | `/batch_insert`   | `batch_insert_texts` (vectors.rs:423)          | Bulk-insert texts (embedded server-side) |
| POST   | `/insert_texts`   | `insert_texts` (vectors.rs:433) — alias         | Identical payload to `/batch_insert`   |
| POST   | `/insert_vectors` | `insert_vectors` (insert.rs:670) — since 3.1.0 | Bulk-insert pre-computed embeddings (skip embedder) |
| POST   | `/batch_search`   | `batch_search_vectors` (search.rs:539)         | Many searches against one collection   |
| POST   | `/batch_update`   | `batch_update_vectors` (search.rs:677)         | Bulk-update vector `data` and/or `payload` |
| POST   | `/batch_delete`   | `batch_delete_vectors` (search.rs:868)         | Bulk-delete a list of ids              |

> The older form `POST /collections/{name}/batch_insert` shown in
> `API_REFERENCE.md` is deprecated. Prefer the flat top-level routes listed
> above — they are what `routing.rs` (lines 300–305) registers today.

Qdrant-compatible batch endpoints (`/qdrant/collections/{name}/points/search/batch`,
`.../recommend/batch`, `.../query/batch`) are out of scope for this guide — see
the [Qdrant compatibility docs](../qdrant/) instead.

---

## Request shape

### `POST /batch_insert` (and `/insert_texts`)

```json
{
  "collection": "docs",
  "texts": [
    {
      "id": "doc-001",
      "text": "First document to index.",
      "metadata": { "source": "wiki", "doc_id": 1 },
      "auto_chunk": true,
      "chunk_size": 1024,
      "chunk_overlap": 128
    },
    { "id": "doc-002", "text": "Second document." }
  ],
  "public_key": "...",
  "auto_chunk": true,
  "chunk_size": 1024,
  "chunk_overlap": 128
}
```

| Field                   | Type   | Required | Notes                                                                    |
|-------------------------|--------|----------|--------------------------------------------------------------------------|
| `collection`            | string | yes      | Target collection. Missing or empty → HTTP 400.                          |
| `texts`                 | array  | yes      | Non-empty; empty array → HTTP 400.                                       |
| `texts[].text`          | string | yes      | Missing/non-string → that entry errors; the batch continues.             |
| `texts[].id`            | string | no       | **Since 3.1.0:** used as the resulting `Vector.id` (non-chunked) or as the prefix for `<id>#<chunk_index>` chunk ids; also echoed back as `client_id` in the result. Pre-3.1.0 was echoed-only. Constraints: non-empty, length ≤ 256, no leading/trailing whitespace, must not contain `#`. Re-inserting with the same `id` upserts in place. |
| `texts[].metadata`      | object | no       | Stored as the vector's `Payload`. **Since 3.1.0** chunk payloads are flat: user fields (`metadata`) sit at the payload root alongside server fields (`content`, `file_path`, `chunk_index`, `parent_id`). Server keys win on collision. Pre-3.1.0 chunks nested everything under `metadata` — see [CHANGELOG migration note](../../../CHANGELOG.md#migrating-from-30x-chunked-payloads). |
| `texts[].public_key`    | string | no       | Per-entry override of the batch-level `public_key` (tenant scoping).     |
| `texts[].auto_chunk`    | bool   | no       | Defaults to batch-level `auto_chunk` (default `true`).                   |
| `texts[].chunk_size`    | number | no       | Per-entry override of the batch-level default.                           |
| `texts[].chunk_overlap` | number | no       | Per-entry override of the batch-level default.                           |

### `POST /insert_vectors` (since 3.1.0)

Use when the client already holds the embeddings (its own embedder, an
external pipeline, a cached re-encode) and wants to skip the server-side
embedding pass. Vector ids are honored verbatim, so this is also the
endpoint to reach for when idempotent upsert by client id matters more
than auto-chunking.

```json
{
  "collection": "docs",
  "vectors": [
    {
      "id": "doc-001",
      "embedding": [0.12, -0.34, 0.56, ...],
      "payload": { "casa": "camara", "parlamentar": "Jack Rocha", "ano": 2023 }
    },
    {
      "id": "doc-002",
      "embedding": [0.91, 0.02, ...],
      "metadata": { "source": "wiki", "lang": "pt-br" }
    }
  ],
  "public_key": "..."
}
```

| Field                    | Type           | Required | Notes                                                                                                                                                          |
|--------------------------|----------------|----------|----------------------------------------------------------------------------------------------------------------------------------------------------------------|
| `collection`             | string         | yes      | Target collection. Auto-created with defaults if missing.                                                                                                      |
| `vectors`                | array          | yes      | Non-empty.                                                                                                                                                     |
| `vectors[].id`           | string         | no       | Same client-id contract as `/insert_texts`. Falls back to a server-generated UUID v4 when omitted.                                                            |
| `vectors[].embedding`    | array<number>  | yes      | Length **must** equal `collection.dimension`. Mismatches, non-arrays, and non-numeric values fail the entry only — the batch continues.                       |
| `vectors[].payload`      | object         | no       | Free-form JSON; stored verbatim as the vector's `Payload`.                                                                                                     |
| `vectors[].metadata`     | object         | no       | Fallback when `payload` is absent — string→string map, mirroring `/insert_texts`. Ignored when `payload` is provided.                                          |
| `vectors[].public_key`   | string         | no       | Per-entry encryption override; falls back to the batch-level `public_key`.                                                                                     |
| `public_key`             | string         | no       | Batch-level encryption key applied to every entry that doesn't override it.                                                                                    |

Response shape mirrors `/insert_texts` (see below).

### `POST /batch_search`

```json
{
  "collection": "docs",
  "queries": [
    { "query": "vector database", "limit": 5, "threshold": 0.2 },
    { "vector": [0.12, 0.45, ...], "limit": 10 }
  ]
}
```

| Field              | Type           | Required | Notes                                                                          |
|--------------------|----------------|----------|--------------------------------------------------------------------------------|
| `collection`       | string         | yes      | Target collection.                                                              |
| `queries`          | array          | yes      | Non-empty.                                                                      |
| `queries[].query`  | string         | \*       | Text — embedded server-side by the active `EmbeddingManager`.                  |
| `queries[].vector` | array<number>  | \*       | Raw dense vector — must match the collection's dimension.                      |
| `queries[].limit`  | number         | no       | Defaults to `10`.                                                               |
| `queries[].threshold` | number      | no       | Similarity threshold filter.                                                    |

\* Exactly one of `query` or `vector` must be provided per entry. Entries
with both missing produce a per-entry error.

### `POST /batch_update`

```json
{
  "collection": "docs",
  "updates": [
    { "id": "doc-001", "vector": [0.1, 0.2, ...] },
    { "id": "doc-002", "payload": { "updated": true } }
  ]
}
```

| Field                  | Type           | Required | Notes                                                                               |
|------------------------|----------------|----------|-------------------------------------------------------------------------------------|
| `collection`           | string         | yes      | Target collection.                                                                   |
| `updates`              | array          | yes      | Non-empty.                                                                           |
| `updates[].id`         | string         | yes      | Missing id → that entry errors; batch continues.                                     |
| `updates[].vector`     | array<number>  | no       | Must match the collection's dimension. Mismatches fail the entry only.              |
| `updates[].payload`    | object / null  | no       | Replaces the stored payload. `null` clears it. Missing field preserves it.           |

### `POST /batch_delete`

```json
{
  "collection": "docs",
  "ids": ["doc-001", "doc-002", "doc-003"]
}
```

| Field          | Type           | Required | Notes                                                         |
|----------------|----------------|----------|---------------------------------------------------------------|
| `collection`   | string         | yes      | Target collection.                                             |
| `ids`          | array<string>  | yes      | Non-empty. Non-string entries fail per-item, batch continues. |

---

## Response shape

### Insert response

```json
{
  "collection": "docs",
  "inserted": 97,
  "failed": 3,
  "count": 100,
  "results": [
    {
      "index": 0,
      "client_id": "doc-001",
      "status": "ok",
      "vector_ids": ["doc-001-chunk-0", "doc-001-chunk-1"],
      "vectors_created": 2,
      "chunked": true
    },
    {
      "index": 17,
      "client_id": "doc-018",
      "status": "error",
      "error": "missing or invalid text field",
      "error_type": "validation_error"
    }
  ]
}
```

### Search response

```json
{
  "collection": "docs",
  "count": 5,
  "succeeded": 4,
  "failed": 1,
  "results": [
    {
      "index": 0,
      "status": "ok",
      "query": "vector database",
      "total_results": 5,
      "results": [ { "id": "...", "score": 0.91, "payload": { ... } }, ... ]
    },
    {
      "index": 3,
      "status": "error",
      "query": null,
      "error": "entry[3] missing both `query` and `vector`",
      "error_type": "validation_error"
    }
  ]
}
```

### Update / Delete response

Update responses use `{collection, count, updated, failed, results}`; delete
responses use `{collection, count, deleted, failed, results}`. Per-item
entries are `{index, id, status: "ok"|"error", error?}`. Example update
failure: `{"index": 42, "id": "doc-043", "status": "error", "error":
"vector dim 512 != collection dim 768"}`.

### Top-level HTTP status

All four REST endpoints return:

- **HTTP 200** when the request itself was well-formed, regardless of how
  many individual items failed. Inspect `failed` / per-item `status` to
  detect partial failures.
- **HTTP 400** only when the top-level request is malformed (missing
  `collection`, missing / empty `texts` / `queries` / `updates` / `ids`
  arrays, or — for updates — the target collection cannot be located).
- Other 4xx/5xx codes follow the server-wide error middleware (auth
  failures, rate limits, etc.). These are *batch-level* failures: no items
  were attempted.

> Vectorizer's REST batch endpoints do **not** return HTTP 207 Multi-Status.
> Partial failures are expressed inside the 200 body via the `failed`
> counter and per-item `status: "error"`.

---

## Atomic vs partial semantics

This is the highest-leverage piece of information on this page — read it
carefully.

### REST endpoints: always partial

All four REST handlers iterate over the request array, attempt each entry
independently, and capture failures into `results[]`. A batch of 100 inserts
where 3 fail produces `{"inserted": 97, "failed": 3}` and the 97 successes
are **persisted and visible** to subsequent reads. There is no rollback.

Source of truth: `do_batch_insert_texts`
(`.../rest_handlers/vectors.rs:271-419`), `batch_search_vectors`
(`search.rs:539-668`), `batch_update_vectors` (`search.rs:677-860`),
`batch_delete_vectors` (`search.rs:868-952`). Each uses a per-entry
`match outcome { Ok => inserted+=1; Err => failed+=1 }` loop.

This is by design — rejecting 100 writes because one payload was malformed
would make bulk ingestion fragile. Always inspect `failed` and iterate
`results[]` for per-item errors.

### In-process Rust API: atomic flag, opt-in

`BatchProcessor::{batch_insert,batch_update,batch_delete,batch_search}`
(`crates/vectorizer/src/batch/processor.rs`) each take
`atomic: Option<bool>`. Default comes from `BatchConfig::atomic_by_default`
(default `true`).

- **`atomic = true`**: The whole batch succeeds or one `BatchError` is
  returned. For `batch_insert`, `VectorStore::insert(&collection, vectors)`
  is called *once with the full list*; all commit or none do.
  - *Exception* for updates/deletes/search (processor.rs:260-344): those
    "atomic" paths still iterate internally and return on the **first**
    failure. Prior successes in the same batch have already been applied —
    "fail-fast", not transactional rollback. Treat the atomic flag as
    strict only for `batch_insert`.
- **`atomic = false`**: All items attempted; partial success is returned as
  `Err(InternalError("Partial success with N errors"))`. *Successes are
  persisted* even though the `Result` is `Err`.

Practical guidance: if you're on REST you always get partial success — plan
for it. If you're linking the Rust library and you need "all or nothing",
use `atomic=true` with `batch_insert` specifically and validate inputs
client-side.

---

## Error code registry

`BatchErrorType` is defined in
`crates/vectorizer/src/batch/error.rs`. Error codes serialize as
`BATCH_<snake_case_variant>` (see `BatchError::error_code()`). `is_retryable`
and `should_retry(max)` policy is defined at `error.rs:129-141`.

| Variant (error_code)                    | Retryable | Meaning                                                                                   | Operator action                                                           |
|-----------------------------------------|-----------|-------------------------------------------------------------------------------------------|---------------------------------------------------------------------------|
| `BATCH_invalid_batch_size`              | no        | Batch length > `max_batch_size` or 0.                                                     | Split the batch client-side (chunk by `chunk_size`).                     |
| `BATCH_invalid_vector_data`             | no        | Non-numeric vector entries or wrong dimension.                                            | Fix client serialization.                                                 |
| `BATCH_invalid_vector_id`               | no        | Missing / malformed id.                                                                   | Require ids client-side.                                                  |
| `BATCH_invalid_collection`              | no        | Collection name missing or collection does not exist.                                     | Create the collection first.                                              |
| `BATCH_invalid_query`                   | no        | Search entry missing both `query` and `vector`, or malformed filters.                     | Fix the query.                                                            |
| `BATCH_memory_limit_exceeded`           | no        | `estimate_memory_usage(n, dim) > max_memory_usage_mb`.                                    | Reduce batch size or raise `BatchConfig::max_memory_usage_mb`.            |
| `BATCH_timeout_exceeded`                | **yes**   | Wall-clock exceeded `BatchConfig::operation_timeout_seconds`.                             | Retry with smaller batch; check downstream HNSW lag.                      |
| `BATCH_resource_unavailable`            | **yes**   | Required resource (embedder, GPU, connection) temporarily not ready.                      | Retry with backoff.                                                       |
| `BATCH_concurrent_batch_limit_exceeded` | no        | Concurrent in-flight batches > `max_concurrent_batches`.                                  | Backpressure on caller; queue client-side.                                |
| `BATCH_embedding_generation_failed`     | no        | The embedding backend rejected the text (e.g. input too long for the model).              | Clean/truncate text; pre-compute embeddings client-side if recurring.     |
| `BATCH_vector_store_error`              | **yes**   | `VectorStore` returned an error (I/O, lock contention, index corruption).                 | Retry once; if persistent, inspect server logs.                           |
| `BATCH_indexing_error`                  | no        | HNSW index failed to accept the vector.                                                   | Inspect the specific `error_message`; may indicate a dimension mismatch.  |
| `BATCH_serialization_error`             | no        | Failed to (de)serialize a payload.                                                        | Fix the client payload.                                                   |
| `BATCH_internal_error`                  | no        | Catch-all. In non-atomic mode, used for "Partial success with N errors" wrapping.         | See `error_message` and, for partial-success cases, the results list.     |
| `BATCH_database_error`                  | no        | Persistence-layer error (mmap, `.vecdb`).                                                 | Check disk, inodes, file permissions.                                     |
| `BATCH_network_error`                   | **yes**   | Downstream network failure (replication, remote embedder, gRPC transport).                | Retry with backoff.                                                       |
| `BATCH_authentication_error`            | no        | JWT/API-key rejected mid-batch.                                                           | Refresh credentials. Do **not** retry with the same token.                |
| `BATCH_custom:<msg>`                    | no        | Custom error injected by an extension.                                                    | Read the message.                                                         |

### Retry policy

`BatchError::is_retryable()` returns `true` **only** for `TimeoutExceeded`,
`ResourceUnavailable`, `NetworkError`, and `VectorStoreError`. Everything
else is permanent until the caller changes something.
`should_retry(max_retries) = is_retryable() && retry_count < max_retries`.

The library retry loop (`BatchConfig::error_retry_attempts`, default `3`)
uses a **flat** `error_retry_delay_ms` between attempts (default 100 ms),
not exponential. Wrap the call yourself for exponential backoff.

> REST callers receive `results[].error_type` (not a `BatchError`). Use the
> same retryable set (`timeout_exceeded`, `resource_unavailable`,
> `network_error`, `vector_store_error`) to decide whether to retry
> individual items.

---

## Size limits & tuning

Limits come from `BatchConfig` in
`crates/vectorizer/src/batch/config.rs`. They apply to the in-process
`BatchProcessor`; the REST handlers do **not** currently enforce them, so
the practical cap on a REST request is (a) the HTTP body limit in Axum
and (b) memory on the server.

### Defaults (`BatchConfig::default`)

| Field                         | Default | Purpose                                                    |
|-------------------------------|---------|------------------------------------------------------------|
| `max_batch_size`              | 1000    | Hard cap on items per call.                                |
| `max_memory_usage_mb`         | 512     | Estimated payload + metadata + processing overhead.        |
| `parallel_workers`            | 4       | Tokio tasks working on chunks concurrently.                |
| `chunk_size`                  | 100     | Chunk size for parallel processing; must be `<= max_batch_size`. |
| `atomic_by_default`           | true    | When `atomic=None` is passed, use atomic mode.             |
| `progress_reporting`          | true    | Emit `BatchProgress` updates.                              |
| `error_retry_attempts`        | 3       | Library-level retry count for retryable errors.            |
| `error_retry_delay_ms`        | 100     | Flat delay between retries.                                |
| `operation_timeout_seconds`   | 300     | 5-minute wall-clock cap per batch.                         |
| `max_concurrent_batches`      | 10      | Concurrent in-flight batches per process.                  |
| `enable_compression`          | false   | Compress large payloads (library-internal).                |
| `compression_threshold_bytes` | 1 MiB   | Above this size, compress (when enabled).                  |

### Profile presets

- `BatchConfig::production()` — `max_batch_size=10_000`, 8 workers, 10-min
  timeout, progress reporting off. Use on dedicated ingestion nodes.
- `BatchConfig::development()` — `max_batch_size=100`, 2 workers, 1-min
  timeout, progress reporting on. Use locally.
- `BatchConfig::testing()` — `max_batch_size=10`, 1 worker, retries off.

### Memory estimation

`BatchConfig::estimate_memory_usage(n, dim)` uses

```
(n * dim * 4 bytes) + (n * 512 bytes metadata) + (n * 64 bytes overhead)
```

### Client-side tuning guidance

Estimated payloads: 384-dim × 1 000 ≈ 2 MB; 768-dim × 1 000 ≈ 3.6 MB;
1536-dim × 1 000 ≈ 6.8 MB. Multiply linearly.

Rules of thumb:

- For 768-dim vectors, batches of ~1 000 are safe under the default
  `max_memory_usage_mb=512` with plenty of headroom.
- Beyond ~10 000 items, split client-side even when the server would accept
  it — larger batches hurt tail latency and make retries expensive.
- `chunk_size` must be `<= max_batch_size`; the validator (`config.rs:147`)
  rejects a config otherwise.

### What happens when a limit is exceeded

- Too many items → `BATCH_invalid_batch_size`.
- Estimated memory too high → `BATCH_memory_limit_exceeded`.
- Operation runs longer than `operation_timeout_seconds` →
  `BATCH_timeout_exceeded` (retryable).
- Too many concurrent batches → `BATCH_concurrent_batch_limit_exceeded`.

---

## Progress tracking

Progress is available from the in-process API only. It is **not** exposed
over REST today — REST callers should poll collection stats or use the
metrics endpoint to monitor long-running ingestion.

`crates/vectorizer/src/batch/progress.rs` provides:

- `ProgressTracker::new(total_items, sender: Option<UnboundedSender<BatchProgress>>)`
  — create a tracker. When `sender` is `Some`, every state change is pushed
  onto the channel.
- `ProgressTracker::update_success()` / `update_failure()` /
  `update_batch(successful, failed)` — increment counters.
- `ProgressTracker::get_progress()` → `BatchProgress` snapshot.
- `ProgressTracker::is_complete()` → `bool`.
- `ProgressTracker::start_auto_updates()` — spawns a Tokio task that pushes
  the snapshot to `sender` every 100 ms.
- `ProgressBar::display()` — terminal progress bar for CLIs.

`BatchProgress` fields:

```rust
pub struct BatchProgress {
    pub total_items: usize,
    pub processed_items: usize,
    pub successful_items: usize,
    pub failed_items: usize,
    pub processing_rate: f64,           // items/sec
    pub estimated_remaining_seconds: f64,
    pub current_memory_mb: f64,
}
```

Example — library consumer with progress reporter:

```rust
use tokio::sync::mpsc;
use vectorizer::batch::progress::ProgressTracker;

let (tx, mut rx) = mpsc::unbounded_channel();
let tracker = ProgressTracker::new(vectors.len(), Some(tx));
tracker.start_auto_updates().await;

tokio::spawn(async move {
    while let Some(p) = rx.recv().await {
        eprintln!("{}/{} ({:.1}/s, ETA {:.0}s)",
            p.processed_items, p.total_items,
            p.processing_rate, p.estimated_remaining_seconds);
    }
});
```

`PerformanceMetrics::from_progress(&progress, total_time)` converts a final
snapshot into a structured metrics object (`avg_rate`, `peak_rate`,
`error_rate`, `memory_usage`).

---

## Parallel execution

`crates/vectorizer/src/batch/parallel.rs` provides `ParallelProcessor`:

- `process_chunks(items, F)` — splits `items` into chunks of
  `BatchConfig::chunk_size`, runs up to `BatchConfig::parallel_workers`
  concurrent Tokio tasks (gated by a `Semaphore`), and collects results.
- `process_items(items, F)` — like above but per-item (one task per item,
  still gated).

Concurrency is enforced by `Semaphore::new(parallel_workers)`. If every
worker is busy, new chunks/items wait for a permit — they do not spill to
an unbounded queue.

### Tuning guidance

- **CPU-bound embedding** (local ONNX / FastEmbed): set `parallel_workers`
  to `num_physical_cores` (not hyperthreads); set `chunk_size` so each
  worker gets ~4 chunks for tail balance.
- **GPU embedding** (`hive-gpu`): `parallel_workers=1` or `2`. GPU calls
  serialize internally; more workers add context switching with no gain.
- **I/O-bound** (remote embedder): `parallel_workers=16–32`, small
  `chunk_size` (10–25) so slow remote calls don't block a full chunk.
- **Writer-heavy** (`batch_insert`): larger `chunk_size` (500–1000); the
  HNSW insert path benefits from amortized index acquisition.

REST endpoints process items sequentially inside the handler — they do
**not** go through `ParallelProcessor`. For parallelism over REST, issue
concurrent HTTP requests (respecting `max_concurrent_batches`).

---

## Usage examples

### Bulk import 10 000 vectors over REST (Python)

```python
import httpx, itertools, time

BASE, TOKEN, CHUNK = "http://localhost:15002", "...", 500

def chunks(it, n):
    it = iter(it)
    while (c := list(itertools.islice(it, n))): yield c

docs = [{"id": f"doc-{i}", "text": t} for i, t in enumerate(corpus)]

with httpx.Client(headers={"Authorization": f"Bearer {TOKEN}"}, timeout=120) as c:
    ok = fail = 0
    for batch in chunks(docs, CHUNK):
        for attempt in range(3):
            r = c.post(f"{BASE}/batch_insert",
                       json={"collection": "docs", "texts": batch})
            if r.status_code == 200: break
            if r.status_code in (429, 503): time.sleep(2 ** attempt); continue
            r.raise_for_status()
        body = r.json()
        ok += body["inserted"]; fail += body["failed"]
        for item in body["results"]:
            if item["status"] == "error": log_failure(item)
    print(f"Ingest: {ok} ok, {fail} failed")
```

### Bulk delete by id list

```bash
curl -sS -X POST "$BASE/batch_delete" \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "collection": "docs",
    "ids": ["doc-001","doc-002","doc-003"]
  }' | jq '{deleted, failed, errors: [.results[] | select(.status=="error")]}'
```

### Concurrent hybrid searches over one collection

```python
queries = [
    {"query": "vector database",  "limit": 5},
    {"query": "semantic search",  "limit": 5},
    {"query": "embedding models", "limit": 5},
]
resp = client.post("/batch_search", json={"collection": "docs", "queries": queries}).json()
assert resp["failed"] == 0, resp
for r in resp["results"]:
    print(r["query"], "->", [(h["id"], h["score"]) for h in r["results"]])
```

---

## Performance expectations

The batch module's stated targets (in `crates/vectorizer/src/batch/mod.rs:13-17`):

- 10 000 vectors/second batch insert.
- ~10× throughput improvement over single-op endpoints.
- < 100 ms latency for batches up to 1 000 operations.

These numbers come from the design spec and are **not** the result of a
recent benchmark on arbitrary hardware. Treat them as a ceiling, not a
contract. For your own workload, measure with `/metrics` (Prometheus
exporter) — key series:

- `vectorizer_insert_requests_total{status="success"|"error"}`
- `vectorizer_insert_latency_seconds` (histogram)
- `vectorizer_batch_active_operations`

A realistic baseline to aim for on commodity hardware (8 cores, 32 GB RAM,
NVMe, 768-dim ONNX embedder): **~2 000–4 000 inserts/sec** sustained when
chunking at 500 items/batch.

---

## Common pitfalls

1. **Mixing batch writes with high-rate individual writes leaves HNSW
   lagged.** Hammering both `/batch_insert` and `/insert` concurrently from
   many clients causes transient index staleness (reads see committed
   payloads but not yet indexed vectors). Prefer one writer pattern per
   collection during bulk ingest.
2. **Retrying a non-retryable error duplicates data.** `invalid_vector_data`
   and `embedding_generation_failed` fail forever with the same input. Gate
   retries on the retryable set only: `timeout_exceeded`,
   `resource_unavailable`, `network_error`, `vector_store_error`.
3. **Ignoring `failed` in the response is silent data loss.** REST endpoints
   return HTTP 200 even when some items failed. Always assert
   `body["failed"] == 0` or iterate `body["results"]` for
   `status == "error"`.
4. **Assuming atomicity when you're not using `atomic=true`.** REST is
   *always* partial. The Rust library with `atomic=true` is strict only for
   `batch_insert`; `batch_update` / `batch_delete` / `batch_search` with
   `atomic=true` stop on the first failure but already-applied operations
   persist.
5. **Batches that outrun `operation_timeout_seconds`.** The batch aborts
   with `timeout_exceeded` but items processed before the timeout are
   persisted. Always set a client-side `id` so upsert semantics deduplicate
   on retry, or plan for duplicates.
6. **Duplicate `id`s within the same batch.** REST handlers do not
   deduplicate: a later entry overwrites an earlier entry's payload. Dedupe
   client-side if you don't want that.
7. **Dimension mismatches in `batch_update` fail per-entry.** The handler
   rejects the entry with `"vector dim N != collection dim M"` and the rest
   of the batch continues. A burst usually means an embedding-model /
   collection-config drift.
8. **Auth expiring mid-batch.** Long batches can outlive short-lived JWTs;
   remaining items get `authentication_error`. Use long-lived API keys for
   ingestion workloads, or refresh tokens proactively.
9. **Hot-looping `batch_search` against one collection.** Search batches
   serialize inside the handler (no internal parallelism on REST). For high
   QPS, issue concurrent `/search` requests and rely on the query cache.
10. **Query cache invalidates per successful write batch.** `batch_update`
    / `batch_delete` in a tight loop thrash the cache. Cluster updates into
    larger batches.

---

## Related documents

- [`API_REFERENCE.md`](./API_REFERENCE.md) — full REST surface (batch
  entries at §Batch Operations).
- [`AUTHENTICATION.md`](./AUTHENTICATION.md) — how tokens / API keys apply to
  batch calls.
- [`../search/`](../search/) — non-batch search endpoints, hybrid search,
  ranking.
- [`../vectors/`](../vectors/) — single-vector insert / update / delete
  endpoints.
- [`../operations/`](../operations/) — Prometheus metrics, ingestion SLOs.
