import { defineConfig, devices } from '@playwright/test';

/**
 * Playwright configuration.
 *
 * The dashboard's e2e suite runs against a local Vite dev server (started
 * automatically by the `webServer` block below). Vite dev mode serves the
 * SPA at base `/` (production uses `/dashboard/`), which keeps the path
 * assertions in `e2e/console-shell.spec.ts` valid as written.
 *
 * The console-shell smoke test does NOT require a live Vectorizer backend:
 * it stubs every `/api/**` dependency through Playwright's `page.route()`
 * inside `e2e/fixtures.ts`, so the suite can run in CI without spinning up
 * the Rust server or seeding admin credentials.
 *
 * The legacy `login.spec.ts` and `setup-wizard.spec.ts` specs were authored
 * against a live backend on `:15002` and pre-existing breakage is not in
 * scope for this configuration change — they will need their own fixtures
 * to run cleanly against the mocked dev server.
 */
export default defineConfig({
  testDir: './e2e',
  fullyParallel: true,
  forbidOnly: !!process.env.CI,
  retries: process.env.CI ? 2 : 0,
  workers: process.env.CI ? 1 : undefined,
  reporter: 'html',
  use: {
    baseURL: 'http://localhost:5179',
    trace: 'on-first-retry',
  },
  projects: [
    {
      name: 'chromium',
      use: { ...devices['Desktop Chrome'] },
    },
  ],
  webServer: {
    command: 'pnpm exec vite --port 5179 --host 127.0.0.1 --strictPort',
    port: 5179,
    reuseExistingServer: !process.env.CI,
    timeout: 60_000,
    stdout: 'pipe',
    stderr: 'pipe',
  },
});
