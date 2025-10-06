/**
 * LangChain.js Integration for Vectorizer
 * 
 * This module provides a LangChain VectorStore implementation for JavaScript/TypeScript
 * that uses Vectorizer as the backend for vector storage and similarity search.
 */

import { VectorStore } from "@langchain/core/vectorstores";
import { Document } from "@langchain/core/documents";
import { Embeddings } from "@langchain/core/embeddings";

/**
 * Configuration for Vectorizer connection
 */
export interface VectorizerConfig {
  host: string;
  port: number;
  apiKey?: string;
  timeout: number;
  collectionName: string;
  autoCreateCollection: boolean;
  batchSize: number;
  similarityThreshold: number;
}

/**
 * Default configuration
 */
export const DEFAULT_CONFIG: VectorizerConfig = {
  host: "localhost",
  port: 15002,
  timeout: 30000,
  collectionName: "langchain_documents",
  autoCreateCollection: true,
  batchSize: 100,
  similarityThreshold: 0.7,
};

/**
 * Custom error class for Vectorizer operations
 */
export class VectorizerError extends Error {
  constructor(message: string, public cause?: Error) {
    super(message);
    this.name = "VectorizerError";
  }
}

/**
 * Client for communicating with Vectorizer API
 */
export class VectorizerClient {
  private baseUrl: string;
  private headers: Record<string, string>;

  constructor(private config: VectorizerConfig) {
    this.baseUrl = `http://${config.host}:${config.port}`;
    this.headers = {
      "Content-Type": "application/json",
    };

    if (config.apiKey) {
      this.headers["Authorization"] = `Bearer ${config.apiKey}`;
    }
  }

  /**
   * Make HTTP request to Vectorizer API
   */
  private async makeRequest(
    method: string,
    endpoint: string,
    body?: any
  ): Promise<any> {
    const url = `${this.baseUrl}${endpoint}`;
    const options: RequestInit = {
      method,
      headers: this.headers,
      signal: AbortSignal.timeout(this.config.timeout),
    };

    if (body) {
      options.body = JSON.stringify(body);
    }

    try {
      const response = await fetch(url, options);
      
      if (!response.ok) {
        throw new Error(`HTTP ${response.status}: ${response.statusText}`);
      }

      return await response.json();
    } catch (error) {
      if (error instanceof Error) {
        throw new VectorizerError(`API request failed: ${error.message}`, error);
      }
      throw new VectorizerError(`API request failed: ${error}`);
    }
  }

  /**
   * Check if Vectorizer is healthy
   */
  async healthCheck(): Promise<boolean> {
    try {
      await this.makeRequest("GET", "/health");
      return true;
    } catch {
      return false;
    }
  }

  /**
   * List all collections
   */
  async listCollections(): Promise<string[]> {
    const response = await this.makeRequest("GET", "/collections");
    return response.collections?.map((col: any) => col.name) || [];
  }

  /**
   * Create a new collection
   */
  async createCollection(
    name: string,
    dimension: number = 512,
    metric: string = "cosine"
  ): Promise<boolean> {
    try {
      await this.makeRequest("POST", "/collections", {
        name,
        dimension,
        metric,
      });
      return true;
    } catch {
      return false;
    }
  }

  /**
   * Delete a collection
   */
  async deleteCollection(name: string): Promise<boolean> {
    try {
      await this.makeRequest("DELETE", `/collections/${name}`);
      return true;
    } catch {
      return false;
    }
  }

  /**
   * Add texts to the collection
   */
  async addTexts(
    texts: string[],
    metadatas?: Record<string, any>[]
  ): Promise<string[]> {
    const vectorIds: string[] = [];
    
    for (let i = 0; i < texts.length; i++) {
      const data = {
        collection: this.config.collectionName,
        text: texts[i],
        metadata: metadatas?.[i] || {}
      };

      const response = await this.makeRequest("POST", "/insert", data);
      vectorIds.push(response.vector_id || "");
    }

    return vectorIds;
  }

  /**
   * Perform similarity search
   */
  async similaritySearch(
    query: string,
    k: number = 4,
    filter?: Record<string, any>
  ): Promise<any[]> {
    const data = {
      query,
      limit: k,
      filter: filter || {}
    };

    const response = await this.makeRequest(
      "POST",
      `/collections/${this.config.collectionName}/search`,
      data
    );

    return response.results || [];
  }

  /**
   * Perform similarity search with scores
   */
  async similaritySearchWithScore(
    query: string,
    k: number = 4,
    filter?: Record<string, any>
  ): Promise<Array<[any, number]>> {
    const results = await this.similaritySearch(query, k, filter);
    return results.map(result => [result, result.score || 0.0]);
  }

  /**
   * Delete vectors by IDs
   */
  async deleteVectors(ids: string[]): Promise<boolean> {
    try {
      await this.makeRequest(
        "DELETE",
        `/collections/${this.config.collectionName}/vectors`,
        { vector_ids: ids }
      );
      return true;
    } catch {
      return false;
    }
  }

  // ===== INTELLIGENT SEARCH METHODS =====

  /**
   * Perform intelligent search with multi-query expansion
   */
  async intelligentSearch(
    query: string,
    collections?: string[],
    maxResults: number = 10,
    domainExpansion: boolean = true,
    technicalFocus: boolean = true,
    mmrEnabled: boolean = true,
    mmrLambda: number = 0.7
  ): Promise<any[]> {
    const data: any = {
      query,
      max_results: maxResults,
      domain_expansion: domainExpansion,
      technical_focus: technicalFocus,
      mmr_enabled: mmrEnabled,
      mmr_lambda: mmrLambda
    };

    if (collections) {
      data.collections = collections;
    }

    const response = await this.makeRequest("POST", "/intelligent_search", data);
    return response.results || [];
  }

  /**
   * Perform semantic search with advanced reranking
   */
  async semanticSearch(
    query: string,
    collection: string,
    maxResults: number = 10,
    semanticReranking: boolean = true,
    crossEncoderReranking: boolean = false,
    similarityThreshold: number = 0.5
  ): Promise<any[]> {
    const data = {
      query,
      collection,
      max_results: maxResults,
      semantic_reranking: semanticReranking,
      cross_encoder_reranking: crossEncoderReranking,
      similarity_threshold: similarityThreshold
    };

    const response = await this.makeRequest("POST", "/semantic_search", data);
    return response.results || [];
  }

  /**
   * Perform context-aware search with metadata filtering
   */
  async contextualSearch(
    query: string,
    collection: string,
    contextFilters?: Record<string, any>,
    maxResults: number = 10,
    contextReranking: boolean = true,
    contextWeight: number = 0.3
  ): Promise<any[]> {
    const data: any = {
      query,
      collection,
      max_results: maxResults,
      context_reranking: contextReranking,
      context_weight: contextWeight
    };

    if (contextFilters) {
      data.context_filters = contextFilters;
    }

    const response = await this.makeRequest("POST", "/contextual_search", data);
    return response.results || [];
  }

  /**
   * Perform multi-collection search with cross-collection reranking
   */
  async multiCollectionSearch(
    query: string,
    collections: string[],
    maxPerCollection: number = 5,
    maxTotalResults: number = 20,
    crossCollectionReranking: boolean = true
  ): Promise<any[]> {
    const data = {
      query,
      collections,
      max_per_collection: maxPerCollection,
      max_total_results: maxTotalResults,
      cross_collection_reranking: crossCollectionReranking
    };

    const response = await this.makeRequest("POST", "/multi_collection_search", data);
    return response.results || [];
  }
}

/**
 * LangChain VectorStore implementation using Vectorizer as backend
 */
export class VectorizerStore extends VectorStore {
  private client: VectorizerClient;

  constructor(
    config: VectorizerConfig,
    embeddings?: Embeddings
  ) {
    super(embeddings || ({} as Embeddings), {});
    this.client = new VectorizerClient(config);
    
    // Ensure collection exists
    if (config.autoCreateCollection) {
      this.ensureCollectionExists();
    }
  }

  /**
   * Ensure the collection exists, create if it doesn't
   */
  private async ensureCollectionExists(): Promise<void> {
    try {
      const collections = await this.client.listCollections();
      if (!collections.includes(this.client["config"].collectionName)) {
        await this.client.createCollection(
          this.client["config"].collectionName,
          512, // Default for v0.3.0
          "cosine"
        );
      }
    } catch (error) {
      console.warn("Failed to ensure collection exists:", error);
    }
  }

  /**
   * Add texts to the vector store
   */
  async addVectors(
    vectors: number[][],
    documents: Document[],
    metadatas?: Record<string, any>[]
  ): Promise<string[]> {
    const texts = documents.map(doc => doc.pageContent);
    const metadataList = metadatas || documents.map(doc => doc.metadata);
    
    return await this.client.addTexts(texts, metadataList);
  }

  /**
   * Add texts to the vector store (LangChain interface)
   */
  async addTexts(
    texts: string[],
    metadatas?: Record<string, any>[]
  ): Promise<string[]> {
    return await this.client.addTexts(texts, metadatas);
  }

  /**
   * Perform similarity search
   */
  async similaritySearch(
    query: string,
    k: number = 4,
    filter?: Record<string, any>
  ): Promise<Document[]> {
    const results = await this.client.similaritySearch(query, k, filter);
    
    return results.map(result => new Document({
      pageContent: result.content || "",
      metadata: result.metadata || {}
    }));
  }

  /**
   * Perform similarity search with scores
   */
  async similaritySearchWithScore(
    query: string,
    k: number = 4,
    filter?: Record<string, any>
  ): Promise<Array<[Document, number]>> {
    const results = await this.client.similaritySearchWithScore(query, k, filter);
    
    return results.map(([result, score]) => [
      new Document({
        pageContent: result.content || "",
        metadata: result.metadata || {}
      }),
      score
    ]);
  }

  /**
   * Delete vectors by IDs
   */
  async delete(ids: string[]): Promise<boolean> {
    return await this.client.deleteVectors(ids);
  }

  // ===== INTELLIGENT SEARCH METHODS =====

  /**
   * Perform intelligent search with multi-query expansion
   */
  async intelligentSearch(
    query: string,
    collections?: string[],
    maxResults: number = 10,
    domainExpansion: boolean = true,
    technicalFocus: boolean = true,
    mmrEnabled: boolean = true,
    mmrLambda: number = 0.7
  ): Promise<Document[]> {
    const results = await this.client.intelligentSearch(
      query, collections, maxResults, domainExpansion,
      technicalFocus, mmrEnabled, mmrLambda
    );

    return results.map(result => new Document({
      pageContent: result.content || "",
      metadata: result.metadata || {}
    }));
  }

  /**
   * Perform semantic search with advanced reranking
   */
  async semanticSearch(
    query: string,
    collection: string,
    maxResults: number = 10,
    semanticReranking: boolean = true,
    crossEncoderReranking: boolean = false,
    similarityThreshold: number = 0.5
  ): Promise<Document[]> {
    const results = await this.client.semanticSearch(
      query, collection, maxResults, semanticReranking,
      crossEncoderReranking, similarityThreshold
    );

    return results.map(result => new Document({
      pageContent: result.content || "",
      metadata: result.metadata || {}
    }));
  }

  /**
   * Perform context-aware search with metadata filtering
   */
  async contextualSearch(
    query: string,
    collection: string,
    contextFilters?: Record<string, any>,
    maxResults: number = 10,
    contextReranking: boolean = true,
    contextWeight: number = 0.3
  ): Promise<Document[]> {
    const results = await this.client.contextualSearch(
      query, collection, contextFilters, maxResults,
      contextReranking, contextWeight
    );

    return results.map(result => new Document({
      pageContent: result.content || "",
      metadata: result.metadata || {}
    }));
  }

  /**
   * Perform multi-collection search with cross-collection reranking
   */
  async multiCollectionSearch(
    query: string,
    collections: string[],
    maxPerCollection: number = 5,
    maxTotalResults: number = 20,
    crossCollectionReranking: boolean = true
  ): Promise<Document[]> {
    const results = await this.client.multiCollectionSearch(
      query, collections, maxPerCollection,
      maxTotalResults, crossCollectionReranking
    );

    return results.map(result => new Document({
      pageContent: result.content || "",
      metadata: result.metadata || {}
    }));
  }

  /**
   * Create VectorizerStore from texts
   */
  static async fromTexts(
    texts: string[],
    metadatas?: Record<string, any>[],
    embeddings?: Embeddings,
    config?: VectorizerConfig
  ): Promise<VectorizerStore> {
    const finalConfig = config || DEFAULT_CONFIG;
    const store = new VectorizerStore(finalConfig, embeddings);
    
    await store.addTexts(texts, metadatas);
    return store;
  }

  /**
   * Create VectorizerStore from documents
   */
  static async fromDocuments(
    documents: Document[],
    embeddings?: Embeddings,
    config?: VectorizerConfig
  ): Promise<VectorizerStore> {
    const texts = documents.map(doc => doc.pageContent);
    const metadatas = documents.map(doc => doc.metadata);
    
    return await VectorizerStore.fromTexts(texts, metadatas, embeddings, config);
  }
}

/**
 * Convenience function to create VectorizerStore
 */
export async function createVectorizerStore(
  host: string = "localhost",
  port: number = 15002,
  collectionName: string = "langchain_documents",
  apiKey?: string,
  config?: Partial<VectorizerConfig>
): Promise<VectorizerStore> {
  const finalConfig: VectorizerConfig = {
    ...DEFAULT_CONFIG,
    host,
    port,
    collectionName,
    apiKey,
    ...config
  };

  return new VectorizerStore(finalConfig);
}

/**
 * Utility functions
 */
export const VectorizerUtils = {
  /**
   * Validate configuration
   */
  validateConfig(config: VectorizerConfig): void {
    if (!config.host) {
      throw new VectorizerError("Host is required");
    }
    if (config.port <= 0 || config.port > 65535) {
      throw new VectorizerError("Port must be between 1 and 65535");
    }
    if (config.batchSize <= 0) {
      throw new VectorizerError("Batch size must be greater than 0");
    }
    if (config.similarityThreshold < 0 || config.similarityThreshold > 1) {
      throw new VectorizerError("Similarity threshold must be between 0 and 1");
    }
  },

  /**
   * Create default configuration
   */
  createDefaultConfig(overrides?: Partial<VectorizerConfig>): VectorizerConfig {
    return { ...DEFAULT_CONFIG, ...overrides };
  },

  /**
   * Check if Vectorizer is available
   */
  async checkAvailability(config: VectorizerConfig): Promise<boolean> {
    const client = new VectorizerClient(config);
    return await client.healthCheck();
  }
};

// Export types and classes
export {
  VectorizerConfig,
  VectorizerClient,
  VectorizerStore,
  VectorizerError,
  DEFAULT_CONFIG
};

// Default export
export default VectorizerStore;
