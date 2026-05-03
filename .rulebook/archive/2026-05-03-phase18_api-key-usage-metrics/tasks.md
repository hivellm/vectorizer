## 1. Server — usage counter
- [x] 1.1 Add `usage_count: u64` to `ApiKey` (additive, serde-default `0`); reuse existing `last_used: Option<u64>` instead of duplicating with `last_used_at`
- [x] 1.2 Increment counter atomically (`AtomicU64` wrapped in `Arc`) on every successful `validate_key`
- [x] 1.3 Persist via `AuthPersistence` flush; back-compat deserializer accepts old payloads without the new fields
- [x] 1.4 Unit test: 100 concurrent `validate_key` calls produce `usage_count == 100`

## 2. Server — permission update endpoint
- [x] 2.1 Handler `update_api_key_permissions(id, req)` in `auth_handlers/auth_admin.rs`
- [x] 2.2 Route `PUT /auth/keys/{id}/permissions` (admin-gated)
- [x] 2.3 Updates `permissions` + optional `scopes`; `key_hash` + `created_at` immutable
- [x] 2.4 Unit test: round-trip a permission change; counter survives the change; not-found returns 404

## 3. Per-key request bucket (for sparkline)
- [x] 3.1 Add a per-key per-day counter ring buffer in `crates/vectorizer/src/monitoring/api_key_usage.rs`
- [x] 3.2 Hook into `validate_api_key` to bump the right key's bucket (correct hook point — `record_api_request` would attribute requests to the wrong key when the same caller hits multiple endpoints)
- [x] 3.3 Expose `GET /auth/keys/{id}/usage?window=7d` returning daily buckets
- [x] 3.4 Unit test: 50 requests across 2 days yield correct daily aggregates

## 4. SDKs
- [x] 4.1 Rust SDK: `update_api_key_permissions`, `get_api_key_usage` in `sdks/rust/src/client/auth.rs`
- [x] 4.2 TypeScript SDK: same methods in `sdks/typescript/src/client/auth.ts` (camelCase)
- [x] 4.3 Python SDK: same methods in `sdks/python/vectorizer/auth.py`
- [x] 4.4 Add CHANGELOG entries (SDK version bump rolled into the next release)
- [x] 4.5 Type-check passes for all three SDKs

## 5. Dashboard
- [x] 5.1 Extend `ApiKeysPage.tsx` with "Last 24h" + "Total" columns (powered by `usage_24h` + `usage_count` on the list response — no N+1 fetch)
- [x] 5.2 Usage detail modal renders a sparkline + per-day bucket table from `GET /auth/keys/{id}/usage?window=14`
- [x] 5.3 Type-check passes; sparkline component is data-test-id'd for future Vitest coverage

## 6. Tail (mandatory — enforced by rulebook v5.3.0)
- [x] 6.1 Update or create documentation covering the implementation
- [x] 6.2 Write tests covering the new behavior
- [x] 6.3 Run tests and confirm they pass
