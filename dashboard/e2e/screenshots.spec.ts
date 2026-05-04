import { test } from './fixtures';

const ROUTES: Array<{ path: string; name: string; waitFor?: string }> = [
  { path: '/overview',      name: 'overview',      waitFor: '.page-title' },
  { path: '/collections',   name: 'collections',   waitFor: '.page-title' },
  { path: '/search',        name: 'search',        waitFor: '.page-title' },
  { path: '/vectors',       name: 'vectors',       waitFor: '.page-title' },
  { path: '/monitoring',    name: 'monitoring',    waitFor: '.page-title' },
  { path: '/cluster',       name: 'replication',   waitFor: '.page-title' },
  { path: '/api-keys',      name: 'api-keys',      waitFor: '.page-title' },
  { path: '/mcp-tools',     name: 'mcp-tools',     waitFor: '.page-title' },
  { path: '/configuration', name: 'settings',      waitFor: '.page-title' },
  { path: '/file-watcher',  name: 'file-watcher',  waitFor: '.page-title' },
  { path: '/graph',         name: 'graph',         waitFor: '.page-title' },
  { path: '/connections',   name: 'connections',   waitFor: '.page-title' },
  { path: '/workspace',     name: 'workspace',     waitFor: '.page-title' },
  { path: '/logs',          name: 'logs',          waitFor: '.page-title' },
  { path: '/backups',       name: 'backups',       waitFor: '.page-title' },
  { path: '/users',         name: 'users',         waitFor: '.page-title' },
  { path: '/docs',          name: 'api-docs',      waitFor: '.page-title' },
];

test.describe.configure({ mode: 'serial' });

test.describe('console screenshots', () => {
  for (const route of ROUTES) {
    test(`captures ${route.name} (${route.path})`, async ({ page }) => {
      await page.setViewportSize({ width: 1440, height: 900 });
      await page.goto(route.path);
      // Wait for the page chrome and the body[data-console] activator
      await page.waitForSelector('.sidebar', { state: 'visible' });
      await page.waitForSelector('.topbar',  { state: 'visible' });
      if (route.waitFor) {
        await page.waitForSelector(route.waitFor, { state: 'visible' });
      }
      // Allow async data fetches and CSS transitions to settle
      await page.waitForTimeout(700);
      await page.screenshot({
        path: `docs/screenshots/${route.name}.png`,
        fullPage: false,
      });
    });
  }
});
