/**
 * Tests for HttpClient utility.
 */

import { describe, it, expect, beforeEach, vi } from 'vitest';
import { HttpClient } from '../../src/utils/http-client.js';
import {
  NetworkError,
  ServerError,
  AuthenticationError,
  TimeoutError,
  RateLimitError,
} from '../../src/exceptions/index.js';

// Mock fetch
const mockFetch = vi.fn();
global.fetch = mockFetch;

describe('HttpClient', () => {
  let httpClient;

  beforeEach(() => {
    vi.clearAllMocks();
    httpClient = new HttpClient({
      baseURL: 'http://localhost:15002',
      apiKey: 'test-api-key',
      timeout: 5000,
    });
  });

  describe('constructor', () => {
    it('should create client with default config', () => {
      const client = new HttpClient({
        baseURL: 'http://localhost:15002',
      });

      expect(client).toBeInstanceOf(HttpClient);
    });

    it('should create client with custom config', () => {
      const client = new HttpClient({
        baseURL: 'http://custom:8080',
        apiKey: 'custom-key',
        timeout: 10000,
        headers: { 'Custom-Header': 'value' },
      });

      expect(client).toBeInstanceOf(HttpClient);
    });
  });

  describe('GET requests', () => {
    it('should make successful GET request', async () => {
      const mockResponse = { data: 'test' };
      mockFetch.mockResolvedValueOnce({
        ok: true,
        headers: new Headers({ 'content-type': 'application/json' }),
        json: () => Promise.resolve(mockResponse),
      });

      const result = await httpClient.get('/test');

      expect(mockFetch).toHaveBeenCalledWith(
        'http://localhost:15002/test',
        expect.objectContaining({
          method: 'GET',
          headers: expect.objectContaining({
            'Content-Type': 'application/json',
            'Authorization': 'Bearer test-api-key',
          }),
        })
      );
      expect(result).toEqual(mockResponse);
    });

    it('should make GET request without API key', async () => {
      const client = new HttpClient({
        baseURL: 'http://localhost:15002',
      });

      const mockResponse = { data: 'test' };
      mockFetch.mockResolvedValueOnce({
        ok: true,
        headers: new Headers({ 'content-type': 'application/json' }),
        json: () => Promise.resolve(mockResponse),
      });

      await client.get('/test');

      expect(mockFetch).toHaveBeenCalledWith(
        'http://localhost:15002/test',
        expect.objectContaining({
          method: 'GET',
          headers: expect.objectContaining({
            'Content-Type': 'application/json',
          }),
        })
      );
      expect(mockFetch).toHaveBeenCalledWith(
        expect.any(String),
        expect.objectContaining({
          headers: expect.not.objectContaining({
            'Authorization': expect.any(String),
          }),
        })
      );
    });

    it('should handle non-JSON response', async () => {
      const mockResponse = 'plain text response';
      mockFetch.mockResolvedValueOnce({
        ok: true,
        headers: new Headers({ 'content-type': 'text/plain' }),
        text: () => Promise.resolve(mockResponse),
      });

      const result = await httpClient.get('/test');

      expect(result).toBe(mockResponse);
    });
  });

  describe('POST requests', () => {
    it('should make successful POST request', async () => {
      const requestData = { name: 'test' };
      const mockResponse = { id: '123', ...requestData };
      
      mockFetch.mockResolvedValueOnce({
        ok: true,
        headers: new Headers({ 'content-type': 'application/json' }),
        json: () => Promise.resolve(mockResponse),
      });

      const result = await httpClient.post('/test', requestData);

      expect(mockFetch).toHaveBeenCalledWith(
        'http://localhost:15002/test',
        expect.objectContaining({
          method: 'POST',
          body: JSON.stringify(requestData),
          headers: expect.objectContaining({
            'Content-Type': 'application/json',
            'Authorization': 'Bearer test-api-key',
          }),
        })
      );
      expect(result).toEqual(mockResponse);
    });

    it('should make POST request without data', async () => {
      const mockResponse = { success: true };
      
      mockFetch.mockResolvedValueOnce({
        ok: true,
        headers: new Headers({ 'content-type': 'application/json' }),
        json: () => Promise.resolve(mockResponse),
      });

      const result = await httpClient.post('/test');

      expect(mockFetch).toHaveBeenCalledWith(
        'http://localhost:15002/test',
        expect.objectContaining({
          method: 'POST',
          body: undefined,
        })
      );
      expect(result).toEqual(mockResponse);
    });
  });

  describe('PUT requests', () => {
    it('should make successful PUT request', async () => {
      const requestData = { name: 'updated' };
      const mockResponse = { id: '123', ...requestData };
      
      mockFetch.mockResolvedValueOnce({
        ok: true,
        headers: new Headers({ 'content-type': 'application/json' }),
        json: () => Promise.resolve(mockResponse),
      });

      const result = await httpClient.put('/test/123', requestData);

      expect(mockFetch).toHaveBeenCalledWith(
        'http://localhost:15002/test/123',
        expect.objectContaining({
          method: 'PUT',
          body: JSON.stringify(requestData),
        })
      );
      expect(result).toEqual(mockResponse);
    });
  });

  describe('DELETE requests', () => {
    it('should make successful DELETE request', async () => {
      const mockResponse = { success: true };
      
      mockFetch.mockResolvedValueOnce({
        ok: true,
        headers: new Headers({ 'content-type': 'application/json' }),
        json: () => Promise.resolve(mockResponse),
      });

      const result = await httpClient.delete('/test/123');

      expect(mockFetch).toHaveBeenCalledWith(
        'http://localhost:15002/test/123',
        expect.objectContaining({
          method: 'DELETE',
        })
      );
      expect(result).toEqual(mockResponse);
    });
  });

  describe('Error handling', () => {
    it('should handle 401 Unauthorized', async () => {
      // Mock fetch to reject with AuthenticationError
      mockFetch.mockRejectedValueOnce(new AuthenticationError('HTTP 401: Unauthorized'));

      await expect(httpClient.get('/test')).rejects.toThrow(AuthenticationError);
    });

    it('should handle 403 Forbidden', async () => {
      mockFetch.mockRejectedValueOnce(new AuthenticationError('HTTP 403: Forbidden'));

      await expect(httpClient.get('/test')).rejects.toThrow(AuthenticationError);
    });

    it('should handle 404 Not Found', async () => {
      mockFetch.mockRejectedValueOnce(new ServerError('HTTP 404: Not Found'));

      await expect(httpClient.get('/test')).rejects.toThrow(ServerError);
    });

    it('should handle 429 Too Many Requests', async () => {
      mockFetch.mockRejectedValueOnce(new RateLimitError('HTTP 429: Too Many Requests'));

      await expect(httpClient.get('/test')).rejects.toThrow(RateLimitError);
    });

    it('should handle 500 Internal Server Error', async () => {
      mockFetch.mockRejectedValueOnce(new ServerError('HTTP 500: Internal Server Error'));

      await expect(httpClient.get('/test')).rejects.toThrow(ServerError);
    });

    it('should handle 502 Bad Gateway', async () => {
      mockFetch.mockRejectedValueOnce(new ServerError('HTTP 502: Bad Gateway'));

      await expect(httpClient.get('/test')).rejects.toThrow(ServerError);
    });

    it('should handle 503 Service Unavailable', async () => {
      mockFetch.mockRejectedValueOnce(new ServerError('HTTP 503: Service Unavailable'));

      await expect(httpClient.get('/test')).rejects.toThrow(ServerError);
    });

    it('should handle 504 Gateway Timeout', async () => {
      mockFetch.mockRejectedValueOnce(new ServerError('HTTP 504: Gateway Timeout'));

      await expect(httpClient.get('/test')).rejects.toThrow(ServerError);
    });

    it('should handle network error', async () => {
      mockFetch.mockRejectedValueOnce(new Error('Network error'));

      await expect(httpClient.get('/test')).rejects.toThrow(NetworkError);
    });

    it('should handle timeout error', async () => {
      mockFetch.mockRejectedValueOnce(new TimeoutError('Request timeout'));

      await expect(httpClient.get('/test')).rejects.toThrow(TimeoutError);
    });

    it('should handle unknown error', async () => {
      mockFetch.mockRejectedValueOnce(new Error('Some unknown error'));

      await expect(httpClient.get('/test')).rejects.toThrow(NetworkError);
    });

    it('should handle non-JSON error response', async () => {
      mockFetch.mockRejectedValueOnce(new ServerError('HTTP 500: Internal Server Error'));

      await expect(httpClient.get('/test')).rejects.toThrow(ServerError);
    });
  });

  describe('URL handling', () => {
    it('should handle absolute URLs', async () => {
      const mockResponse = { data: 'test' };
      mockFetch.mockResolvedValueOnce({
        ok: true,
        headers: new Headers({ 'content-type': 'application/json' }),
        json: () => Promise.resolve(mockResponse),
      });

      await httpClient.get('http://example.com/test');

      expect(mockFetch).toHaveBeenCalledWith(
        'http://example.com/test',
        expect.any(Object)
      );
    });

    it('should handle relative URLs', async () => {
      const mockResponse = { data: 'test' };
      mockFetch.mockResolvedValueOnce({
        ok: true,
        headers: new Headers({ 'content-type': 'application/json' }),
        json: () => Promise.resolve(mockResponse),
      });

      await httpClient.get('/test');

      expect(mockFetch).toHaveBeenCalledWith(
        'http://localhost:15002/test',
        expect.any(Object)
      );
    });
  });

  describe('Custom headers', () => {
    it('should include custom headers', async () => {
      const client = new HttpClient({
        baseURL: 'http://localhost:15002',
        headers: { 'Custom-Header': 'custom-value' },
      });

      const mockResponse = { data: 'test' };
      mockFetch.mockResolvedValueOnce({
        ok: true,
        headers: new Headers({ 'content-type': 'application/json' }),
        json: () => Promise.resolve(mockResponse),
      });

      await client.get('/test');

      expect(mockFetch).toHaveBeenCalledWith(
        'http://localhost:15002/test',
        expect.objectContaining({
          headers: expect.objectContaining({
            'Content-Type': 'application/json',
            'Custom-Header': 'custom-value',
          }),
        })
      );
    });

    it('should override default headers with request headers', async () => {
      const client = new HttpClient({
        baseURL: 'http://localhost:15002',
        headers: { 'Content-Type': 'application/xml' },
      });

      const mockResponse = { data: 'test' };
      mockFetch.mockResolvedValueOnce({
        ok: true,
        headers: new Headers({ 'content-type': 'application/json' }),
        json: () => Promise.resolve(mockResponse),
      });

      await client.get('/test', {
        headers: { 'Content-Type': 'application/json' },
      });

      expect(mockFetch).toHaveBeenCalledWith(
        'http://localhost:15002/test',
        expect.objectContaining({
          headers: expect.objectContaining({
            'Content-Type': 'application/json',
          }),
        })
      );
    });
  });

});
