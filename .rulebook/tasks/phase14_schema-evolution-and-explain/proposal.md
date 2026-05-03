# Proposal: phase14_schema-evolution-and-explain

Source: gap audit follow-up to phase11_sdk-tier-demotion-api. Companion
to phase12 (parity) and phase13 (tier control).

## Why

Operators that own production Vectorizer deployments cannot today:

1. **Rename a collection** without recreating it and updating every
   consumer's name reference.
2. **Reindex** with new HNSW parameters (e.g. raising `M` from 16 to
   32) without rebuilding the dataset and re-embedding everything.
3. **Snapshot a native (non-Qdrant) collection** — snapshots only exist
   under `/qdrant/...` routes today; the native collection model has no
   equivalent.
4. **Explain a search query** to understand why a slow query was slow
   (HNSW visit count, layers traversed, payload-filter cost).
5. **Inspect slow queries** without scraping logs.

These are standard "Day-2 ops" capabilities that any non-trivial
deployment ends up needing. Without them, every operator builds the
same external tooling (rename = manual reindex script, slow-query log =
log-aggregator query, explain = read source). This task closes those
gaps with server-side primitives so the SDKs can expose them uniformly.

## What Changes

### 1. Server — schema evolution endpoints

| Method | Route | Purpose |
|---|---|---|
| `POST` | `/collections/{name}/rename` | Atomically rename a collection; old name kept as alias for one major version |
| `POST` | `/collections/{name}/reindex` | Rebuild the HNSW index with new parameters; same vectors, same encoding |
| `POST` | `/collections/{name}/snapshot` | Create a native snapshot file under the configured backup dir |
| `GET`  | `/collections/{name}/snapshots` | List native snapshots |
| `POST` | `/collections/{name}/snapshots/{snapshot_id}/restore` | Restore from a native snapshot |

### 2. Server — observability endpoints

| Method | Route | Purpose |
|---|---|---|
| `POST` | `/collections/{name}/explain` | Run a search and return the execution trace alongside results |
| `GET`  | `/slow_queries` | Read the in-memory slow-query log |
| `POST` | `/slow_queries/config` | Configure threshold and ring buffer size |

### 3. Implementation

- **Rename**: atomic swap in the `VectorStore` index; keep a tombstone
  alias in the persistence layer for one minor version.
- **Reindex**: build new HNSW off-line with new `M`, `ef_construction`,
  `ef_search` parameters from existing vectors; atomic swap.
- **Native snapshot**: leverage existing `.vecdb` persistence layer;
  emit a single tarball per collection.
- **Explain**: instrument HNSW search to record `visited_nodes`,
  `layer_path`, `payload_filter_evals`, `quantization_score_ms`. Return
  alongside hits.
- **Slow-query log**: ring buffer in `crates/vectorizer/src/cache/`
  capturing queries above the configured latency threshold.

### 4. SDKs (Rust + TS + Python)

For each new server route, expose a typed method in all three SDKs.

```rust
client.rename_collection(old, new)                     -> Result<()>
client.reindex_collection(name, params)                -> Result<ReindexJob>
client.snapshot_collection_native(name, request)       -> Result<SnapshotInfo>
client.list_collection_snapshots_native(name)          -> Result<Vec<SnapshotInfo>>
client.restore_collection_snapshot_native(name, id)    -> Result<()>
client.explain_search(collection, request)             -> Result<ExplainResponse>
client.list_slow_queries(params)                       -> Result<Vec<SlowQueryEntry>>
client.set_slow_query_config(config)                   -> Result<SlowQueryConfig>
```

Workspace bump: server + SDKs 3.5 → 3.6.

## Impact

- Affected specs: `.rulebook/tasks/phase14_schema-evolution-and-explain/specs/schema-evolution-and-explain/spec.md`
- Affected code:
  - `crates/vectorizer-server/src/server/core/routing.rs`
  - `crates/vectorizer-server/src/server/rest_handlers/collections.rs`
  - `crates/vectorizer-server/src/server/rest_handlers/search.rs`
  - `crates/vectorizer/src/db/` (rename atomicity, native snapshot)
  - `crates/vectorizer/src/index/hnsw/` (reindex pipeline, instrumented search for explain)
  - `crates/vectorizer/src/cache/` (slow-query ring buffer)
  - `sdks/{rust,typescript,python}/...`
- Breaking change: NO — additive.
- User benefit: operators can resize HNSW parameters, rename without
  downtime, snapshot natively, and debug slow queries without leaving
  the SDK.

## Constraints

- `rename_collection` MUST keep the old name reachable as an alias for
  at least one minor version (3.6 → 3.7) for client migration.
- `reindex` MUST NOT require re-embedding; it operates on stored
  vectors only.
- `explain` MUST run the same search code path as production search;
  no separate engine that could drift from real behavior.
- Slow-query log MUST be in-memory (ring buffer) — no disk writes —
  and capped by config.

## Acceptance

- Eight new server routes with handlers + integration tests.
- Reindex test: collection with `M=16` reindexed to `M=32`; recall
  improvement and latency change recorded in `benches/reports/`.
- Rename test: alias resolves the old name to the new collection for
  one minor version, then is removed.
- Explain test: a search returns a trace whose `visited_nodes` is
  consistent with HNSW config.
- Slow-query log test: a 200 ms query above a 100 ms threshold appears
  in the ring buffer.
- All three SDKs expose the eight methods.
- Server + SDKs bumped 3.5 → 3.6.
