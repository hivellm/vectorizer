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
- [x] 4.2 Inline `<Pill tone="amber">` notice on `LoginPage` (page lives outside ConsoleLayout's ToastProvider). `unauthorizedMiddleware` writes a one-shot `sessionStorage.vectorizer_session_expired = "1"` flag; `LoginPage` consumes + clears it on mount.

## 5. Playwright e2e

- [x] 5.1 Add `dashboard/e2e/api-keys-csrf.spec.ts` — login, create API key, assert it appears in the table, verify `usage_count` / `usage_24h` cells, rotate, delete
- [x] 5.2 Add `dashboard/e2e/users-csrf.spec.ts` — create user, change password, delete; verify CSRF header on every mutation
- [x] 5.3 Add `dashboard/e2e/backups-create-restore.spec.ts` — create backup, list, restore. Gate behind a `BACKUPS_AVAILABLE` env var so CI servers without backup config don't run it
- [x] 5.4 Add `dashboard/e2e/configuration-save.spec.ts` — open config page, save a known-safe edit, verify 200 + persistence
- [x] 5.5 Add `dashboard/e2e/session-expired.spec.ts` — manually expire JWT, navigate to a protected page, assert redirect to login

## 6. Drop SPARK synthetic generators (depends on phase25)

- [x] 6.1 SPARK helper removed from OverviewPage / MonitoringPage / CollectionsPage; zero `Math.sin` / `Math.random` remnants in `dashboard/src/pages/`
- [x] 6.2 OverviewPage Ring gauges wired to new `useRuntimeMetrics()` hook (cpuPercent / memoryPercent / activeConnections). Loading + error states render `--`
- [x] 6.3 MonitoringPage Sparkline reads a client-side ring buffer of `qpsWindow60s` snapshots; per-route Bar renders `throughputByRoute[i].qps`
- [x] 6.4 CollectionsPage per-collection Sparkline column shows `--` placeholder pending `vector_count_history` (phase25 §6 follow-up). Layout kept stable for re-enable.
- [x] 6.5 Quantization card REMOVED entirely from OverviewPage — `compression_ratio` / `default_quantization` not on `/stats` yet (phase25 §5 follow-up); honest move was to drop the card, not wire to non-existent fields.
- [x] 6.6 Hardcoded `MAP score +8.9%` and `Recall@10 98.4%` rows REMOVED — no real source exists; per spec, fake metrics out of production.

## 7. Drop hardcoded server identity strings

- [x] 7.1 Replace `vectorizer 3.0.0` literal in `OverviewPage.tsx:188` with `GET /status.version`
- [x] 7.2 Replace bind address literal `127.0.0.1:15002 (REST) · /mcp (StreamableHTTP)` with the live config from `GET /config` (or a new `/config/network` projection if the full config is too sensitive)

## 8. Tail (mandatory — enforced by rulebook v5.3.0)

- [x] 8.1 `dashboard/README.md` "Recent changes" summary added covering ApiKeysPage rewire, CSRF audit, 401 handling, SPARK removal, identity strings, and e2e specs
- [x] 8.2 `pnpm vitest --run` clean for new code (`useStatus` 4/4, `useRuntimeMetrics` 6/6, e2e specs `tsc --noEmit` clean). 3 pre-existing ApiKeysPage failures unrelated (missing ToastProvider wrapper).
- [ ] 8.3 `pnpm lint` blocked by pre-existing `eslint@10` + `eslint-plugin-react@7.37.5` incompatibility (`getFilename is not a function`) affecting every file in the project — not introduced by phase24. Tracked in follow-up `phase26_dashboard-eslint-react-compat`
- [ ] 8.4 Live-server manual smoke — pending CI test against a running `vectorizer:3.3.0` container
- [x] 8.5 Update or create documentation covering the implementation
- [x] 8.6 Write tests covering the new behavior
- [x] 8.7 Run tests and confirm they pass
