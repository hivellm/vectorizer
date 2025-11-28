/**
 * Graph models for the Hive Vectorizer SDK.
 * 
 * Models for graph operations including nodes, edges, and relationships.
 */

/**
 * Graph node representing a document/file
 */
export interface GraphNode {
  /** Unique node identifier */
  id: string;
  /** Node type (e.g., "document", "file", "chunk") */
  node_type: string;
  /** Node metadata */
  metadata: Record<string, any>;
}

/**
 * Graph edge representing a relationship between nodes
 */
export interface GraphEdge {
  /** Edge identifier */
  id: string;
  /** Source node ID */
  source: string;
  /** Target node ID */
  target: string;
  /** Relationship type */
  relationship_type: string;
  /** Edge weight (0.0 to 1.0) */
  weight: number;
  /** Edge metadata */
  metadata: Record<string, any>;
  /** Creation timestamp */
  created_at: string;
}

/**
 * Neighbor information
 */
export interface NeighborInfo {
  /** Neighbor node */
  node: GraphNode;
  /** Edge connecting to neighbor */
  edge: GraphEdge;
}

/**
 * Related node information
 */
export interface RelatedNodeInfo {
  /** Related node */
  node: GraphNode;
  /** Distance in hops */
  distance: number;
  /** Relationship weight */
  weight: number;
}

/**
 * Request to find related nodes
 */
export interface FindRelatedRequest {
  /** Maximum number of hops */
  max_hops?: number;
  /** Relationship type filter */
  relationship_type?: string;
}

/**
 * Response for finding related nodes
 */
export interface FindRelatedResponse {
  /** List of related nodes */
  related: RelatedNodeInfo[];
}

/**
 * Request to find path between nodes
 */
export interface FindPathRequest {
  /** Collection name */
  collection: string;
  /** Source node ID */
  source: string;
  /** Target node ID */
  target: string;
}

/**
 * Response for finding path
 */
export interface FindPathResponse {
  /** Path as list of nodes */
  path: GraphNode[];
  /** Whether path was found */
  found: boolean;
}

/**
 * Request to create an edge
 */
export interface CreateEdgeRequest {
  /** Collection name */
  collection: string;
  /** Source node ID */
  source: string;
  /** Target node ID */
  target: string;
  /** Relationship type */
  relationship_type: string;
  /** Optional edge weight */
  weight?: number;
}

/**
 * Response for creating an edge
 */
export interface CreateEdgeResponse {
  /** Created edge ID */
  edge_id: string;
  /** Success status */
  success: boolean;
  /** Status message */
  message: string;
}

/**
 * Response for listing nodes
 */
export interface ListNodesResponse {
  /** List of nodes */
  nodes: GraphNode[];
  /** Total count */
  count: number;
}

/**
 * Response for getting neighbors
 */
export interface GetNeighborsResponse {
  /** List of neighbors */
  neighbors: NeighborInfo[];
}

/**
 * Response for listing edges
 */
export interface ListEdgesResponse {
  /** List of edges */
  edges: GraphEdge[];
  /** Total count */
  count: number;
}

/**
 * Request to discover edges
 */
export interface DiscoverEdgesRequest {
  /** Similarity threshold (0.0 to 1.0) */
  similarity_threshold?: number;
  /** Maximum edges per node */
  max_per_node?: number;
}

/**
 * Response for discovering edges
 */
export interface DiscoverEdgesResponse {
  /** Success status */
  success: boolean;
  /** Number of edges created */
  edges_created: number;
  /** Status message */
  message: string;
}

/**
 * Response for discovery status
 */
export interface DiscoveryStatusResponse {
  /** Total number of nodes */
  total_nodes: number;
  /** Number of nodes with edges */
  nodes_with_edges: number;
  /** Total number of edges */
  total_edges: number;
  /** Progress percentage */
  progress_percentage: number;
}

