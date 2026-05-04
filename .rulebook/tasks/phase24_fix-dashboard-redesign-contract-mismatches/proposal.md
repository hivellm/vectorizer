# Proposal: phase24_fix-dashboard-redesign-contract-mismatches

## Why

PR #266 (`feat(dashboard): console redesign — full visual migration`,
merged 2026-05-04 in commit `7bbb6477`) shipped a complete visual rewrite
of the dashboard onto a new console shell (ConsoleLayout / Sidebar /
Topbar / CommandPalette + primitives) but several pages reference
backend fields that do not exist on the v3.3.0 server, fail to echo the
CSRF token on mutating routes, and call backup endpoints with body
shapes that have not been verified against the route handler.

A static read-only audit of `dashboard/src/pages/*.tsx` against
`crates/vectorizer-server/src/server/core/routing.rs` and
`crates/vectorizer-server/src/server/auth_handlers/types.rs` found **5
BLOCKER** issues (the page is unusable) and **3 MAJOR** issues (the
column shows wrong/zero data) that need to be fixed before users can
exercise the new console.

### BLOCKERs

1. **Missing `X-CSRF-Token` echo on `POST /auth/keys`** — `ApiKeysPage.tsx`
   creates API keys without the header. Server's
   `require_csrf_middleware` (gates `/auth/*` mutations) returns
   `403 missing_csrf_token`. The shared `api-middleware.ts` already
   reads `XSRF-TOKEN` cookie and echoes the header, but the page may be
   bypassing the middleware on this call path. Verify and wire.
2. **Same CSRF gap on `POST /auth/users`** (`UsersPage.tsx`) — create user
   fails with 403.
3. **Same CSRF gap on `PUT /auth/users/{u}/password`** — change password
   fails with 403.
4. **Same CSRF gap on `DELETE /auth/users/{u}`** — delete user fails with
   403.
5. **Same CSRF gap on `POST /config`** (`ConfigurationPage.tsx`) — save
   configuration fails with 403.
6. **`POST /backups/create` and `/backups/restore`** (`BackupsPage.tsx`)
   — body shape unverified against the server handler; CSRF also missing.

(The shared `api-middleware.ts` does have CSRF wiring — items 1–5 may
collapse to one fix once we audit the call path. List them separately so
each page gets its own e2e regression.)

### MAJORs

7. **`ApiKeysPage` accesses `key.calls`** (line 28, 98, 168) but the
   server returns `usage_count: u64` and `usage_24h: u64`
   (`ApiKeyInfo`, `types.rs:75-98`). The "Calls (30d)" column will
   render 0 / undefined for every key.
8. **`ApiKeysPage` accesses `key.last_used_at`** (line 29, 169) but the
   server returns `last_used: Option<u64>` (unix seconds, not ISO
   string). The "Last used" column will render the raw number or
   `undefined`.
9. **`ApiKeysPage` accesses `key.masked` / `key.key_preview` /
   `key.key_prefix`** (line 60-65) — the server's `ApiKeyInfo` does NOT
   carry any key-material field. The actual API key value is only
   returned ONCE on `POST /auth/keys` in `CreateApiKeyResponse.api_key`.
   The "Key" column will always render `—`.

(All three MAJORs come from the redesigned `ApiKeysPage` having been
written against an imagined server schema, not the real one. The phase18
backend work that landed `usage_count` + `usage_24h` was never reflected
on the redesign branch — and our merge resolution to take the PR's
version of `ApiKeysPage.tsx` baked the schema mismatch in.)

## What Changes

Fix each issue at its source. The bulk of the work is in
`dashboard/src/pages/ApiKeysPage.tsx` (rewire the column accessors against
the real `ApiKeyInfo` shape) and verifying every page that calls a
mutating `/auth/*` or `/admin/*` endpoint actually goes through the
shared `api-middleware.ts` which already has CSRF echo wired. Pages that
bypass the middleware get rewired.

A separate but related concern: phase21 (CSRF Bearer JWT exemption) will
relax the CSRF gate for SDK callers using `Authorization: Bearer`. The
dashboard uses the cookie-session flow and is NOT affected by phase21 —
this task is purely "browser-side: send the CSRF token you already have."

## Impact

- Affected code:
  - `dashboard/src/pages/ApiKeysPage.tsx` — rewire columns to real
    server shape (`usage_count`, `usage_24h`, `last_used`, no key
    material on list)
  - `dashboard/src/pages/UsersPage.tsx` — verify CSRF echo on POST /
    PUT / DELETE under `/auth/users`
  - `dashboard/src/pages/BackupsPage.tsx` — verify endpoint shape +
    CSRF echo
  - `dashboard/src/pages/ConfigurationPage.tsx` — verify CSRF echo on
    POST /config
  - `dashboard/src/lib/api-middleware.ts` — confirm CSRF middleware is
    invoked on ALL mutations (not bypassed by direct fetch in any page)
  - `dashboard/e2e/` — add Playwright e2e covering the BLOCKER paths
- Affected specs:
  - phase18 ApiKey list response schema is the source of truth — this
    task adapts the redesigned UI to that schema
- Breaking change: NO (the redesigned dashboard is broken end-to-end at
  the moment; this task is a forward fix, not a behavior change)
- User benefit: Dashboard becomes actually usable on v3.3.0. API Keys
  page renders meaningful data; create/edit/delete user works; backup
  create/restore works; configuration save works.
