# Proposal: phase13_tier-control-primitives

Source: gap audit follow-up to phase11_sdk-tier-demotion-api. Companion
to phase12_sdk-control-surface-parity.

## Why

Phase11 added `move_to_collection` so Cortex's pruner can demote source
events between hot/warm/cold collections. But the demotion model still
forces clients to enumerate IDs client-side and orchestrate
get-insert-delete loops for any *bulk* tier transition. That:

1. Burns N round trips for a sweep that is logically a single
   server-side filter+update.
2. Cannot express "delete every vector older than 90 days" without
   first scrolling the entire collection.
3. Cannot demote *encoding* (fp32 → PQ) on the same collection — a
   client must create a new collection, move every vector, drop the
   old one, and update every consumer's name reference.
4. Has no per-vector TTL, so audit logs and session vectors must be
   pruned by external schedulers.

Cortex's consolidation tier (phase11j) and any production deployment
with retention policies need server-side primitives for these
operations. Without them, every consumer reinvents the same loop.

## What Changes

### 1. Server — new endpoints (additive)

| Method | Route | Purpose |
|---|---|---|
| `POST` | `/collections/{name}/vectors/delete_by_filter` | Delete every vector matching a metadata predicate |
| `POST` | `/collections/{name}/vectors/bulk_update_metadata` | Patch metadata on every vector matching a predicate |
| `POST` | `/collections/{src}/vectors/copy` | Copy (NOT move) vectors to a destination collection |
| `POST` | `/collections/{name}/reencode` | Re-quantize an existing collection in-place (e.g. fp32 → PQ) without re-embedding |
| `POST` | `/collections/{name}/ttl` | Set or clear a per-collection vector TTL |
| `PATCH` | `/collections/{name}/vectors/{id}/expiry` | Set per-vector `expires_at` |

All payloads carry per-id results in the same shape as
`MoveVectorsResponse` so error handling is uniform.

### 2. Server — TTL reaper

A background sweeper that scans the per-collection TTL index every
`ttl_check_interval_secs` (config) and deletes expired vectors. Per-shard
job to avoid a global lock; metrics exposed under `/stats`.

### 3. Re-encoding pipeline

`reencode` MUST: (a) snapshot the source collection, (b) build the new
quantized index off-line, (c) atomically swap the index, (d) keep the
snapshot until the next successful save. No re-embedding.

### 4. SDKs (Rust + TS + Python)

For each new server route, expose a typed method in all three SDKs.

```rust
client.delete_by_filter(collection, filter)            -> Result<DeleteByFilterReport>
client.bulk_update_metadata(collection, filter, patch) -> Result<BulkUpdateReport>
client.copy_vectors(src, dst, ids)                     -> Result<CopyReport>
client.reencode_collection(name, target_encoding)      -> Result<ReencodeJob>
client.set_collection_ttl(name, ttl_secs)              -> Result<()>
client.set_vector_expiry(collection, id, expires_at)   -> Result<()>
```

Workspace bump: SDKs 3.4 → 3.5, server 3.4 → 3.5 (server feature add).

## Impact

- Affected specs: `.rulebook/tasks/phase13_tier-control-primitives/specs/tier-control-primitives/spec.md`
- Affected code:
  - `crates/vectorizer-server/src/server/core/routing.rs` (new routes)
  - `crates/vectorizer-server/src/server/rest_handlers/vectors.rs` (handlers)
  - `crates/vectorizer-server/src/server/rest_handlers/collections.rs` (`reencode`, `set_collection_ttl`)
  - `crates/vectorizer/src/db/` (TTL index, reaper, re-encoding pipeline)
  - `crates/vectorizer/src/quantization/` (in-place re-quantization without re-embed)
  - `sdks/{rust,typescript,python}/...` (six new methods + report types)
- Breaking change: NO — additive. Existing routes/methods unchanged.
- User benefit:
  - Cortex consolidation tier can demote encoding without rebuilding the dataset.
  - One server call replaces N-round-trip pruner loops.
  - Per-vector and per-collection TTL eliminates external retention cron jobs.

## Constraints

- `delete_by_filter` and `bulk_update_metadata` MUST be idempotent at
  the per-vector level (re-running yields the same end-state).
- `reencode` MUST be reversible up to the next save: the snapshot is
  retained until the swap succeeds and the next persistence checkpoint
  completes.
- TTL reaper MUST NOT block writes; uses background tokio task per
  collection.
- Filter syntax reuses the existing search-side filter grammar (no new
  DSL).

## Acceptance

- Six new server routes with handlers + integration tests covering
  happy path, partial-failure surfacing, and concurrency with writes.
- Re-encoding test: build a 384-dim fp32 collection, reencode to PQ,
  verify search recall stays within target tolerance.
- TTL reaper test: write vectors with `expires_at` 100 ms in the
  future, wait, confirm they are gone and metrics report it.
- All three SDKs expose the six methods with typed reports.
- Server + SDK versions bumped 3.4 → 3.5.
