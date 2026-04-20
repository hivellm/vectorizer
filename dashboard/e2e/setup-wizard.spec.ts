/**
 * Setup Wizard E2E tests.
 *
 * Runs against the dashboard served by the live Vectorizer server at
 * {@link baseURL}. The tests cover the UI-level flow only — they do
 * NOT apply a real configuration (which would mutate the host's
 * workspace.yml). Everything is scoped to reads, toggles, and
 * localStorage-backed persistence.
 *
 * Run locally with `pnpm exec playwright test setup-wizard` once the
 * Vectorizer server is running on `http://localhost:15002`.
 */

import { test, expect } from '@playwright/test';

const WIZARD_PROGRESS_KEY = 'vectorizer.setup-wizard.progress.v1';
const SANDBOX_HISTORY_KEY = 'vectorizer.sandbox.history.v1';
const SANDBOX_FAVORITES_KEY = 'vectorizer.sandbox.favorites.v1';

test.describe('Setup Wizard', () => {
  test.beforeEach(async ({ page }) => {
    // Prime the browser with the admin session + clear any wizard state
    // so each test starts from a known baseline.
    await page.goto('/dashboard/login');
    await page.evaluate(
      (keys) => {
        for (const key of keys) localStorage.removeItem(key);
      },
      [WIZARD_PROGRESS_KEY, SANDBOX_HISTORY_KEY, SANDBOX_FAVORITES_KEY]
    );
  });

  test('renders the welcome step with a Get Started button', async ({ page }) => {
    await page.goto('/dashboard/setup');
    await expect(page.getByRole('heading', { name: /Welcome to Vectorizer/i })).toBeVisible({
      timeout: 10000,
    });
    await expect(page.getByRole('button', { name: /Get Started/i })).toBeVisible();
  });

  test('advances from welcome to template selection', async ({ page }) => {
    await page.goto('/dashboard/setup');
    await page.getByRole('button', { name: /Get Started/i }).click();
    await expect(page.getByRole('heading', { name: /Choose a Template/i })).toBeVisible();
  });

  test('resume banner appears when a saved progress snapshot exists', async ({ page }) => {
    // Seed a saved snapshot directly — simulates an interrupted wizard.
    await page.goto('/dashboard/setup');
    await page.evaluate(
      ([key, snapshot]) => {
        localStorage.setItem(key, snapshot);
      },
      [
        WIZARD_PROGRESS_KEY,
        JSON.stringify({
          step: 'folder',
          template: null,
          folderPath: '/workspace/demo',
          projects: [],
          savedAt: new Date().toISOString(),
        }),
      ]
    );
    await page.reload();

    await expect(page.getByText(/Resume your previous setup\?/i)).toBeVisible();
    await expect(page.getByRole('button', { name: /Resume/i })).toBeVisible();
    await expect(page.getByRole('button', { name: /Start fresh/i })).toBeVisible();
  });

  test('"Start fresh" clears the saved progress snapshot', async ({ page }) => {
    await page.goto('/dashboard/setup');
    await page.evaluate(
      ([key, snapshot]) => {
        localStorage.setItem(key, snapshot);
      },
      [
        WIZARD_PROGRESS_KEY,
        JSON.stringify({
          step: 'folder',
          template: null,
          folderPath: '/workspace/demo',
          projects: [],
          savedAt: new Date().toISOString(),
        }),
      ]
    );
    await page.reload();
    await page.getByRole('button', { name: /Start fresh/i }).click();

    const stored = await page.evaluate(
      (key) => localStorage.getItem(key),
      WIZARD_PROGRESS_KEY
    );
    expect(stored).toBeNull();
  });
});

test.describe('API Documentation → Sandbox', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/dashboard/login');
    await page.evaluate(
      (keys) => {
        for (const key of keys) localStorage.removeItem(key);
      },
      [SANDBOX_HISTORY_KEY, SANDBOX_FAVORITES_KEY]
    );
  });

  test('renders the API documentation page with categorized endpoints', async ({ page }) => {
    await page.goto('/dashboard/docs');
    await expect(page.getByRole('heading', { name: /API Documentation/i })).toBeVisible({
      timeout: 10000,
    });
    await expect(page.getByPlaceholder(/Search endpoints/i)).toBeVisible();
  });

  test('sandbox favorites survive a page reload via localStorage', async ({ page }) => {
    await page.goto('/dashboard/docs');

    // Seed the favorites list directly — this test covers the
    // persistence contract, not the click-through flow (which the
    // unit tests already cover for the hook internals).
    await page.evaluate(
      ([key, favorites]) => {
        localStorage.setItem(key, favorites);
      },
      [
        SANDBOX_FAVORITES_KEY,
        JSON.stringify([
          {
            id: 'GET /health ',
            method: 'GET',
            path: '/health',
            pathParams: {},
            body: '',
            ranAt: new Date().toISOString(),
          },
        ]),
      ]
    );

    await page.reload();
    const persisted = await page.evaluate(
      (key) => localStorage.getItem(key),
      SANDBOX_FAVORITES_KEY
    );
    expect(persisted).not.toBeNull();
    expect(persisted).toContain('/health');
  });
});
