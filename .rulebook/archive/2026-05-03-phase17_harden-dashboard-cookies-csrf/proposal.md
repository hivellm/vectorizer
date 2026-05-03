# Proposal: phase17_harden-dashboard-cookies-csrf

Source: phase8 audit follow-up (sections 1.9 + 6.5-6.6).

## Why

The dashboard auth flow ships JWT and session credentials over HTTP cookies
that are NOT marked `HttpOnly`, `Secure`, or `SameSite=Strict`. That leaves
the session vulnerable to:

1. **XSS exfiltration** — any JS injection in the dashboard can read the
   token via `document.cookie`.
2. **CSRF** — POST/PUT/DELETE routes have no CSRF token; a malicious
   third-party page can fire authenticated cross-origin writes once the
   user is logged in.
3. **Cookie-leak in transit** — without `Secure`, the cookie travels over
   plain HTTP if the operator misconfigures TLS termination.

Phase8 enumerated these gaps (1.9 + 6.5-6.6) but they shipped unaddressed
because the auth-bringup work prioritized the routing surface (login,
users, keys) over the cookie hardening. Today the dashboard is reachable
on production binds (0.0.0.0) — the same code path that enforces
authentication should also enforce cookie hygiene.

## What Changes

### 1. Cookie attributes

Every Set-Cookie emitted by `/auth/login` and `/auth/refresh` MUST carry:

- `HttpOnly` — JS cannot read the cookie
- `Secure` — only sent over HTTPS (with a documented dev-mode escape for
  127.0.0.1 plain HTTP)
- `SameSite=Strict` — never sent on cross-site requests
- `Path=/` and `Max-Age` aligned with the JWT `exp`

### 2. CSRF tokens

For the dashboard's authenticated state-changing routes (every POST/PUT/
DELETE under `/auth/*` and `/admin/*`), require a CSRF header:

- Server emits a per-session CSRF token at login time; stored in a
  separate non-HttpOnly cookie so the SPA can read it
- SPA echoes the token in the `X-CSRF-Token` header on every mutating
  request
- Server middleware validates the header against the session's token
  and rejects with HTTP 403 on mismatch

### 3. Dev-mode escape

A config flag `auth.cookies.insecure_dev` (default `false`) drops the
`Secure` flag for local development over plain HTTP. Logged at WARN on
boot when enabled. Refused when binding to 0.0.0.0.

## Impact

- Affected specs: `.rulebook/tasks/phase17_harden-dashboard-cookies-csrf/specs/dashboard-auth-hardening/spec.md`
- Affected code:
  - `crates/vectorizer-server/src/server/auth_handlers/login.rs`
  - `crates/vectorizer-server/src/server/auth_handlers/middleware.rs`
  - `dashboard/src/api/client.ts` (CSRF header echo)
- Breaking change: NO for API consumers (RPC + REST API key flow
  unaffected); MINIMAL for dashboard (SPA must re-login once after
  deploy to obtain new cookie set)
- User benefit: closes the XSS + CSRF + plain-HTTP gaps phase8 left open

## Acceptance

- Every dashboard auth cookie carries `HttpOnly; Secure; SameSite=Strict`
- A POST to `/auth/users` without `X-CSRF-Token` returns 403
- Boot rejects `auth.cookies.insecure_dev=true` when binding to 0.0.0.0
- Integration tests cover happy path + each rejection
