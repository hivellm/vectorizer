/**
 * Main client class for the Hive Vectorizer SDK.
 * 
 * Provides high-level methods for vector operations, semantic search,
 * and collection management.
 */

import { HttpClient } from './utils/http-client.js';
import { createLogger } from './utils/logger.js';

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
  BatchResponse,
  BatchSearchResponse,
} from './models/batch.js';

import {
  SummarizeTextResponse,
  SummarizeContextResponse,
  GetSummaryResponse,
  ListSummariesResponse,
} from './models/summarization.js';

// Removed unused exception imports - exceptions are handled in http-client

export class VectorizerClient {
  constructor(config = {}) {
    this.config = {
      baseURL: 'http://localhost:15002',
      timeout: 30000,
      headers: {},
      logger: { level: 'info', enabled: true },
      ...config,
    };

    this.logger = createLogger(this.config.logger);

    // Initialize HTTP client
    const httpConfig = {
      baseURL: this.config.baseURL,
      timeout: this.config.timeout,
      apiKey: this.config.apiKey,
      headers: this.config.headers,
    };
    this.httpClient = new HttpClient(httpConfig);

    this.logger.info('VectorizerClient initialized', {
      baseURL: this.config.baseURL,
      hasApiKey: !!this.config.apiKey,
    });
  }

  // ===== HEALTH & STATUS =====

  /**
   * Check if the server is healthy.
   */
  async healthCheck() {
    try {
      const response = await this.httpClient.get('/health');
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
      const response = await this.httpClient.get('/stats');
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
   */
  async listCollections() {
    try {
      const response = await this.httpClient.get('/collections');
      this.logger.debug('Collections listed', { count: response.length });
      return response;
    } catch (error) {
      this.logger.error('Failed to list collections', error);
      throw error;
    }
  }

  /**
   * Get collection information.
   */
  async getCollection(collectionName) {
    try {
      const response = await this.httpClient.get(`/collections/${collectionName}`);
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
      const response = await this.httpClient.post('/collections', request);
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
      const response = await this.httpClient.put(`/collections/${collectionName}`, request);
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
      await this.httpClient.delete(`/collections/${collectionName}`);
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
      const response = await this.httpClient.post(
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
   */
  async getVector(collectionName, vectorId) {
    try {
      const response = await this.httpClient.get(`/collections/${collectionName}/vectors/${vectorId}`);
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
      const response = await this.httpClient.put(
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
      await this.httpClient.delete(`/collections/${collectionName}/vectors/${vectorId}`);
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
      const response = await this.httpClient.post(
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
   */
  async searchVectors(collectionName, request) {
    try {
      validateSearchRequest(request);
      const response = await this.httpClient.post(
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
   */
  async searchText(collectionName, request) {
    try {
      validateTextSearchRequest(request);
      const response = await this.httpClient.post(
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
   * @returns {Promise<Object>} Intelligent search response
   */
  async intelligentSearch(request) {
    try {
      const response = await this.httpClient.post('/intelligent_search', request);
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
   * @returns {Promise<Object>} Semantic search response
   */
  async semanticSearch(request) {
    try {
      const response = await this.httpClient.post('/semantic_search', request);
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
   * @returns {Promise<Object>} Contextual search response
   */
  async contextualSearch(request) {
    try {
      const response = await this.httpClient.post('/contextual_search', request);
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
   * @returns {Promise<Object>} Multi-collection search response
   */
  async multiCollectionSearch(request) {
    try {
      const response = await this.httpClient.post('/multi_collection_search', request);
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

  // ===== EMBEDDING OPERATIONS =====

  /**
   * Generate embeddings for text.
   */
  async embedText(request) {
    try {
      validateEmbeddingRequest(request);
      const response = await this.httpClient.post('/embed', request);
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
    // Reinitialize HTTP client with new API key
    const httpConfig = {
      baseURL: this.config.baseURL,
      timeout: this.config.timeout,
      apiKey: this.config.apiKey,
      headers: this.config.headers,
    };
    this.httpClient = new HttpClient(httpConfig);

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
      const response = await this.httpClient.post(
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
      const response = await this.httpClient.post(
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
      const response = await this.httpClient.post(
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
      const response = await this.httpClient.post(
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
      const response = await this.httpClient.post(
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
      const response = await this.httpClient.post(
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
      const response = await this.httpClient.get(
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
      const response = await this.httpClient.get(
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
}
