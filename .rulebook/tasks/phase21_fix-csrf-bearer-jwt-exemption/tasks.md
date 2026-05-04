## 1. Middleware fix

- [ ] 1.1 Extend `require_csrf_middleware` in `crates/vectorizer-server/src/server/auth_handlers/csrf.rs` to bypass CSRF when `Authorization: Bearer` is set and no `vectorizer_session` cookie is present
- [ ] 1.2 Keep the existing `X-API-Key` bypass branch first; the new branch checks `Authorization` header AFTER and only when no session cookie exists
- [ ] 1.3 Update the doc comment on `require_csrf_middleware` to enumerate all exemption rules (X-API-Key, Bearer-without-cookie, GET/HEAD/OPTIONS, CSRF_EXEMPT_PATHS)

## 2. Regression tests

- [ ] 2.1 Add `crates/vectorizer-server/tests/csrf_bearer_exemption.rs` integration test that logs in, drops both `Cookie` and `X-CSRF-Token` headers, calls `POST /auth/keys` with only `Authorization: Bearer <jwt>`, and asserts HTTP 200 + a parsed `ApiKey` response
- [ ] 2.2 In the same file, add a regression test that keeps the `vectorizer_session` cookie, sends it WITHOUT `X-CSRF-Token`, and asserts HTTP 403 `missing_csrf_token` (proves cookie path still gated)
- [ ] 2.3 Add unit test in `csrf.rs` for the exemption predicate covering every branch (X-API-Key, Bearer+no-cookie, cookie+no-csrf, cookie+csrf)

## 3. Tail (mandatory — enforced by rulebook v5.3.0)

- [ ] 3.1 Update `docs/users/api/AUTHENTICATION.md` (or `docs/users/api/CSRF.md` if it exists) with a "When CSRF is required vs exempt" table
- [ ] 3.2 Run `cargo test -p vectorizer-server --test csrf_bearer_exemption` and confirm pass
- [ ] 3.3 Run `cargo clippy -p vectorizer-server -- -D warnings` and confirm zero warnings
