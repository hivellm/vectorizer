# Proposal: phase21_fix-csrf-bearer-jwt-exemption

## Why

The CSRF middleware shipped in phase17
(`crates/vectorizer-server/src/server/auth_handlers/csrf.rs`) blocks every
mutating request under `/auth/*` and `/admin/*` that does not carry an
`X-CSRF-Token` header bound to a server-side session. The exemption logic
exempts `X-API-Key` requests on the grounds that "header-bearer credentials
are not subject to the cross-origin attack the CSRF token defends against"
but applies the same gate to `Authorization: Bearer <jwt>` calls — even
though Bearer is also a header-bearer credential explicitly attached by
the client, not a cookie auto-attached by the browser.

Verified end-to-end against a live `vectorizer:3.3.0` container on
2026-05-04 by logging in via `POST /auth/login`, capturing the access
token, and calling every phase 12 / phase 15 mutating auth admin route
with `Authorization: Bearer <token>` and no `X-CSRF-Token` header. Every
single one returned `403 missing_csrf_token`:

- `POST /auth/keys` (CreateApiKey)
- `POST /auth/keys/{id}/rotate` (RotateApiKey)
- `POST /auth/introspect` (IntrospectToken)
- `PUT /auth/keys/{id}/permissions` (UpdateApiKeyPermissions)
- `DELETE /auth/keys/{id}` (RevokeApiKey)
- `POST /auth/users` / `DELETE /auth/users/{u}` / `PUT /auth/users/{u}/password`

This breaks all five first-party SDKs (Rust, TypeScript, Python, Go, C#)
because the SDKs use `Authorization: Bearer` (or `X-API-Key`) and have no
concept of a session cookie + CSRF token pair — those are dashboard /
browser concerns. The phase20 Go and C# REST parity wire-shape tests were
green only because they ran against a hermetic mock; against the real
server every auth admin call fails.

## What Changes

Extend the CSRF exemption in
`crates/vectorizer-server/src/server/auth_handlers/csrf.rs::require_csrf_middleware`
so that requests carrying a Bearer JWT in the `Authorization` header AND
no `vectorizer_session` cookie bypass the CSRF check, mirroring the
existing `X-API-Key` exemption. The cookie-based dashboard flow is
unaffected: when `vectorizer_session` is present, CSRF is still required.

Detection rule (inside the middleware):

1. If `X-API-Key` header is set → bypass (existing behavior).
2. **NEW**: If `Authorization: Bearer <token>` header is set AND no
   `vectorizer_session` cookie is set → bypass.
3. If `vectorizer_session` cookie is set → require `X-CSRF-Token`.
4. Otherwise → 403 `missing_session`.

This preserves the CSRF defense for the browser flow (which always sets
the cookie via `Set-Cookie` after login) and unblocks SDK / cURL /
service-to-service callers that explicitly attach a Bearer token.

## Impact

- Affected code:
  - `crates/vectorizer-server/src/server/auth_handlers/csrf.rs` —
    extend `require_csrf_middleware` exemption logic
  - `crates/vectorizer-server/tests/csrf_bearer_exemption.rs` (new) —
    regression test that asserts a `Bearer`-only call to `POST /auth/keys`
    succeeds without `X-CSRF-Token`, and that a cookie-bearing call without
    `X-CSRF-Token` is still rejected
  - `docs/users/api/AUTHENTICATION.md` — document the exemption table
- Breaking change: NO (additive — relaxes a too-strict gate; cookie flow
  unchanged)
- User benefit: All 5 SDKs' auth admin methods become usable against
  CSRF-enabled servers without the SDK needing to capture and echo the
  CSRF token. Restores the SDK contract phase20 promised.
