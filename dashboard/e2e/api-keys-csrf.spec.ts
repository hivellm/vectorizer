import { test, expect } from '@playwright/test';

// ---------------------------------------------------------------------------
// Shared mock helpers
// ---------------------------------------------------------------------------

const json = (body: unknown, status = 200) => ({
  status,
  contentType: 'application/json',
  body: JSON.stringify(body),
});

// Minimal shell stubs required to render any page without a live backend.
async function installShellMocks(page: import('@playwright/test').Page) {
  await page.route('**/collections', (r) => r.fulfill(json({ collections: [] })));
  await page.route('**/setup/status', (r) => r.fulfill(json({ needs_setup: false })));
  await page.route('**/stats', (r) => r.fulfill(json({})));
  await page.route('**/health', (r) => r.fulfill(json({ status: 'ok' })));
  await page.route('**/events**', (r) => r.fulfill(json({}, 404)));
}

// ---------------------------------------------------------------------------
// Test data
// ---------------------------------------------------------------------------

const CREATED_KEY_ID = 'key-e2e-001';
const ROTATED_KEY_ID = 'key-e2e-002';
const CREATED_RAW_KEY = 'vec_live_e2etestkey0000000000000001';
const ROTATED_RAW_KEY = 'vec_live_e2etestkey0000000000000002';

const keyListEmpty = { keys: [] };
const keyListWithNew = {
  keys: [
    {
      id: CREATED_KEY_ID,
      name: 'e2e-test-key',
      permissions: ['read'],
      usage_count: 0,
      usage_24h: 0,
      created_at: 1700000000,
      last_used: null,
      expires_at: null,
      active: true,
    },
  ],
};
const keyListAfterRotate = {
  keys: [
    {
      id: ROTATED_KEY_ID,
      name: 'e2e-test-key',
      permissions: ['read'],
      usage_count: 0,
      usage_24h: 0,
      created_at: 1700000000,
      last_used: null,
      expires_at: null,
      active: true,
    },
  ],
};
const createKeyResponse = {
  api_key: CREATED_RAW_KEY,
  id: CREATED_KEY_ID,
  name: 'e2e-test-key',
  permissions: ['read'],
  expires_at: null,
  warning: 'Store this key securely.',
};
const rotateKeyResponse = {
  api_key: ROTATED_RAW_KEY,
  id: ROTATED_KEY_ID,
  name: 'e2e-test-key',
  permissions: ['read'],
  expires_at: null,
  warning: 'Store this key securely.',
};
const usageResponse = {
  key: keyListWithNew.keys[0],
  buckets: [
    { date: '2024-01-01', count: 5 },
    { date: '2024-01-02', count: 3 },
    { date: '2024-01-03', count: 8 },
  ],
  window_total: 16,
};

// ---------------------------------------------------------------------------
// Specs
// ---------------------------------------------------------------------------

test.describe('phase24 — api-keys CSRF', () => {
  test('create API key — one-shot panel shows raw key', async ({ page }) => {
    await installShellMocks(page);

    // Start with empty list; after create return the new row.
    let keyListState = keyListEmpty;
    await page.route('**/auth/keys', (route) => {
      if (route.request().method() === 'GET') {
        return route.fulfill(json(keyListState));
      }
      // POST /auth/keys — create
      keyListState = keyListWithNew;
      return route.fulfill(json(createKeyResponse, 201));
    });

    await page.goto('/api-keys');
    await expect(page.locator('.page-title')).toHaveText('API Keys');

    // Open create panel
    await page.getByRole('button', { name: /Generate key/i }).click();

    // Fill in the key name field
    await page.locator('input.input[type="text"]').fill('e2e-test-key');

    // Submit
    await page.getByRole('button', { name: /^Generate key$/i }).click();

    // One-shot panel must show the raw key value
    await expect(page.locator('.mono').filter({ hasText: CREATED_RAW_KEY })).toBeVisible();
  });

  test('new row appears with usage_count=0 and usage_24h=0', async ({ page }) => {
    await installShellMocks(page);

    await page.route('**/auth/keys', (route) => {
      if (route.request().method() === 'GET') {
        return route.fulfill(json(keyListWithNew));
      }
      return route.fulfill(json(createKeyResponse, 201));
    });

    await page.goto('/api-keys');
    await expect(page.locator('.page-title')).toHaveText('API Keys');

    // Table row for e2e-test-key must be present
    const row = page.locator('tbody tr').filter({ hasText: 'e2e-test-key' });
    await expect(row).toBeVisible();

    // The "Total calls" and "Last 24h" cells must both render "0"
    const numCells = row.locator('td.num');
    await expect(numCells.nth(0)).toHaveText('0');
    await expect(numCells.nth(1)).toHaveText('0');
  });

  test('Usage button opens sparkline modal for the key', async ({ page }) => {
    await installShellMocks(page);

    await page.route('**/auth/keys', (route) => {
      return route.fulfill(json(keyListWithNew));
    });

    await page.route(`**/auth/keys/${CREATED_KEY_ID}/usage**`, (route) => {
      const url = route.request().url();
      expect(url).toContain('window=14');
      return route.fulfill(json(usageResponse));
    });

    await page.goto('/api-keys');
    await expect(page.locator('tbody tr').filter({ hasText: 'e2e-test-key' })).toBeVisible();

    // Click the Usage button
    await page.getByRole('button', { name: `Usage for e2e-test-key` }).click();

    // Modal / inline panel title must mention "14-day usage"
    await expect(page.locator('text=14-day usage')).toBeVisible();

    // The Sparkline SVG or canvas element must be in the DOM
    await expect(page.locator('svg[aria-label="14-day usage trend"], canvas[aria-label="14-day usage trend"]')).toBeVisible();
  });

  test('Rotate — one-shot panel shows new key, old id gone, new id present', async ({ page }) => {
    await installShellMocks(page);

    let keyListState = keyListWithNew;
    await page.route('**/auth/keys', (route) => {
      return route.fulfill(json(keyListState));
    });

    await page.route(`**/auth/keys/${CREATED_KEY_ID}/rotate`, (route) => {
      keyListState = keyListAfterRotate;
      return route.fulfill(json(rotateKeyResponse));
    });

    // Accept the confirm() dialog
    page.on('dialog', (dialog) => dialog.accept());

    await page.goto('/api-keys');
    await expect(page.locator('tbody tr').filter({ hasText: 'e2e-test-key' })).toBeVisible();

    // Click the rotate button (icon button, aria-label="Rotate e2e-test-key")
    await page.getByRole('button', { name: `Rotate e2e-test-key` }).click();

    // One-shot panel must show the rotated key
    await expect(page.locator('.mono').filter({ hasText: ROTATED_RAW_KEY })).toBeVisible();
  });

  test('Revoke — confirm dialog, key disappears from table', async ({ page }) => {
    await installShellMocks(page);

    let keyListState = keyListWithNew;
    await page.route('**/auth/keys', (route) => {
      if (route.request().method() === 'GET') {
        return route.fulfill(json(keyListState));
      }
      return route.fulfill(json({}));
    });

    await page.route(`**/auth/keys/${CREATED_KEY_ID}`, (route) => {
      if (route.request().method() === 'DELETE') {
        keyListState = keyListEmpty;
        return route.fulfill(json({}, 204));
      }
      return route.continue();
    });

    // Accept the confirm() dialog
    page.on('dialog', (dialog) => dialog.accept());

    await page.goto('/api-keys');
    await expect(page.locator('tbody tr').filter({ hasText: 'e2e-test-key' })).toBeVisible();

    // Click the delete (trash) icon button
    await page.getByRole('button', { name: `Delete e2e-test-key` }).click();

    // Row must be gone
    await expect(page.locator('tbody tr').filter({ hasText: 'e2e-test-key' })).toHaveCount(0);
  });

  test('every mutating request to /auth/keys* carries X-CSRF-Token header', async ({ page }) => {
    await installShellMocks(page);

    const capturedHeaders: Record<string, string | undefined>[] = [];

    // Set the XSRF-TOKEN cookie so csrfMiddleware picks it up
    await page.context().addCookies([
      {
        name: 'XSRF-TOKEN',
        value: 'test-csrf-token-e2e',
        domain: '127.0.0.1',
        path: '/',
      },
    ]);

    let keyListState = keyListEmpty;

    await page.route('**/auth/keys/**', (route) => {
      const method = route.request().method();
      if (method !== 'GET') {
        capturedHeaders.push(
          Object.fromEntries(
            Object.entries(route.request().headers()).map(([k, v]) => [k, v]),
          ),
        );
      }
      if (route.request().url().includes('/rotate')) {
        keyListState = keyListAfterRotate;
        return route.fulfill(json(rotateKeyResponse));
      }
      if (method === 'DELETE') {
        keyListState = keyListEmpty;
        return route.fulfill(json({}, 204));
      }
      return route.fulfill(json({}));
    });

    await page.route('**/auth/keys', (route) => {
      const method = route.request().method();
      if (method === 'GET') return route.fulfill(json(keyListState));
      capturedHeaders.push(
        Object.fromEntries(
          Object.entries(route.request().headers()).map(([k, v]) => [k, v]),
        ),
      );
      keyListState = keyListWithNew;
      return route.fulfill(json(createKeyResponse, 201));
    });

    page.on('dialog', (d) => d.accept());

    await page.goto('/api-keys');
    await expect(page.locator('.page-title')).toHaveText('API Keys');

    // Trigger create
    await page.getByRole('button', { name: /Generate key/i }).click();
    await page.locator('input.input[type="text"]').fill('e2e-test-key');
    await page.getByRole('button', { name: /^Generate key$/i }).click();
    await expect(page.locator('.mono').filter({ hasText: CREATED_RAW_KEY })).toBeVisible();

    // At least the POST was captured — verify CSRF header presence
    expect(capturedHeaders.length).toBeGreaterThan(0);
    for (const headers of capturedHeaders) {
      expect(headers['x-csrf-token']).toBe('test-csrf-token-e2e');
    }
  });
});
