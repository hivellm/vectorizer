/**
 * Graph models for the Hive Vectorizer SDK.
 * 
 * Models for graph operations including nodes, edges, and relationships.
 */

import { ValidationError } from '../exceptions/index.js';
import {
  validateNonEmptyString,
  validateNumberRange,
  validateOptional,
  validateRequired,
} from '../utils/validation.js';

/**
 * Graph node representing a document/file
 * @typedef {Object} GraphNode
 * @property {string} id - Unique node identifier
 * @property {string} node_type - Node type (e.g., "document", "file", "chunk")
 * @property {Object<string, any>} metadata - Node metadata
 */

/**
 * Graph edge representing a relationship between nodes
 * @typedef {Object} GraphEdge
 * @property {string} id - Edge identifier
 * @property {string} source - Source node ID
 * @property {string} target - Target node ID
 * @property {string} relationship_type - Relationship type
 * @property {number} weight - Edge weight (0.0 to 1.0)
 * @property {Object<string, any>} metadata - Edge metadata
 * @property {string} created_at - Creation timestamp
 */

/**
 * Neighbor information
 * @typedef {Object} NeighborInfo
 * @property {GraphNode} node - Neighbor node
 * @property {GraphEdge} edge - Edge connecting to neighbor
 */

/**
 * Related node information
 * @typedef {Object} RelatedNodeInfo
 * @property {GraphNode} node - Related node
 * @property {number} distance - Distance in hops
 * @property {number} weight - Relationship weight
 */

/**
 * Request to find related nodes
 * @typedef {Object} FindRelatedRequest
 * @property {number} [max_hops] - Maximum number of hops
 * @property {string} [relationship_type] - Relationship type filter
 */

/**
 * Response for finding related nodes
 * @typedef {Object} FindRelatedResponse
 * @property {RelatedNodeInfo[]} related - List of related nodes
 */

/**
 * Request to find path between nodes
 * @typedef {Object} FindPathRequest
 * @property {string} collection - Collection name
 * @property {string} source - Source node ID
 * @property {string} target - Target node ID
 */

/**
 * Response for finding path
 * @typedef {Object} FindPathResponse
 * @property {GraphNode[]} path - Path as list of nodes
 * @property {boolean} found - Whether path was found
 */

/**
 * Request to create an edge
 * @typedef {Object} CreateEdgeRequest
 * @property {string} collection - Collection name
 * @property {string} source - Source node ID
 * @property {string} target - Target node ID
 * @property {string} relationship_type - Relationship type
 * @property {number} [weight] - Optional edge weight
 */

/**
 * Response for creating an edge
 * @typedef {Object} CreateEdgeResponse
 * @property {string} edge_id - Created edge ID
 * @property {boolean} success - Success status
 * @property {string} message - Status message
 */

/**
 * Response for listing nodes
 * @typedef {Object} ListNodesResponse
 * @property {GraphNode[]} nodes - List of nodes
 * @property {number} count - Total count
 */

/**
 * Response for getting neighbors
 * @typedef {Object} GetNeighborsResponse
 * @property {NeighborInfo[]} neighbors - List of neighbors
 */

/**
 * Response for listing edges
 * @typedef {Object} ListEdgesResponse
 * @property {GraphEdge[]} edges - List of edges
 * @property {number} count - Total count
 */

/**
 * Request to discover edges
 * @typedef {Object} DiscoverEdgesRequest
 * @property {number} [similarity_threshold] - Similarity threshold (0.0 to 1.0)
 * @property {number} [max_per_node] - Maximum edges per node
 */

/**
 * Response for discovering edges
 * @typedef {Object} DiscoverEdgesResponse
 * @property {boolean} success - Success status
 * @property {number} edges_created - Number of edges created
 * @property {string} message - Status message
 */

/**
 * Response for discovery status
 * @typedef {Object} DiscoveryStatusResponse
 * @property {number} total_nodes - Total number of nodes
 * @property {number} nodes_with_edges - Number of nodes with edges
 * @property {number} total_edges - Total number of edges
 * @property {number} progress_percentage - Progress percentage
 */

/**
 * Validates a FindRelatedRequest object
 * @param {Partial<FindRelatedRequest>} request - The request to validate
 * @throws {ValidationError} If validation fails
 */
export function validateFindRelatedRequest(request) {
  if (!request || typeof request !== 'object') {
    throw new ValidationError('FindRelatedRequest must be an object');
  }

  if (request.max_hops !== undefined) {
    if (typeof request.max_hops !== 'number' || request.max_hops < 1 || !Number.isInteger(request.max_hops)) {
      throw new ValidationError('max_hops must be a positive integer');
    }
  }

  if (request.relationship_type !== undefined) {
    validateNonEmptyString(request.relationship_type, 'relationship_type');
  }
}

/**
 * Validates a FindPathRequest object
 * @param {Partial<FindPathRequest>} request - The request to validate
 * @throws {ValidationError} If validation fails
 */
export function validateFindPathRequest(request) {
  if (!request || typeof request !== 'object') {
    throw new ValidationError('FindPathRequest must be an object');
  }

  validateRequired(request.collection, 'collection');
  validateNonEmptyString(request.collection, 'collection');

  validateRequired(request.source, 'source');
  validateNonEmptyString(request.source, 'source');

  validateRequired(request.target, 'target');
  validateNonEmptyString(request.target, 'target');
}

/**
 * Validates a CreateEdgeRequest object
 * @param {Partial<CreateEdgeRequest>} request - The request to validate
 * @throws {ValidationError} If validation fails
 */
export function validateCreateEdgeRequest(request) {
  if (!request || typeof request !== 'object') {
    throw new ValidationError('CreateEdgeRequest must be an object');
  }

  validateRequired(request.collection, 'collection');
  validateNonEmptyString(request.collection, 'collection');

  validateRequired(request.source, 'source');
  validateNonEmptyString(request.source, 'source');

  validateRequired(request.target, 'target');
  validateNonEmptyString(request.target, 'target');

  validateRequired(request.relationship_type, 'relationship_type');
  validateNonEmptyString(request.relationship_type, 'relationship_type');

  if (request.weight !== undefined) {
    validateNumberRange(request.weight, 'weight', 0.0, 1.0);
  }
}

/**
 * Validates a DiscoverEdgesRequest object
 * @param {Partial<DiscoverEdgesRequest>} request - The request to validate
 * @throws {ValidationError} If validation fails
 */
export function validateDiscoverEdgesRequest(request) {
  if (!request || typeof request !== 'object') {
    throw new ValidationError('DiscoverEdgesRequest must be an object');
  }

  if (request.similarity_threshold !== undefined) {
    validateNumberRange(request.similarity_threshold, 'similarity_threshold', 0.0, 1.0);
  }

  if (request.max_per_node !== undefined) {
    if (typeof request.max_per_node !== 'number' || request.max_per_node < 1 || !Number.isInteger(request.max_per_node)) {
      throw new ValidationError('max_per_node must be a positive integer');
    }
  }
}

