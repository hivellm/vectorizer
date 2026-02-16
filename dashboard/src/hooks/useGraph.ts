/**
 * Hook for graph API operations
 */

import { useCallback } from 'react';
import { useApiClient } from './useApiClient';
import { ApiClientError } from '@/lib/api-client';

export interface GraphNode {
  id: string;
  node_type: string;
  metadata: Record<string, unknown>;
  created_at?: string;
}

export interface GraphEdge {
  id: string;
  source: string;
  target: string;
  relationship_type: string;
  weight: number;
  metadata: Record<string, unknown>;
  created_at: string;
}

export interface NeighborInfo {
  node: GraphNode;
  edge: GraphEdge;
}

export interface RelatedNodeInfo {
  node: GraphNode;
  distance: number;
  weight: number;
}

export interface ListNodesResponse {
  nodes: GraphNode[];
  count: number;
}

export interface ListEdgesResponse {
  edges: GraphEdge[];
  count: number;
}

export interface FindRelatedRequest {
  max_hops?: number;
  relationship_type?: string;
}

export interface FindPathRequest {
  collection: string;
  source: string;
  target: string;
}

export interface FindPathResponse {
  path: GraphNode[];
  found: boolean;
}

export interface DiscoverEdgesRequest {
  similarity_threshold?: number;
  max_per_node?: number;
}

export interface DiscoverEdgesResponse {
  success: boolean;
  edges_created: number;
  message: string;
}

export interface DiscoveryStatusResponse {
  total_nodes: number;
  nodes_with_edges: number;
  total_edges: number;
  discovery_progress?: number;
}

export interface EnableGraphResponse {
  success: boolean;
  collection: string;
  message: string;
  node_count: number;
}

export interface GraphStatusResponse {
  collection: string;
  enabled: boolean;
  node_count: number;
  edge_count: number;
}

/**
 * Hook for graph operations
 */
export function useGraph() {
  const apiClient = useApiClient();

  /**
   * List all nodes in a collection
   */
  const listNodes = useCallback(
    async (collection: string): Promise<ListNodesResponse> => {
      try {
        const response = await apiClient.get<ListNodesResponse>(
          `/graph/nodes/${encodeURIComponent(collection)}`
        );
        
        // Validate response
        if (!response || typeof response !== 'object') {
          throw new Error('Invalid response from API: response is not an object');
        }
        
        if (!Array.isArray(response.nodes)) {
          throw new Error('Invalid response from API: nodes is not an array');
        }
        
        return {
          nodes: response.nodes || [],
          count: response.count ?? response.nodes.length,
        };
      } catch (error) {
        if (error instanceof ApiClientError) {
          throw new Error(`Failed to list nodes: ${error.message}`);
        }
        throw error;
      }
    },
    [apiClient]
  );

  /**
   * List all edges in a collection
   * @param collection Collection name
   * @param limit Optional limit (default: no limit - returns all edges)
   */
  const listEdges = useCallback(
    async (collection: string, limit?: number): Promise<ListEdgesResponse> => {
      try {
        const url = limit 
          ? `/graph/collections/${encodeURIComponent(collection)}/edges?limit=${limit}`
          : `/graph/collections/${encodeURIComponent(collection)}/edges`;
        const response = await apiClient.get<ListEdgesResponse>(url);
        
        // Validate response
        if (!response || typeof response !== 'object') {
          throw new Error('Invalid response from API: response is not an object');
        }
        
        if (!Array.isArray(response.edges)) {
          throw new Error('Invalid response from API: edges is not an array');
        }
        
        return {
          edges: response.edges || [],
          count: response.count ?? response.edges.length,
        };
      } catch (error) {
        if (error instanceof ApiClientError) {
          throw new Error(`Failed to list edges: ${error.message}`);
        }
        throw error;
      }
    },
    [apiClient]
  );

  /**
   * Get neighbors of a node
   */
  const getNeighbors = useCallback(
    async (collection: string, nodeId: string): Promise<NeighborInfo[]> => {
      try {
        const response = await apiClient.get<{ neighbors: NeighborInfo[] }>(
          `/graph/nodes/${encodeURIComponent(collection)}/${encodeURIComponent(nodeId)}/neighbors`
        );
        return response.neighbors;
      } catch (error) {
        if (error instanceof ApiClientError) {
          throw new Error(`Failed to get neighbors: ${error.message}`);
        }
        throw error;
      }
    },
    [apiClient]
  );

  /**
   * Find related nodes within N hops
   */
  const findRelated = useCallback(
    async (
      collection: string,
      nodeId: string,
      request: FindRelatedRequest = {}
    ): Promise<RelatedNodeInfo[]> => {
      try {
        const response = await apiClient.post<{ related: RelatedNodeInfo[] }>(
          `/graph/nodes/${encodeURIComponent(collection)}/${encodeURIComponent(nodeId)}/related`,
          request
        );
        return response.related;
      } catch (error) {
        if (error instanceof ApiClientError) {
          throw new Error(`Failed to find related nodes: ${error.message}`);
        }
        throw error;
      }
    },
    [apiClient]
  );

  /**
   * Find shortest path between two nodes
   */
  const findPath = useCallback(
    async (
      collection: string,
      source: string,
      target: string
    ): Promise<FindPathResponse> => {
      try {
        const response = await apiClient.post<FindPathResponse>(
          '/graph/path',
          {
            collection,
            source,
            target,
          }
        );
        return response;
      } catch (error) {
        if (error instanceof ApiClientError) {
          throw new Error(`Failed to find path: ${error.message}`);
        }
        throw error;
      }
    },
    [apiClient]
  );

  /**
   * Create an edge between two nodes
   */
  const createEdge = useCallback(
    async (
      collection: string,
      source: string,
      target: string,
      relationshipType: string,
      weight?: number
    ): Promise<string> => {
      try {
        const response = await apiClient.post<{ edge_id: string; success: boolean; message: string }>(
          '/graph/edges',
          {
            collection,
            source,
            target,
            relationship_type: relationshipType,
            weight,
          }
        );
        if (!response.success) {
          throw new Error(response.message || 'Failed to create edge');
        }
        return response.edge_id;
      } catch (error) {
        if (error instanceof ApiClientError) {
          throw new Error(`Failed to create edge: ${error.message}`);
        }
        throw error;
      }
    },
    [apiClient]
  );

  /**
   * Delete an edge
   */
  const deleteEdge = useCallback(
    async (edgeId: string): Promise<void> => {
      try {
        await apiClient.delete(`/graph/edges/${encodeURIComponent(edgeId)}`);
      } catch (error) {
        if (error instanceof ApiClientError) {
          throw new Error(`Failed to delete edge: ${error.message}`);
        }
        throw error;
      }
    },
    [apiClient]
  );

  /**
   * Discover edges for entire collection
   */
  const discoverEdges = useCallback(
    async (
      collection: string,
      request: DiscoverEdgesRequest = {}
    ): Promise<DiscoverEdgesResponse> => {
      try {
        const response = await apiClient.post<DiscoverEdgesResponse>(
          `/graph/discover/${encodeURIComponent(collection)}`,
          request
        );
        return response;
      } catch (error) {
        if (error instanceof ApiClientError) {
          throw new Error(`Failed to discover edges: ${error.message}`);
        }
        throw error;
      }
    },
    [apiClient]
  );

  /**
   * Discover edges for a specific node
   */
  const discoverEdgesForNode = useCallback(
    async (
      collection: string,
      nodeId: string,
      request: DiscoverEdgesRequest = {}
    ): Promise<DiscoverEdgesResponse> => {
      try {
        const response = await apiClient.post<DiscoverEdgesResponse>(
          `/graph/discover/${encodeURIComponent(collection)}/${encodeURIComponent(nodeId)}`,
          request
        );
        return response;
      } catch (error) {
        if (error instanceof ApiClientError) {
          throw new Error(`Failed to discover edges for node: ${error.message}`);
        }
        throw error;
      }
    },
    [apiClient]
  );

  /**
   * Get discovery status for a collection
   */
  const getDiscoveryStatus = useCallback(
    async (collection: string): Promise<DiscoveryStatusResponse> => {
      try {
        const response = await apiClient.get<DiscoveryStatusResponse>(
          `/graph/discover/${encodeURIComponent(collection)}/status`
        );
        return response;
      } catch (error) {
        if (error instanceof ApiClientError) {
          throw new Error(`Failed to get discovery status: ${error.message}`);
        }
        throw error;
      }
    },
    [apiClient]
  );

  /**
   * Enable graph for a collection
   */
  const enableGraph = useCallback(
    async (collection: string): Promise<EnableGraphResponse> => {
      try {
        const response = await apiClient.post<EnableGraphResponse>(
          `/graph/enable/${encodeURIComponent(collection)}`,
          {}
        );
        return response;
      } catch (error) {
        if (error instanceof ApiClientError) {
          throw new Error(`Failed to enable graph: ${error.message}`);
        }
        throw error;
      }
    },
    [apiClient]
  );

  /**
   * Get graph status for a collection
   */
  const getGraphStatus = useCallback(
    async (collection: string): Promise<GraphStatusResponse> => {
      try {
        const response = await apiClient.get<GraphStatusResponse>(
          `/graph/status/${encodeURIComponent(collection)}`
        );
        return response;
      } catch (error) {
        if (error instanceof ApiClientError) {
          throw new Error(`Failed to get graph status: ${error.message}`);
        }
        throw error;
      }
    },
    [apiClient]
  );

  return {
    listNodes,
    listEdges,
    getNeighbors,
    findRelated,
    findPath,
    createEdge,
    deleteEdge,
    discoverEdges,
    discoverEdgesForNode,
    getDiscoveryStatus,
    enableGraph,
    getGraphStatus,
  };
}

