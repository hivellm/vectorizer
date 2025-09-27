/**
 * Test setup for TypeScript SDK tests.
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
const MockWebSocket = jest.fn().mockImplementation(() => ({
  close: jest.fn(),
  send: jest.fn(),
  addEventListener: jest.fn(),
  removeEventListener: jest.fn(),
  readyState: 1,
}));

// Add static properties to the mock
Object.assign(MockWebSocket, {
  CONNECTING: 0,
  OPEN: 1,
  CLOSING: 2,
  CLOSED: 3,
});

global.WebSocket = MockWebSocket as any;

// Mock AbortController
global.AbortController = jest.fn().mockImplementation(() => ({
  abort: jest.fn(),
  signal: {
    aborted: false,
    addEventListener: jest.fn(),
    removeEventListener: jest.fn(),
  },
}));

// Mock setTimeout and clearTimeout
const originalSetTimeout = global.setTimeout;
const mockSetTimeout = jest.fn((callback, delay) => {
  return originalSetTimeout(callback, delay);
});

// Add __promisify__ property for Node.js compatibility
Object.assign(mockSetTimeout, {
  __promisify__: jest.fn(),
});

global.setTimeout = mockSetTimeout as any;

const originalClearTimeout = global.clearTimeout;
global.clearTimeout = jest.fn((id) => {
  return originalClearTimeout(id);
});
