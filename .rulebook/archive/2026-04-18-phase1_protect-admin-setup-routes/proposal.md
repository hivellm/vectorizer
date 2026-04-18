# Proposal: phase1_protect-admin-setup-routes

## Why

Several administrative and setup routes are registered in `src/server/mod.rs:1449-1500` without the `require_auth_middleware` / RBAC check. Code comments note "may require auth" but enforcement is missing. An unauthenticated attacker can currently:

- `POST /setup/apply` — re-run the setup wizard and overwrite configuration
- `POST /config` — replace runtime configuration
- `POST /admin/restart` — force a service restart (DoS)
- `POST /workspace/add` — register arbitrary workspaces
- `POST /backups/restore` — roll data back to an attacker-chosen state (or exfiltrate via restore path)

Combined with the JWT default-secret issue (`phase1_fix-jwt-default-secret`), this is a full compromise: zero credentials needed.

The project already has `auth_middleware`, `require_auth_middleware`, and role-based helpers in `src/auth/` — they are simply not wired up on these routes.

## What Changes

1. Categorize every route in `src/server/mod.rs` into three buckets:
   - **Public** (`/health`, `/metrics` for unauth'd liveness, `/auth/login`): no auth.
   - **Authenticated** (`/collections`, `/vectors`, `/search`): require valid JWT.
   - **Admin** (`/admin/*`, `/setup/*`, `/config`, `/backups/*`, `/workspace/*`): require JWT + `admin` role.
2. Wrap each bucket with the appropriate `axum::middleware::from_fn` layer so no future route can be added without picking a bucket.
3. Add a startup assertion that every mounted route is in exactly one bucket (compile-time registry or runtime warning).
4. Add integration tests that call each admin route without a token and assert `401` / with a viewer token and assert `403`.

## Impact

- Affected specs: auth spec, API routes spec
- Affected code: `src/server/mod.rs`, `src/server/rest_handlers.rs` (handler signatures may need `AuthContext` extractor), `src/auth/middleware.rs`
- Breaking change: **YES** — unauthenticated setup/admin clients will break. Document migration.
- User benefit: closes a family of auth-bypass vulnerabilities; aligns with principle of least privilege.
