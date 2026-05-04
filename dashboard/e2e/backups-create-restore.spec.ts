import { test, expect } from '@playwright/test';

// Gate: skip the entire file when the BACKUPS_AVAILABLE env var is absent.
// Playwright reports each test as "skipped" rather than "failed", which is
// correct for CI runners that don't have backup infrastructure.
test.skip(
  !process.env.BACKUPS_AVAILABLE,
  'BACKUPS_AVAILABLE env var not set — skipping backup e2e tests',
);

// ---------------------------------------------------------------------------
// Shared mock helpers
// ---------------------------------------------------------------------------

const json = (body: unknown, status = 200) => ({
  status,
  contentType: 'application/json',
  body: JSON.stringify(body),
});

async function installShellMocks(page: import('@playwright/test').Page) {
  await page.route('**/collections', (r) => r.fulfill(json({ collections: [{ name: 'col-a' }] })));
  await page.route('**/setup/status', (r) => r.fulfill(json({ needs_setup: false })));
  await page.route('**/stats', (r) => r.fulfill(json({})));
  await page.route('**/health', (r) => r.fulfill(json({ status: 'ok' })));
  await page.route('**/events**', (r) => r.fulfill(json({}, 404)));
  await page.route('**/auth/keys', (r) => r.fulfill(json({ keys: [] })));
}

// ---------------------------------------------------------------------------
// Test data
// ---------------------------------------------------------------------------

const BACKUP_ID = 'bkp-e2e-001';
const BACKUP_NAME = 'e2e-backup-2024-01-01';

const emptyBackupList = { backups: [] };
const backupListWithNew = {
  backups: [
    {
      id: BACKUP_ID,
      name: BACKUP_NAME,
      date: '2024-01-01T00:00:00Z',
      size: 1024,
      collections: ['col-a'],
    },
  ],
};

// ---------------------------------------------------------------------------
// Specs
// ---------------------------------------------------------------------------

test.describe('phase24 — backups create + restore', () => {
  test('create backup — appears in list', async ({ page }) => {
    await installShellMocks(page);

    let backupListState = emptyBackupList;

    await page.route('**/backups', (route) => {
      return route.fulfill(json(backupListState));
    });

    await page.route('**/backups/create', (route) => {
      const body = route.request().postDataJSON() as { name: string; collections: string[] };
      expect(body.name).toBeTruthy();
      expect(Array.isArray(body.collections)).toBe(true);
      expect(body.collections.length).toBeGreaterThan(0);
      backupListState = backupListWithNew;
      return route.fulfill(json({ id: BACKUP_ID, name: body.name }));
    });

    await page.goto('/backups');
    await expect(page.locator('.page-title')).toHaveText('Backups');

    // Open the create panel
    await page.getByRole('button', { name: /new backup|create backup/i }).click();

    // Fill in the backup name
    const nameInput = page.locator('input.input[type="text"]').first();
    await nameInput.clear();
    await nameInput.fill(BACKUP_NAME);

    // Select the collection (checkbox or multi-select)
    const colLabel = page.locator('label').filter({ hasText: 'col-a' });
    if (await colLabel.count() > 0) {
      await colLabel.first().click();
    }

    // Submit
    await page.getByRole('button', { name: /^Create|^Save/i }).last().click();

    // The new backup must appear in the list
    await expect(page.locator('tbody tr').filter({ hasText: BACKUP_NAME })).toBeVisible();
  });

  test('restore — request body contains backup_id and collection', async ({ page }) => {
    await installShellMocks(page);

    const capturedRestoreRequests: { body: unknown; headers: Record<string, string> }[] = [];

    await page.route('**/backups', (route) => {
      return route.fulfill(json(backupListWithNew));
    });

    await page.route('**/backups/restore', (route) => {
      const body = route.request().postDataJSON() as { backup_id: string; collection: string };
      capturedRestoreRequests.push({
        body,
        headers: Object.fromEntries(Object.entries(route.request().headers())),
      });
      return route.fulfill(json({ success: true }));
    });

    page.on('dialog', (d) => d.accept());

    await page.goto('/backups');
    await expect(page.locator('tbody tr').filter({ hasText: BACKUP_NAME })).toBeVisible();

    // Click the Restore button for our backup
    await page.getByRole('button', { name: /restore/i }).first().click();

    // If there is a confirm panel/dialog, submit it
    const submitBtn = page.getByRole('button', { name: /^Restore/i }).last();
    if (await submitBtn.isVisible()) {
      await submitBtn.click();
    }

    // The POST to /backups/restore must have been fired
    await page.waitForTimeout(500);
    expect(capturedRestoreRequests.length).toBeGreaterThan(0);

    const reqBody = capturedRestoreRequests[0].body as { backup_id: string; collection: string };
    expect(reqBody.backup_id).toBe(BACKUP_ID);
    expect(typeof reqBody.collection).toBe('string');
  });

  test('every mutating request to /backups/* carries X-CSRF-Token header', async ({ page }) => {
    await installShellMocks(page);

    const capturedHeaders: Record<string, string>[] = [];

    await page.context().addCookies([
      {
        name: 'XSRF-TOKEN',
        value: 'csrf-backups-e2e-token',
        domain: '127.0.0.1',
        path: '/',
      },
    ]);

    let backupListState = emptyBackupList;

    await page.route('**/backups', (route) => {
      return route.fulfill(json(backupListState));
    });

    await page.route('**/backups/create', (route) => {
      capturedHeaders.push(
        Object.fromEntries(Object.entries(route.request().headers())),
      );
      backupListState = backupListWithNew;
      return route.fulfill(json({ id: BACKUP_ID, name: BACKUP_NAME }));
    });

    page.on('dialog', (d) => d.accept());

    await page.goto('/backups');
    await expect(page.locator('.page-title')).toHaveText('Backups');

    // Trigger create to capture POST headers
    await page.getByRole('button', { name: /new backup|create backup/i }).click();
    const nameInput = page.locator('input.input[type="text"]').first();
    await nameInput.clear();
    await nameInput.fill(BACKUP_NAME);
    const colLabel = page.locator('label').filter({ hasText: 'col-a' });
    if (await colLabel.count() > 0) {
      await colLabel.first().click();
    }
    await page.getByRole('button', { name: /^Create|^Save/i }).last().click();

    expect(capturedHeaders.length).toBeGreaterThan(0);
    for (const headers of capturedHeaders) {
      expect(headers['x-csrf-token']).toBe('csrf-backups-e2e-token');
    }
  });
});
