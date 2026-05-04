## 1. ApiKeysPage column rewire (MAJOR fixes 7-9)

- [ ] 1.1 Rewire `dashboard/src/pages/ApiKeysPage.tsx` column accessor for "Calls (30d)" from `k.calls` to `k.usage_count` (and a `Last 24h` column from `k.usage_24h`)
- [ ] 1.2 Rewire "Last used" column to read `k.last_used: number | null` and format via `formatDate(new Date(k.last_used * 1000))` (unix seconds → ISO)
- [ ] 1.3 Drop the `k.masked / k.key_preview / k.key_prefix` chain — the server does not return any of those on the list endpoint. Render `—` only after a key was JUST created (using the value the create response returned, surfaced via a one-shot toast/modal)
- [ ] 1.4 Update the local `ApiKey` interface in the page to match the server's `ApiKeyInfo` exactly — remove `calls`, `last_used_at`, `masked`, `key_preview`, `key_prefix`, `role` from the type definition; let `role` be derived locally as a UI-only concept
- [ ] 1.5 Re-add the phase18 14-day usage sparkline modal that the merge dropped: a `Usage` button per row that opens a detail modal hitting `GET /auth/keys/{id}/usage?window=14`, rendering with the existing `Sparkline` console primitive

## 2. CSRF echo verification (BLOCKERs 1-6)

- [ ] 2.1 Audit every `fetch` / `api.post` / `api.put` / `api.delete` call site in `dashboard/src/pages/` and confirm it goes through `api-middleware.ts` (which already reads `XSRF-TOKEN` cookie and echoes `X-CSRF-Token`)
- [ ] 2.2 For any page that bypasses the middleware (raw `fetch`), rewire to use the shared client
- [ ] 2.3 Pages to verify specifically: `ApiKeysPage` (POST /auth/keys, DELETE /auth/keys/{id}, POST /auth/keys/{id}/rotate, PUT /auth/keys/{id}/permissions), `UsersPage` (POST /auth/users, PUT /auth/users/{u}/password, DELETE /auth/users/{u}), `ConfigurationPage` (POST /config), `BackupsPage` (POST /backups/create, POST /backups/restore)

## 3. BackupsPage shape verification (BLOCKER 6)

- [ ] 3.1 Cross-check `BackupsPage.tsx` request bodies for POST /backups/create and POST /backups/restore against the actual handler in `crates/vectorizer-server/src/server/rest_handlers/backups.rs`
- [ ] 3.2 Reconcile any field-name differences (e.g. `backup_id` vs `id`, `collections` vs `collection_names`)
- [ ] 3.3 Surface backend errors (e.g. permission denied, backup not found) as toasts instead of swallowing

## 4. 401 handling (MINOR)

- [ ] 4.1 Confirm `AuthContext.tsx` redirects to `/login` on 401 from any page, OR add explicit 401 → redirect handling to each protected page
- [ ] 4.2 Add a console-shell-aware "session expired" toast on the way out

## 5. Playwright e2e

- [ ] 5.1 Add `dashboard/e2e/api-keys-csrf.spec.ts` — login, create API key, assert it appears in the table, verify `usage_count` / `usage_24h` cells, rotate, delete
- [ ] 5.2 Add `dashboard/e2e/users-csrf.spec.ts` — create user, change password, delete; verify CSRF header on every mutation
- [ ] 5.3 Add `dashboard/e2e/backups-create-restore.spec.ts` — create backup, list, restore. Gate behind a `BACKUPS_AVAILABLE` env var so CI servers without backup config don't run it
- [ ] 5.4 Add `dashboard/e2e/configuration-save.spec.ts` — open config page, save a known-safe edit, verify 200 + persistence
- [ ] 5.5 Add `dashboard/e2e/session-expired.spec.ts` — manually expire JWT, navigate to a protected page, assert redirect to login

## 6. Tail (mandatory — enforced by rulebook v5.3.0)

- [ ] 6.1 Update `dashboard/README.md` "Recent changes" with a summary of the contract fixes
- [ ] 6.2 Run `pnpm vitest --run` (unit) and `pnpm playwright test` (e2e) and confirm all green
- [ ] 6.3 Run `pnpm lint` and confirm zero new warnings
