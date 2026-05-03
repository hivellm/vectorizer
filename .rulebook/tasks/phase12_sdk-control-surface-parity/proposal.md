# Proposal: phase12_sdk-control-surface-parity

Source: gap audit follow-up to phase11_sdk-tier-demotion-api (see
`.rulebook/tasks/phase11_sdk-tier-demotion-api/proposal.md`).

## Why

Phase11 added three primitives (`delete_vector`, `delete_vectors`,
`move_to_collection`) because the SDKs lacked them. An audit of
`crates/vectorizer-server/src/server/core/routing.rs:148-786` against
`sdks/{rust,typescript,python}/src/client/` reveals ~30 more server
routes with NO SDK coverage. Every consumer that wants those features
today must hand-roll HTTP calls — exactly the situation phase11 was
filed to remove. This task closes the parity gap in one sweep so we
stop landing N one-off "expose route X in SDK" tasks.

Concretely the gap covers:

- **Single-vector ops**: `update_vector`, `insert_text` (single),
  `list_vectors`, native `GET /collections/{n}/vectors/{id}`.
- **Batch ops**: `batch_insert_texts`, `insert_vectors`,
  `batch_search_vectors`, `batch_update_vectors`.
- **Search variants**: `search_vectors_by_text`, `search_by_file`,
  verify `hybrid_search` mapping.
- **Discovery pipeline (6 stages)**: `broad_discovery`, `semantic_focus`,
  `promote_readme`, `compress_evidence`, `build_answer_plan`,
  `render_llm_prompt`.
- **Admin/observability**: `get_stats`, `get_status`, `get_logs`,
  `get_indexing_progress`, `force_save_collection`,
  `list_empty_collections`, `cleanup_empty_collections`.
- **Auth/RBAC**: `me`, `logout`, `refresh_token`, full `/auth/keys/*`
  surface, `/auth/users/*` surface, `change_password`,
  `validate_password`.
- **Replication**: `get_replication_status`, `configure_replication`,
  `get_replication_stats`, `list_replicas`.
- **Backups/config**: `get_config`, `update_config` (admin),
  `list_backups`, `create_backup`, `restore_backup`, `restart_server`,
  `/hub/backups/*`, `/hub/usage/{statistics,quota}`,
  `/workspace/{add,remove,config}`.

This is purely additive. No new server endpoints, no behavior changes —
just typed SDK methods over routes the server already serves.

## What Changes

For each route in the gap inventory:

1. Rust SDK method in the right `sdks/rust/src/client/<domain>.rs`
   module (snake_case, returns typed `Result<T>`).
2. TypeScript SDK method in the matching
   `sdks/typescript/src/client/<domain>.ts` (camelCase, returns
   `Promise<T>`).
3. Python SDK method in `sdks/python/vectorizer/<domain>.py`
   (snake_case).
4. Shared model types added to each SDK's `models/` module.
5. Re-exports from the SDK roots so consumers can import without deep
   paths.

Workspace bump: SDKs 3.3 → 3.4 (minor, additive).

## Impact

- Affected specs: `.rulebook/tasks/phase12_sdk-control-surface-parity/specs/sdk-control-surface-parity/spec.md`
- Affected code:
  - `sdks/rust/src/client/{collections,vectors,search,discovery,files,core}.rs`
  - `sdks/rust/src/client/admin.rs` (new module for stats/status/logs/config/backups/workspace)
  - `sdks/rust/src/client/auth.rs` (new module for the auth surface)
  - `sdks/rust/src/client/replication.rs` (new module)
  - `sdks/typescript/src/client/*.ts` (mirror Rust modules)
  - `sdks/python/vectorizer/*.py` (mirror)
  - `sdks/{rust,typescript,python}/README.md`
  - `docs/api/` server reference (note SDK coverage)
- Breaking change: NO — additive only.
- User benefit: every server capability becomes typed-callable in all
  three SDKs. Eliminates the recurring "filed a task to expose route X"
  pattern.

## Constraints

- Wire contracts MUST match the existing server handlers byte-for-byte;
  no client-side reshaping.
- Auth/admin routes MUST surface 401/403 as typed errors, not panics.
- SDK method names MUST follow the existing convention in each language
  (Rust `snake_case`, TS `camelCase`, Python `snake_case`).
- No new server endpoints in this phase — gaps that need server work
  are deferred to phase13/phase14/phase15.

## Acceptance

- Every route listed in the gap inventory has a Rust + TS + Python SDK
  method.
- Each method has a unit test (request shape) and an integration test
  against a live server.
- SDK READMEs document the new surface.
- Workspace SDK version bumped 3.3 → 3.4.
