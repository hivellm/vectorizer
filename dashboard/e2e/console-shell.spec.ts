import { test, expect } from './fixtures';

test.describe('console shell', () => {
  test('sidebar and topbar render on /overview', async ({ page }) => {
    await page.goto('/overview');
    await expect(page.locator('.sidebar')).toBeVisible();
    await expect(page.locator('.topbar')).toBeVisible();
    await expect(page.locator('.sidebar-brand .name')).toHaveText('Vectorizer');
  });

  test('command palette opens with ⌘K', async ({ page }) => {
    await page.goto('/overview');
    // Wait for the layout to mount before firing the global ⌘K shortcut —
    // the keydown listener lives on `window` and is only registered after
    // React commits ConsoleLayout.
    await expect(page.locator('.topbar')).toBeVisible();
    await page.keyboard.press('Meta+k');
    await expect(page.getByPlaceholder(/Search or type a command/)).toBeVisible();
  });

  test('navigates to Collections via palette', async ({ page }) => {
    await page.goto('/overview');
    await expect(page.locator('.topbar')).toBeVisible();
    await page.keyboard.press('Meta+k');
    await page.keyboard.type('Collect');
    await page.keyboard.press('Enter');
    await expect(page).toHaveURL(/\/collections$/);
  });
});
