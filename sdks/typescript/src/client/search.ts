/**
 * Search surface: vector + text + intelligent + semantic + contextual +
 * multi-collection + hybrid.
 *
 * The intelligent / semantic / contextual / multi-collection family
 * targets the v3 search pipeline; `searchVectors` and `searchText`
 * remain available for legacy callers.
 */

import { BaseClient } from './_base';
import {
  ContextualSearchRequest,
  ContextualSearchResponse,
  HybridSearchRequest,
  HybridSearchResponse,
  IntelligentSearchRequest,
  IntelligentSearchResponse,
  MultiCollectionSearchRequest,
  MultiCollectionSearchResponse,
  ReadOptions,
  SearchRequest,
  SearchResponse,
  SemanticSearchRequest,
  SemanticSearchResponse,
  TextSearchRequest,
  validateHybridSearchRequest,
  validateSearchRequest,
  validateSearchResponse,
  validateTextSearchRequest,
} from '../models';

export class SearchClient extends BaseClient {
  /** Vector search via the canonical `/search` endpoint. */
  public async searchVectors(
    collectionName: string,
    request: SearchRequest,
    options?: ReadOptions,
  ): Promise<SearchResponse> {
    try {
      validateSearchRequest(request);
      const transport = this.getReadTransport(options);
      const apiRequest = {
        vector: request.query_vector,
        limit: request.limit,
        threshold: request.threshold,
        include_metadata: request.include_metadata,
        filter: request.filter,
      };
      const response = await transport.post<SearchResponse>('/search', {
        ...apiRequest,
        collection: collectionName,
      });
      const normalizedResponse: SearchResponse = {
        ...response,
        results: response.results || [],
      };
      this.logger.debug('Vector search completed', {
        collectionName,
        resultCount: normalizedResponse.results.length,
      });
      return normalizedResponse;
    } catch (error) {
      this.logger.error('Failed to search vectors', { collectionName, request, error });
      throw error;
    }
  }

  /** Search by text — server runs the embedding pipeline. */
  public async searchText(
    collectionName: string,
    request: TextSearchRequest,
    options?: ReadOptions,
  ): Promise<SearchResponse> {
    try {
      validateTextSearchRequest(request);
      const transport = this.getReadTransport(options);
      const response = await transport.post<SearchResponse>(
        `/collections/${collectionName}/search/text`,
        request,
      );
      validateSearchResponse(response);
      this.logger.debug('Text search completed', {
        collectionName,
        query: request.query,
        resultCount: response.results.length,
      });
      return response;
    } catch (error) {
      this.logger.error('Failed to search text', { collectionName, request, error });
      throw error;
    }
  }

  /** Multi-query expansion + semantic reranking. */
  public async intelligentSearch(
    request: IntelligentSearchRequest,
    options?: ReadOptions,
  ): Promise<IntelligentSearchResponse> {
    try {
      const transport = this.getReadTransport(options);
      const response = await transport.post<IntelligentSearchResponse>(
        '/intelligent_search',
        request,
      );
      this.logger.debug('Intelligent search completed', {
        query: request.query,
        resultCount: response.results?.length || 0,
        collections: request.collections,
      });
      return response;
    } catch (error) {
      this.logger.error('Failed to perform intelligent search', { request, error });
      throw error;
    }
  }

  /** Semantic search with reranking + similarity thresholds. */
  public async semanticSearch(
    request: SemanticSearchRequest,
    options?: ReadOptions,
  ): Promise<SemanticSearchResponse> {
    try {
      const transport = this.getReadTransport(options);
      const response = await transport.post<SemanticSearchResponse>(
        '/semantic_search',
        request,
      );
      this.logger.debug('Semantic search completed', {
        query: request.query,
        collection: request.collection,
        resultCount: response.results?.length || 0,
      });
      return response;
    } catch (error) {
      this.logger.error('Failed to perform semantic search', { request, error });
      throw error;
    }
  }

  /** Context-aware search with metadata filters and contextual reranking. */
  public async contextualSearch(
    request: ContextualSearchRequest,
    options?: ReadOptions,
  ): Promise<ContextualSearchResponse> {
    try {
      const transport = this.getReadTransport(options);
      const response = await transport.post<ContextualSearchResponse>(
        '/contextual_search',
        request,
      );
      this.logger.debug('Contextual search completed', {
        query: request.query,
        collection: request.collection,
        resultCount: response.results?.length || 0,
        contextFilters: request.context_filters,
      });
      return response;
    } catch (error) {
      this.logger.error('Failed to perform contextual search', { request, error });
      throw error;
    }
  }

  /** Cross-collection search with rerank + aggregation. */
  public async multiCollectionSearch(
    request: MultiCollectionSearchRequest,
    options?: ReadOptions,
  ): Promise<MultiCollectionSearchResponse> {
    try {
      const transport = this.getReadTransport(options);
      const response = await transport.post<MultiCollectionSearchResponse>(
        '/multi_collection_search',
        request,
      );
      this.logger.debug('Multi-collection search completed', {
        query: request.query,
        collections: request.collections,
        resultCount: response.results?.length || 0,
      });
      return response;
    } catch (error) {
      this.logger.error('Failed to perform multi-collection search', { request, error });
      throw error;
    }
  }

  /** Dense + sparse hybrid search (RRF / weighted). */
  public async hybridSearch(
    request: HybridSearchRequest,
    options?: ReadOptions,
  ): Promise<HybridSearchResponse> {
    try {
      validateHybridSearchRequest(request);
      const transport = this.getReadTransport(options);
      const payload: Record<string, unknown> = {
        query: request.query,
        alpha: request.alpha ?? 0.7,
        algorithm: request.algorithm ?? 'rrf',
        dense_k: request.dense_k ?? 20,
        sparse_k: request.sparse_k ?? 20,
        final_k: request.final_k ?? 10,
      };
      if (request.query_sparse) {
        payload['query_sparse'] = {
          indices: request.query_sparse.indices,
          values: request.query_sparse.values,
        };
      }
      const response = await transport.post<HybridSearchResponse>(
        `/collections/${request.collection}/hybrid_search`,
        payload,
      );
      this.logger.debug('Hybrid search completed', {
        query: request.query,
        collection: request.collection,
        algorithm: request.algorithm,
        resultCount: response.results?.length || 0,
      });
      return response;
    } catch (error) {
      this.logger.error('Failed to perform hybrid search', { request, error });
      throw error;
    }
  }
}
