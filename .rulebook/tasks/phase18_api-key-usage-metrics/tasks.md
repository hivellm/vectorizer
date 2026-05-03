## 1. Server — usage counter
- [ ] 1.1 Add `usage_count: u64` and `last_used_at: Option<i64>` to `ApiKey` (additive, serde-default `0` / `None`)
- [ ] 1.2 Increment counter atomically (`AtomicU64` wrapped in `Arc`) on every successful `validate_key`
- [ ] 1.3 Persist via `AuthPersistence` flush; back-compat deserializer accepts old payloads without the new fields
- [ ] 1.4 Unit test: 100 concurrent `validate_key` calls produce `usage_count == 100`

## 2. Server — permission update endpoint
- [ ] 2.1 Handler `update_api_key_permissions(id, req)` in `auth_handlers/keys.rs`
- [ ] 2.2 Route `PUT /auth/keys/{id}/permissions` (admin-gated)
- [ ] 2.3 Updates `permissions` + optional `scopes`; `key_hash` + `created_at` immutable
- [ ] 2.4 Integration test: round-trip a permission change; subsequent `validate_key` reflects the new perms

## 3. Per-key request bucket (for sparkline)
- [ ] 3.1 Add a per-key per-day counter ring buffer in `crates/vectorizer/src/monitoring/api_key_usage.rs`
- [ ] 3.2 Hook into `record_api_request` to bump the right key's bucket
- [ ] 3.3 Expose `GET /auth/keys/{id}/usage?window=7d` returning daily buckets
- [ ] 3.4 Integration test: 50 requests across 2 days yield correct daily aggregates

## 4. SDKs
- [ ] 4.1 Rust SDK: `update_api_key_permissions`, `get_api_key_usage` in `sdks/rust/src/client/auth.rs`
- [ ] 4.2 TypeScript SDK: same methods in `sdks/typescript/src/client/auth.ts` (camelCase)
- [ ] 4.3 Python SDK: same methods in `sdks/python/vectorizer/auth.py`
- [ ] 4.4 Bump SDK versions; add CHANGELOG entries
- [ ] 4.5 Unit tests per method

## 5. Dashboard
- [ ] 5.1 Extend `ApiKeysPage.tsx` with "Last 24h" + "Total" columns
- [ ] 5.2 Side panel renders a sparkline from the `usage` endpoint
- [ ] 5.3 Vitest cases for the new columns + panel

## 6. Tail (mandatory — enforced by rulebook v5.3.0)
- [ ] 6.1 Update or create documentation covering the implementation
- [ ] 6.2 Write tests covering the new behavior
- [ ] 6.3 Run tests and confirm they pass
