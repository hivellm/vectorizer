/**
 * Tests for WebSocketClient utility.
 */

import { WebSocketClient } from '../../src/utils/websocket-client';
import { NetworkError, TimeoutError } from '../../src/exceptions';

// Mock WebSocket
const mockWebSocket = {
  close: jest.fn(),
  send: jest.fn(),
  terminate: jest.fn(),
  addEventListener: jest.fn(),
  removeEventListener: jest.fn(),
  on: jest.fn(),
  off: jest.fn(),
  emit: jest.fn(),
  readyState: 1,
  CONNECTING: 0,
  OPEN: 1,
  CLOSING: 2,
  CLOSED: 3,
};

// Mock WebSocket constructor
jest.mock('ws', () => {
  return jest.fn().mockImplementation(() => mockWebSocket);
});

const MockWebSocket = jest.mocked(require('ws'));

describe('WebSocketClient', () => {
  let wsClient: WebSocketClient;

  beforeEach(() => {
    jest.clearAllMocks();
    wsClient = new WebSocketClient({
      url: 'ws://localhost:15001/ws',
      apiKey: 'test-api-key',
      timeout: 5000,
    });
  });

  describe('constructor', () => {
    it('should create client with default config', () => {
      const client = new WebSocketClient({
        url: 'ws://localhost:15001/ws',
      });

      expect(client).toBeInstanceOf(WebSocketClient);
    });

    it('should create client with custom config', () => {
      const client = new WebSocketClient({
        url: 'ws://custom:8080/ws',
        apiKey: 'custom-key',
        timeout: 10000,
        reconnectInterval: 3000,
        maxReconnectAttempts: 3,
      });

      expect(client).toBeInstanceOf(WebSocketClient);
    });
  });

  describe('connect', () => {
    it('should connect successfully', async () => {
      const mockWs = {
        ...mockWebSocket,
        readyState: 1,
      };
      MockWebSocket.mockImplementationOnce(() => mockWs);

      // Mock successful connection
      setTimeout(() => {
        const openHandler = mockWs.on.mock.calls.find(call => call[0] === 'open')?.[1];
        if (openHandler) {
          openHandler();
        }
      }, 0);

      await wsClient.connect();

      expect(MockWebSocket).toHaveBeenCalledWith('ws://localhost:15001/ws', {
        headers: {
          'Authorization': 'Bearer test-api-key',
        },
      });
      expect(wsClient.connected).toBe(true);
    });

    it('should connect without API key', async () => {
      const client = new WebSocketClient({
        url: 'ws://localhost:15001/ws',
      });

      const mockWs = {
        ...mockWebSocket,
        readyState: 1,
      };
      MockWebSocket.mockImplementationOnce(() => mockWs);

      setTimeout(() => {
        const openHandler = mockWs.on.mock.calls.find(call => call[0] === 'open')?.[1];
        if (openHandler) {
          openHandler();
        }
      }, 0);

      await client.connect();

      expect(MockWebSocket).toHaveBeenCalledWith('ws://localhost:15001/ws', {
        headers: {},
      });
    });

    it('should handle connection error', async () => {
      const mockWs = {
        ...mockWebSocket,
        readyState: 3,
      };
      MockWebSocket.mockImplementationOnce(() => mockWs);

      const error = new Error('Connection failed');
      setTimeout(() => {
        mockWs.addEventListener.mock.calls
          .find(call => call[0] === 'error')?.[1](error);
      }, 0);

      await expect(wsClient.connect()).rejects.toThrow(NetworkError);
      expect(wsClient.connected).toBe(false);
    });

    it('should handle connection timeout', async () => {
      const mockWs = {
        ...mockWebSocket,
        readyState: 0,
      };
      MockWebSocket.mockImplementationOnce(() => mockWs);

      // Don't trigger open event, let it timeout
      await expect(wsClient.connect()).rejects.toThrow(TimeoutError);
    });

    it('should not connect if already connected', async () => {
      const mockWs = {
        ...mockWebSocket,
        readyState: 1,
      };
      MockWebSocket.mockImplementationOnce(() => mockWs);

      setTimeout(() => {
        const openHandler = mockWs.on.mock.calls.find(call => call[0] === 'open')?.[1];
        if (openHandler) {
          openHandler();
        }
      }, 0);

      await wsClient.connect();
      expect(wsClient.connected).toBe(true);

      // Try to connect again
      await wsClient.connect();
      expect(MockWebSocket).toHaveBeenCalledTimes(1);
    });

    it('should not connect if already connecting', async () => {
      const mockWs = {
        ...mockWebSocket,
        readyState: 0,
      };
      MockWebSocket.mockImplementationOnce(() => mockWs);

      // Start connection but don't complete it
      const connectPromise = wsClient.connect();

      // Try to connect again while first is in progress
      const secondConnectPromise = wsClient.connect();

      // Both should resolve to the same promise
      expect(connectPromise).toBe(secondConnectPromise);
    });
  });

  describe('disconnect', () => {
    it('should disconnect successfully', () => {
      const mockWs = {
        ...mockWebSocket,
        readyState: 1,
      };
      MockWebSocket.mockImplementationOnce(() => mockWs);

      wsClient.disconnect();

      expect(mockWs.close).toHaveBeenCalledWith(1000, 'Client disconnect');
      expect(wsClient.connected).toBe(false);
    });

    it('should handle disconnect when not connected', () => {
      wsClient.disconnect();
      expect(wsClient.connected).toBe(false);
    });
  });

  describe('send', () => {
    it('should send message when connected', () => {
      const mockWs = {
        ...mockWebSocket,
        readyState: 1,
      };
      MockWebSocket.mockImplementationOnce(() => mockWs);

      // Connect first
      (wsClient as any).isConnected = true;
      wsClient['ws'] = mockWs as any;

      const message = { type: 'ping', timestamp: Date.now() };
      wsClient.send(message);

      expect(mockWs.send).toHaveBeenCalledWith(JSON.stringify(message));
    });

    it('should throw error when not connected', () => {
      const message = { type: 'ping' };

      expect(() => wsClient.send(message)).toThrow(NetworkError);
      expect(() => wsClient.send(message)).toThrow('WebSocket not connected');
    });

    it('should handle send error', () => {
      const mockWs = {
        ...mockWebSocket,
        readyState: 1,
        send: jest.fn().mockImplementation(() => {
          throw new Error('Send failed');
        }),
      };
      MockWebSocket.mockImplementationOnce(() => mockWs);

      (wsClient as any).isConnected = true;
      wsClient['ws'] = mockWs as any;

      const message = { type: 'ping' };

      expect(() => wsClient.send(message)).toThrow(NetworkError);
      expect(() => wsClient.send(message)).toThrow('Failed to send message');
    });
  });

  describe('event handling', () => {
    it('should handle open event', async () => {
      const mockWs = {
        ...mockWebSocket,
        readyState: 1,
      };
      MockWebSocket.mockImplementationOnce(() => mockWs);

      const onOpen = jest.fn();
      wsClient.on('connected', onOpen);

      setTimeout(() => {
        const openHandler = mockWs.on.mock.calls.find(call => call[0] === 'open')?.[1];
        if (openHandler) {
          openHandler();
        }
      }, 0);

      await wsClient.connect();

      expect(onOpen).toHaveBeenCalled();
    });

    it('should handle message event', async () => {
      const mockWs = {
        ...mockWebSocket,
        readyState: 1,
      };
      MockWebSocket.mockImplementationOnce(() => mockWs);

      const onMessage = jest.fn();
      wsClient.on('message', onMessage);

      setTimeout(() => {
        const openHandler = mockWs.on.mock.calls.find(call => call[0] === 'open')?.[1];
        if (openHandler) {
          openHandler();
        }
      }, 0);

      await wsClient.connect();

      // Simulate message
      const messageData = { type: 'pong', timestamp: Date.now() };
      const messageEvent = {
        data: JSON.stringify(messageData),
      };

      mockWs.addEventListener.mock.calls
        .find(call => call[0] === 'message')?.[1](messageEvent);

      expect(onMessage).toHaveBeenCalledWith(messageData);
    });

    it('should handle invalid JSON message', async () => {
      const mockWs = {
        ...mockWebSocket,
        readyState: 1,
      };
      MockWebSocket.mockImplementationOnce(() => mockWs);

      const onError = jest.fn();
      wsClient.on('error', onError);

      setTimeout(() => {
        const openHandler = mockWs.on.mock.calls.find(call => call[0] === 'open')?.[1];
        if (openHandler) {
          openHandler();
        }
      }, 0);

      await wsClient.connect();

      // Simulate invalid JSON message
      const messageEvent = {
        data: 'invalid json',
      };

      mockWs.addEventListener.mock.calls
        .find(call => call[0] === 'message')?.[1](messageEvent);

      expect(onError).toHaveBeenCalledWith(expect.any(Error));
    });

    it('should handle close event', async () => {
      const mockWs = {
        ...mockWebSocket,
        readyState: 1,
      };
      MockWebSocket.mockImplementationOnce(() => mockWs);

      const onClose = jest.fn();
      wsClient.on('disconnected', onClose);

      setTimeout(() => {
        const openHandler = mockWs.on.mock.calls.find(call => call[0] === 'open')?.[1];
        if (openHandler) {
          openHandler();
        }
      }, 0);

      await wsClient.connect();

      // Simulate close
      const closeEvent = {
        code: 1000,
        reason: 'Normal closure',
      };

      mockWs.addEventListener.mock.calls
        .find(call => call[0] === 'close')?.[1](closeEvent);

      expect(onClose).toHaveBeenCalledWith(closeEvent);
      expect(wsClient.connected).toBe(false);
    });

    it('should handle error event', async () => {
      const mockWs = {
        ...mockWebSocket,
        readyState: 1,
      };
      MockWebSocket.mockImplementationOnce(() => mockWs);

      const onError = jest.fn();
      wsClient.on('error', onError);

      setTimeout(() => {
        const openHandler = mockWs.on.mock.calls.find(call => call[0] === 'open')?.[1];
        if (openHandler) {
          openHandler();
        }
      }, 0);

      await wsClient.connect();

      // Simulate error
      const errorEvent = {
        error: new Error('WebSocket error'),
        message: 'Connection error',
      };

      mockWs.addEventListener.mock.calls
        .find(call => call[0] === 'error')?.[1](errorEvent);

      expect(onError).toHaveBeenCalledWith(expect.any(NetworkError));
    });
  });

  describe('reconnection', () => {
    it('should attempt reconnection on close', async () => {
      const mockWs = {
        ...mockWebSocket,
        readyState: 1,
      };
      (global as any).WebSocket.mockImplementation(() => mockWs);

      // First connection
      setTimeout(() => {
        const openHandler = mockWs.on.mock.calls.find(call => call[0] === 'open')?.[1];
        if (openHandler) {
          openHandler();
        }
      }, 0);

      await wsClient.connect();
      expect(wsClient.connected).toBe(true);

      // Simulate close with non-normal code
      const closeEvent = {
        code: 1006, // Abnormal closure
        reason: 'Connection lost',
      };

      mockWs.addEventListener.mock.calls
        .find(call => call[0] === 'close')?.[1](closeEvent);

      expect(wsClient.connected).toBe(false);

      // Should attempt reconnection
      await new Promise(resolve => setTimeout(resolve, 100));
      expect(global.WebSocket).toHaveBeenCalledTimes(2);
    });

    it('should not reconnect on normal closure', async () => {
      const mockWs = {
        ...mockWebSocket,
        readyState: 1,
      };
      (global as any).WebSocket.mockImplementation(() => mockWs);

      setTimeout(() => {
        const openHandler = mockWs.on.mock.calls.find(call => call[0] === 'open')?.[1];
        if (openHandler) {
          openHandler();
        }
      }, 0);

      await wsClient.connect();

      // Simulate normal close
      const closeEvent = {
        code: 1000, // Normal closure
        reason: 'Normal closure',
      };

      mockWs.addEventListener.mock.calls
        .find(call => call[0] === 'close')?.[1](closeEvent);

      // Should not attempt reconnection
      await new Promise(resolve => setTimeout(resolve, 100));
      expect(MockWebSocket).toHaveBeenCalledTimes(1);
    });

    it('should stop reconnecting after max attempts', async () => {
      const client = new WebSocketClient({
        url: 'ws://localhost:15001/ws',
        maxReconnectAttempts: 2,
        reconnectInterval: 10,
      });

      const mockWs = {
        ...mockWebSocket,
        readyState: 1,
      };
      (global as any).WebSocket.mockImplementation(() => mockWs);

      // First connection
      setTimeout(() => {
        const openHandler = mockWs.on.mock.calls.find(call => call[0] === 'open')?.[1];
        if (openHandler) {
          openHandler();
        }
      }, 0);

      await client.connect();

      // Simulate multiple closes
      for (let i = 0; i < 3; i++) {
        const closeEvent = {
          code: 1006,
          reason: 'Connection lost',
        };

        mockWs.addEventListener.mock.calls
          .find(call => call[0] === 'close')?.[1](closeEvent);

        await new Promise(resolve => setTimeout(resolve, 50));
      }

      // Should have attempted reconnection maxReconnectAttempts times
      expect(MockWebSocket).toHaveBeenCalledTimes(3); // 1 initial + 2 reconnects
    });
  });

  describe('connection status', () => {
    it('should return correct connection status', () => {
      expect(wsClient.connected).toBe(false);

      (wsClient as any).isConnected = true;
      expect(wsClient.connected).toBe(true);

      (wsClient as any).isConnected = false;
      expect(wsClient.connected).toBe(false);
    });
  });
});
