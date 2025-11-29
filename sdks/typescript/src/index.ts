/**
 * Hive Vectorizer TypeScript Client SDK
 * 
 * High-performance vector database client for semantic search and embeddings.
 */

// Export all models
export * from './models';

// Export all exceptions
export * from './exceptions';

// Export utilities
export * from './utils';

// Export main client
export * from './client';

// Export types for convenience
export type {
  Vector,
  CreateVectorRequest,
  UpdateVectorRequest,
} from './models/vector';

export type {
  Collection,
  CreateCollectionRequest,
  UpdateCollectionRequest,
  SimilarityMetric,
} from './models/collection';

export type {
  SearchResult,
  SearchResponse,
} from './models/search-result';

export type {
  EmbeddingRequest,
  EmbeddingResponse,
} from './models/embedding-request';

export type {
  SearchRequest,
  TextSearchRequest,
} from './models/search-request';

export type {
  CollectionInfo,
  DatabaseStats,
} from './models/collection-info';

export type {
  HttpClientConfig,
  RequestConfig,
} from './utils/http-client';

export type {
  UMICPClientConfig,
} from './utils/umicp-client';

export type {
  TransportProtocol,
  TransportConfig,
  ITransport,
} from './utils/transport';

export type {
  Logger,
  LogLevel,
  LoggerConfig,
} from './utils/logger';

// Replication/Routing types
export type {
  ReadPreference,
  HostConfig,
  ReadOptions,
} from './models/replication';

