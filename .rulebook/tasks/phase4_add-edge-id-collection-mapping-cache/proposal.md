# Proposal: phase4_add-edge-id-collection-mapping-cache

## Why

`DELETE /graph/edges/{edge_id}` in `src/api/graph.rs` (L668) currently has to scan every collection to find the owning graph, because there is no persistent `edge_id → collection_name` mapping. On a deployment with N collections and M edges per collection, a single delete is `O(N·M)` in the worst case — unacceptable for any large setup.

## What Changes

1. Add an `edge_index: Arc<DashMap<String, String>>` to `GraphApiState` keyed by `edge_id` with value `collection_name`.
2. Populate the index on edge creation and clean up on deletion.
3. In `delete_edge`, consult the index first; fall back to the scan only as a correctness safety net when the index misses.
4. Make the index durable so restarts don't lose it — either rebuild from a scan of all graphs on startup, or persist alongside the graph data in `.vecdb`.

## Impact

- Affected specs: none directly; the REST contract is unchanged.
- Affected code: `src/api/graph.rs`, whatever owns `GraphApiState`, and the persistence layer if we go durable.
- Breaking change: NO.
- User benefit: `O(1)` edge deletion instead of `O(N·M)` — meaningful on multi-collection graphs.
