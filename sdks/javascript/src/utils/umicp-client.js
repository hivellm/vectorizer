/**
 * UMICP client utility using the official @hivellm/umicp SDK.
 * 
 * Wrapper around UMICPWebSocketClient for Vectorizer API requests.
 */

import { StreamableHTTPClient } from '@hivellm/umicp';
import {
  NetworkError,
  ServerError,
  AuthenticationError,
} from '../exceptions/index.js';

export class UMICPClient {
  constructor(config = {}) {
    this.config = {
      host: config.host || 'localhost',
      port: config.port || 15003,
      apiKey: config.apiKey,
      timeout: config.timeout || 30000,
    };

    this.client = null;
    this.connected = false;
  }

  /**
   * Connect to the UMICP server.
   */
  async connect() {
    if (this.connected && this.client) {
      return;
    }

    try {
      const url = `http://${this.config.host}:${this.config.port}`;

      this.client = new StreamableHTTPClient(url, {
        timeout: this.config.timeout,
      });

      this.connected = true;
    } catch (error) {
      throw new NetworkError(`Failed to connect to UMICP server: ${error.message}`);
    }
  }

  /**
   * Disconnect from the UMICP server.
   */
  async disconnect() {
    if (this.client && this.client.close) {
      await this.client.close();
    }
    this.connected = false;
    this.client = null;
  }

  /**
   * Check if connected to the UMICP server.
   */
  isConnected() {
    return this.connected && this.client !== null;
  }

  /**
   * Make a request via UMICP.
   */
  async request(method, path, data, requestConfig = {}) {
    if (!this.isConnected()) {
      await this.connect();
    }

    try {
      // Use StreamableHTTPClient's request method
      const response = await this.client.request(path, {
        method,
        body: data ? JSON.stringify(data) : undefined,
        headers: {
          'Content-Type': 'application/json',
          ...(this.config.apiKey && { 'Authorization': `Bearer ${this.config.apiKey}` }),
          ...(requestConfig.headers || {}),
        },
      });

      return response;
    } catch (error) {
      if (error instanceof ServerError || error instanceof AuthenticationError) {
        throw error;
      }

      if (error instanceof Error) {
        throw new NetworkError(`UMICP request failed: ${error.message}`);
      }

      throw new NetworkError('Unknown UMICP error');
    }
  }

  /**
   * Make a GET request.
   */
  async get(url, requestConfig) {
    return this.request('GET', url, undefined, requestConfig);
  }

  /**
   * Make a POST request.
   */
  async post(url, data, requestConfig) {
    return this.request('POST', url, data, requestConfig);
  }

  /**
   * Make a PUT request.
   */
  async put(url, data, requestConfig) {
    return this.request('PUT', url, data, requestConfig);
  }

  /**
   * Make a DELETE request.
   */
  async delete(url, requestConfig) {
    return this.request('DELETE', url, undefined, requestConfig);
  }

  /**
   * Handle errors and convert them to appropriate exceptions.
   */
  handleError(response) {
    const message = response.message || `UMICP Error ${response.statusCode || 'Unknown'}`;

    const statusCode = response.statusCode || 500;

    switch (statusCode) {
      case 401:
        return new AuthenticationError(message);
      case 403:
        return new AuthenticationError('Access forbidden');
      case 404:
        return new ServerError('Resource not found');
      case 429:
        return new ServerError('Rate limit exceeded');
      case 500:
      case 502:
      case 503:
      case 504:
        return new ServerError(message);
      default:
        return new ServerError(message);
    }
  }
}

