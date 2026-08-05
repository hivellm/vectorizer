# Proposal: phase1_collections-list-hides-lazy-load-progress

Fixes [#391](https://github.com/hivellm/vectorizer/issues/391).

## Why

`GET /collections` answers from whatever the store holds *right now*. At boot
the server spawns a background task (`bootstrap.rs:502`) that calls
`load_all_persisted_collections()`, which inserts collections into the store
one by one as it walks the `.vecdb` catalog. A client that asks during that
window gets a truthful-looking but partial answer.

Observed on `hivehub/vectorizer:3.6.0-fastembed`, ~20s after container start:
**11 collections / 15,100 vectors** for a store that actually holds **181
collections / 133,081 vectors**. Minutes later the same call returned 181.

The response carries nothing that distinguishes this from real data loss:

```json
{ "collections": [ ...11 items... ], "total_collections": 11 }
```

`total_collections` is `collections.len()` (`collections.rs:165`), so it
*agrees* with the partial list and reinforces the wrong conclusion. During the
3.5→3.6 upgrade verification this read as catastrophic loss and triggered
rollback procedures for what was normal warm-up.

The failure mode is worse than a plain outage: an empty or erroring endpoint
makes an operator wait, whereas a confident partial answer makes them act.

## What Changes

Publish load progress and let every read surface it.

1. **`CollectionLoadProgress`** — a small shared counter (`expected`,
   `loaded`, `complete`, plus the failure flag) in the `vectorizer` crate,
   held by the server as `Arc<...>` and passed into a new tracked loader
   entry point. `extract_all_collections()` already materializes the whole
   catalog before the insertion loop, so `expected` is knowable up front and
   `loaded` increments per collection.

2. **`GET /collections`** gains `loading`, `loaded_collections` and
   `expected_collections`. `total_collections` keeps its current meaning
   (count of items in this response) so existing readers are untouched —
   additive, same tactic as the search-envelope mirror in `e9a250af`.

3. **`GET /health`** gains a `readiness` block. `status: "healthy"` stays a
   *liveness* answer: the Dockerfile `HEALTHCHECK` probes `/health`, and
   failing it during warm-up would mark every container unhealthy on a normal
   restart.

4. **`GET /ready`** — new: `200` once loading completes, `503` with
   `Retry-After` while it runs. This is the endpoint orchestrators should gate
   traffic on.

Every exit path must settle the flag — including auto-load disabled
(`bootstrap.rs:714`) and the loader error path (`bootstrap.rs:706`). A server
stuck reporting `loading: true` forever would be a worse bug than the one
being fixed.

## Impact

- Affected specs: none existing; behavior recorded in this task's spec.
- Affected code:
  - `crates/vectorizer/src/db/vector_store/persistence/loading.rs` (tracked loader)
  - `crates/vectorizer/src/db/` (new progress type + export)
  - `crates/vectorizer-server/src/server/mod.rs` (state field)
  - `crates/vectorizer-server/src/server/core/bootstrap.rs` (publish progress)
  - `crates/vectorizer-server/src/server/rest_handlers/collections.rs` (list)
  - `crates/vectorizer-server/src/server/rest_handlers/meta.rs` (health)
  - `crates/vectorizer-server/src/server/core/routing.rs` (`/ready` route)
- Breaking change: NO — all additions; no field changes meaning.
- User benefit: an upgrade no longer looks like data loss. Operators get a
  machine-readable "still warming up, 11 of 181" instead of having to infer it,
  and orchestrators get a real readiness gate.
