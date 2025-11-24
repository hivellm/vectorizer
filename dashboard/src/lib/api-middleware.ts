/**
 * API Middleware System
 * Allows intercepting and transforming API requests/responses
 */

import { ApiClientError } from './api-client';

/**
 * Middleware context passed between middleware functions
 */
export interface MiddlewareContext {
  url: string;
  method: string;
  headers: HeadersInit;
  body?: unknown;
  params?: Record<string, string | number | boolean>;
  response?: Response;
  data?: unknown;
  error?: Error | ApiClientError;
}

/**
 * Middleware function type
 */
export type Middleware = (
  context: MiddlewareContext,
  next: () => Promise<MiddlewareContext>
) => Promise<MiddlewareContext>;

/**
 * Middleware manager
 */
export class MiddlewareManager {
  private middlewares: Middleware[] = [];

  /**
   * Add a middleware function
   */
  use(middleware: Middleware): void {
    this.middlewares.push(middleware);
  }

  /**
   * Execute all middlewares in order
   */
  async execute(context: MiddlewareContext): Promise<MiddlewareContext> {
    let index = 0;

    const next = async (): Promise<MiddlewareContext> => {
      if (index >= this.middlewares.length) {
        return context;
      }

      const middleware = this.middlewares[index++];
      return middleware(context, next);
    };

    return next();
  }

  /**
   * Clear all middlewares
   */
  clear(): void {
    this.middlewares = [];
  }
}

/**
 * Built-in middleware: Request logging
 */
export const loggingMiddleware: Middleware = async (context, next) => {
  const startTime = Date.now();

  console.log(`[API] ${context.method} ${context.url}`, {
    headers: context.headers,
    body: context.body,
    params: context.params,
  });

  const result = await next();

  const duration = Date.now() - startTime;

  if (result.error) {
    console.error(`[API] ${context.method} ${context.url} failed in ${duration}ms`, result.error);
  } else {
    console.log(`[API] ${context.method} ${context.url} succeeded in ${duration}ms`, {
      status: result.response?.status,
      data: result.data,
    });
  }

  return result;
};

/**
 * Built-in middleware: Error handling
 * Wraps middleware execution in try-catch
 */
export const errorHandlingMiddleware: Middleware = async (context, next) => {
  try {
    return await next();
  } catch (error) {
    return {
      ...context,
      error: error instanceof Error ? error : new Error(String(error)),
    };
  }
};

/**
 * Built-in middleware: HTTP request executor
 * This middleware actually makes the HTTP request
 */
export const httpRequestMiddleware: Middleware = async (context, next) => {
  // If response already exists, skip making the request
  if (context.response) {
    console.log('[httpRequestMiddleware] Response already exists, skipping');
    return next();
  }

  console.log('[httpRequestMiddleware] Making HTTP request:', {
    url: context.url,
    method: context.method,
    headers: context.headers,
    hasBody: !!context.body,
  });

  try {
    const body = context.body ? JSON.stringify(context.body) : undefined;

    const response = await fetch(context.url, {
      method: context.method,
      headers: context.headers as HeadersInit,
      body,
    });

    console.log('[httpRequestMiddleware] Response received:', {
      status: response.status,
      statusText: response.statusText,
      ok: response.ok,
      headers: Object.fromEntries(response.headers.entries()),
    });

    // Add response to context and pass to next middleware
    context.response = response;

    // Call next middleware - it will receive context with response
    const result = await next();

    // Ensure response is in result
    return {
      ...result,
      response: result.response || response,
    };
  } catch (error) {
    console.error('[httpRequestMiddleware] Fetch error:', error);
    
    // Create a proper ApiClientError for network/CORS errors
    let errorMessage = 'Network error';
    if (error instanceof TypeError && error.message.includes('Failed to fetch')) {
      errorMessage = 'Failed to fetch. This may be a CORS issue or the server is not reachable.';
    } else if (error instanceof Error) {
      errorMessage = error.message;
    }
    
    return {
      ...context,
      error: new ApiClientError(
        errorMessage,
        0, // Status 0 indicates network error
        'NETWORK_ERROR',
        { originalError: String(error) }
      ),
      response: undefined, // Explicitly set to undefined
    };
  }
};

/**
 * Built-in middleware: Response transformation
 */
export const responseTransformMiddleware: Middleware = async (context, next) => {
  // Get result from previous middleware (httpRequestMiddleware should have added response)
  // Since this is the last middleware, next() would return original context
  // So we use the context directly if it has a response, otherwise call next()
  const result = context.response ? context : await next();

  console.log('[responseTransformMiddleware] Processing response:', {
    hasResponse: !!result.response,
    status: result.response?.status,
    ok: result.response?.ok,
    hasError: !!result.error,
  });

  // If there's an error and no response (e.g., CORS or network error), return early with error
  if (result.error && !result.response) {
    console.log('[responseTransformMiddleware] Error without response, returning error:', result.error);
    return result;
  }

  // If no response and no error, this is unexpected - create an error
  if (!result.response && !result.error) {
    console.warn('[responseTransformMiddleware] No response and no error - this is unexpected');
    result.error = new ApiClientError(
      'No response received from server. This may be a CORS or network issue.',
      0
    );
    return result;
  }

  if (result.response) {
    const contentType = result.response.headers.get('content-type');
    const isJson = contentType?.includes('application/json');
    console.log('[responseTransformMiddleware] Content type:', contentType, 'isJson:', isJson);

    if (!result.response.ok) {
      // Handle error responses
      let errorMessage = `HTTP ${result.response.status}: ${result.response.statusText}`;

      if (isJson) {
        try {
          const errorData = await result.response.clone().json();
          errorMessage = errorData.message || errorData.error || errorMessage;
          result.error = new ApiClientError(
            errorMessage,
            result.response.status,
            errorData.code,
            errorData.details
          );
        } catch {
          // If JSON parsing fails, use status text
          result.error = new ApiClientError(
            errorMessage,
            result.response.status
          );
        }
      } else {
        try {
          const text = await result.response.clone().text();
          result.error = new ApiClientError(
            text || errorMessage,
            result.response.status
          );
        } catch {
          result.error = new ApiClientError(errorMessage, result.response.status);
        }
      }

      return result;
    }

    // Handle success responses
    if (result.response.status === 204 || result.response.headers.get('content-length') === '0') {
      result.data = undefined;
      return result;
    }

    if (isJson) {
      try {
        // Clone response before reading to avoid consuming the body
        const clonedResponse = result.response.clone();
        const data = await clonedResponse.json();
        console.log('[responseTransformMiddleware] Parsed JSON data:', data);
        console.log('[responseTransformMiddleware] Data type:', typeof data, 'isArray:', Array.isArray(data));

        // Handle different response formats
        // If API returns { collections: [...] }, extract the array
        if (data && typeof data === 'object' && 'collections' in data && Array.isArray(data.collections)) {
          console.log('[responseTransformMiddleware] Extracting collections array, length:', data.collections.length);
          result.data = data.collections;
        } else if (data && typeof data === 'object' && 'data' in data && Array.isArray(data.data)) {
          // Handle { data: [...] } format
          console.log('[responseTransformMiddleware] Extracting data field, length:', data.data.length);
          result.data = data.data;
        } else if (Array.isArray(data)) {
          // Direct array response
          console.log('[responseTransformMiddleware] Direct array response, length:', data.length);
          result.data = data;
        } else {
          console.log('[responseTransformMiddleware] Using data as-is:', data);
          result.data = data;
        }
        console.log('[responseTransformMiddleware] Final result.data:', result.data);
        console.log('[responseTransformMiddleware] Final result.data type:', typeof result.data, 'isArray:', Array.isArray(result.data));
        
        // Ensure data is always set (even if null/undefined)
        if (result.data === undefined || result.data === null) {
          console.warn('[responseTransformMiddleware] Warning: result.data is undefined/null after processing, setting to empty object');
          result.data = data || {};
        }
      } catch (error) {
        console.warn('[responseTransformMiddleware] Failed to parse JSON response', error);
        result.error = error instanceof Error ? error : new Error('Failed to parse JSON response');
      }
    } else {
      // Non-JSON response
      try {
        result.data = await result.response.text();
      } catch (error) {
        result.error = error instanceof Error ? error : new Error('Failed to read response');
      }
    }
  }

  return result;
};

/**
 * Built-in middleware: Retry logic
 */
export const retryMiddleware = (maxRetries = 3, retryDelay = 1000): Middleware => {
  return async (context, next) => {
    let lastError: Error | ApiClientError | undefined;
    let attempt = 0;

    while (attempt <= maxRetries) {
      try {
        const result = await next();

        // Retry on network errors or 5xx errors
        if (result.error || (result.response && result.response.status >= 500)) {
          if (attempt < maxRetries) {
            attempt++;
            await new Promise(resolve => setTimeout(resolve, retryDelay * attempt));
            // Reset response and error for retry
            context.response = undefined;
            context.error = undefined;
            continue;
          }
        }

        return result;
      } catch (error) {
        lastError = error instanceof Error ? error : new Error(String(error));

        if (attempt < maxRetries) {
          attempt++;
          await new Promise(resolve => setTimeout(resolve, retryDelay * attempt));
          // Reset for retry
          context.response = undefined;
          context.error = undefined;
          continue;
        }

        return {
          ...context,
          error: lastError,
        };
      }
    }

    return {
      ...context,
      error: lastError || new Error('Max retries exceeded'),
    };
  };
};

/**
 * Built-in middleware: Request timeout
 */
export const timeoutMiddleware = (timeoutMs = 30000): Middleware => {
  return async (context, next) => {
    const timeoutPromise = new Promise<MiddlewareContext>((_, reject) => {
      setTimeout(() => {
        reject(new Error(`Request timeout after ${timeoutMs}ms`));
      }, timeoutMs);
    });

    try {
      return await Promise.race([next(), timeoutPromise]);
    } catch (error) {
      return {
        url: context.url,
        method: context.method,
        headers: context.headers,
        body: context.body,
        params: context.params,
        error: error instanceof Error ? error : new Error(String(error)),
      };
    }
  };
};

/**
 * Built-in middleware: CORS handling
 */
export const corsMiddleware: Middleware = async (context, next) => {
  const headers = {
    ...context.headers,
    'Access-Control-Allow-Origin': '*',
    'Access-Control-Allow-Methods': 'GET, POST, PUT, DELETE, OPTIONS',
    'Access-Control-Allow-Headers': 'Content-Type, Authorization',
  };

  return next().then(result => ({
    ...result,
    headers,
  }));
};

/**
 * Create default middleware stack
 */
export function createDefaultMiddlewareStack(options?: {
  enableLogging?: boolean;
  enableRetry?: boolean;
  retryCount?: number;
  timeout?: number;
}): MiddlewareManager {
  const manager = new MiddlewareManager();

  // Error handling first (wraps everything)
  manager.use(errorHandlingMiddleware);

  // Logging (before request)
  if (options?.enableLogging !== false) {
    manager.use(loggingMiddleware);
  }

  // Timeout (before request)
  if (options?.timeout) {
    manager.use(timeoutMiddleware(options.timeout));
  }

  // Retry (wraps HTTP request)
  if (options?.enableRetry !== false) {
    manager.use(retryMiddleware(options?.retryCount || 3));
  }

  // HTTP request executor (makes the actual request)
  manager.use(httpRequestMiddleware);

  // Response transformation (after request)
  manager.use(responseTransformMiddleware);

  return manager;
}

