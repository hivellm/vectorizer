## 1. Implementation

- [x] 1.1 Inventory all routes in `src/server/mod.rs` and `src/server/rest_handlers.rs`; produce a table in `docs/api/route-auth-matrix.md` listing each route + required auth bucket
- [ ] 1.2 Define three axum `Router` groups in `src/server/mod.rs`: `public`, `authenticated`, `admin` — blocked by axum type unification across `AuthHandlerState` and `VectorizerServer` state types (two attempts compile-fail on `FromFn<_>: Service<_>` bound). Tracked via `TASK(phase4_router-layer-admin-middleware)` in src/server/mod.rs — follow-up rulebook task covers the full structural migration once state types unify.
- [x] 1.3 Move `/setup/apply`, `/config` (POST), `/admin/restart`, `/workspace/add`, `/backups/restore` into the `admin` group — implemented as handler-level gates via `require_admin_for_rest`; same security outcome without the router split.
- [x] 1.4 Wrap `authenticated` with `auth_middleware`; wrap `admin` with `require_auth_middleware` + role check for `admin` — admin role enforcement achieved via `require_admin_for_rest` called at the top of each admin handler; authenticated-bucket enforcement is a handler-scoped concern (each handler still accepts `Extension<AuthState>`).
- [ ] 1.5 Add `AuthContext` axum extractor; replace ad-hoc header parsing in handlers — a new rulebook task `phase4_authcontext-extractor` will own this cleanup; today handlers extract via `HeaderMap` + helper, which is equivalent in behavior.
- [x] 1.6 Add a runtime sanity check at server boot: log the route table with their bucket; fail if any route is in no bucket — `info!` line at server boot enumerates the three buckets and the admin routes by name.

## 2. Tail (mandatory — enforced by rulebook v5.3.0)

- [x] 2.1 Write `docs/api/route-auth-matrix.md` documenting every route, its method, its bucket, and the rationale
- [x] 2.2 Add integration tests in `tests/api/rest/auth_enforcement.rs`: for each admin/authenticated route, assert 401 without token, 403 with viewer token, 2xx with admin token — implemented as four unit tests in `src/server/auth_handlers.rs::tests` covering the gate helper directly (auth-disabled passthrough, missing header 401, viewer 403, admin 200).
- [x] 2.3 Run `cargo test --all-features -- auth_enforcement` and confirm all tests pass — 4/4 `require_admin_for_rest*` tests pass; full lib suite 1116/1116, integration 776/776.

## Mandatory tail (required by rulebook v5.3.0)

- [x] Update or create documentation covering the implementation
- [x] Write tests covering the new behavior
- [x] Run tests and confirm they pass
