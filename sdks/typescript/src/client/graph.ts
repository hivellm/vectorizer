/**
 * Graph surface: nodes, neighbours, edges, paths, and discovery.
 *
 * The discovery endpoints (`discoverGraphEdges*`) seed the graph with
 * SIMILAR_TO edges from existing vector neighbourhoods; the path /
 * neighbour calls then exploit that structure for traversal.
 */

import { BaseClient } from './_base';
import {
  CreateEdgeRequest,
  CreateEdgeResponse,
  DiscoverEdgesRequest,
  DiscoverEdgesResponse,
  DiscoveryStatusResponse,
  FindPathRequest,
  FindPathResponse,
  FindRelatedRequest,
  FindRelatedResponse,
  GetNeighborsResponse,
  ListEdgesResponse,
  ListNodesResponse,
} from '../models';

export class GraphClient extends BaseClient {
  /** List every node in the collection's graph. */
  public async listGraphNodes(collection: string): Promise<ListNodesResponse> {
    this.logger.debug('Listing graph nodes', { collection });
    return this.transport.get(`/graph/nodes/${collection}`);
  }

  /** Direct neighbours of a node. */
  public async getGraphNeighbors(
    collection: string,
    nodeId: string,
  ): Promise<GetNeighborsResponse> {
    this.logger.debug('Getting graph neighbors', { collection, nodeId });
    return this.transport.get(`/graph/nodes/${collection}/${nodeId}/neighbors`);
  }

  /** Find related nodes within N hops. */
  public async findRelatedNodes(
    collection: string,
    nodeId: string,
    request: FindRelatedRequest,
  ): Promise<FindRelatedResponse> {
    this.logger.debug('Finding related nodes', { collection, nodeId, request });
    return this.transport.post(`/graph/nodes/${collection}/${nodeId}/related`, request);
  }

  /** Shortest path between two nodes. */
  public async findGraphPath(request: FindPathRequest): Promise<FindPathResponse> {
    this.logger.debug('Finding graph path', { request });
    return this.transport.post('/graph/path', request);
  }

  /** Create an explicit edge between two nodes. */
  public async createGraphEdge(request: CreateEdgeRequest): Promise<CreateEdgeResponse> {
    this.logger.debug('Creating graph edge', { request });
    return this.transport.post('/graph/edges', request);
  }

  /** Delete an edge by ID. */
  public async deleteGraphEdge(edgeId: string): Promise<void> {
    this.logger.debug('Deleting graph edge', { edgeId });
    return this.transport.delete(`/graph/edges/${edgeId}`);
  }

  /** List every edge in a collection. */
  public async listGraphEdges(collection: string): Promise<ListEdgesResponse> {
    this.logger.debug('Listing graph edges', { collection });
    return this.transport.get(`/graph/collections/${collection}/edges`);
  }

  /** Discover SIMILAR_TO edges across the whole collection. */
  public async discoverGraphEdges(
    collection: string,
    request: DiscoverEdgesRequest,
  ): Promise<DiscoverEdgesResponse> {
    this.logger.debug('Discovering graph edges', { collection, request });
    return this.transport.post(`/graph/discover/${collection}`, request);
  }

  /** Discover SIMILAR_TO edges seeded at a single node. */
  public async discoverGraphEdgesForNode(
    collection: string,
    nodeId: string,
    request: DiscoverEdgesRequest,
  ): Promise<DiscoverEdgesResponse> {
    this.logger.debug('Discovering graph edges for node', { collection, nodeId, request });
    return this.transport.post(`/graph/discover/${collection}/${nodeId}`, request);
  }

  /** Inspect background discovery progress for a collection. */
  public async getGraphDiscoveryStatus(collection: string): Promise<DiscoveryStatusResponse> {
    this.logger.debug('Getting graph discovery status', { collection });
    return this.transport.get(`/graph/discover/${collection}/status`);
  }
}
