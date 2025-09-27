/**
 * Tests for HttpClient utility.
 */

import { HttpClient } from '../../src/utils/http-client.js';
import {
  NetworkError,
  ServerError,
  AuthenticationError,
  TimeoutError,
  RateLimitError,
} from '../../src/exceptions/index.js';

// Mock fetch
const mockFetch = global.fetch;

describe('HttpClient', () => {
  let httpClient;

  beforeEach(() => {
    jest.clearAllMocks();
    httpClient = new HttpClient({
      baseURL: 'http://localhost:15001',
      apiKey: 'test-api-key',
      timeout: 5000,
    });
  });

  describe('constructor', () => {
    it('should create client with default config', () => {
      const client = new HttpClient({
        baseURL: 'http://localhost:15001',
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
        'http://localhost:15001/test',
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
        baseURL: 'http://localhost:15001',
      });

      const mockResponse = { data: 'test' };
      mockFetch.mockResolvedValueOnce({
        ok: true,
        headers: new Headers({ 'content-type': 'application/json' }),
        json: () => Promise.resolve(mockResponse),
      });

      await client.get('/test');

      expect(mockFetch).toHaveBeenCalledWith(
        'http://localhost:15001/test',
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
        'http://localhost:15001/test',
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
        'http://localhost:15001/test',
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
        'http://localhost:15001/test/123',
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
        'http://localhost:15001/test/123',
        expect.objectContaining({
          method: 'DELETE',
        })
      );
      expect(result).toEqual(mockResponse);
    });
  });

  describe('Error handling', () => {
    it('should handle 401 Unauthorized', async () => {
      mockFetch.mockResolvedValueOnce({
        ok: false,
        status: 401,
        statusText: 'Unauthorized',
        json: () => Promise.resolve({ message: 'Invalid API key' }),
      });

      await expect(httpClient.get('/test')).rejects.toThrow(AuthenticationError);
      await expect(httpClient.get('/test')).rejects.toThrow('Invalid API key');
    });

    it('should handle 403 Forbidden', async () => {
      mockFetch.mockResolvedValueOnce({
        ok: false,
        status: 403,
        statusText: 'Forbidden',
        json: () => Promise.resolve({ message: 'Access denied' }),
      });

      await expect(httpClient.get('/test')).rejects.toThrow(AuthenticationError);
      await expect(httpClient.get('/test')).rejects.toThrow('Access forbidden');
    });

    it('should handle 404 Not Found', async () => {
      mockFetch.mockResolvedValueOnce({
        ok: false,
        status: 404,
        statusText: 'Not Found',
        json: () => Promise.resolve({ message: 'Resource not found' }),
      });

      await expect(httpClient.get('/test')).rejects.toThrow(ServerError);
      await expect(httpClient.get('/test')).rejects.toThrow('Resource not found');
    });

    it('should handle 429 Too Many Requests', async () => {
      mockFetch.mockResolvedValueOnce({
        ok: false,
        status: 429,
        statusText: 'Too Many Requests',
        json: () => Promise.resolve({ message: 'Rate limit exceeded' }),
      });

      await expect(httpClient.get('/test')).rejects.toThrow(RateLimitError);
      await expect(httpClient.get('/test')).rejects.toThrow('Rate limit exceeded');
    });

    it('should handle 500 Internal Server Error', async () => {
      mockFetch.mockResolvedValueOnce({
        ok: false,
        status: 500,
        statusText: 'Internal Server Error',
        json: () => Promise.resolve({ message: 'Internal server error' }),
      });

      await expect(httpClient.get('/test')).rejects.toThrow(ServerError);
      await expect(httpClient.get('/test')).rejects.toThrow('Internal server error');
    });

    it('should handle 502 Bad Gateway', async () => {
      mockFetch.mockResolvedValueOnce({
        ok: false,
        status: 502,
        statusText: 'Bad Gateway',
        json: () => Promise.resolve({ message: 'Bad gateway' }),
      });

      await expect(httpClient.get('/test')).rejects.toThrow(ServerError);
    });

    it('should handle 503 Service Unavailable', async () => {
      mockFetch.mockResolvedValueOnce({
        ok: false,
        status: 503,
        statusText: 'Service Unavailable',
        json: () => Promise.resolve({ message: 'Service unavailable' }),
      });

      await expect(httpClient.get('/test')).rejects.toThrow(ServerError);
    });

    it('should handle 504 Gateway Timeout', async () => {
      mockFetch.mockResolvedValueOnce({
        ok: false,
        status: 504,
        statusText: 'Gateway Timeout',
        json: () => Promise.resolve({ message: 'Gateway timeout' }),
      });

      await expect(httpClient.get('/test')).rejects.toThrow(ServerError);
    });

    it('should handle network error', async () => {
      mockFetch.mockRejectedValueOnce(new Error('Network error'));

      await expect(httpClient.get('/test')).rejects.toThrow(NetworkError);
      await expect(httpClient.get('/test')).rejects.toThrow('Network error');
    });

    it('should handle timeout error', async () => {
      const abortError = new Error('Request timeout');
      abortError.name = 'AbortError';
      mockFetch.mockRejectedValueOnce(abortError);

      await expect(httpClient.get('/test')).rejects.toThrow(TimeoutError);
      await expect(httpClient.get('/test')).rejects.toThrow('Request timeout');
    });

    it('should handle unknown error', async () => {
      const unknownError = new Error('Unknown error');
      mockFetch.mockRejectedValueOnce(unknownError);

      await expect(httpClient.get('/test')).rejects.toThrow(NetworkError);
      await expect(httpClient.get('/test')).rejects.toThrow('Unknown error');
    });

    it('should handle non-JSON error response', async () => {
      mockFetch.mockResolvedValueOnce({
        ok: false,
        status: 500,
        statusText: 'Internal Server Error',
        json: () => Promise.reject(new Error('Invalid JSON')),
      });

      await expect(httpClient.get('/test')).rejects.toThrow(ServerError);
      await expect(httpClient.get('/test')).rejects.toThrow('HTTP 500: Internal Server Error');
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
        'http://localhost:15001/test',
        expect.any(Object)
      );
    });
  });

  describe('Custom headers', () => {
    it('should include custom headers', async () => {
      const client = new HttpClient({
        baseURL: 'http://localhost:15001',
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
        'http://localhost:15001/test',
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
        baseURL: 'http://localhost:15001',
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
        'http://localhost:15001/test',
        expect.objectContaining({
          headers: expect.objectContaining({
            'Content-Type': 'application/json',
          }),
        })
      );
    });
  });

  describe('Timeout handling', () => {
    it('should use custom timeout', async () => {
      const client = new HttpClient({
        baseURL: 'http://localhost:15001',
        timeout: 10000,
      });

      const mockResponse = { data: 'test' };
      mockFetch.mockResolvedValueOnce({
        ok: true,
        headers: new Headers({ 'content-type': 'application/json' }),
        json: () => Promise.resolve(mockResponse),
      });

      await client.get('/test');

      expect(mockFetch).toHaveBeenCalledWith(
        'http://localhost:15001/test',
        expect.objectContaining({
          signal: expect.any(AbortSignal),
        })
      );
    });

    it('should use request-specific timeout', async () => {
      const mockResponse = { data: 'test' };
      mockFetch.mockResolvedValueOnce({
        ok: true,
        headers: new Headers({ 'content-type': 'application/json' }),
        json: () => Promise.resolve(mockResponse),
      });

      await httpClient.get('/test', { timeout: 15000 });

      expect(mockFetch).toHaveBeenCalledWith(
        'http://localhost:15001/test',
        expect.objectContaining({
          signal: expect.any(AbortSignal),
        })
      );
    });
  });
});
