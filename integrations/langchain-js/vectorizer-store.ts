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
