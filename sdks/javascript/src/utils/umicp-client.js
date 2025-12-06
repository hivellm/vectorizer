/**
 * UMICP client utility using the official @hivellm/umicp SDK.
 * 
 * Wrapper around UMICPWebSocketClient for Vectorizer API requests.
 * 
 * Note: @hivellm/umicp is an optional dependency. If it's not installed,
 * this module will fail to load, which is handled gracefully by the transport layer.
 */

import {
  NetworkError,
  ServerError,
  AuthenticationError,
} from '../exceptions/index.js';

// Lazy load UMICP module - it's an optional dependency
let StreamableHTTPClient = null;

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

    // Try to load UMICP module if not already loaded
    if (!StreamableHTTPClient) {
      try {
        const umicpModule = await import('@hivellm/umicp');
        StreamableHTTPClient = umicpModule.StreamableHTTPClient;
      } catch (error) {
        throw new Error(
          '@hivellm/umicp is not installed. Install it with: npm install @hivellm/umicp'
        );
      }
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
   * Make a POST request with FormData (for file uploads).
   * @param {string} url - URL to post to
   * @param {FormData} formData - FormData object containing the file and other fields
   * @param {Object} requestConfig - Additional request configuration
   * @returns {Promise<Object>} Response data
   */
  async postFormData(url, formData, requestConfig = {}) {
    if (!this.isConnected()) {
      await this.connect();
    }

    const fullUrl = `http://${this.config.host}:${this.config.port}${url}`;

    // Don't set Content-Type for FormData - let the browser set it with boundary
    const headers = {
      'X-UMICP-Protocol': 'true',
    };

    if (requestConfig.headers) {
      Object.assign(headers, requestConfig.headers);
    }

    // Remove Content-Type if set - browser needs to set it for multipart/form-data
    delete headers['Content-Type'];

    if (this.config.apiKey) {
      headers['Authorization'] = `Bearer ${this.config.apiKey}`;
    }

    try {
      const response = await fetch(fullUrl, {
        method: 'POST',
        headers,
        body: formData,
      });

      if (!response.ok) {
        const error = this.handleError({ statusCode: response.status, message: response.statusText });
        throw error;
      }

      const contentType = response.headers?.get('content-type');
      if (contentType && contentType.includes('application/json')) {
        return await response.json();
      }

      return await response.text();
    } catch (error) {
      if (error instanceof ServerError || error instanceof AuthenticationError) {
        throw error;
      }

      if (error instanceof Error) {
        throw new NetworkError(`UMICP FormData request failed: ${error.message}`);
      }

      throw new NetworkError('Unknown UMICP error');
    }
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
