/**
 * Tests for HttpClient utility.
 */

import { HttpClient } from '../../src/utils/http-client';
import {
  NetworkError,
  ServerError,
  AuthenticationError,
  TimeoutError,
  RateLimitError,
} from '../../src/exceptions';

// Mock fetch
const mockFetch = jest.fn() as jest.MockedFunction<typeof fetch>;
global.fetch = mockFetch;

describe('HttpClient', () => {
  let httpClient: HttpClient;

  beforeEach(() => {
    jest.clearAllMocks();
    
    // Default successful response
    mockFetch.mockResolvedValue({
      ok: true,
      status: 200,
      statusText: 'OK',
      headers: new Headers({ 'content-type': 'application/json' }),
      json: jest.fn().mockResolvedValue({}),
      text: jest.fn().mockResolvedValue(''),
    } as any);
    
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
      } as Response);

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
      } as Response);

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
      } as Response);

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
      } as Response);

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
      } as Response);

      const result = await httpClient.post('/test');

      expect(mockFetch).toHaveBeenCalledWith(
        'http://localhost:15001/test',
        expect.objectContaining({
          method: 'POST',
        })
      );
      
      // Verify body is not set when no data is provided
      const callArgs = mockFetch.mock.calls[0]?.[1] as RequestInit;
      expect(callArgs?.body).toBeUndefined();
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
      } as Response);

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
      } as Response);

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
        headers: new Headers({ 'content-type': 'application/json' }),
        json: () => Promise.resolve({ message: 'Invalid API key' }),
      } as Response);

      await expect(httpClient.get('/test')).rejects.toThrow(AuthenticationError);
    });

    it('should handle 403 Forbidden', async () => {
      mockFetch.mockResolvedValueOnce({
        ok: false,
        status: 403,
        statusText: 'Forbidden',
        headers: new Headers({ 'content-type': 'application/json' }),
        json: () => Promise.resolve({ message: 'Access denied' }),
      } as Response);

      await expect(httpClient.get('/test')).rejects.toThrow(AuthenticationError);
    });

    it('should handle 404 Not Found', async () => {
      mockFetch.mockResolvedValueOnce({
        ok: false,
        status: 404,
        statusText: 'Not Found',
        headers: new Headers({ 'content-type': 'application/json' }),
        json: () => Promise.resolve({ message: 'Resource not found' }),
      } as Response);

      await expect(httpClient.get('/test')).rejects.toThrow(ServerError);
    });

    it('should handle 429 Too Many Requests', async () => {
      mockFetch.mockResolvedValueOnce({
        ok: false,
        status: 429,
        statusText: 'Too Many Requests',
        headers: new Headers({ 'content-type': 'application/json' }),
        json: () => Promise.resolve({ message: 'Rate limit exceeded' }),
      } as Response);

      await expect(httpClient.get('/test')).rejects.toThrow(RateLimitError);
    });

    it('should handle 500 Internal Server Error', async () => {
      mockFetch.mockResolvedValueOnce({
        ok: false,
        status: 500,
        statusText: 'Internal Server Error',
        headers: new Headers({ 'content-type': 'application/json' }),
        json: () => Promise.resolve({ message: 'Internal server error' }),
      } as Response);

      await expect(httpClient.get('/test')).rejects.toThrow(ServerError);
    });

    it('should handle 502 Bad Gateway', async () => {
      mockFetch.mockResolvedValueOnce({
        ok: false,
        status: 502,
        statusText: 'Bad Gateway',
        json: () => Promise.resolve({ message: 'Bad gateway' }),
      } as Response);

      await expect(httpClient.get('/test')).rejects.toThrow(ServerError);
    });

    it('should handle 503 Service Unavailable', async () => {
      mockFetch.mockResolvedValueOnce({
        ok: false,
        status: 503,
        statusText: 'Service Unavailable',
        json: () => Promise.resolve({ message: 'Service unavailable' }),
      } as Response);

      await expect(httpClient.get('/test')).rejects.toThrow(ServerError);
    });

    it('should handle 504 Gateway Timeout', async () => {
      mockFetch.mockResolvedValueOnce({
        ok: false,
        status: 504,
        statusText: 'Gateway Timeout',
        json: () => Promise.resolve({ message: 'Gateway timeout' }),
      } as Response);

      await expect(httpClient.get('/test')).rejects.toThrow(ServerError);
    });

    it('should handle network error', async () => {
      mockFetch.mockRejectedValueOnce(new Error('Network error'));

      await expect(httpClient.get('/test')).rejects.toThrow(NetworkError);
    });

    it('should handle timeout error', async () => {
      const abortError = new Error('Request timeout');
      abortError.name = 'AbortError';
      mockFetch.mockRejectedValueOnce(abortError);

      await expect(httpClient.get('/test')).rejects.toThrow(TimeoutError);
    });

    it('should handle unknown error', async () => {
      const unknownError = new Error('Unknown error');
      mockFetch.mockRejectedValueOnce(unknownError);

      await expect(httpClient.get('/test')).rejects.toThrow(NetworkError);
    });

    it('should handle non-JSON error response', async () => {
      mockFetch.mockResolvedValueOnce({
        ok: false,
        status: 500,
        statusText: 'Internal Server Error',
        json: () => Promise.reject(new Error('Invalid JSON')),
      } as Response);

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
      } as Response);

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
      } as Response);

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
      } as Response);

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
      } as Response);

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
      } as Response);

      await client.get('/test');

      expect(mockFetch).toHaveBeenCalledWith(
        'http://localhost:15001/test',
        expect.objectContaining({
          signal: expect.objectContaining({
            aborted: false,
            addEventListener: expect.any(Function),
            removeEventListener: expect.any(Function),
          }),
        })
      );
    });

    it('should use request-specific timeout', async () => {
      const mockResponse = { data: 'test' };
      mockFetch.mockResolvedValueOnce({
        ok: true,
        headers: new Headers({ 'content-type': 'application/json' }),
        json: () => Promise.resolve(mockResponse),
      } as Response);

      await httpClient.get('/test', { timeout: 15000 });

      expect(mockFetch).toHaveBeenCalledWith(
        'http://localhost:15001/test',
        expect.objectContaining({
          signal: expect.objectContaining({
            aborted: false,
            addEventListener: expect.any(Function),
            removeEventListener: expect.any(Function),
          }),
        })
      );
    });
  });
});
