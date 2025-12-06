/**
 * UMICP client utility for making API requests using the UMICP protocol.
 * 
 * This is a simplified wrapper around the UMICP WebSocket transport.
 * For production use, consider implementing a full UMICP HTTP/2 or custom transport.
 */

import { NetworkError, ServerError, AuthenticationError, TimeoutError } from '../exceptions';

export interface UMICPClientConfig {
  host: string;
  port: number;
  apiKey?: string;
  timeout?: number;
}

export interface RequestConfig {
  headers?: Record<string, string>;
  timeout?: number;
  params?: Record<string, any>;
}

/**
 * UMICPClient - A simplified client that uses standard HTTP with
 * UMICP protocol headers for compatibility.
 * 
 * Note: This is a transitional implementation. Full UMICP protocol
 * support requires the server to implement UMICP WebSocket or HTTP/2 endpoints.
 */
export class UMICPClient {
  private config: Required<UMICPClientConfig>;
  private connected: boolean = false;

  constructor(config: UMICPClientConfig) {
    this.config = {
      host: config.host,
      port: config.port,
      apiKey: config.apiKey || '',
      timeout: config.timeout || 30000,
    };
  }

  /**
   * Connect to the UMICP server.
   */
  public async connect(): Promise<void> {
    // For now, this is a no-op since we're using HTTP
    // In a full implementation, this would establish a WebSocket/HTTP2 connection
    this.connected = true;
  }

  /**
   * Disconnect from the UMICP server.
   */
  public async disconnect(): Promise<void> {
    this.connected = false;
  }

  /**
   * Check if connected to the UMICP server.
   */
  public isConnected(): boolean {
    return this.connected;
  }

  /**
   * Make a request via UMICP-compatible HTTP.
   */
  public async request<T = unknown>(
    method: string,
    path: string,
    data?: unknown,
    requestConfig?: RequestConfig
  ): Promise<T> {
    if (!this.isConnected()) {
      await this.connect();
    }

    const url = `http://${this.config.host}:${this.config.port}${path}`;
    
    const headers: Record<string, string> = {
      'Content-Type': 'application/json',
      'X-UMICP-Protocol': 'true',
    };

    if (requestConfig?.headers) {
      Object.assign(headers, requestConfig.headers);
    }

    if (this.config.apiKey) {
      headers['Authorization'] = `Bearer ${this.config.apiKey}`;
    }

    const controller = new AbortController();
    const timeout = setTimeout(() => {
      controller.abort();
    }, requestConfig?.timeout || this.config.timeout);

    try {
      const fetchOptions: RequestInit = {
        method,
        headers,
        signal: controller.signal,
      };

      if (data) {
        fetchOptions.body = JSON.stringify(data);
      }

      const response = await fetch(url, fetchOptions);

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
      
      if (error instanceof ServerError || 
          error instanceof AuthenticationError || 
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
   * Make a GET request.
   */
  public async get<T = unknown>(url: string, requestConfig?: RequestConfig): Promise<T> {
    return this.request<T>('GET', url, undefined, requestConfig);
  }

  /**
   * Make a POST request.
   */
  public async post<T = unknown>(url: string, data?: unknown, requestConfig?: RequestConfig): Promise<T> {
    return this.request<T>('POST', url, data, requestConfig);
  }

  /**
   * Make a PUT request.
   */
  public async put<T = unknown>(url: string, data?: unknown, requestConfig?: RequestConfig): Promise<T> {
    return this.request<T>('PUT', url, data, requestConfig);
  }

  /**
   * Make a DELETE request.
   */
  public async delete<T = unknown>(url: string, requestConfig?: RequestConfig): Promise<T> {
    return this.request<T>('DELETE', url, undefined, requestConfig);
  }

  /**
   * Make a POST request with FormData (for file uploads).
   */
  public async postFormData<T = unknown>(url: string, formData: FormData, requestConfig?: RequestConfig): Promise<T> {
    if (!this.isConnected()) {
      await this.connect();
    }

    const fullUrl = `http://${this.config.host}:${this.config.port}${url}`;

    // Don't set Content-Type for FormData - let the browser set it with boundary
    const headers: Record<string, string> = {
      'X-UMICP-Protocol': 'true',
    };

    if (requestConfig?.headers) {
      Object.assign(headers, requestConfig.headers);
    }

    // Remove Content-Type if set - browser needs to set it for multipart/form-data
    delete headers['Content-Type'];

    if (this.config.apiKey) {
      headers['Authorization'] = `Bearer ${this.config.apiKey}`;
    }

    const controller = new AbortController();
    const timeout = setTimeout(() => {
      controller.abort();
    }, requestConfig?.timeout || this.config.timeout);

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

      if (error instanceof ServerError ||
          error instanceof AuthenticationError ||
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
   * Handle errors and convert them to appropriate exceptions.
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
