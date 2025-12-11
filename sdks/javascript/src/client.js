/**
 * Main client class for the Hive Vectorizer SDK.
 * 
 * Provides high-level methods for vector operations, semantic search,
 * and collection management.
 */

import { TransportFactory, parseConnectionString } from './utils/transport.js';
import { createLogger } from './utils/logger.js';
import { validateNonEmptyString } from './utils/validation.js';

import {
  validateVector,
  validateCreateVectorRequest,
} from './models/vector.js';

import {
  validateCollection,
  validateCreateCollectionRequest,
} from './models/collection.js';

import {
  validateCollectionInfo,
  validateDatabaseStats,
} from './models/collection-info.js';

import {
  validateSearchResponse,
} from './models/search-result.js';

import {
  validateEmbeddingRequest,
  validateEmbeddingResponse,
} from './models/embedding-request.js';

import {
  validateSearchRequest,
  validateTextSearchRequest,
} from './models/search-request.js';

import {
  validateHybridSearchRequest,
} from './models/hybrid-search.js';

import {
  BatchResponse,
  BatchSearchResponse,
} from './models/batch.js';

import {
  validateFindRelatedRequest,
  validateFindPathRequest,
  validateCreateEdgeRequest,
  validateDiscoverEdgesRequest,
} from './models/graph.js';

import {
  validateFileUploadResponse,
  validateFileUploadConfig,
} from './models/file-upload.js';

// Summarization models are used internally but not directly exported

// Removed unused exception imports - exceptions are handled in http-client

/**
 * @typedef {'master' | 'replica' | 'nearest'} ReadPreference
 * Read preference for routing read operations.
 */

/**
 * @typedef {Object} HostConfig
 * @property {string} master - Master node URL
 * @property {string[]} replicas - Replica node URLs
 */

/**
 * @typedef {Object} ReadOptions
 * @property {ReadPreference} [readPreference] - Override read preference for this operation
 */

export class VectorizerClient {
  constructor(config = {}) {
    this.config = {
      baseURL: 'http://localhost:15002',
      timeout: 30000,
      headers: {},
      logger: { level: 'info', enabled: true },
      readPreference: 'replica',
      ...config,
    };

    this.logger = createLogger(this.config.logger);

    /** @type {ReadPreference} */
    this.readPreference = this.config.readPreference || 'replica';
    /** @type {boolean} */
    this.isReplicaMode = false;
    /** @type {import('./utils/transport.js').ITransport|undefined} */
    this.masterTransport = undefined;
    /** @type {import('./utils/transport.js').ITransport[]} */
    this.replicaTransports = [];
    /** @type {number} */
    this.replicaIndex = 0;

    // Check if using master/replica configuration
    if (this.config.hosts) {
      this._initializeReplicaMode();
      return;
    }

    // Determine protocol and create transport
    if (this.config.connectionString) {
      // Use connection string
      const transportConfig = parseConnectionString(this.config.connectionString, this.config.apiKey);
      this.transport = TransportFactory.create(transportConfig);
      this.protocol = transportConfig.protocol;
      
      this.logger.info('VectorizerClient initialized from connection string', {
        protocol: this.protocol,
        connectionString: this.config.connectionString,
        hasApiKey: !!this.config.apiKey,
      });
    } else {
      // Use explicit configuration
      this.protocol = this.config.protocol || 'http';
      
      if (this.protocol === 'http') {
        const httpConfig = {
          baseURL: this.config.baseURL,
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
        
        const umicpConfig = {
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
   * @private
   */
  _initializeReplicaMode() {
    const { hosts } = this.config;
    if (!hosts) return;

    this.isReplicaMode = true;
    this.protocol = 'http';

    // Create master transport
    const masterHttpConfig = {
      baseURL: hosts.master,
      ...(this.config.timeout && { timeout: this.config.timeout }),
      ...(this.config.headers && { headers: this.config.headers }),
      ...(this.config.apiKey && { apiKey: this.config.apiKey }),
    };
    this.masterTransport = TransportFactory.create({ protocol: 'http', http: masterHttpConfig });

    // Create replica transports
    this.replicaTransports = hosts.replicas.map(replicaUrl => {
      const replicaHttpConfig = {
        baseURL: replicaUrl,
        ...(this.config.timeout && { timeout: this.config.timeout }),
        ...(this.config.headers && { headers: this.config.headers }),
        ...(this.config.apiKey && { apiKey: this.config.apiKey }),
      };
      return TransportFactory.create({ protocol: 'http', http: replicaHttpConfig });
    });

    // Default transport is master (for backward compatibility)
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
   * @private
   * @returns {import('./utils/transport.js').ITransport}
   */
  _getWriteTransport() {
    if (this.isReplicaMode && this.masterTransport) {
      return this.masterTransport;
    }
    return this.transport;
  }

  /**
   * Get transport for read operations based on read preference.
   * @private
   * @param {ReadOptions} [options] - Optional read options
   * @returns {import('./utils/transport.js').ITransport}
   */
  _getReadTransport(options) {
    if (!this.isReplicaMode) {
      return this.transport;
    }

    const preference = options?.readPreference || this.readPreference;

    switch (preference) {
      case 'master':
        return this.masterTransport;

      case 'replica':
        if (this.replicaTransports.length === 0) {
          return this.masterTransport;
        }
        // Round-robin selection
        const transport = this.replicaTransports[this.replicaIndex];
        this.replicaIndex = (this.replicaIndex + 1) % this.replicaTransports.length;
        return transport;

      case 'nearest':
        // For now, use round-robin as a simple implementation
        if (this.replicaTransports.length === 0) {
          return this.masterTransport;
        }
        const nearestTransport = this.replicaTransports[this.replicaIndex];
        this.replicaIndex = (this.replicaIndex + 1) % this.replicaTransports.length;
        return nearestTransport;

      default:
        return this.masterTransport;
    }
  }

  /**
   * Execute a callback with master transport for read-your-writes scenarios.
   * @param {function(VectorizerClient): Promise<T>} callback - Callback to execute
   * @returns {Promise<T>}
   * @template T
   */
  async withMaster(callback) {
    const masterClient = new VectorizerClient({
      ...this.config,
      readPreference: 'master',
    });
    return callback(masterClient);
  }

  /**
   * Get the current transport protocol being used.
   */
  getProtocol() {
    return this.protocol;
  }

  // ===== HEALTH & STATUS =====

  /**
   * Check if the server is healthy.
   */
  async healthCheck() {
    try {
      const response = await this.transport.get('/health');
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
  async getDatabaseStats() {
    try {
      const response = await this.transport.get('/stats');
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
   * @param {ReadOptions} [options] - Optional read options
   */
  async listCollections(options) {
    try {
      const transport = this._getReadTransport(options);
      const response = await transport.get('/collections');
      this.logger.debug('Collections listed', { count: response.length });
      return response;
    } catch (error) {
      this.logger.error('Failed to list collections', error);
      throw error;
    }
  }

  /**
   * Get collection information.
   * @param {string} collectionName - Collection name
   * @param {ReadOptions} [options] - Optional read options
   */
  async getCollection(collectionName, options) {
    try {
      const transport = this._getReadTransport(options);
      const response = await transport.get(`/collections/${collectionName}`);
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
   */
  async createCollection(request) {
    try {
      validateCreateCollectionRequest(request);
      const transport = this._getWriteTransport();
      const response = await transport.post('/collections', request);
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
   */
  async updateCollection(collectionName, request) {
    try {
      const transport = this._getWriteTransport();
      const response = await transport.put(`/collections/${collectionName}`, request);
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
   */
  async deleteCollection(collectionName) {
    try {
      const transport = this._getWriteTransport();
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
   */
  async insertVectors(collectionName, vectors) {
    try {
      vectors.forEach(validateCreateVectorRequest);
      const transport = this._getWriteTransport();
      const response = await transport.post(
        `/collections/${collectionName}/vectors`,
        { vectors }
      );
      this.logger.info('Vectors inserted', { collectionName, count: vectors.length });
      return response;
    } catch (error) {
      this.logger.error('Failed to insert vectors', { collectionName, count: vectors.length, error });
      throw error;
    }
  }

  /**
   * Get a vector by ID.
   * @param {string} collectionName - Collection name
   * @param {string} vectorId - Vector ID
   * @param {ReadOptions} [options] - Optional read options
   */
  async getVector(collectionName, vectorId, options) {
    try {
      const transport = this._getReadTransport(options);
      const response = await transport.get(`/collections/${collectionName}/vectors/${vectorId}`);
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
   */
  async updateVector(collectionName, vectorId, request) {
    try {
      const transport = this._getWriteTransport();
      const response = await transport.put(
        `/collections/${collectionName}/vectors/${vectorId}`,
        request
      );
      validateVector(response);
      this.logger.info('Vector updated', { collectionName, vectorId });
      return response;
    } catch (error) {
      this.logger.error('Failed to update vector', { collectionName, vectorId, request, error });
      throw error;
    }
  }

  /**
   * Delete a vector.
   */
  async deleteVector(collectionName, vectorId) {
    try {
      const transport = this._getWriteTransport();
      await transport.delete(`/collections/${collectionName}/vectors/${vectorId}`);
      this.logger.info('Vector deleted', { collectionName, vectorId });
    } catch (error) {
      this.logger.error('Failed to delete vector', { collectionName, vectorId, error });
      throw error;
    }
  }

  /**
   * Delete multiple vectors.
   */
  async deleteVectors(collectionName, vectorIds) {
    try {
      const transport = this._getWriteTransport();
      const response = await transport.post(
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
   * @param {string} collectionName - Collection name
   * @param {Object} request - Search request
   * @param {ReadOptions} [options] - Optional read options
   */
  async searchVectors(collectionName, request, options) {
    try {
      validateSearchRequest(request);
      const transport = this._getReadTransport(options);
      const response = await transport.post(
        `/collections/${collectionName}/search`,
        request
      );
      validateSearchResponse(response);
      this.logger.debug('Vector search completed', { collectionName, resultCount: response.results.length });
      return response;
    } catch (error) {
      this.logger.error('Failed to search vectors', { collectionName, request, error });
      throw error;
    }
  }

  /**
   * Search for similar vectors using text query.
   * @param {string} collectionName - Collection name
   * @param {Object} request - Text search request
   * @param {ReadOptions} [options] - Optional read options
   */
  async searchText(collectionName, request, options) {
    try {
      validateTextSearchRequest(request);
      const transport = this._getReadTransport(options);
      const response = await transport.post(
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
   * @param {Object} request - Intelligent search request
   * @param {string} request.query - Search query
   * @param {string[]} [request.collections] - Collections to search (optional)
   * @param {number} [request.max_results=10] - Maximum number of results
   * @param {boolean} [request.domain_expansion=true] - Enable domain expansion
   * @param {boolean} [request.technical_focus=true] - Enable technical focus
   * @param {boolean} [request.mmr_enabled=true] - Enable MMR diversification
   * @param {number} [request.mmr_lambda=0.7] - MMR balance parameter (0.0-1.0)
   * @param {ReadOptions} [options] - Optional read options
   * @returns {Promise<Object>} Intelligent search response
   */
  async intelligentSearch(request, options) {
    try {
      const transport = this._getReadTransport(options);
      const response = await transport.post('/intelligent_search', request);
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
   * @param {Object} request - Semantic search request
   * @param {string} request.query - Search query
   * @param {string} request.collection - Collection to search
   * @param {number} [request.max_results=10] - Maximum number of results
   * @param {boolean} [request.semantic_reranking=true] - Enable semantic reranking
   * @param {boolean} [request.cross_encoder_reranking=false] - Enable cross-encoder reranking
   * @param {number} [request.similarity_threshold=0.5] - Minimum similarity threshold
   * @param {ReadOptions} [options] - Optional read options
   * @returns {Promise<Object>} Semantic search response
   */
  async semanticSearch(request, options) {
    try {
      const transport = this._getReadTransport(options);
      const response = await transport.post('/semantic_search', request);
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
   * @param {Object} request - Contextual search request
   * @param {string} request.query - Search query
   * @param {string} request.collection - Collection to search
   * @param {Object} [request.context_filters] - Metadata-based context filters
   * @param {number} [request.max_results=10] - Maximum number of results
   * @param {boolean} [request.context_reranking=true] - Enable context-aware reranking
   * @param {number} [request.context_weight=0.3] - Weight of context factors (0.0-1.0)
   * @param {ReadOptions} [options] - Optional read options
   * @returns {Promise<Object>} Contextual search response
   */
  async contextualSearch(request, options) {
    try {
      const transport = this._getReadTransport(options);
      const response = await transport.post('/contextual_search', request);
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
   * @param {Object} request - Multi-collection search request
   * @param {string} request.query - Search query
   * @param {string[]} request.collections - Collections to search
   * @param {number} [request.max_per_collection=5] - Maximum results per collection
   * @param {number} [request.max_total_results=20] - Maximum total results
   * @param {boolean} [request.cross_collection_reranking=true] - Enable cross-collection reranking
   * @param {ReadOptions} [options] - Optional read options
   * @returns {Promise<Object>} Multi-collection search response
   */
  async multiCollectionSearch(request, options) {
    try {
      const transport = this._getReadTransport(options);
      const response = await transport.post('/multi_collection_search', request);
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
   * @param {Object} request - Hybrid search request
   * @param {string} request.collection - Collection name
   * @param {string} request.query - Text query for dense vector search
   * @param {Object} [request.query_sparse] - Optional sparse vector query
   * @param {number} [request.alpha=0.7] - Alpha parameter for blending (0.0-1.0)
   * @param {string} [request.algorithm='rrf'] - Scoring algorithm: 'rrf', 'weighted', or 'alpha'
   * @param {number} [request.dense_k=20] - Number of dense results to retrieve
   * @param {number} [request.sparse_k=20] - Number of sparse results to retrieve
   * @param {number} [request.final_k=10] - Final number of results to return
   * @param {ReadOptions} [options] - Optional read options
   * @returns {Promise<Object>} Hybrid search response
   */
  async hybridSearch(request, options) {
    try {
      validateHybridSearchRequest(request);
      const payload = {
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
      const transport = this._getReadTransport(options);
      const response = await transport.post(
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
   * @param {ReadOptions} [options] - Optional read options
   * @returns {Promise<Object>} Qdrant collection list response
   */
  async qdrantListCollections(options) {
    try {
      const transport = this._getReadTransport(options);
      return await transport.get('/qdrant/collections');
    } catch (error) {
      this.logger.error('Failed to list Qdrant collections', { error });
      throw error;
    }
  }

  /**
   * Get collection information (Qdrant-compatible API).
   * @param {string} name - Collection name
   * @param {ReadOptions} [options] - Optional read options
   * @returns {Promise<Object>} Qdrant collection info response
   */
  async qdrantGetCollection(name, options) {
    try {
      const transport = this._getReadTransport(options);
      return await transport.get(`/qdrant/collections/${name}`);
    } catch (error) {
      this.logger.error('Failed to get Qdrant collection', { name, error });
      throw error;
    }
  }

  /**
   * Create collection (Qdrant-compatible API).
   * @param {string} name - Collection name
   * @param {Object} config - Qdrant collection configuration
   * @returns {Promise<Object>} Qdrant operation result
   */
  async qdrantCreateCollection(name, config) {
    try {
      const transport = this._getWriteTransport();
      return await transport.put(`/qdrant/collections/${name}`, { config });
    } catch (error) {
      this.logger.error('Failed to create Qdrant collection', { name, error });
      throw error;
    }
  }

  /**
   * Upsert points to collection (Qdrant-compatible API).
   * @param {string} collection - Collection name
   * @param {Array} points - List of Qdrant point structures
   * @param {boolean} [wait=false] - Wait for operation completion
   * @returns {Promise<Object>} Qdrant operation result
   */
  async qdrantUpsertPoints(collection, points, wait = false) {
    try {
      const transport = this._getWriteTransport();
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
   * @param {string} collection - Collection name
   * @param {number[]} vector - Query vector
   * @param {number} [limit=10] - Maximum number of results
   * @param {Object} [filter] - Optional Qdrant filter
   * @param {boolean} [withPayload=true] - Include payload in results
   * @param {boolean} [withVector=false] - Include vector in results
   * @param {ReadOptions} [options] - Optional read options
   * @returns {Promise<Object>} Qdrant search response
   */
  async qdrantSearchPoints(collection, vector, limit = 10, filter, withPayload = true, withVector = false, options) {
    try {
      const payload = {
        vector,
        limit,
        with_payload: withPayload,
        with_vector: withVector,
      };
      if (filter) {
        payload.filter = filter;
      }
      const transport = this._getReadTransport(options);
      return await transport.post(`/qdrant/collections/${collection}/points/search`, payload);
    } catch (error) {
      this.logger.error('Failed to search Qdrant points', { collection, error });
      throw error;
    }
  }

  /**
   * Delete points from collection (Qdrant-compatible API).
   * @param {string} collection - Collection name
   * @param {Array<string|number>} pointIds - List of point IDs to delete
   * @param {boolean} [wait=false] - Wait for operation completion
   * @returns {Promise<Object>} Qdrant operation result
   */
  async qdrantDeletePoints(collection, pointIds, wait = false) {
    try {
      const transport = this._getWriteTransport();
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
   * @param {string} collection - Collection name
   * @param {Array<string|number>} pointIds - List of point IDs to retrieve
   * @param {boolean} [withPayload=true] - Include payload in results
   * @param {boolean} [withVector=false] - Include vector in results
   * @param {ReadOptions} [options] - Optional read options
   * @returns {Promise<Object>} Qdrant retrieve response
   */
  async qdrantRetrievePoints(collection, pointIds, withPayload = true, withVector = false, options) {
    try {
      const params = new URLSearchParams({
        ids: pointIds.join(','),
        with_payload: String(withPayload),
        with_vector: String(withVector),
      });
      const transport = this._getReadTransport(options);
      return await transport.get(`/qdrant/collections/${collection}/points?${params.toString()}`);
    } catch (error) {
      this.logger.error('Failed to retrieve Qdrant points', { collection, error });
      throw error;
    }
  }

  /**
   * Count points in collection (Qdrant-compatible API).
   * @param {string} collection - Collection name
   * @param {Object} [filter] - Optional Qdrant filter
   * @param {ReadOptions} [options] - Optional read options
   * @returns {Promise<Object>} Qdrant count response
   */
  async qdrantCountPoints(collection, filter, options) {
    try {
      const payload = {};
      if (filter) {
        payload.filter = filter;
      }
      const transport = this._getReadTransport(options);
      return await transport.post(`/qdrant/collections/${collection}/points/count`, payload);
    } catch (error) {
      this.logger.error('Failed to count Qdrant points', { collection, error });
      throw error;
    }
  }

  // ===== QDRANT ADVANCED FEATURES (1.14.x) =====

  async qdrantListCollectionSnapshots(collection) {
    try {
      return await this.transport.get(`/qdrant/collections/${collection}/snapshots`);
    } catch (error) {
      this.logger.error('Failed to list collection snapshots', { collection, error });
      throw error;
    }
  }

  async qdrantCreateCollectionSnapshot(collection) {
    try {
      return await this.transport.post(`/qdrant/collections/${collection}/snapshots`, {});
    } catch (error) {
      this.logger.error('Failed to create collection snapshot', { collection, error });
      throw error;
    }
  }

  async qdrantDeleteCollectionSnapshot(collection, snapshotName) {
    try {
      return await this.transport.delete(`/qdrant/collections/${collection}/snapshots/${snapshotName}`);
    } catch (error) {
      this.logger.error('Failed to delete collection snapshot', { collection, snapshotName, error });
      throw error;
    }
  }

  async qdrantRecoverCollectionSnapshot(collection, location) {
    try {
      return await this.transport.post(`/qdrant/collections/${collection}/snapshots/recover`, { location });
    } catch (error) {
      this.logger.error('Failed to recover collection snapshot', { collection, location, error });
      throw error;
    }
  }

  async qdrantListAllSnapshots() {
    try {
      return await this.transport.get('/qdrant/snapshots');
    } catch (error) {
      this.logger.error('Failed to list all snapshots', { error });
      throw error;
    }
  }

  async qdrantCreateFullSnapshot() {
    try {
      return await this.transport.post('/qdrant/snapshots', {});
    } catch (error) {
      this.logger.error('Failed to create full snapshot', { error });
      throw error;
    }
  }

  async qdrantListShardKeys(collection) {
    try {
      return await this.transport.get(`/qdrant/collections/${collection}/shards`);
    } catch (error) {
      this.logger.error('Failed to list shard keys', { collection, error });
      throw error;
    }
  }

  async qdrantCreateShardKey(collection, shardKey) {
    try {
      return await this.transport.put(`/qdrant/collections/${collection}/shards`, { shard_key: shardKey });
    } catch (error) {
      this.logger.error('Failed to create shard key', { collection, error });
      throw error;
    }
  }

  async qdrantDeleteShardKey(collection, shardKey) {
    try {
      return await this.transport.post(`/qdrant/collections/${collection}/shards/delete`, { shard_key: shardKey });
    } catch (error) {
      this.logger.error('Failed to delete shard key', { collection, error });
      throw error;
    }
  }

  async qdrantGetClusterStatus() {
    try {
      return await this.transport.get('/qdrant/cluster');
    } catch (error) {
      this.logger.error('Failed to get cluster status', { error });
      throw error;
    }
  }

  async qdrantClusterRecover() {
    try {
      return await this.transport.post('/qdrant/cluster/recover', {});
    } catch (error) {
      this.logger.error('Failed to recover cluster', { error });
      throw error;
    }
  }

  async qdrantRemovePeer(peerId) {
    try {
      return await this.transport.delete(`/qdrant/cluster/peer/${peerId}`);
    } catch (error) {
      this.logger.error('Failed to remove peer', { peerId, error });
      throw error;
    }
  }

  async qdrantListMetadataKeys() {
    try {
      return await this.transport.get('/qdrant/cluster/metadata/keys');
    } catch (error) {
      this.logger.error('Failed to list metadata keys', { error });
      throw error;
    }
  }

  async qdrantGetMetadataKey(key) {
    try {
      return await this.transport.get(`/qdrant/cluster/metadata/keys/${key}`);
    } catch (error) {
      this.logger.error('Failed to get metadata key', { key, error });
      throw error;
    }
  }

  async qdrantUpdateMetadataKey(key, value) {
    try {
      return await this.transport.put(`/qdrant/cluster/metadata/keys/${key}`, { value });
    } catch (error) {
      this.logger.error('Failed to update metadata key', { key, error });
      throw error;
    }
  }

  async qdrantQueryPoints(collection, request) {
    try {
      return await this.transport.post(`/qdrant/collections/${collection}/points/query`, request);
    } catch (error) {
      this.logger.error('Failed to query points', { collection, error });
      throw error;
    }
  }

  async qdrantBatchQueryPoints(collection, request) {
    try {
      return await this.transport.post(`/qdrant/collections/${collection}/points/query/batch`, request);
    } catch (error) {
      this.logger.error('Failed to batch query points', { collection, error });
      throw error;
    }
  }

  async qdrantQueryPointsGroups(collection, request) {
    try {
      return await this.transport.post(`/qdrant/collections/${collection}/points/query/groups`, request);
    } catch (error) {
      this.logger.error('Failed to query points groups', { collection, error });
      throw error;
    }
  }

  async qdrantSearchPointsGroups(collection, request) {
    try {
      return await this.transport.post(`/qdrant/collections/${collection}/points/search/groups`, request);
    } catch (error) {
      this.logger.error('Failed to search points groups', { collection, error });
      throw error;
    }
  }

  async qdrantSearchMatrixPairs(collection, request) {
    try {
      return await this.transport.post(`/qdrant/collections/${collection}/points/search/matrix/pairs`, request);
    } catch (error) {
      this.logger.error('Failed to search matrix pairs', { collection, error });
      throw error;
    }
  }

  async qdrantSearchMatrixOffsets(collection, request) {
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
  async embedText(request) {
    try {
      validateEmbeddingRequest(request);
      const response = await this.transport.post('/embed', request);
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
  getConfig() {
    return { ...this.config };
  }

  /**
   * Update the API key.
   */
  setApiKey(apiKey) {
    this.config.apiKey = apiKey;
    // Reinitialize transport with new API key
    if (this.protocol === 'http') {
      const httpConfig = {
        baseURL: this.config.baseURL,
        ...(this.config.timeout && { timeout: this.config.timeout }),
        ...(this.config.headers && { headers: this.config.headers }),
        ...(this.config.apiKey && { apiKey: this.config.apiKey }),
      };
      this.transport = TransportFactory.create({ protocol: 'http', http: httpConfig });
    } else if (this.protocol === 'umicp' && this.config.umicp) {
      const umicpConfig = {
        host: this.config.umicp.host || 'localhost',
        port: this.config.umicp.port || 15003,
        ...(this.config.apiKey && { apiKey: this.config.apiKey }),
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
   * @param {string} collection - Collection name
   * @param {BatchInsertRequest} request - Batch insert request
   * @returns {Promise<BatchResponse>} Batch operation response
   */
  async batchInsertTexts(collection, request) {
    this.logger.debug('Batch inserting texts', { collection, count: request.texts.length });

    try {
      const response = await this.transport.post(
        `/batch_insert`,
        request.toJSON()
      );

      this.logger.info('Batch insert completed', {
        collection,
        successful: response.successful_operations,
        failed: response.failed_operations,
        duration: response.duration_ms,
      });

      return new BatchResponse(response);
    } catch (error) {
      this.logger.error('Batch insert failed', { collection, error });
      throw error;
    }
  }

  /**
   * Batch search vectors in a collection
   * @param {string} collection - Collection name
   * @param {BatchSearchRequest} request - Batch search request
   * @returns {Promise<BatchSearchResponse>} Batch search response
   */
  async batchSearchVectors(collection, request) {
    this.logger.debug('Batch searching vectors', { collection, queries: request.queries.length });

    try {
      const response = await this.transport.post(
        `/batch_search`,
        request.toJSON()
      );

      this.logger.info('Batch search completed', {
        collection,
        successful: response.successful_queries,
        failed: response.failed_queries,
        duration: response.duration_ms,
      });

      return new BatchSearchResponse(response);
    } catch (error) {
      this.logger.error('Batch search failed', { collection, error });
      throw error;
    }
  }

  /**
   * Batch update vectors in a collection
   * @param {string} collection - Collection name
   * @param {BatchUpdateRequest} request - Batch update request
   * @returns {Promise<BatchResponse>} Batch operation response
   */
  async batchUpdateVectors(collection, request) {
    this.logger.debug('Batch updating vectors', { collection, count: request.updates.length });

    try {
      const response = await this.transport.post(
        `/batch_update`,
        request.toJSON()
      );

      this.logger.info('Batch update completed', {
        collection,
        successful: response.successful_operations,
        failed: response.failed_operations,
        duration: response.duration_ms,
      });

      return new BatchResponse(response);
    } catch (error) {
      this.logger.error('Batch update failed', { collection, error });
      throw error;
    }
  }

  /**
   * Batch delete vectors from a collection
   * @param {string} collection - Collection name
   * @param {BatchDeleteRequest} request - Batch delete request
   * @returns {Promise<BatchResponse>} Batch operation response
   */
  async batchDeleteVectors(collection, request) {
    this.logger.debug('Batch deleting vectors', { collection, count: request.vector_ids.length });

    try {
      const response = await this.transport.post(
        `/batch_delete`,
        request.toJSON()
      );

      this.logger.info('Batch delete completed', {
        collection,
        successful: response.successful_operations,
        failed: response.failed_operations,
        duration: response.duration_ms,
      });

      return new BatchResponse(response);
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
   * @param {SummarizeTextRequest} request - Summarization request
   * @returns {Promise<SummarizeTextResponse>} Summarization response
   */
  /*
  async summarizeText(request) {
    this.logger.debug('Summarizing text', { method: request.method, textLength: request.text.length });

    try {
      const response = await this.transport.post(
        '/summarize/text',
        request.toJSON()
      );

      this.logger.info('Text summarized successfully', {
        summaryId: response.summary_id,
        originalLength: response.original_length,
        summaryLength: response.summary_length,
        compressionRatio: response.compression_ratio,
        method: response.method,
      });

      return new SummarizeTextResponse(response);
    } catch (error) {
      this.logger.error('Text summarization failed', { error });
      throw error;
    }
  }

  /**
   * Summarize context using various methods
   * @param {SummarizeContextRequest} request - Context summarization request
   * @returns {Promise<SummarizeContextResponse>} Context summarization response
   */
  /*
  async summarizeContext(request) {
    this.logger.debug('Summarizing context', { method: request.method, contextLength: request.context.length });

    try {
      const response = await this.transport.post(
        '/summarize/context',
        request.toJSON()
      );

      this.logger.info('Context summarized successfully', {
        summaryId: response.summary_id,
        originalLength: response.original_length,
        summaryLength: response.summary_length,
        compressionRatio: response.compression_ratio,
        method: response.method,
      });

      return new SummarizeContextResponse(response);
    } catch (error) {
      this.logger.error('Context summarization failed', { error });
      throw error;
    }
  }

  /**
   * Get a specific summary by ID
   * @param {string} summaryId - Summary ID
   * @returns {Promise<GetSummaryResponse>} Summary response
   */
  /*
  async getSummary(summaryId) {
    this.logger.debug('Getting summary', { summaryId });

    try {
      const response = await this.transport.get(
        `/summaries/${summaryId}`
      );

      this.logger.info('Summary retrieved successfully', {
        summaryId: response.summary_id,
        method: response.method,
        summaryLength: response.summary_length,
      });

      return new GetSummaryResponse(response);
    } catch (error) {
      this.logger.error('Get summary failed', { summaryId, error });
      throw error;
    }
  }

  /**
   * List summaries with optional filtering
   * @param {Object} query - Query parameters
   * @param {string} [query.method] - Filter by summarization method
   * @param {string} [query.language] - Filter by language
   * @param {number} [query.limit] - Maximum number of summaries to return
   * @param {number} [query.offset] - Offset for pagination
   * @returns {Promise<ListSummariesResponse>} List of summaries response
   */
  /*
  async listSummaries(query = {}) {
    this.logger.debug('Listing summaries', { query });

    try {
      const response = await this.transport.get(
        '/summaries',
        { params: query }
      );

      this.logger.info('Summaries listed successfully', {
        count: response.summaries.length,
        totalCount: response.total_count,
      });

      return new ListSummariesResponse(response);
    } catch (error) {
      this.logger.error('List summaries failed', { error });
      throw error;
    }
  }
  */

  /**
   * Close the client and clean up resources.
   */
  async close() {
    this.logger.info('VectorizerClient closed');
  }

  // =============================================================================
  // FILE UPLOAD OPERATIONS
  // =============================================================================

  /**
   * Upload a file for automatic text extraction, chunking, and indexing.
   *
   * @param {string} collectionName - Target collection name
   * @param {File|Blob} file - File or Blob to upload
   * @param {string} filename - Name of the file
   * @param {Object} [options] - Upload options
   * @param {number} [options.chunkSize] - Chunk size in characters
   * @param {number} [options.chunkOverlap] - Chunk overlap in characters
   * @param {Object} [options.metadata] - Additional metadata to attach to all chunks
   * @param {string} [options.publicKey] - Optional ECC public key for payload encryption (PEM/hex/base64 format)
   * @returns {Promise<Object>} File upload response
   */
  async uploadFile(collectionName, file, filename, options = {}) {
    try {
      validateNonEmptyString(collectionName, 'collectionName');
      validateNonEmptyString(filename, 'filename');

      this.logger.debug('Uploading file', {
        collectionName,
        filename,
        options
      });

      const formData = new FormData();
      formData.append('file', file, filename);
      formData.append('collection_name', collectionName);

      if (options.chunkSize !== undefined) {
        formData.append('chunk_size', String(options.chunkSize));
      }

      if (options.chunkOverlap !== undefined) {
        formData.append('chunk_overlap', String(options.chunkOverlap));
      }

      if (options.metadata !== undefined) {
        formData.append('metadata', JSON.stringify(options.metadata));
      }

      if (options.publicKey !== undefined) {
        formData.append('public_key', options.publicKey);
      }

      const transport = this._getWriteTransport();
      const response = await transport.postFormData('/files/upload', formData);

      validateFileUploadResponse(response);

      this.logger.info('File uploaded successfully', {
        filename,
        chunksCreated: response.chunks_created,
        vectorsCreated: response.vectors_created,
      });

      return response;
    } catch (error) {
      this.logger.error('Failed to upload file', { collectionName, filename, error });
      throw error;
    }
  }

  /**
   * Upload file content directly as a string.
   *
   * @param {string} collectionName - Target collection name
   * @param {string} content - File content as string
   * @param {string} filename - Name of the file (used for language detection)
   * @param {Object} [options] - Upload options
   * @param {number} [options.chunkSize] - Chunk size in characters
   * @param {number} [options.chunkOverlap] - Chunk overlap in characters
   * @param {Object} [options.metadata] - Additional metadata to attach to all chunks
   * @param {string} [options.publicKey] - Optional ECC public key for payload encryption (PEM/hex/base64 format)
   * @returns {Promise<Object>} File upload response
   */
  async uploadFileContent(collectionName, content, filename, options = {}) {
    try {
      validateNonEmptyString(collectionName, 'collectionName');
      validateNonEmptyString(content, 'content');
      validateNonEmptyString(filename, 'filename');

      this.logger.debug('Uploading file content', {
        collectionName,
        filename,
        contentLength: content.length,
        options
      });

      // Create a Blob from the content
      const blob = new Blob([content], { type: 'text/plain' });

      return await this.uploadFile(collectionName, blob, filename, options);
    } catch (error) {
      this.logger.error('Failed to upload file content', { collectionName, filename, error });
      throw error;
    }
  }

  /**
   * Get the server's file upload configuration.
   *
   * @returns {Promise<Object>} File upload configuration
   */
  async getUploadConfig() {
    try {
      this.logger.debug('Getting upload configuration');

      const transport = this._getReadTransport();
      const response = await transport.get('/files/config');

      validateFileUploadConfig(response);

      this.logger.debug('Upload configuration retrieved', {
        maxFileSize: response.max_file_size,
        allowedExtensions: response.allowed_extensions?.length,
      });

      return response;
    } catch (error) {
      this.logger.error('Failed to get upload configuration', { error });
      throw error;
    }
  }

  // =============================================================================
  // DISCOVERY OPERATIONS
  // =============================================================================

  /**
   * Complete discovery pipeline with intelligent search and prompt generation.
   * @param {Object} params - Discovery parameters
   * @param {string} params.query - User question or search query
   * @param {string[]} [params.include_collections] - Collections to include
   * @param {string[]} [params.exclude_collections] - Collections to exclude
   * @param {number} [params.max_bullets] - Maximum evidence bullets
   * @param {number} [params.broad_k] - Broad search results
   * @param {number} [params.focus_k] - Focus search results per collection
   * @returns {Promise<Object>} Discovery response with LLM-ready prompt
   */
  async discover(params) {
    this.logger.debug('Running discovery pipeline', params);
    return this.transport.post('/discover', params);
  }

  /**
   * Pre-filter collections by name patterns.
   * @param {Object} params - Filter parameters
   * @param {string} params.query - Search query for filtering
   * @param {string[]} [params.include] - Include patterns
   * @param {string[]} [params.exclude] - Exclude patterns
   * @returns {Promise<Object>} Filtered collections
   */
  async filterCollections(params) {
    this.logger.debug('Filtering collections', params);
    return this.transport.post('/discovery/filter_collections', params);
  }

  /**
   * Rank collections by relevance.
   * @param {Object} params - Scoring parameters
   * @param {string} params.query - Search query for scoring
   * @param {number} [params.name_match_weight] - Weight for name matching
   * @param {number} [params.term_boost_weight] - Weight for term boost
   * @param {number} [params.signal_boost_weight] - Weight for signals
   * @returns {Promise<Object>} Scored collections
   */
  async scoreCollections(params) {
    this.logger.debug('Scoring collections', params);
    return this.transport.post('/discovery/score_collections', params);
  }

  /**
   * Generate query variations.
   * @param {Object} params - Expansion parameters
   * @param {string} params.query - Original query to expand
   * @param {number} [params.max_expansions] - Maximum expansions
   * @param {boolean} [params.include_definition] - Include definition queries
   * @param {boolean} [params.include_features] - Include features queries
   * @param {boolean} [params.include_architecture] - Include architecture queries
   * @returns {Promise<Object>} Expanded queries
   */
  async expandQueries(params) {
    this.logger.debug('Expanding queries', params);
    return this.transport.post('/discovery/expand_queries', params);
  }

  // =============================================================================
  // FILE OPERATIONS
  // =============================================================================

  /**
   * Retrieve complete file content from a collection.
   * @param {Object} params - File content parameters
   * @param {string} params.collection - Collection name
   * @param {string} params.file_path - Relative file path within collection
   * @param {number} [params.max_size_kb] - Maximum file size in KB
   * @returns {Promise<Object>} File content and metadata
   */
  async getFileContent(params) {
    this.logger.debug('Getting file content', params);
    return this.transport.post('/file/content', params);
  }

  /**
   * List all indexed files in a collection.
   * @param {Object} params - List files parameters
   * @param {string} params.collection - Collection name
   * @param {string[]} [params.filter_by_type] - Filter by file types
   * @param {number} [params.min_chunks] - Minimum number of chunks
   * @param {number} [params.max_results] - Maximum number of results
   * @param {string} [params.sort_by] - Sort order (name, size, chunks, recent)
   * @returns {Promise<Object>} List of files with metadata
   */
  async listFilesInCollection(params) {
    this.logger.debug('Listing files in collection', params);
    return this.transport.post('/file/list', params);
  }

  /**
   * Get extractive or structural summary of an indexed file.
   * @param {Object} params - File summary parameters
   * @param {string} params.collection - Collection name
   * @param {string} params.file_path - Relative file path within collection
   * @param {string} [params.summary_type] - Type of summary (extractive, structural, both)
   * @param {number} [params.max_sentences] - Maximum sentences for extractive summary
   * @returns {Promise<Object>} File summary
   */
  async getFileSummary(params) {
    this.logger.debug('Getting file summary', params);
    return this.transport.post('/file/summary', params);
  }

  /**
   * Retrieve chunks in original file order for progressive reading.
   * @param {Object} params - File chunks parameters
   * @param {string} params.collection - Collection name
   * @param {string} params.file_path - Relative file path within collection
   * @param {number} [params.start_chunk] - Starting chunk index
   * @param {number} [params.limit] - Number of chunks to retrieve
   * @param {boolean} [params.include_context] - Include prev/next chunk hints
   * @returns {Promise<Object>} File chunks
   */
  async getFileChunksOrdered(params) {
    this.logger.debug('Getting file chunks', params);
    return this.transport.post('/file/chunks', params);
  }

  /**
   * Generate hierarchical project structure overview.
   * @param {Object} params - Project outline parameters
   * @param {string} params.collection - Collection name
   * @param {number} [params.max_depth] - Maximum directory depth
   * @param {boolean} [params.include_summaries] - Include file summaries in outline
   * @param {boolean} [params.highlight_key_files] - Highlight important files like README
   * @returns {Promise<Object>} Project outline
   */
  async getProjectOutline(params) {
    this.logger.debug('Getting project outline', params);
    return this.transport.post('/file/outline', params);
  }

  /**
   * Find semantically related files using vector similarity.
   * @param {Object} params - Related files parameters
   * @param {string} params.collection - Collection name
   * @param {string} params.file_path - Reference file path
   * @param {number} [params.limit] - Maximum number of related files
   * @param {number} [params.similarity_threshold] - Minimum similarity score 0.0-1.0
   * @param {boolean} [params.include_reason] - Include explanation of why files are related
   * @returns {Promise<Object>} Related files
   */
  async getRelatedFiles(params) {
    this.logger.debug('Getting related files', params);
    return this.transport.post('/file/related', params);
  }

  /**
   * Semantic search filtered by file type.
   * @param {Object} params - Search by file type parameters
   * @param {string} params.collection - Collection name
   * @param {string} params.query - Search query
   * @param {string[]} params.file_types - File extensions to search
   * @param {number} [params.limit] - Maximum results
   * @param {boolean} [params.return_full_files] - Return complete file content
   * @returns {Promise<Object>} Search results
   */
  async searchByFileType(params) {
    this.logger.debug('Searching by file type', params);
    return this.transport.post('/file/search_by_type', params);
  }

  // =============================================================================
  // GUI-SPECIFIC API METHODS
  // =============================================================================

  /**
   * Get server status (GUI endpoint).
   * @returns {Promise<Object>} Server status including version, uptime, collections count
   */
  async getStatus() {
    this.logger.debug('Getting server status');
    return this.transport.get('/status');
  }

  /**
   * Get recent logs (GUI endpoint).
   * @param {Object} [params] - Log parameters
   * @param {number} [params.lines] - Number of lines to return
   * @param {string} [params.level] - Log level filter
   * @returns {Promise<Object>} Recent log entries
   */
  async getLogs(params = {}) {
    this.logger.debug('Getting logs', params);
    return this.transport.get('/logs', params);
  }

  /**
   * Force save a specific collection (GUI endpoint).
   * @param {string} name - Collection name
   * @returns {Promise<Object>} Save result
   */
  async forceSaveCollection(name) {
    this.logger.debug('Force saving collection', { name });
    return this.transport.post(`/collections/${name}/force-save`, {});
  }

  /**
   * Add a workspace (GUI endpoint).
   * @param {Object} params - Workspace parameters
   * @param {string} params.name - Workspace name
   * @param {string} params.path - Workspace path
   * @param {Array<Object>} params.collections - Collection configurations
   * @returns {Promise<Object>} Add workspace result
   */
  async addWorkspace(params) {
    this.logger.debug('Adding workspace', params);
    return this.transport.post('/workspace/add', params);
  }

  /**
   * Remove a workspace (GUI endpoint).
   * @param {Object} params - Remove parameters
   * @param {string} params.name - Workspace name
   * @returns {Promise<Object>} Remove result
   */
  async removeWorkspace(params) {
    this.logger.debug('Removing workspace', params);
    return this.transport.post('/workspace/remove', params);
  }

  /**
   * List all workspaces (GUI endpoint).
   * @returns {Promise<Object>} List of workspaces
   */
  async listWorkspaces() {
    this.logger.debug('Listing workspaces');
    return this.transport.get('/workspace/list');
  }

  /**
   * Get server configuration (GUI endpoint).
   * @returns {Promise<Object>} Server configuration
   */
  async getServerConfig() {
    this.logger.debug('Getting server configuration');
    return this.transport.get('/config');
  }

  /**
   * Update server configuration (GUI endpoint).
   * @param {Object} config - Configuration object
   * @returns {Promise<Object>} Update result
   */
  async updateConfig(config) {
    this.logger.debug('Updating server configuration', config);
    return this.transport.post('/config', config);
  }

  /**
   * Restart the server (GUI endpoint - admin only).
   * @returns {Promise<Object>} Restart result
   */
  async restartServer() {
    this.logger.debug('Requesting server restart');
    return this.transport.post('/admin/restart', {});
  }

  /**
   * List available backups (GUI endpoint).
   * @returns {Promise<Object>} List of backups with metadata
   */
  async listBackups() {
    this.logger.debug('Listing backups');
    return this.transport.get('/backups/list');
  }

  /**
   * Create a new backup (GUI endpoint).
   * @param {Object} [params] - Backup parameters
   * @param {string} [params.name] - Optional backup name
   * @returns {Promise<Object>} Backup creation result with filename
   */
  async createBackup(params = {}) {
    this.logger.debug('Creating backup', params);
    return this.transport.post('/backups/create', params);
  }

  /**
   * Restore from a backup (GUI endpoint).
   * @param {Object} params - Restore parameters
   * @param {string} params.filename - Backup filename to restore
   * @returns {Promise<Object>} Restore result
   */
  async restoreBackup(params) {
    this.logger.debug('Restoring backup', params);
    return this.transport.post('/backups/restore', params);
  }

  /**
   * Get backup directory path (GUI endpoint).
   * @returns {Promise<Object>} Backup directory path
   */
  async getBackupDirectory() {
    this.logger.debug('Getting backup directory');
    return this.transport.get('/backups/directory');
  }

  // ========== Graph Operations ==========

  /**
   * List all nodes in a collection's graph.
   * @param {string} collection - Collection name
   * @returns {Promise<ListNodesResponse>} List of nodes
   */
  async listGraphNodes(collection) {
    try {
      validateNonEmptyString(collection, 'collection');
      this.logger.debug('Listing graph nodes', { collection });
      const response = await this.transport.get(`/graph/nodes/${collection}`);
      this.logger.debug('Graph nodes listed', { collection, count: response.count });
      return response;
    } catch (error) {
      this.logger.error('Failed to list graph nodes', { collection, error });
      throw error;
    }
  }

  /**
   * Get neighbors of a specific node.
   * @param {string} collection - Collection name
   * @param {string} nodeId - Node ID
   * @returns {Promise<GetNeighborsResponse>} List of neighbors
   */
  async getGraphNeighbors(collection, nodeId) {
    try {
      validateNonEmptyString(collection, 'collection');
      validateNonEmptyString(nodeId, 'nodeId');
      this.logger.debug('Getting graph neighbors', { collection, nodeId });
      const response = await this.transport.get(`/graph/nodes/${collection}/${nodeId}/neighbors`);
      this.logger.debug('Graph neighbors retrieved', { collection, nodeId, count: response.neighbors?.length });
      return response;
    } catch (error) {
      this.logger.error('Failed to get graph neighbors', { collection, nodeId, error });
      throw error;
    }
  }

  /**
   * Find related nodes within N hops.
   * @param {string} collection - Collection name
   * @param {string} nodeId - Node ID
   * @param {FindRelatedRequest} request - Find related request
   * @returns {Promise<FindRelatedResponse>} List of related nodes
   */
  async findRelatedNodes(collection, nodeId, request) {
    try {
      validateNonEmptyString(collection, 'collection');
      validateNonEmptyString(nodeId, 'nodeId');
      validateFindRelatedRequest(request);
      this.logger.debug('Finding related nodes', { collection, nodeId, request });
      const response = await this.transport.post(`/graph/nodes/${collection}/${nodeId}/related`, request);
      this.logger.debug('Related nodes found', { collection, nodeId, count: response.related?.length });
      return response;
    } catch (error) {
      this.logger.error('Failed to find related nodes', { collection, nodeId, request, error });
      throw error;
    }
  }

  /**
   * Find shortest path between two nodes.
   * @param {FindPathRequest} request - Find path request
   * @returns {Promise<FindPathResponse>} Path between nodes
   */
  async findGraphPath(request) {
    try {
      validateFindPathRequest(request);
      this.logger.debug('Finding graph path', { request });
      const response = await this.transport.post('/graph/path', request);
      this.logger.debug('Graph path found', { 
        collection: request.collection, 
        found: response.found, 
        pathLength: response.path?.length 
      });
      return response;
    } catch (error) {
      this.logger.error('Failed to find graph path', { request, error });
      throw error;
    }
  }

  /**
   * Create an explicit edge between two nodes.
   * @param {CreateEdgeRequest} request - Create edge request
   * @returns {Promise<CreateEdgeResponse>} Created edge response
   */
  async createGraphEdge(request) {
    try {
      validateCreateEdgeRequest(request);
      this.logger.debug('Creating graph edge', { request });
      const response = await this.transport.post('/graph/edges', request);
      this.logger.info('Graph edge created', { 
        collection: request.collection, 
        edgeId: response.edge_id,
        success: response.success 
      });
      return response;
    } catch (error) {
      this.logger.error('Failed to create graph edge', { request, error });
      throw error;
    }
  }

  /**
   * Delete an edge by ID.
   * @param {string} edgeId - Edge ID
   * @returns {Promise<void>}
   */
  async deleteGraphEdge(edgeId) {
    try {
      validateNonEmptyString(edgeId, 'edgeId');
      this.logger.debug('Deleting graph edge', { edgeId });
      await this.transport.delete(`/graph/edges/${edgeId}`);
      this.logger.info('Graph edge deleted', { edgeId });
    } catch (error) {
      this.logger.error('Failed to delete graph edge', { edgeId, error });
      throw error;
    }
  }

  /**
   * List all edges in a collection.
   * @param {string} collection - Collection name
   * @returns {Promise<ListEdgesResponse>} List of edges
   */
  async listGraphEdges(collection) {
    try {
      validateNonEmptyString(collection, 'collection');
      this.logger.debug('Listing graph edges', { collection });
      const response = await this.transport.get(`/graph/collections/${collection}/edges`);
      this.logger.debug('Graph edges listed', { collection, count: response.count });
      return response;
    } catch (error) {
      this.logger.error('Failed to list graph edges', { collection, error });
      throw error;
    }
  }

  /**
   * Discover SIMILAR_TO edges for entire collection.
   * @param {string} collection - Collection name
   * @param {DiscoverEdgesRequest} request - Discover edges request
   * @returns {Promise<DiscoverEdgesResponse>} Discovery response
   */
  async discoverGraphEdges(collection, request) {
    try {
      validateNonEmptyString(collection, 'collection');
      validateDiscoverEdgesRequest(request);
      this.logger.debug('Discovering graph edges', { collection, request });
      const response = await this.transport.post(`/graph/discover/${collection}`, request);
      this.logger.info('Graph edges discovered', { 
        collection, 
        edgesCreated: response.edges_created,
        success: response.success 
      });
      return response;
    } catch (error) {
      this.logger.error('Failed to discover graph edges', { collection, request, error });
      throw error;
    }
  }

  /**
   * Discover SIMILAR_TO edges for a specific node.
   * @param {string} collection - Collection name
   * @param {string} nodeId - Node ID
   * @param {DiscoverEdgesRequest} request - Discover edges request
   * @returns {Promise<DiscoverEdgesResponse>} Discovery response
   */
  async discoverGraphEdgesForNode(collection, nodeId, request) {
    try {
      validateNonEmptyString(collection, 'collection');
      validateNonEmptyString(nodeId, 'nodeId');
      validateDiscoverEdgesRequest(request);
      this.logger.debug('Discovering graph edges for node', { collection, nodeId, request });
      const response = await this.transport.post(`/graph/discover/${collection}/${nodeId}`, request);
      this.logger.info('Graph edges discovered for node', { 
        collection, 
        nodeId,
        edgesCreated: response.edges_created,
        success: response.success 
      });
      return response;
    } catch (error) {
      this.logger.error('Failed to discover graph edges for node', { collection, nodeId, request, error });
      throw error;
    }
  }

  /**
   * Get discovery status for a collection.
   * @param {string} collection - Collection name
   * @returns {Promise<DiscoveryStatusResponse>} Discovery status
   */
  async getGraphDiscoveryStatus(collection) {
    try {
      validateNonEmptyString(collection, 'collection');
      this.logger.debug('Getting graph discovery status', { collection });
      const response = await this.transport.get(`/graph/discover/${collection}/status`);
      this.logger.debug('Graph discovery status retrieved', { 
        collection, 
        progress: response.progress_percentage,
        totalEdges: response.total_edges 
      });
      return response;
    } catch (error) {
      this.logger.error('Failed to get graph discovery status', { collection, error });
      throw error;
    }
  }
}
