# Proposal: phase3_split-server-mod-monolith

> **Part of the oversized-files audit.** See
> [docs/refactoring/oversized-files-audit.md](../../../docs/refactoring/oversized-files-audit.md)
> for the full inventory and severity rubric. This task covers the
> `critical`-severity `server/mod.rs` entry.

## Why

`src/server/mod.rs` is **3,315 lines** combining:

- Server bootstrap and lifecycle
- Router construction (every route registered here)
- Shared `AppState`
- Middleware registration
- Startup banner / credential printing
- Hub / HiveHub integration
- 34 unwraps/expects in various places

After `phase3_split-rest-handlers-monolith` moves handler *implementations* out, this file still carries route *wiring* and state. Splitting it further is needed to:

- Make `phase1_protect-admin-setup-routes` clean — route buckets become their own sub-modules.
- Isolate startup side-effects (credential generation, cert loading) for unit testing.
- Drop unwraps in bootstrap paths by pushing each subsystem into its own fallible `init()` function.

## What Changes

Decompose `src/server/mod.rs`:

- `server/mod.rs` — thin orchestrator (<300 LOC)
- `server/state.rs` — `AppState` struct + builders
- `server/routes/mod.rs` — composition of route buckets
- `server/routes/public.rs` — `/health`, `/metrics`, `/auth/login`, etc.
- `server/routes/authenticated.rs` — protected data routes
- `server/routes/admin.rs` — admin/setup/config
- `server/routes/mcp.rs` — MCP websocket route
- `server/bootstrap.rs` — startup sequence (config load → validate → init subsystems → bind listener)
- `server/shutdown.rs` — graceful shutdown, drain, snapshot

Files ≤400 LOC each. Clear separation between "what routes exist", "what state they need", and "how the server boots".

## Impact

- Affected specs: none
- Affected code: `src/server/mod.rs`, new `src/server/*` submodules, `src/bin/vectorizer-cli.rs` (entrypoint unchanged signature)
- Breaking change: NO
- User benefit: unblocks auth-bucket enforcement, reduces unwrap surface, makes boot sequence testable.
