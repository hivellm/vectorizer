## 1. Extractor

- [x] 1.1 Defined `pub struct AdminAuth(pub Option<AuthState>)` in `src/server/auth_handlers/extractors.rs`.
- [x] 1.2 Implemented `FromRequestParts<VectorizerServer>` (the actual handler state) backed by an internal `extract_admin(Option<&AuthHandlerState>, &HeaderMap)` helper so unit tests don't need a full server.
- [x] 1.3 On missing token: returns `ErrorResponse` with `StatusCode::UNAUTHORIZED`. On non-admin: returns `ErrorResponse` with `StatusCode::FORBIDDEN`. Auth-globally-disabled: yields `AdminAuth(None)` preserving legacy no-auth semantics.

## 2. Companion

- [x] 2.1 Defined `pub struct Authenticated(pub Option<AuthState>)` for the authenticated bucket.
- [x] 2.2 Same `FromRequestParts` pattern, rejects only when token is absent or invalid.

## 3. Migration

- [x] 3.1 Migrated all 5 admin handlers from `headers: HeaderMap + require_admin_for_rest(...)` to `_admin: AdminAuth`: `apply_setup_config`, `update_config`, `restart_server`, `add_workspace`, `restore_backup`.
- [x] 3.2 Dropped the `HeaderMap` parameter from all 5 handlers.

## 4. Tests

- [x] 4.1 Direct extractor tests for both `AdminAuth` and `Authenticated` covering: auth-globally-disabled, missing token (401), non-admin token (403 for AdminAuth; pass for Authenticated), admin token (pass). Total 8 new tests.
- [x] 4.2 Existing `require_admin_for_rest_*` tests kept green for backward compat - helper is still exported for any non-axum caller.

## 5. Tail (mandatory)

- [x] 5.1 Updated `docs/api/route-auth-matrix.md`: admin bucket row points to `AdminAuth`, added an "Extractor pattern" section with an example signature, Testing section points to the new tests. The 5 admin routes in the routes-by-bucket table now reference the extractor instead of the helper call.
- [x] 5.2 Tests above cover the new behavior (8 extractor tests + 4 existing helper tests).
- [x] 5.3 `cargo test --lib --all-features`: 1158/1158 pass; `cargo clippy --lib --all-features -- -D warnings`: 0 warnings; `cargo fmt`: clean.
