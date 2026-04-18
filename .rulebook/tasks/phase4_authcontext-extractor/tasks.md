## 1. Extractor

- [ ] 1.1 Define `pub struct AdminAuth(pub AuthState)` in `src/server/auth_handlers.rs`.
- [ ] 1.2 Implement `FromRequestParts<S>` where `S: FromRef<AuthHandlerState> + Send + Sync`.
- [ ] 1.3 On missing token: return `(StatusCode::UNAUTHORIZED, Json(AuthErrorResponse))`. On non-admin token: return `(StatusCode::FORBIDDEN, ...)`.

## 2. Companion

- [ ] 2.1 Define `pub struct Authenticated(pub AuthState)` for the authenticated bucket (any logged-in user).
- [ ] 2.2 Same `FromRequestParts` pattern, rejects only when the token is absent/invalid.

## 3. Migration

- [ ] 3.1 Replace `headers: HeaderMap + require_admin_for_rest(...)` with `_admin: AdminAuth` in the 5 admin handlers (`apply_setup_config`, `update_config`, `restart_server`, `add_workspace`, `restore_backup`).
- [ ] 3.2 Drop the `HeaderMap` parameter once every caller moves to the extractor.

## 4. Tests

- [ ] 4.1 Direct extractor test: hand-craft a `Parts` with/without an admin token; assert the extractor yields the right result.
- [ ] 4.2 End-to-end: one admin route with the new extractor; assert 401 / 403 / 200 flow.

## 5. Tail (mandatory)

- [ ] 5.1 Update `docs/api/route-auth-matrix.md` to reference the extractor pattern instead of the helper-call pattern.
- [ ] 5.2 Tests above cover the new behavior.
- [ ] 5.3 Run `cargo test --all-features` and confirm pass.
