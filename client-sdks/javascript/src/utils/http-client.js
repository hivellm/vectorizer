/**
 * HTTP client utility for making API requests using native fetch.
 */

import {
  NetworkError,
  ServerError,
  AuthenticationError,
  TimeoutError,
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

    const controller = new AbortController();
    const timeout = setTimeout(() => {
      controller.abort();
    }, options.timeout || this.config.timeout);

    try {
      const response = await fetch(fullUrl, {
        ...options,
        headers,
        signal: controller.signal,
      });

      clearTimeout(timeout);

      if (!response.ok) {
        throw await this.handleError(response);
      }

      const contentType = response.headers.get('content-type');
      if (contentType && contentType.includes('application/json')) {
        return await response.json();
      }

      return await response.text();
    } catch (error) {
      clearTimeout(timeout);
      
      if (error instanceof Error) {
        if (error.name === 'AbortError') {
          throw new TimeoutError('Request timeout');
        }
        throw new NetworkError(error.message);
      }
      
      throw new NetworkError('Unknown network error');
    }
  }

  /**
   * Handle HTTP errors and convert them to appropriate exceptions.
   */
  async handleError(response) {
    let message = `HTTP ${response.status}: ${response.statusText}`;
    
    try {
      const errorData = await response.json();
      message = errorData.message || message;
    } catch {
      // Ignore JSON parsing errors, use default message
    }

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
