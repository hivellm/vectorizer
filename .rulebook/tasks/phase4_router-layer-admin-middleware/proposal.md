# Proposal: phase4_router-layer-admin-middleware

## Why

`phase1_protect-admin-setup-routes` closed the admin auth gap via a
handler-level gate (`require_admin_for_rest` called at the top of each
admin handler). The ORIGINAL design in the proposal wanted a cleaner
router-level split with three axum `Router` groups (public /
authenticated / admin) each wrapped with the appropriate middleware
layer. That failed to compile because axum's `from_fn_with_state` layer
could not unify the two state types the admin bucket spans —
`AuthHandlerState` (for `auth_handlers::*`) and `VectorizerServer` (for
`rest_handlers::*` + `setup_handlers::*`) — after `.with_state(...)` had
been applied. Two attempts produced:

```
the trait `tonic::codegen::Service<Request<Body>>` is not implemented
for `FromFn<..., ..., _>`
```

The handler-level gate is correct and tested, but it leaves the admin
rule duplicated in every handler rather than asserted structurally at
the router boundary. This task migrates to the structural form once the
state types are unified.

## What Changes

1. Refactor `VectorizerServer` and `AuthHandlerState` so they share a
   single state type (e.g. wrap both in a `RouterState { server,
   auth_handler }` struct that implements `FromRef` for both).
2. Extract admin routes into a dedicated `admin_router: Router<()>`.
3. Apply `require_admin_middleware` via
   `from_fn_with_state(auth_state, require_admin_middleware)` on the
   grouped admin router.
4. Delete the per-handler `require_admin_for_rest` calls from the 5+
   admin handlers — the router layer is now authoritative.
5. Keep the regression tests; add one that asserts a new route registered
   under the admin group gets 401/403 without needing per-handler changes.

## Impact

- Affected specs: none (internal refactor).
- Affected code: `src/server/mod.rs`, `src/server/auth_handlers.rs`,
  `src/server/rest_handlers.rs`, `src/server/setup_handlers.rs`.
- Breaking change: NO — external behavior is identical; only the
  enforcement point moves.
- User benefit: structural guarantee that any future admin route is
  protected by virtue of registration, not by reviewer vigilance.
