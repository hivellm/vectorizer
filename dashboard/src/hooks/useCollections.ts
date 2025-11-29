/**
 * Hook for collections API operations
 */

import { useCallback } from 'react';
import { useApiClient } from './useApiClient';
import { ApiClientError } from '@/lib/api-client';

export interface Collection {
  name: string;
  dimension: number;
  metric: 'cosine' | 'euclidean' | 'dot';
  vector_count?: number;
  document_count?: number;
  created_at?: string;
  updated_at?: string;
  config?: Record<string, unknown>;
  // Additional fields from API
  embedding_provider?: string;
  size?: {
    total?: string;
    total_bytes?: number;
    index?: string;
    index_bytes?: number;
    payload?: string;
    payload_bytes?: number;
  };
  quantization?: {
    enabled?: boolean;
    type?: string;
    bits?: number;
  };
  normalization?: {
    enabled?: boolean;
    level?: string;
  };
  indexing_status?: {
    status?: string;
    progress?: number;
    total_documents?: number;
    processed_documents?: number;
    errors?: number;
    start_time?: string;
    end_time?: string;
  };
}

export interface CreateCollectionRequest {
  name: string;
  dimension?: number;
  metric?: 'cosine' | 'euclidean' | 'dot';
  config?: Record<string, unknown>;
}

/**
 * Hook for collections operations
 */
export function useCollections() {
  const api = useApiClient();

  /**
   * List all collections
   */
  const listCollections = useCallback(async (): Promise<Collection[]> => {
    try {
      console.log('[useCollections] Fetching collections from /collections');
      // Middleware already extracts the array from { collections: [...] } format
      const data = await api.get<Collection[]>('/collections');
      console.log('[useCollections] Raw response:', data);
      
      // Ensure we always return an array
      if (Array.isArray(data)) {
        console.log('[useCollections] Returning array with', data.length, 'collections');
        return data;
      }
      
      // Fallback: if middleware didn't extract, try to extract here
      if (data && typeof data === 'object' && 'collections' in data) {
        const response = data as { collections: Collection[] };
        const collections = Array.isArray(response.collections) ? response.collections : [];
        console.log('[useCollections] Extracted collections from object:', collections.length);
        return collections;
      }
      
      console.warn('[useCollections] Unexpected response format:', data);
      return [];
    } catch (error) {
      console.error('[useCollections] Error:', error);
      if (error instanceof ApiClientError) {
        throw error;
      }
      throw new ApiClientError('Failed to list collections', 500);
    }
  }, [api]);

  /**
   * Get collection details
   */
  const getCollection = useCallback(async (name: string): Promise<Collection> => {
    try {
      return await api.get<Collection>(`/collections/${encodeURIComponent(name)}`);
    } catch (error) {
      if (error instanceof ApiClientError) {
        throw error;
      }
      throw new ApiClientError(`Failed to get collection: ${name}`, 500);
    }
  }, [api]);

  /**
   * Create a new collection
   */
  const createCollection = useCallback(async (request: CreateCollectionRequest): Promise<Collection> => {
    try {
      return await api.post<Collection>('/collections', request);
    } catch (error) {
      if (error instanceof ApiClientError) {
        throw error;
      }
      throw new ApiClientError('Failed to create collection', 500);
    }
  }, [api]);

  /**
   * Delete a collection
   */
  const deleteCollection = useCallback(async (name: string): Promise<void> => {
    try {
      await api.delete(`/collections/${encodeURIComponent(name)}`);
    } catch (error) {
      if (error instanceof ApiClientError) {
        throw error;
      }
      throw new ApiClientError(`Failed to delete collection: ${name}`, 500);
    }
  }, [api]);

  return {
    listCollections,
    getCollection,
    createCollection,
    deleteCollection,
  };
}

