/**
 * Transport abstraction layer for Vectorizer client.
 * 
 * Supports multiple transport protocols:
 * - HTTP/HTTPS (default)
 * - UMICP (Universal Messaging and Inter-process Communication Protocol)
 * 
 * Note: UMICP requires the optional @hivellm/umicp package to be installed.
 */

import { HttpClient } from './http-client.js';

// Import UMICP client - it's optional and will handle missing @hivellm/umicp gracefully
import { UMICPClient } from './umicp-client.js';

/**
 * Transport factory that creates the appropriate client based on protocol.
 */
export class TransportFactory {
  /**
   * Create a transport client based on configuration.
   * @param {Object} config - Transport configuration
   * @param {string} config.protocol - Protocol type ('http' or 'umicp')
   * @param {Object} [config.http] - HTTP configuration
   * @param {Object} [config.umicp] - UMICP configuration
   * @returns {HttpClient|UMICPClient} Transport client instance
   * @throws {Error} If UMICP is requested but @hivellm/umicp is not installed
   */
  static create(config) {
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
        if (!UMICPClient) {
          throw new Error(
            'UMICP transport requires @hivellm/umicp to be installed. ' +
            'Install it with: npm install @hivellm/umicp'
          );
        }
        return new UMICPClient(config.umicp);

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
 * 
 * @param {string} connectionString - Connection URI
 * @param {string} [apiKey] - Optional API key
 * @returns {Object} Transport configuration
 */
export function parseConnectionString(connectionString, apiKey) {
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

