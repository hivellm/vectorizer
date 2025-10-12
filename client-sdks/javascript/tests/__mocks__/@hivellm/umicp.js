// Mock UMICP module for Jest tests
// Since UMICP uses import.meta which Jest doesn't support in CJS mode

export class StreamableHTTPClient {
  constructor(url, config = {}) {
    this.url = url;
    this.config = config;
  }

  async request(path, options = {}) {
    // Mock implementation for testing
    return { success: true, data: 'mocked response' };
  }

  async close() {
    // Mock close
  }
}

export class UMICPWebSocketClient {
  constructor(url, options = {}) {
    this.url = url;
    this.options = options;
  }

  on(event, handler) {
    // Mock event handler
  }

  once(event, handler) {
    // Mock once handler
  }

  async send(data) {
    // Mock send
  }

  async close() {
    // Mock close
  }
}

export const Envelope = class {
  constructor(options) {
    this.options = options;
  }

  serialize() {
    return JSON.stringify(this.options);
  }
};

export const OperationType = {
  CONTROL: 0,
  DATA: 1,
  ACK: 2,
  ERROR: 3,
};

export const UMICP = {
  version: '0.1.3',
  hasWebSocketTransport: true,
  hasHTTP2Transport: false,
};

export default UMICP;

