## 1. State unification

- [ ] 1.1 Introduce a combined `RouterState { server: VectorizerServer, auth_handler: AuthHandlerState }` in `src/server/` with `FromRef<RouterState>` impls for each inner state.
- [ ] 1.2 Update handler signatures to use `State<RouterState>` or rely on `FromRef` so no handler breaks.

## 2. Router split

- [ ] 2.1 Build `public_router`, `authenticated_router`, `admin_router` each with `Router<RouterState>`.
- [ ] 2.2 Apply `require_admin_middleware` via `from_fn_with_state` on `admin_router`.
- [ ] 2.3 Apply `auth_middleware` on `authenticated_router` so `Extension<AuthState>` is populated for every request.

## 3. Cleanup

- [ ] 3.1 Remove per-handler `require_admin_for_rest(...)` calls from the 5 admin handlers once the router layer enforces admin role.
- [ ] 3.2 Keep `require_admin_for_rest` in the codebase only for tests or for callers outside the router (if any).

## 4. Tests

- [ ] 4.1 Regression: the existing `require_admin_for_rest_*` unit tests keep passing (they exercise the helper directly).
- [ ] 4.2 New: add a route under `admin_router` without touching its handler; assert it 401s without a token, 403s with a viewer token.

## 5. Tail (mandatory)

- [ ] 5.1 Update `docs/api/route-auth-matrix.md` replacing the "handler-level gate" explanation with the new router-level enforcement diagram.
- [ ] 5.2 Tests above cover the new behavior.
- [ ] 5.3 Run `cargo test --all-features` and confirm pass.
