## 1. ApiKeysPage column rewire (MAJOR fixes 7-9)

- [x] 1.1 Rewire `dashboard/src/pages/ApiKeysPage.tsx` column accessor for "Calls (30d)" from `k.calls` to `k.usage_count` (and a `Last 24h` column from `k.usage_24h`)
- [x] 1.2 Rewire "Last used" column to read `k.last_used: number | null` and format via `formatDate(new Date(k.last_used * 1000))` (unix seconds → ISO)
- [x] 1.3 Drop the `k.masked / k.key_preview / k.key_prefix` chain — the server does not return any of those on the list endpoint. Render `—` only after a key was JUST created (using the value the create response returned, surfaced via a one-shot toast/modal)
- [x] 1.4 Update the local `ApiKey` interface in the page to match the server's `ApiKeyInfo` exactly — remove `calls`, `last_used_at`, `masked`, `key_preview`, `key_prefix`, `role` from the type definition; let `role` be derived locally as a UI-only concept
- [x] 1.5 Re-add the phase18 14-day usage sparkline modal that the merge dropped: a `Usage` button per row that opens a detail modal hitting `GET /auth/keys/{id}/usage?window=14`, rendering with the existing `Sparkline` console primitive

## 2. CSRF echo verification (BLOCKERs 1-6)

- [x] 2.1 Audit every `fetch` / `api.post` / `api.put` / `api.delete` call site in `dashboard/src/pages/` and confirm it goes through `api-middleware.ts` — audit result: ALL pages use `useApiClient`; no raw `fetch` bypasses found
- [x] 2.2 For any page that bypasses the middleware (raw `fetch`), rewire to use the shared client — N/A; no bypass found in 2.1
- [x] 2.3 Pages verified: ApiKeysPage / UsersPage / ConfigurationPage / BackupsPage all route through the shared client; CSRF echo is automatic

## 3. BackupsPage shape verification (BLOCKER 6)

- [x] 3.1 Cross-checked `BackupsPage.tsx` request bodies vs `crates/vectorizer-server/src/server/rest_handlers/backups.rs` — shapes already match
- [x] 3.2 Field-name reconciliation — N/A; no mismatches found in 3.1
- [x] 3.3 Backend error surfacing — already routed through the shared client's error toast pipeline (no per-page change needed)

## 4. 401 handling (MINOR)

- [x] 4.1 Confirm `AuthContext.tsx` redirects to `/login` on 401 from any page, OR add explicit 401 → redirect handling to each protected page
- [ ] 4.2 Add a console-shell-aware "session expired" toast on the way out

## 5. Playwright e2e

- [ ] 5.1 Add `dashboard/e2e/api-keys-csrf.spec.ts` — login, create API key, assert it appears in the table, verify `usage_count` / `usage_24h` cells, rotate, delete
- [ ] 5.2 Add `dashboard/e2e/users-csrf.spec.ts` — create user, change password, delete; verify CSRF header on every mutation
- [ ] 5.3 Add `dashboard/e2e/backups-create-restore.spec.ts` — create backup, list, restore. Gate behind a `BACKUPS_AVAILABLE` env var so CI servers without backup config don't run it
- [ ] 5.4 Add `dashboard/e2e/configuration-save.spec.ts` — open config page, save a known-safe edit, verify 200 + persistence
- [ ] 5.5 Add `dashboard/e2e/session-expired.spec.ts` — manually expire JWT, navigate to a protected page, assert redirect to login

## 6. Drop SPARK synthetic generators (depends on phase25)

- [ ] 6.1 Remove the `SPARK(n, base, amp)` helper from `OverviewPage.tsx`, `MonitoringPage.tsx`, `CollectionsPage.tsx` — every callsite must point at a real endpoint
- [ ] 6.2 Wire `OverviewPage` Ring gauges (CPU%, MEMORY%, CONNECTIONS) to `GET /metrics/runtime` (phase25); render `--` instead of `0` when the endpoint returns null
- [ ] 6.3 Wire `MonitoringPage` Sparkline (Total throughput) and Bar (per-route throughput) to `/metrics/runtime.throughput_by_route`
- [ ] 6.4 Wire `CollectionsPage` per-collection Sparkline to `GET /collections/{n}.vector_count_history` (phase25)
- [ ] 6.5 Replace the Quantization card hardcoded `4.0×` / `SQ-8bit · default` on `OverviewPage` with `GET /stats.compression_ratio` and `.default_quantization` (phase25)
- [ ] 6.6 Drop hardcoded `MAP score +8.9%` and `Recall@10 98.4%` from the Quantization card. If no real source exists yet, REMOVE the rows entirely; do not leave fake metrics in production

## 7. Drop hardcoded server identity strings

- [ ] 7.1 Replace `vectorizer 3.0.0` literal in `OverviewPage.tsx:188` with `GET /status.version`
- [ ] 7.2 Replace bind address literal `127.0.0.1:15002 (REST) · /mcp (StreamableHTTP)` with the live config from `GET /config` (or a new `/config/network` projection if the full config is too sensitive)

## 8. Tail (mandatory — enforced by rulebook v5.3.0)

- [ ] 8.1 Update `dashboard/README.md` "Recent changes" with a summary of the contract fixes + the SPARK→real-data migration
- [ ] 8.2 Run `pnpm vitest --run` (unit) and `pnpm playwright test` (e2e) and confirm all green
- [ ] 8.3 Run `pnpm lint` and confirm zero new warnings
- [ ] 8.4 Manual smoke against a live `vectorizer:3.3.0` container: every Ring/Bar/Sparkline on the dashboard renders a value derived from a real endpoint (no `Math.sin`, no `Math.random`, no hardcoded literal)
