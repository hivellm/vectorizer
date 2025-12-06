/**
 * Transport abstraction layer for Vectorizer client.
 * 
 * Supports multiple transport protocols:
 * - HTTP/HTTPS (default)
 * - UMICP (Universal Messaging and Inter-process Communication Protocol)
 */

import { HttpClient, HttpClientConfig } from './http-client';
import { UMICPClient, UMICPClientConfig } from './umicp-client';

export type TransportProtocol = 'http' | 'umicp';

export interface TransportConfig {
  protocol?: TransportProtocol;
  http?: HttpClientConfig;
  umicp?: UMICPClientConfig;
}

export interface RequestConfig {
  headers?: Record<string, string>;
  timeout?: number;
  params?: Record<string, any>;
}

/**
 * Transport interface that both HTTP and UMICP clients implement.
 */
export interface ITransport {
  get<T = unknown>(url: string, config?: RequestConfig): Promise<T>;
  post<T = unknown>(url: string, data?: unknown, config?: RequestConfig): Promise<T>;
  put<T = unknown>(url: string, data?: unknown, config?: RequestConfig): Promise<T>;
  delete<T = unknown>(url: string, config?: RequestConfig): Promise<T>;
  postFormData<T = unknown>(url: string, formData: FormData, config?: RequestConfig): Promise<T>;
}

/**
 * Transport factory that creates the appropriate client based on protocol.
 */
export class TransportFactory {
  /**
   * Create a transport client based on configuration.
   */
  static create(config: TransportConfig): ITransport {
    const protocol = config.protocol || 'http';

    switch (protocol) {
      case 'http':
        if (!config.http) {
          throw new Error('HTTP configuration is required when using HTTP protocol');
        }
        return new HttpClient(config.http);

      case 'umicp':
        if (!config.umicp) {
          throw new Error('UMICP configuration is required when using UMICP protocol');
        }
        const umicpClient = new UMICPClient(config.umicp);
        return umicpClient;

      default:
        throw new Error(`Unsupported protocol: ${protocol}`);
    }
  }
}

/**
 * Helper function to parse a connection string into transport config.
 * 
 * Examples:
 * - "http://localhost:15002" -> HTTP transport
 * - "https://api.example.com" -> HTTPS transport
 * - "umicp://localhost:15003" -> UMICP transport
 */
export function parseConnectionString(connectionString: string, apiKey?: string): TransportConfig {
  const url = new URL(connectionString);

  switch (url.protocol) {
    case 'http:':
    case 'https:':
      return {
        protocol: 'http',
        http: {
          baseURL: `${url.protocol}//${url.host}`,
          ...(apiKey && { apiKey }),
        },
      };

    case 'umicp:':
      return {
        protocol: 'umicp',
        umicp: {
          host: url.hostname,
          port: parseInt(url.port || '15003', 10),
          ...(apiKey && { apiKey }),
        },
      };

    default:
      throw new Error(`Unsupported protocol in connection string: ${url.protocol}`);
  }
}

