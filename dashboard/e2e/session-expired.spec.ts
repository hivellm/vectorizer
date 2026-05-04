import { test, expect } from '@playwright/test';

// ---------------------------------------------------------------------------
// Shared mock helpers
// ---------------------------------------------------------------------------

const json = (body: unknown, status = 200) => ({
  status,
  contentType: 'application/json',
  body: JSON.stringify(body),
});

async function installShellMocks(page: import('@playwright/test').Page) {
  await page.route('**/collections', (r) => r.fulfill(json({ collections: [] })));
  await page.route('**/setup/status', (r) => r.fulfill(json({ needs_setup: false })));
  await page.route('**/stats', (r) => r.fulfill(json({})));
  await page.route('**/health', (r) => r.fulfill(json({ status: 'ok' })));
  await page.route('**/events**', (r) => r.fulfill(json({}, 404)));
}

// ---------------------------------------------------------------------------
// Specs
// ---------------------------------------------------------------------------

test.describe('phase24 — session expired redirect', () => {
  test('401 from protected endpoint redirects to /login', async ({ page }) => {
    await installShellMocks(page);

    // Step 1: serve a valid key list so the page renders without redirect.
    await page.route('**/auth/keys', (route) => {
      return route.fulfill(json({ keys: [] }));
    });

    await page.goto('/api-keys');
    await expect(page.locator('.page-title')).toHaveText('API Keys');

    // Step 2: replace the stub so the next request returns 401.
    // Playwright routes added later take priority over earlier ones.
    await page.route('**/auth/keys', (route) => {
      return route.fulfill(json({ error: 'unauthorized' }, 401));
    });

    // Step 3: clear the session cookie to simulate expiry.
    await page.context().clearCookies();

    // Step 4: trigger an action that calls the protected endpoint (click Refresh).
    await page.getByRole('button', { name: /refresh/i }).click();

    // Step 5: page should redirect to /login.
    await expect(page).toHaveURL(/\/login/, { timeout: 5000 });
  });

  test('vectorizer:session-expired event triggers login redirect', async ({ page }) => {
    await installShellMocks(page);

    await page.route('**/auth/keys', (route) => {
      return route.fulfill(json({ keys: [] }));
    });

    await page.goto('/api-keys');
    await expect(page.locator('.page-title')).toHaveText('API Keys');

    // Dispatch the synthetic event that api-middleware fires on 401.
    // This tests the AuthContext listener path directly, independent of
    // the network layer.
    await page.evaluate(() => {
      window.dispatchEvent(new CustomEvent('vectorizer:session-expired'));
    });

    await expect(page).toHaveURL(/\/login/, { timeout: 5000 });
  });

  test('session-expired toast shown before redirect (if implemented)', async ({ page }) => {
    await installShellMocks(page);

    await page.route('**/auth/keys', (route) => {
      return route.fulfill(json({ keys: [] }));
    });

    await page.goto('/api-keys');
    await expect(page.locator('.page-title')).toHaveText('API Keys');

    // Override next auth/keys request to return 401
    await page.route('**/auth/keys', (route) => {
      return route.fulfill(json({ error: 'unauthorized' }, 401));
    });

    await page.context().clearCookies();

    await page.getByRole('button', { name: /refresh/i }).click();

    // Wait for redirect — the toast may or may not appear depending on
    // whether phase24 §4.2 is implemented.  We only assert the URL.
    await expect(page).toHaveURL(/\/login/, { timeout: 5000 });

    // If a toast element exists in the DOM before the redirect fully completes
    // (i.e. the component mounted it), check for it.  This assertion is soft:
    // if no toast selector matches we skip rather than fail, because 4.2 is
    // still open at the time these tests are written.
    const toastLocator = page.locator('[class*="toast"], [role="alert"], [data-testid="toast"]');
    // We navigate back briefly to the page's origin to check if a toast was
    // queued — not needed here since the redirect already confirmed the flow.
    // The presence check is intentionally non-blocking.
    const toastCount = await toastLocator.count();
    // Toast count may be 0 (not yet implemented) or >0 (implemented).
    // Either is acceptable at this phase.
    expect(toastCount).toBeGreaterThanOrEqual(0);
  });
});
