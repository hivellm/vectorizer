## 1. Implementation

- [ ] 1.1 Inventory all routes in `src/server/mod.rs` and `src/server/rest_handlers.rs`; produce a table in `docs/api/route-auth-matrix.md` listing each route + required auth bucket
- [ ] 1.2 Define three axum `Router` groups in `src/server/mod.rs`: `public`, `authenticated`, `admin`
- [ ] 1.3 Move `/setup/apply`, `/config` (POST), `/admin/restart`, `/workspace/add`, `/backups/restore` into the `admin` group
- [ ] 1.4 Wrap `authenticated` with `auth_middleware`; wrap `admin` with `require_auth_middleware` + role check for `admin`
- [ ] 1.5 Add `AuthContext` axum extractor; replace ad-hoc header parsing in handlers
- [ ] 1.6 Add a runtime sanity check at server boot: log the route table with their bucket; fail if any route is in no bucket

## 2. Tail (mandatory — enforced by rulebook v5.3.0)

- [ ] 2.1 Write `docs/api/route-auth-matrix.md` documenting every route, its method, its bucket, and the rationale
- [ ] 2.2 Add integration tests in `tests/api/rest/auth_enforcement.rs`: for each admin/authenticated route, assert 401 without token, 403 with viewer token, 2xx with admin token
- [ ] 2.3 Run `cargo test --all-features -- auth_enforcement` and confirm all tests pass

## Mandatory tail (required by rulebook v5.3.0)

- [ ] Update or create documentation covering the implementation
- [ ] Write tests covering the new behavior
- [ ] Run tests and confirm they pass
