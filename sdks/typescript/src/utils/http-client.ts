/**
 * HTTP client utility for making API requests using native fetch.
 */

import { NetworkError, ServerError, AuthenticationError, TimeoutError, RateLimitError } from '../exceptions';

export interface HttpClientConfig {
  baseURL: string;
  timeout?: number;
  apiKey?: string;
  headers?: Record<string, string>;
}

export interface RequestConfig {
  headers?: Record<string, string>;
  timeout?: number;
  params?: Record<string, any>;
}

export class HttpClient {
  private config: HttpClientConfig;

  constructor(config: HttpClientConfig) {
    this.config = {
      timeout: 30000,
      ...config,
    };
  }

  /**
   * Make a GET request.
   */
  public async get<T = unknown>(url: string, requestConfig?: RequestConfig): Promise<T> {
    const response = await this.request<T>(url, {
      method: 'GET',
      ...requestConfig,
    });
    return response;
  }

  /**
   * Make a POST request.
   */
  public async post<T = unknown>(url: string, data?: unknown, requestConfig?: RequestConfig): Promise<T> {
    const requestOptions: RequestInit & RequestConfig = {
      method: 'POST',
      ...requestConfig,
    };
    
    if (data) {
      requestOptions.body = JSON.stringify(data);
    }
    
    const response = await this.request<T>(url, requestOptions);
    return response;
  }

  /**
   * Make a PUT request.
   */
  public async put<T = unknown>(url: string, data?: unknown, requestConfig?: RequestConfig): Promise<T> {
    const requestOptions: RequestInit & RequestConfig = {
      method: 'PUT',
      ...requestConfig,
    };
    
    if (data) {
      requestOptions.body = JSON.stringify(data);
    }
    
    const response = await this.request<T>(url, requestOptions);
    return response;
  }

  /**
   * Make a DELETE request.
   */
  public async delete<T = unknown>(url: string, requestConfig?: RequestConfig): Promise<T> {
    const response = await this.request<T>(url, {
      method: 'DELETE',
      ...requestConfig,
    });
    return response;
  }

  /**
   * Make a POST request with FormData (for file uploads).
   */
  public async postFormData<T = unknown>(url: string, formData: FormData, requestConfig?: RequestConfig): Promise<T> {
    const response = await this.requestFormData<T>(url, formData, requestConfig);
    return response;
  }

  /**
   * Make a generic HTTP request.
   */
  private async request<T = unknown>(url: string, options: RequestInit & RequestConfig): Promise<T> {
    const fullUrl = url.startsWith('http') ? url : `${this.config.baseURL}${url}`;
    
    const headers: Record<string, string> = {
      'Content-Type': 'application/json',
    };
    
    if (this.config.headers) {
      Object.assign(headers, this.config.headers);
    }
    
    if (options.headers) {
      Object.assign(headers, options.headers);
    }

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
        const error = await this.handleError(response);
        throw error;
      }

      const contentType = response.headers.get('content-type');
      if (contentType && contentType.includes('application/json')) {
        return await response.json() as T;
      }

      return (await response.text()) as unknown as T;
    } catch (error) {
      clearTimeout(timeout);
      
      // Re-throw errors that are already our custom exceptions
      if (error instanceof ServerError || 
          error instanceof AuthenticationError || 
          error instanceof RateLimitError || 
          error instanceof TimeoutError) {
        throw error;
      }
      
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
   * Make a FormData HTTP request (for file uploads).
   * Note: Content-Type header is not set to allow the browser to set it with boundary.
   */
  private async requestFormData<T = unknown>(url: string, formData: FormData, options?: RequestConfig): Promise<T> {
    const fullUrl = url.startsWith('http') ? url : `${this.config.baseURL}${url}`;

    // Don't set Content-Type for FormData - let the browser set it with boundary
    const headers: Record<string, string> = {};

    if (this.config.headers) {
      Object.assign(headers, this.config.headers);
    }

    if (options?.headers) {
      Object.assign(headers, options.headers);
    }

    // Remove Content-Type if set - browser needs to set it for multipart/form-data
    delete headers['Content-Type'];

    if (this.config.apiKey) {
      headers['Authorization'] = `Bearer ${this.config.apiKey}`;
    }

    const controller = new AbortController();
    const timeout = setTimeout(() => {
      controller.abort();
    }, options?.timeout || this.config.timeout);

    try {
      const response = await fetch(fullUrl, {
        method: 'POST',
        headers,
        body: formData,
        signal: controller.signal,
      });

      clearTimeout(timeout);

      if (!response.ok) {
        const error = await this.handleError(response);
        throw error;
      }

      const contentType = response.headers.get('content-type');
      if (contentType && contentType.includes('application/json')) {
        return await response.json() as T;
      }

      return (await response.text()) as unknown as T;
    } catch (error) {
      clearTimeout(timeout);

      // Re-throw errors that are already our custom exceptions
      if (error instanceof ServerError ||
          error instanceof AuthenticationError ||
          error instanceof RateLimitError ||
          error instanceof TimeoutError) {
        throw error;
      }

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
  private async handleError(response: Response): Promise<Error> {
    let message = `HTTP ${response.status}: ${response.statusText}`;
    
    try {
      const errorData = await response.json() as { message?: string };
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
