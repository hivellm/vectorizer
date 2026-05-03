# Proposal: phase18_api-key-usage-metrics

Source: phase8 audit follow-up (sections 9A.6 + 9A.11 + 9B.8).

## Why

Today an API key has a single `last_used` timestamp and an `active`
boolean. There is no:

1. **Usage counter** — operators cannot tell which keys are hot vs
   dormant, can't bill per-key, can't size rate limits per key class.
2. **Permission update endpoint** — to change a key's permissions the
   operator must revoke + recreate, which forces every consumer of that
   key to re-deploy with a new credential.
3. **Dashboard usage display** — the ApiKeysPage shows a key's name,
   prefix, and creation date; it has no panel for "X requests in the
   last 24 h" so operators can't spot anomalous usage.

These gaps were enumerated in phase8 (9A.6 + 9A.11 + 9B.8) but skipped
because the auth-bringup focus was on the create/list/revoke surface.

## What Changes

### 1. Server — usage counter

- Add `usage_count: u64` and `last_used_at: Option<i64>` to the `ApiKey`
  struct (additive, serde-default `0` / `None`).
- Increment `usage_count` and refresh `last_used_at` atomically on every
  successful `validate_key` call.
- Persist via the existing `AuthPersistence` flush.

### 2. Server — permission update

- New handler `PUT /auth/keys/{id}/permissions` accepting
  `{permissions: [...], scopes?: [...]}`.
- Updates only the listed fields; `key_hash` and `created_at` are
  immutable.
- Admin-gated.

### 3. SDK + Dashboard

- Rust + TS + Python SDK methods: `update_api_key_permissions(id, req)`.
- Dashboard `ApiKeysPage`: add a "Last 24h" + "Total" column; clicking
  a key opens a side panel with a sparkline of the per-day counter
  (recorded by hooking the existing `metrics::record_api_request` into
  a per-key bucket).

## Impact

- Affected specs: `.rulebook/tasks/phase18_api-key-usage-metrics/specs/api-key-usage-metrics/spec.md`
- Affected code:
  - `crates/vectorizer/src/auth/api_keys.rs` (counter + struct fields)
  - `crates/vectorizer/src/auth/persistence.rs` (serde back-compat)
  - `crates/vectorizer-server/src/server/auth_handlers/keys.rs` (new
    handler + counter increment hook)
  - `sdks/{rust,typescript,python}/.../auth.{rs,ts,py}`
  - `dashboard/src/pages/ApiKeysPage.tsx`
- Breaking change: NO — additive (existing keys deserialize with `usage_count: 0`)
- User benefit: operators can size rate limits, spot anomalous usage,
  and rotate permissions without revoking the key

## Acceptance

- Every successful `validate_key` increments `usage_count` exactly once
- `PUT /auth/keys/{id}/permissions` returns the updated `ApiKey`
- SDK methods land in all three SDKs with version bump
- Dashboard renders the counter columns + sparkline
