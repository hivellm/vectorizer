/**
 * Main client class for the Hive Vectorizer SDK.
 * 
 * Provides high-level methods for vector operations, semantic search,
 * and collection management.
 */

import { HttpClientConfig } from './utils/http-client';
import { UMICPClientConfig } from './utils/umicp-client';
import { ITransport, TransportFactory, TransportProtocol, parseConnectionString } from './utils/transport';
import { createLogger, Logger, LoggerConfig } from './utils/logger';

import {
  Vector,
  CreateVectorRequest,
  UpdateVectorRequest,
  Collection,
  CreateCollectionRequest,
  UpdateCollectionRequest,
  SearchResponse,
  EmbeddingRequest,
  EmbeddingResponse,
  SearchRequest,
  TextSearchRequest,
  CollectionInfo,
  DatabaseStats,
  BatchInsertRequest,
  BatchSearchRequest,
  BatchUpdateRequest,
  BatchDeleteRequest,
  BatchResponse,
  BatchSearchResponse,
  // Intelligent search models
  IntelligentSearchRequest,
  IntelligentSearchResponse,
  SemanticSearchRequest,
  SemanticSearchResponse,
  ContextualSearchRequest,
  ContextualSearchResponse,
  MultiCollectionSearchRequest,
  MultiCollectionSearchResponse,
  // Hybrid search models
  HybridSearchRequest,
  HybridSearchResponse,
  validateHybridSearchRequest,
  // Graph models
  FindRelatedRequest,
  FindRelatedResponse,
  FindPathRequest,
  FindPathResponse,
  CreateEdgeRequest,
  CreateEdgeResponse,
  ListNodesResponse,
  GetNeighborsResponse,
  ListEdgesResponse,
  DiscoverEdgesRequest,
  DiscoverEdgesResponse,
  DiscoveryStatusResponse,
  // File upload models
  FileUploadResponse,
  FileUploadConfig,
  UploadFileOptions,
  // Replication/routing models
  ReadPreference,
  HostConfig,
  ReadOptions,
} from './models';


import {
  validateVector,
  validateCreateVectorRequest,
  validateCollection,
  validateCreateCollectionRequest,
  validateSearchResponse,
  validateEmbeddingRequest,
  validateEmbeddingResponse,
  validateSearchRequest,
  validateTextSearchRequest,
  validateCollectionInfo,
  validateDatabaseStats,
} from './models';

export interface VectorizerClientConfig {
  /** Base URL for the Vectorizer API (HTTP/HTTPS) - for single node deployments */
  baseURL?: string;
  /** Host configuration for master/replica topology */
  hosts?: HostConfig;
  /** Read preference for routing read operations (default: 'replica' if hosts configured, otherwise N/A) */
  readPreference?: ReadPreference;
  /** Connection string (supports http://, https://, umicp://) */
  connectionString?: string;
  /** Transport protocol to use */
  protocol?: TransportProtocol;
  /** API key for authentication */
  apiKey?: string;
  /** Request timeout in milliseconds */
  timeout?: number;
  /** Custom headers for requests (HTTP only) */
  headers?: Record<string, string>;
  /** UMICP-specific configuration */
  umicp?: Partial<UMICPClientConfig>;
  /** Logger configuration */
  logger?: LoggerConfig;
}

export class VectorizerClient {
  private transport!: ITransport;
  private masterTransport?: ITransport;
  private replicaTransports: ITransport[] = [];
  private replicaIndex: number = 0;
  private logger: Logger;
  private config: VectorizerClientConfig;
  private protocol!: TransportProtocol;
  private readPreference: ReadPreference = 'replica';
  private isReplicaMode: boolean = false;

  constructor(config: VectorizerClientConfig = {}) {
    this.config = {
      baseURL: 'http://localhost:15002',
      timeout: 30000,
      headers: {},
      logger: { level: 'info', enabled: true },
      readPreference: 'replica',
      ...config,
    };

    this.logger = createLogger(this.config.logger);
    this.readPreference = this.config.readPreference || 'replica';

    // Check if using master/replica configuration
    if (this.config.hosts) {
      this.initializeReplicaMode();
      return;
    }

    // Determine protocol and create transport
    if (this.config.connectionString) {
      // Use connection string
      const transportConfig = parseConnectionString(this.config.connectionString, this.config.apiKey);
      this.transport = TransportFactory.create(transportConfig);
      this.protocol = transportConfig.protocol!;

      this.logger.info('VectorizerClient initialized from connection string', {
        protocol: this.protocol,
        connectionString: this.config.connectionString,
        hasApiKey: !!this.config.apiKey,
      });
    } else {
      // Use explicit configuration
      this.protocol = this.config.protocol || 'http';

      if (this.protocol === 'http') {
        const httpConfig: HttpClientConfig = {
          baseURL: this.config.baseURL!,
          ...(this.config.timeout && { timeout: this.config.timeout }),
          ...(this.config.headers && { headers: this.config.headers }),
          ...(this.config.apiKey && { apiKey: this.config.apiKey }),
        };
        this.transport = TransportFactory.create({ protocol: 'http', http: httpConfig });

        this.logger.info('VectorizerClient initialized with HTTP', {
          baseURL: this.config.baseURL,
          hasApiKey: !!this.config.apiKey,
        });
      } else if (this.protocol === 'umicp') {
        if (!this.config.umicp) {
          throw new Error('UMICP configuration is required when using UMICP protocol');
        }

        const umicpConfig: UMICPClientConfig = {
          host: this.config.umicp.host || 'localhost',
          port: this.config.umicp.port || 15003,
          ...(this.config.apiKey && { apiKey: this.config.apiKey }),
          ...(this.config.timeout && { timeout: this.config.timeout }),
          ...this.config.umicp,
        };
        this.transport = TransportFactory.create({ protocol: 'umicp', umicp: umicpConfig });

        this.logger.info('VectorizerClient initialized with UMICP', {
          host: umicpConfig.host,
          port: umicpConfig.port,
          hasApiKey: !!this.config.apiKey,
        });
      }
    }
  }

  /**
   * Initialize replica mode with master and replica transports.
   */
  private initializeReplicaMode(): void {
    const { hosts } = this.config;
    if (!hosts) return;

    this.isReplicaMode = true;
    this.protocol = 'http';

    // Create master transport
    const masterHttpConfig: HttpClientConfig = {
      baseURL: hosts.master,
      ...(this.config.timeout && { timeout: this.config.timeout }),
      ...(this.config.headers && { headers: this.config.headers }),
      ...(this.config.apiKey && { apiKey: this.config.apiKey }),
    };
    this.masterTransport = TransportFactory.create({ protocol: 'http', http: masterHttpConfig });

    // Create replica transports
    this.replicaTransports = hosts.replicas.map(replicaUrl => {
      const replicaHttpConfig: HttpClientConfig = {
        baseURL: replicaUrl,
        ...(this.config.timeout && { timeout: this.config.timeout }),
        ...(this.config.headers && { headers: this.config.headers }),
        ...(this.config.apiKey && { apiKey: this.config.apiKey }),
      };
      return TransportFactory.create({ protocol: 'http', http: replicaHttpConfig });
    });

    // Default transport is master (for backward compatibility with internal methods)
    this.transport = this.masterTransport;

    this.logger.info('VectorizerClient initialized with master/replica topology', {
      master: hosts.master,
      replicas: hosts.replicas,
      readPreference: this.readPreference,
      hasApiKey: !!this.config.apiKey,
    });
  }

  /**
   * Get transport for write operations (always master).
   */
  private getWriteTransport(): ITransport {
    if (this.isReplicaMode && this.masterTransport) {
      return this.masterTransport;
    }
    return this.transport;
  }

  /**
   * Get transport for read operations based on read preference.
   */
  private getReadTransport(options?: ReadOptions): ITransport {
    if (!this.isReplicaMode) {
      return this.transport;
    }

    const preference = options?.readPreference || this.readPreference;

    switch (preference) {
      case 'master':
        return this.masterTransport!;

      case 'replica':
        if (this.replicaTransports.length === 0) {
          // Fallback to master if no replicas configured
          return this.masterTransport!;
        }
        // Round-robin selection
        const transport = this.replicaTransports[this.replicaIndex]!;
        this.replicaIndex = (this.replicaIndex + 1) % this.replicaTransports.length;
        return transport;

      case 'nearest':
        // For now, use round-robin as a simple implementation
        // Future: implement latency-based selection
        if (this.replicaTransports.length === 0) {
          return this.masterTransport!;
        }
        const nearestTransport = this.replicaTransports[this.replicaIndex]!;
        this.replicaIndex = (this.replicaIndex + 1) % this.replicaTransports.length;
        return nearestTransport;

      default:
        return this.masterTransport!;
    }
  }

  /**
   * Execute a callback with master transport for read-your-writes scenarios.
   * All operations within the callback will be routed to master.
   */
  public async withMaster<T>(callback: (client: VectorizerClient) => Promise<T>): Promise<T> {
    // Create a temporary client configured to always use master
    const masterClient = new VectorizerClient({
      ...this.config,
      readPreference: 'master',
    });

    try {
      return await callback(masterClient);
    } finally {
      // Cleanup if needed
    }
  }

  /**
   * Get the current transport protocol being used.
   */
  public getProtocol(): TransportProtocol {
    return this.protocol;
  }

  // ===== HEALTH & STATUS =====

  /**
   * Check if the server is healthy.
   */
  public async healthCheck(): Promise<{ status: string; timestamp: string }> {
    try {
      const response = await this.transport.get<{ status: string; timestamp: string }>('/health');
      this.logger.debug('Health check successful', response);
      return response;
    } catch (error) {
      this.logger.error('Health check failed', error);
      throw error;
    }
  }

  /**
   * Get database statistics.
   */
  public async getDatabaseStats(): Promise<DatabaseStats> {
    try {
      const response = await this.transport.get<DatabaseStats>('/stats');
      validateDatabaseStats(response);
      this.logger.debug('Database stats retrieved', response);
      return response;
    } catch (error) {
      this.logger.error('Failed to get database stats', error);
      throw error;
    }
  }

  // ===== COLLECTION MANAGEMENT =====

  /**
   * List all collections.
   * @param options - Optional read options for routing override
   */
  public async listCollections(options?: ReadOptions): Promise<Collection[]> {
    try {
      const transport = this.getReadTransport(options);
      const response = await transport.get<{ collections: Collection[] } | Collection[]>('/collections');
      // Handle both array and {collections: [...]} response formats
      const collections = Array.isArray(response) ? response : (response.collections || []);
      this.logger.debug('Collections listed', { count: collections.length });
      return collections;
    } catch (error) {
      this.logger.error('Failed to list collections', error);
      throw error;
    }
  }

  /**
   * Get collection information.
   * @param options - Optional read options for routing override
   */
  public async getCollection(collectionName: string, options?: ReadOptions): Promise<CollectionInfo> {
    try {
      const transport = this.getReadTransport(options);
      const response = await transport.get<CollectionInfo>(`/collections/${collectionName}`);
      validateCollectionInfo(response);
      this.logger.debug('Collection info retrieved', { collectionName });
      return response;
    } catch (error) {
      this.logger.error('Failed to get collection info', { collectionName, error });
      throw error;
    }
  }

  /**
   * Create a new collection.
   * (Write operation - always routed to master)
   */
  public async createCollection(request: CreateCollectionRequest): Promise<Collection> {
    try {
      validateCreateCollectionRequest(request);
      const transport = this.getWriteTransport();
      const response = await transport.post<Collection>('/collections', request);
      validateCollection(response);
      this.logger.info('Collection created', { collectionName: request.name });
      return response;
    } catch (error) {
      this.logger.error('Failed to create collection', { request, error });
      throw error;
    }
  }

  /**
   * Update an existing collection.
   * (Write operation - always routed to master)
   */
  public async updateCollection(collectionName: string, request: UpdateCollectionRequest): Promise<Collection> {
    try {
      const transport = this.getWriteTransport();
      const response = await transport.put<Collection>(`/collections/${collectionName}`, request);
      validateCollection(response);
      this.logger.info('Collection updated', { collectionName });
      return response;
    } catch (error) {
      this.logger.error('Failed to update collection', { collectionName, request, error });
      throw error;
    }
  }

  /**
   * Delete a collection.
   * (Write operation - always routed to master)
   */
  public async deleteCollection(collectionName: string): Promise<void> {
    try {
      const transport = this.getWriteTransport();
      await transport.delete(`/collections/${collectionName}`);
      this.logger.info('Collection deleted', { collectionName });
    } catch (error) {
      this.logger.error('Failed to delete collection', { collectionName, error });
      throw error;
    }
  }

  // ===== VECTOR OPERATIONS =====

  /**
   * Insert vectors into a collection.
   * (Write operation - always routed to master)
   */
  public async insertVectors(collectionName: string, vectors: CreateVectorRequest[], publicKey?: string): Promise<{ inserted: number }> {
    try {
      vectors.forEach(validateCreateVectorRequest);
      const transport = this.getWriteTransport();
      // Use Qdrant-compatible API for inserting points
      const points = vectors.map((v, idx) => ({
        id: v.id ?? `${Date.now()}-${idx}`,
        vector: v.data,
        payload: v.metadata ?? {}
      }));
      const payload: any = { points };
      // Use publicKey from parameter or from first vector that has it
      const effectivePublicKey = publicKey || vectors.find(v => v.publicKey)?.publicKey;
      if (effectivePublicKey) {
        payload.public_key = effectivePublicKey;
      }
      await transport.put<{ status?: string; result?: { operation_id?: number; status?: string } }>(
        `/qdrant/collections/${collectionName}/points`,
        payload
      );
      this.logger.info('Vectors inserted', { collectionName, count: vectors.length, encrypted: !!effectivePublicKey });
      return { inserted: vectors.length };
    } catch (error) {
      this.logger.error('Failed to insert vectors', { collectionName, count: vectors.length, error });
      throw error;
    }
  }

  /**
   * Get a vector by ID.
   * @param options - Optional read options for routing override
   */
  public async getVector(collectionName: string, vectorId: string, options?: ReadOptions): Promise<Vector> {
    try {
      const transport = this.getReadTransport(options);
      const response = await transport.get<Vector>(`/collections/${collectionName}/vectors/${vectorId}`);
      validateVector(response);
      this.logger.debug('Vector retrieved', { collectionName, vectorId });
      return response;
    } catch (error) {
      this.logger.error('Failed to get vector', { collectionName, vectorId, error });
      throw error;
    }
  }

  /**
   * Update a vector.
   * (Write operation - always routed to master)
   */
  public async updateVector(collectionName: string, vectorId: string, request: UpdateVectorRequest): Promise<Vector> {
    try {
      const transport = this.getWriteTransport();
      const payload: any = { ...request };
      if (request.publicKey) {
        payload.public_key = request.publicKey;
        delete payload.publicKey;
      }
      const response = await transport.put<Vector>(
        `/collections/${collectionName}/vectors/${vectorId}`,
        payload
      );
      validateVector(response);
      this.logger.info('Vector updated', { collectionName, vectorId, encrypted: !!request.publicKey });
      return response;
    } catch (error) {
      this.logger.error('Failed to update vector', { collectionName, vectorId, request, error });
      throw error;
    }
  }

  /**
   * Delete a vector.
   * (Write operation - always routed to master)
   */
  public async deleteVector(collectionName: string, vectorId: string): Promise<void> {
    try {
      const transport = this.getWriteTransport();
      await transport.delete(`/collections/${collectionName}/vectors/${vectorId}`);
      this.logger.info('Vector deleted', { collectionName, vectorId });
    } catch (error) {
      this.logger.error('Failed to delete vector', { collectionName, vectorId, error });
      throw error;
    }
  }

  /**
   * Delete multiple vectors.
   * (Write operation - always routed to master)
   */
  public async deleteVectors(collectionName: string, vectorIds: string[]): Promise<{ deleted: number }> {
    try {
      const transport = this.getWriteTransport();
      const response = await transport.post<{ deleted: number }>(
        `/collections/${collectionName}/vectors/delete`,
        { vector_ids: vectorIds }
      );
      this.logger.info('Vectors deleted', { collectionName, count: vectorIds.length });
      return response;
    } catch (error) {
      this.logger.error('Failed to delete vectors', { collectionName, count: vectorIds.length, error });
      throw error;
    }
  }

  // ===== SEARCH OPERATIONS =====

  /**
   * Search for similar vectors.
   * @param options - Optional read options for routing override
   */
  public async searchVectors(collectionName: string, request: SearchRequest, options?: ReadOptions): Promise<SearchResponse> {
    try {
      validateSearchRequest(request);
      const transport = this.getReadTransport(options);
      // API expects 'vector' field, not 'query_vector'
      const apiRequest = {
        vector: request.query_vector,
        limit: request.limit,
        threshold: request.threshold,
        include_metadata: request.include_metadata,
        filter: request.filter
      };
      const response = await transport.post<SearchResponse>(
        `/search`,
        { ...apiRequest, collection: collectionName }
      );
      // Ensure response has results array
      const normalizedResponse: SearchResponse = {
        ...response,
        results: response.results || []
      };
      this.logger.debug('Vector search completed', { collectionName, resultCount: normalizedResponse.results.length });
      return normalizedResponse;
    } catch (error) {
      this.logger.error('Failed to search vectors', { collectionName, request, error });
      throw error;
    }
  }

  /**
   * Search for similar vectors using text query.
   * @param options - Optional read options for routing override
   */
  public async searchText(collectionName: string, request: TextSearchRequest, options?: ReadOptions): Promise<SearchResponse> {
    try {
      validateTextSearchRequest(request);
      const transport = this.getReadTransport(options);
      const response = await transport.post<SearchResponse>(
        `/collections/${collectionName}/search/text`,
        request
      );
      validateSearchResponse(response);
      this.logger.debug('Text search completed', { collectionName, query: request.query, resultCount: response.results.length });
      return response;
    } catch (error) {
      this.logger.error('Failed to search text', { collectionName, request, error });
      throw error;
    }
  }

  // ===== INTELLIGENT SEARCH OPERATIONS =====

  /**
   * Advanced intelligent search with multi-query expansion and semantic reranking.
   * @param options - Optional read options for routing override
   */
  public async intelligentSearch(request: IntelligentSearchRequest, options?: ReadOptions): Promise<IntelligentSearchResponse> {
    try {
      const transport = this.getReadTransport(options);
      const response = await transport.post<IntelligentSearchResponse>('/intelligent_search', request);
      this.logger.debug('Intelligent search completed', {
        query: request.query,
        resultCount: response.results?.length || 0,
        collections: request.collections
      });
      return response;
    } catch (error) {
      this.logger.error('Failed to perform intelligent search', { request, error });
      throw error;
    }
  }

  /**
   * Semantic search with advanced reranking and similarity thresholds.
   * @param options - Optional read options for routing override
   */
  public async semanticSearch(request: SemanticSearchRequest, options?: ReadOptions): Promise<SemanticSearchResponse> {
    try {
      const transport = this.getReadTransport(options);
      const response = await transport.post<SemanticSearchResponse>('/semantic_search', request);
      this.logger.debug('Semantic search completed', {
        query: request.query,
        collection: request.collection,
        resultCount: response.results?.length || 0
      });
      return response;
    } catch (error) {
      this.logger.error('Failed to perform semantic search', { request, error });
      throw error;
    }
  }

  /**
   * Context-aware search with metadata filtering and contextual reranking.
   * @param options - Optional read options for routing override
   */
  public async contextualSearch(request: ContextualSearchRequest, options?: ReadOptions): Promise<ContextualSearchResponse> {
    try {
      const transport = this.getReadTransport(options);
      const response = await transport.post<ContextualSearchResponse>('/contextual_search', request);
      this.logger.debug('Contextual search completed', {
        query: request.query,
        collection: request.collection,
        resultCount: response.results?.length || 0,
        contextFilters: request.context_filters
      });
      return response;
    } catch (error) {
      this.logger.error('Failed to perform contextual search', { request, error });
      throw error;
    }
  }

  /**
   * Multi-collection search with cross-collection reranking and aggregation.
   * @param options - Optional read options for routing override
   */
  public async multiCollectionSearch(request: MultiCollectionSearchRequest, options?: ReadOptions): Promise<MultiCollectionSearchResponse> {
    try {
      const transport = this.getReadTransport(options);
      const response = await transport.post<MultiCollectionSearchResponse>('/multi_collection_search', request);
      this.logger.debug('Multi-collection search completed', {
        query: request.query,
        collections: request.collections,
        resultCount: response.results?.length || 0
      });
      return response;
    } catch (error) {
      this.logger.error('Failed to perform multi-collection search', { request, error });
      throw error;
    }
  }

  /**
   * Hybrid search combining dense and sparse vectors.
   * @param options - Optional read options for routing override
   */
  public async hybridSearch(request: HybridSearchRequest, options?: ReadOptions): Promise<HybridSearchResponse> {
    try {
      validateHybridSearchRequest(request);
      const transport = this.getReadTransport(options);
      const payload: any = {
        query: request.query,
        alpha: request.alpha ?? 0.7,
        algorithm: request.algorithm ?? 'rrf',
        dense_k: request.dense_k ?? 20,
        sparse_k: request.sparse_k ?? 20,
        final_k: request.final_k ?? 10,
      };
      if (request.query_sparse) {
        payload.query_sparse = {
          indices: request.query_sparse.indices,
          values: request.query_sparse.values,
        };
      }
      const response = await transport.post<HybridSearchResponse>(
        `/collections/${request.collection}/hybrid_search`,
        payload
      );
      this.logger.debug('Hybrid search completed', {
        query: request.query,
        collection: request.collection,
        algorithm: request.algorithm,
        resultCount: response.results?.length || 0
      });
      return response;
    } catch (error) {
      this.logger.error('Failed to perform hybrid search', { request, error });
      throw error;
    }
  }

  // ===== QDRANT COMPATIBILITY METHODS =====

  /**
   * List all collections (Qdrant-compatible API).
   * (Read operation - routed based on readPreference)
   */
  public async qdrantListCollections(options?: ReadOptions): Promise<any> {
    try {
      const transport = this.getReadTransport(options);
      return await transport.get('/qdrant/collections');
    } catch (error) {
      this.logger.error('Failed to list Qdrant collections', { error });
      throw error;
    }
  }

  /**
   * Get collection information (Qdrant-compatible API).
   * (Read operation - routed based on readPreference)
   */
  public async qdrantGetCollection(name: string, options?: ReadOptions): Promise<any> {
    try {
      const transport = this.getReadTransport(options);
      return await transport.get(`/qdrant/collections/${name}`);
    } catch (error) {
      this.logger.error('Failed to get Qdrant collection', { name, error });
      throw error;
    }
  }

  /**
   * Create collection (Qdrant-compatible API).
   * (Write operation - always routed to master)
   */
  public async qdrantCreateCollection(name: string, config: any): Promise<any> {
    try {
      const transport = this.getWriteTransport();
      return await transport.put(`/qdrant/collections/${name}`, { config });
    } catch (error) {
      this.logger.error('Failed to create Qdrant collection', { name, error });
      throw error;
    }
  }

  /**
   * Upsert points to collection (Qdrant-compatible API).
   * (Write operation - always routed to master)
   */
  public async qdrantUpsertPoints(collection: string, points: any[], wait: boolean = false): Promise<any> {
    try {
      const transport = this.getWriteTransport();
      return await transport.put(`/qdrant/collections/${collection}/points`, {
        points,
        wait,
      });
    } catch (error) {
      this.logger.error('Failed to upsert Qdrant points', { collection, error });
      throw error;
    }
  }

  /**
   * Search points in collection (Qdrant-compatible API).
   * (Read operation - routed based on readPreference)
   */
  public async qdrantSearchPoints(
    collection: string,
    vector: number[],
    limit: number = 10,
    filter?: any,
    withPayload: boolean = true,
    withVector: boolean = false,
    options?: ReadOptions
  ): Promise<any> {
    try {
      const transport = this.getReadTransport(options);
      const payload: any = {
        vector,
        limit,
        with_payload: withPayload,
        with_vector: withVector,
      };
      if (filter) {
        payload.filter = filter;
      }
      return await transport.post(`/qdrant/collections/${collection}/points/search`, payload);
    } catch (error) {
      this.logger.error('Failed to search Qdrant points', { collection, error });
      throw error;
    }
  }

  /**
   * Delete points from collection (Qdrant-compatible API).
   * (Write operation - always routed to master)
   */
  public async qdrantDeletePoints(collection: string, pointIds: (string | number)[], wait: boolean = false): Promise<any> {
    try {
      const transport = this.getWriteTransport();
      return await transport.post(`/qdrant/collections/${collection}/points/delete`, {
        points: pointIds,
        wait,
      });
    } catch (error) {
      this.logger.error('Failed to delete Qdrant points', { collection, error });
      throw error;
    }
  }

  /**
   * Retrieve points by IDs (Qdrant-compatible API).
   * (Read operation - routed based on readPreference)
   */
  public async qdrantRetrievePoints(
    collection: string,
    pointIds: (string | number)[],
    withPayload: boolean = true,
    withVector: boolean = false,
    options?: ReadOptions
  ): Promise<any> {
    try {
      const transport = this.getReadTransport(options);
      // Build query string manually for better compatibility
      const params = [
        `ids=${encodeURIComponent(pointIds.join(','))}`,
        `with_payload=${String(withPayload)}`,
        `with_vector=${String(withVector)}`,
      ].join('&');
      return await transport.get(`/qdrant/collections/${collection}/points?${params}`);
    } catch (error) {
      this.logger.error('Failed to retrieve Qdrant points', { collection, error });
      throw error;
    }
  }

  /**
   * Count points in collection (Qdrant-compatible API).
   * (Read operation - routed based on readPreference)
   */
  public async qdrantCountPoints(collection: string, filter?: any, options?: ReadOptions): Promise<any> {
    try {
      const transport = this.getReadTransport(options);
      const payload: any = {};
      if (filter) {
        payload.filter = filter;
      }
      return await transport.post(`/qdrant/collections/${collection}/points/count`, payload);
    } catch (error) {
      this.logger.error('Failed to count Qdrant points', { collection, error });
      throw error;
    }
  }

  // ===== QDRANT ADVANCED FEATURES (1.14.x) =====

  public async qdrantListCollectionSnapshots(collection: string): Promise<any> {
    try {
      return await this.transport.get(`/qdrant/collections/${collection}/snapshots`);
    } catch (error) {
      this.logger.error('Failed to list collection snapshots', { collection, error });
      throw error;
    }
  }

  public async qdrantCreateCollectionSnapshot(collection: string): Promise<any> {
    try {
      return await this.transport.post(`/qdrant/collections/${collection}/snapshots`, {});
    } catch (error) {
      this.logger.error('Failed to create collection snapshot', { collection, error });
      throw error;
    }
  }

  public async qdrantDeleteCollectionSnapshot(collection: string, snapshotName: string): Promise<any> {
    try {
      return await this.transport.delete(`/qdrant/collections/${collection}/snapshots/${snapshotName}`);
    } catch (error) {
      this.logger.error('Failed to delete collection snapshot', { collection, snapshotName, error });
      throw error;
    }
  }

  public async qdrantRecoverCollectionSnapshot(collection: string, location: string): Promise<any> {
    try {
      return await this.transport.post(`/qdrant/collections/${collection}/snapshots/recover`, { location });
    } catch (error) {
      this.logger.error('Failed to recover collection snapshot', { collection, location, error });
      throw error;
    }
  }

  public async qdrantListAllSnapshots(): Promise<any> {
    try {
      return await this.transport.get('/qdrant/snapshots');
    } catch (error) {
      this.logger.error('Failed to list all snapshots', { error });
      throw error;
    }
  }

  public async qdrantCreateFullSnapshot(): Promise<any> {
    try {
      return await this.transport.post('/qdrant/snapshots', {});
    } catch (error) {
      this.logger.error('Failed to create full snapshot', { error });
      throw error;
    }
  }

  public async qdrantListShardKeys(collection: string): Promise<any> {
    try {
      return await this.transport.get(`/qdrant/collections/${collection}/shards`);
    } catch (error) {
      this.logger.error('Failed to list shard keys', { collection, error });
      throw error;
    }
  }

  public async qdrantCreateShardKey(collection: string, shardKey: any): Promise<any> {
    try {
      return await this.transport.put(`/qdrant/collections/${collection}/shards`, { shard_key: shardKey });
    } catch (error) {
      this.logger.error('Failed to create shard key', { collection, error });
      throw error;
    }
  }

  public async qdrantDeleteShardKey(collection: string, shardKey: any): Promise<any> {
    try {
      return await this.transport.post(`/qdrant/collections/${collection}/shards/delete`, { shard_key: shardKey });
    } catch (error) {
      this.logger.error('Failed to delete shard key', { collection, error });
      throw error;
    }
  }

  public async qdrantGetClusterStatus(): Promise<any> {
    try {
      return await this.transport.get('/qdrant/cluster');
    } catch (error) {
      this.logger.error('Failed to get cluster status', { error });
      throw error;
    }
  }

  public async qdrantClusterRecover(): Promise<any> {
    try {
      return await this.transport.post('/qdrant/cluster/recover', {});
    } catch (error) {
      this.logger.error('Failed to recover cluster', { error });
      throw error;
    }
  }

  public async qdrantRemovePeer(peerId: string): Promise<any> {
    try {
      return await this.transport.delete(`/qdrant/cluster/peer/${peerId}`);
    } catch (error) {
      this.logger.error('Failed to remove peer', { peerId, error });
      throw error;
    }
  }

  public async qdrantListMetadataKeys(): Promise<any> {
    try {
      return await this.transport.get('/qdrant/cluster/metadata/keys');
    } catch (error) {
      this.logger.error('Failed to list metadata keys', { error });
      throw error;
    }
  }

  public async qdrantGetMetadataKey(key: string): Promise<any> {
    try {
      return await this.transport.get(`/qdrant/cluster/metadata/keys/${key}`);
    } catch (error) {
      this.logger.error('Failed to get metadata key', { key, error });
      throw error;
    }
  }

  public async qdrantUpdateMetadataKey(key: string, value: any): Promise<any> {
    try {
      return await this.transport.put(`/qdrant/cluster/metadata/keys/${key}`, { value });
    } catch (error) {
      this.logger.error('Failed to update metadata key', { key, error });
      throw error;
    }
  }

  public async qdrantQueryPoints(collection: string, request: any): Promise<any> {
    try {
      return await this.transport.post(`/qdrant/collections/${collection}/points/query`, request);
    } catch (error) {
      this.logger.error('Failed to query points', { collection, error });
      throw error;
    }
  }

  public async qdrantBatchQueryPoints(collection: string, request: any): Promise<any> {
    try {
      return await this.transport.post(`/qdrant/collections/${collection}/points/query/batch`, request);
    } catch (error) {
      this.logger.error('Failed to batch query points', { collection, error });
      throw error;
    }
  }

  public async qdrantQueryPointsGroups(collection: string, request: any): Promise<any> {
    try {
      return await this.transport.post(`/qdrant/collections/${collection}/points/query/groups`, request);
    } catch (error) {
      this.logger.error('Failed to query points groups', { collection, error });
      throw error;
    }
  }

  public async qdrantSearchPointsGroups(collection: string, request: any): Promise<any> {
    try {
      return await this.transport.post(`/qdrant/collections/${collection}/points/search/groups`, request);
    } catch (error) {
      this.logger.error('Failed to search points groups', { collection, error });
      throw error;
    }
  }

  public async qdrantSearchMatrixPairs(collection: string, request: any): Promise<any> {
    try {
      return await this.transport.post(`/qdrant/collections/${collection}/points/search/matrix/pairs`, request);
    } catch (error) {
      this.logger.error('Failed to search matrix pairs', { collection, error });
      throw error;
    }
  }

  public async qdrantSearchMatrixOffsets(collection: string, request: any): Promise<any> {
    try {
      return await this.transport.post(`/qdrant/collections/${collection}/points/search/matrix/offsets`, request);
    } catch (error) {
      this.logger.error('Failed to search matrix offsets', { collection, error });
      throw error;
    }
  }

  // ===== EMBEDDING OPERATIONS =====

  /**
   * Generate embeddings for text.
   */
  public async embedText(request: EmbeddingRequest): Promise<EmbeddingResponse> {
    try {
      validateEmbeddingRequest(request);
      const response = await this.transport.post<EmbeddingResponse>('/embed', request);
      validateEmbeddingResponse(response);
      this.logger.debug('Text embedding generated', { text: request.text, model: response.model });
      return response;
    } catch (error) {
      this.logger.error('Failed to generate embedding', { request, error });
      throw error;
    }
  }

  // ===== UTILITY METHODS =====

  /**
   * Get the current configuration.
   */
  public getConfig(): Readonly<VectorizerClientConfig> {
    return { ...this.config };
  }

  /**
   * Update the API key.
   */
  public setApiKey(apiKey: string): void {
    this.config.apiKey = apiKey;

    // Reinitialize transport with new API key
    if (this.protocol === 'http' && this.config.baseURL) {
      const httpConfig: HttpClientConfig = {
        baseURL: this.config.baseURL,
        ...(this.config.timeout && { timeout: this.config.timeout }),
        ...(this.config.headers && { headers: this.config.headers }),
        apiKey: this.config.apiKey,
      };
      this.transport = TransportFactory.create({ protocol: 'http', http: httpConfig });
    } else if (this.protocol === 'umicp' && this.config.umicp) {
      const umicpConfig: UMICPClientConfig = {
        host: this.config.umicp.host || 'localhost',
        port: this.config.umicp.port || 15003,
        apiKey: this.config.apiKey,
        ...(this.config.timeout && { timeout: this.config.timeout }),
        ...this.config.umicp,
      };
      this.transport = TransportFactory.create({ protocol: 'umicp', umicp: umicpConfig });
    }

    this.logger.info('API key updated');
  }

  // ==================== BATCH OPERATIONS ====================

  /**
   * Batch insert texts into a collection (embeddings generated automatically)
   */
  public async batchInsertTexts(
    collection: string,
    request: BatchInsertRequest
  ): Promise<BatchResponse> {
    this.logger.debug('Batch inserting texts', { collection, count: request.texts.length });

    try {
      const response = await this.transport.post<BatchResponse>(
        `/batch_insert`,
        request
      );

      this.logger.info('Batch insert completed', {
        collection,
        successful: response.successful_operations,
        failed: response.failed_operations,
        duration: response.duration_ms,
      });

      return response;
    } catch (error) {
      this.logger.error('Batch insert failed', { collection, error });
      throw error;
    }
  }

  /**
   * Batch search vectors in a collection
   */
  public async batchSearchVectors(
    collection: string,
    request: BatchSearchRequest
  ): Promise<BatchSearchResponse> {
    this.logger.debug('Batch searching vectors', { collection, queries: request.queries.length });

    try {
      const response = await this.transport.post<BatchSearchResponse>(
        `/batch_search`,
        request
      );

      this.logger.info('Batch search completed', {
        collection,
        successful: response.successful_queries,
        failed: response.failed_queries,
        duration: response.duration_ms,
      });

      return response;
    } catch (error) {
      this.logger.error('Batch search failed', { collection, error });
      throw error;
    }
  }

  /**
   * Batch update vectors in a collection
   */
  public async batchUpdateVectors(
    collection: string,
    request: BatchUpdateRequest
  ): Promise<BatchResponse> {
    this.logger.debug('Batch updating vectors', { collection, count: request.updates.length });

    try {
      const response = await this.transport.post<BatchResponse>(
        `/batch_update`,
        request
      );

      this.logger.info('Batch update completed', {
        collection,
        successful: response.successful_operations,
        failed: response.failed_operations,
        duration: response.duration_ms,
      });

      return response;
    } catch (error) {
      this.logger.error('Batch update failed', { collection, error });
      throw error;
    }
  }

  /**
   * Batch delete vectors from a collection
   */
  public async batchDeleteVectors(
    collection: string,
    request: BatchDeleteRequest
  ): Promise<BatchResponse> {
    this.logger.debug('Batch deleting vectors', { collection, count: request.vector_ids.length });

    try {
      const response = await this.transport.post<BatchResponse>(
        `/batch_delete`,
        request
      );

      this.logger.info('Batch delete completed', {
        collection,
        successful: response.successful_operations,
        failed: response.failed_operations,
        duration: response.duration_ms,
      });

      return response;
    } catch (error) {
      this.logger.error('Batch delete failed', { collection, error });
      throw error;
    }
  }

  // =============================================================================
  // SUMMARIZATION METHODS
  // =============================================================================

  // NOTE: Summarization endpoints are not available in the current server version
  // The following methods are commented out until summarization is re-implemented

  /*
  /**
   * Summarize text using various methods
   */
  /*
  public async summarizeText(request: SummarizeTextRequest): Promise<SummarizeTextResponse> {
    this.logger.debug('Summarizing text', { method: request.method, textLength: request.text.length });

    try {
      const response = await this.transport.post<SummarizeTextResponse>(
        '/summarize/text',
        request
      );

      this.logger.info('Text summarized successfully', {
        summaryId: response.summary_id,
        originalLength: response.original_length,
        summaryLength: response.summary_length,
        compressionRatio: response.compression_ratio,
        method: response.method,
      });

      return response;
    } catch (error) {
      this.logger.error('Text summarization failed', { error });
      throw error;
    }
  }

  /**
   * Summarize context using various methods
   */
  /*
  public async summarizeContext(request: SummarizeContextRequest): Promise<SummarizeContextResponse> {
    this.logger.debug('Summarizing context', { method: request.method, contextLength: request.context.length });

    try {
      const response = await this.transport.post<SummarizeContextResponse>(
        '/summarize/context',
        request
      );

      this.logger.info('Context summarized successfully', {
        summaryId: response.summary_id,
        originalLength: response.original_length,
        summaryLength: response.summary_length,
        compressionRatio: response.compression_ratio,
        method: response.method,
      });

      return response;
    } catch (error) {
      this.logger.error('Context summarization failed', { error });
      throw error;
    }
  }

  /**
   * Get a specific summary by ID
   */
  /*
  public async getSummary(summaryId: string): Promise<GetSummaryResponse> {
    this.logger.debug('Getting summary', { summaryId });

    try {
      const response = await this.transport.get<GetSummaryResponse>(
        `/summaries/${summaryId}`
      );

      this.logger.info('Summary retrieved successfully', {
        summaryId: response.summary_id,
        method: response.method,
        summaryLength: response.summary_length,
      });

      return response;
    } catch (error) {
      this.logger.error('Get summary failed', { summaryId, error });
      throw error;
    }
  }

  /**
   * List summaries with optional filtering
   */
  /*
  public async listSummaries(query?: ListSummariesQuery): Promise<ListSummariesResponse> {
    this.logger.debug('Listing summaries', { query });

    try {
      const response = await this.transport.get<ListSummariesResponse>(
        '/summaries',
        query ? { params: query } : {}
      );

      this.logger.info('Summaries listed successfully', {
        count: response.summaries.length,
        totalCount: response.total_count,
      });

      return response;
    } catch (error) {
      this.logger.error('List summaries failed', { error });
      throw error;
    }
  }
  */

  /**
   * Close the client and clean up resources.
   */
  public async close(): Promise<void> {
    this.logger.info('VectorizerClient closed');
  }

  // =============================================================================
  // DISCOVERY OPERATIONS
  // =============================================================================

  /**
   * Complete discovery pipeline with intelligent search and prompt generation.
   */
  public async discover(params: {
    query: string;
    include_collections?: string[];
    exclude_collections?: string[];
    max_bullets?: number;
    broad_k?: number;
    focus_k?: number;
  }): Promise<any> {
    this.logger.debug('Running discovery pipeline', params);
    return this.transport.post('/discover', params);
  }

  /**
   * Pre-filter collections by name patterns.
   */
  public async filterCollections(params: {
    query: string;
    include?: string[];
    exclude?: string[];
  }): Promise<any> {
    this.logger.debug('Filtering collections', params);
    return this.transport.post('/discovery/filter_collections', params);
  }

  /**
   * Rank collections by relevance.
   */
  public async scoreCollections(params: {
    query: string;
    name_match_weight?: number;
    term_boost_weight?: number;
    signal_boost_weight?: number;
  }): Promise<any> {
    this.logger.debug('Scoring collections', params);
    return this.transport.post('/discovery/score_collections', params);
  }

  /**
   * Generate query variations.
   */
  public async expandQueries(params: {
    query: string;
    max_expansions?: number;
    include_definition?: boolean;
    include_features?: boolean;
    include_architecture?: boolean;
  }): Promise<any> {
    this.logger.debug('Expanding queries', params);
    return this.transport.post('/discovery/expand_queries', params);
  }

  // =============================================================================
  // FILE OPERATIONS
  // =============================================================================

  /**
   * Retrieve complete file content from a collection.
   */
  public async getFileContent(params: {
    collection: string;
    file_path: string;
    max_size_kb?: number;
  }): Promise<any> {
    this.logger.debug('Getting file content', params);
    return this.transport.post('/file/content', params);
  }

  /**
   * List all indexed files in a collection.
   */
  public async listFilesInCollection(params: {
    collection: string;
    filter_by_type?: string[];
    min_chunks?: number;
    max_results?: number;
    sort_by?: 'name' | 'size' | 'chunks' | 'recent';
  }): Promise<any> {
    this.logger.debug('Listing files in collection', params);
    return this.transport.post('/file/list', params);
  }

  /**
   * Get extractive or structural summary of an indexed file.
   */
  public async getFileSummary(params: {
    collection: string;
    file_path: string;
    summary_type?: 'extractive' | 'structural' | 'both';
    max_sentences?: number;
  }): Promise<any> {
    this.logger.debug('Getting file summary', params);
    return this.transport.post('/file/summary', params);
  }

  /**
   * Retrieve chunks in original file order for progressive reading.
   */
  public async getFileChunksOrdered(params: {
    collection: string;
    file_path: string;
    start_chunk?: number;
    limit?: number;
    include_context?: boolean;
  }): Promise<any> {
    this.logger.debug('Getting file chunks', params);
    return this.transport.post('/file/chunks', params);
  }

  /**
   * Generate hierarchical project structure overview.
   */
  public async getProjectOutline(params: {
    collection: string;
    max_depth?: number;
    include_summaries?: boolean;
    highlight_key_files?: boolean;
  }): Promise<any> {
    this.logger.debug('Getting project outline', params);
    return this.transport.post('/file/outline', params);
  }

  /**
   * Find semantically related files using vector similarity.
   */
  public async getRelatedFiles(params: {
    collection: string;
    file_path: string;
    limit?: number;
    similarity_threshold?: number;
    include_reason?: boolean;
  }): Promise<any> {
    this.logger.debug('Getting related files', params);
    return this.transport.post('/file/related', params);
  }

  /**
   * Semantic search filtered by file type.
   */
  public async searchByFileType(params: {
    collection: string;
    query: string;
    file_types: string[];
    limit?: number;
    return_full_files?: boolean;
  }): Promise<any> {
    this.logger.debug('Searching by file type', params);
    return this.transport.post('/file/search_by_type', params);
  }

  // ============================================================================
  // GUI-Specific API Methods
  // ============================================================================

  /**
   * Get server status (GUI endpoint).
   */
  async getStatus(): Promise<{
    status: string;
    version: string;
    uptime: number;
    collections: number;
    total_vectors: number;
  }> {
    this.logger.debug('Getting server status');
    return this.transport.get('/status');
  }

  /**
   * Get recent logs (GUI endpoint).
   */
  async getLogs(params?: {
    lines?: number;
    level?: string;
  }): Promise<{ logs: string[] }> {
    this.logger.debug('Getting logs', params);
    return this.transport.get('/logs', params ? { params } : undefined);
  }

  /**
   * Force save a specific collection (GUI endpoint).
   */
  async forceSaveCollection(name: string): Promise<{ success: boolean; message: string }> {
    this.logger.debug('Force saving collection', { name });
    return this.transport.post(`/collections/${name}/force-save`, {});
  }

  /**
   * Add a workspace (GUI endpoint).
   */
  async addWorkspace(params: {
    name: string;
    path: string;
    collections: Array<{
      name: string;
      path: string;
      exclude_patterns?: string[];
    }>;
  }): Promise<{ success: boolean; message: string }> {
    this.logger.debug('Adding workspace', params);
    return this.transport.post('/workspace/add', params);
  }

  /**
   * Remove a workspace (GUI endpoint).
   */
  async removeWorkspace(params: {
    name: string;
  }): Promise<{ success: boolean; message: string }> {
    this.logger.debug('Removing workspace', params);
    return this.transport.post('/workspace/remove', params);
  }

  /**
   * List all workspaces (GUI endpoint).
   */
  async listWorkspaces(): Promise<{
    workspaces: Array<{
      name: string;
      path: string;
      collections: number;
    }>;
  }> {
    this.logger.debug('Listing workspaces');
    return this.transport.get('/workspace/list');
  }

  /**
   * Get server configuration (GUI endpoint).
   */
  async getServerConfig(): Promise<Record<string, any>> {
    this.logger.debug('Getting server configuration');
    return this.transport.get('/config');
  }

  /**
   * Update server configuration (GUI endpoint).
   */
  async updateConfig(config: Record<string, any>): Promise<{
    success: boolean;
    message: string;
  }> {
    this.logger.debug('Updating server configuration', config);
    return this.transport.post('/config', config);
  }

  /**
   * Restart the server (GUI endpoint - admin only).
   */
  async restartServer(): Promise<{ success: boolean; message: string }> {
    this.logger.debug('Requesting server restart');
    return this.transport.post('/admin/restart', {});
  }

  /**
   * List available backups (GUI endpoint).
   */
  async listBackups(): Promise<{
    backups: Array<{
      filename: string;
      size: number;
      created_at: string;
    }>;
  }> {
    this.logger.debug('Listing backups');
    return this.transport.get('/backups/list');
  }

  /**
   * Create a new backup (GUI endpoint).
   */
  async createBackup(params?: {
    name?: string;
  }): Promise<{
    success: boolean;
    message: string;
    filename?: string;
  }> {
    this.logger.debug('Creating backup', params);
    return this.transport.post('/backups/create', params || {});
  }

  /**
   * Restore from a backup (GUI endpoint).
   */
  async restoreBackup(params: {
    filename: string;
  }): Promise<{
    success: boolean;
    message: string;
  }> {
    this.logger.debug('Restoring backup', params);
    return this.transport.post('/backups/restore', params);
  }

  /**
   * Get backup directory path (GUI endpoint).
   */
  async getBackupDirectory(): Promise<{
    directory: string;
  }> {
    this.logger.debug('Getting backup directory');
    return this.transport.get('/backups/directory');
  }

  // ========== Graph Operations ==========

  /**
   * List all nodes in a collection's graph.
   */
  async listGraphNodes(collection: string): Promise<ListNodesResponse> {
    this.logger.debug('Listing graph nodes', { collection });
    return this.transport.get(`/graph/nodes/${collection}`);
  }

  /**
   * Get neighbors of a specific node.
   */
  async getGraphNeighbors(collection: string, nodeId: string): Promise<GetNeighborsResponse> {
    this.logger.debug('Getting graph neighbors', { collection, nodeId });
    return this.transport.get(`/graph/nodes/${collection}/${nodeId}/neighbors`);
  }

  /**
   * Find related nodes within N hops.
   */
  async findRelatedNodes(
    collection: string,
    nodeId: string,
    request: FindRelatedRequest
  ): Promise<FindRelatedResponse> {
    this.logger.debug('Finding related nodes', { collection, nodeId, request });
    return this.transport.post(`/graph/nodes/${collection}/${nodeId}/related`, request);
  }

  /**
   * Find shortest path between two nodes.
   */
  async findGraphPath(request: FindPathRequest): Promise<FindPathResponse> {
    this.logger.debug('Finding graph path', { request });
    return this.transport.post('/graph/path', request);
  }

  /**
   * Create an explicit edge between two nodes.
   */
  async createGraphEdge(request: CreateEdgeRequest): Promise<CreateEdgeResponse> {
    this.logger.debug('Creating graph edge', { request });
    return this.transport.post('/graph/edges', request);
  }

  /**
   * Delete an edge by ID.
   */
  async deleteGraphEdge(edgeId: string): Promise<void> {
    this.logger.debug('Deleting graph edge', { edgeId });
    return this.transport.delete(`/graph/edges/${edgeId}`);
  }

  /**
   * List all edges in a collection.
   */
  async listGraphEdges(collection: string): Promise<ListEdgesResponse> {
    this.logger.debug('Listing graph edges', { collection });
    return this.transport.get(`/graph/collections/${collection}/edges`);
  }

  /**
   * Discover SIMILAR_TO edges for entire collection.
   */
  async discoverGraphEdges(
    collection: string,
    request: DiscoverEdgesRequest
  ): Promise<DiscoverEdgesResponse> {
    this.logger.debug('Discovering graph edges', { collection, request });
    return this.transport.post(`/graph/discover/${collection}`, request);
  }

  /**
   * Discover SIMILAR_TO edges for a specific node.
   */
  async discoverGraphEdgesForNode(
    collection: string,
    nodeId: string,
    request: DiscoverEdgesRequest
  ): Promise<DiscoverEdgesResponse> {
    this.logger.debug('Discovering graph edges for node', { collection, nodeId, request });
    return this.transport.post(`/graph/discover/${collection}/${nodeId}`, request);
  }

  /**
   * Get discovery status for a collection.
   */
  async getGraphDiscoveryStatus(collection: string): Promise<DiscoveryStatusResponse> {
    this.logger.debug('Getting graph discovery status', { collection });
    return this.transport.get(`/graph/discover/${collection}/status`);
  }

  // ============================================================
  // File Upload Operations
  // ============================================================

  /**
   * Upload a file for indexing.
   *
   * The file will be validated, chunked, and indexed into the specified collection.
   * If the collection doesn't exist, it will be created automatically.
   *
   * @param file - File to upload (File object in browser, Buffer in Node.js)
   * @param filename - Filename (used for extension detection)
   * @param collectionName - Target collection name
   * @param options - Upload options
   * @returns Upload response with results
   *
   * @example
   * ```typescript
   * // Browser
   * const file = fileInput.files[0];
   * const response = await client.uploadFile(file, file.name, 'my-docs');
   *
   * // Node.js
   * const buffer = fs.readFileSync('document.md');
   * const response = await client.uploadFile(buffer, 'document.md', 'my-docs');
   * ```
   */
  async uploadFile(
    file: File | Buffer | Uint8Array,
    filename: string,
    collectionName: string,
    options: UploadFileOptions = {}
  ): Promise<FileUploadResponse> {
    this.logger.debug('Uploading file', { filename, collectionName, options });

    // Create FormData
    const formData = new FormData();

    // Handle different input types
    if (typeof File !== 'undefined' && file instanceof File) {
      formData.append('file', file, filename);
    } else if (typeof Blob !== 'undefined') {
      const blob = new Blob([file as Buffer | Uint8Array]);
      formData.append('file', blob, filename);
    } else {
      throw new Error('File upload requires File, Buffer, or Uint8Array');
    }

    formData.append('collection_name', collectionName);

    if (options.chunkSize !== undefined) {
      formData.append('chunk_size', options.chunkSize.toString());
    }

    if (options.chunkOverlap !== undefined) {
      formData.append('chunk_overlap', options.chunkOverlap.toString());
    }

    if (options.metadata !== undefined) {
      formData.append('metadata', JSON.stringify(options.metadata));
    }

    if (options.publicKey !== undefined) {
      formData.append('public_key', options.publicKey);
    }

    const response = await this.transport.postFormData<FileUploadResponse>('/files/upload', formData);

    this.logger.info('File uploaded successfully', {
      filename,
      chunksCreated: response.chunks_created,
      vectorsCreated: response.vectors_created,
    });

    return response;
  }

  /**
   * Upload file content directly for indexing.
   *
   * Similar to uploadFile, but accepts content directly as a string.
   *
   * @param content - File content as string
   * @param filename - Filename (used for extension detection)
   * @param collectionName - Target collection name
   * @param options - Upload options
   * @returns Upload response with results
   *
   * @example
   * ```typescript
   * const code = `fn main() { println!("Hello!"); }`;
   * const response = await client.uploadFileContent(code, 'main.rs', 'rust-code');
   * ```
   */
  async uploadFileContent(
    content: string,
    filename: string,
    collectionName: string,
    options: UploadFileOptions = {}
  ): Promise<FileUploadResponse> {
    this.logger.debug('Uploading file content', { filename, collectionName, options });

    // Convert string to Uint8Array
    const encoder = new TextEncoder();
    const buffer = encoder.encode(content);

    return this.uploadFile(buffer, filename, collectionName, options);
  }

  /**
   * Get file upload configuration from the server.
   *
   * @returns Upload configuration with server limits and settings
   *
   * @example
   * ```typescript
   * const config = await client.getUploadConfig();
   * console.log(`Max file size: ${config.max_file_size_mb}MB`);
   * console.log(`Allowed extensions: ${config.allowed_extensions.join(', ')}`);
   * ```
   */
  async getUploadConfig(): Promise<FileUploadConfig> {
    this.logger.debug('Getting file upload configuration');
    return this.transport.get('/files/config');
  }
}
