## 1. Implementation

- [x] 1.1 Apply `require_auth_middleware` as a router layer on
  `rest_routes` in
  `crates/vectorizer-server/src/server/core/routing.rs` when
  `self.auth_handler_state.is_some()`, re-ordering merges so public
  routes (`/auth/login`, `/health`, `/prometheus/metrics`,
  `/umicp/discover`, `/umicp/health`, dashboard SPA) stay anonymous.
- [x] 1.2 Move `/auth/validate-password` into the public bucket
  (already routes through `public_auth_router` — confirm it is merged
  after the auth layer).
- [x] 1.3 Verify the admin router still works under the new layer
  stack (admin routes must still 401 unauthenticated, 403 without
  `Role::Admin`).
- [x] 1.4 Add a startup log line at INFO when auth enforcement is
  active so operators see the mode in the banner.
- [x] 1.5 Confirm the dashboard SPA's own login flow still fetches
  `/auth/login` unauthenticated and stores the JWT for subsequent
  calls.

## 2. Tail (mandatory — enforced by rulebook v5.3.0)

- [x] 2.1 Update or create documentation covering the implementation
  (new `docs/users/api/AUTH.md` or extend
  `docs/users/api/API_REFERENCE.md#authentication`; CHANGELOG entry
  under `3.0.0 > Security`).
- [x] 2.2 Write tests covering the new behavior (integration test at
  `crates/vectorizer/tests/api/rest/auth_enforcement_real.rs` that
  boots the server with `VECTORIZER_JWT_SECRET` set, asserts
  `/collections` returns 401 unauthenticated, 200 with a valid JWT,
  and that the public routes (`/health`, `/prometheus/metrics`,
  `/auth/login`, `/umicp/discover`, `/dashboard/`) still answer
  anonymously).
- [x] 2.3 Run tests and confirm they pass
  (`cargo test --workspace --lib --all-features` plus the new
  integration test).
