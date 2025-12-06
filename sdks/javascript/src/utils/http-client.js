/**
 * HTTP client utility for making API requests using native fetch.
 */

import {
  NetworkError,
  ServerError,
  AuthenticationError,
  RateLimitError,
} from '../exceptions/index.js';

export class HttpClient {
  constructor(config = {}) {
    this.config = {
      timeout: 30000,
      ...config,
    };
  }

  /**
   * Make a GET request.
   */
  async get(url, requestConfig = {}) {
    const response = await this.request(url, {
      method: 'GET',
      ...requestConfig,
    });
    return response;
  }

  /**
   * Make a POST request.
   */
  async post(url, data, requestConfig = {}) {
    const response = await this.request(url, {
      method: 'POST',
      body: data ? JSON.stringify(data) : undefined,
      ...requestConfig,
    });
    return response;
  }

  /**
   * Make a PUT request.
   */
  async put(url, data, requestConfig = {}) {
    const response = await this.request(url, {
      method: 'PUT',
      body: data ? JSON.stringify(data) : undefined,
      ...requestConfig,
    });
    return response;
  }

  /**
   * Make a DELETE request.
   */
  async delete(url, requestConfig = {}) {
    const response = await this.request(url, {
      method: 'DELETE',
      ...requestConfig,
    });
    return response;
  }

  /**
   * Make a POST request with FormData (for file uploads).
   * @param {string} url - URL to post to
   * @param {FormData} formData - FormData object containing the file and other fields
   * @param {Object} requestConfig - Additional request configuration
   * @returns {Promise<Object>} Response data
   */
  async postFormData(url, formData, requestConfig = {}) {
    const response = await this.requestFormData(url, formData, requestConfig);
    return response;
  }

  /**
   * Make a generic HTTP request.
   */
  async request(url, options = {}) {
    const fullUrl = url.startsWith('http') ? url : `${this.config.baseURL}${url}`;
    
    const headers = {
      'Content-Type': 'application/json',
      ...this.config.headers,
      ...options.headers,
    };

    if (this.config.apiKey) {
      headers['Authorization'] = `Bearer ${this.config.apiKey}`;
    }

    // const controller = new AbortController();
    // const timeout = setTimeout(() => {
    //   controller.abort();
    // }, options.timeout || this.config.timeout);

    try {
      const response = await fetch(fullUrl, {
        ...options,
        headers,
        // signal: controller.signal,
      });

      // clearTimeout(timeout);

      if (response && !response.ok) {
        throw this.handleError(response);
      }

      if (!response) {
        throw new NetworkError('No response received');
      }

      const contentType = response.headers?.get('content-type');
      if (contentType && contentType.includes('application/json')) {
        return await response.json();
      }

      return await response.text();
    } catch (error) {
      // clearTimeout(timeout);

      // If it's already a VectorizerError (from handleError), re-throw it
      if (error instanceof Error && error.name && error.name.includes('Error') && error.constructor.name !== 'Error') {
        throw error;
      }

      if (error instanceof Error) {
        // if (error.name === 'AbortError') {
        //   throw new TimeoutError('Request timeout');
        // }
        throw new NetworkError(error.message);
      }

      throw new NetworkError('Unknown network error');
    }
  }

  /**
   * Make a FormData HTTP request (for file uploads).
   * Note: Content-Type header is not set to allow the browser to set it with boundary.
   * @param {string} url - URL to post to
   * @param {FormData} formData - FormData object
   * @param {Object} options - Request options
   * @returns {Promise<Object>} Response data
   */
  async requestFormData(url, formData, options = {}) {
    const fullUrl = url.startsWith('http') ? url : `${this.config.baseURL}${url}`;

    // Don't set Content-Type for FormData - let the browser set it with boundary
    const headers = {};

    if (this.config.headers) {
      Object.assign(headers, this.config.headers);
    }

    if (options.headers) {
      Object.assign(headers, options.headers);
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

      if (response && !response.ok) {
        throw this.handleError(response);
      }

      if (!response) {
        throw new NetworkError('No response received');
      }

      const contentType = response.headers?.get('content-type');
      if (contentType && contentType.includes('application/json')) {
        return await response.json();
      }

      return await response.text();
    } catch (error) {
      // If it's already a VectorizerError (from handleError), re-throw it
      if (error instanceof Error && error.name && error.name.includes('Error') && error.constructor.name !== 'Error') {
        throw error;
      }

      if (error instanceof Error) {
        throw new NetworkError(error.message);
      }

      throw new NetworkError('Unknown network error');
    }
  }

  /**
   * Handle HTTP errors and convert them to appropriate exceptions.
   */
  handleError(response) {
    const message = `HTTP ${response.status}: ${response.statusText}`;

    switch (response.status) {
      case 401:
        return new AuthenticationError(message);
      case 403:
        return new AuthenticationError('Access forbidden');
      case 404:
        return new ServerError('Resource not found');
      case 429:
        return new RateLimitError('Rate limit exceeded');
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
