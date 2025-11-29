import { defineStore } from 'pinia';
import { ref, computed } from 'vue';
import { VectorizerClient } from '@hivellm/vectorizer-sdk';
import type { Collection, SearchResult, IndexingProgress, IndexingStatus } from '@shared/types';
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
    collections.value.reduce((sum: number, col: Collection) => sum + col.vector_count, 0)
  );

  const avgDimension = computed(() => {
    if (collections.value.length === 0) return 0;
    return Math.round(
      collections.value.reduce((sum: number, col: Collection) => sum + col.dimension, 0) / collections.value.length
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
      
      let rawCollections: any[] = [];
      
      // Handle both direct array and wrapped response
      if (Array.isArray(response)) {
        rawCollections = response;
      } else if (response && typeof response === 'object' && 'collections' in response) {
        // Response is wrapped in { collections: [...] }
        rawCollections = (response as any).collections;
      }
      
      // Map API response to our Collection interface
      collections.value = rawCollections.map((col: any): Collection => ({
        name: col.name,
        dimension: col.dimension,
        metric: (col.metric?.toLowerCase() || 'cosine') as 'cosine' | 'euclidean' | 'dot',
        vector_count: col.vector_count || col.document_count || 0,
        embedding_provider: col.embedding_provider || 'unknown',
        indexing_status: {
          status: (col.indexing_status?.status || 'completed') as IndexingStatus,
          progress: col.indexing_status?.progress || 1,
          last_updated: col.indexing_status?.end_time || col.created_at
        },
        quantization: col.quantization ? {
          enabled: col.quantization.enabled || false,
          type: col.quantization.type
        } : undefined,
        size: col.size ? {
          total: col.size.total || '0 B',
          total_bytes: col.size.total_bytes || 0
        } : undefined
      }));
    } catch (err) {
      error.value = err instanceof Error ? err.message : 'Failed to load collections';
      collections.value = [];
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
        similarity_metric: data.metric as 'cosine' | 'euclidean' | 'dot_product'
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
      collections.value = collections.value.filter((c: Collection) => c.name !== name);
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

      // Use SDK method
      const response = await client.value.searchText(collectionName, {
        query,
        limit
      });
      return response.results || [];
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

      // Use SDK method
      const response = await client.value.insertText(collectionName, {
        text,
        metadata
      });
      return response.id || response.vector_id || 'unknown';
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

      await client.value.batchInsertTexts(collectionName, {
        texts
      });
    } catch (err) {
      error.value = err instanceof Error ? err.message : 'Failed to batch insert texts';
      throw err;
    } finally {
      loading.value = false;
    }
  }

  async function listVectors(
    collectionName: string,
    limit: number = 50,
    offset: number = 0
  ): Promise<SearchResult[]> {
    if (!client.value) {
      throw new Error('Client not initialized');
    }

    try {
      loading.value = true;
      error.value = null;

      // Use direct API call to list vectors
      const config = client.value.getConfig();
      const response = await fetch(`${config.baseURL}/collections/${collectionName}/vectors?limit=${limit}&offset=${offset}`, {
        method: 'GET',
        headers: {
          'Content-Type': 'application/json',
          ...(config.apiKey ? { 'Authorization': `Bearer ${config.apiKey}` } : {})
        }
      });

      if (!response.ok) {
        throw new Error(`Failed to list vectors: ${response.statusText}`);
      }

      const data = await response.json();
      return data.vectors || [];
    } catch (err) {
      error.value = err instanceof Error ? err.message : 'Failed to list vectors';
      throw err;
    } finally {
      loading.value = false;
    }
  }

  // Graph operations
  async function listGraphNodes(collectionName: string): Promise<any> {
    if (!client.value) {
      throw new Error('Client not initialized');
    }

    try {
      loading.value = true;
      error.value = null;

      const config = client.value.getConfig();
      const response = await fetch(`${config.baseURL}/graph/nodes/${encodeURIComponent(collectionName)}`, {
        method: 'GET',
        headers: {
          'Content-Type': 'application/json',
          ...(config.apiKey ? { 'Authorization': `Bearer ${config.apiKey}` } : {})
        }
      });

      if (!response.ok) {
        throw new Error(`Failed to list graph nodes: ${response.statusText}`);
      }

      return await response.json();
    } catch (err) {
      error.value = err instanceof Error ? err.message : 'Failed to list graph nodes';
      throw err;
    } finally {
      loading.value = false;
    }
  }

  async function listGraphEdges(collectionName: string): Promise<any> {
    if (!client.value) {
      throw new Error('Client not initialized');
    }

    try {
      loading.value = true;
      error.value = null;

      const config = client.value.getConfig();
      const response = await fetch(`${config.baseURL}/graph/collections/${encodeURIComponent(collectionName)}/edges`, {
        method: 'GET',
        headers: {
          'Content-Type': 'application/json',
          ...(config.apiKey ? { 'Authorization': `Bearer ${config.apiKey}` } : {})
        }
      });

      if (!response.ok) {
        throw new Error(`Failed to list graph edges: ${response.statusText}`);
      }

      return await response.json();
    } catch (err) {
      error.value = err instanceof Error ? err.message : 'Failed to list graph edges';
      throw err;
    } finally {
      loading.value = false;
    }
  }

  async function getGraphNeighbors(collectionName: string, nodeId: string): Promise<any> {
    if (!client.value) {
      throw new Error('Client not initialized');
    }

    try {
      loading.value = true;
      error.value = null;

      const config = client.value.getConfig();
      const response = await fetch(`${config.baseURL}/graph/nodes/${encodeURIComponent(collectionName)}/${encodeURIComponent(nodeId)}/neighbors`, {
        method: 'GET',
        headers: {
          'Content-Type': 'application/json',
          ...(config.apiKey ? { 'Authorization': `Bearer ${config.apiKey}` } : {})
        }
      });

      if (!response.ok) {
        throw new Error(`Failed to get neighbors: ${response.statusText}`);
      }

      return await response.json();
    } catch (err) {
      error.value = err instanceof Error ? err.message : 'Failed to get neighbors';
      throw err;
    } finally {
      loading.value = false;
    }
  }

  async function findRelatedNodes(collectionName: string, nodeId: string, maxHops?: number, relationshipType?: string): Promise<any> {
    if (!client.value) {
      throw new Error('Client not initialized');
    }

    try {
      loading.value = true;
      error.value = null;

      const body: any = {};
      if (maxHops !== undefined) body.max_hops = maxHops;
      if (relationshipType) body.relationship_type = relationshipType;

      const config = client.value.getConfig();
      const response = await fetch(`${config.baseURL}/graph/nodes/${encodeURIComponent(collectionName)}/${encodeURIComponent(nodeId)}/related`, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
          ...(config.apiKey ? { 'Authorization': `Bearer ${config.apiKey}` } : {})
        },
        body: JSON.stringify(body)
      });

      if (!response.ok) {
        throw new Error(`Failed to find related nodes: ${response.statusText}`);
      }

      return await response.json();
    } catch (err) {
      error.value = err instanceof Error ? err.message : 'Failed to find related nodes';
      throw err;
    } finally {
      loading.value = false;
    }
  }

  async function findGraphPath(collectionName: string, source: string, target: string): Promise<any> {
    if (!client.value) {
      throw new Error('Client not initialized');
    }

    try {
      loading.value = true;
      error.value = null;

      const config = client.value.getConfig();
      const response = await fetch(`${config.baseURL}/graph/path`, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
          ...(config.apiKey ? { 'Authorization': `Bearer ${config.apiKey}` } : {})
        },
        body: JSON.stringify({
          collection: collectionName,
          source,
          target
        })
      });

      if (!response.ok) {
        throw new Error(`Failed to find path: ${response.statusText}`);
      }

      return await response.json();
    } catch (err) {
      error.value = err instanceof Error ? err.message : 'Failed to find path';
      throw err;
    } finally {
      loading.value = false;
    }
  }

  async function createGraphEdge(collectionName: string, source: string, target: string, relationshipType: string, weight?: number): Promise<any> {
    if (!client.value) {
      throw new Error('Client not initialized');
    }

    try {
      loading.value = true;
      error.value = null;

      const body: any = {
        collection: collectionName,
        source,
        target,
        relationship_type: relationshipType
      };
      if (weight !== undefined) body.weight = weight;

      const config = client.value.getConfig();
      const response = await fetch(`${config.baseURL}/graph/edges`, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
          ...(config.apiKey ? { 'Authorization': `Bearer ${config.apiKey}` } : {})
        },
        body: JSON.stringify(body)
      });

      if (!response.ok) {
        throw new Error(`Failed to create edge: ${response.statusText}`);
      }

      return await response.json();
    } catch (err) {
      error.value = err instanceof Error ? err.message : 'Failed to create edge';
      throw err;
    } finally {
      loading.value = false;
    }
  }

  async function deleteGraphEdge(edgeId: string): Promise<void> {
    if (!client.value) {
      throw new Error('Client not initialized');
    }

    try {
      loading.value = true;
      error.value = null;

      const config = client.value.getConfig();
      const response = await fetch(`${config.baseURL}/graph/edges/${encodeURIComponent(edgeId)}`, {
        method: 'DELETE',
        headers: {
          'Content-Type': 'application/json',
          ...(config.apiKey ? { 'Authorization': `Bearer ${config.apiKey}` } : {})
        }
      });

      if (!response.ok) {
        throw new Error(`Failed to delete edge: ${response.statusText}`);
      }
    } catch (err) {
      error.value = err instanceof Error ? err.message : 'Failed to delete edge';
      throw err;
    } finally {
      loading.value = false;
    }
  }

  async function discoverGraphEdges(collectionName: string, similarityThreshold?: number, maxPerNode?: number): Promise<any> {
    if (!client.value) {
      throw new Error('Client not initialized');
    }

    try {
      loading.value = true;
      error.value = null;

      const body: any = {};
      if (similarityThreshold !== undefined) body.similarity_threshold = similarityThreshold;
      if (maxPerNode !== undefined) body.max_per_node = maxPerNode;

      const config = client.value.getConfig();
      const response = await fetch(`${config.baseURL}/graph/discover/${encodeURIComponent(collectionName)}`, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
          ...(config.apiKey ? { 'Authorization': `Bearer ${config.apiKey}` } : {})
        },
        body: JSON.stringify(body)
      });

      if (!response.ok) {
        throw new Error(`Failed to discover edges: ${response.statusText}`);
      }

      return await response.json();
    } catch (err) {
      error.value = err instanceof Error ? err.message : 'Failed to discover edges';
      throw err;
    } finally {
      loading.value = false;
    }
  }

  async function discoverGraphEdgesForNode(collectionName: string, nodeId: string, similarityThreshold?: number, maxPerNode?: number): Promise<any> {
    if (!client.value) {
      throw new Error('Client not initialized');
    }

    try {
      loading.value = true;
      error.value = null;

      const body: any = {};
      if (similarityThreshold !== undefined) body.similarity_threshold = similarityThreshold;
      if (maxPerNode !== undefined) body.max_per_node = maxPerNode;

      const config = client.value.getConfig();
      const response = await fetch(`${config.baseURL}/graph/discover/${encodeURIComponent(collectionName)}/${encodeURIComponent(nodeId)}`, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
          ...(config.apiKey ? { 'Authorization': `Bearer ${config.apiKey}` } : {})
        },
        body: JSON.stringify(body)
      });

      if (!response.ok) {
        throw new Error(`Failed to discover edges for node: ${response.statusText}`);
      }

      return await response.json();
    } catch (err) {
      error.value = err instanceof Error ? err.message : 'Failed to discover edges for node';
      throw err;
    } finally {
      loading.value = false;
    }
  }

  async function getGraphDiscoveryStatus(collectionName: string): Promise<any> {
    if (!client.value) {
      throw new Error('Client not initialized');
    }

    try {
      loading.value = true;
      error.value = null;

      const config = client.value.getConfig();
      const response = await fetch(`${config.baseURL}/graph/discover/${encodeURIComponent(collectionName)}/status`, {
        method: 'GET',
        headers: {
          'Content-Type': 'application/json',
          ...(config.apiKey ? { 'Authorization': `Bearer ${config.apiKey}` } : {})
        }
      });

      if (!response.ok) {
        throw new Error(`Failed to get discovery status: ${response.statusText}`);
      }

      return await response.json();
    } catch (err) {
      error.value = err instanceof Error ? err.message : 'Failed to get discovery status';
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
    batchInsertTexts,
    listVectors,
    // Graph operations
    listGraphNodes,
    listGraphEdges,
    getGraphNeighbors,
    findRelatedNodes,
    findGraphPath,
    createGraphEdge,
    deleteGraphEdge,
    discoverGraphEdges,
    discoverGraphEdgesForNode,
    getGraphDiscoveryStatus
  };
});

