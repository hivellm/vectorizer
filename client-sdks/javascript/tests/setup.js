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

// Mock WebSocket for tests
global.WebSocket = jest.fn().mockImplementation(() => ({
  close: jest.fn(),
  send: jest.fn(),
  addEventListener: jest.fn(),
  removeEventListener: jest.fn(),
  readyState: 1,
  CONNECTING: 0,
  OPEN: 1,
  CLOSING: 2,
  CLOSED: 3,
}));

// Mock AbortController
global.AbortController = jest.fn().mockImplementation(() => ({
  abort: jest.fn(),
  signal: {},
}));

// Mock setTimeout and clearTimeout
global.setTimeout = jest.fn((callback, delay) => {
  return setTimeout(callback, delay);
});

global.clearTimeout = jest.fn((id) => {
  return clearTimeout(id);
});
