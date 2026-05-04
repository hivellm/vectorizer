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
  await page.route('**/auth/keys', (r) => r.fulfill(json({ keys: [] })));
}

// ---------------------------------------------------------------------------
// Test data
// ---------------------------------------------------------------------------

const TEST_USERNAME = 'e2e-user-1';

const emptyUserList = { users: [] };
const userListWithNew = {
  users: [
    {
      user_id: 'usr-e2e-001',
      username: TEST_USERNAME,
      roles: ['Viewer'],
      created_at: '2024-01-01T00:00:00Z',
      last_login_at: null,
    },
  ],
};

// ---------------------------------------------------------------------------
// Specs
// ---------------------------------------------------------------------------

test.describe('phase24 — users CSRF', () => {
  test('create user — appears in table', async ({ page }) => {
    await installShellMocks(page);

    let userListState = emptyUserList;

    await page.route('**/auth/users', (route) => {
      const method = route.request().method();
      if (method === 'GET') return route.fulfill(json(userListState));
      // POST /auth/users — create
      userListState = userListWithNew;
      return route.fulfill(json({ user_id: 'usr-e2e-001', username: TEST_USERNAME }));
    });

    await page.goto('/users');
    await expect(page.locator('.page-title')).toHaveText('Users');

    // Open create panel
    await page.getByRole('button', { name: /Add user|Create user|New user/i }).click();

    // Fill username and password fields
    const inputs = page.locator('input.input');
    await inputs.nth(0).fill(TEST_USERNAME);
    await inputs.nth(1).fill('SecurePass1!');
    await inputs.nth(2).fill('SecurePass1!');

    await page.getByRole('button', { name: /Create|Add/i }).last().click();

    // Row must appear
    await expect(page.locator('tbody tr').filter({ hasText: TEST_USERNAME })).toBeVisible();
  });

  test('change password — PUT sent to correct endpoint', async ({ page }) => {
    await installShellMocks(page);

    const capturedRequests: { url: string; method: string; headers: Record<string, string> }[] = [];

    await page.route('**/auth/users', (route) => {
      return route.fulfill(json(userListWithNew));
    });

    await page.route(`**/auth/users/${TEST_USERNAME}/password`, (route) => {
      capturedRequests.push({
        url: route.request().url(),
        method: route.request().method(),
        headers: Object.fromEntries(Object.entries(route.request().headers())),
      });
      return route.fulfill(json({ success: true }));
    });

    await page.goto('/users');
    await expect(page.locator('tbody tr').filter({ hasText: TEST_USERNAME })).toBeVisible();

    // Click the change password button for this user
    await page.getByRole('button', { name: /password|change/i }).first().click();

    const pwdInputs = page.locator('input[type="password"]');
    await pwdInputs.nth(0).fill('NewPass456!');
    await pwdInputs.nth(1).fill('NewPass456!');

    await page.getByRole('button', { name: /Save|Change|Update/i }).last().click();

    expect(capturedRequests.length).toBeGreaterThan(0);
    expect(capturedRequests[0].method).toBe('PUT');
    expect(capturedRequests[0].url).toContain(`/auth/users/${TEST_USERNAME}/password`);
  });

  test('delete user — row disappears', async ({ page }) => {
    await installShellMocks(page);

    let userListState = userListWithNew;

    await page.route('**/auth/users', (route) => {
      return route.fulfill(json(userListState));
    });

    await page.route(`**/auth/users/${TEST_USERNAME}`, (route) => {
      if (route.request().method() === 'DELETE') {
        userListState = emptyUserList;
        return route.fulfill(json({}, 204));
      }
      return route.continue();
    });

    page.on('dialog', (d) => d.accept());

    await page.goto('/users');
    await expect(page.locator('tbody tr').filter({ hasText: TEST_USERNAME })).toBeVisible();

    await page.getByRole('button', { name: /delete|remove/i }).first().click();

    await expect(page.locator('tbody tr').filter({ hasText: TEST_USERNAME })).toHaveCount(0);
  });

  test('every mutating request to /auth/users* carries X-CSRF-Token header', async ({ page }) => {
    await installShellMocks(page);

    const capturedHeaders: Record<string, string>[] = [];

    // Plant the XSRF-TOKEN cookie so csrfMiddleware picks it up
    await page.context().addCookies([
      {
        name: 'XSRF-TOKEN',
        value: 'csrf-users-e2e-token',
        domain: '127.0.0.1',
        path: '/',
      },
    ]);

    let userListState = emptyUserList;

    await page.route('**/auth/users', (route) => {
      const method = route.request().method();
      if (method === 'GET') return route.fulfill(json(userListState));
      capturedHeaders.push(
        Object.fromEntries(Object.entries(route.request().headers())),
      );
      userListState = userListWithNew;
      return route.fulfill(json({ user_id: 'usr-e2e-001', username: TEST_USERNAME }));
    });

    await page.route(`**/auth/users/${TEST_USERNAME}/**`, (route) => {
      const method = route.request().method();
      if (method !== 'GET') {
        capturedHeaders.push(
          Object.fromEntries(Object.entries(route.request().headers())),
        );
      }
      return route.fulfill(json({ success: true }));
    });

    await page.route(`**/auth/users/${TEST_USERNAME}`, (route) => {
      const method = route.request().method();
      if (method === 'DELETE') {
        capturedHeaders.push(
          Object.fromEntries(Object.entries(route.request().headers())),
        );
        userListState = emptyUserList;
        return route.fulfill(json({}, 204));
      }
      return route.continue();
    });

    page.on('dialog', (d) => d.accept());

    await page.goto('/users');
    await expect(page.locator('.page-title')).toHaveText('Users');

    // POST /auth/users — create
    await page.getByRole('button', { name: /Add user|Create user|New user/i }).click();
    const inputs = page.locator('input.input');
    await inputs.nth(0).fill(TEST_USERNAME);
    await inputs.nth(1).fill('SecurePass1!');
    await inputs.nth(2).fill('SecurePass1!');
    await page.getByRole('button', { name: /Create|Add/i }).last().click();
    await expect(page.locator('tbody tr').filter({ hasText: TEST_USERNAME })).toBeVisible();

    // PUT /auth/users/{u}/password — change password
    await page.getByRole('button', { name: /password|change/i }).first().click();
    const pwdInputs = page.locator('input[type="password"]');
    await pwdInputs.nth(0).fill('NewPass456!');
    await pwdInputs.nth(1).fill('NewPass456!');
    await page.getByRole('button', { name: /Save|Change|Update/i }).last().click();

    // DELETE /auth/users/{u}
    await page.getByRole('button', { name: /delete|remove/i }).first().click();

    // All captured mutating requests must carry the CSRF token
    expect(capturedHeaders.length).toBeGreaterThan(0);
    for (const headers of capturedHeaders) {
      expect(headers['x-csrf-token']).toBe('csrf-users-e2e-token');
    }
  });
});
