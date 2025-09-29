/**
 * Main client class for the Hive Vectorizer SDK.
 * 
 * Provides high-level methods for vector operations, semantic search,
 * and collection management.
 */

import { HttpClient, HttpClientConfig } from './utils/http-client';
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
  // Summarization models
  SummarizeTextRequest,
  SummarizeTextResponse,
  SummarizeContextRequest,
  SummarizeContextResponse,
  GetSummaryResponse,
  ListSummariesResponse,
  ListSummariesQuery,
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
  /** Base URL for the Vectorizer API */
  baseURL?: string;
  /** API key for authentication */
  apiKey?: string;
  /** Request timeout in milliseconds */
  timeout?: number;
  /** Custom headers for requests */
  headers?: Record<string, string>;
  /** Logger configuration */
  logger?: LoggerConfig;
}

export class VectorizerClient {
  private httpClient: HttpClient;
  private logger: Logger;
  private config: Required<Omit<VectorizerClientConfig, 'apiKey'>> & { apiKey?: string };

  constructor(config: VectorizerClientConfig = {}) {
    this.config = {
      baseURL: 'http://localhost:15001',
      timeout: 30000,
      headers: {},
      logger: { level: 'info', enabled: true },
      ...config,
    };

    this.logger = createLogger(this.config.logger);

    // Initialize HTTP client
    const httpConfig: HttpClientConfig = {
      baseURL: this.config.baseURL,
      timeout: this.config.timeout,
      headers: this.config.headers,
      ...(this.config.apiKey && { apiKey: this.config.apiKey }),
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
  public async healthCheck(): Promise<{ status: string; timestamp: string }> {
    try {
      const response = await this.httpClient.get<{ status: string; timestamp: string }>('/api/v1/health');
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
      const response = await this.httpClient.get<DatabaseStats>('/api/v1/stats');
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
  public async listCollections(): Promise<Collection[]> {
    try {
      const response = await this.httpClient.get<Collection[]>('/api/v1/collections');
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
  public async getCollection(collectionName: string): Promise<CollectionInfo> {
    try {
      const response = await this.httpClient.get<CollectionInfo>(`/api/v1/collections/${collectionName}`);
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
  public async createCollection(request: CreateCollectionRequest): Promise<Collection> {
    try {
      validateCreateCollectionRequest(request);
      const response = await this.httpClient.post<Collection>('/api/v1/collections', request);
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
  public async updateCollection(collectionName: string, request: UpdateCollectionRequest): Promise<Collection> {
    try {
      const response = await this.httpClient.put<Collection>(`/api/v1/collections/${collectionName}`, request);
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
  public async deleteCollection(collectionName: string): Promise<void> {
    try {
      await this.httpClient.delete(`/api/v1/collections/${collectionName}`);
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
  public async insertVectors(collectionName: string, vectors: CreateVectorRequest[]): Promise<{ inserted: number }> {
    try {
      vectors.forEach(validateCreateVectorRequest);
      const response = await this.httpClient.post<{ inserted: number }>(
        `/api/v1/collections/${collectionName}/vectors`,
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
  public async getVector(collectionName: string, vectorId: string): Promise<Vector> {
    try {
      const response = await this.httpClient.get<Vector>(`/api/v1/collections/${collectionName}/vectors/${vectorId}`);
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
  public async updateVector(collectionName: string, vectorId: string, request: UpdateVectorRequest): Promise<Vector> {
    try {
      const response = await this.httpClient.put<Vector>(
        `/api/v1/collections/${collectionName}/vectors/${vectorId}`,
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
  public async deleteVector(collectionName: string, vectorId: string): Promise<void> {
    try {
      await this.httpClient.delete(`/api/v1/collections/${collectionName}/vectors/${vectorId}`);
      this.logger.info('Vector deleted', { collectionName, vectorId });
    } catch (error) {
      this.logger.error('Failed to delete vector', { collectionName, vectorId, error });
      throw error;
    }
  }

  /**
   * Delete multiple vectors.
   */
  public async deleteVectors(collectionName: string, vectorIds: string[]): Promise<{ deleted: number }> {
    try {
      const response = await this.httpClient.post<{ deleted: number }>(
        `/api/v1/collections/${collectionName}/vectors/delete`,
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
  public async searchVectors(collectionName: string, request: SearchRequest): Promise<SearchResponse> {
    try {
      validateSearchRequest(request);
      const response = await this.httpClient.post<SearchResponse>(
        `/api/v1/collections/${collectionName}/search`,
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
  public async searchText(collectionName: string, request: TextSearchRequest): Promise<SearchResponse> {
    try {
      validateTextSearchRequest(request);
      const response = await this.httpClient.post<SearchResponse>(
        `/api/v1/collections/${collectionName}/search/text`,
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

  // ===== EMBEDDING OPERATIONS =====

  /**
   * Generate embeddings for text.
   */
  public async embedText(request: EmbeddingRequest): Promise<EmbeddingResponse> {
    try {
      validateEmbeddingRequest(request);
      const response = await this.httpClient.post<EmbeddingResponse>('/api/v1/embed', request);
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
    // Reinitialize HTTP client with new API key
    const httpConfig: HttpClientConfig = {
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
   */
  public async batchInsertTexts(
    collection: string,
    request: BatchInsertRequest
  ): Promise<BatchResponse> {
    this.logger.debug('Batch inserting texts', { collection, count: request.texts.length });

    try {
      const response = await this.httpClient.post<BatchResponse>(
        `/api/v1/collections/${collection}/batch/insert`,
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
      const response = await this.httpClient.post<BatchSearchResponse>(
        `/api/v1/collections/${collection}/batch/search`,
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
      const response = await this.httpClient.post<BatchResponse>(
        `/api/v1/collections/${collection}/batch/update`,
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
      const response = await this.httpClient.post<BatchResponse>(
        `/api/v1/collections/${collection}/batch/delete`,
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

  /**
   * Summarize text using various methods
   */
  public async summarizeText(request: SummarizeTextRequest): Promise<SummarizeTextResponse> {
    this.logger.debug('Summarizing text', { method: request.method, textLength: request.text.length });

    try {
      const response = await this.httpClient.post<SummarizeTextResponse>(
        '/api/v1/summarize/text',
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
  public async summarizeContext(request: SummarizeContextRequest): Promise<SummarizeContextResponse> {
    this.logger.debug('Summarizing context', { method: request.method, contextLength: request.context.length });

    try {
      const response = await this.httpClient.post<SummarizeContextResponse>(
        '/api/v1/summarize/context',
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
  public async getSummary(summaryId: string): Promise<GetSummaryResponse> {
    this.logger.debug('Getting summary', { summaryId });

    try {
      const response = await this.httpClient.get<GetSummaryResponse>(
        `/api/v1/summaries/${summaryId}`
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
  public async listSummaries(query?: ListSummariesQuery): Promise<ListSummariesResponse> {
    this.logger.debug('Listing summaries', { query });

    try {
      const response = await this.httpClient.get<ListSummariesResponse>(
        '/api/v1/summaries',
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

  /**
   * Close the client and clean up resources.
   */
  public async close(): Promise<void> {
    this.logger.info('VectorizerClient closed');
  }
}
