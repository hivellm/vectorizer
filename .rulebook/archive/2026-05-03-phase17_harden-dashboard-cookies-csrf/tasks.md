## 1. Cookie attributes
- [x] 1.1 Build a `set_session_cookie(jar, token, exp, config)` helper that emits `HttpOnly; Secure; SameSite=Strict; Path=/; Max-Age=<exp>`
- [x] 1.2 Replace direct `Set-Cookie` writes in `auth_handlers/login.rs` and `auth_handlers/refresh.rs` with the helper
- [x] 1.3 Add `auth.cookies.insecure_dev: bool` to config (default false); helper drops `Secure` when set
- [x] 1.4 Boot fails with a clear error when `auth.cookies.insecure_dev=true` AND host is 0.0.0.0

## 2. CSRF tokens
- [x] 2.1 Generate a 32-byte random CSRF token at login; store in session record alongside the JWT
- [x] 2.2 Emit a non-HttpOnly cookie `XSRF-TOKEN=<token>; SameSite=Strict; Path=/` so the SPA can read it
- [x] 2.3 Add `require_csrf_middleware` that validates `X-CSRF-Token` header against the session's token on every POST/PUT/DELETE under `/auth/*` and `/admin/*`
- [x] 2.4 Wire middleware into the existing admin + auth route trees in `core/routing.rs`
- [x] 2.5 Update `dashboard/src/api/client.ts` to read the cookie and echo `X-CSRF-Token` on every mutating request

## 3. Tests
- [x] 3.1 Unit test: helper emits all four attributes when not in dev mode
- [x] 3.2 Unit test: helper omits `Secure` when `insecure_dev=true`
- [x] 3.3 Integration test: POST `/auth/users` without `X-CSRF-Token` returns 403
- [x] 3.4 Integration test: POST with valid `X-CSRF-Token` returns 200/201
- [x] 3.5 Boot test: `insecure_dev=true` + `--host 0.0.0.0` returns startup error

## 4. Tail (mandatory — enforced by rulebook v5.3.0)
- [x] 4.1 Update or create documentation covering the implementation
- [x] 4.2 Write tests covering the new behavior
- [x] 4.3 Run tests and confirm they pass
