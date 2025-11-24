/**
 * API Client for Vectorizer REST API
 * Handles HTTP requests to the Vectorizer server
 */

import { getApiUrl } from '@/config/env';
import { MiddlewareManager, MiddlewareContext, createDefaultMiddlewareStack } from './api-middleware';

export type ApiError = {
  message: string;
  code?: string;
  details?: unknown;
};

export class ApiClientError extends Error {
  constructor(
    message: string,
    public status?: number,
    public code?: string,
    public details?: unknown,
  ) {
    super(message);
    this.name = 'ApiClientError';
  }
}

/**
 * API Response wrapper
 */
export type ApiResponse<T> = {
  data: T;
  error?: ApiError;
};

/**
 * Request options
 */
export type RequestOptions = RequestInit & {
  params?: Record<string, string | number | boolean>;
};

/**
 * API Client class
 */
export class ApiClient {
  private baseUrl: string;
  private middlewareManager: MiddlewareManager;

  constructor(baseUrl?: string, middlewareManager?: MiddlewareManager) {
    // If baseUrl is empty string or undefined, use same origin (like dashboard v1)
    // Otherwise use the provided baseUrl or default from env
    if (baseUrl === '') {
      this.baseUrl = '';
    } else {
      this.baseUrl = baseUrl || getApiUrl('');
    }
    this.middlewareManager = middlewareManager || createDefaultMiddlewareStack({
      enableLogging: import.meta.env.DEV,
      enableRetry: true,
      retryCount: 3,
      timeout: 30000,
    });
  }

  /**
   * Get middleware manager
   */
  getMiddlewareManager(): MiddlewareManager {
    return this.middlewareManager;
  }

  /**
   * Build URL with query parameters
   */
  private buildUrl(endpoint: string, params?: Record<string, string | number | boolean>): string {
    // If endpoint is already a full URL, use it as-is
    if (endpoint.startsWith('http')) {
      const url = new URL(endpoint);
      if (params) {
        Object.entries(params).forEach(([key, value]) => {
          if (value !== undefined && value !== null) {
            url.searchParams.append(key, String(value));
          }
        });
      }
      return url.toString();
    }
    
    // If baseUrl is empty, use same origin (like dashboard v1)
    const base = this.baseUrl || (typeof window !== 'undefined' ? window.location.origin : 'http://127.0.0.1:15002');
    const url = `${base}${endpoint}`;
    
    if (!params || Object.keys(params).length === 0) {
      return url;
    }

    const searchParams = new URLSearchParams();
    Object.entries(params).forEach(([key, value]) => {
      if (value !== undefined && value !== null) {
        searchParams.append(key, String(value));
      }
    });

    const queryString = searchParams.toString();
    return queryString ? `${url}?${queryString}` : url;
  }


  /**
   * GET request
   */
  async get<T>(endpoint: string, options?: RequestOptions): Promise<T> {
    const url = this.buildUrl(endpoint, options?.params);
    console.log('[ApiClient] GET request:', { endpoint, url, baseUrl: this.baseUrl });
    
    const context: MiddlewareContext = {
      url,
      method: 'GET',
      headers: {
        'Content-Type': 'application/json',
        ...options?.headers,
      },
      params: options?.params,
    };

    console.log('[ApiClient] Executing middleware with context:', context);

    // Execute middleware stack
    const result = await this.middlewareManager.execute(context);

    console.log('[ApiClient] Middleware result:', { 
      hasError: !!result.error, 
      hasData: result.data !== undefined,
      responseStatus: result.response?.status,
      error: result.error?.message 
    });

    if (result.error) {
      console.error('[ApiClient] Error from middleware:', result.error);
      throw result.error;
    }

    if (result.data === undefined) {
      console.warn('[ApiClient] No data received, response status:', result.response?.status);
    }

    return (result.data !== undefined ? result.data : undefined) as T;
  }

  /**
   * POST request
   */
  async post<T>(endpoint: string, data?: unknown, options?: RequestOptions): Promise<T> {
    const url = this.buildUrl(endpoint, options?.params);
    
    const context: MiddlewareContext = {
      url,
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
        ...options?.headers,
      },
      body: data,
      params: options?.params,
    };

    // Execute middleware stack
    const result = await this.middlewareManager.execute(context);

    if (result.error) {
      throw result.error;
    }

    return (result.data !== undefined ? result.data : undefined) as T;
  }

  /**
   * PUT request
   */
  async put<T>(endpoint: string, data?: unknown, options?: RequestOptions): Promise<T> {
    const url = this.buildUrl(endpoint, options?.params);
    
    const context: MiddlewareContext = {
      url,
      method: 'PUT',
      headers: {
        'Content-Type': 'application/json',
        ...options?.headers,
      },
      body: data,
      params: options?.params,
    };

    // Execute middleware stack
    const result = await this.middlewareManager.execute(context);

    if (result.error) {
      throw result.error;
    }

    return (result.data !== undefined ? result.data : undefined) as T;
  }

  /**
   * DELETE request
   */
  async delete<T>(endpoint: string, options?: RequestOptions): Promise<T> {
    const url = this.buildUrl(endpoint, options?.params);
    
    const context: MiddlewareContext = {
      url,
      method: 'DELETE',
      headers: {
        'Content-Type': 'application/json',
        ...options?.headers,
      },
      params: options?.params,
    };

    // Execute middleware stack
    const result = await this.middlewareManager.execute(context);

    if (result.error) {
      throw result.error;
    }

    return (result.data !== undefined ? result.data : undefined) as T;
  }
}

/**
 * Default API client instance
 */
export const apiClient = new ApiClient();

