# Proposal: phase8_gate-data-routes-when-auth-enabled

## Why

When `auth.enabled: true` in `config.yml`, the server:

1. Mounts `/auth/login`, `/auth/me`, `/auth/keys/*`, etc.
2. Auto-creates a `root` user with a random password written to
   `%APPDATA%\vectorizer\.root_credentials`.
3. Issues valid JWTs from `/auth/login`.
4. Applies `require_admin_middleware` to the admin router
   (`/workspace/*`, `/setup/*`, `/admin/restart`, `/backups/*`).
5. Does NOT apply any auth middleware to the data routes —
   `GET /collections`, `POST /insert`, `POST /batch_insert`,
   `POST /search`, `POST /embed`, `POST /collections/{name}/vectors*`,
   `DELETE /collections/{name}`, and every other REST data surface.

Empirical repro on 2026-04-20 during probe 3.1 of
`phase8_release-v3-runtime-verification`:

```
$ VECTORIZER_JWT_SECRET=<hex> ./target/release/vectorizer --host 127.0.0.1 &
$ curl -s -o /dev/null -w "%{http_code}\n" http://127.0.0.1:15002/collections
200                     # unauthenticated, auth.enabled: true
$ curl -s -X POST http://127.0.0.1:15002/auth/login -d '{"username":"root","password":"<pw>"}'
{"access_token":"<jwt>", ...}
$ curl -s http://127.0.0.1:15002/collections -H "Authorization: Bearer <jwt>"
{"collections":[...]}   # authenticated, same result
```

Probe 3.1 acceptance requires "unauthenticated calls return 401" —
they currently return 200. Enabling auth is cosmetic for data-surface
callers: issuing a JWT gates nothing.

The existing `require_auth_middleware` in
`crates/vectorizer-server/src/server/auth_handlers/middleware.rs:49`
is the right tool; it is wired up for admin routes but never mounted
on the data routes merged at
`crates/vectorizer-server/src/server/core/routing.rs:696-700`.

Source: `docs/releases/v3.0.0-verification.md` finding F7.

## What Changes

Apply `require_auth_middleware` at the router level whenever
`self.auth_handler_state.is_some()`. Structure:

1. Build the data-routes `Router` as today, then conditionally wrap:

   ```rust
   let rest_routes = if let Some(auth_state) = self.auth_handler_state.clone() {
       rest_routes
           .merge(protected_auth_router)
           .layer(axum::middleware::from_fn_with_state(
               auth_state,
               crate::server::auth_handlers::require_auth_middleware,
           ))
           .merge(public_auth_router)
   } else {
       rest_routes
   };
   ```

   The `public_auth_router` (which carries `/auth/login` and
   `/auth/validate-password`) must be merged AFTER the `.layer(...)`
   so it stays anonymous-accessible. Same for `/health`,
   `/prometheus/metrics`, the dashboard SPA, and the UMICP discovery
   endpoint.

2. **Public route inventory** — the contract this fix locks in:
   - `GET /health`
   - `GET /prometheus/metrics`
   - `POST /auth/login`
   - `POST /auth/validate-password`
   - `GET /umicp/discover`
   - `GET /umicp/health`
   - `GET /dashboard/**` (React SPA shell)
   - Static assets under `/dashboard/assets/**`

   Everything else (data, admin, setup) requires `authenticated: true`
   when auth is enabled.

3. Backwards-compat — when `auth.enabled: false` (default for
   single-user local setups), no middleware attaches and every route
   stays anonymous, matching today's behavior.

4. Document the contract in a new `docs/users/api/AUTH.md` (or update
   `docs/users/api/API_REFERENCE.md` > Authentication section).

## Impact

- Affected specs: `docs/users/api/API_REFERENCE.md#authentication`,
  `CHANGELOG.md#3.0.0 > Security`.
- Affected code:
  - `crates/vectorizer-server/src/server/core/routing.rs:696-700`
    (apply the layer; re-order merges so public routes stay open)
  - possibly `src/server/auth_handlers/mod.rs` (re-export
    `require_auth_middleware` if not already `pub`)
- Breaking change: YES for operators who enabled auth but relied on
  the unauthenticated-data-access drift (v3 dev builds) — document
  under `3.0.0 > Security` as a corrected enforcement gap.
- User benefit: when an operator sets `auth.enabled: true`, enabling
  auth actually gates the data surface. Closes F7 + unblocks probe
  3.1 of `phase8_release-v3-runtime-verification`.
