import { defineStore } from 'pinia';
import { ref, computed } from 'vue';
import { VectorizerClient } from '@hivellm/vectorizer-client';
import type { Collection, SearchResult, IndexingProgress } from '@shared/types';
import { useConnectionsStore } from './connections';

export const useVectorizerStore = defineStore('vectorizer', () => {
  // State
  const client = ref<VectorizerClient | null>(null);
  const collections = ref<Collection[]>([]);
  const loading = ref(false);
  const error = ref<string | null>(null);

  // Computed
  const isConnected = computed(() => client.value !== null);
  
  const totalVectors = computed(() => 
    collections.value.reduce((sum, col) => sum + col.vector_count, 0)
  );

  const avgDimension = computed(() => {
    if (collections.value.length === 0) return 0;
    return Math.round(
      collections.value.reduce((sum, col) => sum + col.dimension, 0) / collections.value.length
    );
  });

  // Actions
  async function initializeClient(host: string, port: number, apiKey?: string): Promise<boolean> {
    try {
      loading.value = true;
      error.value = null;

      client.value = new VectorizerClient({
        baseURL: `http://${host}:${port}`,
        apiKey,
        timeout: 30000,
        logger: {
          level: 'info',
          enabled: true
        }
      });

      // Test connection
      await client.value.listCollections();
      
      return true;
    } catch (err) {
      error.value = err instanceof Error ? err.message : 'Failed to connect';
      client.value = null;
      return false;
    } finally {
      loading.value = false;
    }
  }

  async function disconnect(): Promise<void> {
    client.value = null;
    collections.value = [];
    error.value = null;
  }

  async function loadCollections(): Promise<void> {
    if (!client.value) {
      throw new Error('Client not initialized');
    }

    try {
      loading.value = true;
      error.value = null;

      const response = await client.value.listCollections();
      collections.value = response as unknown as Collection[];
    } catch (err) {
      error.value = err instanceof Error ? err.message : 'Failed to load collections';
      throw err;
    } finally {
      loading.value = false;
    }
  }

  async function createCollection(data: {
    name: string;
    dimension: number;
    metric: string;
  }): Promise<void> {
    if (!client.value) {
      throw new Error('Client not initialized');
    }

    try {
      loading.value = true;
      error.value = null;

      await client.value.createCollection({
        name: data.name,
        dimension: data.dimension,
        metric: data.metric as 'cosine' | 'euclidean' | 'dot'
      });

      await loadCollections();
    } catch (err) {
      error.value = err instanceof Error ? err.message : 'Failed to create collection';
      throw err;
    } finally {
      loading.value = false;
    }
  }

  async function deleteCollection(name: string): Promise<void> {
    if (!client.value) {
      throw new Error('Client not initialized');
    }

    try {
      loading.value = true;
      error.value = null;

      await client.value.deleteCollection(name);
      collections.value = collections.value.filter(c => c.name !== name);
    } catch (err) {
      error.value = err instanceof Error ? err.message : 'Failed to delete collection';
      throw err;
    } finally {
      loading.value = false;
    }
  }

  async function search(
    collectionName: string,
    query: string,
    limit: number = 10
  ): Promise<SearchResult[]> {
    if (!client.value) {
      throw new Error('Client not initialized');
    }

    try {
      loading.value = true;
      error.value = null;

      const response = await client.value.searchByText(collectionName, {
        query,
        limit
      });

      return response.results as unknown as SearchResult[];
    } catch (err) {
      error.value = err instanceof Error ? err.message : 'Search failed';
      throw err;
    } finally {
      loading.value = false;
    }
  }

  async function insertText(
    collectionName: string,
    text: string,
    metadata?: Record<string, unknown>
  ): Promise<string> {
    if (!client.value) {
      throw new Error('Client not initialized');
    }

    try {
      loading.value = true;
      error.value = null;

      const response = await client.value.insertText(collectionName, text, metadata);
      return response.id;
    } catch (err) {
      error.value = err instanceof Error ? err.message : 'Failed to insert text';
      throw err;
    } finally {
      loading.value = false;
    }
  }

  async function batchInsertTexts(
    collectionName: string,
    texts: Array<{ text: string; metadata?: Record<string, unknown> }>
  ): Promise<void> {
    if (!client.value) {
      throw new Error('Client not initialized');
    }

    try {
      loading.value = true;
      error.value = null;

      await client.value.batchInsertTexts(collectionName, texts);
    } catch (err) {
      error.value = err instanceof Error ? err.message : 'Failed to batch insert texts';
      throw err;
    } finally {
      loading.value = false;
    }
  }

  return {
    // State
    client: computed(() => client.value),
    collections: computed(() => collections.value),
    loading: computed(() => loading.value),
    error: computed(() => error.value),
    isConnected,
    totalVectors,
    avgDimension,

    // Actions
    initializeClient,
    disconnect,
    loadCollections,
    createCollection,
    deleteCollection,
    search,
    insertText,
    batchInsertTexts
  };
});

