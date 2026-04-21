# Proposal: phase4_authcontext-extractor

## Why

After `phase1_protect-admin-setup-routes`, admin handlers accept a bare
`axum::http::HeaderMap` extractor and call `require_admin_for_rest`
inside the body. That is clear but it leaks the "extract credentials
from headers" detail into every handler. An idiomatic axum solution is
a typed extractor: handlers declare `admin: AdminAuth` and axum runs
the extractor, returning 401/403 automatically if the extractor fails.

## What Changes

1. Define `pub struct AdminAuth(pub AuthState)` in `src/server/auth_handlers.rs`.
2. Implement `FromRequestParts<S>` for `AdminAuth` where `S:
   FromRef<AuthHandlerState>` (or the unified `RouterState` from
   `phase4_router-layer-admin-middleware`, whichever lands first).
3. The extractor: pulls the Authorization header, validates it against
   the auth manager, checks `Role::Admin`, rejects with `(StatusCode,
   Json<AuthErrorResponse>)` on failure.
4. Migrate the 5 admin handlers from `headers: HeaderMap +
   require_admin_for_rest(...)` to `_admin: AdminAuth`.
5. Optionally add a companion `Authenticated(pub AuthState)` extractor
   for the authenticated bucket.

## Impact

- Affected specs: none.
- Affected code: `src/server/auth_handlers.rs`,
  `src/server/rest_handlers.rs`, `src/server/setup_handlers.rs`.
- Breaking change: NO (internal refactor, handler signatures change
  but routes stay identical).
- User benefit: cleaner handler signatures, reusable extractor for any
  future admin route, less boilerplate.
