## 1. Config + struct
- [ ] 1.1 Add `dev_mode_skip_loopback: bool` to `AuthConfig` (serde-default false)
- [ ] 1.2 Update `AuthConfig::validate` to accept the new field

## 2. Middleware short-circuit
- [ ] 2.1 In `auth_handlers/middleware.rs`, when the flag is on, build an implicit `local-dev-admin` `UserClaims` (Role::Admin, empty scopes) and inject as Extension
- [ ] 2.2 Emit `X-Vectorizer-Dev-Mode: true` response header on every request handled in this mode
- [ ] 2.3 Unit test: middleware short-circuits and adds the header when flag is on

## 3. Boot guard
- [ ] 3.1 In `core/routing.rs::start`, before binding, refuse boot when `dev_mode_skip_loopback=true` AND host ∉ {`127.0.0.1`, `::1`, `localhost`}
- [ ] 3.2 Log a multi-line WARN banner when the flag is engaged on a loopback bind
- [ ] 3.3 Boot test: `dev_mode_skip_loopback=true` + `--host 0.0.0.0` returns startup error
- [ ] 3.4 Boot test: `dev_mode_skip_loopback=true` + `--host 127.0.0.1` boots successfully and emits the WARN banner

## 4. Integration
- [ ] 4.1 Integration test: with the flag on, a `GET /collections` without any token returns 200
- [ ] 4.2 Integration test: with the flag off (default), the same call returns 401

## 5. Documentation
- [ ] 5.1 Add a "Local development" subsection to `docs/users/api/AUTH.md` documenting the flag + the loopback constraint

## 6. Tail (mandatory — enforced by rulebook v5.3.0)
- [ ] 6.1 Update or create documentation covering the implementation
- [ ] 6.2 Write tests covering the new behavior
- [ ] 6.3 Run tests and confirm they pass
