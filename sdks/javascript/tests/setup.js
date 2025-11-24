/**
 * Test setup for JavaScript SDK tests.
 */

import { beforeAll, afterAll, vi } from 'vitest';

// Global test setup
beforeAll(() => {
  // Set test timeout (vitest uses testTimeout option in config)
});

// Global test teardown
afterAll(() => {
  // Cleanup if needed
});

// Mock fetch for tests
global.fetch = vi.fn();

// Mock AbortController
global.AbortController = vi.fn().mockImplementation(() => ({
  abort: vi.fn(),
  signal: {},
}));

// Use real setTimeout and clearTimeout for tests
