## 1. Config + struct
- [x] 1.1 Add `dev_mode_skip_loopback: bool` to `AuthConfig` (serde-default false)
- [x] 1.2 No additional `validate()` change required — the boot guard rejects misconfiguration before the listener opens

## 2. Middleware short-circuit
- [x] 2.1 In `auth_handlers/middleware.rs`, when the flag is on, build an implicit `local-dev-admin` `UserClaims` (Role::Admin, empty scopes) and inject as Extension via the new `local_dev_admin_state` helper; wired into `auth_middleware`, `require_auth_middleware`, `require_admin_middleware`, and `require_admin_from_headers`
- [x] 2.2 Emit `X-Vectorizer-Dev-Mode: true` response header on every request handled in this mode
- [x] 2.3 CSRF middleware short-circuits in dev mode so mutating routes are reachable without a session token
- [x] 2.4 Unit tests: middleware short-circuits + adds the header (GET + POST + admin paths)

## 3. Boot guard
- [x] 3.1 In `core/routing.rs::start`, before binding, refuse boot when `dev_mode_skip_loopback=true` AND host ∉ {`127.0.0.1`, `::1`, `localhost`}
- [x] 3.2 Log a multi-line WARN banner when the flag is engaged on a loopback bind
- [x] 3.3 Loopback-host predicate centralised in `cookies::is_loopback_host` so the cookie boot guard and the dev-mode boot guard apply the same definition; unit-tested

## 4. Integration
- [x] 4.1 Router-level test: with the flag on, anon GET on a `require_auth_middleware`-gated route returns 200 with the dev-mode header
- [x] 4.2 Router-level test: with the flag off (default), the same call returns 401

## 5. Documentation
- [x] 5.1 New "Local Development" section in `docs/users/api/AUTHENTICATION.md` covering config, behaviour, boot guard, and a quick try-it

## 6. Tail (mandatory — enforced by rulebook v5.3.0)
- [x] 6.1 Update or create documentation covering the implementation
- [x] 6.2 Write tests covering the new behavior
- [x] 6.3 Run tests and confirm they pass
