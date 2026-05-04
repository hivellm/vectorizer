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

// API origin used by `useApiClient` in dev mode. Mocks are scoped to this
// origin so they don't intercept the SPA's own routes on the Vite dev
// server (e.g. visiting `/collections` would otherwise be served the JSON
// stub instead of the SPA index).
const API = 'http://localhost:15002';

async function installShellMocks(page: Page) {
  // Playwright iterates `page.route()` rules in REVERSE registration order
  // (last registered wins), so register the broad catch-all FIRST and the
  // specific overrides LAST. The catch-all returns an empty 200 for any
  // API call we didn't explicitly stub, which keeps loading states from
  // hanging the screenshot spec on pages that hit unfamiliar endpoints.
  await page.route(`${API}/**`, (route) => route.fulfill(json({})));

  // Some pages probe `/collections/<name>/...` — empty 200 keeps them quiet.
  await page.route(`${API}/collections/**`, (route) =>
    route.fulfill(json({})),
  );

  // Auth probe + collections list. AuthContext.checkAuthRequired() fetches
  // /collections and only treats the server as auth-required when it gets
  // a 401 — any other status (including this 200 with an empty list) marks
  // auth as NOT required, so ProtectedRoute lets the route render.
  await page.route(`${API}/collections`, (route) =>
    route.fulfill(json({ collections: [] })),
  );

  // Setup wizard guard. `useSetupAutoRedirect` calls `/setup/status` from
  // the same origin as the dev server (relative URL); returning
  // `needs_setup: false` keeps the requested route from being replaced
  // with `/setup`. Match both origins (relative + cross-origin) so the
  // hook is satisfied regardless of which client made the call.
  await page.route('**/setup/status', (route) =>
    route.fulfill(json({ needs_setup: false })),
  );

  // Metrics poller (useMetrics).
  await page.route(`${API}/stats`, (route) => route.fulfill(json({})));

  // System stats poller (useStats reads /health for cache + WAL fields).
  await page.route(`${API}/health`, (route) =>
    route.fulfill(json({ status: 'ok' })),
  );

  // Events poller (useEvents). A 404 makes the hook flip to
  // `available = false` and stop polling, exactly the placeholder path.
  await page.route(`${API}/events**`, (route) => route.fulfill(json({}, 404)));

  // API key list — some chrome surfaces preload it.
  await page.route(`${API}/auth/keys`, (route) =>
    route.fulfill(json({ keys: [] })),
  );

  // FileWatcher metrics. The page renders `metrics && (...)` blocks that
  // unconditionally read deep fields (e.g. `metrics.files.total_files_processed`),
  // so an empty `{}` from the catch-all crashes the React tree on /file-watcher.
  // Returning a fully-shaped zeroed object lets the page render its empty
  // state without throwing.
  await page.route(`${API}/metrics`, (route) =>
    route.fulfill(
      json({
        timing: {
          avg_file_processing_ms: 0,
          avg_discovery_ms: 0,
          avg_sync_ms: 0,
          uptime_seconds: 0,
          peak_processing_ms: 0,
        },
        files: {
          total_files_processed: 0,
          files_processed_success: 0,
          files_processed_error: 0,
          files_skipped: 0,
          files_in_progress: 0,
          files_discovered: 0,
          files_removed: 0,
          files_indexed_realtime: 0,
        },
        system: {
          memory_usage_bytes: 0,
          cpu_usage_percent: 0,
          thread_count: 0,
          active_file_handles: 0,
          disk_io_ops_per_sec: 0,
          network_io_bytes_per_sec: 0,
        },
        network: {
          total_api_requests: 0,
          successful_api_requests: 0,
          failed_api_requests: 0,
          avg_api_response_ms: 0,
          peak_api_response_ms: 0,
          active_connections: 0,
        },
        status: {
          total_errors: 0,
          total_warnings: 0,
          is_healthy: true,
        },
      }),
    ),
  );

  // FileWatcher config endpoint (`/workspace/config`). Returns a minimally
  // shaped config so `getStatus()` can deref `global_settings.file_watcher`
  // without throwing.
  await page.route(`${API}/workspace/config`, (route) =>
    route.fulfill(
      json({
        global_settings: {
          file_watcher: {
            enabled: false,
            watch_paths: [],
            auto_discovery: true,
            enable_auto_update: true,
            hot_reload: true,
            exclude_patterns: [],
          },
        },
      }),
    ),
  );
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
