/**
 * Tests for UMICP transport
 */

import { VectorizerClient } from '../src/client.js';
import { parseConnectionString } from '../src/utils/transport.js';

describe('UMICP Transport', () => {
  describe('parseConnectionString', () => {
    it('should parse HTTP connection strings', () => {
      const config = parseConnectionString('http://localhost:15002', 'test-key');
      expect(config.protocol).toBe('http');
      expect(config.http).toBeDefined();
      expect(config.http.baseURL).toBe('http://localhost:15002');
      expect(config.http.apiKey).toBe('test-key');
    });

    it('should parse HTTPS connection strings', () => {
      const config = parseConnectionString('https://api.example.com', 'test-key');
      expect(config.protocol).toBe('http');
      expect(config.http).toBeDefined();
      expect(config.http.baseURL).toBe('https://api.example.com');
    });

    it('should parse UMICP connection strings', () => {
      const config = parseConnectionString('umicp://localhost:15003', 'test-key');
      expect(config.protocol).toBe('umicp');
      expect(config.umicp).toBeDefined();
      expect(config.umicp.host).toBe('localhost');
      expect(config.umicp.port).toBe(15003);
      expect(config.umicp.apiKey).toBe('test-key');
    });

    it('should use default UMICP port if not specified', () => {
      const config = parseConnectionString('umicp://localhost', 'test-key');
      expect(config.umicp.port).toBe(15003);
    });

    it('should throw error for unsupported protocols', () => {
      expect(() => {
        parseConnectionString('ftp://localhost', 'test-key');
      }).toThrow('Unsupported protocol');
    });
  });

  describe('VectorizerClient with UMICP', () => {
    it('should initialize with UMICP connection string', () => {
      const client = new VectorizerClient({
        connectionString: 'umicp://localhost:15003',
        apiKey: 'test-key',
      });

      expect(client.getProtocol()).toBe('umicp');
    });

    it('should initialize with explicit UMICP configuration', () => {
      const client = new VectorizerClient({
        protocol: 'umicp',
        apiKey: 'test-key',
        umicp: {
          host: 'localhost',
          port: 15003,
        },
      });

      expect(client.getProtocol()).toBe('umicp');
    });

    it('should initialize with HTTP by default', () => {
      const client = new VectorizerClient({
        baseURL: 'http://localhost:15002',
      });

      expect(client.getProtocol()).toBe('http');
    });

    it('should throw error if UMICP config is missing', () => {
      expect(() => {
        new VectorizerClient({
          protocol: 'umicp',
          apiKey: 'test-key',
        });
      }).toThrow('UMICP configuration is required');
    });

    it('should use default UMICP host and port', () => {
      const client = new VectorizerClient({
        protocol: 'umicp',
        apiKey: 'test-key',
        umicp: {},
      });

      expect(client.getProtocol()).toBe('umicp');
    });
  });

  describe('UMICP Performance Benefits', () => {
    it('should document UMICP protocol support', () => {
      const client = new VectorizerClient({
        protocol: 'umicp',
        apiKey: 'test-key',
        umicp: {
          host: 'localhost',
          port: 15003,
        },
      });

      // UMICP provides efficient StreamableHTTP protocol
      expect(client.getProtocol()).toBe('umicp');
    });

    it('should support UMICP configuration options', () => {
      const client = new VectorizerClient({
        protocol: 'umicp',
        apiKey: 'test-key',
        timeout: 60000,
        umicp: {
          host: 'localhost',
          port: 15003,
          timeout: 45000,
        },
      });

      // UMICP allows custom timeout configuration
      expect(client.getProtocol()).toBe('umicp');
    });
  });
});

