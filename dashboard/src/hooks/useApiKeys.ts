/**
 * Hook for API key management
 */

import { useState } from 'react';
import { useApiClient } from './useApiClient';
import { useAuth } from '@/contexts/AuthContext';

export interface CreateApiKeyRequest {
  name: string;
  permissions: string[];
  expires_in_days?: number | null;
}

export interface CreateApiKeyResponse {
  api_key: string;
  id: string;
  name: string;
  permissions: string[];
  expires_at?: number | null;
  warning: string;
}

export function useApiKeys() {
  const api = useApiClient();
  const { token } = useAuth();
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const createApiKey = async (request: CreateApiKeyRequest): Promise<CreateApiKeyResponse> => {
    setLoading(true);
    setError(null);
    try {
      const response = await api.post<CreateApiKeyResponse>('/auth/keys', {
        name: request.name,
        permissions: request.permissions,
        expires_in_days: request.expires_in_days,
      }, {
        headers: { Authorization: `Bearer ${token}` },
      });
      return response;
    } catch (err) {
      const errorMessage = err instanceof Error ? err.message : 'Failed to create API key';
      setError(errorMessage);
      throw new Error(errorMessage);
    } finally {
      setLoading(false);
    }
  };

  return {
    createApiKey,
    loading,
    error,
  };
}
