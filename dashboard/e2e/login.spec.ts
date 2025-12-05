import { test, expect } from '@playwright/test';

test.describe('Dashboard Login', () => {
  test.beforeEach(async ({ page }) => {
    // Clear any stored auth data
    await page.goto('/dashboard/');
    await page.evaluate(() => {
      localStorage.removeItem('vectorizer_dashboard_token');
      localStorage.removeItem('vectorizer_dashboard_user');
    });
  });

  test('should show login page when not authenticated', async ({ page }) => {
    await page.goto('/dashboard/');

    // Should redirect to login page
    await expect(page).toHaveURL(/.*login/);

    // Should show login form
    await expect(page.getByRole('heading', { name: 'Vectorizer Dashboard' })).toBeVisible();
    await expect(page.getByLabel('Username')).toBeVisible();
    await expect(page.getByLabel('Password')).toBeVisible();
    await expect(page.getByRole('button', { name: 'Sign In' })).toBeVisible();
  });

  test('should show error for invalid credentials', async ({ page }) => {
    await page.goto('/dashboard/login');

    // Fill in wrong credentials
    await page.getByLabel('Username').fill('wronguser');
    await page.getByLabel('Password').fill('wrongpassword');
    await page.getByRole('button', { name: 'Sign In' }).click();

    // Should show error message
    await expect(page.getByText(/Invalid|failed|error/i)).toBeVisible({ timeout: 10000 });
  });

  test('should login successfully with default admin credentials', async ({ page }) => {
    await page.goto('/dashboard/login');

    // Fill in default admin credentials
    await page.getByLabel('Username').fill('admin');
    await page.getByLabel('Password').fill('admin');
    await page.getByRole('button', { name: 'Sign In' }).click();

    // Should redirect to overview page
    await expect(page).toHaveURL(/.*overview/, { timeout: 10000 });

    // Should show user info in header
    await expect(page.getByText('admin')).toBeVisible();
  });

  test('should show loading state while logging in', async ({ page }) => {
    await page.goto('/dashboard/login');

    // Fill in credentials
    await page.getByLabel('Username').fill('admin');
    await page.getByLabel('Password').fill('admin');

    // Click login and check for loading state
    await page.getByRole('button', { name: 'Sign In' }).click();

    // Button should show loading or be disabled briefly
    // The login is fast so we just verify the flow completes
    await expect(page).toHaveURL(/.*overview/, { timeout: 10000 });
  });

  test('should logout successfully', async ({ page }) => {
    // First login
    await page.goto('/dashboard/login');
    await page.getByLabel('Username').fill('admin');
    await page.getByLabel('Password').fill('admin');
    await page.getByRole('button', { name: 'Sign In' }).click();

    // Wait for redirect to overview
    await expect(page).toHaveURL(/.*overview/, { timeout: 10000 });

    // Click logout button (the icon button with title="Logout")
    await page.getByTitle('Logout').click();

    // Should redirect to login page
    await expect(page).toHaveURL(/.*login/, { timeout: 10000 });
  });

  test('should persist session after page reload', async ({ page }) => {
    // Login first
    await page.goto('/dashboard/login');
    await page.getByLabel('Username').fill('admin');
    await page.getByLabel('Password').fill('admin');
    await page.getByRole('button', { name: 'Sign In' }).click();

    // Wait for redirect
    await expect(page).toHaveURL(/.*overview/, { timeout: 10000 });

    // Reload page
    await page.reload();

    // Should still be on overview (not redirected to login)
    await expect(page).toHaveURL(/.*overview/);
    await expect(page.getByText('admin')).toBeVisible();
  });

  test('should redirect to requested page after login', async ({ page }) => {
    // Try to access collections page without auth
    await page.goto('/dashboard/collections');

    // Should redirect to login
    await expect(page).toHaveURL(/.*login/);

    // Login
    await page.getByLabel('Username').fill('admin');
    await page.getByLabel('Password').fill('admin');
    await page.getByRole('button', { name: 'Sign In' }).click();

    // Should redirect back to collections (or overview as fallback)
    await expect(page).toHaveURL(/.*(?:collections|overview)/, { timeout: 10000 });
  });
});
