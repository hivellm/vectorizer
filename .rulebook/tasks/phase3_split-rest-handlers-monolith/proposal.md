# Proposal: phase3_split-rest-handlers-monolith

## Why

`src/server/rest_handlers.rs` is **3,795 lines** containing 22+ handler functions, mixed argument parsing, UUID generation TODOs, and silent-error patterns. This is a classic Cursor-generated god-file. Problems:

- Change review is impossible at scale — any PR touching this file reads as "massive".
- 15 unsafe `.ok().unwrap()` chains are hidden in the noise.
- Two handlers writing to the same resource can drift in validation/error-shape without anyone noticing.
- No easy way to reuse per-resource helpers between handlers.
- IDE navigation / symbol search becomes painful.

Splitting by resource aligns with REST conventions and makes `phase1_protect-admin-setup-routes` (auth buckets) and `phase3_reduce-unwrap-in-handlers` drastically easier to apply.

## What Changes

Split `rest_handlers.rs` into a submodule tree under `src/server/handlers/`:

- `handlers/mod.rs` — re-exports + shared helpers
- `handlers/collections.rs` — CRUD on collections
- `handlers/vectors.rs` — insert/upsert/delete/get vectors
- `handlers/search.rs` — search / recommend / scroll
- `handlers/aliases.rs` — alias create/delete/resolve
- `handlers/snapshots.rs` — snapshots, backups, restore
- `handlers/admin.rs` — setup, config, restart, workspace
- `handlers/intelligent_search.rs` — intelligent search endpoints
- `handlers/graph.rs` — graph endpoints
- `handlers/meta.rs` — health, info, metrics exposure

Each file ≤500 LOC. Shared helpers (error mapping, pagination, auth extractor) go to `handlers/common.rs`. The router in `src/server/mod.rs` imports explicit handler fns.

## Impact

- Affected specs: none (internal refactor)
- Affected code: `src/server/rest_handlers.rs` (deleted), `src/server/handlers/*` (new), `src/server/mod.rs` (imports)
- Breaking change: NO — public API, behavior, and route paths unchanged; only internal structure.
- User benefit: reviewable diffs, isolated bug fixes, enables stricter per-resource auth/validation, enables parallel development without merge conflicts.
