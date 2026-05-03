# Proposal: phase11_sdk-tier-demotion-api

Source: https://github.com/hivellm/vectorizer/issues/265
Cortex tracking: phase11o_vectorizer_demotion_api in https://github.com/hivellm/cortex

## Why

Cortex's consolidation tier (phase11j) ships a pruning daemon that demotes
source events between Vectorizer collections on a hot/warm/cold schedule
(0-7d → 7-90d → 90-365d → expire). Without server-side move + SDK delete
primitives the pruner can only no-op on the vector side, the warm + cold
tiers stay empty, and the hot tier grows monotonically — which defeats
the consolidation tier's cost model.

Audit of `vectorizer-sdk = "3.2"` against the demotion need:

| Need                              | SDK 3.2                       | Status |
|-----------------------------------|-------------------------------|--------|
| Read a vector by id               | `get_vector(collection, id)`  | exists |
| Insert into another collection    | `insert_texts(...)`           | embeds anew, doesn't transfer the source vector |
| Delete a single vector            | (none)                        | missing in SDK; server has `DELETE /collections/{name}/vectors/{id}` (rest_handlers/vectors.rs:149) |
| Batch delete                      | (none)                        | missing in SDK; server has `POST /batch_delete` (rest_handlers/search.rs:868) |
| Move vectors between collections  | (none)                        | missing on both sides |

Two of three gaps are SDK-only. The third (move) needs a new server
endpoint with insert-before-delete ordering so a mid-batch crash leaves
a duplicate (recoverable) instead of data loss.

## What Changes

### 1. Server — new endpoint `POST /collections/{src}/vectors/move`

Body:

```json
{ "destination": "<dst-collection>", "ids": ["vec-1", "vec-2"] }
```

Response:

```json
{
  "src": "cortex.consolidation.fp32",
  "dst": "cortex.consolidation.pq",
  "requested": 2,
  "moved": 2,
  "failed": 0,
  "results": [
    { "id": "vec-1", "status": "ok" },
    { "id": "vec-2", "status": "ok" }
  ]
}
```

Per-id failure statuses: `missing_in_src`, `dst_insert_failed`,
`src_delete_failed`. Handler MUST insert into dst first, only then delete
from src. Per-id failures captured in `results` without aborting the
batch (operator wants partial progress for tier-demotion sweeps).

### 2. Rust SDK (`sdks/rust/src/client/vectors.rs`)

```rust
pub async fn delete_vector(&self, collection: &str, vector_id: &str) -> Result<()>;
pub async fn delete_vectors(&self, collection: &str, ids: &[String]) -> Result<DeleteReport>;
pub async fn move_to_collection(&self, src: &str, dst: &str, ids: &[String]) -> Result<MoveReport>;
```

`DeleteReport` and `MoveReport` mirror the per-id `results` shape.

### 3. TypeScript + Python SDKs

Same three methods, matching the existing client surface for those SDKs
(`sdks/typescript/src/client/vectors.ts`, `sdks/python/...`).

## Impact

- Affected specs: `.rulebook/tasks/phase11_sdk-tier-demotion-api/specs/sdk-tier-demotion-api/spec.md`
- Affected code:
  - `crates/vectorizer-server/src/server/core/routing.rs` (new route)
  - `crates/vectorizer-server/src/server/rest_handlers/vectors.rs` (new handler `move_vectors`)
  - `sdks/rust/src/client/vectors.rs` (three new methods + report types)
  - `sdks/typescript/src/client/vectors.ts` (three new methods)
  - `sdks/python/...` (three new methods)
  - SDK READMEs + server API reference under `docs/`
- Breaking change: NO — purely additive on the wire; SDK semver minor 3.2 → 3.3
- User benefit: unblocks Cortex's tier-demotion pruner; gives every Vectorizer
  consumer a delete + cross-collection-move primitive that until now had to
  be open-coded against raw HTTP.

## Constraints

- Atomicity per vector: dst insert before src delete. No cross-collection
  transaction expected — best-effort with clear error surfacing.
- Dim / encoding mismatch between src and dst: surface in
  `MoveReport.results` per-row; do NOT pre-validate (operator may
  intentionally cross encoding tiers).
- Back-compat: additive only. Existing 3.2 clients keep working.

## Acceptance

- Server `POST /collections/{src}/vectors/move` route lands with handler
  + integration test (insert-before-delete ordering verified).
- Rust SDK exposes the three methods returning typed `Result`s.
- TS + Python SDKs expose the same three methods.
- SDK READMEs + server API reference document the surface.
- Workspace SDK version bumped 3.2 → 3.3.
