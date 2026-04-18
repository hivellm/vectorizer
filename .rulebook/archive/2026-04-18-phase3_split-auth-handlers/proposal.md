# Proposal: phase3_split-auth-handlers

## Why

`src/server/auth_handlers.rs` is **1,917 lines** and grew by ~170 during phase1 when the admin gate and `Secret<T>` plumbing landed. The file covers seven distinct concerns:

1. Request/response type definitions (top ~250 lines).
2. `AuthHandlerState` struct + impl.
3. Public handlers (login, validate-password).
4. Authenticated-user handlers (me, logout, refresh, api-keys).
5. Admin handlers (create/list/delete user, change password).
6. Middleware helpers (`auth_middleware`, `require_auth_middleware`, `require_admin_middleware`, `require_admin_for_rest`, `extract_auth_from_request`).
7. Tests.

See [docs/refactoring/oversized-files-audit.md](../../../docs/refactoring/oversized-files-audit.md).

## What Changes

Create `src/server/auth_handlers/` with one file per concern:

- `types.rs` — request/response structs.
- `state.rs` — `AuthHandlerState` + impls.
- `public.rs` — `/auth/login`, `/auth/validate-password`.
- `authenticated.rs` — me / logout / refresh / api-keys.
- `admin.rs` — user management.
- `middleware.rs` — the five middleware + extractor helpers.
- `tests.rs` — the existing test block.

Keep `src/server/auth_handlers.rs` as a `mod.rs` re-exporting the public surface; `pub use` every handler function name that `src/server/mod.rs` already registers so route wiring is unchanged.

## Impact

- Affected specs: none.
- Affected code: `src/server/auth_handlers.rs`, new `src/server/auth_handlers/*.rs`.
- Breaking change: NO.
- User benefit: each concern reviewable in isolation; next time someone adds an admin endpoint they touch ~200 lines, not ~2000.
