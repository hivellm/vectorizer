/**
 * Playwright fixtures for the console-shell smoke suite.
 *
 * Installs `page.route()` stubs for every REST endpoint the console shell
 * touches on first paint, so the spec can run without a live Vectorizer
 * backend or admin credentials. Using `auto: true` makes the fixture apply
 * to every test in the file that imports `test`/`expect` from this module.
 *
 * Endpoints stubbed:
 *
 *   - `/collections`     — AuthContext probes this to decide whether the
 *                          server requires auth. A 200 response means
 *                          `authRequired = false`, so `ProtectedRoute`
 *                          treats the user as authenticated and skips the
 *                          login redirect. The same endpoint also feeds
 *                          `useCollections().listCollections()`.
 *   - `/setup/status`    — `useSetupAutoRedirect` consults this from the
 *                          relative origin (Vite dev = `:5173`). Returning
 *                          `{ needs_setup: false }` keeps the user on the
 *                          requested route instead of bouncing to /setup.
 *   - `/stats`           — `useMetrics` polling source.
 *   - `/health`          — `useStats` polling source.
 *   - `/events`          — `useEvents` polls this; a 404 puts the hook in
 *                          its "endpoint unavailable" placeholder state
 *                          and stops further polling.
 *   - `/auth/keys`       — `useApiKeys` (loaded by topbar/menu helpers in
 *                          some routes). Empty list keeps everything
 *                          quiet.
 *
 * Hooks resolve URLs through `useApiClient`, which in dev mode targets
 * `http://localhost:15002` (cross-origin to the dev server on `:5173`).
 * Playwright's wildcard pattern `**\/path` matches across origins, so the
 * same single rule covers both the same-origin (relative) and cross-origin
 * variants.
 */
import { test as base, type Page } from '@playwright/test';

const json = (body: unknown, status = 200) => ({
  status,
  contentType: 'application/json',
  body: JSON.stringify(body),
});

async function installShellMocks(page: Page) {
  // Auth probe + collections list. AuthContext.checkAuthRequired() fetches
  // /collections and only treats the server as auth-required when it gets
  // a 401 — any other status (including this 200 with an empty list) marks
  // auth as NOT required, so ProtectedRoute lets the route render.
  await page.route('**/collections', (route) =>
    route.fulfill(json({ collections: [] })),
  );

  // Setup wizard guard. `useSetupAutoRedirect` calls `/setup/status`
  // relative to the dev origin; returning `needs_setup: false` keeps the
  // requested route from being replaced with `/setup`.
  await page.route('**/setup/status', (route) =>
    route.fulfill(json({ needs_setup: false })),
  );

  // Metrics poller (useMetrics).
  await page.route('**/stats', (route) => route.fulfill(json({})));

  // System stats poller (useStats reads /health for cache + WAL fields).
  await page.route('**/health', (route) => route.fulfill(json({ status: 'ok' })));

  // Events poller (useEvents). A 404 makes the hook flip to
  // `available = false` and stop polling, exactly the placeholder path.
  await page.route('**/events**', (route) => route.fulfill(json({}, 404)));

  // API key list — some chrome surfaces preload it.
  await page.route('**/auth/keys', (route) => route.fulfill(json({ keys: [] })));
}

export const test = base.extend<{ withShellMocks: void }>({
  withShellMocks: [
    async ({ page }, use) => {
      await installShellMocks(page);
      await use();
    },
    { auto: true },
  ],
});

export { expect } from '@playwright/test';
