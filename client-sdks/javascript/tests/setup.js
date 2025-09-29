/**
 * Test setup for JavaScript SDK tests.
 */

// Global test setup
beforeAll(() => {
  // Set test timeout
  jest.setTimeout(10000);
});

// Global test teardown
afterAll(() => {
  // Cleanup if needed
});

// Mock fetch for tests
global.fetch = jest.fn();

// Mock AbortController
global.AbortController = jest.fn().mockImplementation(() => ({
  abort: jest.fn(),
  signal: {},
}));

// Use real setTimeout and clearTimeout for tests
