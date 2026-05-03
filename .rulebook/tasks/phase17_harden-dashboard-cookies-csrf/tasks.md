## 1. Cookie attributes
- [ ] 1.1 Build a `set_session_cookie(jar, token, exp, config)` helper that emits `HttpOnly; Secure; SameSite=Strict; Path=/; Max-Age=<exp>`
- [ ] 1.2 Replace direct `Set-Cookie` writes in `auth_handlers/login.rs` and `auth_handlers/refresh.rs` with the helper
- [ ] 1.3 Add `auth.cookies.insecure_dev: bool` to config (default false); helper drops `Secure` when set
- [ ] 1.4 Boot fails with a clear error when `auth.cookies.insecure_dev=true` AND host is 0.0.0.0

## 2. CSRF tokens
- [ ] 2.1 Generate a 32-byte random CSRF token at login; store in session record alongside the JWT
- [ ] 2.2 Emit a non-HttpOnly cookie `XSRF-TOKEN=<token>; SameSite=Strict; Path=/` so the SPA can read it
- [ ] 2.3 Add `require_csrf_middleware` that validates `X-CSRF-Token` header against the session's token on every POST/PUT/DELETE under `/auth/*` and `/admin/*`
- [ ] 2.4 Wire middleware into the existing admin + auth route trees in `core/routing.rs`
- [ ] 2.5 Update `dashboard/src/api/client.ts` to read the cookie and echo `X-CSRF-Token` on every mutating request

## 3. Tests
- [ ] 3.1 Unit test: helper emits all four attributes when not in dev mode
- [ ] 3.2 Unit test: helper omits `Secure` when `insecure_dev=true`
- [ ] 3.3 Integration test: POST `/auth/users` without `X-CSRF-Token` returns 403
- [ ] 3.4 Integration test: POST with valid `X-CSRF-Token` returns 200/201
- [ ] 3.5 Boot test: `insecure_dev=true` + `--host 0.0.0.0` returns startup error

## 4. Tail (mandatory — enforced by rulebook v5.3.0)
- [ ] 4.1 Update or create documentation covering the implementation
- [ ] 4.2 Write tests covering the new behavior
- [ ] 4.3 Run tests and confirm they pass
