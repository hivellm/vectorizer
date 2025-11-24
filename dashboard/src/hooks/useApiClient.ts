/**
 * Hook for API client access
 * Uses API base URL from env config (localhost:15002 in dev, same origin in prod)
 */

import { useMemo } from 'react';
import { ApiClient } from '@/lib/api-client';
// getApiUrl imported but not used - API client uses baseUrl parameter instead

/**
 * Get API client instance
 * Uses API base URL from env config
 */
export function useApiClient(): ApiClient {
  // Use API base URL from env config
  // In dev: http://localhost:15002
  // In prod (served by Rust server): same origin
  return useMemo(() => {
    const baseUrl = import.meta.env.VITE_API_BASE_URL || 
      (import.meta.env.DEV 
        ? 'http://localhost:15002'
        : '');
    return new ApiClient(baseUrl);
  }, []);
}

